FROM rust:1.72.0-alpine3.18 as builder
RUN apk add musl-dev
RUN apk add pkgconfig
RUN apk add libressl-dev
WORKDIR /app
COPY . .
RUN cargo build -r

FROM gcr.io/distroless/cc
COPY --from=builder /app/target/release/elton /
CMD ["./elton"]
