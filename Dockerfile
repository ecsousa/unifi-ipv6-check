FROM rust:1.80 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/unifi-ipv6-check /app/unifi-ipv6-check

ENV DNS_NAME=""
ENV CF_ZONE_ID=""
ENV CF_APIKEY=""
ENV SERVER_BASE_URL=""
ENV SERVER_USERNAME=""
ENV SERVER_PASSWORD=""
ENV SERVER_MAC_ADDRESS=""
ENV CLIENT_BASE_URL=""
ENV CLIENT_USERNAME=""
ENV CLIENT_PASSWORD=""
ENV CLIENT_NETWORK_ID=""

ENTRYPOINT ["/app/unifi-ipv6-check"]
