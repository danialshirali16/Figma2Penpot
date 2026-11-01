# Technical Risk Assessment

Status: M0 baseline; reviewed at M1. This register is reviewed at the end of every
milestone; new risks are added when they are discovered, not when they bite.

Scoring: **Likelihood** and **Impact** are High / Medium / Low. A risk with High/High needs
a mitigation before the milestone that depends on it starts.

---

## 1. Top risks

### R1 — Figma schema or semantics drift

- **Likelihood:** High over the life of the project. **Impact:** High.
- **Why:** `.fig` has no public specification. The embedded Kiwi schema makes structural
  decoding robust, but field meaning can change between Figma releases, and new node types
  can appear without notice.
- **Mitigation:** decode with the embedded schema; skip unknown fields; treat unknown node
  types as reported placeholders; keep a fixture per Figma version; run the corpus in CI on
  every change; never gate on a version number.
- **Early warning:** a new fixture produces `FIG-UNKNOWN-*` events or a field that used to
  be populated is suddenly absent.
- **Owner milestone:** M1 (reader), M3 (mapping), ongoing.

### R2 — Penpot import strictness and version drift

- **Likelihood:** High. **Impact:** High.
- **Why:** The importer validates against evolving Clojure schemas. A field that is
  optional in one release may be required in the next; a field the library emits today may
  be rejected tomorrow. We pinned 2.18.5, but users will run other versions.
- **Mitigation:** pin the target in an ADR; generate golden archives; validate structurally
  against observed rules; keep an optional real-import check in CI; refuse to claim
  **Verified** without an import; document the tested Penpot version in every report.
- **Early warning:** a structurally valid archive fails to import, or a field appears in
  the format document but not in generated output.
- **M2 evidence:** `circle.penpot` and `text-group.penpot` imported into 2.18.5-RC1 with
  `error?=false`; screenshots and inspector values in `docs/MANUAL_QA.md`.
- **Owner milestone:** M2, M3, M8.

### R3 — Coordinate and transform errors

- **Likelihood:** High if unaddressed. **Impact:** High (everything is misplaced).
- **Why:** `.fig` is parent-relative; Penpot stores absolute page coordinates. Rotation,
  flips, and nested transforms compound the problem. The official builder does not
  normalize coordinates; the writer must.
- **Mitigation:** one matrix utility, property-tested; a nested fixture with frames at
  non-zero origins; a golden test that asserts absolute positions; a visual check after
  real import. Never convert a path without the accumulated matrix.
- **Early warning:** nested content appears offset by its parent's position; rotated shapes
  land in the wrong place; path geometry drifts from its frame.
- **M2 evidence:** Card at absolute (20,30); inspector showed Red Rect relative (10,20),
  which is absolute (30,50) in the file. Coordinates are working for translated geometry.
- **Owner milestone:** M2 (basic), M3 (rotation/flip/paths).

### R4 — Component and override fidelity

- **Likelihood:** High. **Impact:** High for design-system migrations.
- **Why:** Overrides are deltas keyed by `overrideKey` paths, nested instances are
  recursive, variants add another resolution layer, and Penpot's `shapeRef`/`touched`
  reconciliation is subtle. Getting the structure wrong produces files that open but
  silently lose overrides.
- **Mitigation:** build the component index before conversion; materialize instances with a
  depth cap; resolve overrides outermost-wins; emit an event per dropped or unresolved
  override; test a fixture with nested instances and a variant matrix; real import check
  before **Verified**.
- **Early warning:** instances lose text or fills after import; component counts don't
  match the index; Penpot shows "broken component" states.
- **Owner milestone:** M3.7.

### R5 — Token and mode mapping

- **Likelihood:** High. **Impact:** Medium-High.
- **Why:** Figma modes are per-collection and per-node; Penpot themes are file-wide.
  There is no lossless mapping. Token names must be sanitized consistently or references
  dangle.
- **Mitigation:** define an explicit policy (resolved values by default; token-faithful
  mode as an opt-in); sanitize set names and references with one function; report every
  dropped binding; never claim token fidelity without an import.
