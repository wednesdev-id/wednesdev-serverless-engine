# HTTP/3 QUIC Protocol

**Wednes Serverless Engine** includes native, first-class support for **HTTP/3** and the **QUIC** transport protocol.

While traditional serverless gateways rely solely on TCP-based HTTP/1.1 and HTTP/2, `wednesd` embeds an asynchronous QUIC engine powered by [`quinn`](https://github.com/quinn-rs/quinn) and [`rustls`](https://github.com/rustls/rustls).

---

## Why HTTP/3 for Serverless?

Serverless workloads are inherently bursty, frequently invoked over mobile networks, and sensitive to network overhead:

### 1. Zero-RTT Connection Resumption
On repeated requests from the same client, HTTP/3 allows sending application payloads in the very first packet (**0-RTT**), eliminating TCP handshake latency and reducing end-to-end round-trip time.

### 2. Elimination of Head-of-Line (HoL) Blocking
In HTTP/2, all multiplexed streams share a single TCP connection. If a single packet is dropped, TCP halts all streams until that packet is retransmitted. QUIC multiplexes streams over independent UDP channels: packet loss on one stream does not stall unrelated function invocations.

### 3. Connection Migration
When mobile or edge clients switch networks (e.g., from Wi-Fi to 5G cellular), QUIC connections stay alive using Connection IDs instead of IP/port 4-tuples, ensuring uninterrupted API calls without renegotiating TLS.

---

## Protocol Comparison

| Capability | HTTP/1.1 | HTTP/2 | HTTP/3 (QUIC) |
|:---|:---:|:---:|:---:|
| **Underlying Transport** | TCP | TCP | UDP |
| **TLS Handshake** | Separate step (1-2 RTT) | Integrated with TLS 1.2/1.3 | Integrated with TLS 1.3 (1 RTT / 0-RTT) |
| **Stream Multiplexing** | No (Head-of-Line blocking) | Yes (Application-level) | Yes (Transport-level, independent streams) |
| **Packet Loss Resilience** | Stalls entire connection | Stalls entire connection | Only affects the specific degraded stream |
| **Connection Migration** | Breaks on IP change | Breaks on IP change | Seamlessly survives network handoffs |

---

## Architecture in `wednesd`

In `wednesd`, the QUIC engine runs concurrently with the standard TCP listener:

```
                      Client Request
                            │
            ┌───────────────┴───────────────┐
            ▼ (TCP)                         ▼ (UDP)
      TCP Listener (8080)             QUIC Listener (8443)
       HTTP/1.1 & HTTP/2               HTTP/3 (via quinn)
            │                               │
            └───────────────┬───────────────┘
                            ▼
                    Shared axum Router
                    (Route: /fn/:id)
                            │
                            ▼
                    Wasmtime Execution
```

### Self-Signed & Custom TLS Certificates
When started with the `--http3` flag, `wednesd`:
1. Inspects TLS configuration. If no certificates are provided, it generates an in-memory, self-signed TLS certificate via `rcgen` valid for `localhost`.
2. Configures `rustls` server config with the ALPN protocols `["h3", "h3-29"]`.
3. Binds the UDP socket on the specified address.

---

## Enabling HTTP/3

### Starting the Daemon

```bash
# Start wednesd with HTTP/3 QUIC enabled on port 8443
wednesd run \
  --registry ./registry.json \
  --listen 0.0.0.0:8443 \
  --http3
```

Log output:
```text
2026-10-19T10:00:00Z  INFO wednesd: Generating self-signed TLS certificate for HTTP/3 QUIC on 0.0.0.0:8443
2026-10-19T10:00:00Z  INFO wednesd: wednesd HTTP gateway listening on 0.0.0.0:8443
```

---

## Testing HTTP/3 Invocations

You can verify HTTP/3 connectivity using modern `curl` compiled with nghttp3/quiche:

```bash
# Test with HTTP/3 directly (-k skips self-signed cert validation)
curl --http3-only -k -X POST https://127.0.0.1:8443/fn/hello \
     -H "Content-Type: application/json" \
     -d '{"ping": "pong"}'
```

Output:
```json
{"status": 200, "message": "Pong over HTTP/3 QUIC"}
```

Alternatively, use HTTPie with HTTP/3 support:

```bash
http --http3 --verify=no POST https://127.0.0.1:8443/fn/hello ping=pong
```
