FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /build
COPY . .

RUN cargo fetch --locked
RUN cargo install --locked --path ballsd

FROM scratch 

COPY --from=builder /usr/local/cargo/bin/ballsd /
ENV PATH /

ENTRYPOINT [ "ballsd" ]
