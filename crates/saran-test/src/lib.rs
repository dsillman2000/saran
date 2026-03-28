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
/// saran_test!("XX-01", test_my_feature, {
///     assert_eq!(1, 1);
/// });
/// ```
///
/// # Output (with --nocapture)
///
/// ```ignore
/// test tests::test_my_feature ... [XX-01] ok
/// ```
///
/// Note: Tests must run with `--test-threads=1` to prevent output interleaving.
#[macro_export]
macro_rules! saran_test {
    ($tag:expr, $name:ident, $body:block) => {
        #[test]
        fn $name() {
            print!("[{}] ", $tag);
            $body
        }
    };
}
