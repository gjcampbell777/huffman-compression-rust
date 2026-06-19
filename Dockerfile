# Stage 1: Build binary
FROM rust:latest AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Stage 2: Final minimal execution layer
FROM debian:bookworm-slim
WORKDIR /app
ENTRYPOINT ["/app/huffman-compression-rust"]
