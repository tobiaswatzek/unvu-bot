FROM rust:1.67 as buildbase
WORKDIR /usr/src/unvu-bot
RUN apt-get update && apt-get install -y libopus-dev
RUN cargo install cargo-chef --locked

FROM buildbase AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM buildbase AS builder
COPY --from=planner /usr/src/unvu-bot/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim AS runtime
RUN apt-get update && apt-get install -y libopus-dev && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/unvu-bot /usr/local/bin/unvu-bot
ENV UNVU_BOT_SECRET=
ENV GUILD_ID=
ENV RUST_LOG=warn
CMD ["unvu-bot"]
