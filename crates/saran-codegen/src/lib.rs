//! Code generation for Saran CLI wrappers.
//!
//! This crate transforms validated `WrapperDefinition` types into complete, compilable Rust source
//! code for `clap`-powered CLI binaries.
//!
//! # Overview
//!
//! The `saran-codegen` crate is responsible for the **code generation phase** of the wrapper build process.
//! It takes a pre-validated `WrapperDefinition` (from `saran-parser`) and generates:
//!
//! - A complete `main.rs` file with clap CLI parsing, variable resolution, and command routing
//! - A `Cargo.toml` manifest for the wrapper project
//!
//! # Input & Output
//!
//! **Input:** A validated `WrapperDefinition`
//! **Output:** A tuple of strings `(main_rs_source, cargo_toml_source)`
//!
//! # Design Assumptions
//!
//! 1. **Input is Pre-Validated:** All `WrapperDefinition` inputs have already passed validation in
//!    `saran-parser`. Codegen does NOT re-validate.
//! 2. **saran-core Functions Exist:** The generated code calls functions from `saran-core` that
//!    already exist and are complete (e.g., `resolve_wrapper_vars()`, `exec_action()`).
//! 3. **No Compilation:** Generated code is returned as strings. Compilation and file I/O are
//!    orchestrated by the `saran` CLI, not by codegen.
//! 4. **Rust Syntax Correctness:** Cargo validates the generated Rust syntax at build time.
//!    Codegen does NOT perform compile-time checks.
//! 5. **Fixed Dependency Versions:** Generated wrappers always link against the same `saran-core`
//!    version as the `saran` CLI that generated them.
//!
//! # Error Handling
//!
//! All code generation errors are represented by the [`CodegenError`] enum. Since input is
//! pre-validated, any codegen errors indicate bugs in the codegen implementation itself,
//! not user error. Therefore:
//!
//! - All errors should include clear, actionable messages for debugging
//! - No `panic!()` calls; all failures use `Result<T, CodegenError>`
//! - Error recovery is not expected; callers should propagate errors to the user
//!
//! # Example
//!
//! ```ignore
//! use saran_codegen::generate;
//! use saran_parser::parse_wrapper;
//!
//! let yaml = std::fs::read_to_string("my-wrapper.yaml")?;
//! let wrapper_def = parse_wrapper(&yaml)?;  // Already validated
//!
//! let (main_rs, cargo_toml) = generate(&wrapper_def)?;
//! println!("Generated main.rs:\n{}", main_rs);
//! println!("Generated Cargo.toml:\n{}", cargo_toml);
//! ```

use saran_types::WrapperDefinition;
use std::collections::BTreeSet;
use std::fmt;

/// Errors that can occur during code generation.
///
/// Code generation errors indicate either invalid input or bugs in the codegen implementation.
/// Since input is pre-validated by `saran-parser`, any codegen error is effectively a bug
/// in the codegen logic itself.
#[derive(Debug, Clone)]
pub enum CodegenError {
    /// The wrapper definition is invalid in a way that codegen cannot handle.
    ///
    /// This should not occur in normal operation, since `saran-parser` validates all inputs
    /// before they reach codegen. If this error occurs, it indicates either:
    ///
    /// - A bug in the parser that missed a validation rule
    /// - A bug in codegen that failed to handle a valid input
    /// - A mismatch between parser validation and codegen expectations
    InvalidWrapperDefinition(String),

    /// A bug in the code generation templates or string building logic.
    ///
    /// This error indicates that the codegen implementation has a defect that produces
    /// syntactically invalid Rust code. Examples include:
    ///
    /// - Unclosed braces or parentheses
    /// - Invalid clap attribute syntax
    /// - Malformed string interpolation
    /// - Invalid Cargo.toml TOML syntax
    TemplateSyntaxError(String),

    /// An unexpected internal state was encountered during code generation.
    ///
    /// This error indicates a logic bug in the codegen implementation where an invariant
    /// was violated or an unexpected code path was taken. Examples include:
    ///
    /// - A required component (handler, CLI struct, etc.) is missing after assembly
    /// - Command metadata is corrupted or inconsistent
    /// - Vector or map operations failed unexpectedly
    InternalError(String),
}

impl fmt::Display for CodegenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodegenError::InvalidWrapperDefinition(msg) => {
                write!(f, "Invalid wrapper definition: {}", msg)
            }
            CodegenError::TemplateSyntaxError(msg) => {
                write!(f, "Template syntax error: {}", msg)
            }
            CodegenError::InternalError(msg) => {
                write!(f, "Internal codegen error: {}", msg)
            }
        }
    }
}

