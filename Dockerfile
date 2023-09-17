FROM rust:1.72.0

WORKDIR /usr/src/elton
COPY . .

RUN cargo build -r

CMD ["./target/release/elton"]
