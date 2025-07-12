# Overview

This project is an opinionated base setup for sveltekit apps with a rust based microservice backend.

# App

...

# Services

The backend is structured into microservices. A clients request always reaches the `gateway` service
first in which the request is authenticated and then forwarded the the respective microservice.
The gateway exposes a restful http server using `axum`. Within the backend, the communication is done
via `grpc` using `tonic` and protos are compiled with `prost` to rust code.

## Microservice structure
- `main.rs`: setup (e.g. read env variables, open db connection) and run the service
- `lib.rs`: expose service boundaries such as the `proto.rs` for other microservices (see `Microservice boundaries`)
- `handler.rs`: implement the http/grpc endpoints
- `db.rs`: the database layer
- `utils.rs`: shared methods between endpoints, models, etc.

## Microservice boundaries (`lib.rs`)

Microservices must have access to the api layer of other microservices, which means they must have access to
the proto generated client and request/response messages of other microservices. This may be solve by
- compiling the protos in a common `proto` library and including the common library in the microservice, or
- compiling the proto that belongs to the service as part of the service and exposing it in `lib.rs`.
This setup uses the second solution. It avoids introducing a shared `proto` library and additionally
each service can define which part of the proto it wants to expose. Note: the `lib.rs` should not expose
more than needed by other service, so usually it only exposes the full or parts of the `proto.rs`.

## Shared dependencies (`workspace`)

Microservices have a lot of dependencies in common, such as tonic, prost, tokio, serde etc. This may lead to
a drift in dependency versions, where microservice a depends on a different version of package x than micro-
service b. The solution is to put all microservices in a `workspace` and define the share dependencies as
a workspace dependency.

## Deployment of microservices

### Deploy a single microservice (`docker`)

The workspace structure of the microservices makes containerization a bit more complex, since a single service
cannot be build without access to the workspaces dependencies. This is why the `Dockerfile` is found on workspace
level. One must pass a `SERVICE_NAME` build-arg to docker build step to specify which microservice should be
containerized.

All microservices of the backend are deployed together with docker compose.

### Cache external dependencies between docker builds (`cargo-chef`)

This setup uses `cargo-chef` to split a container build of a microservice into two steps: i) compile all external
dependencies and ii) compile the microservice's binary. Step i) can be cached in most cases were only the service
code changes, which leads to optimized build times.

# Protos

Communication in the backend is done via `gRPC` which naturally uses `proto` file. `proto` files are compiled into
rust and typescript code.Therefore the backend can share request/response models with the frontend.

# Tracing

## Further reads
[Logging basics](https://heikoseeberger.de/2023-07-29-dist-tracing-1/)
[Tracing within a single service]( https://heikoseeberger.de/2023-08-18-dist-tracing-2/ )
[Inter service tracing]( https://heikoseeberger.de/2023-08-28-dist-tracing-3/ )
