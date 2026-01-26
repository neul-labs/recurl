# recurl Docker image
# Build: docker build -t recurl .
# Run: docker run --rm recurl https://example.com

# Build stage
FROM rust:1.75-bookworm AS builder

WORKDIR /build

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy source
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY tests ./tests

# Build release binaries
RUN cargo build --release
RUN cargo build --release --bin recurld --features daemon

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    chromium \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -s /bin/bash recurl

# Copy binaries
COPY --from=builder /build/target/release/recurl /usr/local/bin/
COPY --from=builder /build/target/release/recurld /usr/local/bin/

# Set permissions
RUN chmod +x /usr/local/bin/recurl /usr/local/bin/recurld

# Switch to non-root user
USER recurl
WORKDIR /home/recurl

# Default command
ENTRYPOINT ["recurl"]
CMD ["--help"]

# Labels
LABEL org.opencontainers.image.title="recurl"
LABEL org.opencontainers.image.description="Drop-in curl replacement with automatic anti-bot bypass"
LABEL org.opencontainers.image.source="https://github.com/user/recurl"
LABEL org.opencontainers.image.licenses="MIT"
