FROM builder as builder
ADD . /src/
RUN npm -v
RUN cargo build --release --target "`arch`-unknown-linux-gnu"

FROM alpine:3.18.2
COPY --from=builder /src/target/"`arch`-unknown-linux-gnu"/release/cloud /app/
