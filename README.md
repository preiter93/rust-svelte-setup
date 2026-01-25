# rust-svelte-setup

This project is an exploration in creating a standard setup for a microservice backend using Rust. The main focus is on backend architecture, simple CRUD operations, no event-driven architecture. The focus is on simplicity, type safety and testability.

# Architecture

See [ARCHITECTURE.md](./ARCHITECTURE.md).

# Where is it used so far?

This setup powers my personal website for tracking running data: [runaround.world](https://runaround.world) (feel free to give it a try, but its early stage - it only supports data from polar and strava at the moment). It works really well. Rust + Postgres delivers the performance you'd expect and in practice there's no need to optimize beyond just writing sane Rust code. So don't worry about a few clones here and there. I like the type safety that Rust provides, there are rarely any issues that I have to debug after it compiles. And if there are issues, tracing helps to track them down quickly.

# Similar Projects

There are a few similar projects from which I drew inspiration, however there weren't as many as I expected. Here are some of them:

- [rusve](https://github.com/mpiorowski/rusve)
- [rust-microservice-template](https://github.com/nkz-soft/rust-microservice-template)
- [rust-simple-event-driven-microservices](https://github.com/Jamesmallon1/rust-simple-event-driven-microservices)
