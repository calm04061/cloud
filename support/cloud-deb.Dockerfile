FROM registry.cn-beijing.aliyuncs.com/dinghan/cloud:builder
RUN cargo install cargo-deb &&\
  rm -rf /root/.cargo/git &&\
  rm -rf /root/.cargo/registry
