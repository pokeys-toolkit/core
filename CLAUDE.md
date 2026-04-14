# CLAUDE.md

## Project

`pokeys-lib` — a pure Rust library (`cdylib + rlib`) for controlling PoKeys hardware devices over USB and Ethernet. It provides digital/analog I/O, PWM, encoders, stepper motor control (Pulse Engine v2), matrix keyboards, LCD/7-seg displays, and communication protocols (SPI, I2C, UART, CAN, 1-Wire, uSPIBridge).

- **Rust edition**: 2024
- **Toolchain**: pinned at 1.88.0 (`rust-toolchain.toml`)
- **Repo**: https://github.com/pokeys-toolkit/core

---

## Essential Commands

```bash
# Build
cargo build

# Run all non-hardware tests
cargo test

# Run a specific test file
cargo test --test unit_tests
cargo test --test integration_tests

# Run tests requiring physical hardware (disabled by default)
cargo test --features hardware-tests

# Lint
cargo clippy -- -D warnings

# Format
cargo fmt

# Check formatting without changing files
cargo fmt --check

# Security/supply-chain audit
cargo deny check
cargo audit

# Build docs
cargo doc --no-deps --document-private-items
```

---

## Architecture

```
src/
├── lib.rs                  # Public API surface, module exports
├── types.rs                # Shared types (PinFunction, DeviceConnectionType, etc.)
├── error.rs                # Error enum (thiserror), recovery strategies, FFI return codes
├── device.rs               # PoKeysDevice struct — connection, device info
├── communication.rs        # Low-level protocol: retries, checksums, timeouts
├── model_manager.rs        # Load/validate/list device models
├── models.rs               # Device model structs; YAML parsing
├── io/
│   ├── mod.rs              # Digital/analog pin ops, bulk operations
│   └── private.rs          # Internal helpers
├── pwm.rs                  # 6 PWM channels on pins 17-22 (25MHz precision)
├── encoders.rs             # Quadrature encoders (4x/2x modes)
├── pulse_engine.rs         # Stepper motor control (Pulse Engine v2)
├── keyboard_matrix.rs      # Matrix keyboard scanning (4x4–16x8)
├── matrix.rs               # LED matrix control
├── lcd.rs                  # LCD text display
├── sensors.rs              # EasySensors, RTC, 1-Wire temperature
├── network.rs              # Network device discovery
└── protocols/
    ├── mod.rs
    ├── i2c.rs              # I2C master: auto fragmentation, retry, device scan
    ├── spi.rs              # SPI master, multi-CS
    ├── uart.rs             # Serial communication
    ├── can.rs              # CAN bus
    ├── onewire.rs          # 1-Wire protocol
    ├── uspibridge.rs       # uSPIBridge (SPI→I2C bridge with custom pinout)
    └── convenience.rs      # Convenience wrappers

models/                     # YAML device capability definitions
    PoKeys56U.yaml
    PoKeys56E.yaml
    PoKeys57U.yaml
    PoKeys57E.yaml

tests/                      # Integration test suite (no hardware required)
examples/                   # Standalone usage examples (16 programs)
```

---

## Coding Conventions

### Error handling
- All public functions return `Result<T>` where `Result<T> = std::result::Result<T, PoKeysError>`
- `PoKeysError` is defined with `thiserror` in `error.rs` — add new variants there, never use raw strings
- Error variants carry context fields (e.g., `I2cPacketTooLarge { size, max_size, suggestion }`)
- Implement `is_recoverable()` and `recovery_strategy()` for new error variants that represent transient conditions
- Legacy C FFI callers expect integer return codes — map new errors in the existing return code table in `error.rs`

### Logging
- Use the `log` crate: `log::error!`, `log::warn!`, `log::info!`, `log::debug!`
- `warn!` for recoverable protocol issues (e.g., retry loops), `debug!` for state tracing

### Device models
- Pin capabilities are defined in `models/*.yaml`, not hardcoded in Rust
- Validate pin functions against the loaded model before executing hardware operations
- Related capabilities must be paired (e.g., `Encoder_1A` requires `Encoder_1B` on the next pin)

### Concurrency
- Shared state uses `Arc<Mutex<_>>` or `LazyLock`; never introduce raw `static mut`

### Performance
- Prefer bulk pin operations over per-pin calls (28x faster in benchmarks)
- Document any new bulk API with its speedup rationale

---

## Testing Strategy

- **Unit tests** (`tests/unit_tests.rs`): core logic only, no I/O, always run in CI
- **Integration tests** (`tests/integration_tests.rs`, `tests/protocol_tests.rs`, etc.): simulate device interactions without physical hardware
- **Hardware tests** (`tests/hardware_integration.rs.disabled`): require a real PoKeys device; gated behind `--features hardware-tests` and not run in CI; the `.disabled` suffix prevents accidental inclusion
- When adding new features, add tests to the appropriate test file — do not add hardware-dependent assertions to the standard test suite

---

## Release Process

Releases are **fully automated** — do not bump versions or edit `CHANGELOG.md` by hand.

All commits to `main` trigger `.github/workflows/release.yml`:

| Commit type | Version bump |
|---|---|
| `fix:` | patch (0.0.x) |
| `feat:` | minor (0.x.0) |
| `feat!:` or `BREAKING CHANGE` footer | major (x.0.0) |

The workflow: runs tests → bumps `Cargo.toml` → updates `CHANGELOG.md` → creates git tag → creates GitHub Release → publishes to crates.io.

Use **Conventional Commits** for all commit messages (`feat:`, `fix:`, `docs:`, `refactor:`, `perf:`, `test:`, `chore:`).

---

## CI

`.github/workflows/ci.yml` runs on every push to `main`/`develop` and all PRs to `main`:

1. **test**: `cargo test`, `cargo clippy -D warnings`, `cargo fmt --check` (Ubuntu)
2. **build**: release build matrix across Ubuntu, Windows, macOS

Keep clippy clean — `-D warnings` is enforced. The current `#[allow(...)]` attributes in the codebase are tracked for cleanup; do not add new blanket allows.

---

## Pre-commit Hooks

`.pre-commit-config.yaml` runs on every commit: formatting, `cargo check`, `cargo audit`, unit/protocol/integration tests, and doc build. Install with `pre-commit install`. Scripts in `scripts/` automate setup (`setup-devops.sh`).

---

## Dependency Policy

`deny.toml` enforces license and supply-chain rules (targets: Linux, Windows, macOS x86_64). Allowed licenses include MIT, Apache-2.0, BSD-2/3, ISC, MPL-2.0, LGPL. Do not add dependencies with GPL or proprietary licenses. Run `cargo deny check` before adding any new dependency.