- **Early warning:** token references resolve to nothing; colors are correct but bindings
  are gone without an event.
- **Owner milestone:** M3.8.

### R6 — Text and font fidelity

- **Likelihood:** Medium-High. **Impact:** Medium.
- **Why:** Mixed styles, line height units, letter spacing units, list formatting, missing
  fonts, and Penpot's string-typed style values are all easy to get subtly wrong.
- **Mitigation:** start with single-style text; add runs with fixtures; record missing fonts;
  never silently substitute; document that list markers may be written into text because
  Penpot has no list model.
- **Early warning:** text reflows on import; font weights change; empty strings cause
  rejection.
- **Owner milestone:** M3.4.

### R7 — Boolean operations and masks

- **Likelihood:** Medium. **Impact:** Medium.
- **Why:** Figma stores cached geometry plus boolean parameters; Penpot has its own bool
  model. Reconstructing editability from cached paths is possible but can lose the
  operation tree.
- **Mitigation:** prefer preserving the bool tree; fall back to a path with a reported
  loss; test nested booleans and open paths.
- **Owner milestone:** M3.1/M3.2.

### R8 — Large files, memory, and instance expansion

- **Likelihood:** High on component-heavy files. **Impact:** High on 8 GiB machines.
- **Why:** The reference converter reports that cost scales with the number of instance
  copies, not only file size. Expanding every instance can multiply the node count.
- **Mitigation:** stream assets; bound concurrency; page splitting; cap expansion depth;
  measure peak RSS on named fixtures; fail with an actionable error instead of OOM.
- **Early warning:** peak RSS grows with batch size; a single file exhausts memory; the UI
  freezes during allocation.
- **Owner milestone:** M5, M7.

### R9 — Parser panic or resource exhaustion

- **Likelihood:** Medium without hardening. **Impact:** High (crash, hang, disk fill).
- **Why:** Untrusted counts, lengths, nesting, and compressed data. An off-the-shelf Kiwi
  parser allocated from untrusted array counts, so M1 replaced it with a bounded in-house
  decoder. ZIP and zstd add their own attack surface.
- **Mitigation:** checked arithmetic, absolute caps, depth limits, no indexing without
  bounds checks, fuzzing from day one, `catch_unwind` at the job boundary, process
  isolation as a fallback.
- **M1 evidence:** 829,577 fuzz executions, no crashes; hostile-archive tests pass.
- **Early warning:** fuzz crashes, timeouts, or allocations that scale with declared rather
  than actual input size.
- **Owner milestone:** M1, M7.

### R10 — License contamination

- **Likelihood:** Medium if contributors copy snippets. **Impact:** High (project cannot
  ship under its chosen license).
- **Why:** The reference converter is unlicensed and contains MPL-2.0-derived code; the
  official exporter and Penpot are MPL-2.0. A copied function can pull obligations into the
  project.
- **Mitigation:** no code reuse from those sources; record provenance for fixtures; review
  every dependency license; `cargo-deny` in CI; a contribution policy that requires
  attribution and origin.
- **Early warning:** a PR contains code with no provenance; a dependency adds a copyleft
  license; a fixture has no recorded source.
- **Owner milestone:** M0, ongoing.

### R11 — GUI accessibility and platform fidelity

- **Likelihood:** Medium. **Impact:** Medium-High.
- **Why:** `egui` is immediate-mode and not a native AppKit/WinUI control tree. Screen
  reader support, focus order, and native menu behavior need deliberate work.
- **Mitigation:** keyboard-first design; labels on every control; `accesskit`; manual QA
  with VoiceOver and Narrator; document known gaps rather than claiming full compliance.
- **Owner milestone:** M4.

### R12 — Cross-platform packaging and signing

- **Likelihood:** Medium. **Impact:** Medium.
- **Why:** Windows MSI/EXE and macOS `.app`/`.dmg` require platform tooling, signing
  identities, and notarization. The current machine can only build macOS arm64.
- **Mitigation:** GitHub Actions matrix from M1; unsigned dev artifacts early; signing plan
  documented; label unverified platforms honestly.
- **Owner milestone:** M8.

### R13 — Report accuracy drift

