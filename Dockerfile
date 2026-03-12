FROM rust:trixie

WORKDIR /app

COPY . .

RUN cargo build --release

CMD ["./target/release/mip"]
