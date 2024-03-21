FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev protoc

WORKDIR /build
COPY ./ballsd .

RUN cargo fetch --locked
RUN cargo install --locked

FROM scratch 

COPY --from=builder /usr/local/cargo/bin/ballsd /

ENTRYPOINT [ "ballsd" ]
