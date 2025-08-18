#!/bin/bash

# Push wrapper that runs comprehensive checks first

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

print_status "Running comprehensive pre-push checks..."

# Run the pre-push hook script
if ./scripts/pre-push-hook.sh; then
    print_success "All pre-push checks passed!"

    print_status "Proceeding with push..."
    git push "$@"

    print_success "Push completed successfully!"
else
    print_error "Pre-push checks failed!"
    print_error "Please fix the issues above and try again."
    exit 1
fi
