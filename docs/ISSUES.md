# 架构评审 Issue 清单

- 记录时间：2026-10-29
- 来源：项目架构评审
- 状态：ARCH-001 ~ ARCH-004 已在本地完成第一轮修复；ARCH-005 已完成第一批整改；其余为 `Open`。修复均未提交。
- 优先级：`P0` 正确性/安全，`P1` 可运维性/扩展性，`P2` 长期架构收敛

| ID | 优先级 | 标题 | 领域 |
|---|---|---|---|
| ARCH-001 | P0 | 全局 StorageFacade 写锁覆盖网络 IO | 并发/性能 |
| ARCH-002 | P0 | 存储适配器按 CloudType 缓存而非账号 | 并发/多账号 |
| ARCH-003 | P0 | VFS 同步边界反复创建 Tokio runtime | 运行时 |
| ARCH-004 | P0 | 缺少唯一约束、事务和按文件串行化 | 一致性 |
| ARCH-005 | P0 | 安全边界不足 | 安全 |
| ARCH-006 | P1 | 配置分散且存在硬编码默认值 | 配置 |
| ARCH-007 | P1 | 本地块缓存无上限、无淘汰 | 容量 |
| ARCH-008 | P1 | DAV/NFS/FUSE 协议实现不完整且大量 unwrap | 协议 |
| ARCH-009 | P1 | API 响应契约与错误语义不一致 | API |
| ARCH-010 | P1 | 可观测性与关键路径测试不足 | 运维/质量 |
| ARCH-011 | P1 | 构建发布不可复现且产物不一致 | 构建 |
| ARCH-012 | P1 | storage 反向依赖 service，VFS 职责过载 | 分层 |
| ARCH-013 | P2 | 后台任务队列与资源模型需要演进 | 任务/容量 |

---

## ARCH-001 [P0] 全局 StorageFacade 写锁覆盖网络 IO

**状态**：已修复（第一轮，未提交）。`STORAGE_FACADE` 不再持有全局写锁；facade 只短暂访问注册表，网络 IO 在账号级 provider 锁内执行。

**证据**

- `lib/storage/src/lib.rs:20`：全局 `STORAGE_FACADE: Lazy<Arc<RwLock<StorageFacade>>>`。
- `lib/storage/src/storage_facade.rs:122`、`:212`：上传和读取都通过 facade。
- `lib/cloud/src/task/scan.rs`：调用 `STORAGE_FACADE.write().await.upload_content(...)`。
- `lib/cloud/src/fs/vfs.rs`：`read_content_from_cloud` 使用 `STORAGE_FACADE.write().await.content(...)`。
- `lib/cloud/src/task/refresh_token.rs`：持写锁遍历所有账号刷新 token。

**影响**

- 所有云厂商、所有账号的上传/下载/刷新 token 被全局串行化。
- 单个慢请求会阻塞整个存储层，形成 head-of-line blocking。
- `scan` 的 16 并发信号量在全局锁面前基本失效。

**建议**

- `StorageFacade` 改为只读注册表，不承载可变状态。
- 按 `cloud_meta_id` 或 provider 实例加独立锁/信号量/限流器。
- 绝不在全局锁内执行网络 IO；锁只保护内存注册表。

**验收标准**

- 网络请求路径中不存在全局 `STORAGE_FACADE.write()`。
- 两个不同账号/厂商可以并发上传，互不阻塞。
- 增加并发集成测试，验证慢请求不会阻塞其他账号。

---

## ARCH-002 [P0] 存储适配器按 CloudType 缓存而非账号

**状态**：已修复（第一轮，未提交）。内置 provider 按 `cloud_meta_id` 缓存实例和加锁；插件 provider 仍按插件实例复用。

**证据**

- `lib/storage/src/storage_facade.rs:35`：`holder: HashMap<CloudType, Box<Arc<Mutex<dyn Storage + Send>>>>`。
- `lib/storage/src/storage_facade.rs` 的 `get_cloud`：按 `CloudType` 复用同一个适配器实例和互斥锁。

**影响**

