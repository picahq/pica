# syntax = devthefuture/dockerfile-x

ARG EXECUTABLE=core-watchdog
INCLUDE Dockerfile.common

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/core-watchdog/target/release/core-watchdog /usr/local/bin
ENTRYPOINT /usr/local/bin/core-watchdog
