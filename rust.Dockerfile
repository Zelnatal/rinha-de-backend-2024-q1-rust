FROM rust:1.76-alpine as builder
WORKDIR /app
COPY ./Cargo.toml .
COPY ./Cargo.lock .
COPY ./src ./src
COPY ./.sqlx ./.sqlx

RUN apk add --no-cache musl-dev

RUN cargo build --release

FROM alpine:latest
WORKDIR /app
COPY --from=builder /app/target/release/rinha-backend-2024-q1-rust .
ENTRYPOINT [ "./rinha-backend-2024-q1-rust" ]