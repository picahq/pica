# syntax = devthefuture/dockerfile-x

ARG EXECUTABLE=api
INCLUDE Dockerfile.common

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/api/target/release/api /usr/local/bin
ENTRYPOINT /usr/local/bin/api
