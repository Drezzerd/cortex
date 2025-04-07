# === Builder stage ===
FROM rust:1.81 as builder

WORKDIR /app

# Dépendances système
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# Pré-copie pour cache optimal
COPY cortex-core/rust/cortex-id/Cargo.toml ./Cargo.toml

RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release || true
RUN rm -rf src

# Copie du code source complet
COPY cortex-core/rust/cortex-id/ .

# Compilation
RUN cargo build --release

# === Image finale ===
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /root/
COPY --from=builder /app/target/release/cortex-id .

ENTRYPOINT ["./cortex-id"]