- 同一云厂商的多个账号共享同一个 `Mutex`，无法做账号级并发隔离。
- 一个账号的慢请求或 token 刷新会阻塞同厂商的其他账号。
- 无法按账号实现独立限流、熔断和健康状态。

**建议**

- 以 `cloud_meta_id` 作为实例和锁的 key。
- 适配器尽量保持无状态，账号级 token/客户端由 `CloudMeta` 注入。
- 如需共享客户端，按账号维护独立的限流器和缓存。

**验收标准**

- 同厂商的两个账号可以并行上传和读取。
- 单个账号异常不会阻塞其他账号。
- 增加多账号并发测试。

---

## ARCH-003 [P0] VFS 同步边界反复创建 Tokio runtime

**状态**：已修复（第一轮，未提交）。FUSE 使用统一的 `fs::runtime` bridge；VFS 核心方法改为 async，NFS 直接使用 async API。

**证据**

- `lib/cloud/src/fs/vfs.rs:139` 附近：`del_file` 创建 runtime 并 `block_on`。
- `lib/cloud/src/fs/vfs.rs:275` 附近：`lookup` 创建 runtime 并 `block_on`。
- `lib/cloud/src/fs/vfs.rs:293` 附近：`file_info` 创建 runtime 并 `block_on`。
- `lib/cloud/src/fs/vfs.rs:350` 附近：`setattr` 创建 runtime 并 `block_on`。

**影响**

- 每次同步调用都创建多线程 runtime，开销高。
- 如果这些同步方法被 Actix/DAV 等异步路径调用，存在 nested runtime 风险。
- 阻塞线程不可控，容易放大尾延迟。

**建议**

- VFS 核心只保留 async API。
- FUSE 同步回调通过专用桥接层调用 async VFS，例如统一 `Handle` + `spawn_blocking`。
- 消除 VFS 内部所有 `Runtime::new` / `block_on`。
- 明确 Actix、NFS、DAV、FUSE 各自使用的 runtime 边界。

**验收标准**

- `lib/cloud/src/fs/vfs.rs` 不再创建 Tokio runtime。
- FUSE、DAV、NFS 三条路径都有测试覆盖。
- 异步路径中不会出现 nested runtime panic。

---

## ARCH-004 [P0] 缺少唯一约束、事务和按文件串行化

**状态**：已修复（第一轮，未提交）。新增 `0.0.3` 迁移和 partial unique index；块元数据/副本创建事务化；VFS 写入引入 per-inode async lock。

**证据**

- `lib/service/src/migrate/version/0.0.1.sql:31`：`file_meta` 没有 `(parent_id, name, deleted=0)` 唯一约束。
- `lib/service/src/migrate/version/0.0.1.sql:47`：`file_block_meta` 没有 `(file_meta_id, block_index)` 唯一约束。
- `lib/service/src/migrate/version/0.0.1.sql:62`：`cloud_file_block` 没有 `(file_block_id, cloud_meta_id)` 唯一约束。
- `lib/service/src/meta/file/file_block_meta.rs` 的 `save_file_block_meta`：先 insert，再按条件 select，再插入副本，未使用事务。
- NFS 使用 `RwLock<VirtualFileSystem>` 串行化，但 FUSE/DAV 没有同等保护。
- `delete_file_meta` 递归删除目录，未使用事务。

**影响**

- 并发创建同名文件可能产生重复记录。
- 并发写同一文件块可能产生重复块元数据或副本记录。
- 多步操作中断会留下孤儿块、孤儿副本或半删除目录。
- NFS/FUSE/DAV 并发语义不一致，容易产生难以复现的数据损坏。

**建议**

- 增加 partial unique index，例如：
  - `file_meta(parent_id, name) where deleted = 0`
  - `file_block_meta(file_meta_id, block_index) where deleted = 0`
  - `cloud_file_block(file_block_id, cloud_meta_id) where deleted = 0`
- 将“文件创建 + 块元数据 + 副本行”放入同一事务。
- 引入按 inode 的异步锁，统一 FUSE/DAV/NFS 的并发语义。
- 为上传和清理流程增加幂等键或状态版本号。

**验收标准**

