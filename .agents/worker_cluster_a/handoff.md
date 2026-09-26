# Handoff Report: Cluster A Implementer (R2: Positive AST Provenance & Anti-Regex Hardening)

## 1. Observation
- **Issue #378 & #376 (Epistemic Exemption Regex Bypass)**:
  * In `skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/factual_grounding_validator.py`, `_has_epistemic_exemption()` and `EPISTEMIC_EXEMPTION_PATTERN` previously matched strings such as `[TIER-3: DESIGN]`, `(TIER-3)`, `[TIER-4: TBD]`, `(Declared Assumption)`, or `Declared Assumption`.
  * In `_validate_structural_assertions`, `_validate_numeric_assertions`, and `_validate_protocols`, finding this pattern caused the validator to execute `continue`, unconditionally exempting the entire line from structural count checks, numeric bound validation, and protocol verification.
  * Downstream specifications were evading factual grounding by appending dummy epistemic exemption tags to fabricated OEM numbers and protocols.
- **Issue #364 (Mermaid and Code Block Parsing Bypasses)**:
  * In `_validate_numeric_assertions` (`factual_grounding_validator.py`), lines inside markdown code blocks (`if in_code_block: continue`) unconditionally skipped numeric validation.
  * In Mermaid diagrams, parsing was gated on `in_sequence_diagram` (`if not in_sequence_diagram: continue`), completely skipping `stateDiagram`, `stateDiagram-v2`, `flowchart`, `graph`, and `classDiagram` blocks. Furthermore, lines matching `MERMAID_STRUCTURAL_KEYWORDS_PATTERN` (including `alt `, `opt `, `par `) skipped numeric evaluation.
- **Issue #377 (Lack of Typed Parameter Dictionary AST Projection)**:
  * `skills/schema-specification-engineering/SKILL.md` instructed extracting raw package AST slices without pre-compiling or projecting a closed-world typed parameter dictionary into subagent prompt payloads.
  * Generative subagents lacked an authoritative list of declared attributes, types, units, and bounds, leading to metric hallucination in architectural specifications.
  * `SchemaGroundTruth` and `FactualGroundingValidator` lacked projection methods for export and markdown formatting.

## 2. Logic Chain
1. *Eliminating Negative Regex Heuristics (#378, #376)*:
   - Deprecated `_has_epistemic_exemption()` to emit `DeprecationWarning` and unconditionally return `False`.
   - Removed `if _has_epistemic_exemption(line_str): continue` from `_validate_structural_assertions`, `_validate_numeric_assertions`, and `_validate_protocols`.
   - Enforced positive closed-world AST provenance: any claim containing numbers, counts, or protocols must resolve against AST nodes extracted from `.pipeline/schema.sysml` or carry an explicit verified SSOT citation (`<!-- Source: schema/... §locator -->` or `%% Source: ...`).
2. *Hardening Mermaid and Code Block Numeric Grounding (#364)*:
   - In `_validate_numeric_assertions` and `_validate_protocols`, generalized code block and Mermaid parsing so that:
     * Mermaid diagram headers (`sequenceDiagram`, `stateDiagram-v2`, `classDiagram`, `flowchart TD`) and pure layout lines (`autonumber`, `direction LR`, `end`) are skipped.
     * All diagram bodies (sequence messages, state transitions with guard conditions, flowchart edges, class attributes, notes) are evaluated for physical numeric quantities and protocols.
     * Markdown code blocks (YAML, text, configuration snippets) are not unconditionally skipped; physical dimensional quantities are validated against AST ground truth.
     * Comment-level citations (`%% Source: ...`, `# Source: ...`, `// Source: ...`, `<!-- Source: ... -->`) are parsed across both diagram and code fences to allow legitimate grounded references.
3. *Projecting Closed-World Typed Parameter Dictionaries (#377)*:
   - Added `to_typed_parameter_dictionary()` and `format_typed_parameter_dictionary_markdown()` to `SchemaGroundTruth` and `FactualGroundingValidator`.
   - Formatted parameter dictionaries project name, type (`Integer`, `Number`, `String`), unit, value, owning component, and bound type (`upper`, `lower`, `nominal`, `exact`) using closed M2 metamodel types.
   - Updated `skills/schema-specification-engineering/SKILL.md` to add `## Closed-World AST Parameter Projection & Anti-Hallucination Mandate` and updated Step 1 (Epic dispatch) and Step 2 (Feature dispatch) to mandate injecting the typed parameter dictionary into all subagent prompt payloads.
4. *TDD Verification*:
   - Implemented 13 unit tests in `tests/test_factual_grounding_validator.py` and 3 unit tests in `tests/test_ast_manifest_dispatch_contracts.py`.
   - Verified the initial failing RED state (11 failures) and resolved each to achieve 100% pass rate (16/16 pass in 0.15s).
   - Validated against full baseline via `python3 scripts/verify_downstream_baseline.py .` with exit code 0 (Check 19 and Check 23 passing).

## 3. Caveats
- Non-dimensional integers or code loops without ISO/IEC 80000 units (e.g. `for i in range(10):`) in code blocks naturally pass numeric checks because `ISO_80000_PHYSICAL_UNITS.get(unit.lower())` is `None`. This correctly avoids false positives on non-physical code syntax.
- Legacy tests in other clusters (e.g., `tests/test_polyrepo_propagation_gate.py` for Cluster B and mock tests in Cluster D) are owned by peer workers and do not impact Cluster A files.

## 4. Conclusion
All four targeted issues (#378, #377, #376, #364) are fully remediated:
- Negative regex epistemic exemption bypasses are eliminated; `_has_epistemic_exemption()` is deprecated and returns `False`.
- Positive closed-world AST provenance is enforced across all documents, Mermaid diagrams (sequence, state, flow, class), and markdown code blocks.
- Typed parameter dictionary AST projection and prompt injection mandates are codified in `skills/schema-specification-engineering/SKILL.md` and implemented in `factual_grounding_validator.py`.
- 100% test pass rate with zero regressions across all Cluster A files.

## 5. Verification Method
Execute the following verification commands from the repository root:
```bash
# 1. Run Cluster A TDD test suites (16 tests, 100% pass)
python3 -m pytest tests/test_factual_grounding_validator.py tests/test_ast_manifest_dispatch_contracts.py

# 2. Run parity_auditor factual grounding test suite
python3 -m pytest skills/spec-orchestrator/parity_auditor/tests/test_factual_grounding_validator.py

# 3. Verify downstream baseline Check 19 and Check 23 compliance
python3 scripts/verify_downstream_baseline.py .

# 4. Verify syntax compilation
python3 -m py_compile skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/factual_grounding_validator.py tests/test_factual_grounding_validator.py tests/test_ast_manifest_dispatch_contracts.py
```
