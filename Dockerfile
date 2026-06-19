# Stage 1: Build binary
FROM rust:1.85-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Stage 2: Final minimal execution layer
FROM debian:bookworm-slim
WORKDIR /app
ENTRYPOINT ["/app/huffman-compression-rust"]