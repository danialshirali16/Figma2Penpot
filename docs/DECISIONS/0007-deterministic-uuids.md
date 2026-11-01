# ADR-0007 — Deterministic UUIDs from Figma GUIDs

- **Status:** Accepted
- **Date:** 2026-11-01
- **Related:** `crates/conversion-core`, `docs/PENPOT_FORMAT.md`

## Context

Penpot shapes, pages, and files need UUIDs. Random UUIDs work, but make golden reports,
archive comparisons, and user support harder: the same input produces different output on
every run.

Figma GUIDs (`sessionID:localID`) are unique within a document. They are stable for the
same file.

## Decision

Derive Penpot UUIDs with UUID v5 (SHA-1, fixed project namespace):

- file: `file:<source path>` — moving the source changes the file ID;
- page: `page:<sessionID>:<localID>`;
- shape: `shape:<sessionID>:<localID>`;
- root frame: the all-zero UUID required by Penpot.

IDs are stable for a given input and deterministic in tests. They are not secrets.

## Consequences

- Two runs on the same input produce byte-identical archives (verified in
  `archive_is_deterministic`), which makes goldens and diffs meaningful.
- Renaming/moving a source file changes the Penpot file ID, but not page/shape IDs.
- A future linked-file feature must avoid treating the derived file ID as a content hash.
- If UUID v5 is replaced, old and new outputs will differ; that requires a versioned report
  or a migration note.
