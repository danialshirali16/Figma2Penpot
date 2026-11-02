# 0011 — Bounded compatibility fixes for private Figma files

Date: 2026-11-02
Status: Accepted

## Context

Five privately supplied Figma files exposed failures not covered by the small public
fixtures: sections pruned their descendants, embedded masters were unavailable to instances,
empty fill caches hid valid stroke outlines, and output validation stopped at 10,000 entries.
Real Penpot imports additionally rejected non-finite layout gaps, invalid token names, and
unsupported blend identifiers. One container had non-finite source dimensions.

The requested outcome was conversion without issues. Import acceptance and lossless fidelity
are separate claims; the latter cannot be inferred from acceptance or a validator pass.

## Options

- Remove limits or suppress diagnostics: rejected; inputs remain untrusted and losses must be visible.
- Preserve only placeholder instance bounds or prune sections: rejected; this loses visible content.
- Reuse the reference converter/runtime: rejected under the existing Rust and licensing rules.
- Add bounded Rust mappings and retain explicit limitations: chosen.

## Decision

- Map sections to unclipped boards, with an approximation event. Decode frame clipping and
  supported clipping overrides explicitly.
- Build the closure of visible/referenced component definitions in dependency order. Restore
  required embedded masters on an additional Imported components page, retaining their links.
  Break cycles deterministically with diagnostics; bound traversal and expanded/cache shapes.
- Resolve override addresses through nested instance keys, including imported-library keys.
  Keep unmapped override field names in a dedicated diagnostic. Cached child constraints and
  nontrivial instance transforms remain Partial and are reported.
- Translate path endpoints and control points with copied component shapes. Ignore known empty
  fill caches when a valid stroke outline exists; missing or corrupt geometry remains an error.
- Recover unknown container bounds only from finite converted children, retaining the uncertainty
  in the event. Unknown layout metrics use a recorded finite fallback and mapped distribution.
- Escape incompatible token names using `_uHEX_`, disambiguate collection-local collisions, and
  use the same resulting names for converted bindings. Original names remain in events.
- Validate target token names, layout numbers, and blend identifiers. Approximate linear burn/dodge
  with multiply/screen, reporting each affected element; reverse fill/stroke stacks for Penpot.
- Default limits: 250,000 expanded/cache shapes; archive validation 300,000 entries,
  64 MiB per entry, 512 MiB total. Expose limits through `ConvertOptions`/`ValidationLimits`.

The reference README was consulted at commit
[`c29242f60ebe3579265260894b40e9bcfe54ffb3`](https://github.com/girafic/penpot-converter/blob/c29242f60ebe3579265260894b40e9bcfe54ffb3/README.md).
Its documented format facts guided independent Rust mappings. GitHub still reported no
repository license on 2026-11-02. No upstream implementation or documentation text was copied
into the product; the research copy stays outside this repository (ADR-0002 still applies).

## Consequences

- All five files import into local Penpot 2.18.5-RC1; this verifies import compatibility.
- Overall fidelity remains Partial, with event-based warnings, unsupported concepts, and manual
  review still required. No fidelity percentage or blanket lossless claim is made.
- The private files, converted designs, reports, and import evidence stay outside tracked fixtures.
  Regression tests use existing redistributable fixtures modified in memory and synthetic models.
- Native Windows/macOS visual checks, exhaustive visual comparison, edit/reopen persistence,
  fuzzing of the new component cases, and license/advisory tooling remain separate checks.
