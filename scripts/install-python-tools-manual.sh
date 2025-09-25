#!/bin/bash

# Manual Python Tools Installation Script
# Use this if the main setup script fails due to externally-managed-environment errors

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

print_status "Manual Python Tools Installation"
echo

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Method 1: Using pipx (recommended)
if command_exists pipx; then
    print_status "Installing tools using pipx (recommended method)..."

    pipx install pre-commit
    pipx install detect-secrets

    print_success "Tools installed via pipx"

elif command_exists pip3; then
    print_status "pipx not found, trying alternative methods..."

    # Method 2: Virtual environment
    print_status "Creating virtual environment for DevOps tools..."

    python3 -m venv ~/.devops-venv
    source ~/.devops-venv/bin/activate

    pip install pre-commit detect-secrets

    deactivate

    # Create wrapper scripts
    mkdir -p ~/.local/bin

    cat > ~/.local/bin/pre-commit << 'EOF'
#!/bin/bash
source ~/.devops-venv/bin/activate
exec pre-commit "$@"
EOF

    cat > ~/.local/bin/detect-secrets << 'EOF'
#!/bin/bash
source ~/.devops-venv/bin/activate
exec detect-secrets "$@"
EOF

    chmod +x ~/.local/bin/pre-commit
    chmod +x ~/.local/bin/detect-secrets

    print_success "Tools installed in virtual environment with wrapper scripts"

else
    print_error "Neither pipx nor pip3 found. Please install Python package manager."
    exit 1
fi

# Verify installation
print_status "Verifying installation..."

# Add ~/.local/bin to PATH for this session
export PATH="$HOME/.local/bin:$PATH"

if command_exists pre-commit; then
    print_success "pre-commit is available"
else
    print_error "pre-commit is not available in PATH"
fi

if command_exists detect-secrets; then
    print_success "detect-secrets is available"
else
    print_error "detect-secrets is not available in PATH"
fi

echo
print_status "Add this to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
echo "export PATH=\"\$HOME/.local/bin:\$PATH\""
echo
print_status "Then restart your shell or run: source ~/.bashrc"
echo
print_success "Manual installation complete!"
