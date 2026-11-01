# M2–M3.5 QA evidence

Screenshots from a local Penpot **2.18.5** stack (backend reports `2.18.5-RC1`) on
2026-11-01. The stack used the official `docker/images/docker-compose.yaml` at tag 2.18.5,
with telemetry disabled, and was stopped after verification.

Only project-owned or MIT-licensed fixtures were imported. No private designs are present.

| Image | Shows |
|---|---|
| `circle-import.png` | `circle.penpot` opened in Penpot: white 350×350 frame, red ellipse with a dark stroke |
| `text-group-import.png` | `text-group.penpot` opened: Card frame, red rounded rect, green ellipse, `Hello Penpot` text, blue group rect |
| `text-group-layers-expanded.png` | Layer hierarchy `Card > Title / Green Ellipse / Red Rect / Badge Group > Group Rect`, with Card inspector X 20, Y 30, W 400, H 300 |
| `red-rect-inspect.png` | `Red Rect` inspector: W 100, H 50, X 10, Y 20 relative to Card, fill `FF0000`, stroke `000000` 2 px, radius 4 |
| `text-inspect.png` | `Title` inspector: Inter 16, weight 400, X 10, Y 100 relative to Card, text visible |
| `group-inspect.png` | `Badge Group` inspector: W 80, H 30, X 5, Y 205 relative to Card, child `Group Rect` present |
| `vector-paths-expanded.png` | 20 path layers under the OpenFigs frame; white vector icon rendered |
| `stroke-vector-import.png` | Stroke-only line imported and rendered green (reported as an outlined approximation) |
| `effects-gradient-import.png` | `effects-gradient.penpot` opened: gradient rectangle with shadow/blur |
| `effects-gradient-inspect.png` | Gradient rectangle selected; gradient fill visible |
| `effects-gradient-panel.png` | Effects panel listing shadow, layer blur, and background blur |
| `image-import.png` | First image fixture opened |
| `image-red-import.png` | Red 8×8 image fill rendered in the board |
| `image-inspect.png` | `Image Fill` rect selected with image fill |
| `auto-layout-import.png` | Auto Layout fixture opened: two rects in a flex row |
| `auto-layout-inspect.png` | Auto Layout frame inspector showing flex, gap 12, padding 8/20/12/10 |
Import log evidence is recorded in `docs/MANUAL_QA.md`. Reopen and move/edit persistence
were not tested. Text metrics have no screenshot yet; they are unit-tested only.
