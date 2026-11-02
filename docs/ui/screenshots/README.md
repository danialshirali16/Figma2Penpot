# UI screenshots

Reviewed **2026-11-02**. These images document the macOS designer UI before ADR-0011.
They are not visual evidence for the new component/section compatibility mappings or for
native Windows/Linux behavior. See `../../MANUAL_QA.md` for current verification scope.

Captured on **macOS 27.2 (arm64, Apple M1)** on 2026-11-02 from the native app built with
`cargo build --offline -p desktop-app`. Images are Retina captures scaled to 1440 px wide.

- `home-*`, `files-*`, `details-*`, `settings-*`, `converting-*`, `results-*`,
  `report-*`, `preferences-*`: each screen in light/dark at `large` and `compact` sizes.
- `gallery-*`: dev-only component gallery.
- `files-empty-light-large.png`: empty queue.
- `results-error-light-large.png`: one real conversion and one real overwrite failure.
- `licenses-light-large.png`: offline license modal.
- `clear-light-large.png`: clear-all confirmation.

The Files and Results captures use copies of the repository's redistributable fixtures,
renamed for readability (`Landing page.fig`, `Design system.fig`, `Checkout flow.fig`,
`Masks.fig`). The Report capture is the real JSON report generated from `Masks.fig`. No
private or client files were used.

Preview hooks are developer-only environment variables in
`crates/desktop-app/src/app.rs`; they are not exposed in the product UI. A representative
regeneration command (the exact matrix is in the session notes):

```bash
cargo build --offline -p desktop-app
F2P_PREVIEW_SIZE=large F2P_PREVIEW_THEME=light F2P_PREVIEW_QUIET=1 \
  F2P_PREVIEW_SCREEN=report F2P_PREVIEW_REPORT=/tmp/Masks.report.json \
  target/debug/figma2penpot
```
