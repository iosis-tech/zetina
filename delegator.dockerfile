# Use the base runtime image
FROM zetina-runtime

# Build
RUN cargo build --release --bin zetina-delegator

# Expose necessary ports
EXPOSE 5678/udp 5679/tcp 3010/tcp
