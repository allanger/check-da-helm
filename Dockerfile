FROM rust:1.66.1-alpine3.17 as builder
WORKDIR /src
RUN apk update && apk add --no-cache gcc musl-dev
COPY ./ .
RUN cargo build --release --jobs 2

FROM alpine:3.17.1
COPY --from=builder /src/target/release/cdh /bin/cdh
WORKDIR /workdir
ENTRYPOINT ["/bin/cdh"]
