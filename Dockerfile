FROM rust:bookworm

ARG APP_NAME=api_ict

WORKDIR /srv/app

COPY . .

RUN cargo build --locked --release
RUN cp ./target/release/$APP_NAME /bin/server

EXPOSE 8000

CMD ["/bin/server"]
