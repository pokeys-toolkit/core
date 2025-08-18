#!/bin/bash

# Test script to verify pre-commit is working

echo "Testing pre-commit setup..."
echo

echo "1. Testing pre-commit run on all files:"
if pre-commit run --all-files; then
    echo "✅ Pre-commit checks passed!"
else
    echo "❌ Pre-commit checks failed!"
    echo "This is normal if you haven't fixed formatting/linting issues yet."
fi

echo
echo "2. Testing individual hooks:"
echo "   - Cargo format check:"
cargo fmt --check && echo "   ✅ Formatting OK" || echo "   ❌ Needs formatting"

echo "   - Cargo clippy check:"
cargo clippy --all-targets --all-features -- -D warnings >/dev/null 2>&1 && echo "   ✅ Clippy OK" || echo "   ❌ Clippy issues found"

echo "   - Cargo audit check:"
cargo audit >/dev/null 2>&1 && echo "   ✅ Security audit OK" || echo "   ❌ Security issues found"

echo
echo "3. Available commands:"
echo "   - Run all checks: pre-commit run --all-files"
echo "   - Run on staged files: pre-commit run"
echo "   - Commit with checks: ./scripts/commit-with-checks.sh -m 'message'"
echo "   - Push with checks: ./scripts/push-with-checks.sh"
echo "   - Git aliases: git precommit, git pc"
