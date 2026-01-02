# rcurl Docker image
# Build: docker build -t rcurl .
# Run: docker run --rm rcurl https://example.com

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
RUN cargo build --release --bin rcurld --features daemon

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
RUN useradd -m -s /bin/bash rcurl

# Copy binaries
COPY --from=builder /build/target/release/rcurl /usr/local/bin/
COPY --from=builder /build/target/release/rcurld /usr/local/bin/

# Set permissions
RUN chmod +x /usr/local/bin/rcurl /usr/local/bin/rcurld

# Switch to non-root user
USER rcurl
WORKDIR /home/rcurl

# Default command
ENTRYPOINT ["rcurl"]
CMD ["--help"]

# Labels
LABEL org.opencontainers.image.title="rcurl"
LABEL org.opencontainers.image.description="Drop-in curl replacement with automatic anti-bot bypass"
LABEL org.opencontainers.image.source="https://github.com/user/rcurl"
LABEL org.opencontainers.image.licenses="MIT"
