ARG SERVICE_NAME

FROM rust:1.88-bookworm AS chef
WORKDIR /services

# Install cargo-chef and cargo-reduce-workspace-recipe
RUN cargo install cargo-chef --locked --version 0.1.73 \
    && cargo install --git https://github.com/preiter93/cargo-reduce-recipe --tag v0.1.0

# Prepare and reduce the workspace recipe
FROM chef as planner
ARG SERVICE_NAME
ENV SERVICE_NAME=${SERVICE_NAME}
COPY . .
RUN cargo chef prepare --bin ${SERVICE_NAME} --recipe-path recipe.json \
    && cargo-reduce-recipe --recipe-path-in recipe.json --recipe-path-out recipe-reduced.json

# Build the dependencies
FROM chef as builder
ARG SERVICE_NAME
ENV SERVICE_NAME=${SERVICE_NAME}
COPY --from=planner /services/recipe-reduced.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --bin ${SERVICE_NAME}

# Build the binary
COPY . .
RUN cargo build --release --bin ${SERVICE_NAME}

# Run the service
FROM debian:bookworm-slim AS runtime
ARG SERVICE_NAME
COPY --from=builder /services/target/release/${SERVICE_NAME} /usr/local/bin/main
ENTRYPOINT ["/usr/local/bin/main"]
