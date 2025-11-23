ARG SERVICE_NAME

FROM rust:1.88-bookworm AS chef
WORKDIR /services

# Install cargo-chef
RUN cargo install --git https://github.com/preiter93/cargo-chef --branch reduce-workspace

# Prepare and reduce the workspace recipe
FROM chef AS planner
ARG SERVICE_NAME
ENV SERVICE_NAME=${SERVICE_NAME}
COPY . .
RUN cargo chef prepare --recipe-path recipe.json --bin ${SERVICE_NAME}

# Build the dependencies
FROM chef AS builder
ARG SERVICE_NAME
ENV SERVICE_NAME=${SERVICE_NAME}
COPY --from=planner /services/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --bin ${SERVICE_NAME}

# Build the binary
COPY . .
RUN cargo build --release --bin ${SERVICE_NAME}

# Run the service
FROM debian:bookworm-slim AS runtime
ARG SERVICE_NAME
COPY --from=builder /services/target/release/${SERVICE_NAME} /usr/local/bin/main
ENTRYPOINT ["/usr/local/bin/main"]