- 数据库层能阻止重复文件名和重复块记录。
- 关键多步操作具备事务边界。
- 并发写同一文件的集成测试通过，不出现数据损坏或重复记录。

---

## ARCH-005 [P0] 安全边界不足

**状态**：部分修复（第一轮，未提交）。NFS 默认改为 `127.0.0.1:11111` 并支持 `NFS_BIND_ADDR`；OAuth 回调支持 `PUBLIC_BASE_URL`。token 加密、Basic Auth 限流和插件信任模型仍待处理。

**证据**

- `lib/cloud/src/bin/cloud.rs:42`：NFS 固定监听 `0.0.0.0:11111`，无认证。
- `lib/cloud/src/web/mod.rs:69`：`/api` 和 `/dav` 使用 Basic Auth，默认无 TLS。
- `cloud_meta.auth` 中的 OAuth token 以明文存入 SQLite。
- `lib/plugin-manager/src/lib.rs`：原生插件通过 `Library::new` 在进程内加载，无签名、权限或 ABI 版本校验。
- `lib/cloud/src/web/cloud/authorize.rs`：OAuth 回调地址基于 `Host` 头拼接。

**影响**

- NFS 暴露到公网时等于无认证文件系统。
- Basic Auth 和 token 可能被窃听。
- 数据库泄露会直接泄露云盘凭据。
- 恶意或版本不匹配的原生插件可以执行任意代码、导致进程崩溃。

**建议**

- NFS 默认只监听 `127.0.0.1`，公网监听必须显式配置并文档化风险。
- 明确要求反向代理终止 TLS；补充 HTTPS 部署说明。
- OAuth token 使用应用级密钥加密后落库。
- 为 Basic Auth 增加失败限流/锁定策略。
- 原生插件建立信任模型：允许列表、签名、ABI 版本声明，至少明确“仅加载可信插件”。
- OAuth 回调地址改为显式配置，不信任任意 `Host` 头。

**验收标准**

- 默认配置下 NFS 不暴露到公网。
- 敏感 token 不以明文存储。
- 插件加载失败或 ABI 不匹配时安全降级。
- 安全部署说明覆盖 TLS、防火墙、插件来源。

---

## ARCH-006 [P1] 配置分散且存在硬编码默认值

**证据**

- `.env`：端口、CORS、插件目录等。
- `lib/persistence/application.yml`：数据库 URL，通过 `include_str!` 编译进二进制。
- 数据库 `config` 表：`MOUNT_PATH`、`TEMP_PATH`、`MAX_NUMBER_OF_SHARD`、`CLOUD_FILE_ROOT`。
- 硬编码默认值：
  - `lib/cloud/src/fs/vfs.rs:21`：`/var/lib/storage/temp`
  - `lib/cloud/src/bin/cloud.rs`：NFS `11111`
  - `lib/storage/src/storage.rs`：`/app/share-desk`
  - `lib/cloud/src/web/cloud/authorize.rs`：`pan.calm0406.com:8080`

**影响**

- 部署迁移时容易只改一处，导致运行行为不一致。
- 不同环境（macOS、Pi、Docker）默认路径不同，启动失败难排查。
- 配置优先级不明确，排障成本高。

**建议**

- 定义单一配置 schema，明确 `.env`、文件、数据库三层优先级。
- 启动时统一校验必填项并输出有效配置摘要。
- 移除硬编码业务路径和域名，改为显式配置。
- 数据库只保留运行期可动态修改的配置。

**验收标准**

- 新增一个环境不需要修改代码中的硬编码路径。
- 启动日志能看出最终生效的关键配置。
- 配置缺失时有明确错误，而不是运行到一半才失败。

---

## ARCH-007 [P1] 本地块缓存无上限、无淘汰

**证据**

- `lib/cloud/src/fs/vfs.rs`：所有上传和读取都落到 `TEMP_PATH` 下的块文件。
- 清理逻辑只处理已删除文件的块，没有容量上限、LRU/TTL 或磁盘水位保护。
- 没有缓存命中率、占用空间等指标。

**影响**

- 长期运行缓存会持续增长，最终可能占满磁盘。
- 磁盘满会导致写入失败、上传失败和数据库异常。
- 无法判断缓存是否有效，也无法做容量规划。

