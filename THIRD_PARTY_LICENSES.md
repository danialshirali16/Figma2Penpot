# Third-Party Licenses

This file lists the third-party components currently in the build graph. It is generated from
`cargo metadata` and updated deliberately; release packaging must regenerate it with the full
license texts.

No upstream Figma or Penpot source code is bundled. Research-only projects are listed in
`docs/LICENSE_ASSESSMENT.md`.

## Bundled assets

### Inter

- File: `crates/desktop-app/assets/fonts/Inter-VariableFont_opsz,wght.ttf`
- Copyright 2016 The Inter Project Authors (https://github.com/rsms/inter)
- License: SIL Open Font License, Version 1.1
- Full license: `crates/desktop-app/assets/fonts/OFL.txt`

The font is bundled for UI typography. It is not modified.

### UI icons

UI icons are project-owned line drawings rendered by `desktop-app/src/components.rs`.
No third-party icon font or icon set is bundled.

## Rust dependencies

All packages below are used under the license expression shown. The project's dual
MIT/Apache-2.0 license is compatible with each expression.

| Package | License |
|---|---|
| `adler2` | 0BSD OR MIT OR Apache-2.0 |
| `anstream` | MIT OR Apache-2.0 |
| `anstyle` | MIT OR Apache-2.0 |
| `anstyle-parse` | MIT OR Apache-2.0 |
| `anstyle-query` | MIT OR Apache-2.0 |
| `anstyle-wincon` | MIT OR Apache-2.0 |
| `anyhow` | MIT OR Apache-2.0 |
| `bitflags` | MIT OR Apache-2.0 |
| `bumpalo` | MIT OR Apache-2.0 |
| `cfg-if` | MIT OR Apache-2.0 |
| `cfg_aliases` | MIT |
| `clap` | MIT OR Apache-2.0 |
| `clap_builder` | MIT OR Apache-2.0 |
| `clap_derive` | MIT OR Apache-2.0 |
| `clap_lex` | MIT OR Apache-2.0 |
| `colorchoice` | MIT OR Apache-2.0 |
| `crc32fast` | MIT OR Apache-2.0 |
| `equivalent` | Apache-2.0 OR MIT |
| `flate2` | MIT OR Apache-2.0 |
| `getrandom` | MIT OR Apache-2.0 |
| `hashbrown` | MIT OR Apache-2.0 |
| `heck` | MIT OR Apache-2.0 |
| `indexmap` | Apache-2.0 OR MIT |
| `is_terminal_polyfill` | MIT OR Apache-2.0 |
| `itoa` | MIT OR Apache-2.0 |
| `js-sys` | MIT OR Apache-2.0 |
| `libc` | MIT OR Apache-2.0 |
| `memchr` | Unlicense OR MIT |
| `miniz_oxide` | MIT OR Zlib OR Apache-2.0 |
| `nix` | MIT |
| `once_cell_polyfill` | MIT OR Apache-2.0 |
| `proc-macro2` | MIT OR Apache-2.0 |
| `quote` | MIT OR Apache-2.0 |
| `r-efi` | MIT OR Apache-2.0 OR LGPL-2.1-or-later |
| `rustversion` | MIT OR Apache-2.0 |
| `ruzstd` | MIT |
| `serde` | MIT OR Apache-2.0 |
| `serde_core` | MIT OR Apache-2.0 |
| `serde_derive` | MIT OR Apache-2.0 |
| `serde_json` | MIT OR Apache-2.0 |
| `simd-adler32` | MIT |
| `strsim` | MIT |
| `syn` | MIT OR Apache-2.0 |
| `thiserror` | MIT OR Apache-2.0 |
| `thiserror-impl` | MIT OR Apache-2.0 |
| `twox-hash` | MIT |
| `typed-path` | MIT OR Apache-2.0 |
| `unicode-ident` | (MIT OR Apache-2.0) AND Unicode-3.0 |
| `utf8parse` | Apache-2.0 OR MIT |
| `wasm-bindgen` | MIT OR Apache-2.0 |
| `wasm-bindgen-macro` | MIT OR Apache-2.0 |
| `wasm-bindgen-macro-support` | MIT OR Apache-2.0 |
| `wasm-bindgen-shared` | MIT OR Apache-2.0 |
| `windows-link` | MIT OR Apache-2.0 |
| `windows-sys` | MIT OR Apache-2.0 |
| `futures-core` | MIT OR Apache-2.0 |
| `futures-task` | MIT OR Apache-2.0 |
| `futures-util` | MIT OR Apache-2.0 |
| `pin-project-lite` | Apache-2.0 OR MIT |
| `sha1_smol` | MIT |
| `slab` | MIT |
| `uuid` | Apache-2.0 OR MIT |
| `zip` | MIT |
| `zmij` | MIT |

Platform-specific crates (`windows-*`, `wasm-*`, `nix`, `r-efi`) are listed because they
appear in the resolved graph; they are not all compiled on every target.

## Test fixtures

`tests/fixtures/` contains four `.fig` files from `OpenFig-org/openfig-core` at commit
`0d67354`, licensed MIT. The copyright notice and permission text are reproduced in
`tests/fixtures/README.md`, next to the files.

## Updating this file

1. Run `cargo metadata --format-version 1` after dependency changes.
2. Add or remove entries; never silently drop a license.
3. At M8, generate the full text bundle with `cargo-about` and attach it to releases.
4. `cargo-deny` must include `0BSD` and `Unicode-3.0` in its allowlist for the current graph.
