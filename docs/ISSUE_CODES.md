# Issue Codes

Stable identifiers used by diagnostics. Once a code ships, its meaning must not change; add
a new code instead.

This file documents **reader warnings** from `fig-reader` and **conversion events** from
`conversion-core` / `conversion-report`. Codes are stable: once shipped, their meaning must
not change; add a new code instead.

## Reader warnings

| Code | Severity | Meaning | Suggested action | Test |
|---|---|---|---|---|
| `FIG-WARN-UNKNOWN-ENTRY` | Warning | The ZIP contained entries other than `canvas.fig`, `meta.json`, `thumbnail.png`, or `images/*`. They were ignored. | None for conversion; inspect if a future format feature is expected. | `unknown_entries_produce_a_warning_and_valid_canvas_still_parses` |
| `FIG-WARN-IMAGE-NAME` | Warning | An `images/` entry name is not a 40-character lowercase SHA-1 hex string. | The entry is still listed; conversion must verify the paint reference before using it. | `sha1_names_are_lowercase_and_exact` |
| `FIG-WARN-META` | Warning | `meta.json` exists but could not be parsed as JSON or did not contain a usable name/version. | The file can still be inspected; document name is shown as Unknown. | Pending negative fixture |
| `FIG-WARN-NONFINITE` | Warning | A node's size or transform contains NaN or infinity. | Preserve the raw value for diagnostics; do not emit invalid Penpot geometry. | Conversion fail path; explicit fixture pending |
| `FIG-WARN-CYCLE` | Warning | Some active nodes are not reachable from a root because of a parent cycle or missing links. | Inspect the parent GUIDs; do not recurse indefinitely. | Pending synthetic fixture |

## Conversion events (M2)

| Code | Severity | Outcome | Meaning | Test |
|---|---|---|---|---|
| `F2P-NODE-CONVERTED` | Info | Converted | A supported node was emitted as a Penpot shape. | `circle_converts_with_correct_counts`, `text_group_converts_absolute_geometry_and_text` |
| `F2P-NODE-APPROXIMATED` | Warning | Converted with modification | A supported node was emitted with a documented approximation. | `stroke_only_vector_is_reported_as_approximated`; more fixtures pending |
| `F2P-NODE-UNSUPPORTED` | Warning | Unsupported | A Figma node type has no M2 mapping; the node and subtree were skipped. | `unsupported_vector_is_reported_not_dropped` |
| `F2P-NODE-FAILED` | Error | Failed | A node had non-finite or unusable geometry and was skipped. | Unit path; explicit fixture pending |
| `F2P-GROUP-EMPTY` | Warning | Skipped | An empty Figma group has no Penpot equivalent. | Pending explicit fixture |
| `F2P-PAINT-UNSUPPORTED` | Warning | Converted with modification | A gradient/image/unknown paint was skipped; remaining solid paints were kept. | `gradient_paint_is_reported` |
| `F2P-STROKE-UNSUPPORTED` | Warning | Converted with modification | A non-solid stroke paint was skipped. | Unit path; fixture pending |
| `F2P-STROKE-ALIGNMENT` | Warning | Converted with modification | An unknown stroke alignment was replaced with center. | `unknown_stroke_alignment_is_reported` |
| `F2P-STROKE-WIDTH` | Warning | Converted with modification | A non-finite stroke width caused strokes to be skipped. | Unit path; fixture pending |
| `F2P-VECTOR-STROKE-OUTLINE` | Warning | Converted with modification | A vector had only stroke geometry; the expanded outline was converted as a filled path. | `stroke_only_vector_is_reported_as_approximated` |
| `F2P-VECTOR-NETWORK-LOST` | Warning | Converted with modification | `vectorData` network handles were not preserved; path commands were used instead. | `filled_vector_converts_to_multiple_paths` |
| `F2P-VECTOR-EMPTY` | Warning | Unsupported | A vector node had no fill or stroke geometry blobs. | Pending explicit fixture |
| `F2P-VECTOR-FAILED` | Error | Failed | A vector blob index or command stream was invalid. | Pending explicit fixture |
| `F2P-TEXT-EMPTY` | Warning | Skipped | A text node had no usable characters and was skipped. | `empty_text_is_skipped` |

## Rules

- Codes are uppercase, prefixed by subsystem (`FIG-` reader, `F2P-` conversion/report).
- A warning means the file was read or converted but something was skipped, approximated, or
  could not be determined. It is never silently upgraded to success.
- An error is returned as a typed `FigError` variant, not as a warning code. Errors do not
  produce partial `FigDocument` output.
- Every new code needs a test asserting it, a row here, and a note in `SUPPORT_MATRIX.md` if
  it describes a feature limitation.
