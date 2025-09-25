#!/bin/bash

# Commit wrapper that runs pre-commit checks first

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_status "Running pre-commit checks..."

if pre-commit run --all-files; then
    print_success "All pre-commit checks passed!"

    print_status "Proceeding with commit..."
    git commit "$@"

    print_success "Commit completed successfully!"
else
    print_error "Pre-commit checks failed!"
    print_error "Please fix the issues above and try again."
    print_status "You can run 'pre-commit run --all-files' to see all issues."
    exit 1
fi
