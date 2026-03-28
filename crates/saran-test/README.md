# saran-test

Test utilities for Saran crates.

## Purpose

Provides test macros for tagged tests that print specification IDs during test execution. Useful for correlating tests with specification documents.

## Usage

```rust
use saran_test::saran_test;

saran_test!("TP-01", test_my_feature, {
    assert_eq!(1, 1);
});
```

## Output (with `--nocapture`)

```
test tests::test_my_feature ... [TP-01] ok
```

## Adding to a Crate

Add to `Cargo.toml`:

```toml
[dev-dependencies]
saran-test.workspace = true
```

Import in test files:

```rust
use saran_test::saran_test;
```
