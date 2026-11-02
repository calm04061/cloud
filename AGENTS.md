# AGENTS.md

Rust workspace for a personal cloud-drive server. `lib/cloud` is the only binary (`cloud`): Actix HTTP API under `/api`, WebDAV under `/dav`, NFS on port `11111`, and FUSE mounts. `lib/cloud-ui` is a separate native plugin that serves the Vue UI; it is not a dependency of `cloud`.

## Layout

- `lib/api` — shared traits (`Plugin`, `WasiPlugin`), `Capacity`, `ErrorInfo`, constants
- `lib/persistence` — domain structs, sqlx SQLite types, embedded `application.yml`
- `lib/service` — `CONTEXT` DB pool, managers, version migrations
- `lib/storage` — provider impls (ali, baidu, google, onedrive, sftp, local) behind `STORAGE_FACADE`; depends only on `api`/`persistence`/`plugin-manager`, DB access is injected at startup via `storage::repository` traits
- `lib/cloud` — entrypoint `src/bin/cloud.rs`, web routes, DAV/FUSE/NFS, cron tasks; implements the storage repositories in `src/repository.rs` and must call `init_storage_repositories()` in main
- `lib/plugin-manager` — loads native `.so`/`.dylib`/`.dll` and WASI `.wasm` plugins
- `lib/wasi-plugin`, `lib/wasi-web` — wasmtime runtimes; `wasi-web` is a workspace member but not wired into `plugin-manager` yet
- `demo-wasi-*`, `test-wasi-plugin`, `demo-wasi-web` — deliberately excluded from the workspace

## CodeGraph

At agent startup, check `.codegraph/codegraph.db`:

- If it exists, run `codegraph sync .`
- If it does not exist, run `codegraph init`
- If the `codegraph` CLI is unavailable, report that and continue without blocking.

## Build / verify

- Backend only: `cargo build -p cloud` — use this; a bare root `cargo build` also builds `cloud-ui`, whose `build.rs` runs `npm install` + `npm run build` only when `web/dist` is missing (prebuild with `make build-ui` to decouple Node/npm).
- Focused check: `cargo check -p <crate>`. Tests: `cargo test -p cloud -p storage -p service` (concurrency/DB constraint suites in `tests/` dirs and `#[cfg(test)] mod tests;` sibling files); `cargo test -p persistence` is legacy.
- No rustfmt/clippy config. CI has a `check` job (`cargo check --workspace --exclude cloud-ui` + tests) in `.github/workflows/docker-publish.yml`; the Docker job builds arm64+amd64.
- Frontend dev: `npm run dev` in `lib/cloud-ui/web` (Vite proxies `/api` to `127.0.0.1:8088`; `ALLOWED_ORIGINS` must allow `http://localhost:5173`).
- aarch64-linux cross-build uses static fuse3/ssl/crypto linker flags in `.cargo/config.toml`; `make build-image` builds via Docker (release build, mounts `Cargo.lock`). `make deploy-pi` copies `target/release/lib*.so` to `pi:/opt/cloud/plugin`.

## Run locally

- Run from the repo root, not `lib/cloud` (both have `.env`; CWD decides which is loaded). Startup hard-requires `.env` and `log4rs.yaml`; `./data` is auto-created.
- Native plugins come from `$PLUGIN_DIR` (default `./plugin`); if the directory is missing they are silently skipped with a warning, so create it before copying the UI plugin.
- `cargo run -p cloud` does not build or serve the UI. To serve it: `cargo build -p cloud-ui`, copy `target/debug/libcloud_ui.so` (`.dylib` on macOS) into `./plugin`, then start `cloud`.
- HTTP port is `HTTP_PORT` (default `8088`); NFS binds `NFS_BIND_ADDR` (default `127.0.0.1:11111`). OAuth callbacks use `PUBLIC_BASE_URL` when set, otherwise the request `Host` header.

## Config / DB gotchas

