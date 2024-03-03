FROM rust:1.76 as builder
WORKDIR /app
COPY ./Cargo.toml .
COPY ./Cargo.lock .
COPY ./src ./src
COPY ./.sqlx ./.sqlx

RUN cargo build --release

FROM debian:12.5
WORKDIR /app
COPY --from=builder /app/target/release/rinha-backend-2024-q1-rust .
ENTRYPOINT [ "./rinha-backend-2024-q1-rust" ]