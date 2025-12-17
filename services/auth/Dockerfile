# This file is generated. Do not edit directly.

FROM rust:1.88-bookworm AS chef
WORKDIR /services

# Install cargo-chef
RUN cargo install --git https://github.com/preiter93/cargo-chef --branch reduce-workspace

# Prepare dependency recipe
FROM chef AS planner
COPY . .
RUN cargo chef prepare --bin auth --recipe-path recipe.json

# Build the dependencies
FROM chef AS builder
COPY --from=planner /services/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --bin auth

# Build the binary
COPY ../.docker-gen/Cargo.toml.auth Cargo.toml
COPY ../Cargo.lock Cargo.lock
COPY ../pkg/setup pkg/setup
COPY ../pkg/oauth pkg/oauth
COPY ../auth auth
COPY ../pkg/testutils pkg/testutils
COPY ../pkg/common pkg/common
RUN cargo build --release --bin auth

# Run the service
FROM debian:bookworm-slim AS runtime
COPY --from=builder /services/target/release/auth /usr/local/bin/main
ENTRYPOINT ["/usr/local/bin/main"]