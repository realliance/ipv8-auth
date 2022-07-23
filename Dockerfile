FROM docker.io/rust:1.62-slim as BUILDER

WORKDIR /app

RUN apt-get update && apt-get install -y libpq-dev

ADD . .

RUN --mount=type=cache,target=/app/target cargo install --debug --locked --root install --path .

CMD ["/app/install/bin/ipv8-auth"]

FROM gcr.io/distroless/cc

COPY --from=BUILDER /usr/lib /usr/lib
COPY --from=BUILDER /lib /lib
COPY --from=BUILDER /app/install /app/install

CMD ["/app/install/bin/ipv8-auth"]
