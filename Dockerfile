FROM rust:1.82 as builder

WORKDIR /app
# Remove Cargo.lock if present to force regeneration
RUN rm -f Cargo.lock
COPY Cargo.toml ./
# Copy Cargo.lock if it exists, but don't fail if missing
COPY Cargo.lock* ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/esl-learning-platform /app/
COPY static ./static

EXPOSE 3000

CMD ["./esl-learning-platform"]
