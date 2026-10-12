# Markdown Specification & Requirements Ingestion Specification

## 1. Overview & Architectural Principles

The `ingest-sysml` crate provides pure Rust Ground 0.0 ingestion for Level 0 OEM requirements, semi-structured Markdown documents, Bill of Materials (BOM) tables, interface/port definitions, and parametric constraints.
It translates raw requirement specifications into canonical SysML v2 AST structures (`deap_core::sysml_ast::PackageDef`) and outputs compliant SysML v2 textual models that compile cleanly via `compile-sysml`.

Strict Invariants:
- Pure schema-driven compiler: zero hardcoded domain concepts. All identifiers and structures derive deterministically from input text and tables.
- Zero Unicode em dashes (\u2014): use ASCII `--` or `-` exclusively across all code and documentation.
- Zero-mocking live persistence mandate (.pipeline/constitution.md Section 1.9).
- Full compatibility with the canonical `compile-sysml` parser and serializer.

---

## 2. Requirement Document Structure (`REQ-*.md`)

OEM requirement files conform to the following markdown structural schema:

### 2.1 YAML Frontmatter
```yaml
---
id: REQ-0001
title: "Abstract MBSE Compiler Mandate & Pure Schema-Driven Execution"
subsystem: "Subsystem 1: System Vision, Bootstrapping & Foundational Invariants"
uuidv5: 5a4d7a99-0419-5c2b-ba0e-ccabdb05f1c9
---
```
Fields extracted:
- `id`: Unique requirement identifier string (e.g. `REQ-0001`).
- `title`: Human-readable title of the requirement specification.
- `subsystem`: Architectural subsystem or grouping domain.
- `uuidv5`: Deterministic RFC 4122 namespace UUID string.

### 2.2 Title Heading & Contract Metadata Table
Heading 1 declares package scope:
```markdown
# [REQ-0001] Abstract MBSE Compiler Mandate & Pure Schema-Driven Execution
```
Package name sanitization strips the bracketed identifier, yielding:
`Abstract_MBSE_Compiler_Mandate_Pure_Schema_Driven_Execution`.

Metadata table:
```markdown
| Metadata Field | Contract Specification |
| :--- | :--- |
| **Requirement ID** | `REQ-0001` |
| **Deterministic UUIDv5** | `5a4d7a99-0419-5c2b-ba0e-ccabdb05f1c9` |
| **Subsystem** | Subsystem 1: System Vision, Bootstrapping & Foundational Invariants |
| **Complexity Class** | Class $\mathbf{P}$ ($O(N)$ linear scan & AST graph construction) |
| **NP Frontier Anchor** | N/A (Deterministic P) |
| **Diagnostic Code Bindings** | `E0100` |
| **Governing Standard** | IEEE 29148-2018 / RFC 2119 |
```
Metadata rows map to package-level `AttributeDef` declarations with sanitized attribute names and typed literals.

### 2.3 Section 1: Normative Statement
Declared via `## 1. Normative Statement`.
Contains the authoritative behavioral requirements, prose constraints, and diagnostic error codes.
Translated to `PartDef` named `_1_Normative_Statement` with attached prose doc comment.

### 2.4 Section 2: Formal Invariant
Declared via `## 2. Formal Invariant`.
Contains mathematical invariant formulas in KaTeX / LaTeX math blocks (`$$...$$` and ```math ... ```).
Translated to `PartDef` named `_2_Formal_Invariant` with attached mathematical invariant doc comment.

### 2.5 Section 3: Computational Complexity & Algorithmic Bounds
Declared via `## 3. Computational Complexity & Algorithmic Bounds`.
Specifies algorithmic time/space bounds, NP-hard frontier bypass criteria, and termination guarantees.
Translated to `PartDef` named `_3_Computational_Complexity_Algorithmic_Bounds`.

### 2.6 Section 4: Verification & Conformance Criteria (Acceptance Criteria)
Declared via `## 4. Verification & Conformance Criteria (IEEE 29148 Acceptance Criteria)`.
Contains subsection headings `### AC-01: ...`, `### AC-02: ...`, etc.
Each acceptance criterion defines:
- **Given:** Precondition domain state.
- **When:** Stimulus or operational trigger.
- **Then:** Expected outcome, invariant preservation, or actuator output.
- **Diagnostic:** Diagnostic error code emitted on failure.

Each AC translates to a distinct `PartDef` (e.g. `AC_01_Pure_Schema_Driven_Symbol_Derivation`) with its Given-When-Then BDD scenario captured in its SysML docstring.
