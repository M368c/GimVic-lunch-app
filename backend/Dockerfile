FROM rust:slim AS chef
WORKDIR /app
RUN cargo install cargo-chef

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*


RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release --bin backend

FROM debian:bookworm-slim AS runner
WORKDIR /app

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/backend ./backend

ENV TZ=Europe/Ljubljana
EXPOSE 3000

CMD ["./backend"]