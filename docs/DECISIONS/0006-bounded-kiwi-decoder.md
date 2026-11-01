# ADR-0006 — Bounded in-house Kiwi decoder

- **Status:** Accepted
- **Date:** 2026-11-01
- **Supersedes:** the decoding choice in ADR-0004
- **Related:** `docs/FIG_FORMAT.md`, `docs/RISKS.md` §R9

## Context

ADR-0004 initially selected the MIT `kiwi-schema 0.2.1` crate for schema and message
decoding. During M1 implementation, inspection of its decoding path showed:

```rust
let len = bb.read_var_uint()? as usize;
let mut array = Vec::with_capacity(len);
```

The array length is read from the untrusted message and passed directly to
`Vec::with_capacity`. A five-byte varint can request billions of elements before any data
is read. That allocation cannot be bounded or recovered from by the caller, and a failed
allocation aborts the process rather than returning an error. Wrapping the crate is
therefore not sufficient for the "never allocate unboundedly from file-supplied lengths"
requirement.

Other options:

1. Keep the crate and accept the risk. Rejected: the input is untrusted and the failure mode
   is an abort.
2. Vendor and patch the crate. Possible under MIT, but adds a large third-party surface
   whose upstream is not maintained for our limits.
3. Write a bounded schema/message decoder in-house against the documented Kiwi format.
   More code, but every length and depth check is explicit and testable.

## Decision

Implement the Kiwi schema and message decoder in `crates/fig-reader/src/kiwi/`, with:

- A checked `Cursor` for all primitive reads.
- Explicit limits for definitions, fields, names, arrays, blobs, nodes, depth, and zstd
  window.
- `Vec::new()` growth instead of preallocating from untrusted counts.
- Byte arrays handled as slices/ranges, not one allocation per byte.
- Unknown enum values preserved as `UNKNOWN(n)` and reported, never treated as a crash.
- Byte-array blob ranges into the decompressed message buffer, so assets are not copied
  twice.

The MIT crate remains a research reference for the wire format, not a dependency.
`flate2`/`miniz_oxide` and `ruzstd` remain for decompression.

## Evidence

- Parses four real fixtures: `circle.fig` (550 definitions), `OpenFigs.fig`, and two
  vector-network files (627 definitions each).
- 14 integration tests and 13 unit tests pass.
- Hostile inputs (truncated, corrupt chunk length, zip-slip, entry-count bomb, compression
  ratio bomb) return typed errors without panicking.
- `cargo fuzz run read_fig --sanitizer none -- -max_total_time=60`:
  **829,577 executions**, 61 seconds, no crashes, 61 MB peak RSS.

## Consequences

- No external Kiwi parser in the runtime dependency graph; one less supply-chain surface.
- More code to maintain: the schema parser, message decoder, and their limits.
- The decoder must be kept honest by fixtures and fuzzing on every change; a fixture from a
  new Figma version is the regression test.
- If a future Figma version introduces a Kiwi primitive we do not implement, it produces a
  typed decode error naming the type, not a silent misread.
