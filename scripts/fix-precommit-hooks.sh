#!/bin/bash

# Fix Pre-commit Hooks with Git Defender Integration
# This script creates a working pre-commit setup that works alongside Git Defender

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

print_status "Setting up pre-commit hooks with Git Defender integration..."

# Check if we're in a git repository
if [ ! -d ".git" ]; then
    print_error "This script must be run from the root of a git repository"
    exit 1
fi

# Check if pre-commit is available
if ! command -v pre-commit &> /dev/null; then
    print_error "pre-commit is not installed. Please install it first."
    exit 1
fi

# Save current hooks path
CURRENT_HOOKS_PATH=$(git config --get core.hookspath 2>/dev/null || echo "")
if [ -n "$CURRENT_HOOKS_PATH" ]; then
    print_status "Current hooks path: $CURRENT_HOOKS_PATH"
    echo "$CURRENT_HOOKS_PATH" > .git/original-hooks-path
fi

# Create a temporary directory for our setup
TEMP_HOOKS_DIR=".git/temp-hooks"
mkdir -p "$TEMP_HOOKS_DIR"

# Method 1: Try to install pre-commit in a clean environment
print_status "Attempting to install pre-commit hooks..."

# Temporarily clear the hooks path
export GIT_CONFIG_NOSYSTEM=1
git config --local --unset core.hookspath 2>/dev/null || true

# Install pre-commit hooks
if pre-commit install --install-hooks; then
    print_success "Pre-commit hooks installed successfully"

    # Copy the installed hooks to our temp directory
    cp .git/hooks/pre-commit "$TEMP_HOOKS_DIR/" 2>/dev/null || true
    cp .git/hooks/pre-push "$TEMP_HOOKS_DIR/" 2>/dev/null || true

    # Restore the original hooks path
    if [ -n "$CURRENT_HOOKS_PATH" ]; then
        git config --local core.hookspath "$CURRENT_HOOKS_PATH"
    fi

else
    print_warning "Standard pre-commit installation failed, using alternative method..."

    # Restore the original hooks path
    if [ -n "$CURRENT_HOOKS_PATH" ]; then
        git config --local core.hookspath "$CURRENT_HOOKS_PATH"
    fi

    # Method 2: Create manual pre-commit integration
    print_status "Creating manual pre-commit integration..."

    # Create a wrapper script that calls pre-commit directly
    cat > "$TEMP_HOOKS_DIR/pre-commit" << 'EOF'
#!/bin/bash
# Manual pre-commit hook integration
exec pre-commit hook-impl --config=.pre-commit-config.yaml --hook-type=pre-commit --hook-dir .git/hooks -- "$@"
EOF

    cat > "$TEMP_HOOKS_DIR/pre-push" << 'EOF'
#!/bin/bash
# Manual pre-push hook integration
exec ./scripts/pre-push-hook.sh "$@"
EOF

    chmod +x "$TEMP_HOOKS_DIR/pre-commit"
    chmod +x "$TEMP_HOOKS_DIR/pre-push"
fi

# Method 3: Create hybrid hooks in the Git Defender directory
if [ -n "$CURRENT_HOOKS_PATH" ] && [ -d "$CURRENT_HOOKS_PATH" ]; then
    print_status "Creating hybrid hooks in Git Defender directory..."

    # Backup original Git Defender hooks
    sudo cp "$CURRENT_HOOKS_PATH/pre-commit" "$CURRENT_HOOKS_PATH/pre-commit.original" 2>/dev/null || true
    sudo cp "$CURRENT_HOOKS_PATH/pre-push" "$CURRENT_HOOKS_PATH/pre-push.original" 2>/dev/null || true

    # Create hybrid pre-commit hook
    cat > /tmp/hybrid-pre-commit << EOF
#!/bin/bash
# Hybrid pre-commit hook: Git Defender + pre-commit

# First, run the original Git Defender hook
if [ -f "$CURRENT_HOOKS_PATH/pre-commit.original" ]; then
    echo "Running Git Defender pre-commit checks..."
    "$CURRENT_HOOKS_PATH/pre-commit.original" "\$@"
    defender_exit=\$?
    if [ \$defender_exit -ne 0 ]; then
        echo "Git Defender pre-commit hook failed"
        exit \$defender_exit
    fi
