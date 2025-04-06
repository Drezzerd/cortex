# === Builder stage ===
FROM rust:1.81 as builder

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# Pre-cache dependencies
COPY cortex-core/rust/cortex-id/Cargo.toml ./
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    echo "pub fn dummy() {}" > src/lib.rs && \
    mkdir -p src/bin && \
    echo "fn main() {}" > src/bin/cortex-node.rs && \
    cargo build --release

# Copy real source code
COPY cortex-core/rust/cortex-id/src ./src

# Final build
RUN cargo build --release

# === Final image ===
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /root/
COPY --from=builder /app/target/release/cortex-node .

ENTRYPOINT ["./cortex-node"]
