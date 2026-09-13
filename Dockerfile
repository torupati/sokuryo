# ---- build stage ----
FROM rust:1.88-slim AS builder

WORKDIR /app
RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
# cache dependencies by building a dummy main first
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

COPY src ./src
RUN touch src/main.rs && cargo build --release

# ---- runtime stage ----
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/sokuryo /usr/local/bin/sokuryo

EXPOSE 3000
CMD ["sokuryo"]
