FROM ubuntu:23.04
#定义时区参数
ENV TZ=Asia/Shanghai
#设置时区
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo '$TZ' > /etc/timezone
RUN apt update &&\
  apt install -y pkg-config build-essential libssl-dev curl &&\
  apt autoclean &&\
  rm -rf /var/lib/apt/lists/*
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable --profile minimal
ENV PATH /root/.cargo/bin:$PATH
WORKDIR /src

