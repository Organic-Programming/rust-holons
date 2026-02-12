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
| `holons::transport` | `listen(uri)` — async URI-based listener factory (tcp, unix, stdio, mem) |
| `holons::serve` | `parse_flags()` — CLI arg extraction |
| `holons::identity` | `parse_holon(path)` — HOLON.md parser with serde |

## Test

```bash
cargo test
```