- **Likelihood:** Medium. **Impact:** High (the product's differentiator).
- **Why:** It is easy to add a mapping branch that forgets to emit an event, or to compute
  counts from output instead of traversal.
- **Mitigation:** pass a sink to every pass; test the count invariant
  (`converted + approximated + skipped + unsupported + failed == detected`); golden reports;
  code review checklist; no branch without a decision record.
- **Early warning:** a fixture's report has no event where a known lossy mapping occurs;
  counts don't add up.
- **M2 evidence:** counts are tested for circle (2/2), text-group (6/6), and an unsupported
  vector (1 converted/1 unsupported); golden report checked in.
- **Owner milestone:** M2 onward.

### R14 — Fixture scarcity and provenance

- **Likelihood:** High. **Impact:** Medium.
- **Why:** Redistributable `.fig` files are hard to obtain; synthetic files may not exercise
  real-world structure; user files cannot be committed without permission.
- **Mitigation:** use the MIT openfig fixtures; generate synthetic cases; keep user-provided
  files outside the repo and record hashes/permission; build a corpus manifest.
- **Owner milestone:** M1.

### R15 — Cancellation and atomicity races

- **Likelihood:** Medium. **Impact:** High (corrupt output presented as success).
- **Why:** Cancellation can land between temp write, validation, and rename; crashes leave
  stale temp files; two jobs can target the same output name.
- **Mitigation:** write to a unique temp file in the destination volume; validate before
  rename; rename is the commit point; cancellation only deletes temp files; startup cleanup
  by marker/age; duplicate-name policy resolves before conversion.
- **Early warning:** a cancelled job leaves a file at the final path; two outputs collide;
  temp files accumulate.
- **Owner milestone:** M2/M5.

### R16 — Scope creep into editing or cloud features

- **Likelihood:** Medium. **Impact:** Medium.
- **Why:** Users will ask for Penpot→Figma, live sync, plugin support, or auto-repair.
- **Mitigation:** non-goals in README and roadmap; say no in the issue template; keep the
  architecture free of network and editing assumptions.
- **Owner milestone:** ongoing.

## 2. Risk matrix

| ID | Risk | Likelihood | Impact | Gate |
|---|---|---|---|---|
| R1 | Figma drift | High | High | M1 |
| R2 | Penpot drift | High | High | M2 |
| R3 | Coordinates | High | High | M2 |
| R4 | Components | High | High | M3.7 |
| R5 | Tokens | High | Medium | M3.8 |
| R6 | Text/fonts | Medium | Medium | M3.4 |
| R7 | Booleans/masks | Medium | Medium | M3.1 |
| R8 | Large files | High | High | M5 |
| R9 | Parser safety | Medium | High | M1 |
| R10 | License | Medium | High | M0 |
| R11 | Accessibility | Medium | Medium | M4 |
| R12 | Packaging | Medium | Medium | M8 |
| R13 | Report accuracy | Medium | High | M2 |
| R14 | Fixtures | High | Medium | M1 |
| R15 | Atomicity | Medium | High | M5 |
| R16 | Scope creep | Medium | Medium | ongoing |

## 3. Assumptions that could invalidate the plan

1. Penpot 2.18.5 remains available and its importer behaves as its source suggests.
2. The embedded Kiwi schema continues to describe the data well enough for typed mapping.
3. The openfig fixtures are representative enough to bootstrap; real files will add cases.
4. `egui` can meet the accessibility bar with reasonable custom work.
5. A pure-Rust zstd decoder (`ruzstd`) handles every `.fig` payload we encounter.
6. Users can obtain `.fig` files through Figma's "Save local copy" and are legally allowed
   to convert them. The app does not bypass Figma's access controls.

If any assumption fails, record it in `docs/DECISIONS/` and adjust the roadmap rather than
quietly weakening a claim.

## 4. Review cadence

- At the end of every milestone: re-score, close resolved risks, add new ones.
- Before any **Verified** claim: confirm the risk's mitigation is evidenced, not just
  planned.
- After a real-import failure: add the failing fixture to the corpus and a regression test
  before fixing the mapping.
