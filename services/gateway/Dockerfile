# TODO: Use multistage build
# TODO: Use slim image
FROM rust:1.88.0-bookworm as dev

WORKDIR /services

COPY . .

EXPOSE 3000

WORKDIR /services/gateway

CMD cargo run
