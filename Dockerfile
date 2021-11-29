FROM ekidd/rust-musl-builder:stable as builder
ADD --chown=rust:rust . ./
RUN cargo build --release

FROM alpine:latest
LABEL org.prx.app="yes"
EXPOSE 3000

RUN apk update \
    && apk add --no-cache ca-certificates tzdata \
    && rm -rf /var/cache/apk/*

COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/play_proxy /usr/local/bin/play_proxy

CMD ["/usr/local/bin/play_proxy"]
