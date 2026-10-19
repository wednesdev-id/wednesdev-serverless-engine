# Docker Deployment

Deploying **Wednes Serverless Engine** via Docker provides a clean, containerized operational boundary for hosting on Dokploy, Kubernetes, or standard Docker hosts.

---

## Production Dockerfile

Use a multi-stage build to compile `wednesd` with Rust and produce a minimal final runtime image based on `debian:bookworm-slim` or `alpine`:

```dockerfile
# Stage 1: Build binary
FROM rust:1.80-slim-bookworm AS builder

WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY . .
RUN cargo build --release --bin wednesd

# Stage 2: Minimal runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/*
RUN useradd -u 1000 -m -s /bin/bash wednes

WORKDIR /var/lib/wednes
COPY --from=builder /app/target/release/wednesd /usr/local/bin/wednesd

RUN chown -R wednes:wednes /var/lib/wednes
USER wednes

EXPOSE 8080 8443/udp

ENTRYPOINT ["wednesd", "run"]
CMD ["--registry", "/var/lib/wednes/registry.json", "--listen", "0.0.0.0:8080", "--log-json"]
```

---

## Docker Compose Setup

Run `wednesd` alongside a Traefik edge reverse proxy and persistent volume mounts:

```yaml
version: "3.8"

services:
  wednesd:
    image: wednesdev/wednes-engine:latest
    container_name: wednesd
    restart: unless-stopped
    ports:
      - "8080:8080"
      - "8443:8443/udp"
    volumes:
      - /data/wednes/registry.json:/var/lib/wednes/registry.json:ro
      - /data/wednes/artifacts:/var/lib/wednes/artifacts:ro
    environment:
      - RUST_LOG=info
    deploy:
      resources:
        limits:
          cpus: "2.0"
          memory: 1536M
        reservations:
          cpus: "0.2"
          memory: 128M
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.wednes.rule=Host(`functions.wednesdev.id`)"
      - "traefik.http.routers.wednes.entrypoints=websecure"
      - "traefik.http.routers.wednes.tls.certresolver=letsencrypt"
      - "traefik.http.services.wednes.loadbalancer.server.port=8080"

networks:
  default:
    name: wednes-network
```

---

## Security Hardening in Docker

When operating `wednesd` in container environments:
1. **Read-Only Root Filesystem:** Run with `--read-only` flag; mount only necessary temporary scratch directories as `tmpfs`.
2. **Drop All Capabilities:** Drop unnecessary Linux kernel capabilities with `cap_drop: [ALL]`.
3. **No Root User:** Always run as non-root UID (e.g. `1000`).
4. **Isolate Socket Mounts:** Never mount `/var/run/docker.sock` into the `wednesd` container.
