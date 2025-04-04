# === Builder stage ===
FROM rust:1.86 as builder

WORKDIR /app

COPY cortex-id/Cargo.toml ./Cargo.toml
COPY cortex-id/src ./src

RUN apt-get update && apt-get install -y musl-tools && \
    rustup target add x86_64-unknown-linux-musl && \
    cargo build --release --target x86_64-unknown-linux-musl

# === Final image ===
FROM alpine:latest

RUN apk add --no-cache ca-certificates

WORKDIR /root/

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/cortex-id .

CMD ["./cortex-id"]
