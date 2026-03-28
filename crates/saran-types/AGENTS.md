# AGENTS.md — saran-types Development Notes

## Purpose

This document guides developers maintaining the `saran-types` crate. It documents design decisions, implementation constraints, and workflows.

## Core Design Principles

### 1. **Types Only, No Logic**

`saran-types` contains **only data structure definitions**. No validation, parsing, serialization logic, or business rules.

- **Validation** belongs in `saran-validation`
- **Parsing** belongs in `saran-parser`
- **Serialization** is handled by `serde`
- **Transformation** belongs in `saran-codegen` or `saran-core`

**Why?** Keeps types lightweight, dependency-minimal, and reusable across crates without circular dependencies.

### 2. **Spec Alignment**

Every type definition corresponds directly to a section in `spec/saran-format.md`:

| Type                | Spec Section                   |
| ------------------- | ------------------------------ |
| `WrapperDefinition` | Top-level structure            |
| `Command`           | Command definition             |
| `Action`            | Actions entry                  |
| `OptionalFlag`      | Optional flag definition       |
| `VarDecl`           | vars entry                     |
| `PositionalArg`     | Positional argument definition |
| `CliRequirement`    | requires entry                 |
| `QuotaEntry`        | quotas entry                   |

When the spec changes, update the corresponding type and doc comments immediately.

### 3. **Serde Serialization**

All types use `#[serde(derive)]` for YAML round-tripping. Key patterns:

- **Required fields** — no `Option`, no serde attributes
- **Optional fields** — `Option<T>` + `#[serde(skip_serializing_if = "Option::is_none")]`
- **Defaults** — `#[serde(default)]` or custom default functions
- **Collection fields** — use `Vec<T>` with `#[serde(default)]` to handle omission

Example:

```rust
pub struct VarDecl {
    pub name: String,                           // Required
    #[serde(default)]
    pub required: bool,                         // Optional, defaults to false
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,               // Optional, omitted if None
}
```

### 4. **Ordered Collections**

Commands are stored in `BTreeMap<String, Command>` (not `HashMap`) to:

- Maintain stable iteration order
- Simplify codegen (predictable subcommand order)
- Match YAML serialization order

### 5. **Doc Comments for Spec Reference**

Every public type and field has a doc comment. Doc comments should:

- Explain the field's purpose and usage
- Reference the spec section (e.g., `See: spec/saran-format.md#commands`)
- Include example YAML or Rust if helpful
- Document constraints (e.g., allowed characters, mutual exclusivity)

**Format example:**

```rust
/// The environment variable name (e.g., `GH_REPO`, `REDIS_HOST`).
/// Must satisfy `[A-Za-z_][A-Za-z0-9_]*`.
pub name: String,
```

## Test Coverage

Tests in `src/lib.rs` verify:

1. **Roundtrip serialization** — types serialize to YAML and deserialize identically
2. **Field relationships** — mutually exclusive fields are tested (e.g., `required` vs `default`)
3. **Type variants** — enum variants are tested (e.g., `QuotaLimit::Literal` vs `Variable`)
4. **Field content** — basic invariants (e.g., action args contain substitution references)

Tests are lightweight and focused on **type structure**, not validation logic.

## Adding New Types

When adding a new type to support a spec change:

1. **Read the spec section** thoroughly
2. **Create the struct** with all fields from the spec
3. **Add doc comments** to every field, referencing the spec
4. **Choose field types carefully**:
   - Required fields → no `Option`
   - Optional fields with defaults → use serde `default`
   - Mutual exclusivity → document in comments (validation enforces)
5. **Add a test** verifying serialization or basic invariants
6. **Update spec reference** in AGENTS.md

## Modifying Existing Types

When the spec changes:

1. **Update the struct fields** to match the spec
2. **Update doc comments** to reflect spec changes
3. **Update serde attributes** if serialization rules change
4. **Add/update tests** for new variants or relationships
5. **DO NOT add validation logic** — validation belongs in `saran-validation`

## Dependencies

`saran-types` depends only on:

- **serde** (serialization framework)
- (No other internal crates)

This is intentional. `saran-types` is the **root of the dependency tree** and must remain lightweight.

All other crates depend on `saran-types`; none of the reverse should be true.

## Versioning

`saran-types` is versioned with the workspace (`0.1.0`). When types change in breaking ways, increment the workspace version across all crates.

## Debugging

If types don't round-trip through YAML:

```bash
cd crates/saran-types
cargo test --lib
```

Test output will show serialization/deserialization differences.

To manually test YAML round-tripping:

```rust
let wrapper = WrapperDefinition { ... };
let yaml = serde_yaml::to_string(&wrapper)?;
let roundtrip: WrapperDefinition = serde_yaml::from_str(&yaml)?;
assert_eq!(wrapper, roundtrip);
```

## Future Considerations

- **Validation errors as types** — if validation errors become complex, move to `saran-types` for reuse
- **Custom deserializers** — if YAML schema evolves, custom serde implementations belong here
- **Builder patterns** — if type construction becomes complex, consider builder types
