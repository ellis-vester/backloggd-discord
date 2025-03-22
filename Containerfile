FROM rust:1-slim-bullseye as builder
WORKDIR /usr/src/backloggd-discord
COPY Cargo.toml .
COPY Cargo.lock .
RUN mkdir ./src && echo 'fn main() { println!("Dummy!"); }' > ./src/main.rs

RUN cargo build --release

RUN rm -rf ./src
COPY ./src ./src
RUN touch -a -m ./src/main.rs
RUN cargo build --release

FROM debian:bullseye-slim

# curl is required for reqwest to work
# https://github.com/seanmonstar/reqwest/issues/2536
RUN apt-get update && apt-get install -y curl
RUN apt-get update && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/backloggd-discord/target/release/backloggd-discord /usr/local/bin/backloggd-discord
CMD ["backloggd-discord"]

