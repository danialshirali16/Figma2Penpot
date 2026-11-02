# Conversion baseline

Reviewed **2026-11-02**. The baseline still passes with the compatibility changes, and
its expected counts/timings were not changed. The five private compatibility files are not
redistributable fixtures or new performance baselines; their import evidence is described
in `../../docs/MANUAL_QA.md` §14.

`baseline.json` records the expected conversion counts, status, and reference performance
for every committed fixture. The automated check is
`crates/f2p-cli/tests/baseline.rs`:

```bash
cargo test -p f2p-cli --test baseline
```

It runs the real `f2p-cli convert` binary on each fixture in a temporary directory, compares
detected/converted/approximated/skipped/unsupported/failed counts and file status, and runs
the structural validator on every generated archive.

## Performance fields

`wallMs` and `peakRssMb` are reference measurements from the machine recorded in
`docs/PERFORMANCE.md`, with a documented **30% wall-time tolerance** for manual comparison.
They are **not** asserted in CI: CI runner speed varies, and timing tests would be flaky.
The baseline check asserts correctness only.

Update the counts/status deliberately when a conversion change legitimately changes them,
and explain why in the commit and `PROGRESS.md`. Update performance numbers only with a new
measurement on a named machine.
