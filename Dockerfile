FROM rust:1.99.1 AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    musl-tools \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-linux-musl

RUN echo "mcp:x:10001:10001:mcp:/nonexistent:/bin/false" > /etc/passwd_mcp

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --target x86_64-unknown-linux-musl --bin forgejo-mcp

FROM scratch

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder /etc/passwd_mcp /etc/passwd
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/forgejo-mcp /forgejo-mcp

USER 10001:10001
EXPOSE 8000

ENTRYPOINT ["/forgejo-mcp"]
