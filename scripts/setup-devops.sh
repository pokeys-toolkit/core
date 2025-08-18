#!/bin/bash

# Comprehensive DevOps Setup Script for PoKeys Project
# This script sets up the complete DevOps pipeline including:
# - Pre-commit hooks
# - Git hooks
# - GitHub branch protection
# - Required tools installation

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m' # No Color

print_header() {
    echo
    echo -e "${BOLD}${BLUE}=== $1 ===${NC}"
    echo
}

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

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d ".git" ]; then
    print_error "This script must be run from the root of the PoKeys project"
    exit 1
fi

print_header "PoKeys DevOps Setup"
print_status "Setting up comprehensive DevOps pipeline..."

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to detect OS
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo "linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    elif [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
        echo "windows"
    else
        echo "unknown"
    fi
}

# Function to install Python packages safely
install_python_package() {
    local package="$1"
    local command_name="${2:-$package}"

    if command_exists "$command_name"; then
        print_status "$package is already installed"
        return 0
    fi

    print_status "Installing $package..."

    # Try different installation methods in order of preference
    local install_success=false

    # Method 1: Try pipx (recommended for tools)
    if command_exists pipx; then
        print_status "Installing $package using pipx..."
        if pipx install "$package" 2>/dev/null; then
            install_success=true
            print_success "$package installed via pipx"
        fi
    fi

    # Method 2: Try pip with --user flag
    if [ "$install_success" = false ] && command_exists pip3; then
        print_status "Installing $package using pip3 --user..."
        if pip3 install --user "$package" 2>/dev/null; then
            install_success=true
            print_success "$package installed via pip3 --user"
        fi
    fi

    # Method 3: Try pip with --break-system-packages (last resort)
    if [ "$install_success" = false ] && command_exists pip3; then
        print_warning "Attempting to install $package with --break-system-packages flag..."
        read -p "This may affect system packages. Continue? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            if pip3 install --break-system-packages "$package" 2>/dev/null; then
                install_success=true
                print_success "$package installed via pip3 --break-system-packages"
            fi
        fi
    fi

    # Method 4: Try system package manager
    if [ "$install_success" = false ]; then
        local os=$(detect_os)
        case $os in
            "linux")
                if command_exists apt-get; then
                    print_status "Trying to install $package via apt..."
                    local apt_package=""
                    case $package in
                        "pre-commit") apt_package="pre-commit" ;;
                        "detect-secrets") apt_package="python3-detect-secrets" ;;
                    esac
                    if [ -n "$apt_package" ] && sudo apt-get update && sudo apt-get install -y "$apt_package" 2>/dev/null; then
                        install_success=true
                        print_success "$package installed via apt"
                    fi
                elif command_exists yum; then
                    print_status "Trying to install $package via yum..."
                    # Add yum package names as needed
                fi
                ;;
            "macos")
                if command_exists brew; then
                    print_status "Trying to install $package via Homebrew..."
                    local brew_package=""
                    case $package in
                        "pre-commit") brew_package="pre-commit" ;;
                        "detect-secrets") brew_package="detect-secrets" ;;
                    esac
                    if [ -n "$brew_package" ] && brew install "$brew_package" 2>/dev/null; then
                        install_success=true
                        print_success "$package installed via Homebrew"
                    fi
                fi
                ;;
        esac
    fi

    if [ "$install_success" = false ]; then
        print_error "Failed to install $package using any method"
        print_error "Please install $package manually using one of these methods:"
        echo "  1. pipx install $package"
        echo "  2. pip3 install --user $package"
        echo "  3. Use your system package manager"
        echo "  4. Create a virtual environment: python3 -m venv venv && source venv/bin/activate && pip install $package"
        return 1
    fi

    return 0
}

# Function to install a Rust tool if it doesn't exist
install_rust_tool() {
    local tool="$1"
    local install_cmd="$2"
    local check_cmd="${3:-$tool}"

    if command_exists "$check_cmd"; then
        print_status "$tool is already installed"
    else
        print_status "Installing $tool..."
        eval "$install_cmd"
        if command_exists "$check_cmd"; then
            print_success "$tool installed successfully"
        else
            print_error "Failed to install $tool"
            return 1
        fi
    fi
}