impl std::error::Error for CodegenError {}

/// Generate Rust source code and Cargo.toml content for a wrapper.
///
/// Takes a pre-validated wrapper definition and produces:
///
/// - `main_rs`: Complete Rust source code for the wrapper binary
/// - `cargo_toml`: Valid Cargo.toml manifest for the wrapper project
///
/// # Arguments
///
/// * `wrapper_def` - A validated `WrapperDefinition`. Must have passed validation in `saran-parser`.
///
/// # Returns
///
/// * `Ok((main_rs, cargo_toml))` - Tuple of generated source code strings
/// * `Err(CodegenError)` - Code generation failed
///
/// # Errors
///
/// Returns `CodegenError` if:
///
/// - The wrapper definition contains data that codegen cannot process
/// - Generated code templates have internal syntax errors
/// - Unexpected state is encountered during generation
///
/// # Example
///
/// ```ignore
/// use saran_codegen::generate;
/// use saran_parser::parse_wrapper;
///
/// let yaml = std::fs::read_to_string("my-wrapper.yaml")?;
/// let wrapper_def = parse_wrapper(&yaml)?;  // Validates here
/// let (main_rs, cargo_toml) = generate(&wrapper_def)?;
/// ```
pub fn generate(_wrapper_def: &WrapperDefinition) -> Result<(String, String), CodegenError> {
    // TODO: Implement code generation phases
    // This is a stub for Phase 1 infrastructure.
    // Implementation will be added in Phase 2+ (M2.1 through M3.2).

    // For now, return a placeholder that satisfies the signature
    Err(CodegenError::InternalError(
        "Code generation not yet implemented".to_string(),
    ))
}

/// M2.2: Generate the variable declarations function.
///
/// Creates Rust code that returns a `Vec<VarDecl>` containing all variable declarations
/// from the wrapper definition.
///
/// # Arguments
///
/// * `wrapper_def` - The wrapper definition containing the variables to declare
///
/// # Returns
///
/// Rust function code as a string:
/// ```ignore
/// fn get_var_declarations() -> Vec<VarDecl> {
///     vec![
///         VarDecl { name: "VAR1".to_string(), required: true, default: None },
///         VarDecl { name: "VAR2".to_string(), required: false, default: Some("value".to_string()) },
///     ]
/// }
/// ```
///
/// # Error Handling
///
/// Returns `InternalError` if the wrapper definition is corrupted in an unexpected way.
/// Since input is pre-validated, such errors indicate bugs in codegen.
#[allow(dead_code)]
pub(crate) fn generate_var_declarations(
    wrapper_def: &WrapperDefinition,
) -> Result<String, CodegenError> {
    let mut code = String::from("fn get_var_declarations() -> Vec<VarDecl> {\n");
    code.push_str("    vec![\n");

    for var_decl in &wrapper_def.vars {
        code.push_str("        VarDecl {\n");
        code.push_str(&format!(
            "            name: \"{}\".to_string(),\n",
            var_decl.name
        ));
        code.push_str(&format!("            required: {},\n", var_decl.required));

        // Handle the default field
        if let Some(default_val) = &var_decl.default {
            code.push_str(&format!(
                "            default: Some(\"{}\".to_string()),\n",
                escape_string(default_val)
            ));
        } else {
            code.push_str("            default: None,\n");
        }

        code.push_str("        },\n");
    }

    code.push_str("    ]\n");
    code.push_str("}\n");

    Ok(code)
}

