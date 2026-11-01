# ADR-0002 — Reimplement without code reuse

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** engineering
- **Related:** `docs/LICENSE_ASSESSMENT.md`, `docs/RESEARCH_SOURCES.md`

## Context

We studied three sources:

- `girafic/penpot-converter` — no license declared; its core is copied from MPL-2.0 code.
- `penpot/penpot-exporter-figma-plugin` — MPL-2.0.
- `penpot/penpot` — MPL-2.0.

The product must be pure Rust and permissively licensed. Porting or translating the
exporter would carry MPL obligations into the translated files. Copying from the reference
converter is not permitted at all.

## Options

1. **Fork the reference converter** — fastest, but unlicensed, Node-based, and violates the
   pure-Rust rule.
2. **Translate the MPL exporter to Rust** — possible, but creates MPL-covered Rust files,
   adds a large translation surface, and inherits upstream design choices.
3. **Reimplement from documented behavior and observed formats** — slower initially, but
   keeps licensing clean and lets the architecture fit the report-first product.
4. **Call the exporter as a subprocess** — violates pure Rust and offline simplicity.

## Decision

Reimplement in Rust without reusing upstream code, tests, comments, or documentation text.
Use upstream projects as behavioral references and cite the facts they establish.

This is **not** a formal clean-room: the same engineers read upstream source. We state that
plainly rather than claiming a process we did not follow. What we guarantee is no copying or
translation of expression, documented provenance for facts, and license checks for every
dependency.

## Consequences

- Slower feature ramp; every mapping must be derived and tested.
- The architecture can be report-first, which the upstream parser is not.
- No MPL obligations, so dual MIT/Apache-2.0 is viable.
- Contributors must not paste upstream code. Review checks provenance.
- Before 1.0, counsel should review the process and the attribution files.