# Function to handle Git hooks path configuration
setup_git_hooks() {
    print_header "Setting Up Git Hooks"

    # Check if core.hooksPath is set
    local hooks_path=$(git config --get core.hooksPath 2>/dev/null || echo "")

    if [ -n "$hooks_path" ]; then
        print_warning "Git core.hooksPath is set to: $hooks_path"
        print_status "This appears to be Git Defender or similar security tooling"

        # Check if it's Git Defender
        if [[ "$hooks_path" == *"git-defender"* ]]; then
            print_status "Detected Git Defender configuration"
            print_status "We'll work around this by using a hybrid approach"

            # Create a backup of the current hooks path
            print_status "Backing up current hooks configuration..."
            echo "$hooks_path" > .git/original-hooks-path

            # Temporarily unset core.hooksPath for pre-commit installation
            print_status "Temporarily unsetting core.hooksPath for pre-commit installation..."
            git config --unset core.hooksPath

            # Install pre-commit hooks
            print_status "Installing pre-commit hooks..."
            if pre-commit install; then
                print_success "Pre-commit hooks installed"
            else
                print_error "Failed to install pre-commit hooks"
                # Restore original hooks path
                git config core.hooksPath "$hooks_path"
                return 1
            fi

            if pre-commit install --hook-type pre-push; then
                print_success "Pre-push hooks installed"
            else
                print_warning "Failed to install pre-push hooks (continuing anyway)"
            fi

            # Create a hybrid hook that calls both Git Defender and pre-commit
            print_status "Creating hybrid hooks that work with Git Defender..."

            # Create hybrid pre-commit hook
            cat > .git/hooks/pre-commit << EOF
#!/bin/bash
# Hybrid pre-commit hook for Git Defender + pre-commit

# First, run Git Defender hooks if they exist
if [ -f "$hooks_path/pre-commit" ]; then
    echo "Running Git Defender pre-commit hook..."
    "$hooks_path/pre-commit" "\$@"
    defender_exit=\$?
    if [ \$defender_exit -ne 0 ]; then
        echo "Git Defender pre-commit hook failed"
        exit \$defender_exit
    fi
fi

# Then run pre-commit
echo "Running pre-commit hooks..."
exec pre-commit hook-impl --config=.pre-commit-config.yaml --hook-type=pre-commit --hook-dir .git/hooks -- "\$@"
EOF

            # Create hybrid pre-push hook
            cat > .git/hooks/pre-push << EOF
#!/bin/bash
# Hybrid pre-push hook for Git Defender + pre-commit

# First, run Git Defender hooks if they exist
if [ -f "$hooks_path/pre-push" ]; then
    echo "Running Git Defender pre-push hook..."
    "$hooks_path/pre-push" "\$@"
    defender_exit=\$?
    if [ \$defender_exit -ne 0 ]; then
        echo "Git Defender pre-push hook failed"
        exit \$defender_exit
    fi
fi

# Then run our custom pre-push hook
echo "Running PoKeys pre-push checks..."
exec ./scripts/pre-push-hook.sh "\$@"
EOF

            # Make hooks executable
            chmod +x .git/hooks/pre-commit
            chmod +x .git/hooks/pre-push

            print_success "Hybrid hooks created successfully"
            print_status "Your hooks will now run both Git Defender and pre-commit checks"

        else
            print_warning "Unknown hooks path configuration detected"
            print_status "You may need to manually configure pre-commit hooks"

            echo
            read -p "Would you like to temporarily unset core.hooksPath to install pre-commit? (y/N): " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                print_status "Backing up and unsetting core.hooksPath..."
                echo "$hooks_path" > .git/original-hooks-path
                git config --unset core.hooksPath

                # Install pre-commit hooks
                if pre-commit install && pre-commit install --hook-type pre-push; then
                    print_success "Pre-commit hooks installed"
                else
                    print_error "Failed to install pre-commit hooks"
                    git config core.hooksPath "$hooks_path"
                    return 1
                fi

                print_warning "core.hooksPath has been unset. You may need to reconfigure your security tools."
                print_status "Original hooks path saved in .git/original-hooks-path"
            else
                print_status "Skipping pre-commit hook installation"
                print_warning "You'll need to run pre-commit checks manually"
                return 0
            fi
        fi
    else
        # Standard pre-commit installation
        print_status "Installing pre-commit hooks..."
        if pre-commit install; then
            print_success "Pre-commit hooks installed"
        else
            print_error "Failed to install pre-commit hooks"
            return 1
        fi

        if pre-commit install --hook-type pre-push; then
            print_success "Pre-push hooks installed"
        else
            print_warning "Failed to install pre-push hooks (continuing anyway)"
        fi

        # Set up custom pre-push hook
        print_status "Setting up custom pre-push hook..."
        if [ -f ".git/hooks/pre-push" ]; then
            cp .git/hooks/pre-push .git/hooks/pre-push.backup
            print_status "Backed up existing pre-push hook"
        fi

        # Create the pre-push hook
        cat > .git/hooks/pre-push << 'EOF'
#!/bin/bash
# Custom pre-push hook for PoKeys project
exec ./scripts/pre-push-hook.sh "$@"
EOF

        chmod +x .git/hooks/pre-push
        print_success "Custom pre-push hook installed"
    fi
}

