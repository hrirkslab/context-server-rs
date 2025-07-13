# Multi-stage build for optimized production image
FROM rust:1.76-alpine AS builder

# Install required dependencies
RUN apk add --no-cache musl-dev sqlite-dev

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build for release
RUN cargo build --release

# Runtime stage
FROM alpine:3.19

# Install runtime dependencies
RUN apk add --no-cache sqlite

# Create non-root user
RUN addgroup -g 1001 -S context-server && \
    adduser -S -D -H -u 1001 -h /app -s /sbin/nologin -G context-server context-server

WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/context-server-rs ./

# Create config directory
RUN mkdir -p /app/config && \
    chown -R context-server:context-server /app

# Switch to non-root user
USER context-server

# Expose health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD echo '{"jsonrpc":"2.0","id":1,"method":"ping"}' | nc -q 1 localhost 3000 || exit 1

# Run the binary
CMD ["./context-server-rs"]
