# syntax = devthefuture/dockerfile-x

ARG EXECUTABLE=archiver
INCLUDE Dockerfile.common

FROM mongo:6
WORKDIR /app
COPY --from=builder /app/archiver/target/release/archiver /usr/local/bin
ENTRYPOINT /usr/local/bin/archiver