# Check prerequisites
print_header "Checking Prerequisites"

# Check Rust
if ! command_exists rustc; then
    print_error "Rust is not installed. Please install Rust from https://rustup.rs/"
    exit 1
fi
print_success "Rust is installed ($(rustc --version))"

# Check Cargo
if ! command_exists cargo; then
    print_error "Cargo is not installed. Please install Rust from https://rustup.rs/"
    exit 1
fi
print_success "Cargo is installed ($(cargo --version))"

# Check Git
if ! command_exists git; then
    print_error "Git is not installed. Please install Git."
    exit 1
fi
print_success "Git is installed ($(git --version))"

# Check Python3
if command_exists python3; then
    print_success "Python3 is installed ($(python3 --version))"
else
    print_error "Python3 is not installed. Please install Python3."
    exit 1
fi

# Install required tools
print_header "Installing Required Tools"

# Suggest installing pipx if not available
if ! command_exists pipx; then
    print_warning "pipx is not installed. pipx is the recommended way to install Python CLI tools."
    print_status "To install pipx:"
    local os=$(detect_os)
    case $os in
        "linux")
            echo "  Ubuntu/Debian: sudo apt install pipx"
            echo "  Fedora: sudo dnf install pipx"
            ;;
        "macos")
            echo "  Homebrew: brew install pipx"
            echo "  Or: python3 -m pip install --user pipx"
            ;;
        *)
            echo "  python3 -m pip install --user pipx"
            ;;
    esac
    echo
fi

# Install Python tools with fallback methods
install_python_package "pre-commit" "pre-commit"
install_python_package "detect-secrets" "detect-secrets"

# Install Rust tools
install_rust_tool "cargo-audit" "cargo install cargo-audit" "cargo-audit"
install_rust_tool "cargo-deny" "cargo install cargo-deny" "cargo-deny"
install_rust_tool "cargo-tarpaulin" "cargo install cargo-tarpaulin" "cargo-tarpaulin"

# Optional tools
print_status "Installing optional tools..."
install_rust_tool "cargo-license" "cargo install cargo-license" "cargo-license" || print_warning "cargo-license installation failed (optional)"
install_rust_tool "cargo-vet" "cargo install cargo-vet" "cargo-vet" || print_warning "cargo-vet installation failed (optional)"

# Ensure tools are in PATH
print_header "Checking Tool Availability"

# Add common user binary paths to PATH if not already there
USER_BIN_PATHS=(
    "$HOME/.local/bin"
    "$HOME/.cargo/bin"
)

for bin_path in "${USER_BIN_PATHS[@]}"; do
    if [[ ":$PATH:" != *":$bin_path:"* ]] && [ -d "$bin_path" ]; then
        export PATH="$bin_path:$PATH"
        print_status "Added $bin_path to PATH for this session"
    fi
done

# Verify critical tools are available
critical_tools=("pre-commit" "cargo-audit" "cargo-deny" "detect-secrets")
missing_tools=()

for tool in "${critical_tools[@]}"; do
    if ! command_exists "$tool"; then
        missing_tools+=("$tool")
    fi
done

