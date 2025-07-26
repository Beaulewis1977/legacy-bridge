# Multi-stage Dockerfile for LegacyBridge
# Optimized for size and security

# Stage 1: Rust builder
FROM rust:1.75-slim AS rust-builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy Rust project files
COPY legacybridge/dll-build/Cargo.toml legacybridge/dll-build/Cargo.lock ./
COPY legacybridge/dll-build/src ./src

# Build the DLL
RUN cargo build --release --features dll-export

# Stage 2: Node.js builder
FROM node:20-alpine AS node-builder

WORKDIR /build

# Copy package files
COPY legacybridge/package*.json ./

# Install dependencies
RUN npm ci --only=production

# Copy source files
COPY legacybridge/ ./

# Build the frontend
RUN npm run build

# Stage 3: Final runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 -s /bin/bash legacybridge

# Create app directory
WORKDIR /app

# Copy built artifacts from builder stages
COPY --from=rust-builder /build/target/release/liblegacybridge.so ./lib/
COPY --from=node-builder /build/.next ./.next
COPY --from=node-builder /build/public ./public
COPY --from=node-builder /build/node_modules ./node_modules
COPY --from=node-builder /build/package.json ./

# Copy configuration files
COPY legacybridge/include/legacybridge.h ./include/

# Set ownership
RUN chown -R legacybridge:legacybridge /app

# Switch to non-root user
USER legacybridge

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/api/health || exit 1

# Set build arguments
ARG VERSION=unknown
ARG BUILD_DATE=unknown
LABEL version="${VERSION}" \
      build-date="${BUILD_DATE}" \
      description="LegacyBridge Enterprise RTF-Markdown Converter" \
      maintainer="LegacyBridge Team"

# Start the application
CMD ["npm", "start"]