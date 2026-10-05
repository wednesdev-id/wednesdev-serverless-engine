# Security

## Reporting

Do not publish exploit details, credentials, or malicious artifacts in public issues. Use GitHub's private vulnerability reporting for [wednesdev-id/wednesdev-serverless-engine](https://github.com/wednesdev-id/wednesdev-serverless-engine/security/advisories/new) if enabled. If unavailable, open a public issue requesting a private contact **without vulnerability details**. No response-time or supported-release guarantee is currently offered; project is alpha.

Include affected revision, OS/architecture, Wasmtime version, bounded reproduction, impact, and whether the same daemon PID survives. Coordinate disclosure before posting details.

## Trust model

Guest WASM is untrusted; the operator, host OS, daemon binary, build toolchain, adapter, registry, and artifact directory are trusted. Typed ABI validation is necessary but does not make arbitrary guest code safe. Wasmtime isolation is not a guarantee against runtime vulnerabilities or all resource exhaustion.

Guests receive no host filesystem preopens, inherited environment/secrets, or outbound HTTP capability by default. They cannot invoke arbitrary host executables or access host devices through exposed APIs. Capability enablement is not generally implemented: unsupported manifest fields/requests must be rejected, not treated as policy enforcement. Clock/logging configuration is unsupported; do not assume a total absence of nondeterministic WASI facilities.

Per-invocation memory limits and epoch deadlines bound specific guest resources, not total daemon RSS or wall-clock request duration. Global/per-function admission rejects saturation with 429; it is not authentication, abuse prevention, or a host resource budget. Compilation, response copies, HTTP buffering, caches, and runtime bugs can still exhaust host resources.

## Operator precautions

- Bind to localhost by default. Before public exposure, use an authenticated TLS reverse proxy with request-size limits, timeouts, and rate limits.
- Run as an unprivileged user with OS resource limits and no unnecessary secrets. Keep Wasmtime and dependencies patched.
- Keep registry and artifacts writable only by trusted operators. Never deserialize attacker-controlled native compiled artifacts.
- Serialize deploy writers. Registry rename does not atomically publish artifact contents, prevent symlink substitution, or provide multi-writer transactions.
- Enforce client-side deadlines and test same-process recovery after loops, memory exhaustion, malformed artifacts, and traps.
- Verify adapter download checksum in `scripts/build-fixtures.sh`; review dependency/tool upgrades before adopting them.

No persistent deploy compilation cache exists. Do not interpret deploy-time compilation as a guarantee that restart avoids compilation, or treat successful smoke tests as full V0 acceptance.