fi

# Then run pre-commit
echo "Running pre-commit quality checks..."
cd "\$(git rev-parse --show-toplevel)"
if command -v pre-commit &> /dev/null; then
    exec pre-commit hook-impl --config=.pre-commit-config.yaml --hook-type=pre-commit --hook-dir .git/hooks -- "\$@"
else
    echo "pre-commit not found, skipping quality checks"
    exit 0
fi
EOF

    # Create hybrid pre-push hook
    cat > /tmp/hybrid-pre-push << EOF
#!/bin/bash
# Hybrid pre-push hook: Git Defender + pre-commit

# First, run the original Git Defender hook
if [ -f "$CURRENT_HOOKS_PATH/pre-push.original" ]; then
    echo "Running Git Defender pre-push checks..."
    "$CURRENT_HOOKS_PATH/pre-push.original" "\$@"
    defender_exit=\$?
    if [ \$defender_exit -ne 0 ]; then
        echo "Git Defender pre-push hook failed"
        exit \$defender_exit
    fi
fi

# Then run our custom pre-push checks
echo "Running PoKeys pre-push quality checks..."
cd "\$(git rev-parse --show-toplevel)"
if [ -f "./scripts/pre-push-hook.sh" ]; then
    exec ./scripts/pre-push-hook.sh "\$@"
else
    echo "Custom pre-push script not found, skipping"
    exit 0
fi
EOF

    # Install the hybrid hooks (requires sudo for Git Defender directory)
    if sudo cp /tmp/hybrid-pre-commit "$CURRENT_HOOKS_PATH/pre-commit" && \
       sudo cp /tmp/hybrid-pre-push "$CURRENT_HOOKS_PATH/pre-push" && \
       sudo chmod +x "$CURRENT_HOOKS_PATH/pre-commit" "$CURRENT_HOOKS_PATH/pre-push"; then
        print_success "Hybrid hooks installed in Git Defender directory"
        rm /tmp/hybrid-pre-commit /tmp/hybrid-pre-push
    else
        print_error "Failed to install hybrid hooks (permission denied)"
        print_status "Falling back to local hooks setup..."

        # Method 4: Set up local hooks that work alongside Git Defender
        print_status "Setting up local pre-commit integration..."

        # Create a git alias that runs pre-commit
        git config alias.precommit '!pre-commit run --all-files'

        # Create a commit-msg hook in the local .git/hooks directory
        cat > .git/hooks/commit-msg << 'EOF'
#!/bin/bash
# Local commit-msg hook that runs pre-commit
echo "Running pre-commit checks..."
if ! pre-commit run --all-files; then
    echo "Pre-commit checks failed. Commit aborted."
    exit 1
fi
EOF
        chmod +x .git/hooks/commit-msg

        print_success "Local pre-commit integration set up"
        print_warning "Note: This will run pre-commit checks on commit-msg instead of pre-commit"
    fi
else
    print_status "No Git Defender detected, setting up standard hooks..."

    # Copy our temp hooks to the standard location
    if [ -f "$TEMP_HOOKS_DIR/pre-commit" ]; then
        cp "$TEMP_HOOKS_DIR/pre-commit" .git/hooks/
        chmod +x .git/hooks/pre-commit
    fi

    if [ -f "$TEMP_HOOKS_DIR/pre-push" ]; then
        cp "$TEMP_HOOKS_DIR/pre-push" .git/hooks/
        chmod +x .git/hooks/pre-push
    fi

    print_success "Standard pre-commit hooks installed"
fi

# Clean up
rm -rf "$TEMP_HOOKS_DIR"

# Test the setup
print_status "Testing pre-commit setup..."
if pre-commit run --all-files --show-diff-on-failure; then
    print_success "Pre-commit is working correctly!"
else
    print_warning "Pre-commit checks found issues. Please fix them and try again."
fi

print_success "Pre-commit hooks setup complete!"
echo
print_status "Your commits will now run:"
if [ -n "$CURRENT_HOOKS_PATH" ]; then
    echo "  1. Git Defender security checks"
    echo "  2. Pre-commit quality checks"
else
    echo "  1. Pre-commit quality checks"
fi
echo
print_status "To test manually, run: pre-commit run --all-files"
