ARG REPO=ghcr.io/calm04061
FROM ${REPO}/cloud:builder as builder
ADD . /src/
RUN cargo build --release --target `rustup target list |grep install|awk '{print $1}'`
