#!/bin/bash

# Script to set up GitHub branch protection rules
# This script configures branch protection for the main branch

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

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    print_error "GitHub CLI (gh) is not installed"
    print_error "Please install it from: https://cli.github.com/"
    exit 1
fi

# Check if user is authenticated
if ! gh auth status &> /dev/null; then
    print_error "You are not authenticated with GitHub CLI"
    print_error "Please run: gh auth login"
    exit 1
fi

# Get repository information
REPO_OWNER=$(gh repo view --json owner --jq .owner.login)
REPO_NAME=$(gh repo view --json name --jq .name)

if [ -z "$REPO_OWNER" ] || [ -z "$REPO_NAME" ]; then
    print_error "Could not determine repository information"
    print_error "Make sure you're in a GitHub repository directory"
    exit 1
fi

print_status "Setting up branch protection for $REPO_OWNER/$REPO_NAME"
echo

# Function to set up branch protection
setup_branch_protection() {
    local branch="$1"

    print_status "Configuring branch protection for '$branch' branch..."

    # Create the branch protection rule
    gh api \
        --method PUT \
        -H "Accept: application/vnd.github+json" \
        -H "X-GitHub-Api-Version: 2022-11-28" \
        "/repos/$REPO_OWNER/$REPO_NAME/branches/$branch/protection" \
        --input - << EOF
{
  "required_status_checks": {
    "strict": true,
    "contexts": [
      "Code Formatting",
      "Clippy Linting",
      "Security Audit",
      "Test Suite (ubuntu-latest, stable)",
      "Test Suite (windows-latest, stable)",
      "Test Suite (macos-latest, stable)",
      "Documentation",
      "Minimum Supported Rust Version",
      "Code Coverage",
      "Dependency Check",
      "CI Success"
    ]
  },
  "enforce_admins": true,
  "required_pull_request_reviews": {
    "required_approving_review_count": 1,
    "dismiss_stale_reviews": true,
    "require_code_owner_reviews": true,
    "require_last_push_approval": true
  },
  "restrictions": null,
  "allow_force_pushes": false,
  "allow_deletions": false,
  "block_creations": false,
  "required_conversation_resolution": true,
  "lock_branch": false,
  "allow_fork_syncing": true
}
EOF

    if [ $? -eq 0 ]; then
        print_success "Branch protection configured for '$branch'"
    else
        print_error "Failed to configure branch protection for '$branch'"
        return 1
    fi
}

# Set up protection for main branch
setup_branch_protection "main"

echo
print_status "Branch protection rules configured:"
echo "  ✓ Require pull request reviews (1 approving review)"
echo "  ✓ Dismiss stale reviews when new commits are pushed"
echo "  ✓ Require review from code owners"
echo "  ✓ Require approval of the most recent reviewable push"
echo "  ✓ Require status checks to pass before merging"
echo "  ✓ Require branches to be up to date before merging"
echo "  ✓ Require conversation resolution before merging"
echo "  ✓ Enforce restrictions for administrators"
echo "  ✓ Prevent force pushes"
echo "  ✓ Prevent branch deletion"
echo

print_status "Required status checks:"
echo "  • Code Formatting (cargo fmt)"
echo "  • Clippy Linting (cargo clippy)"
echo "  • Security Audit (cargo audit)"
echo "  • Test Suite (Ubuntu, Windows, macOS)"
echo "  • Documentation Build"
echo "  • MSRV Compatibility Check"
echo "  • Code Coverage"
echo "  • Dependency Check"
echo "  • Overall CI Success"
echo

print_success "Branch protection setup complete!"
echo

print_warning "Important notes:"
echo "  • Direct pushes to 'main' branch are now blocked"
echo "  • All changes must go through pull requests"
echo "  • All status checks must pass before merging"
echo "  • At least 1 approving review is required"
echo "  • Administrators are also subject to these rules"
echo

print_status "To make changes to the main branch:"
echo "  1. Create a feature branch: git checkout -b feature/your-feature"
echo "  2. Make your changes and commit them"
echo "  3. Push the branch: git push origin feature/your-feature"
echo "  4. Create a pull request on GitHub"
echo "  5. Wait for CI checks to pass and get approval"
echo "  6. Merge the pull request"
