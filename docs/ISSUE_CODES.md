# Issue Codes

Stable identifiers used by diagnostics. Once a code ships, its meaning must not change; add
a new code instead.

This file currently documents **reader warnings** from `fig-reader`. Conversion report codes
(`F2P-*`) will be added with the report crate in M2; they will use the same registry.

| Code | Severity | Meaning | Suggested action | Test |
|---|---|---|---|---|
| `FIG-WARN-UNKNOWN-ENTRY` | Warning | The ZIP contained entries other than `canvas.fig`, `meta.json`, `thumbnail.png`, or `images/*`. They were ignored. | None for conversion; inspect if a future format feature is expected. | `unknown_entries_produce_a_warning_and_valid_canvas_still_parses` |
| `FIG-WARN-IMAGE-NAME` | Warning | An `images/` entry name is not a 40-character lowercase SHA-1 hex string. | The entry is still listed; conversion must verify the paint reference before using it. | Unit coverage in `container.rs` (`sha1_names_are_lowercase_and_exact`) |
| `FIG-WARN-META` | Warning | `meta.json` exists but could not be parsed as JSON or did not contain a usable name/version. | The file can still be inspected; document name is shown as Unknown. | Manual/negative follow-up in M2 |
| `FIG-WARN-NONFINITE` | Warning | A node's size or transform contains NaN or infinity. | Preserve the raw value for diagnostics; do not emit invalid Penpot geometry. | M2 conversion test |
| `FIG-WARN-CYCLE` | Warning | Some active nodes are not reachable from a root because of a parent cycle or missing links. | Inspect the parent GUIDs; do not recurse indefinitely. | Pending synthetic fixture |

## Rules

- Codes are uppercase, prefixed by subsystem (`FIG-` reader, `F2P-` conversion/report).
- A warning means the file was read but something was skipped, approximated, or could not be
  determined. It is never silently upgraded to success.
- An error is returned as a typed `FigError` variant, not as a warning code. Errors do not
  produce partial `FigDocument` output.
- Every new code needs a test asserting it, a row here, and a note in `SUPPORT_MATRIX.md` if
  it describes a feature limitation.
