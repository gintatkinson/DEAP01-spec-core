---
issue_id: 303
title: "Feature Implementation: Tokenized Python comment scanner"
version: "1.0"
date: "2026-09-15"
---

# Feature Implementation: Tokenized Python comment scanner

This document tracks the technical implementation details, architectural decisions, and verification steps for Issue #303.

## 1. Scope and Implementation Details

- **Issue:** #303 (Tokenized Python comment scanner)
- **Goal:** Correctly parse Python witness tags from `tokenize.COMMENT` tokens to prevent false-positive phantom witness findings triggered by string literals in test files.

### 1.1 Technical Changes
- Updated `crates/verify-baseline/src/checks/traceability.rs` to tokenize comment regions when parsing source files.
- The tokenizer falls back cleanly to line-splitting if lexical scanning encounters malformed syntax.
- Only comment tokens are passed to `_parse_witness_tags()`, completely ignoring string literals.
- Added unit test `test_witness_registry_ignores_phantom_witnesses_in_string_literals` to `crates/verify-baseline/tests/` to assert string literals do not emit phantom witness findings.

## 2. Code Realization Table

| Feature / Attribute | Implemented Source Files | Classes / Methods | Platform Extension |
| :--- | :--- | :--- | :--- |
| **Tokenized Comment Parsing** | `crates/verify-baseline/src/checks/traceability.rs` | `build_witness_registry` | `.rs` |
| **Ignore String Literals Verification** | `crates/verify-baseline/tests/` | `test_witness_registry_ignores_phantom_witnesses_in_string_literals` | `.rs` |

## 3. Verification & Testing

### 3.1 Unit Testing
- Executed `cargo test --workspace`
- Confirmed all tests passed, including the new assertions.
- Confirmed the fix strictly prevents `OBL-PHANTOM-99` strings from registering as phantom obligations.

### 3.2 Linter & Quality Gates
- Ran `./target/release/reconcile-backlog` to verify no regressions in overall parity auditor constraints.
- Tests passed cleanly. All code paths preserve 1.9 Zero-Mocking configurations and 3-Layer LUI targets.
