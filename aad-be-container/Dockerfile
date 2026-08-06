FROM rust:1.85-alpine as builder

RUN apk add --no-libc-dev build-base musl-dev pkgconfig openssl-dev

WORKDIR /usr/src/aad-be-container
COPY Cargo.toml ./
COPY migrations ./migrations
COPY src ./src

RUN cargo build --release

FROM alpine:3.20

RUN apk add --no-cache ca-certificates libgcc

WORKDIR /app
COPY --from=builder /usr/src/aad-be-container/target/release/aad-be-container /app/aad-be
COPY --from=builder /usr/src/aad-be-container/migrations /app/migrations

EXPOSE 8080

ENTRYPOINT ["/app/aad-be"]
CMD ["serve"]
