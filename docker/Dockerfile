FROM rust:1.91 AS builder
RUN apt-get update && apt-get install -y libssl-dev pkg-config
WORKDIR /app

COPY .env ./.env
COPY Cargo.toml Cargo.lock ./
COPY src ./src


RUN cargo build --release

FROM debian:bookworm-slim
RUN useradd -m appuser
WORKDIR /app
COPY --from=builder /app/target/release/remoterelay /usr/local/bin/remoterelay
COPY --from=builder /app/.env /app/.env
USER appuser
CMD ["remoterelay", "--config=mongodb"]
