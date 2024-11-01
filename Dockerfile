FROM rust:bookworm as builder

ARG APP_NAME=api_ict

WORKDIR /srv/app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --locked --release

FROM alpine:3.20

RUN apk add --no-cache libgcc libstdc++
COPY --from=builder /srv/app/target/release/$APP_NAME /bin/server

RUN chmod +x /bin/server

EXPOSE 8000

CMD ["/bin/server"]
