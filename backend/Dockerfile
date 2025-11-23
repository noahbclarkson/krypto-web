# syntax=docker/dockerfile:1.7-labs

############################
# Stage 0: Chef Planner
############################
FROM rust:1.90-slim-bookworm AS planner
WORKDIR /app

ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

RUN cargo install cargo-chef --locked
COPY . .
RUN cargo chef prepare --recipe-path /app/recipe.json

############################
# Stage 1: Tools (sqlx-cli)
############################
FROM rust:1.90-slim-bookworm AS tools
WORKDIR /tools
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install sqlx-cli --no-default-features --features rustls,postgres --locked
RUN cargo install cargo-chef --locked

############################
# Stage 2: Builder
############################
FROM rust:1.90-slim-bookworm AS builder
WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential pkg-config libssl-dev ca-certificates \
    git clang cmake zlib1g-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy tools
COPY --from=tools /usr/local/cargo/bin/ /usr/local/bin/

# Optional: Support private GitHub dependencies
ARG GITHUB_TOKEN=""
RUN if [ -n "$GITHUB_TOKEN" ]; then \
    git config --global url."https://oauth2:${GITHUB_TOKEN}@github.com/".insteadOf "https://github.com/"; \
    fi

# Build dependencies first (cargo-chef optimization)
COPY --from=planner /app/recipe.json /app/recipe.json
RUN cargo chef cook --release --recipe-path /app/recipe.json

# Copy source code
COPY . /app

# Set required environment variables
ARG DATABASE_URL
ENV DATABASE_URL=$DATABASE_URL
ARG CARGO_BIN=""
ARG CARGO_FEATURES=""

# Ensure DATABASE_URL is provided
RUN test -n "$DATABASE_URL" || { echo "ERROR: DATABASE_URL is required for compilation"; exit 1; }

# Run migrations before compilation
RUN echo "Running database migrations..." && \
    sqlx migrate run --database-url "$DATABASE_URL"

# Build the application
RUN echo "Building application..." && \
    if [ -n "$CARGO_FEATURES" ]; then \
    cargo build --release ${CARGO_BIN:+--bin "$CARGO_BIN"} --features "$CARGO_FEATURES"; \
    else \
    cargo build --release ${CARGO_BIN:+--bin "$CARGO_BIN"}; \
    fi

# Clean up git config
RUN if [ -n "$GITHUB_TOKEN" ]; then \
    git config --global --unset url."https://oauth2:${GITHUB_TOKEN}@github.com/".insteadOf || true; \
    fi

############################
# Stage 3: Runtime
############################
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates openssl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd --system app && useradd --system --gid app --create-home app
USER app

# Copy only the binary
COPY --from=builder /app/target/release/backend ./backend

# Copy migrations (optional, useful for troubleshooting)
COPY --from=builder /app/migrations ./migrations

EXPOSE 8080
CMD ["./backend"]
