# === Builder stage ===
FROM rust:1.81 as builder

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# Pré-copie pour cache optimal
COPY cortex-core/rust/cortex-id/Cargo.toml .

# Dummy build to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release || true
RUN rm -rf src

# Copy actual full source
COPY cortex-core/rust/cortex-id/ .

# Compile
RUN cargo build --release

# === Final image ===
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /root/
COPY --from=builder /app/target/release/cortex-node .

ENTRYPOINT ["./cortex-node"]
