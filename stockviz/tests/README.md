# Integration tests

Most StockViz tests live in `src/**` (`#[cfg(test)]`, kittest, proptest) for easy navigation with `rg 'r\[verify'`.

| File | Purpose |
|------|---------|
| **`sealed_demo.rs`** | Live **`sealed_test`** demo (`r[talk.sealed.test]`). Run: `cargo nextest run --test sealed_demo` |

Tracey `impl` / `verify` for `talk.sealed.test` are on `quiz_sealed_test_traced` in `src/talk_quiz.rs` (proc-macro tests are not scanned as code units).
