FROM lukemathwalker/cargo-chef:latest-rust-latest AS chef
WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update -y && apt-get install -y musl-tools
RUN cargo build --verbose --locked --release --target x86_64-unknown-linux-musl

FROM rust:slim-bookworm AS runtime
WORKDIR app
COPY --from=builder /app/target/release/cargo-msrv /usr/local/bin
ENTRYPOINT ["cargo-msrv", "msrv"]
