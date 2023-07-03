FROM registry.cn-beijing.aliyuncs.com/dinghan/rust:ubuntu-1.70.0
RUN apt update &&\
  apt install -y libfuse3-dev npm &&\
  apt autoclean &&\
  rm -rf /var/lib/apt/lists/*