**建议**

- 增加缓存配额和磁盘水位。
- 实现 LRU/TTL 淘汰，但必须排除“尚未上传成功”的块。
- 缓存目录与数据库状态联动，避免淘汰正在使用的块。
- 暴露缓存大小、命中率、淘汰数等指标。

**验收标准**

- 缓存达到配额后能自动淘汰且不影响未完成上传。
- 磁盘水位告警可用。
- 有缓存命中率和占用空间指标。

---

## ARCH-008 [P1] DAV/NFS/FUSE 协议实现不完整且大量 unwrap

**证据**

- `lib/cloud/src/fs/dav/meta.rs:102`：`write_buf` 仍是 `todo!()`。
- `lib/cloud/src/fs/dav/meta.rs:128`：`read_bytes` 按请求长度推进 `pos`，而不是实际返回长度。
- `lib/cloud/src/fs/dav/cloud_dav_filesystem.rs`：`read_dir`、`open` 等路径存在 `unwrap`。
- `lib/cloud/src/fs/nfs/cloud_nfs_filesystem.rs:123`：`create_exclusive` 为 `todo!()`。
- NFS `symlink`、`readlink` 也是 `todo!()`。
- FUSE `setattr` 只处理 size，忽略 mode/uid/gid/时间。
- FUSE `readdir` 没有分页处理。
- NFS 返回的 uid/gid 固定为 1000，与元数据不一致。

**影响**

- 某些 DAV/NFS 客户端会触发 panic 或功能失败。
- 协议语义不一致，用户在不同挂载方式下看到不同行为。
- `unwrap` 会把客户端输入或临时故障放大为进程崩溃。

**建议**

- 协议适配层禁止 `todo!()` 和业务路径 `unwrap`。
- 统一返回 typed protocol error。
- 补齐 DAV/NFS/FUSE 的常用操作和属性语义。
- 增加真实客户端契约测试，例如 curl、rclone、davfs2、系统 NFS 客户端。

**验收标准**

- 协议处理路径无 `todo!()`，无未处理 panic。
- 常用客户端可完成创建、读写、重命名、删除、列目录。
- FUSE/DAV/NFS 的属性行为一致。

---

## ARCH-009 [P1] API 响应契约与错误语义不一致

**证据**

- `lib/cloud/src/web/common.rs`：`WebResult` 成功和失败都可能返回 `code: 0`。
- `/api/storage/meta` 的列表和新增在数据库错误时返回 `WebResult::empty()`。
- `lib/api/src/error.rs` 的 `ResponseError` 没有覆盖 `status_code`，错误基本都成为 HTTP 500。
- 错误响应体是 `ErrorInfo` enum JSON，与 `{message, code, data}` 不一致。

**影响**

- 前端无法区分“成功但无数据”和“失败”。
- 外部调用方无法按 HTTP 状态码或统一错误码处理。
- API 契约不稳定，后续兼容成本高。

**建议**

- 统一响应 envelope：`{message, code, data}`。
- 明确 HTTP 状态码映射：400/401/403/404/409/500。
- 业务失败返回非 0 code，而不是 `empty()`。
- 补充 API 契约文档和集成测试。

**验收标准**

- 所有 API 成功/失败响应结构一致。
- 错误具备正确 HTTP 状态码和稳定业务错误码。
- 前端不再依赖 `code: 0 + data: null` 判断失败。

---

## ARCH-010 [P1] 可观测性与关键路径测试不足

**证据**

- 目前主要依赖 `log4rs` 文本日志。
- 没有 health/readiness、metrics、trace。
- 测试仅有 `persistence` 少量用例。
- VFS、上传状态机、DAV/NFS 协议适配缺少集成测试。

**影响**

- 线上问题只能靠日志猜测。
- 并发、重试、清理、缓存等关键路径回归风险高。
- 重构时缺少安全网。

**建议**

- 增加 `/health` 和 `/ready`。
- 引入结构化日志和 trace id。
- 暴露关键指标：上传队列长度、成功率、缓存命中率、锁等待时间、磁盘占用。
- 为 VFS、上传状态机、清理任务、DAV/NFS 增加集成测试和故障注入测试。

