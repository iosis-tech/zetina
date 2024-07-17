# Use the official Rust image from the Docker Hub
FROM rust:1.79.0-slim

# Set the working directory inside the container
WORKDIR /zetina-delegator

# Copy the rest of the application source code
COPY . .

RUN cargo update -p rustls@0.23.11 --precise 0.23.10
RUN cargo update -p rustls-webpki@0.102.5 --precise 0.102.4

# Build the application in release mode
RUN cargo build --release --bin zetina-delegator

# Expose necessary ports
EXPOSE 5678/udp 5679/tcp 3010/tcp