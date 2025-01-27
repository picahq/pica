# syntax = devthefuture/dockerfile-x

ARG EXECUTABLE=database
INCLUDE Dockerfile.common

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/database/target/release/database /usr/local/bin
ENTRYPOINT /usr/local/bin/database
