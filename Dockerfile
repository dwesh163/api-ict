FROM rust:latest AS builder
WORKDIR /srv/app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --locked --release

FROM ubuntu:24.04
WORKDIR /app
RUN apt update && apt install -y --no-install-recommends libgcc-s1 libstdc++6 libc6 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /srv/app/target/release/api_ict /app/server
RUN chmod +x /app/server
USER 1000
EXPOSE 8000

CMD ["/app/server"]
