FROM rust:bookworm as builder
ARG APP_NAME=api_ict
WORKDIR /srv/app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --locked --release

FROM alpine:3.20
RUN apk add --no-cache libgcc libstdc++ libc6-compat
WORKDIR /app
COPY --from=builder /srv/app/target/release/$APP_NAME /app/server
RUN chmod +x /app/server
EXPOSE 8000
USER 1000
CMD ["/app/server"]