/// Helper function to escape strings for Rust string literals.
///
/// Escapes backslashes and double quotes to produce valid Rust string literals.
#[allow(dead_code)]
pub(crate) fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// M2.1: Generate the clap CLI structure (Parser struct and Commands enum).
///
/// Creates Rust code for the `Cli` struct with `#[derive(Parser)]` and the `Commands` enum
/// with `#[derive(Subcommand)]`. Each command becomes an enum variant with positional args
/// and optional flags as struct fields.
///
/// # Arguments
///
/// * `wrapper_def` - The wrapper definition containing commands and arguments
///
/// # Returns
///
/// A tuple of `(cli_struct_code, commands_enum_code)` as strings
///
/// # Error Handling
///
/// Returns `InvalidWrapperDefinition` if duplicate arg/flag names exist in a command,
/// or `TemplateSyntaxError` if reserved clap keywords are detected.
#[allow(dead_code)]
pub(crate) fn generate_cli_struct(
    wrapper_def: &WrapperDefinition,
) -> Result<(String, String), CodegenError> {
    // Generate the Cli struct
    let mut cli_code = String::from("#[derive(Parser)]\n");
    cli_code.push_str(&format!("#[command(name = \"{}\")]\n", wrapper_def.name));
    cli_code.push_str(&format!(
        "#[command(version = \"{}\")]\n",
        wrapper_def.version
    ));

    // Build the about text
    let about_text = wrapper_def.help.as_deref().unwrap_or("");
    cli_code.push_str(&format!(
        "#[command(about = \"{}\")]\n",
        escape_string(about_text)
    ));

    cli_code.push_str("#[command(author = \"Generated by Saran\")]\n");
    cli_code.push_str("struct Cli {\n");
    cli_code.push_str("    #[command(subcommand)]\n");
    cli_code.push_str("    command: Commands,\n");
    cli_code.push_str("}\n\n");

    // Generate the Commands enum
    let mut enum_code = String::from("#[derive(Subcommand)]\n");
    enum_code.push_str("enum Commands {\n");

    for (command_name, command) in &wrapper_def.commands {
        // Add help text for the command
        if let Some(help) = &command.help {
            enum_code.push_str(&format!("    /// {}\n", escape_string(help)));
        }

        // Generate the enum variant name (PascalCase from kebab-case)
        let variant_name = to_pascal_case(command_name);

        // If the command has no args, it's a simple unit variant
        if command.args.is_empty() {
            enum_code.push_str(&format!("    {},\n", variant_name));
        } else {
            // Generate a struct variant with positional args and optional flags
            enum_code.push_str(&format!("    {} {{\n", variant_name));

            // Collect all arg/flag names to detect duplicates
            let mut seen_names = BTreeSet::new();

            // Generate positional arguments
            for arg in &command.args {
                // Use the arg's display name for clap, but store in var_name field
                let field_name = arg.name.replace('-', "_");

                if !seen_names.insert(field_name.clone()) {
                    return Err(CodegenError::InvalidWrapperDefinition(format!(
                        "Duplicate argument name '{}' in command '{}'",
                        arg.name, command_name
                    )));
                }

                // Add help text if present
                if let Some(help) = &arg.help {
                    enum_code.push_str(&format!("        /// {}\n", escape_string(help)));
                }

                // Positional args have #[arg()] attribute
                if arg.required {
                    enum_code.push_str("        #[arg(required = true)]\n");
                } else {
                    enum_code.push_str("        #[arg(required = false)]\n");
                }

                enum_code.push_str(&format!("        {}: String,\n", field_name));
            }

            // Generate optional flags (from actions' optional_flags)
            for action in &command.actions {
                for flag in &action.optional_flags {
                    // Use the flag name without dashes as the field name
                    let field_name = flag.name.trim_start_matches('-').replace('-', "_");

                    if !seen_names.insert(field_name.clone()) {
                        return Err(CodegenError::InvalidWrapperDefinition(format!(
                            "Duplicate flag name '{}' in command '{}'",
                            flag.name, command_name
                        )));
                    }

                    // Add help text if present
                    if let Some(help) = &flag.help {
                        enum_code.push_str(&format!("        /// {}\n", escape_string(help)));
                    }

                    // Generate clap attribute for the flag
                    enum_code
                        .push_str(&format!("        #[arg(long = \"{}\")]\n", &flag.name[2..])); // Remove -- prefix

                    // Generate the field type based on flag_type and repeated
                    let field_type = match flag.flag_type.as_str() {
                        "bool" => "bool".to_string(),
                        "str" => {
                            if flag.repeated {
                                "Vec<String>".to_string()
                            } else {
                                "Option<String>".to_string()
                            }
                        }
                        "int" => {
                            if flag.repeated {
                                "Vec<i32>".to_string()
                            } else {
                                "Option<i32>".to_string()
                            }
                        }
                        "enum" => {
                            if flag.repeated {
                                "Vec<String>".to_string()
                            } else {
                                "Option<String>".to_string()
                            }
                        }
                        _ => "Option<String>".to_string(), // Fallback
                    };

                    enum_code.push_str(&format!("        {}: {},\n", field_name, field_type));
                }
            }

            enum_code.push_str("    },\n");
        }
    }

    enum_code.push_str("}\n");

    Ok((cli_code, enum_code))
}

/// Convert kebab-case to PascalCase
///
/// Example: "pull-request" -> "PullRequest"
fn to_pascal_case(s: &str) -> String {
    s.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests;
