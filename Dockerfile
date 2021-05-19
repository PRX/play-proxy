FROM rust as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin play_proxy

FROM rust as runtime
WORKDIR /app
COPY --from=builder /app/target/release/play_proxy /usr/local/bin
ENTRYPOINT ["/usr/local/bin/play_proxy"]
