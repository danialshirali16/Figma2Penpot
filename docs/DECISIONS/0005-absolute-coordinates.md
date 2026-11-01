# ADR-0005 — Penpot geometry is absolute; accumulate transforms

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** engineering
- **Related:** `docs/FIG_FORMAT.md` §6, `docs/PENPOT_FORMAT.md` §6.2

## Context

Figma `.fig` transforms are parent-relative. Penpot's shape JSON has `parentId` and
`frameId`, which makes it tempting to assume child coordinates are parent-relative too.
They are not.

Evidence gathered in M0:

1. Penpot's frame renderer places children inside a `<g>` without translating by the
   frame's `x/y` (`penpot/2.18.5 frontend/src/app/main/ui/shapes/frame.cljs`).
2. The reference converter accumulates every ancestor matrix before writing and documents
   the rule (`penpot-converter/.../toDocument.ts:614-615`, README lines 134-138).
3. The official builder library stores the `x/y` it is given. A probe with a frame at
   `(100,200)` and a child added at `(5,7)` wrote the child at `(5,7)`. The library does not
   normalize; the caller must supply absolute coordinates.

A naive reading of (3) alone can look like "relative coordinates", but (1) settles the
interpretation: the file would render the child at page `(5,7)`, outside its frame, which is
why the writer must add the parent offset.

## Decision

Store **absolute page coordinates** in Penpot shapes:

- Accumulate the Figma transform from the page root for every node.
- Use the result for `x/y`, `selrect`, `points`, and path content.
- Keep `parentId`/`frameId` for containment, clipping, and grouping, not for positioning.
- Preserve rotation/flip in the Penpot `transform` matrix; verify the representation with
  a fixture before claiming fidelity.

## Consequences

- The writer needs a correct, property-tested matrix implementation from M2.
- Paths must be transformed, not copied, or they drift from their frames.
- Nested fixtures with non-zero frame origins are mandatory tests.
- Real Penpot import remains the final proof; this ADR records the source-level evidence,
  not an imported verification.
