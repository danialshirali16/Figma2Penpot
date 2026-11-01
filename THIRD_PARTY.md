# Third-Party Notices

This project is licensed under **MIT OR Apache-2.0**.

Direct runtime dependencies and their licenses:

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

Transitive dependencies retain their own licenses. Generate a complete inventory with a
tool such as `cargo deny` / `cargo license` and review it before a stable release. `cargo
deny` is not installed in the current development environment; this is a release blocker for
a signed v1.0, not for the current pre-alpha artifacts.

Linux desktop packages use the system GTK3, X11/Wayland, and OpenGL libraries; those are
provided by the operating system, not bundled here.
