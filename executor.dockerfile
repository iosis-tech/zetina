# Use the base runtime image
FROM zetina-runtime:latest

# Build
RUN cargo build --release --bin zetina-executor
