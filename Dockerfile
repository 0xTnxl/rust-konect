# Build stage
FROM rust:1.82 as builder

WORKDIR /app

# Copy workspace files
COPY Cargo.toml ./
COPY backend/ ./backend/

# Build the application
RUN cd backend && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary
COPY --from=builder /app/target/release/rust-konect-backend /usr/local/bin/app

# Copy frontend files
COPY frontend/ ./frontend/

# Create uploads directory
RUN mkdir -p uploads

# Expose port
EXPOSE 3000

# Run the application
CMD ["app"]