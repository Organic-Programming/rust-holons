---
# Cartouche v1
title: "rust-holons — Rust SDK for Organic Programming"
author:
  name: "B. ALTER"
  copyright: "© 2026 Benoit Pereira da Silva"
created: 2026-02-12
revised: 2026-02-12
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
| `holons::transport` | `listen(uri)`, `parse_uri(uri)`, `scheme(uri)` |
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

## Test

```bash
cargo test
```
