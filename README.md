# rust-svelte-setup

This project is an exploration in creating a standard setup for a microservice backend using Rust. The main focus is on backend architecture, simple CRUD operations, no event-driven architecture. The focus is on simplicity, type safety and testability.

# Architecture

Check out the [ARCHITECTURE.md](./ARCHITECTURE.md) for details.

# How to run

1. Copy `.env.example` to `.env` and adjust as needed.
2. Generate code and Dockerfiles:
   ```
   just generate
   ```
3. Build and deploy the backend:
   ```
   just build-services
   just deploy
   ```
4. To run the app locally (in the `app` directory):
   ```
   npm run dev -- --open
   ```
   Or build and deploy the app:
   ```
   just build-app
   just deploy-app
   ```

# But now be real, how does it compare to go?

I use go professionally, so I think I can give a bit of perspective. The tldr is: for large software projects I’d still choose go for the majority of services, but I’d definitely consider Rust for performance-critical parts (see this good read: https://engineering.grab.com/counter-service-how-we-rewrote-it-in-rust). So having a standard Rust setup in the toolkit is a win. For a hobby project like this one? I just prefer writing Rust. Its like solzing puzzles for me.

**What I love about Rust:**
- I just love the language more than Go. It’s more expressive and I feel good if I manage to write a nice functional style map or find a good use case for traits.
- Type safety. In Go it’s easy to forget passing values to structs and let’s be honest, who creates explicit constructors for everything?
- Performance: blazingly fast. But have I benchmarked? No. And oes it matter for my app with 1 user (me)? Also no. 
- Nil pointer exception: In Go it’s just a tad too easy to get a nil pointer exception and crash your microservice. Want to access a nested proto struct but haven’t checked the parent for nil? Boom...
- Compile with features: It’s nice to use features to gate testutils behind a service. In Go, it’s not straightforward to share testutils without polluting the public API between services.
- Error handling: I don’t mind Go’s verbosity, but Rust has more batteries here with `anyhow` and `thiserror`. It just clicks more for me even though I haven’t fully found my groove.
- No garbage collection: Just one problem less to care for.

**The negatives:**
- The big one is compile time/docker time. Rebuilding a full service from scratch in Docker on a mac can take up to 10 minutes. Want to parallelize this over 10 microservices? Your memory is killed. I put a lot of effort into optimizing caching, using cargo-chef, fixing cargo-chef, autogenerating optimal Dockerfiles (see architecture). But here Go just wins, by a margin that’s not even fun. How does it compile so fast? Maybe I just need to crank some compiler flags in Rust, but I haven’t gotten around to that.
- Table testing is a bit cumbersome in Rust. I use rstest and really like it, but it’s macro-based, which always breaks my formatting in nvim...
- gRPC gateway: I thought this was a standard gRPC thing. Was surprised Rust doesn’t have a good gRPC gateway. Maybe tonic adds one at some point? (https://github.com/hyperium/tonic/issues/332)
- HTTP/gRPC middleware: Took me quite some time to write gRPC middleware in Rust. That’s a lot easier in Go, but once you figure out the Rust/tower way, it’s kinda fun.
- I like how easy it is to onboard new people in Go while in Rust I’d probably spend days explaining generics, lifetimes, async traits and would fumble most of the explanations. What's that Pin thing again?

# Where is it used so far?

A backend with a similar setup to this one powers my personal website for tracking running data: [runaround.world](https://runaround.world) (feel free to give it a try, but its early stage - it only supports data from polar and strava at the moment). It works really well. Rust + Postgres delivers the performance you'd expect and in practice there's no need to optimize beyond just writing sane Rust code. So don't worry about a few clones here and there. I like the type safety that Rust provides, there are rarely any issues that I have to debug after it compiles. And if there are issues, tracing helps to track them down quickly.

# Similar Projects

There are a few similar projects from which I drew inspiration, however there weren't as many as I expected. Here are some of them:

- [rusve](https://github.com/mpiorowski/rusve)
- [rust-microservice-template](https://github.com/nkz-soft/rust-microservice-template)
- [rust-simple-event-driven-microservices](https://github.com/Jamesmallon1/rust-simple-event-driven-microservices)
