FROM lukemathwalker/cargo-chef:latest AS chef
WORKDIR /bszet-mind

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /bszet-mind/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin bszet-mind

# We do not need the Rust toolchain to run the binary!
FROM debian:buster-slim AS runtime
WORKDIR /bszet-mind
COPY --from=builder /bszet-mind/target/release/bszet-mind /usr/local/bin
ENTRYPOINT ["/usr/local/bin/bszet-mind"]
