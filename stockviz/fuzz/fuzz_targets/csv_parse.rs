//! CSV parse fuzz harness (**`r[test.fuzz.csv]`**).
//!
//! Exercises safe rejection (no panic) for malformed headers, non-finite OHLCV,
//! negative volume, unsorted/duplicate dates, invalid UTF-8, and oversized inputs
//! per **`r[data.validation]`**. **`Result::Err` is OK; panic is not.**

// r[impl test.fuzz.csv]
// r[impl talk.fuzz.setup]
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = stockviz::data::parse_csv_bytes(data);
});
