FROM rust:1.62.0-alpine3.16 as builder
RUN apk add alpine-sdk
RUN mkdir -p /src
WORKDIR /src
COPY ./ .
RUN cargo build --release

FROM registry.kci.rocks/build_images/k8s-helmfile-deploy
RUN apk add --no-cache libstdc++ gcompat && apk add --no-cache yq --repository=http://dl-cdn.alpinelinux.org/alpine/edge/community
COPY --from=builder /src/target/release/helmfile_checker /bin/helmfile_checker
ENV RUST_LOG=info
ENTRYPOINT ["/bin/helmfile_checker"]
