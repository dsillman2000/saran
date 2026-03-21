# Saran Test Specification

This directory contains test specifications and cases for Saran. The project follows a test-driven development (TDD) approach: tests are written first to define expected behavior, then implementation follows.

## Unit Test Specifications

Detailed unit test plans are in [`unit/`](./unit/):

- **[`unit/README.md`](./unit/README.md)** — Overview of 108 unit tests across 5 domains
- **[`unit/01-yaml-validation.md`](./unit/01-yaml-validation.md)** — 59 tests for YAML schema validation
- **[`unit/02-token-parsing.md`](./unit/02-token-parsing.md)** — 6 tests for `$VAR_NAME` token parsing
- **[`unit/03-variable-resolution.md`](./unit/03-variable-resolution.md)** — 14 tests for variable priority chain
- **[`unit/04-substitution-resolution.md`](./unit/04-substitution-resolution.md)** — 10 tests for value substitution
- **[`unit/05-argument-assembly.md`](./unit/05-argument-assembly.md)** — 19 tests for child process argv assembly

See [`unit/DEPENDENCIES.md`](./unit/DEPENDENCIES.md) for dependency analysis and [`unit/IMPLEMENTATION.md`](./unit/IMPLEMENTATION.md) for the 3-week implementation plan.

## Approach

- **Tests first** — Define expected behavior through concrete test cases before implementation
- **Spec-aligned** — Tests correspond directly to rules in the Saran specification documents
- **Descriptive names** — Test names explain what is being validated
- **Implementation-agnostic** — Tests define interfaces and behavior, not internal implementation details
