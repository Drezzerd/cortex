# === Builder stage ===
FROM rust:1.81 as builder

WORKDIR /app

# Installation des dépendances système
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# Pré-copie pour optimiser le cache
COPY core/Cargo.toml ./Cargo.toml

RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release || true
RUN rm -rf src

# Copie du code source complet
COPY core/ .

# Compilation de l'exécutable
RUN cargo build --release

# === Image finale ===
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# Création d'un utilisateur non-root
RUN adduser --disabled-password --gecos "" cortexuser

# Passage à l'utilisateur non-root
USER cortexuser
WORKDIR /home/cortexuser

# Copie de l'exécutable depuis le builder
COPY --from=builder /app/target/release/cortex-id .

ENTRYPOINT ["./cortex-id"]
