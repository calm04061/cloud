FROM debian:bullseye-slim
ARG RUST_VERSION=1.81.0
ARG NODE_VERSION=20.17.0
WORKDIR /src

ENV PATH="/root/.cargo/bin:/opt/node/bin:${PATH}"
RUN apt update -y && \
    apt install -y gcc pkg-config curl libssl-dev libfuse3-dev git  && \
    apt clean -y && \
    rm -rf \
    /var/cache/debconf/* \
    /var/lib/apt/lists/* \
    /var/log/* \
    /var/tmp/*  && \
    rm -rf /tmp/*
RUN cd /opt && \
    curl -L https://nodejs.org/dist/v${NODE_VERSION}/node-v${NODE_VERSION}-linux-arm64.tar.xz -o node-v${NODE_VERSION}-linux-arm64.tar.xz &&\
    tar xf node-v${NODE_VERSION}-linux-arm64.tar.xz && \
    mv node-v${NODE_VERSION}-linux-arm64 node &&\
    rm node-v${NODE_VERSION}-linux-arm64.tar.xz && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain ${RUST_VERSION}

ADD . /src
RUN cargo build --release