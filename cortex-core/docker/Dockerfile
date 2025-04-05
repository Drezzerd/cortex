# === Builder stage ===
FROM rust:1.81 as builder

WORKDIR /app

# Copie les fichiers nécessaires
COPY cortex-core/rust/cortex-id/ ./

# Compilation statique
RUN apt-get update && apt-get install -y musl-tools && \
    rustup target add x86_64-unknown-linux-musl && \
    cargo build --release --target x86_64-unknown-linux-musl

# === Final image ===
FROM alpine:latest
RUN apk add --no-cache ca-certificates

WORKDIR /root/
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/cortex-node .

ENTRYPOINT ["./cortex-node"]
