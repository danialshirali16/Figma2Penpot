# Privacy

Figma 2 Penpot is local-first.

- Conversion runs entirely on this machine.
- The app does not send telemetry, analytics, crash reports, or design contents anywhere.
- The conversion path makes no network requests.
- Settings are stored locally under the platform configuration directory for
  `app.figma2penpot.desktop`.
- Output files, reports, and temporary files are written only to the locations you choose
  (plus the system temporary directory used by the atomic writer).

The app contains no accounts, cloud sync, or auto-update pings. Future optional network
features would require explicit opt-in and a separate review; none exist in v1.
