# syntax = devthefuture/dockerfile-x

ARG EXECUTABLE=core-database
INCLUDE Dockerfile.common

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/core-database/target/release/core-database /usr/local/bin
ENTRYPOINT /usr/local/bin/core-database
