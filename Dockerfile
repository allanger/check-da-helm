FROM rust:1.70.0-alpine3.18 as builder
WORKDIR /src
RUN apk update && apk add --no-cache gcc musl-dev
COPY ./ .
RUN rustup default nightly && rustup update
RUN cargo build --release --jobs 2 -Z sparse-registry 

FROM alpine:3.18
COPY --from=builder /src/target/release/cdh /bin/cdh
WORKDIR /workdir
ENTRYPOINT ["/bin/cdh"]
