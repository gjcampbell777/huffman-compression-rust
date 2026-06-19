# Stage 1: Build binary
FROM rust:1.85-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Stage 2: Final minimal execution layer
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/huffman-compression-rs /app/huffman-compression-rust
ENTRYPOINT ["/app/huffman-compression-rust"]
