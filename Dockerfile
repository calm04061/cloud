FROM --platform=amd64 rust as builder
#RUN sed -i 's/deb.debian.org/mirror1.calm0406.tk/g' /etc/apt/sources.list && \
#    sed -i 's/security.debian.org/mirror1.calm0406.tk/g' /etc/apt/sources.list &&\
#    sed -i 's/archive.archive.ubuntu.com/mirror1.calm0406.tk/g' /etc/apt/sources.list
 ##   sed -i 's/ports.ubuntu.com/mirror1.calm0406.tk/g' /etc/apt/sources.list.d/ports.list

#ENV RUSTUP_DIST_SERVER https://rust.calm0406.tk/rust-static
RUN rustup target add armv7-unknown-linux-gnueabihf

ENV PKG_CONFIG_SYSROOT_DIR /usr/lib/arm-linux-gnueabihf/

RUN dpkg --add-architecture armhf && \
    apt-get update && \
    apt-get install -y libfuse3-dev:armhf  \
    libssl-dev:armhf  \
    libsqlite3-dev:armhf gcc-arm-linux-gnueabihf
RUN apt-get install -y libfuse3-dev

WORKDIR /app
ADD . /app
RUN cargo build --release --target armv7-unknown-linux-gnueabihf

FROM --platform=armhf debian:bullseye-slim
RUN apt-get update && \
    apt-get install -y libfuse3-dev  \
    libssl-dev  \
    libsqlite3-dev  \
    ca-certificates && \
    apt clean -y && \
    rm -rf \
    /var/cache/debconf/* \
    /var/lib/apt/lists/* \
    /var/log/* \
    /var/tmp/* \
    && rm -rf /tmp/*

WORKDIR /app

ADD log4rs.yaml /app/
ADD .env /app/

COPY --from=builder /app/target/armv7-unknown-linux-gnueabihf/release/cloud  /app/cloud

CMD ["/app/cloud"]
