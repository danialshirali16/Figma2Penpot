# ADR-0004 — `.fig` decoding: `kiwi-schema` + pure-Rust compression

- **Status:** Superseded by ADR-0006 for the decoder choice
- **Date:** 2026-11-01
- **Deciders:** engineering
- **Related:** `docs/FIG_FORMAT.md`, `docs/RISKS.md`

## Context

A `.fig` payload is:

```text
fig-kiwi header -> deflateRaw Kiwi schema -> zstd (or deflate) Kiwi message
```

The schema is embedded and self-describing. We need a decoder that is small, auditable,
pure Rust, and safe on untrusted input.

Options considered:

1. **Use the Rust `kiwi-schema` crate (MIT, author of Kiwi)** for schema and value decoding,
   plus `flate2`/`miniz_oxide` and `ruzstd`. Wrap it with our own limits and typed layer.
2. **Write a purpose-built Kiwi decoder** from the format description.
3. **Use the C `zstd` binding** for decompression.
4. **Reuse the JavaScript `kiwi-schema`/`openfig-core`** — forbidden (no JS runtime).

## Decision

~~Use `kiwi-schema 0.2.1` (MIT) for schema and message decoding, `flate2` with the
`miniz_oxide` backend for raw deflate, and `ruzstd` (MIT, pure Rust) for zstd.~~

**Superseded by ADR-0006.** The crate allocates arrays directly from untrusted counts; the
implemented decoder is in-house and bounded. `flate2`/`miniz_oxide` and `ruzstd` remain.

The crate was treated as untrusted-input code:

- Validate all sizes and counts before calling it.
- Cap schema/message decompressed size, chunk count, node count, blob count, and depth.
- Fuzz the wrapper and the crate together.
- If fuzzing shows an unbounded allocation, panic, or unenforceable limit, **vendor a
  patched MIT copy** or replace it with an in-house decoder. Do not ship a known-unsafe
  dependency.

## Evidence

A scratch Rust probe decoded `circle.fig` and `OpenFigs.fig` end to end:

- 550 schema definitions; root `Message` at definition index 404 with 44 fields.
- 5 nodes / 3 blobs and 21 blobs respectively.
- Truncated input returned a typed error, not a panic.
- Compression magic matched zstd; schema inflated with raw deflate.

The probe is research code, not product code.

## Consequences

- No JavaScript runtime and no C zstd binding are needed.
- `kiwi-schema` has no resource limits; the wrapper carries that responsibility.
- The crate is from 2023. Kiwi is stable, but we pin it and test against real files.
- If the fallback is triggered, the vendored copy must keep the MIT notice and be tracked in
  `THIRD_PARTY_LICENSES`.
