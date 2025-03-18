FROM rust:1-slim-bullseye as builder
WORKDIR /usr/src/backloggd-discord
COPY ./src ./src
COPY Cargo.toml .
COPY Cargo.lock .

RUN ls -la

RUN cargo install --path .

RUN ls -la 

FROM debian:bullseye-slim
RUN apt-get update && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/backloggd-discord /usr/local/bin/backloggd-discord
CMD ["backloggd-discord"]

