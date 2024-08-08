# Use the base runtime image
FROM zetina-runtime:latest

# Expose necessary ports
EXPOSE 5678/udp 5679/tcp 3000/tcp

# Build
RUN cargo build --release --bin zetina-executor
