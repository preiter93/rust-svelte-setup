# TODO: Use multistage build
# TODO: Use slim image
# TODO: Use prebuild image

FROM rust:1.88.0-bookworm as dev
RUN cargo install cargo-chef

ARG SERVICE_NAME=gateway
ENV SERVICE_NAME=${SERVICE_NAME}

WORKDIR /services

COPY . .

RUN cargo chef prepare  --recipe-path recipe.json --bin ${SERVICE_NAME}
RUN cargo chef cook --release --recipe-path recipe.json --bin ${SERVICE_NAME}
RUN cargo build --release --bin ${SERVICE_NAME}

# EXPOSE 3000

# WORKDIR /services/${SERVICE_NAME}

# CMD ./target/release/${SERVICE_NAME} 