**验收标准**

- 能通过指标判断系统是否健康。
- 关键路径有自动化测试覆盖。
- 故障注入下能观察到明确错误和恢复行为。

---

## ARCH-011 [P1] 构建发布不可复现且产物不一致

**证据**

- `.gitignore:14` 忽略 `Cargo.lock`（二进制项目通常应提交）。
- `Makefile` 的 `build-image` 使用 `cargo build`，随后复制 `target/debug/cloud` 到 `target/release/`。
- `Dockerfile` 使用 release 构建，与 Makefile 语义不一致。
- `lib/cloud-ui/build.rs` 在 Cargo 构建过程中执行 `npm install` / `npm run build`，后端构建隐式依赖 Node/npm。

**影响**

- 不同环境构建出的依赖版本可能不同。
- 发布产物可能是 debug 二进制，性能和体积不符合预期。
- 后端 CI 容易被前端工具链阻塞。

**建议**

- 提交 `Cargo.lock`，或明确说明这是纯库仓库并接受非锁定依赖。
- 统一发布流程为 release 构建，修复 `Makefile` 的产物路径。
- 将 UI 构建拆成独立步骤或独立产物，避免 Cargo 构建隐式跑 npm。
- CI 增加 `cargo check --workspace` 和关键测试。

**验收标准**

- 发布产物明确是 release 构建。
- 相同 commit 可复现相同依赖版本。
- 不安装 Node/npm 也能构建后端核心。

---

## ARCH-012 [P1] storage 反向依赖 service，VFS 职责过载

**证据**

- `lib/storage` 依赖 `service`，`StorageFacade` 直接使用 `CONTEXT.config_manager` 和 `CONTEXT.cloud_meta_manager`。
- `lib/cloud/src/fs/vfs.rs` 同时负责：
  - 目录树/命名空间查询
  - 块拆分与重组
  - 本地缓存读写
  - 云端副本读取
  - 文件元数据更新
- 协议层（DAV/NFS/FUSE）直接依赖 VFS，缺少应用服务层。

**影响**

- 存储层难以独立测试和复用。
- VFS 变更影响面大，职责边界不清。
- 事务、锁、缓存策略难以集中治理。

**建议**

- 让 `storage` 通过注入的配置和 repository 获取数据，移除 `service::CONTEXT` 依赖。
- 在协议层和 VFS 之间引入应用服务层。
- 将 VFS 拆分为命名空间服务、块编排服务、缓存服务和副本规划器。
- 保持模块化单体，不建议当前阶段拆微服务。

**验收标准**

- `storage` 不直接依赖 `service`。
- VFS 核心职责可独立测试。
- 协议层不直接操作数据库或缓存细节。

---

## ARCH-013 [P2] 后台任务队列与资源模型需要演进

**证据**

- `lib/cloud/src/task/mod.rs` 使用 `tokio-cron-scheduler` 每 5-20 秒轮询数据库。
- 上传队列依赖 `cloud_file_block` 状态字段，没有显式 outbox/任务表。
- `lib/cloud/src/bin/cloud.rs:19` 固定 `worker_threads = 50`。
- NFS 任务以 detached `tokio::spawn` 运行，主服务退出时没有显式优雅关闭。

**影响**

- 任务缺少重试策略、死信队列和幂等保障。
- 多实例部署时轮询任务可能重复执行。
- 固定 50 线程在 Raspberry Pi 等设备上资源浪费明显。
- 退出时 NFS 连接和上传任务可能被硬中断。

**建议**

- 将上传/清理演进为显式任务表或 outbox，支持重试、退避、死信和幂等。
- 为任务增加 jitter 和全局/账号级并发限制。
- 线程数改为配置项，根据部署环境调整。
- 增加 graceful shutdown，等待 NFS 和任务收尾。

**验收标准**

- 任务失败可重试且不会重复上传。
- 多实例部署不会重复处理同一任务。
- 资源参数可通过配置调整。
- 退出时无未完成写入被静默丢弃。
