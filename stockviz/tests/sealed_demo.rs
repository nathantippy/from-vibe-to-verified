//! Live `sealed_test` demo (subprocess isolation). Tracey tags: `r[talk.sealed.test]` in `src/talk_quiz.rs`.

use sealed_test::prelude::*;

#[sealed_test(env = [("STOCKVIZ_SEALED_DEMO", "isolated")])]
fn quiz_sealed_env_isolated() {
    assert_eq!(std::env::var("STOCKVIZ_SEALED_DEMO").unwrap(), "isolated");
}
