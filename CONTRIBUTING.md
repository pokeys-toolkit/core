# Contributing to PoKeys Core Library

## Conventional Commits

This project uses [Conventional Commits](https://www.conventionalcommits.org/) for automatic semantic versioning.

### Commit Message Format

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Types

- **feat**: A new feature (triggers minor version bump)
- **fix**: A bug fix (triggers patch version bump)
- **docs**: Documentation only changes
- **style**: Changes that do not affect the meaning of the code
- **refactor**: A code change that neither fixes a bug nor adds a feature
- **perf**: A code change that improves performance
- **test**: Adding missing tests or correcting existing tests
- **chore**: Changes to the build process or auxiliary tools

### Breaking Changes

Add `!` after the type or include `BREAKING CHANGE:` in the footer to trigger a major version bump:

```
feat!: remove deprecated MAX7219 functionality

BREAKING CHANGE: MAX7219 module has been completely removed
```

### Examples

```bash
# Patch version bump (0.3.6 -> 0.3.7)
git commit -m "fix: resolve encoder pin numbering issue"

# Minor version bump (0.3.6 -> 0.4.0)
git commit -m "feat: add new SPI communication protocol"

# Major version bump (0.3.6 -> 1.0.0)
git commit -m "feat!: redesign device connection API"
```

## Automatic Releases

Releases are automatically created when commits are pushed to the `main` branch:

1. The workflow analyzes commit messages since the last tag
2. Determines the appropriate version bump (patch/minor/major)
3. Updates `Cargo.toml` and `Cargo.lock`
4. Creates a git tag and GitHub release
5. Publishes to crates.io

No manual version bumping or tagging is required.
