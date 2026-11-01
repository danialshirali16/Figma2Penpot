# Research Sources and Provenance

Snapshot: **2026-11-01**. All upstream checkouts used for M0 are pinned below by commit.
Research checkouts live outside this repository at `~/Downloads/f2p-research-upstream/`
and are **not** part of the project source tree.

---

## 1. Repositories studied

| Project | Commit | Commit date | Declared license | What it was used for |
|---|---|---|---|---|
| [`girafic/penpot-converter`](https://github.com/girafic/penpot-converter) | `c29242f60ebe3579265260894b40e9bcfe54ffb3` | 2026-09-08 | **None declared** | Behavior reference: pipeline, traps, feature coverage, CLI. No code reused. |
| [`penpot/penpot-exporter-figma-plugin`](https://github.com/penpot/penpot-exporter-figma-plugin) | `eae25d8f796ecf690f47a6a30a5b2db56762600e` | 2026-10-26 | **MPL-2.0** | Behavior reference: Figma→Penpot mapping, Penpot model types. No code reused. |
| [`penpot/penpot`](https://github.com/penpot/penpot) | tag `2.18.5` (`f44566963daf61fce4d8608e7c5e8c27c5853a88`) | 2026-10-26 | **MPL-2.0** | Authoritative `.penpot` importer/export schemas and shape model. No code reused. |
| [`OpenFig-org/openfig-core`](https://github.com/OpenFig-org/openfig-core) | `0d67354fcd6d14c37b89c42f55330faf0a79e1ce` | 2026-08-02 | **MIT** (package metadata; no LICENSE file in repo) | `.fig` format notes and MIT-licensed fixtures. No runtime use. |
| [`evanw/kiwi`](https://github.com/evanw/kiwi) / `kiwi-schema` crate | `0.2.1` (crate) | 2023-09-03 | **MIT** | Wire-format reference; not a runtime dependency (ADR-0006) |
| [`penpot/penpot` `library/`](https://github.com/penpot/penpot/tree/2.18.5/library) / `@penpot/library` | `1.2.0` | — | **MPL-2.0** | Generated a real `.penpot` probe to observe the archive format. No code reused. |

The reference converter is explicitly a Node port of the official MPL-2.0 exporter: its
README says the upstream `ui-src/parser` is taken "1:1", with four patches. It declares no
license itself. That combination is why it is treated as reference-only (see
`docs/LICENSE_ASSESSMENT.md`).

## 2. Documentation and community sources

| Source | URL | Used for |
|---|---|---|
| Penpot file format (technical) | <https://github.com/penpot/penpot/blob/2.18.5/docs/technical-guide/developer/data-model/penpot-file-format.md> | Archive structure, manifest, shape fields |
| Penpot file format (user guide) | <https://help.penpot.app/user-guide/export-import/penpot-file-format/> | v3 status, inspection workflow |
| Penpot migration guide | <https://help.penpot.app/user-guide/first-steps/migration-guide/> | What users are told to expect (Flex/Grid, tokens, validation) |
| Community thread | <https://community.penpot.app/t/migrate-figma-components-to-penpot/10870/5> | Original converter's scope and caveats |
| openfig `.fig` notes | `openfig-core/docs/archive.md`, `containers.md`, `nodes.md`, `vector.md`, `overrides.md` | `.fig` container, chunks, node model, vector blobs |
| Rust Kiwi crate | <https://crates.io/crates/kiwi-schema> | Wire-format evaluation; rejected for unbounded array allocation (ADR-0006) |

The community thread (6 posts, 2026-09-07/08) documents a real migration attempt: the
official exporter failed on a design-system file ("Penpot apparently can't verify the
file"), page-only and reduced-file retries did not solve it, and participants discussed
Figma MCP as an alternative. The reference converter's author then announced the local
`.fig` route in post 5. This is the user problem the product exists to solve, and it also
confirms that import verification failures are a real failure mode, not a theoretical one.

## 3. Empirical checks performed

These are the checks whose results the documents rely on. They are reproducible; commands
are recorded in `PROGRESS.md`.

| Check | Method | Result |
|---|---|---|
| `.fig` is a ZIP container | `python3 zipfile` over `openfig-core/test-fixtures/circle.fig` | Entries: `canvas.fig`, `thumbnail.png`, `meta.json`, `images/` |
| Header and chunk layout | Byte inspection | `fig-kiwi`, version `101`, chunk 0 deflate, chunk 1 starts `28 b5 2f fd` (zstd) |
| Schema is 550 definitions, root `Message` | M0 Rust probe; re-confirmed by the M1 in-house reader | Schema decodes; root at def index 404; 44 fields |
| M1 reader | `cargo test`, CLI, fuzz | 4 fixtures parse; 829,577 fuzz executions without a crash |
| Full read path works in Rust | `kiwi-probe` on two fixtures | Nodes and blobs decoded; truncated input returned a typed error, no panic |
| `.penpot` archive layout | `@penpot/library 1.2.0` generated a probe | `manifest.json`, `files/*.json`, `pages/*.json`, per-shape JSON, root frame |
| Penpot file data version | Generated probe + 2.18.5 migrations | `version: 67` |
| Coordinates are absolute | Penpot frame renderer + reference converter + library probe | Children are rendered without a parent translate; writer must accumulate transforms |

The probe generator is **not** part of the product. Its output (`probe.penpot`,
`probe2.penpot`, `coord.penpot`) is test data used only to observe the format.

## 4. Fixtures and their license

`openfig-core/test-fixtures/` is part of an MIT-declared package. The fixtures are
reference inputs, not generated output. Planned use in M1:

- `circle.fig` — minimal document, 5 nodes, 3 blobs.
- `OpenFigs.fig` — richer document, version 106.
- `vector-network/*.fig` — vector-network conformance cases, including a 2.2 MB file with
  embedded images.

If these are copied into `tests/fixtures/`, the MIT notice and the upstream commit must be
recorded in `THIRD_PARTY_LICENSES` and next to the fixtures. Do not add files produced by
our own writer as conformance references.

## 5. Reported vs. verified

The documents use three confidence words:

- **Confirmed** — reproduced here by byte inspection, source reading, or a runnable check.
- **Reported** — stated by an upstream README or issue, not independently reproduced.
- **Unknown** — not established; must not be assumed.

Known discrepancies found during research:

1. The reference converter's README lists effects as implemented (line 947) and as open
   (line 951). The code is the tie-breaker in M1; the README alone is not evidence.
2. Penpot's technical format doc shows `transform` as an array and fills as
   `{color, opacity}`; the 2.18.5-generated file uses an object
   `{a,b,c,d,e,f}` and `{fillColor, fillOpacity}`. The generated file and schema source
   win. The doc example is stale.
3. The technical doc's manifest example uses `refer`; the 2.18.5 schema field is
   `referer`, and the importer accepts either (`(or referer refer)`). Emit `referer`.
4. The reference converter's README calls Penpot coordinates absolute, while a naive
   reading of a probe file can look relative because the library stores what it is given.
   Penpot's frame renderer does not translate children, confirming absolute storage. See
   ADR-0005.

Discrepancies are recorded rather than smoothed over. If a later finding contradicts one
of these, update the document and the ADR.
