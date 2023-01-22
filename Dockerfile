FROM lukemathwalker/cargo-chef:latest AS chef
WORKDIR /bszet-mind

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

ENV USER=bszet-mind
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

COPY --from=planner /bszet-mind/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin bszet-mind

# We do not need the Rust toolchain to run the binary!
FROM debian:slim AS runtime

COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /bszet-mind
COPY --from=builder /bszet-mind/target/release/bszet-mind /usr/local/bin

USER bszet-mind:bszet-mind
ENTRYPOINT ["/usr/local/bin/bszet-mind"]
