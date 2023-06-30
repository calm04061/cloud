FROM ghcr.io/calm04061/cloud:builder as builder
ADD . /src/
RUN cargo build --release --target `rustup target list |grep install|awk '{print $1}'`
RUN cp /src/target/`rustup target list |grep install|awk '{print $1}'`/release/cloud /src/target/cloud

FROM alpine:3.18.2
WORKDIR /app/
VOLUME /app/data
COPY --from=builder /src/target/cloud /app/
COPY log4rs.yaml /app/
CMD ["./cloud"]
