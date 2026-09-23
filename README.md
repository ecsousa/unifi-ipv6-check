# Unifi IPv6 Check & Updater

A Rust-based utility that acts as a dynamic DNS (DDNS) updater and VPN configuration orchestrator for Unifi routers. 

This tool checks the public IPv6 address of a Unifi "server" router. If the IP address has changed (mismatches the Cloudflare AAAA DNS record), it automatically updates a specific VPN client configuration on a second Unifi "client" router to point to the new address, and simultaneously updates the Cloudflare DNS record.

## How it works

1. Resolves `DNS_NAME` for its current `AAAA` record.
2. Authenticates with the Unifi Server router and queries its WAN IPv6 address using the device's MAC address.
3. Compares the DNS-resolved IP against the router's current WAN IPv6.
4. If they differ, it authenticates with the Unifi Client router and updates the `wireguard_client_peer_ip` in the specified VPN Network configuration.
5. Updates the Cloudflare DNS `AAAA` record to reflect the new IP address.

## Configuration

The application is configured entirely via environment variables:

| Variable | Description |
|---|---|
| `DNS_NAME` | The hostname to resolve and update in Cloudflare. |
| `CF_ZONE_ID` | Cloudflare Zone ID. |
| `CF_APIKEY` | Cloudflare API Token (requires DNS edit permissions). |
| `SERVER_BASE_URL` | Base URL of the Unifi Server router (e.g., `https://192.168.1.1`). |
| `SERVER_USERNAME` | Username for the Unifi Server router. |
| `SERVER_PASSWORD` | Password for the Unifi Server router. |
| `SERVER_MAC_ADDRESS` | MAC address of the Unifi Server router device. |
| `CLIENT_BASE_URL` | Base URL of the Unifi Client router. |
| `CLIENT_USERNAME` | Username for the Unifi Client router. |
| `CLIENT_PASSWORD` | Password for the Unifi Client router. |
| `CLIENT_NETWORK_ID` | The internal Unifi Network ID of the VPN client configuration to update. |

## Usage

### Native execution

Build and run using `cargo`:

```bash
cargo run --release
```

By default, the application runs continuously, checking for IP changes every 5 minutes.

**Flags:**
- `-o`, `--once`: Run the logic only once and exit.
- `-v`, `--verbose`: Enable debug and trace logging.

Example:
```bash
cargo run --release -- --once --verbose
```

### Docker

You can use the provided `Dockerfile` to build and run the application in a container:

```bash
docker build -t unifi-ipv6-check .
docker run -d \
  -e DNS_NAME="your.hostname.com" \
  -e CF_ZONE_ID="..." \
  -e CF_APIKEY="..." \
  -e SERVER_BASE_URL="..." \
  -e SERVER_USERNAME="..." \
  -e SERVER_PASSWORD="..." \
  -e SERVER_MAC_ADDRESS="..." \
  -e CLIENT_BASE_URL="..." \
  -e CLIENT_USERNAME="..." \
  -e CLIENT_PASSWORD="..." \
  -e CLIENT_NETWORK_ID="..." \
  unifi-ipv6-check
```

## GitHub Actions

This repository includes GitHub Actions workflows for continuous integration and delivery. 
- Pushing to `main` builds the Docker image natively for AMD64 and ARM64 via `cross` and publishes to GitHub Container Registry (GHCR).
- Publishing a GitHub release or pushing a tag (e.g. `v1.0.0`) publishes multi-architecture binaries as release assets along with semantic Docker tags.
