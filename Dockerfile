FROM rust:1.78-slim-buster as builder

WORKDIR /app-build

COPY . .

RUN apt update && apt install -y libssl-dev pkg-config  && cargo build --release

FROM debian:buster-slim

WORKDIR /app

RUN apt update && apt install -y libssl-dev pkg-config && apt clean 

COPY --from=builder /app-build/target/release/github-oauth-userinfo /app/github-oauth-userinfo


CMD ["./github-oauth-userinfo"]
