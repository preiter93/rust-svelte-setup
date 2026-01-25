# This file is generated. Do not edit directly.

FROM rust:1.88-bookworm AS chef
WORKDIR /services

# Install cargo-chef
RUN cargo install --git https://github.com/preiter93/cargo-chef --branch reduce-workspace

# Prepare dependency recipe
FROM chef AS planner
COPY . .
RUN cargo chef prepare --bin gateway --recipe-path recipe.json

# Build the dependencies
FROM chef AS builder
COPY --from=planner /services/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --bin gateway

# Build the binary
COPY ../.docker-gen/Cargo.toml.gateway Cargo.toml
COPY ../Cargo.lock Cargo.lock
COPY ../auth auth
COPY ../gateway gateway
COPY ../pkg/common pkg/common
COPY ../pkg/database pkg/database
COPY ../pkg/oauth pkg/oauth
COPY ../pkg/setup pkg/setup
COPY ../pkg/testutils pkg/testutils
COPY ../user user
RUN cargo build --release --bin gateway

# Run the service
FROM debian:bookworm-slim AS runtime
COPY --from=builder /services/target/release/gateway /usr/local/bin/main
ENTRYPOINT ["/usr/local/bin/main"]