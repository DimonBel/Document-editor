# ── Builder stage ──────────────────────────────────────────────────────────
FROM rust:1.77-slim AS builder
WORKDIR /app

# Cache dependencies before copying source
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main(){}' > src/main.rs && cargo build --release
RUN rm -f target/release/deps/whiteboard_server*

# Build the real binary
COPY src ./src
RUN cargo build --release

# ── Runtime stage ──────────────────────────────────────────────────────────
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/whiteboard-server .

# Run as a non-root user. The `nobody` user is already present in
# the debian base image; we only need to make sure the binary is
# world-readable (it is by default after `cargo build --release`)
# and the working dir is writable for the user.
RUN chown nobody:nogroup /app

ENV HOST=0.0.0.0
ENV PORT=8080
EXPOSE 8080
USER nobody
CMD ["./whiteboard-server"]
