# Implementation Plan: Adversarial Code Audit of PR #409 (SysML v2 Compiler & AST Engine)

## 1. Executive Summary & Objective

This implementation plan governs the systematic adversarial code audit of Pull Request [#409](https://github.com/gintatkinson/DEAP01-spec-core/pull/409) (`review(compiler): SysML v2 Compiler & AST Parser Engine`) against the four core correctness risk pillars plus semantic traceability, in accordance with `.agents/skills/adversarial-code-auditor/SKILL.md` and repository governance in `.pipeline/constitution.md`.

### Audit Targets in Scope
1. `scripts/compile_sysml.py` (5,379 additions):
   - SysML v2 model compiler, validator, and textual model serializer.
   - Closed-loop bidirectional synchronization (`--forward-sync` and `--reverse-sync`).
   - STPA Cartesian product hazard matrix generator ($U = A \times G$) and Run-Time Assurance (RTA) / SLDV formal proof synthesizer.
   - FMECA criticality scoring engine.
2. `skills/spec-orchestrator/scripts/sysmlv2_ast.py` (3,192 additions):
   - SysML v2 / KerML lexical scanner and tokenizer.
   - Strongly-typed AST representations: `Package`, `PartDef`, `ItemDef`, `PortDef`, `ActionDef`, `StateDef`, `ConstraintDef`, `UseCaseDef`, `TestCaseDef`.
   - AST traversal, serialization, and visitor dispatch mechanisms.

---

## 2. Risk Pillars & Focus Areas

| Risk Pillar | Audit Focus & Failure Modes | Target Files |
|---|---|---|
| **Resource Lifecycle** | Unclosed file descriptors in model serializers/deserializers, unbounded memory growth during recursive AST parsing, missing flush on atomic file writes, regex engine catastrophic backtracking (ReDoS). | `compile_sysml.py`, `sysmlv2_ast.py` |
| **Concurrency & Re-entrancy** | AST node shared mutation during traversal, global state contamination across compiler invocations, race conditions during bidirectional sync, non-reentrant tokenizers. | `compile_sysml.py`, `sysmlv2_ast.py` |
| **Semantic Traceability & Invariants** | SysML v2 / KerML OMG 2023-02-01 specification fidelity, pure schema-driven compiler invariant (zero hardcoded domain concepts), preservation of 6 core model constructs, exact Cartesian product math in STPA expansion, bidirectional sync roundtrip lossless preservation. | `compile_sysml.py`, `sysmlv2_ast.py` |
| **Test Integrity & Regression Harness** | Bare assertions without diagnostics, missing AST edge cases (nested sub-packages, anonymous ports, circular specialization hierarchies), presence of reproducer test targets for all discovered defects. | `compile_sysml.py`, `sysmlv2_ast.py` |
| **Memory Safety & Recursion Bounds** | Deeply nested KerML block expressions causing recursion depth overflow (`RecursionError`), circular reference memory leaks in AST parent/child cyclic references, unbounded string concatenations in code generation loops. | `compile_sysml.py`, `sysmlv2_ast.py` |

---

## 3. Work Packages

### WP-01: Adversarial Code Audit of `scripts/compile_sysml.py` [COMPLETED]
- **Target File**: `scripts/compile_sysml.py`
- **Assigned Worker**: Context-isolated subagents (`CompileSysml Resource Auditor`, `CompileSysml Semantic Auditor`, `CompileSysml Concurrency Auditor`)
- **Findings Filed**:
  - [#411](https://github.com/gintatkinson/DEAP01-spec-core/issues/411): `[AUDIT] [compile_sysml.py]: Uncleaned temporary files leak on atomic write exception paths` (Important)
  - [#412](https://github.com/gintatkinson/DEAP01-spec-core/issues/412): `[AUDIT] [compile_sysml.py]: Forward-sync pairs interactions to test cases positionally and drops requirement verification traceability` (Important)
  - [#413](https://github.com/gintatkinson/DEAP01-spec-core/issues/413): `[AUDIT] [compile_sysml.py]: Multi-file TOCTOU race window during schema compilation desynchronizes model and digest` (Important)

### WP-02: Adversarial Code Audit of `skills/spec-orchestrator/scripts/sysmlv2_ast.py` [COMPLETED]
- **Target File**: `skills/spec-orchestrator/scripts/sysmlv2_ast.py`
- **Assigned Worker**: Context-isolated subagents (`SysML AST Memory Auditor`, `SysML AST Semantic Auditor`, `SysML AST Resource Auditor`)
- **Findings Filed**:
  - [#414](https://github.com/gintatkinson/DEAP01-spec-core/issues/414): `[AUDIT] [sysmlv2_ast.py]: RequirementDef Normative Text Dropped on Roundtrip and Shadowed by Docstrings` (Important)
  - [#415](https://github.com/gintatkinson/DEAP01-spec-core/issues/415): `[AUDIT] [sysmlv2_ast.py]: SysMLTestCaseDef Verify Requirement Parsing Truncates Namespaces and Multi-Requirement Lists` (Important)
  - [#416](https://github.com/gintatkinson/DEAP01-spec-core/issues/416): `[AUDIT] [sysmlv2_ast.py]: Hardcoded Domain Protocol Regex False Positives Violate Pure Schema Compiler Invariant` (Important)
  - [#417](https://github.com/gintatkinson/DEAP01-spec-core/issues/417): `[AUDIT] [sysmlv2_ast.py]: SysMLPackage Omits Hierarchical Collectors for TestCases, Interactions, and ItemDefs` (Important)
  - [#418](https://github.com/gintatkinson/DEAP01-spec-core/issues/418): `[AUDIT] [sysmlv2_ast.py]: Unbounded Recursion Depth in Cyclic PartDef Traversal Causes Call Stack Exhaustion (RecursionError)` (Important)
  - [#419](https://github.com/gintatkinson/DEAP01-spec-core/issues/419): `[AUDIT] [sysmlv2_ast.py]: Recursive Subpackage Traversal Duplicates AST Elements and Causes Severe GC Churn` (Important)
  - [#420](https://github.com/gintatkinson/DEAP01-spec-core/issues/420): `[AUDIT] [sysmlv2_ast.py]: Exponential Memory Duplication in Package Element Accessors Causes Resource Exhaustion` (Important)

### WP-03: Defect Dossier Verification & Issue Tracker Submission [COMPLETED]
- All 10 defect dossiers verified with 12 mechanical checks (offline Mermaid syntax gate, 5 Whys, test-target discovery annotations).
- All 10 issues created on GitHub issue tracker and verified live.

### WP-04: PR Review Synthesis & Walkthrough Report [COMPLETED]
- Unified review comment published to PR [#409](https://github.com/gintatkinson/DEAP01-spec-core/pull/409#issuecomment-5992172098).
- Final audit dossier report delivered to coordinator.

---

## 4. Verification Gates Matrix

| Gate | Verification Check | Acceptance Standard | Status |
|---|---|---|---|
| **7-Section Skeleton Gate** | Check 1: `^## [1-7]\.` count | Exactly 7 sections per finding | **PASSED** |
| **Audit Source & Metadata** | Checks 2-5: Audit Source, SEVERITY, FILE_LOCATION, Section 1 bullets | 100% compliant with SKILL.md Section 2 | **PASSED** |
| **5 Whys Completeness** | Check 6: `^[1-5]\. \*\*Why .*\?\*\* Because .*` | Exactly 5 Whys per finding | **PASSED** |
| **Mermaid Syntax Offline Gate** | Check 7: `parity_auditor.validators.mermaid_syntax_validator.check_mermaid_text()` | Exit code `0` for all Critical/Important diagrams | **PASSED** |
| **Code Block Balancing** | Check 9: Triple-backtick balancing | Even count, valid language tags | **PASSED** |
| **Title Format Gate** | Check 11: `^\[AUDIT\] \[[^\]]+\]: .+` | Strict naming standard matching | **PASSED** |
| **Test-Target Discovery Gate** | Check 12: `<!-- test-target: [path] -->` annotation | Valid test-target path annotated in every finding | **PASSED** |
| **Remote Issue Verification Gate** | `gh issue view <id>` for all filed issues | Published payload verified live on GitHub | **PASSED** |
