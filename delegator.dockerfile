# Stage 1: Use the official Rust image from the Docker Hub
FROM rust:latest

# Set the working directory inside the container
WORKDIR /zetina-delegator

# Copy the rest of the application source code
COPY . .

# Expose necessary ports
EXPOSE 5678/udp 5679/tcp 3000/tcp

# Build the application in release mode
RUN cargo build --release --bin zetina-delegator