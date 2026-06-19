# Stage 1: Build binary
FROM rust:1.85-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Stage 2: Final minimal execution layer
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Set global installation paths BEFORE installing Rust
ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV PATH="/usr/local/cargo/bin:${PATH}"

# Install Rust (it will now install to the paths above)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Grant read and execute permissions to all users
RUN chmod -R 755 /usr/local/cargo /usr/local/rustup

WORKDIR /app
COPY --from=builder /app/target/release/huffman-compression-rs /app/huffman-compression-rust
ENTRYPOINT ["/app/huffman-compression-rust"]
