# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.2.0] - 2026-10-17

### Added
- **High-Speed Rust Core Engine (`rust-core/`):** Implemented sub-millisecond network governance evaluation, adaptive supervision, and SHA-256 proof-chain verification in native Rust.
- **Native Rust CLI (`jfp-paradox`):** Added command-line utility with commands:
  - `jfp-paradox eval <metrics.yaml|json>` — instant network health verdict.
  - `jfp-paradox verify-chain <proof_log.jsonl>` — tamper-evident chain verification.
  - `jfp-paradox explain <metrics>` — executive risk assessment.
- **Universal Multi-Format Support:** First-class parsing of both **YAML** and **JSON** metric payloads and configurations.
- **Standard Test Runner (`tests/run_tests.py`):** Added a self-contained test runner without external pytest runtime dependency.

### Fixed
- **Packaging Syntax Fixes:** Repaired broken `pyproject.toml` dependencies and metadata blocks.
- **Module Packaging:** Added `__init__.py` modules across `daemon/`, `shared/`, `console/`, `tools/`, and `tests/`.
- **Governor Criticality Logic:** Fixed critical breach logic in Governor evaluation to ensure any `*_critical` event triggers the appropriate defensive state.

---

## [0.1.0] - 2026-06-19

### Added
- Autonomous network governance daemon
- Network metrics collection (latency, packet loss, jitter, bandwidth)
- Two-layer governance (Governor + Supervisor)
- Action executor (tc, iptables, ip route)
- SHA-256 proof logger
- JSON-RPC interface (8 methods)
- Network simulator (7 scenarios)
- Electron desktop app
- Comprehensive test suite (46 tests, 98% passing)
- CI/CD workflows (GitHub Actions)

### Security
- Dry-run mode enabled by default
- Token-gated operations
- Actions allowlisted
- Two-layer approval required

---

## [0.0.0] - 2026-06-14

### Added
- Initial commit

---

[Unreleased]: https://github.com/etechvictoria-ui/jfp-paradox/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/etechvictoria-ui/jfp-paradox/releases/v0.1.0
[0.0.0]: https://github.com/etechvictoria-ui/jfp-paradox/releases/v0.0.0
