# Changelog

All notable changes to this project will be documented in this file.

## [0.20.0] - 2026-04-17

- feat: expose device system load status (command 0x05) (#10)
- fix(network): implement set_network_configuration (command 0xE0, option 10) (#9)

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
