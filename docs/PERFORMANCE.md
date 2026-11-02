# Performance and Reliability

Recorded: **2026-11-02**. The timings/RSS below are historical macOS measurements.
Session 16 compatibility validation used Windows-hosted Linux containers and did not
remeasure these benchmarks. The existing correctness baseline still passes among the
116 workspace tests; its counts and performance values were not changed. Re-measure before
trusting these numbers on another machine.

## Environment

| Item | Value |
|---|---|
| OS | macOS 27.2, arm64 |
| CPU | 8 logical cores (Apple M1) |
| RAM | 8 GiB |
| Rust | 1.99.1, release profile |
| Penpot target | 2.18.5 / file data v67 |

## Conversion benchmarks

Method: one run per fixture with `target/release/f2p-cli convert <fixture> --out <tmp> --force`,
measured with `/usr/bin/time -l`. Times are wall-clock seconds; RSS is maximum resident set
size. These are not statistically rigorous. The checked-in correctness baseline lives in
`tests/benchmarks/baseline.json`; timing fields there are reference values with a documented
30% manual tolerance and are not asserted in CI.

| Fixture | Detected nodes | Wall time | Peak RSS | Result |
|---|---:|---:|---:|---|
| `circle.fig` | 2 | 0.03 s | 5.3 MB | Completed |
| `text-group.fig` | 6 | 0.01 s | 5.5 MB | Completed |
| `OpenFigs.fig` | 2 | 0.01 s | 6.2 MB | Completed with warnings |
| `effects-gradient.fig` | 2 | 0.01 s | 5.3 MB | Completed |
| `image.fig` | 2 | 0.01 s | 5.3 MB | Completed |
| `auto-layout.fig` | 3 | 0.01 s | 5.5 MB | Completed with warnings |
| `styles.fig` | 3 | 0.01 s | 5.4 MB | Completed |
| `components.fig` | 9 | 0.01 s | 5.6 MB | Completed |
| `tokens.fig` | 2 | 0.01 s | 5.3 MB | Completed |
| `masks-boolean.fig` | 5 | 0.01 s | 5.3 MB | Completed with warnings |
| `large-synthetic.fig` | 3,300 | 0.23 s | 66.7 MB | Completed; 2.4 MB archive, 3,312 validation checks |
| Local 4,629-node file | 4,445 | 1.09 s | 226.1 MB | Failed status: 1 failed node + 33 unsupported, all reported; validator passed 6,698 checks |

## Batch memory

Release `f2p-cli batch`, 2 workers, same `circle.fig` copied to unique names:

| Files | Peak RSS |
|---:|---:|
| 1 | 5.2 MB |
| 4 | 6.8 MB |
| 16 | 7.0 MB |

Memory does not scale linearly with batch size at a fixed worker count. This is a
single-machine measurement; larger documents and more workers will raise the peak.

## Fuzzing

Command:

```bash
cargo fuzz run read_fig --sanitizer none -- -max_total_time=60 -print_final_stats=1
```

Result on 2026-11-02:

```text
Done 818204 runs in 61 second(s)
stat::average_exec_per_sec: 13413
stat::peak_rss_mb: 49
```

No crash artifacts were produced (`fuzz/artifacts/read_fig` was empty). This is one
60-second campaign, not a proof of absence of panics.

## Regression checks

`crates/f2p-cli/tests/baseline.rs` checks every committed fixture (including the 3,303-node
synthetic file) against `tests/benchmarks/baseline.json`: counts, status, and structural
validation. Timing values
are documented with a 30% wall-time tolerance but are not asserted in CI.

## Known gaps

- Measured scale coverage remains the committed 3,303-node synthetic fixture and one
  historical local 4,629-node file. Five additional private version 106/110 files now pass
  conversion/import checks, but their expanded component working sets have not been
  benchmarked for peak RSS. They are compatibility cases, not new timing baselines.
- Expanded/cache shapes default to 250,000; validator defaults are 300,000 entries,
  64 MiB per entry, and 512 MiB total uncompressed data. These bounds are tested but are
  not a guarantee of acceptable memory use on every 8 GiB machine (ADR-0011).
- Historical fuzz campaigns predate the new embedded-component and override mappings;
  targeted fuzzing of those cases remains Planned.
- Low-disk and cancellation-during-write paths are not exercised. A forced write failure
  (destination is a directory) is now covered and leaves no temporary files.
- Benchmarks use one run each; noise is unquantified.
- The local 4,629-node file is not committed (third-party/private input); only aggregate
  numbers are recorded.
