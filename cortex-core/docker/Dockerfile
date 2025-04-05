# === Builder stage ===
FROM rust:1.81 as builder

WORKDIR /app

# Installation des outils nécessaires
RUN apt-get update && apt-get install -y musl-tools && \
    rustup target add x86_64-unknown-linux-musl

# Copier d'abord Cargo.toml pour optimiser la mise en cache des dépendances
COPY cortex-core/rust/cortex-id/Cargo.toml ./

# Créer une structure temporaire pour compiler les dépendances
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    echo "pub fn dummy() {}" > src/lib.rs && \
    mkdir -p src/bin && \
    echo "fn main() {}" > src/bin/cortex-node.rs && \
    echo "pub mod discovery;" > src/lib.rs && \
    echo "pub mod identity;" >> src/lib.rs && \
    mkdir -p src/discovery && \
    echo "// dummy" > src/discovery/mod.rs && \
    mkdir -p src/identity && \
    echo "// dummy" > src/identity/mod.rs && \
    cargo build --release --target x86_64-unknown-linux-musl

# Copier le vrai code source
COPY cortex-core/rust/cortex-id/src ./src

# Compiler pour de vrai
RUN touch src/lib.rs src/main.rs src/bin/cortex-node.rs && \
    cargo build --release --target x86_64-unknown-linux-musl

# === Final image ===
FROM alpine:latest

RUN apk add --no-cache ca-certificates

WORKDIR /root/
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/cortex-node .

ENTRYPOINT ["./cortex-node"]