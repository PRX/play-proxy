FROM ekidd/rust-musl-builder:stable as builder
WORKDIR /play_proxy
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
ADD ./src ./src
RUN cargo build --release


FROM alpine:latest
LABEL org.prx.app="yes"

ARG APP=/usr/src/app

EXPOSE 3000

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN addgroup -S $APP_USER \
    && adduser -S -g $APP_USER $APP_USER

RUN apk update \
    && apk add --no-cache ca-certificates tzdata \
    && rm -rf /var/cache/apk/*

COPY --from=builder /play_proxy/target/x86_64-unknown-linux-musl/release/play_proxy ${APP}/play_proxy

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

CMD ["./play_proxy"]