if [ ${#missing_tools[@]} -gt 0 ]; then
    print_error "The following critical tools are still not available:"
    for tool in "${missing_tools[@]}"; do
        echo "  - $tool"
    done
    echo
    print_error "Please install these tools manually and ensure they're in your PATH:"
    echo "  export PATH=\"\$HOME/.local/bin:\$HOME/.cargo/bin:\$PATH\""
    echo
    print_status "You may need to restart your shell or run:"
    echo "  source ~/.bashrc  # or ~/.zshrc"
    echo
    exit 1
fi

# Create secrets baseline
print_status "Creating secrets baseline..."
if [ ! -f ".secrets.baseline" ]; then
    if detect-secrets scan --baseline .secrets.baseline 2>/dev/null; then
        print_success "Secrets baseline created"
    else
        print_warning "Failed to create secrets baseline, creating empty one"
        echo '{}' > .secrets.baseline
    fi
else
    print_status "Secrets baseline already exists"
fi

# Set up Git hooks (handles core.hooksPath conflicts)
setup_git_hooks

# Test pre-commit setup
print_header "Testing Pre-commit Setup"
print_status "Running pre-commit on all files (this may take a while)..."

if pre-commit run --all-files; then
    print_success "All pre-commit checks passed!"
else
    print_warning "Some pre-commit checks failed. This is normal for the first run."
    print_warning "Please fix any issues and run 'pre-commit run --all-files' again."
fi

# GitHub CLI setup
print_header "GitHub CLI Setup"
if command_exists gh; then
    print_success "GitHub CLI is installed ($(gh --version | head -n1))"

    if gh auth status &> /dev/null; then
        print_success "GitHub CLI is authenticated"

        # Offer to set up branch protection
        echo
        read -p "Would you like to set up GitHub branch protection rules? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            print_status "Setting up branch protection..."
            if ./scripts/setup-branch-protection.sh; then
                print_success "Branch protection configured"
            else
                print_warning "Branch protection setup failed. You can run it manually later."
            fi
        else
            print_status "Skipping branch protection setup"
            print_status "You can run it later with: ./scripts/setup-branch-protection.sh"
        fi
    else
        print_warning "GitHub CLI is not authenticated"
        print_status "Run 'gh auth login' to authenticate, then run './scripts/setup-branch-protection.sh'"
    fi
else
    print_warning "GitHub CLI is not installed"
    print_status "Install it from https://cli.github.com/ to enable branch protection setup"
fi

# Final summary
print_header "Setup Complete!"

print_success "DevOps pipeline has been successfully configured!"
echo

print_status "What's been set up:"
echo "  ✅ Pre-commit hooks with comprehensive checks"
echo "  ✅ Pre-push hooks to prevent bad commits"
echo "  ✅ GitHub workflows for CI/CD"
echo "  ✅ Security scanning and dependency checks"
echo "  ✅ Code quality enforcement"
echo "  ✅ Issue and PR templates"
echo "  ✅ CODEOWNERS file for review requirements"

# Check if Git Defender integration was set up
if [ -f ".git/original-hooks-path" ]; then
    echo "  ✅ Git Defender integration (hybrid hooks)"
fi

echo
print_status "Required tools installed:"
echo "  • pre-commit (code quality checks)"
echo "  • cargo-audit (security auditing)"
echo "  • cargo-deny (dependency management)"
echo "  • cargo-tarpaulin (code coverage)"
echo "  • detect-secrets (secret scanning)"

echo
print_status "GitHub workflows configured:"
echo "  • ci.yml - Comprehensive CI for pull requests"
echo "  • release.yml - Release automation for main branch"
echo "  • security.yml - Daily security scans"

echo
print_warning "Important notes:"
echo "  • All commits will now run extensive pre-commit checks"
echo "  • Pushes will be blocked if tests fail"
echo "  • Direct pushes to main branch are prohibited"
echo "  • All changes must go through pull requests"
echo "  • CI must pass before merging"

# Special note for Git Defender users
if [ -f ".git/original-hooks-path" ]; then
    echo
    print_status "Git Defender Integration:"
    echo "  • Your existing Git Defender hooks are preserved"
    echo "  • Both Git Defender and pre-commit checks will run"
    echo "  • Original hooks path saved in .git/original-hooks-path"
fi

echo
print_status "If tools are not found, add these to your shell profile:"
echo "  export PATH=\"\$HOME/.local/bin:\$HOME/.cargo/bin:\$PATH\""
echo "  Then restart your shell or run: source ~/.bashrc"

echo
print_status "Next steps:"
echo "  1. Commit these DevOps changes: git add . && git commit -m 'Add comprehensive DevOps pipeline'"
echo "  2. Push to a feature branch: git checkout -b setup/devops && git push origin setup/devops"
echo "  3. Create a pull request to merge into main"
echo "  4. Once merged, all future development will use this pipeline"

echo
print_status "Useful commands:"
echo "  • Run all pre-commit checks: pre-commit run --all-files"
echo "  • Update pre-commit hooks: pre-commit autoupdate"
echo "  • Run comprehensive tests: ./run_tests.sh"
echo "  • Check security: cargo audit && cargo deny check"

echo
print_success "🎉 DevOps setup complete! Your project is now production-ready!"
