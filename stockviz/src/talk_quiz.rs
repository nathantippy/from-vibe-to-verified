//! Talk quiz examples — Rust testing gotchas (grep `r[talk.quiz` or `r[talk.cargo.test.unit]`).

// r[impl talk.cargo.test.unit]
// r[impl talk.sealed.test]
// r[impl talk.quiz.q1]
// r[impl talk.quiz.q2]
// r[impl talk.quiz.q3]
// r[impl talk.quiz.q4]
// r[impl talk.quiz.q5]
// r[impl talk.quiz.q6]
// r[impl talk.quiz.q7]

// r[impl talk.quiz.q4]
#[cfg(test)]
fn private_double(n: i32) -> i32 {
    n * 2
}

/// Demo function for documentation tests (quiz Q6).
///
/// ```
/// use stockviz::talk_quiz::demo_add;
/// assert_eq!(demo_add(2, 2), 4);
/// ```
// r[impl talk.quiz.q6]
pub fn demo_add(a: i32, b: i32) -> i32 {
    a + b
}

// r[impl talk.cargo.test.unit]
#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::private_double;

    // r[impl talk.quiz.q1]
    // r[verify talk.cargo.test.unit]
    // r[verify talk.quiz.q1]
    #[test]
    fn quiz_q1_cfg_test_runs() {
        assert_eq!(2 + 2, 4);
    }

    // r[impl talk.quiz.q2]
    // r[verify talk.quiz.q2]
    // Live fail demo: change `expected` to `"division by zero"` then `cargo nextest run -- --ignored`.
    #[test]
    #[ignore = "talk demo: run with cargo nextest run -- --ignored"]
    #[should_panic(expected = "attempt to divide by zero")]
    fn quiz_q2_should_panic_demo() {
        let zero: i32 = std::hint::black_box(0);
        let _ = 10 / zero;
    }

    // r[impl talk.quiz.q3]
    // r[verify talk.quiz.q3]
    /// Returning `Result` from tests is idiomatic; using `?` on `open` would fail the test with this error printed.
    #[test]
    fn quiz_q3_result_err_path() -> Result<(), Box<dyn std::error::Error>> {
        let path = std::path::Path::new("nonexistent_stockviz_talk_quiz_file.txt");
        let err = std::fs::File::open(path).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
        Ok(())
    }

    // r[impl talk.quiz.q4]
    // r[verify talk.quiz.q4]
    #[test]
    fn quiz_q4_private_helper() {
        assert_eq!(private_double(5), 10);
    }

    // r[impl talk.quiz.q5]
    // r[verify talk.quiz.q5]
    #[test]
    #[ignore = "expensive database test — run: cargo nextest run -- --ignored"]
    fn quiz_q5_ignored_expensive() {
        // Placeholder for `cargo nextest run -- --ignored` demo.
    }

    // r[impl talk.quiz.q6]
    // r[verify talk.quiz.q6]
    #[test]
    fn quiz_q6_demo_add() {
        // Mutation testing needs discriminating assertions: (2,2) alone cannot kill + → *.
        assert_eq!(super::demo_add(2, 2), 4);
        assert_eq!(super::demo_add(2, 3), 5);
    }

    // r[impl talk.quiz.q7]
    // r[verify talk.quiz.q7]
    #[test]
    fn quiz_q7_refcell_counter() {
        let counter = RefCell::new(0);
        *counter.borrow_mut() += 1;
        assert_eq!(*counter.borrow(), 1);
    }

    // r[impl talk.sealed.test]
    // r[verify talk.sealed.test]
    #[test]
    fn quiz_sealed_test_traced() {
        // Subprocess + env isolation: `cargo nextest run -p stockviz --test sealed_demo`
        assert!(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/sealed_demo.rs")
                .is_file()
        );
    }
}
