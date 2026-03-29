#!/bin/bash
# Coverage script for Saran project
# Usage: ./scripts/coverage.sh
# Runs coverage for all crates and displays a summary table

set -e

# Check if cargo-llvm-cov is installed
if ! command -v cargo-llvm-cov &> /dev/null; then
    echo "Error: cargo-llvm-cov not found."
    echo "Install with: cargo install cargo-llvm-cov"
    exit 1
fi

# Discover all crates in crates/ directory
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
    echo "Running coverage for: $CRATE"
    echo "========================================"

    cd crates/$CRATE

    # Check if this is a types-only crate (no implementation to cover)
    if [ "$CRATE" = "saran-types" ] || [ "$CRATE" = "saran-test" ]; then
        echo "Note: $CRATE is a utility/types-only crate with minimal implementation logic."
        echo "Coverage will be low by design. Running tests instead..."
        cargo test
        echo "Tests passed!"
        cd - > /dev/null
        echo ""
        continue
    fi

    # Run coverage with summary-only output
    cargo llvm-cov --summary-only -q

    cd - > /dev/null
    echo ""
done

# Run coverage once at workspace level and extract JSON for final summary
echo "========================================"
echo "COVERAGE SUMMARY FOR ALL CRATES"
echo "========================================"

JSON_OUTPUT=$(cargo llvm-cov --summary-only -q --json 2>/dev/null | tail -1)

if [ -z "$JSON_OUTPUT" ]; then
    echo "No coverage data available"
    exit 1
fi

# Parse JSON to get totals
TOTALS=$(echo "$JSON_OUTPUT" | jq -r '
    .data[0].totals.regions.percent | floor | tostring
')
TOTAL_LINES=$(echo "$JSON_OUTPUT" | jq -r '
    .data[0].totals.lines.percent | floor | tostring
')
TOTAL_FUNCS=$(echo "$JSON_OUTPUT" | jq -r '
    .data[0].totals.functions.percent | floor | tostring
')

# Display header
echo "Timestamp: $(echo "$JSON_OUTPUT" | jq -r '.cargo_llvm_cov.manifest_path') coverage report"
echo ""
echo "TOTALS:"
printf "  %-12s %s%%\n" "Regions:" "$TOTALS"
printf "  %-12s %s%%\n" "Lines:" "$TOTAL_LINES"
printf "  %-12s %s%%\n" "Functions:" "$TOTAL_FUNCS"
echo ""
echo "FILES:"
printf "  %-48s %8s %8s %8s\n" "File" "Regions" "Lines" "Functions"

# Display each file with aligned columns using printf
echo "$JSON_OUTPUT" | jq -r '.data[0].files[] |
    .filename as $full |
    ($full | split("/home/dsillman2000/rust-projects/saran/")[-1]) as $short |
    (.summary.regions.percent | floor | tostring) as $r |
    (.summary.lines.percent | floor | tostring) as $l |
    (.summary.functions.percent | floor | tostring) as $f |
    "  " + $short + "|" + $r + "%|" + $l + "%|" + $f + "%"
' | while IFS='|' read -r file regions lines functions; do
    printf "  %-48s %7s %7s %8s\n" "$file" "$regions" "$lines" "$functions"
done
