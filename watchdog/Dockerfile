# syntax = devthefuture/dockerfile-x

ARG EXECUTABLE=watchdog
INCLUDE Dockerfile.common

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/watchdog/target/release/watchdog /usr/local/bin
ENTRYPOINT /usr/local/bin/watchdog
