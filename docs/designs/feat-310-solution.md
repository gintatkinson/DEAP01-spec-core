---
issue_id: 310
title: "Feature Implementation: Closed-world port and connection parity"
version: "1.0"
date: "2026-09-15"
---

# Feature Implementation: Closed-world port and connection parity

## Overview
Implemented closed-world parity validators for `ICD_01_SYSTEM_INTERFACE_MATRIX.md` against SysML ports and connections in the core compiler.

## Changes Made
- **`crates/verify-baseline/src/checks/traceability.rs`**: Added `icd-port-missing-from-roster` check for `sysml_model.ports` against `icd01_ports` and `icd-connection-missing-from-roster` check for `sysml_model.connections` against `icd01_connections`.
- **`crates/verify-baseline/tests/`**: Created a new test module using TDD (RED-GREEN-REFACTOR) to verify both rules. Included docstrings and `/// Realises: [Issue310/ICDCompletenessValidator]` traceability tags.

## Code Realization Table

| Feature / Artifact | Implemented Source File | Class / Method |
| --- | --- | --- |
| `icd-port-missing-from-roster` | `crates/verify-baseline/src/checks/traceability.rs` | `check_icd_completeness` |
| `icd-connection-missing-from-roster` | `crates/verify-baseline/src/checks/traceability.rs` | `check_icd_completeness` |
| Port/Connection Parity Unit Tests | `crates/verify-baseline/tests/` | `test_sysml_port_missing_from_roster`, `test_sysml_connection_missing_from_roster` |

## Validation Results
- All workspace tests (`cargo test --workspace`) passed successfully in isolated test runs.
- Ran `./target/release/verify-baseline` (Gate 23 / ICD Completeness & Signal Flow Parity Audit) on the upstream compiler repository and verified successful parity enforcement.
- Verified docstring presence and LUI 3-Layer Definition of Done compliance.