- DB URL: `DATABASE_URL` env wins over the compiled-in `lib/persistence/application.yml` default (`sqlite://./data/file.db?mode=rwc`). The default path is relative to CWD; set `DATABASE_URL` to point tests or a second instance at another DB.
- `TEMP_PATH` priority: `.env` > DB `config` table > built-in default (`/var/lib/storage/temp`); all FUSE/DAV/NFS/scan/clean paths read it via `cloud::fs::config::temp_path()`. `MOUNT_PATH` is DB-only (required for FUSE). `MAX_NUMBER_OF_SHARD` reads env with DB fallback. Startup logs an effective-config summary and panics if `TEMP_PATH` cannot be created.
- Local block cache eviction: `CACHE_MAX_GB` (default 20) with `CACHE_WATERMARK_PERCENT` (default 80) runs in the `evict_cache` cron task; blocks whose replica rows still lack `cloud_file_id` are never evicted.
- Upload queue: `cloud_file_block` rows are the outbox — scan picks Init / backoff-expired UploadFail / hash-stale rows (limit 8, CAS claim to Uploading); failures back off exponentially (`UPLOAD_BACKOFF_BASE_SECONDS`, default 60, cap 1h + jitter) until `UPLOAD_MAX_ATTEMPTS` (default 10) then go dead-letter `FileStatus::Dead(10)`; `reset` task returns Uploading rows stuck >10min to Init (crash recovery). FileStatus codes are 1-based (Init=1 … CleanFail=9, Dead=10).
- Tokio worker threads: `TOKIO_WORKER_THREADS` env, default CPU count (was fixed 50).
- Migrations are not auto-discovered: add `lib/service/src/migrate/version/*.sql` **and** register the `include_str!` + `versions_migrate.add(...)` in `ServiceContext::upgrade` (`lib/service/src/lib.rs`). `all.sql` is not executed; the runner splits each migration on `;`.
- `0.0.1.sql` seeds the `admin` user, the `/` file row, and `CLOUD_FILE_ROOT`. There is no user CRUD API; `/api` and `/dav` use Basic Auth against the `user` table.
- `cloud_meta.auth` is encrypted at rest with AES-GCM. `DATA_ENCRYPTION_KEY` is optional; if unset, a key file is created at `./data/encryption.key`. Existing plaintext rows are encrypted during startup migration.
- The root `.env` is tracked and already contains live provider credentials. Do not add more secrets there; update `.env.example` instead.

## Plugins

- Native plugins load from `$PLUGIN_DIR` (default `./plugin`); WASI plugins from `$WASI_PLUGIN_DIR` (default `./wasi-plugins`).
- Native plugin allowlist is `PLUGIN_ALLOWLIST` (comma-separated names); unset means all native plugins are trusted. Native plugins should export `plugin_abi_version` and match `api::PLUGIN_ABI_VERSION`.
- Native plugin ABI: export `plugin_meta`, plus one `fn(&mut actix_web::web::ServiceConfig)` per `Capacity::WEB(name)`.
- WASI ABI: `name`/`version`/`capacities` return pointers into wasm memory; methods are `call_<method>(i32,i32)->i32`. `STORAGE` and `WEB` capacities for WASI are currently logged and skipped.
- The UI is only served when a native plugin exposing the `config` WEB capacity is present.

## Platform notes

- FUSE and SFTP are `#[cfg(not(windows))]`; NFS startup itself is not cfg-gated. `lib/cloud/build.rs` uses vcpkg for zstd/openssl only on Windows.
- macOS builds enable fuser's `macos-no-mount` feature (no macFUSE/pkg-config needed), so FUSE mounting is unavailable there; Linux builds use real FUSE.
- API JSON envelope is `{message, code, data}` for both success and failure; `ErrorInfo` maps to HTTP status (404 FileNotFound, 409 FileAlreadyExist, 401 Http401/Http402, else 500) and serializes into the same envelope. `/health` and `/ready` probes are unauthenticated.
