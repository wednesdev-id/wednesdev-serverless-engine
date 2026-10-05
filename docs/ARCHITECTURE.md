# Architecture

## Request path

`HTTP gateway → scheduler → RuntimeBackend → Wasmtime component`

- `wednes-cli`: `wednesd` executable, direct component loading, registry deploy/run.
- `wednes-core`: request/response types, strict manifest validation, JSON file registry.
- `wednes-gateway`: Axum routes `/fn/:id` and `/fn/:id/*path`; global and per-function semaphore admission, rejecting saturation with HTTP 429.
- `wednes-runtime`: backend contract; `wednes-runtime-wasmtime`: compilation, typed export validation, per-invocation stores, WASI context, limits and epoch interruption.
- `wit/`: `wednes:function@0.1.0` contract; `sdk/rust` and `examples/`: guest-side bindings and examples.

Each invocation uses fresh guest state; compiled components are reused within process. Linear-memory limits do not bound total host RSS, response copies, all allocations, or compiled code. Epoch interruption limits guest execution, not an end-to-end request deadline.

## Registry walkthrough

First complete README build steps. Keep manifest, component, and registry in same directory: deploy resolves artifact relative to manifest; run resolves it relative to registry. Artifact must be a basename, not a path. For reproducible hello setup:

```bash
cp docs/hello.toml target/hello.toml
./target/release/wednesd deploy --manifest target/hello.toml --registry target/registry.json
./target/release/wednesd run --registry target/registry.json --listen 127.0.0.1:8080
```

In another terminal, POST to `http://127.0.0.1:8080/fn/hello` as in README. Stop with Ctrl-C. Deployment validates manifest, compiles component, checks typed ABI, then replaces registry metadata via rename. Existing metadata is retained if validation fails. This is not transactional artifact storage: do not modify deployed component files concurrently, and serialize deploy writers. Deployment does not copy artifacts into registry directory.

## Current boundaries

- No persistent deploy compilation cache. Deploy's compiled component dies with deploy process; run compiles/loads source again. Old M3 notes claiming managed persistent caching are superseded by this document.
- No live reload: restart daemon to consume changed registry.
- No control plane, distributed scheduler, authentication, TLS termination, billing, or durable guest state.
- Environment, filesystem, and outbound HTTP enablement are unsupported; logging/clock capability configuration is rejected. Do not infer configurable capability enforcement from proposed fields.
- Registry is trusted local metadata, not a concurrent multi-user database or a content-addressed artifact store.

See [security](SECURITY.md) for trust boundaries and deployment precautions.
