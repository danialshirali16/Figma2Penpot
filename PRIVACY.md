# Privacy

Figma 2 Penpot is local-first.

- Conversion runs entirely on this machine.
- The app does not send telemetry, analytics, crash reports, or design contents anywhere.
- The conversion path makes no network requests.
- Settings are stored locally under the platform configuration directory for
  `app.figma2penpot.desktop`.
- Outputs and their temporary archive files are written on the destination volume; reports
  include source paths, layer names, IDs, and conversion diagnostics. Treat reports as
  private design data when sharing them.

The app contains no accounts, cloud sync, or auto-update pings. Future optional network
features would require explicit opt-in and a separate review; none exist in v1.

Development verification may explicitly import generated files into an isolated local
Penpot instance. This is separate from the product conversion path. The 2026-11-02 checks
used loopback HTTP with telemetry disabled; private files and reports were kept outside
tracked fixtures. See `docs/MANUAL_QA.md` and `docs/DEV_ENVIRONMENT.md`.
