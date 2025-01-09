ARG RUST_VERSION=1.83

FROM rust:${RUST_VERSION} AS build

WORKDIR /usr/src/app

RUN --mount=type=bind,target=./ cargo fetch --locked

RUN --mount=type=bind,target=./ cargo build --release --target-dir /target

FROM debian:bookworm-slim AS final

COPY --from=build /target/release/s2d /bin/

EXPOSE 8080

ENTRYPOINT [ "/bin/s2d" ]