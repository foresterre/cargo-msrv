FROM lukemathwalker/cargo-chef:latest-rust-1-alpine3.22 AS chef
WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --verbose --locked --release

FROM alpine:3.22 AS runtime
WORKDIR app
COPY --from=builder /app/target/release/cargo-msrv /usr/local/bin
ENTRYPOINT ["cargo-msrv", "msrv"]
