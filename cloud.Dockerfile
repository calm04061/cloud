FROM ghcr.io/calm04061/rust:ubuntu-1.70.0
RUN apt update &&\
  apt install -y libfuse3-dev npm &&\
  apt autoclean &&\
  rm -rf /var/lib/apt/lists/* &&\
  cargo install cargo-deb &&\
  rm -rf /root/.cargo/git &&\
  rm -rf /root/.cargo/registry
