ARG REPO=registry.cn-beijing.aliyuncs.com/dinghan
FROM ${REPO}/cloud:builder as builder
ADD . /src/
RUN cargo build --release --target `rustup target list |grep install|awk '{print $1}'`
RUN cp /src/target/`rustup target list |grep install|awk '{print $1}'`/release/cloud /src/target/cloud

FROM ${REPO}/cloud:base
COPY --from=builder /src/target/cloud /app/
