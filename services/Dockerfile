# Usage:
# docker build --build-arg SERVICE_NAME=gateway -t srb/gateway .

ARG SERVICE_NAME=default

FROM lukemathwalker/cargo-chef:latest-rust-1.88-bookworm as chef
WORKDIR /services

# Prepare dependency graph
FROM chef as planner
ARG SERVICE_NAME
ENV SERVICE_NAME=${SERVICE_NAME}
COPY . .
RUN cargo chef prepare --recipe-path recipe.json --bin ${SERVICE_NAME}

# Build the dependencies
FROM chef as builder
ARG SERVICE_NAME
ENV SERVICE_NAME=${SERVICE_NAME}
RUN cargo install cargo-chef
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
