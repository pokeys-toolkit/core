#!/bin/bash

# Pre-push hook for PoKeys project
# This script runs comprehensive tests before allowing a push
# It prevents pushes if any tests fail

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[PRE-PUSH]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_status "Running pre-push checks..."
echo

# Check if we're pushing to main branch
protected_branch='main'
current_branch=$(git symbolic-ref HEAD | sed -e 's,.*/\(.*\),\1,')

if [ "$current_branch" = "$protected_branch" ]; then
    print_error "Direct push to '$protected_branch' branch is not allowed!"
    print_error "Please create a pull request instead."
    echo
    print_status "To push to main branch (emergency only):"
    print_status "  git push --no-verify origin main"
    echo
    exit 1
fi

# Function to run a test and check result
run_check() {
    local check_name="$1"
    local command="$2"

    print_status "Running $check_name..."

    if eval "$command" > /dev/null 2>&1; then
        print_success "$check_name passed"
        return 0
    else
        print_error "$check_name failed"
        print_error "Command: $command"
        return 1
    fi
}

# Track if any checks failed
failed_checks=0

# Code formatting check
if ! run_check "code formatting" "cargo fmt --all -- --check"; then
    print_error "Code is not properly formatted. Run: cargo fmt"
    ((failed_checks++))
fi

# Clippy linting
if ! run_check "clippy linting" "cargo clippy --all-targets --all-features -- -D warnings"; then
    print_error "Clippy found issues. Run: cargo clippy --fix"
    ((failed_checks++))
fi

# Security audit
if ! run_check "security audit" "cargo audit --deny warnings"; then
    print_error "Security vulnerabilities found. Review and fix them."
    ((failed_checks++))
fi

# Build check
if ! run_check "build" "cargo build --all-targets --all-features"; then
    print_error "Build failed. Fix compilation errors."
    ((failed_checks++))
fi

# Unit tests
if ! run_check "unit tests" "cargo test --test unit_tests"; then
    print_error "Unit tests failed. Fix failing tests."
    ((failed_checks++))
fi

# Protocol tests
if ! run_check "protocol tests" "cargo test --test protocol_tests"; then
    print_error "Protocol tests failed. Fix failing tests."
    ((failed_checks++))
fi

# Integration tests
if ! run_check "integration tests" "cargo test --test integration_tests"; then
    print_error "Integration tests failed. Fix failing tests."
    ((failed_checks++))
fi

# Documentation tests
if ! run_check "documentation tests" "cargo test --doc"; then
    print_error "Documentation tests failed. Fix doc examples."
    ((failed_checks++))
fi

# Documentation build
if ! run_check "documentation build" "cargo doc --no-deps --document-private-items --all-features"; then
    print_error "Documentation build failed. Fix doc issues."
    ((failed_checks++))
fi

echo

# Summary
if [ $failed_checks -eq 0 ]; then
    print_success "All pre-push checks passed! ✅"
    print_status "Push is allowed to proceed."
    echo
    exit 0
else
    print_error "❌ $failed_checks check(s) failed!"
    print_error "Push is blocked until all issues are resolved."
    echo
    print_status "To fix issues:"
    print_status "  1. Run the failing commands shown above"
    print_status "  2. Fix any issues found"
    print_status "  3. Commit your fixes"
    print_status "  4. Try pushing again"
    echo
    print_status "To bypass this check (NOT RECOMMENDED):"
    print_status "  git push --no-verify"
    echo
    exit 1
fi
