FROM ubuntu:20.04 as builder
#定义时区参数
ENV TZ=Asia/Shanghai
#设置时区
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo '$TZ' > /etc/timezone
RUN apt update && \
  apt install -y pkg-config build-essential libssl-dev curl libfuse3-dev npm &&\
  apt autoclean && \
  rm -rf /var/lib/apt/lists/*
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable-gnu
ENV PATH=/root/.cargo/bin:$PATH
WORKDIR /src
ADD . /src/
RUN cargo build --release --target "`arch`-unknown-linux-gnu"

FROM alpine:3.18.2
COPY --from=builder /src/target/"`arch`-unknown-linux-gnu"/release/cloud /app/
