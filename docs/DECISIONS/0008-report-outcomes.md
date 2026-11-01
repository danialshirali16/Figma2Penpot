# ADR-0008 — Report outcomes: Skipped and Unsupported are distinct

- **Status:** Accepted
- **Date:** 2026-11-01
- **Related:** `docs/ISSUE_CODES.md`, `crates/conversion-report`

## Context

The M0 sketch used one terminal outcome, `NotConverted`, plus separate counts for skipped
and unsupported elements. That makes the count derivation ambiguous: both categories map to
the same event outcome, so the report has to infer the category from a code or a flag.

The product promise is that the report tells the user *why* something is absent. "Not
converted because the node type is unsupported" is materially different from "skipped
because it was empty or intentionally omitted".

## Decision

Use five explicit outcomes:

- `Converted`
- `ConvertedWithModification`
- `Skipped`
- `Unsupported`
- `Failed`

Counts map one-to-one to these outcomes. An element's terminal outcome is the worst outcome
among its events, so `converted + approximated + skipped + unsupported + failed == detected`
for elements. Additional warning events do not create extra elements.

## Consequences

- Counts are derived mechanically; no inference from issue-code strings.
- Report consumers can filter "unsupported" without parsing codes.
- Every new issue code must choose an outcome deliberately; the vocabulary is closed.
- The JSON schema version is 1; changing the enum requires a schema version bump.
