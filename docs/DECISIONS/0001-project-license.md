# ADR-0001 — Project license: MIT OR Apache-2.0

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** project owner (per brief), engineering
- **Related:** `docs/LICENSE_ASSESSMENT.md`

## Context

The project is open source and must remain usable by individuals and companies. No upstream
code is reused: the reference converter is unlicensed, and the official exporter and Penpot
are MPL-2.0. All planned dependencies are permissively licensed.

## Options

1. **MIT only** — simplest, widely understood, no patent grant.
2. **Apache-2.0 only** — explicit patent grant, more formal, some compatibility friction
   with MIT-only projects.
3. **MIT OR Apache-2.0 (dual)** — the Rust ecosystem convention; users choose.
4. **MPL-2.0** — file-level copyleft; unnecessary if no MPL code is reused, and less
   convenient for some embedders.
5. **GPL-3.0** — strong copyleft; conflicts with the goal of broad adoption.

## Decision

Dual-license the project as **MIT OR Apache-2.0**.

## Consequences

- Every source file is available under either license, at the user's option.
- Apache-2.0 supplies an express patent grant; MIT keeps the terms short.
- All planned dependencies are compatible.
- If MPL-2.0 code is ever copied or translated, the affected files must remain MPL-2.0 and
  this ADR must be revisited. That is a stop-and-reassess event.
- `LICENSE-MIT`, `LICENSE-APACHE`, and `NOTICE` must exist before the first public release.
- Legal review is recommended before 1.0; this ADR records an engineering decision, not
  legal advice.
