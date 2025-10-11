# Build stage
FROM rust:1.60-slim as builder

WORKDIR /usr/src/codex-orchestrator
COPY . .

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y libssl1.1 ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /usr/src/codex-orchestrator/target/release/codex-orchestrator /usr/local/bin/

# Create a non-root user
RUN useradd -m codex
USER codex

# Set the working directory
WORKDIR /home/codex

# Expose the port the app runs on
EXPOSE 3000

# Set the default command to run the application
CMD ["codex-orchestrator"]
