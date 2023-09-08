FROM alpine:3.18.2
WORKDIR /app/
VOLUME /app/data
COPY ../log4rs.yaml /app/
CMD ["./cloud"]

