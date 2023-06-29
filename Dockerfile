FROM builder as builder
ADD . /src/
RUN npm -v
RUN cargo build --release --target "`arch`-unknown-linux-gnu"
RUN cp /src/target/`arch`-unknown-linux-gnu/release/cloud /src/target/cloud

FROM alpine:3.18.2
RUN mkdir -p /src/config
COPY --from=builder /src/target/cloud /app/
COPY log4rs.yaml /app/