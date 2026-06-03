//! Structured pipeline fuzz (**`r[test.fuzz.pipeline]`**).
//!
//! LibFuzzer supplies `&[u8]`; we decode [`PipelineFuzzInput`] via shared
//! [`Arbitrary`](stockviz::test_inputs) (legacy wire format). See `docs/ARBITRARY_TESTING.md`.

// r[impl test.fuzz.pipeline]
// r[impl test.arbitrary.shared]
// r[impl talk.fuzz.setup]
#![no_main]

use arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;
use stockviz::test_inputs::PipelineFuzzInput;

fuzz_target!(|data: &[u8]| {
    if let Ok(input) = PipelineFuzzInput::from_legacy_bytes(data) {
        stockviz::fuzz_harness::exercise_pipeline_input(&input);
        return;
    }
    let mut u = Unstructured::new(data);
    if let Ok(input) = <PipelineFuzzInput as Arbitrary>::arbitrary(&mut u) {
        stockviz::fuzz_harness::exercise_pipeline_input(&input);
    }
});
