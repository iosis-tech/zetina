# Use the base runtime image
FROM runtime

# Build
RUN cargo build --release --bin zetina-executor

# Expose necessary ports
EXPOSE 5678/udp 5679/tcp 50052/tcp

# Set the default command to run when the container starts
CMD ["bash", "-ci", "cargo run --release --bin zetina-executor"]
