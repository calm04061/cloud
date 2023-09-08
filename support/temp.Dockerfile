FROM ghcr.io/calm04061/rust:ub
RUN export TEMP_TARGET=`rustup target list |grep install|awk '{print $1}'`&& env
ENV BUILD_TARGET=$(`rustup target list`)
RUN env
WORKDIR /src
#RUN apt update &&\
#  apt install -y libfuse3-dev npm &&\
#  apt autoclean &&\
#  rm -rf /var/lib/apt/lists/*
#RUN cargo install cargo-deb

