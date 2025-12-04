# Multi-stage build para o backend Rust
FROM rust:1.75-slim as builder

# Instalar dependências necessárias
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copiar Cargo.toml e Cargo.lock primeiro (para cache)
COPY backend/Cargo.toml ./Cargo.toml
COPY backend/Cargo.lock ./Cargo.lock

# Build apenas as dependências (cache layer)
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copiar código fonte completo
COPY backend/src ./src

# Build final
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Instalar dependências runtime
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copiar binário
COPY --from=builder /app/target/release/erp-backend .

# Variáveis de ambiente (serão sobrescritas pelo Railway)
ENV RUST_LOG=info
ENV PORT=8080

EXPOSE 8080

CMD ["./erp-backend"]
