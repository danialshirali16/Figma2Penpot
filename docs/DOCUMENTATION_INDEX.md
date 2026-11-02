# Documentation index

Reviewed **2026-11-02** against the source compatibility work in ADR-0011. The five private
outputs import into local Penpot 2.18.5-RC1; import compatibility is **Verified**, overall
fidelity is **Partial**. Current source passed 116 workspace tests, formatting, and Clippy
in a Linux container. Private data and local delivery artifacts are not public fixtures.

## Current guidance

| Document | Role |
|---|---|
| [README](../README.md) | Product overview, actual workflow, source-versus-release status |
| [PROGRESS](../PROGRESS.md) | Canonical living state, recorded commands/results, session history |
| [State](../State.md) | Compatibility pointer to PROGRESS; no second status ledger |
| [Roadmap](../Roadmap.md) | Milestone gates, implemented scope, remaining acceptance work |
| [AGENTS](../AGENTS.md) | Repository rules, build/check commands, privacy and reporting requirements |
| [Project brief](PROJECT_BRIEF.md) | Requirements and non-goals; not an implementation checklist |
| [Architecture](ARCHITECTURE.md) | Current crates, materialized pipeline, standard-thread queue, limits and known gaps |
| [Support matrix](SUPPORT_MATRIX.md) | Authoritative per-feature support labels and limitations |
| [Issue codes](ISSUE_CODES.md) | Reader warnings and 30 converter issue codes |
| [Figma format](FIG_FORMAT.md) | Observed framing/schema/node behavior and explicit unknowns |
| [Penpot format](PENPOT_FORMAT.md) | Pinned v3 target, writer/validator schema, archive bounds |
| [Development environment](DEV_ENVIRONMENT.md) | Current Windows/container facts and dated historical macOS evidence |
| [Manual QA](MANUAL_QA.md) | Real imports, verified observations, outstanding native/visual/edit checks |
| [Performance](PERFORMANCE.md) | Historical benchmark/fuzz results and scope limits; no new private-file peak-RSS claim |
| [Risks](RISKS.md) | Current component, token, memory, format, accessibility and release risks |
| [Release](RELEASE.md) | Published v0.1.1 artifacts, later source-only changes, next-release gates |
| [Privacy](../PRIVACY.md) | Offline conversion, private report contents, local development verification |
| [License assessment](LICENSE_ASSESSMENT.md) | Upstream reuse decisions and outstanding compliance scans |
| [Research sources](RESEARCH_SOURCES.md) | Pinned references, observations, fixture provenance |
| [Third-party overview](../THIRD_PARTY.md) | Direct dependencies and release notice obligations |
| [Third-party inventory](../THIRD_PARTY_LICENSES.md) | Locked Linux metadata inventory of 221 packages and bundled assets |
| [NOTICE](../NOTICE) | Attribution/trademark and fixture/font notice pointers |

## Design and evidence

| Document | Role / evidence scope |
|---|---|
| [UI specification](UI_SPEC.md) | Design contract, copy mappings, accessibility and implementation discrepancies |
| [Wireframes](ui/WIREFRAMES.md) | Design intent, not proof of native behavior |
| [UI review](ui/REVIEW.md) | Dated macOS heuristic review; corrected batch-preference claim |
| [UI screenshot index](ui/screenshots/README.md) | Historical macOS captures using redistributable fixtures |
| [Import screenshot index](qa/README.md) | Historical fixture imports; private compatibility screenshots excluded |
| [Fixture provenance](../tests/fixtures/README.md) | Four MIT reference and nine project-owned synthetic inputs |
| [Benchmark baseline guide](../tests/benchmarks/README.md) | Fixture correctness checks and unchanged historical timing references |

## Decisions and immutable legal texts

ADRs [0001](DECISIONS/0001-project-license.md) through
[0010](DECISIONS/0010-ui-redesign.md) retain their historical context; later decisions
and current evidence do not retroactively turn proposals into verified features.
[ADR-0011](DECISIONS/0011-private-file-compatibility.md) records the bounded component,
geometry, paint, token, and target-schema compatibility changes and their remaining limits.
The project [MIT](../LICENSE-MIT) and [Apache-2.0](../LICENSE-APACHE) license texts and the
bundled font's [OFL](../crates/desktop-app/assets/fonts/OFL.txt) are unchanged legal texts.

Documentation was reviewed as a set. Static requirements, historical ADRs, license texts,
screenshots, goldens, and timing baselines are retained where still accurate; they are not
newly certified by a documentation update. The source compatibility changes are not in
existing v0.1.1 binaries. No new release is implied by committing or merging these changes.

## Outstanding work

Component properties/swaps, resized child layout, full transform handling, grids, mixed
text, effects and token-mode semantics remain Partial or Not supported as documented in
the support matrix. Batch token preference propagation needs a regression test/fix.
Full visual comparisons, component edits/reopening, native platform interactions,
accessibility, targeted component fuzz/scale work, cross-platform license texts, and
license/advisory scans remain pending. Do not infer 100% fidelity from import acceptance.
