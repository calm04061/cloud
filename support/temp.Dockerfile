FROM alpine:3.19.1
ARG RUST_VERSION=1.76.0
WORKDIR /src

ENV PATH="/root/.cargo/bin:/opt/node/bin:${PATH}"
RUN apk add --no-cache gcc musl-dev pkgconf curl openssl-dev openssl-libs-static fuse3-dev fuse3-static nodejs npm

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain ${RUST_VERSION}


