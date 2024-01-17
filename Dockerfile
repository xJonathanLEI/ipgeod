FROM rust:alpine AS build

RUN apk add libc-dev

WORKDIR /src
COPY . /src

RUN cargo build --release

FROM alpine

LABEL org.opencontainers.image.source=https://github.com/xJonathanLEI/ipgeod

COPY --from=build /src/target/release/ipgeod /usr/bin/

ENTRYPOINT [ "ipgeod" ]
