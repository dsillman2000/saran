#!/bin/bash
# Coverage script for Saran project
# Usage: ./scripts/coverage.sh [crate]
#   - With no args: runs coverage for all crates
#   - With crate name: runs coverage for that specific crate

set -e

# Function to run coverage for a single crate
run_coverage_for_crate() {
    local CRATE="$1"

    echo "Running coverage for: $CRATE"
    echo "========================================"

    # Check if cargo-llvm-cov is installed (only once, on first run)
    if [ "$FIRST_RUN" = "true" ]; then
        if ! command -v cargo-llvm-cov &> /dev/null; then
            echo "Error: cargo-llvm-cov not found."
            echo "Install with: cargo install cargo-llvm-cov"
            exit 1
        fi
        FIRST_RUN="false"
    fi

    cd crates/$CRATE

    # Check if this is a types-only crate (no implementation to cover)
    if [ "$CRATE" = "saran-types" ] || [ "$CRATE" = "saran-test" ]; then
        echo "Note: $CRATE is a utility/types-only crate with minimal implementation logic."
        echo "Coverage will be low by design. Running tests instead..."
        cargo test
        echo "Tests passed!"
        echo "========================================"
        echo "Skipping coverage for $CRATE."
        cd - > /dev/null
        return
    fi

    # Run coverage with HTML output
    cargo llvm-cov --html --output-dir ../coverage/$CRATE

    echo "========================================"
    echo "Coverage report generated at: coverage/$CRATE/index.html"
    echo ""
    cd - > /dev/null
}

FIRST_RUN="true"

# If a specific crate is provided, run coverage for that crate only
if [ -n "$1" ]; then
    CRATE="$1"
    run_coverage_for_crate "$CRATE"
    echo ""
    echo "To view:"
    echo "  open coverage/$CRATE/index.html"
    exit 0
fi

# Otherwise, discover and run coverage for all crates
echo "Discovering all crates in crates/ directory..."
CRATES=$(find crates -maxdepth 1 -mindepth 1 -type d ! -name "coverage" -exec basename {} \; | sort)

if [ -z "$CRATES" ]; then
    echo "No crates found in crates/ directory."
    exit 1
fi

echo "Found crates: $CRATES"
echo "========================================"
echo ""

# Run coverage for each crate
for CRATE in $CRATES; do
    run_coverage_for_crate "$CRATE"
    echo ""
done

echo "========================================"
echo "Coverage complete for all crates!"
