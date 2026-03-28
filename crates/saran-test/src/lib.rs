//! Test utilities for Saran crates.
//!
//! Provides macros for tagged tests that print specification IDs during test execution.

/// A test macro that prints a tag before the test runs.
///
/// Useful for correlating tests with specification documents. The tag is printed
/// to stdout when tests are run with `--nocapture`.
///
/// # Usage
///
/// ```ignore
/// use saran_test::saran_test;
///
/// saran_test!("TP-01", test_my_feature, {
///     assert_eq!(1, 1);
/// });
/// ```
///
/// # Output (with --nocapture)
///
/// ```
/// test tests::test_my_feature ... [TP-01] ok
/// ```
#[macro_export]
macro_rules! saran_test {
    ($tag:expr, $name:ident, $body:block) => {
        #[test]
        fn $name() {
            use std::io::Write;
            print!("[{}] ", $tag);
            std::io::stdout().flush().ok();
            $body
        }
    };
}
