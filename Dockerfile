FROM rust:1.76-slim AS build
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev libsqlite3-dev && rm -rf /var/lib/apt/lists/*
COPY . .
RUN cargo build --release -p magicmerlin -p magicmerlin-gateway

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 libsqlite3-0 && rm -rf /var/lib/apt/lists/*
RUN useradd -m -u 1000 merlin
USER merlin
WORKDIR /home/merlin
COPY --from=build /app/target/release/magicmerlin /usr/local/bin/magicmerlin
COPY --from=build /app/target/release/magicmerlin-gateway /usr/local/bin/magicmerlin-gateway
ENV MAGICMERLIN_GATEWAY_PORT=18789
EXPOSE 18789
ENTRYPOINT ["magicmerlin-gateway"]
CMD ["--serve", "18789", "--bind", "0.0.0.0"]
