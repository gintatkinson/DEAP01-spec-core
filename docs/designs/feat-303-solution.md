---
issue_id: 303
title: Tokenized Python comment scanner
---

# Feature Implementation: Tokenized Python comment scanner

This document tracks the technical implementation details, architectural decisions, and verification steps for Issue #303.

## 1. Scope and Implementation Details

- **Issue:** #303 (Tokenized Python comment scanner)
- **Goal:** Correctly parse Python witness tags from `tokenize.COMMENT` tokens to prevent false-positive phantom witness findings triggered by string literals in test files.

### 1.1 Technical Changes
- Updated `parity_auditor/validators/obligation_witness_validator.py` to use `tokenize.generate_tokens()` when parsing `.py` files.
- The `tokenize` generator is wrapped in a `try...except` block catching `tokenize.TokenError` and `IndentationError` to fallback to raw line-splitting if tokenization fails.
- Only tokens of type `tokenize.COMMENT` are passed to `_parse_witness_tags()`, completely ignoring `tokenize.STRING`.
- Added unit test `test_witness_registry_ignores_phantom_witnesses_in_python_string_literals` to `tests/test_coverage_digest_and_witness_registry.py` to assert string literals do not emit phantom witness findings.

## 2. Code Realization Table

| Feature / Attribute | Implemented Source Files | Classes / Methods | Platform Extension |
| :--- | :--- | :--- | :--- |
| **Tokenized Comment Parsing** | `skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/obligation_witness_validator.py` | `build_witness_registry` | `.py` |
| **Ignore String Literals Verification** | `tests/test_coverage_digest_and_witness_registry.py` | `test_witness_registry_ignores_phantom_witnesses_in_python_string_literals` | `.py` |

## 3. Verification & Testing

### 3.1 Unit Testing
- Executed `PYTHONPATH=skills/spec-orchestrator/parity_auditor/src python3 -m unittest tests/test_coverage_digest_and_witness_registry.py`
- Confirmed all tests passed, including the new assertions.
- Confirmed the fix strictly prevents `OBL-PHANTOM-99` strings from registering as phantom obligations.

### 3.2 Linter & Quality Gates
- Ran `reconcile_backlog.py` to verify no regressions in overall parity auditor constraints.
- Tests passed cleanly. All code paths preserve 1.9 Zero-Mocking configurations and 3-Layer LUI targets.
