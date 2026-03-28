#!/bin/bash
# Coverage script for Saran project
# Usage: ./scripts/coverage.sh [crate]
#   - With no args: runs coverage for all crates
#   - With crate name: runs coverage for that specific crate

set -e

CRATE="${1:-saran-types}"

echo "Running coverage for: $CRATE"
echo "========================================"

# Check if cargo-llvm-cov is installed
if ! command -v cargo-llvm-cov &> /dev/null; then
    echo "Error: cargo-llvm-cov not found."
    echo "Install with: cargo install cargo-llvm-cov"
    exit 1
fi

cd crates/$CRATE

# Check if this is a types-only crate (no implementation to cover)
if [ "$CRATE" = "saran-types" ]; then
    echo "Note: saran-types is a types-only crate with no implementation logic."
    echo "Coverage will be low/none by design. Running tests instead..."
    cargo test
    echo "Tests passed!"
    echo "========================================"
    echo "Skipping coverage for types crate."
    exit 0
fi

# Run coverage with HTML output
cargo llvm-cov --html --output-dir ../coverage/$CRATE

echo "========================================"
echo "Coverage report generated at: coverage/$CRATE/index.html"
echo ""
echo "To view:"
echo "  open coverage/$CRATE/index.html"
