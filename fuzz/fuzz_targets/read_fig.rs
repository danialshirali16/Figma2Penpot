#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Tiny limits keep each iteration fast; the goal is parser safety, not
    // decoding a real document.
    let limits = fig_reader::Limits::for_fuzzing();
    let _ = fig_reader::read_fig_bytes("fuzz.fig", data, &limits);
});
