# syntax = devthefuture/dockerfile-x

ARG EXECUTABLE=core-archiver
INCLUDE Dockerfile.common

FROM mongo:6
WORKDIR /app
COPY --from=builder /app/core-archiver/target/release/core-archiver /usr/local/bin
ENTRYPOINT /usr/local/bin/core-archiver
