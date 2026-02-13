---
# Cartouche v1
title: "rust-holons — Rust SDK for Organic Programming"
author:
  name: "B. ALTER"
  copyright: "© 2026 Benoit Pereira da Silva"
created: 2026-02-12
revised: 2026-02-13
lang: en-US
origin_lang: en-US
translation_of: null
translator: null
access:
  humans: true
  agents: false
status: draft
---
# rust-holons

**Rust SDK for Organic Programming** — transport, serve, and identity
utilities for building holons in Rust.

## API surface

| Module | Description |
|--------|-------------|
| `holons::transport` | `listen(uri)`, `listen_stdio()`, `dial_tcp(uri)`, `dial_unix(uri)`, `parse_uri(uri)`, `scheme(uri)` |
| `holons::serve` | `parse_flags(args)` |
| `holons::identity` | `parse_holon(path)` |

## Transport URI support

Recognized:

- `tcp://`
- `unix://`
- `stdio://`
- `mem://`
- `ws://`
- `wss://`

Current runtime listeners:

- native async listeners: `tcp://`, `unix://`
- in-process test listener: `mem://`
- `ws://` / `wss://` currently provide normalized listener metadata in `transport::Listener::Ws`

## Parity Notes vs Go Reference

Implemented parity:

- URI parsing and listener dispatch semantics
- Native runtime listeners for `tcp://`, `unix://`, and `mem://`
- Native dialers for `tcp://` and `unix://`
- `listen_stdio()` async stdin/stdout transport wrapper
- Standard serve flag parsing
- HOLON identity parsing

Not currently achievable in this minimal Rust core (justified gaps):

- `ws://` / `wss://` runtime listener parity:
  - No official tonic WebSocket server transport for standard gRPC HTTP/2 framing.
  - Exposed as metadata only.
- gRPC `Dial("ws://...")` / `Dial("wss://...")`:
  - The tonic stack has no official HTTP/2-over-WebSocket client connector in core crates.
  - A safe implementation requires a custom bridge/proxy layer and is intentionally out of scope for this minimal core.
- Full Go-style transport-agnostic gRPC client helpers (`Dial`, `DialStdio`, `DialMem`, `DialWebSocket`):
  - Rust currently exposes direct transport primitives (`dial_tcp`, `dial_unix`, `listen_stdio`) and still needs a dedicated tonic adapter layer for equivalent high-level helpers.

## Test

```bash
cargo test
```
