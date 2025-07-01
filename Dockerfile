# Stage 1: Build frontend
FROM node:20-alpine AS frontend-builder

WORKDIR /app/frontend

COPY frontend/package.json frontend/package-lock.json ./
RUN npm install

COPY frontend/ .

RUN npm run build

# Stage 2: Build backend
FROM rust:latest AS backend-builder

WORKDIR /app

# Copy frontend artifacts
COPY --from=frontend-builder /app/frontend/dist /app/frontend/dist

# Install build dependencies
RUN apt-get update && apt-get install -y libssl-dev pkg-config

# Build backend
COPY . .
RUN cargo build --release

# Stage 3: Final image
FROM debian:12-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -r -s /bin/false -m -d /app smokeping

WORKDIR /app

# Create data directory for SQLite database with proper ownership
RUN mkdir -p /data && chown smokeping:smokeping /data && chmod 755 /data

COPY --from=backend-builder /app/target/release/smokeping-rs .
RUN chown smokeping:smokeping /app/smokeping-rs && chmod +x /app/smokeping-rs

USER smokeping

EXPOSE 3000

CMD ["./smokeping-rs"]
