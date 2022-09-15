FROM rust:alpine as builder
LABEL maintainer="yukselberkay <https://github.com/yukselberkay>"
RUN apk add --no-cache build-base

WORKDIR /usr/src/pmanager
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
RUN cargo install --path .

FROM alpine:3.12
LABEL author = "yukselberkay <https://github.com/yukselberkay>"
RUN addgroup -S pmanager && adduser -S -G pmanager pmanager

COPY --from=builder /usr/local/cargo/bin/pmanager /usr/local/bin/pmanager
USER pmanager
ENTRYPOINT [ "/usr/local/bin/pmanager" ]
