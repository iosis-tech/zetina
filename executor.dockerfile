# Use the base runtime image
FROM zetina-runtime:latest

# Build
RUN cargo build --release --bin zetina-executor

# Expose necessary ports
EXPOSE 5678/udp 5679/tcp