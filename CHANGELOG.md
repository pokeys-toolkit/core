# Changelog

All notable changes to this project will be documented in this file.

## [1.0.1] - 2026-04-27

### Documentation and dependency maintenance (no library changes)

- chore(docs): upgrade Astro 5.13.2 → 6.1.6 and migrate from Tailwind 3 (`@astrojs/tailwind`) to Tailwind 4 (`@tailwindcss/vite`). Regenerates `docs/package-lock.json`, closing 43 open Dependabot alerts (14 high / 20 medium / 9 low).
- chore(ci): bump docs workflow (`.github/workflows/docs.yml`) to Node 22 (required by Astro 6).
- chore(deps): add `.github/dependabot.yml` for grouped weekly cargo / npm / github-actions PRs.

No Rust source changes. Crate functionality and public API are identical to 1.0.0.

## [1.0.0] - 2026-04-27

- feat!(encoders): fix wire format + add typed options for fast and ultra-fast encoders (#24)

## [0.21.9] - 2026-04-27

### Breaking changes (previously-broken APIs corrected)

- `configure_fast_encoders(options: u8, enable_index: bool)` → `configure_fast_encoders(config: FastEncoderConfiguration, options: FastEncoderOptions)`. The old signature sent the wrong bytes: the user's composite options byte went to the `FastEncodersConfiguration` slot and a fictional `enable_index` flag went to the `FastEncodersOptions` slot. Command `0xCE` has no `enable_index` field. Fast-encoder configuration has not been working correctly for any caller of this function.
- `configure_ultra_fast_encoder(enable, enable_4x_sampling, signal_mode_direction_clock, invert_direction, reset_on_index, filter_delay)` → `configure_ultra_fast_encoder(enable, options: UltraFastEncoderOptions, reset_on_index, filter_delay)`. The old implementation packed options into the wrong bits (bit 1/2/3 instead of 0/1/2) and hand-rolled a request buffer that double-shifted the command byte. Direction invert has never actually worked on the ultra-fast encoder.
- `read_ultra_fast_encoder_config() -> Result<(bool, u8, u32)>` → `Result<(bool, UltraFastEncoderOptions, u32)>`. Also now uses the library's standard request path instead of a hand-rolled buffer.

### New types

- `FastEncoderConfiguration` — typed selector for the two fast-encoder pin layouts (`Config1` = 0x01, `Config2` = 0x10, `Disabled` = 0x00).
- `FastEncoderOptions` — typed per-encoder direction-invert + sampling-mode flags for command `0xCE`. Bit masks match PoLabs' PoKeysLib C reference (`ePK_FastEncoderOptions`): disable_4x = 0x10, invert_1 = 0x20, invert_2 = 0x40, invert_3 = 0x80.
- `UltraFastEncoderOptions` — typed flags for command `0x1C`. Bit masks match PoKeysLib (`ePK_UltraFastEncoderOptions`): invert_direction = 0x01, signal_mode = 0x02, enable_4x = 0x04.

## [0.21.8] - 2026-04-27

- feat(io): support hardware invert bit on digital pin functions (#23)
- fix(io): return actual pin state from get_digital_input (#22)
- fix(protocol): align I/O command bytes and response parsing with spec v14.3.2025 (#21)
- fix(io): use 0x10 uniformly for all pins; revert erroneous 0xC0 workaround (#20)

Note: supersedes the v0.22.x / v0.23.0 tags created by the automated release
workflow, which were never published to crates.io or as GitHub Releases.

## [0.23.0] - 2026-04-27

- feat(io): support hardware invert bit on digital pin functions (#23)

## [0.22.3] - 2026-04-27

- fix(io): return actual pin state from get_digital_input (#22)

## [0.22.2] - 2026-04-26

- fix(protocol): align I/O command bytes and response parsing with spec v14.3.2025 (#21)

## [0.22.1] - 2026-04-26

- fix(io): use 0x10 uniformly for all pins; revert erroneous 0xC0 workaround (#20)

## [0.22.0] - 2026-04-26

- feat(device): network config & device name examples; fix 1-based pin in set_pin_function (#15)

## [0.21.0] - 2026-04-25

- feat(device): expose set_device_name and set_network_configuration (#14)

## [0.20.0] - 2026-04-18

- perf(io): bulk set_all_pin_functions via 0xC0 (set) (#13)
- feat(device): implement reboot operation (command 0xF3) (#12)
- feat(io): bulk device-status read (0xCC) and complete analog-output writes (0x41) (#11)

## [0.19.4] - 2026-04-17

- feat: expose device system load status (command 0x05) (#10)

## [0.19.3] - 2026-04-14

- fix(network): implement set_network_configuration (command 0xE0, option 10)

## [0.19.2] - 2026-04-14

- fix(oem): remove trailing blank line to satisfy rustfmt
- fix(oem): remove always-true constant assertion flagged by clippy
- fix(oem): add OEM parameter read/write/clear and device location support

## [0.19.1] - 2025-09-25



## [0.19.0] - 2025-09-25



## [0.18.0] - 2025-09-13

- feat: Add remaining API documentation pages
- feat: Add comprehensive API documentation with professional layout
- fix: Escape all curly braces in Astro code blocks
- fix: Escape Astro template syntax in PWM documentation

## [0.17.0] - 2025-09-13

- fix(fmt): Correcting cargo format issues
- fix: Remove needless return statement in servo stop method
- feat: Add servo control types and functionality

## [0.16.0] - 2025-09-10

- fix: Add Default implementation for PwmData
- feat: Implement 25MHz PWM control with servo calibration

## [0.15.0] - 2025-09-06

- fix: resolve doctest compilation error in keyboard_matrix.rs
- fix: resolve GitHub release action issues
- fix: resolve CI clippy warnings in examples
- fix: resolve CI failures - formatting and matrix keyboard tests
- fix: add proper device connection to simple matrix keyboard example
- feat: add simple matrix keyboard example
- fix: resolve Astro syntax error in matrix keyboard documentation
- fix: implement correct PoKeys matrix keyboard protocol (0xCA)
- feat: separate keyboard matrix functionality from general matrix module

## [0.14.0] - 2025-08-21

- Merge pull request #1 from pokeys-toolkit/feature/uspibridge-custom-pinout
- fix: apply cargo fmt formatting fixes
- feat: complete uSPIBridge I2C integration with all commands
- fix: align uSPIBridge implementation with actual firmware
- feat: implement uSPIBridge custom pinout support

## [0.13.1] - 2025-08-19



## [0.13.0] - 2025-08-19

- fix: Resolve CI clippy and formatting issues
- feat: Enhanced I2C support and comprehensive testing integration
- fix(docs): Remove performance from README.md

## [0.12.3] - 2025-08-19

- fix: exclude documentation changes from triggering library releases

## [0.12.2] - 2025-08-19

- fix: escape curly braces in git dependency example
- fix: properly escape curly braces in code examples using HTML entities

## [0.12.1] - 2025-08-19

- fix: escape HTML characters in Rust code examples for Astro build

## [0.12.0] - 2025-08-19

- feat: add comprehensive getting started guide

## [0.11.0] - 2025-08-19

- feat: add documentation section with Getting Started, Examples, and API links

## [0.10.0] - 2025-08-19

- feat: recreate site with exact Astrolus design

## [0.9.0] - 2025-08-19

- feat: implement Astrolus-inspired design with gradient hero and glass morphism cards

## [0.8.0] - 2025-08-19

- feat: add links to latest release and crates.io on main page

## [0.7.0] - 2025-08-18

- feat: update main page to highlight Rust language support

## [0.6.0] - 2025-08-18

- feat: add Astro documentation site with Starlight theme

## [0.5.2] - 2025-08-18



## [0.5.1] - 2025-08-18

- fix: allow dirty working directory for cargo publish

## [0.5.0] - 2025-08-18

- feat: complete semantic versioning implementation

## [0.4.0] - 2025-08-18

- fix: add write permissions to release workflow
- feat: add automatic changelog generation and release notes
- feat: implement semantic versioning with conventional commits

## [0.3.6] - 2025-08-18

- feat: implement semantic versioning with conventional commits
- fix: remove flaky file_monitoring_tests.rs
- fix: examples and README after MAX7219 removal
- feat: Complete MAX7219 removal and cleanup
