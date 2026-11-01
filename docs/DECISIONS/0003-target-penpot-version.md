# ADR-0003 — Target Penpot 2.18.5 and file data version 67

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** engineering
- **Related:** `docs/PENPOT_FORMAT.md`

## Context

Penpot's file format evolves. The importer validates against schemas that change between
releases, and the file data version is bumped by migrations. A converter that targets
"latest" cannot be tested reproducibly.

At research time:

- Latest stable Penpot: **2.18.5**, released 2026-10-26.
- The `develop` branch already contains 2.19 release candidates and newer import limits.
- `@penpot/library` 1.2.0 generates archives with file data version **67**.
- Penpot 2.18.5's migrations list ends at `legacy-67`.
- The 2.18.5 importer accepts archive versions 1 and 3; v3 is the ZIP format.

## Options

1. **Target the latest stable release (2.18.5).** Reproducible, testable, and what most
   users run.
2. **Target `develop`.** Newest fields, but unstable and likely to change before release.
3. **Target several versions at once.** More coverage, more test cost, unclear minimum.
4. **Target an older LTS-like release.** Misses features and fixes; no evidence it is
   materially more stable.

## Decision

Target **Penpot 2.18.5**, archive v3, manifest version 1, file data version **67**, using
`@penpot/library` 1.2.0 output as the format oracle. Record the target in every report.

Do not emit fields added after 2.18.5 (for example `externalLibraries` in the manifest).

## Consequences

- Golden archives and structural validation are pinned to a known schema.
- Users on newer Penpot versions may work, but are **unverified** until tested; the report
  states the target version.
- A Penpot upgrade requires a new ADR, regenerated probes, updated goldens, and a real
  import check.
- The 2.18.5 importer has fewer text-entry limits than `develop`; our writer must be
  stricter than the importer regardless.
- Real import verification is still pending because no instance is available locally.
