# Third-Party Notices

This project is licensed under **MIT OR Apache-2.0**.

Reviewed **2026-11-02**. No dependency or lockfile changes were made for ADR-0011.
The resolved Linux inventory (221 packages) is in `THIRD_PARTY_LICENSES.md`; other targets
and the full license-text bundle still need regeneration before release.

Direct runtime dependencies and their declared licenses:

| Crate | License |
|---|---|
| `eframe` / `egui` | MIT OR Apache-2.0 |
| `rfd` | MIT |
| `dirs` | MIT OR Apache-2.0 |
| `serde` / `serde_json` | MIT OR Apache-2.0 |
| `thiserror` | MIT OR Apache-2.0 |
| `anyhow` | MIT OR Apache-2.0 |
| `uuid` | Apache-2.0 OR MIT |
| `flate2` | MIT OR Apache-2.0 |
| `ruzstd` | MIT |
| `zip` | MIT |
| `clap` | MIT OR Apache-2.0 |
| `tempfile` | MIT OR Apache-2.0 |
| `tracing` | MIT |

Transitive dependencies retain their own licenses. Generate a complete inventory with a
tool such as `cargo deny` / `cargo license` and review it before a stable release.
`cargo-deny` and `cargo-audit` were unavailable in the verification environment; this is a release blocker for
a signed v1.0, not for the current pre-alpha artifacts.

Linux desktop packages use the system GTK3, X11/Wayland, and OpenGL libraries; those are
provided by the operating system, not bundled here.
