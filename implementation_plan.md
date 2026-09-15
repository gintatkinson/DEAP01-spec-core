# Goal Description

Implement Work Package 4.2 to resolve Issue #303 (Tokenized Python comment scanner). This will prevent Python string literals containing obligation witness tags (like `OBL-PHANTOM-99` used in tests) from leaking into `registry.phantom_witnesses` by using Python's `tokenize` module to only scan tokens of type `tokenize.COMMENT`.

## User Review Required

Please review this implementation plan. Since the prompt included the PROCEED keyword, I am still required by the Strict Planning Gate to verify the implementation plan with you first before directly making code modifications.

## Open Questions

None. The requirements are clear.

## Proposed Changes

### Core Auditing Logic

#### [MODIFY] obligation_witness_validator.py
- **Path:** `/Users/perkunas/jail/DEAP01-spec-core/skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/obligation_witness_validator.py`
- Add imports for `tokenize` and `io`.
- Modify `build_witness_registry` Step 3 (Test Witnesses) and Step 4 (Code Witnesses):
  - When scanning a `.py` file, instantiate `io.StringIO(content)` and use `tokenize.generate_tokens()`.
  - Filter for tokens where the token type equals `tokenize.COMMENT`.
  - Scan the token string for witness tags.
  - Track line numbers from the token metadata (1-indexed).
  - Add a fallback mechanism (e.g. via try-except on `tokenize.TokenError` or generic Exception) that reverts to raw splitlines if tokenization fails.

### Tests

#### [MODIFY] test_coverage_digest_and_witness_registry.py
- **Path:** `/Users/perkunas/jail/DEAP01-spec-core/tests/test_coverage_digest_and_witness_registry.py`
- Add a new test method (e.g., `test_witness_registry_ignores_python_string_literals`).
- Construct a dummy `.py` file that includes a string literal containing `/// ObligationWitness: [OBL-PHANTOM-99]`.
- Verify that `build_witness_registry` does not flag it as a phantom witness.

## Verification Plan

### Automated Tests
- Run `PYTHONPATH=skills/spec-orchestrator/parity_auditor/src python3 -m unittest tests/test_coverage_digest_and_witness_registry.py` from the root directory `/Users/perkunas/jail/DEAP01-spec-core` and ensure 0 issues.
- Run the linter `flake8` or `black --check` (or similar, if applicable) to ensure structural correctness.

### Manual Verification
- Code review of the changes to confirm tokenization logic is correctly integrated and falls back properly.
