| Attribute | Specification Detail |
| :--- | :--- |
| **Blueprint ID** | `BP-05` |
| **Requirement ID** | `REQ-0605` |
| **Title** | Lossless CST Safety Docstring Synthesizer and Check 36 Verification Blueprint |
| **Status** | `Approved` |
| **Target Crates** | `crates/deap-core, crates/compile-sysml, crates/verify-baseline` |
| **Tracked Issues** | `#498, #500, #501, #508, #510, #742` |
| **Generation Mode** | `subagent` |
| **Specification Source** | `Cargo.toml:1-40 / crates/deap-core/src/lib.rs:1-40 / crates/compile-sysml/src/lib.rs:1-40 / crates/verify-baseline/src/lib.rs:1-40` |

## 1. Executive Summary & Derivation-Annotation Contract

The Digital Engineering Agent Platform (DEAP) specification and verification toolchain (`UnifiedSafetyDocstringSynthesizer`, blueprint reference `DEAP-BP-DET-05`, requirement **REQ-0605**, UUIDv5 `6ba7b810-9dad-11d1-80b4-00c04fd430c5`, Complexity Class P, diagnostic code bindings **E0100**, **E0304**, **E0400**, governing standards DO-178C Level A / ISO 26262 ASIL D / ECSS-E-ST-40C / IEEE 29148-2018) transitions all Rust safety docstring synthesis and baseline verification off fragile hand-rolled line, regex, and character scanners onto `ra_ap_syntax` (`ast::HasDocComments`, `ast::HasAttrs`), `ra_ap_text_edit` (`TextEdit`, `TextEditBuilder`), `ra_ap_line_index` (`LineIndex`), `ra_ap_vfs` (`Vfs`, `FileId`), `ra_ap_paths` (`AbsPathBuf`, `RelPathBuf`), `rowan` (`TextRange`, `TextSize`, `SyntaxNode`, `SyntaxToken`), `camino` (`Utf8Path`, `Utf8PathBuf`), and `ignore` (`WalkBuilder`) under **Contract A** and **Contract B**. Historically, over 8,200 lines of heuristic string slicing and ad-hoc `.lines()` parsers were duplicated across `compile-sysml` and `verify-baseline`, causing false-positive docstring violations, string-literal mangling, comment-token collision, and severe docstring bloat where non-actuator helper items were populated with redundant multi-section templates.

| Claim / Requirement | SSOT Input Anchor (file:line / #ID) | Methodology Citation | Derivation Grade (Directly Evidenced / Analytically Derived / Declared Assumption) | Caveat / Downstream Impact |
| :--- | :--- | :--- | :--- | :--- |
| **REQ-0605**: Lossless CST docstring synthesis and Check 36 verification via `ra_ap_syntax` (`ast::HasDocComments`, `ast::HasAttrs`) + `ra_ap_text_edit` + `ra_ap_line_index` + `ra_ap_vfs` + `ra_ap_paths` + `rowan` + `camino` + `ignore` (**Contract A** & **Contract B**) | `Cargo.toml:1-40 / crates/deap-core/src/lib.rs:1-40` / `#498, #500, #501, #508, #510, #742` | `DO-178C Level A / ISO/IEC/IEEE 29148:2018` | `Directly Evidenced` | Replaces all hand-rolled Rust line/comment scanners with lossless CST traversal, virtual filesystem indexing, surgical byte-span text editing, and zero `#![feature(rustc_private)]`. |
| **SC-01**: Explicit legacy hand-rolled scanner deletion (`~8,200+` SLOC) and 3-Layer DoD `< 380` exec SLOC submodule decomposition (**Contract A** & **Contract B**) | `crates/compile-sysml/src/lib.rs:1-40 / crates/verify-baseline/src/lib.rs:1-40` / `#742` | `Leveson STPA / Design by Contract (DbC)` | `Directly Evidenced` | Strips `-4,900+` SLOC of legacy docstring scanners in `compile-sysml` and `-3,300+` SLOC in `verify-baseline`, replacing them with four focused `< 380` exec SLOC submodules closing all 3 DoD layers. |
| **SC-02**: Lossless CST docstring traversal reading exact `///`, `//!`, and `#[doc = "..."]` via `ra_ap_syntax::ast::{HasDocComments, HasAttrs}` and `rowan::TextRange` without touching literals or code comments (**Contract A**) | `crates/deap-core/src/lib.rs:1-40 / crates/compile-sysml/src/main.rs:1-40` / `#500, #501, #508` | `DO-178C Section 6.3.4 / DO-330 TQL-4` | `Directly Evidenced` | Leverages `ra_ap_syntax`, `rowan`, `ra_ap_vfs`, `ra_ap_paths`, `camino`, and `ignore` to isolate outer/inner doc comments and doc attributes strictly from string literals and inline comments. |
| **SC-03**: Anti-bloat Tier-A vs Tier-B/C contract (`<= 4`-line Tier-B/C `[REQ-*]`/`[FLS: fls_*]` docstrings; multi-section Tier-A strictly for explicit `[DOC-TIER: A]` entrypoints) (**Contract B**) | `crates/compile-sysml/src/lib.rs:1-40 / crates/verify-baseline/src/main.rs:1-40` / `#498, #510` | `OMG SysML v2 / MIL-STD-1629A` | `Analytically Derived` | Halts docstring bloat by enforcing compact `<= 4`-line docstrings carrying `[REQ-*]` and `[FLS: fls_*]` for all internal items, reserving 7-section blocks exclusively for explicit `[DOC-TIER: A]` actuators. |
| **SC-04**: Bit-for-bit idempotent synthesis ($\Delta_{\text{idempotent}}(S) = 0$) via `ra_ap_text_edit::TextEdit`, `ra_ap_line_index::LineIndex`, and anti-padding compaction (**Contract A** & **Contract B**) | `Cargo.toml:1-40 / crates/verify-baseline/src/lib.rs:1-40` / `#498` | `ECSS-E-ST-40C / ISO 26262 Part 6` | `Analytically Derived` | Guarantees non-overlapping surgical byte-span edits, bit-for-bit idempotency on repeated application, `--compact-tier-bc` compaction, and pre-built binary `.sysml` roundtrip parity. |

### Net-Negative Legacy Scanner Deletion & Submodule Decomposition Targets

Forensic audit of issues `#498`, `#500`, `#501`, `#508`, `#510`, and `#742` identified that the workspace maintained ~8,200+ SLOC of fragile, hand-rolled Rust docstring and comment parsers that attempted to simulate compiler lexical analysis with string lookaheads, regexes, and magic byte offsets:
1. **`compile-sysml` Hand-Rolled Docstring Scanner Deletion (`-4,900+` SLOC)**: Explicitly deletes and retires 4,900+ SLOC of manual line scanners, delimiter state trackers, and template replacers in `compile-sysml` (`crates/compile-sysml/src/lib.rs:1-40`).
2. **`verify-baseline` Check 36 Hand-Rolled Gate Deletion (`-3,300+` SLOC)**: Explicitly deletes and retires 3,300+ SLOC of heuristic docblock auditors, section scanners, and brace-depth balancers in `verify-baseline` (`crates/verify-baseline/src/lib.rs:1-40`).
3. **Net Legacy Scanner Deletion Target**: `~8,200+` SLOC net deletion of unmaintainable hand-rolled text-scanning boilerplate across the toolchain.
4. **3-Layer DoD Target Submodule Layout (`< 380` Executable SLOC per Module)**:
   - `deap_core::rust_cst::doc_edit` (planned `doc_edit.rs` under `crates/deap-core/src/`, `<= 340` exec SLOC, DoD Layer 1 & 2): Lossless CST item extraction, `ast::HasDocComments` / `ast::HasAttrs` traversal, `rowan::TextRange` span computation, `ra_ap_vfs` / `ra_ap_paths` / `camino` / `ignore` file discovery, and `ra_ap_text_edit` surgical span planning.
   - `compile_sysml::docstring_cst` (planned `docstring_cst.rs` under `crates/compile-sysml/src/`, `<= 360` exec SLOC, DoD Layer 2 & 3): Forward deterministic docstring synthesis, `--compact-tier-bc` anti-padding compaction, and idempotent file injection.
   - `verify_baseline::checks::docstring_cst_gate` (planned `docstring_cst_gate.rs` under `crates/verify-baseline/src/checks/`, `<= 280` exec SLOC, DoD Layer 2 & 3): Check 36 lossless CST completeness verification and non-zero CLI diagnostic reporting.
   - `verify_baseline::checks::docstring_tier_rules` (planned `docstring_tier_rules.rs` under `crates/verify-baseline/src/checks/`, `<= 240` exec SLOC, DoD Layer 1 & 2): Tier-A (`[DOC-TIER: A]`) vs Tier-B/C validation, `<= 4`-line Tier-B/C ceiling, and `[REQ-*]` / `[FLS: fls_*]` anchor validation.

### Contract A & Contract B Architectural Governance

1. **Contract A (Lossless CST Crate Stack & Explicit Hand-Rolled Scanner Deletion)**:
   - Enforces deterministic AST/CST inspection and workspace traversal exclusively through `ra_ap_syntax` (`ast::HasDocComments`, `ast::HasAttrs`), `ra_ap_text_edit` (`TextEdit`, `TextEditBuilder`), `ra_ap_line_index` (`LineIndex`), `ra_ap_vfs` (`Vfs`, `FileId`), `ra_ap_paths` (`AbsPathBuf`, `RelPathBuf`), `rowan` (`TextRange`, `TextSize`, `SyntaxNode`, `SyntaxToken`), `camino` (`Utf8Path`, `Utf8PathBuf`), and `ignore` (`WalkBuilder`).
   - Prohibits `#![feature(rustc_private)]`, `regex` crate usage (Check 34), `.unwrap()`, `.expect()`, `panic!`, and raw `[a..b]` slice indexing, and mandates explicit deletion of the ~8,200+ SLOC of legacy hand-rolled line/comment scanners.
2. **Contract B (Anti-Bloat Tier-A/B/C Docstrings, 3-Layer DoD `< 380`-SLOC Submodules & Pre-Built Binary Parity)**:
   - Enforces `<= 4`-line Tier-B/C docstrings carrying `[REQ-*]` and `[FLS: fls_*]` traceability anchors on all internal declarations, while restricting multi-section 7-part docblocks strictly to explicit `[DOC-TIER: A]` safety-actuator entrypoints.
   - Enforces `< 380` executable SLOC per submodule across all 3 Definition of Done layers (`Domain State & Data Model`, `Logic & State Management`, `Presentation & Actuator Interface Binding` with zero `N/A`), and keeps all `.sysml` constructs and `file:line` anchors (`Cargo.toml`, `crates/*/src/lib.rs`, `crates/*/src/main.rs`) 100% compatible with `./target/release/verify-baseline`.

### Deterministic Lossless CST Replacement Architecture

To eliminate heuristic docstring parsing and enforce anti-bloat tier rules under **Contract A** and **Contract B**:
1. **`ra_ap_syntax` & `rowan` Lossless Traversal**: Discover workspace `.rs` files deterministically via `ignore::WalkBuilder`, `camino::Utf8Path`, `ra_ap_paths::AbsPathBuf`, and `ra_ap_vfs::Vfs`. Parse Rust sources into `ra_ap_syntax::SourceFile` and inspect `ast::Item` nodes (`Fn`, `Struct`, `Enum`, `Trait`, `Impl`, `Const`, `TypeAlias`, `Module`) via `ast::HasDocComments` and `ast::HasAttrs` over `rowan::TextRange` spans. Extract raw `DocComment` tokens and `#[doc = "..."]` attributes directly from the CST, completely ignoring string literals, macro invocations, and interior code comments.
2. **`ra_ap_text_edit` & `ra_ap_line_index` Surgical Synthesis**: Use `ra_ap_line_index::LineIndex` to map line/column positions to UTF-8 `rowan::TextSize` byte offsets, and construct non-overlapping surgical edit plans with `ra_ap_text_edit::TextEditBuilder`. Second passes produce empty edit sets ($\Delta_{\text{idempotent}}(S) = 0$).
3. **Anti-Bloat Tier-A vs Tier-B/C Contract**: Multi-section docblocks are strictly restricted to explicit `[DOC-TIER: A]` safety-actuator entrypoints. All other documented declarations enforce concise `<= 4`-line Tier-B/C docstrings carrying `[REQ-*]` and `[FLS: fls_*]` traceability anchors.

---

## 2. Structural Component Metamodel

The `UnifiedSafetyDocstringSynthesizer` subsystem coordinates lossless CST docstring extraction, tiered forward synthesis, anti-padding compaction, and Check 36 baseline verification.

### Component: UnifiedSafetyDocstringSynthesizer

#### 2.1 Component Bill of Materials (BOM)

| Part Name | Type | Quantity | Description |
| :--- | :--- | :--- | :--- |
| `LosslessCstDocstringScanner` | Component | 1 | Canonical `< 380`-SLOC CST traversal engine in `deap_core::rust_cst::doc_edit` inspecting `ra_ap_syntax::ast::{HasDocComments, HasAttrs}`, `rowan::TextRange`, `ra_ap_vfs`, `ra_ap_paths`, `camino`, and `ignore` under Contract A |
| `TieredDocstringSurgicalSynthesizer` | Component | 1 | Forward `< 380`-SLOC docstring injector in `compile_sysml::docstring_cst` executing `ra_ap_text_edit` and `ra_ap_line_index` surgical plans with `--compact-tier-bc` under Contract A and Contract B |
| `DocstringTierEnforcementGate` | Component | 1 | Verifier in `< 380`-SLOC `verify_baseline::checks::docstring_tier_rules` enforcing explicit `[DOC-TIER: A]` actuator restriction and `<= 4`-line Tier-B/C `[REQ-*]` / `[FLS: fls_*]` ceiling under Contract B |
| `Check36CstCompletenessVerifier` | Component | 1 | Check 36 gate in `< 380`-SLOC `verify_baseline::checks::docstring_cst_gate` auditing 100% item coverage, legacy scanner retirement, and bit-for-bit idempotency under Contract A and Contract B |

#### 2.2 Component Attributes & State Properties

| Attribute | Type | Default | Unit | Description |
| :--- | :--- | :--- | :--- | :--- |
| `legacy_scanner_deletion_sloc` | `Integer` | `8200` | `lines` | Net-negative SLOC target for legacy hand-rolled docstring scanner deletion |
| `max_submodule_exec_sloc` | `Integer` | `380` | `lines` | Maximum permissible executable SLOC ceiling for newly decomposed CST submodules |
| `max_tier_bc_doc_lines` | `Integer` | `4` | `lines` | Maximum allowed docstring line count for Tier-B/C non-actuator declarations |
| `docstring_idempotency_diff_bytes` | `Integer` | `0` | `bytes` | Byte diff produced when re-running docstring synthesis on an already synthesized file |

#### 2.3 Component Interface Ports

| Port | Direction | Type | Protocol | Description |
| :--- | :--- | :--- | :--- | :--- |
| `SourceCodeFileInputPort` | `in` | `String` | `FilesystemUTF8` | Ingests CRLF/LF-normalized UTF-8 `.rs` source files via `camino::Utf8Path` |
| `SysmlSchemaTraceabilityPort` | `in` | `PackageDef` | `AstReference` | Supplies parsed SysML v2 `RequirementDef` and `ConstraintDef` AST hierarchies |
| `EnrichedSourceOutputPort` | `out` | `String` | `AtomicFileWrite` | Emits bit-for-bit idempotent `.rs` files enriched with surgically edited tiered docstrings |
| `Check36VerificationReportPort` | `out` | `Boolean` | `GateVerdict` | Emits deterministic Check 36 completeness, tier compliance, and idempotency verdicts |

#### 2.4 Formal Parametric & Structural Constraints

| Constraint ID | Parameter | Min | Max | Unit | Expression |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `C-DOC-01` | `legacy_scanner_deletion_sloc` | 8000 | 12000 | lines | `legacy_scanner_deletion_sloc >= 8000` |
| `C-DOC-02` | `max_submodule_exec_sloc` | 0 | 380 | lines | `max_submodule_exec_sloc <= 380` |
| `C-DOC-03` | `max_tier_bc_doc_lines` | 1 | 4 | lines | `max_tier_bc_doc_lines <= 4` |
| `C-DOC-04` | `docstring_idempotency_diff_bytes` | 0 | 0 | bytes | `docstring_idempotency_diff_bytes == 0` |

---

## 3. UML & Behavioral Topology

### 3.1 Structural Class Topology

```mermaid
classDiagram
    class UnifiedSafetyDocstringSynthesizer {
        +Integer legacy_scanner_deletion_sloc
        +Integer max_submodule_exec_sloc
        +Integer max_tier_bc_doc_lines
        +Integer docstring_idempotency_diff_bytes
        +extract_item_doc_info(Item item, LineIndex line_index) Option~DocCommentInfo~
        +build_tier_bc_docstring(String req_id, String fls_id, String summary) String
        +plan_docstring_surgical_edits(String source, Utf8Path file_path, bool compact_tier_bc) Result~DocstringSynthesisPlan~
        +apply_docstring_synthesis(String source, DocstringSynthesisPlan plan) Result~String~
        +audit_source_file_docstrings(Utf8Path file_path, String source) Result~DocstringGateViolationList~
    }
    class DocCommentInfo {
        +DocTier tier
        +usize line_count
        +bool has_req_anchor
        +bool has_fls_anchor
        +TextRange text_range
    }
    class DocstringSynthesisPlan {
        +Utf8PathBuf target_path
        +bool tier_bc_compacted
    }
    class DocstringGateViolation {
        +Utf8PathBuf file_path
        +usize line_number
        +String item_ident
        +String detail
    }
    UnifiedSafetyDocstringSynthesizer "1" *-- "1..*" DocCommentInfo : "inspects"
    UnifiedSafetyDocstringSynthesizer "1" *-- "1" DocstringSynthesisPlan : "builds"
    UnifiedSafetyDocstringSynthesizer "1" *-- "0..*" DocstringGateViolation : "reports"
```

### 3.2 Unified Lossless CST Synthesis & Check 36 Verification Flow

```mermaid
flowchart TD
    subgraph "Tier 1 - deap-core Lossless CST Engine"
        direction TB
        L1["ra_ap_syntax::SourceFile::parse<br/>CRLF/LF Normalized UTF-8"]
        L2["ast::HasDocComments & ast::HasAttrs<br/>Exact Comment Node Traversal"]
        L3["extract_item_doc_info<br/>DocCommentInfo Extraction"]
        L1 --> L2
        L2 --> L3
    end

    subgraph "Tier 2 - compile-sysml Surgical Synthesizer"
        direction TB
        S1["plan_docstring_surgical_edits<br/>TextEditBuilder Span Collector"]
        S2["build_tier_bc_docstring<br/>Anti-Bloat & Compact Tier-B/C"]
        S3["apply_docstring_synthesis<br/>ra_ap_text_edit Application"]
        S1 --> S2
        S2 --> S3
    end

    subgraph "Tier 3 - verify-baseline Check 36 & Tier Gate"
        direction TB
        V1["audit_source_file_docstrings<br/>100% Item Coverage Check"]
        V2["docstring_tier_rules<br/>Tier-A Actuator & Line Ceiling"]
        V3["Idempotency Verification<br/>Empty TextEdit Verification"]
        V1 --> V2
        V2 --> V3
    end

    L3 --> S1
    L3 --> V1
    S3 --> V1
```

### 3.3 End-to-End Deterministic Synthesis & Verification Sequence Diagram

```mermaid
sequenceDiagram
    actor CLI as "Compiler CLI / CI Pipeline"
    participant Core as "deap_core::rust_cst::doc_edit"
    participant Synth as "compile_sysml::docstring_cst"
    participant Gate as "verify_baseline::checks::docstring_cst_gate"
    participant Ingest as "ingest-sysml"

    CLI->>Synth: "plan_docstring_surgical_edits(source, path, compact)"
    activate Synth
    Synth->>Core: "extract_item_doc_info(item, line_index)"
    activate Core
    Core-->>Synth: "Return DocCommentInfo with tier, span, anchors"
    deactivate Core
    Synth->>Synth: "apply_docstring_synthesis(source, plan)"
    Synth-->>CLI: "Emit bit-for-bit idempotent source"
    deactivate Synth

    CLI->>Gate: "audit_source_file_docstrings(path, source)"
    activate Gate
    Gate->>Core: "extract_item_doc_info(item, line_index)"
    activate Core
    Core-->>Gate: "Return DocCommentInfo"
    deactivate Core
    Gate-->>CLI: "Emit Vec of DocstringGateViolation"
    deactivate Gate

    CLI->>Ingest: "Run ingest-sysml --schema blueprint-05 --format markdown"
    activate Ingest
    Ingest-->>CLI: "Emit canonical SysML v2 AST for REQ-0605 and SHA-256 digest"
    deactivate Ingest
```

---


## 4. Formal Mathematical & Safety Constraints

### 4.1 Mathematical Complexity & Idempotency Formulations

Under **REQ-0605** and parametric constraints `C-DOC-01` through `C-DOC-04`, lossless CST parsing, surgical text editing, and forward docstring injection satisfy the following linear complexity and idempotency bounds:

$$
\begin{aligned}
T_{\text{cst}}(B) &= O(B) \\
T_{\text{edit}}(E) &= O(E \log E) \\
\Delta_{\text{idempotent}}(S) &= \lVert f_{\text{inject}}(f_{\text{inject}}(S, M), M) - f_{\text{inject}}(S, M) \rVert_1 = 0
\end{aligned}
$$

- Parameter Definitions & Engineering Units:
- $T_{\text{cst}}(B)$: Lossless CST traversal and token classification time complexity over $B$ UTF-8 source bytes ($B \ge 0$).
- $T_{\text{edit}}(E)$: Time complexity for sorting, validating, and applying $E$ disjoint text edit operations via `ra_ap_text_edit::TextEdit`.
- $S$: Input source file byte vector prior to docstring synthesis.
- $M$: Authoritative SysML v2 `PackageDef` schema model containing requirement and constraint definitions.
- $f_{\text{inject}}(S, M)$: Deterministic forward docstring synthesis operator in `compile-sysml --inject-safety-docstrings`.
- $\Delta_{\text{idempotent}}(S)$: L1 byte-stream difference between first and second docstring synthesis passes ($\Delta_{\text{idempotent}}(S) = 0\text{ bytes}$).

### 4.2 Safety Constraints & Parametric Invariants

- **SC-01 (Net-Negative Deletion & 3-Layer DoD Modularity Invariant -- `C-DOC-01`, `C-DOC-02`, Contract A & Contract B)**: `legacy_scanner_deletion_sloc >= 8000` and `max_submodule_exec_sloc <= 380`. All hand-rolled line, regex, and character parsers in `compile-sysml` (`-4,900+` SLOC) and `verify-baseline` (`-3,300+` SLOC) are explicitly deleted and replaced with modular 3-Layer DoD submodules under 380 executable SLOC.
- **SC-02 (Lossless CST & VFS Traversal Invariant -- Contract A)**: Docstring tokens and attributes are discovered strictly via `ra_ap_syntax::ast::{HasDocComments, HasAttrs}`, `rowan::TextRange`, `ra_ap_line_index::LineIndex`, `ra_ap_vfs::Vfs`, `ra_ap_paths::AbsPathBuf`, `camino::Utf8Path`, and `ignore::WalkBuilder` over parsed `SourceFile` nodes. Raw string literals, comments inside function bodies, and test items are never confused with item docstrings.
- **SC-03 (Anti-Bloat Tier-A vs Tier-B/C Invariant -- `C-DOC-03`, Contract B)**: `max_tier_bc_doc_lines <= 4`. Items not marked with explicit `[DOC-TIER: A]` enforce concise docstrings carrying `[REQ-*]` and `[FLS: fls_*]` anchors spanning at most 4 lines, restricting multi-section 7-part docblocks strictly to `[DOC-TIER: A]` safety-actuator entrypoints.
- **SC-04 (Bit-for-Bit Synthesis Idempotency -- `C-DOC-04`, Contract A & Contract B)**: `docstring_idempotency_diff_bytes == 0`. Applying `DocstringSynthesisPlan` across any workspace crate via `ra_ap_text_edit::{TextEdit, TextEditBuilder}` produces zero diff on consecutive runs, with all edits represented as non-overlapping `rowan::TextRange` byte spans.

---

## 5. Acceptance Criteria & Verification Matrix

<!-- test-target: crates/verify-baseline/tests/blueprint_gate_tests.rs -->

- **Test-Target**: `crates/verify-baseline/tests/blueprint_gate_tests.rs`

### AC-01: Lossless CST Docstring Traversal via `ra_ap_syntax`
- **Given**: Rust source files discovered via `ignore::WalkBuilder`, `camino::Utf8Path`, `ra_ap_paths::AbsPathBuf`, and `ra_ap_vfs::Vfs` containing `///` outer line docstrings, `//!` inner module docstrings, `#[doc = "..."]` attributes, raw string literals containing doc markers (`r#"/// not a doc"#`), and interior block comments.
- **When**: `deap_core::rust_cst::doc_edit::extract_item_doc_info` parses the source file under **Contract A**.
- **Then**: Exact docstring tokens and `rowan::TextRange` spans are resolved via `ast::HasDocComments` and `ast::HasAttrs` without misclassifying string literals or interior code comments, and without invoking any hand-rolled `.lines()` or `.trim_start_matches("///")` scanners.

### AC-02: Deterministic Surgical Byte-Span Synthesis via `ra_ap_text_edit`
- **Given**: A Rust source file with undocumented or partially documented `ast::Item` nodes indexed with `ra_ap_line_index::LineIndex`.
- **When**: `compile_sysml::docstring_cst::plan_docstring_surgical_edits` generates a `DocstringSynthesisPlan` and applies it via `apply_docstring_synthesis` under **Contract A** and **Contract B**.
- **Then**: Docstrings are inserted or updated using non-overlapping `TextEditBuilder` and `TextEdit` byte spans, subsequent execution yields zero edits ($\Delta_{\text{idempotent}}(S) = 0$), and compilation requires zero `#![feature(rustc_private)]`.

### AC-03: Anti-Bloat Tier-A vs Tier-B/C Contract and Line Ceilings
- **Given**: A codebase containing both explicit `[DOC-TIER: A]` public safety-actuator entrypoints and internal Tier-B/C helper functions, structs, and type aliases.
- **When**: `verify_baseline::checks::docstring_tier_rules` and `docstring_cst_gate` audit the declarations under **Contract B**.
- **Then**: Multi-section Tier-A docblocks are validated exclusively on items with explicit `[DOC-TIER: A]` markers, while all other documented items enforce concise `<= 4`-line Tier-B/C docstrings carrying `[REQ-*]` and `[FLS: fls_*]` anchors (failing closed with a non-zero exit code if any Tier-B/C docstring exceeds 4 lines or lacks anchors).

### AC-04: Net-Negative Legacy Scanner Deletion and `< 380` Exec SLOC Ceiling
- **Given**: The legacy codebase containing ~8,200+ SLOC of hand-rolled string scanners in `compile-sysml` (`-4,900+` SLOC) and `verify-baseline` (`-3,300+` SLOC).
- **When**: The refactored CST architecture is deployed under **Contract A** and **Contract B**.
- **Then**: At least 8,000 legacy hand-rolled scanner SLOC are explicitly deleted, and every newly created 3-Layer DoD submodule (`doc_edit.rs`, `docstring_cst.rs`, `docstring_cst_gate.rs`, `docstring_tier_rules.rs`) strictly complies with the `< 380` executable SLOC ceiling (failing closed if any submodule exceeds 380 executable SLOC).

### AC-05: Dogfood Ingestion and SysML v2 Compilation of REQ-0605
- **Given**: This blueprint specification `docs/designs/blueprint-05-deterministic-safety-docstring-synthesizer.md` citing stable workspace `file:line` anchors (`Cargo.toml`, `crates/*/src/lib.rs`, `crates/*/src/main.rs`).
- **When**: `./target/release/verify-baseline check-blueprint docs/designs/blueprint-05-deterministic-safety-docstring-synthesizer.md --dry-run` and `./target/release/ingest-sysml --schema docs/designs/blueprint-05-deterministic-safety-docstring-synthesizer.md --format markdown` are executed.
- **Then**: All 12 mechanical verification gates pass (`12/12`), canonical SysML v2 AST structures are emitted, and the verification exits with code 0.

---

## 6. Proposed Deterministic Implementation Contract

### 6.1 Lossless CST Docstring Extraction & Classification (`deap_core::rust_cst::doc_edit`)

The following typed contracts define lossless CST extraction, virtual filesystem indexing, and non-overlapping surgical text editing across macOS, Linux, and Windows under **Contract A** and **Contract B** using `ra_ap_syntax` (`ast::HasDocComments`, `ast::HasAttrs`), `ra_ap_text_edit`, `ra_ap_line_index`, `ra_ap_vfs`, `ra_ap_paths`, `rowan`, `camino`, and `ignore` with zero `#![feature(rustc_private)]`:

```rust
use camino::{Utf8Path, Utf8PathBuf};
use ignore::WalkBuilder;
use ra_ap_line_index::LineIndex;
use ra_ap_paths::{AbsPathBuf, RelPathBuf};
use ra_ap_syntax::{ast::{self, HasAttrs, HasDocComments}, AstNode, SourceFile};
use ra_ap_text_edit::{TextEdit, TextEditBuilder};
use ra_ap_vfs::{FileId, Vfs, VfsPath};
use rowan::{TextRange, TextSize};

/// Maximum permitted line count for Tier-B/C non-actuator docstrings.
/// [REQ-0605] [FLS: fls_max_tier_bc_lines]
pub const MAX_TIER_BC_DOC_LINES: usize = 4;

/// Safety documentation tier classifying items into Tier-A actuators or Tier-B/C internal items.
/// [REQ-0605] [FLS: fls_doc_tier_decl]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocTier {
    TierA,
    TierBC,
}

/// Parsed docstring metadata extracted losslessly from `ra_ap_syntax` and `rowan` CST nodes.
/// [REQ-0605] [FLS: fls_doc_comment_info]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocCommentInfo {
    pub tier: DocTier,
    pub line_count: usize,
    pub has_req_anchor: bool,
    pub has_fls_anchor: bool,
    pub text_range: TextRange,
    pub byte_len: TextSize,
}

/// Concrete byte-span text edit plan containing non-overlapping `rowan::TextRange` replacements.
/// [REQ-0605] [FLS: fls_synthesis_plan]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocstringSynthesisPlan {
    pub target_path: Utf8PathBuf,
    pub abs_path: Option<AbsPathBuf>,
    pub rel_path: Option<RelPathBuf>,
    pub file_id: Option<FileId>,
    pub edits: Vec<(TextRange, String)>,
    pub tier_bc_compacted: bool,
}

/// Structured baseline verification failure emitted during Check 36 CST auditing.
/// [REQ-0605] [FLS: fls_gate_violation]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocstringGateViolation {
    pub file_path: Utf8PathBuf,
    pub line_number: usize,
    pub item_ident: String,
    pub detail: String,
}

/// Discovers `.rs` files via `ignore::WalkBuilder`, `camino::Utf8Path`, and `ra_ap_vfs::Vfs`.
/// [REQ-0605] [FLS: fls_discover_vfs_sources]
pub fn discover_rust_sources_into_vfs(
    root: &Utf8Path,
    vfs: &mut Vfs,
) -> Result<Vec<(FileId, Utf8PathBuf)>, String> {
    let mut loaded = Vec::new();
    for entry_res in WalkBuilder::new(root.as_std_path()).standard_filters(true).build() {
        let entry = entry_res.map_err(|err| err.to_string())?;
        let Some(utf8_path) = Utf8Path::from_path(entry.path()) else {
            continue;
        };
        if utf8_path.extension() == Some("rs") {
            let vfs_path = VfsPath::new_virtual_path(utf8_path.as_str().to_owned());
            let bytes = std::fs::read(utf8_path.as_std_path()).map_err(|err| err.to_string())?;
            vfs.set_file_contents(vfs_path.clone(), Some(bytes));
            if let Some((file_id, _)) = vfs.file_id(&vfs_path) {
                loaded.push((file_id, utf8_path.to_path_buf()));
            }
        }
    }
    Ok(loaded)
}

/// Extracts docstring metadata from an `ast::Item` via `ast::HasDocComments` and `ast::HasAttrs`.
/// [REQ-0605] [FLS: fls_extract_item_doc]
pub fn extract_item_doc_info(
    item: &ast::Item,
    line_index: &LineIndex,
) -> Option<DocCommentInfo> {
    let mut ranges = Vec::new();
    let mut combined = String::new();
    for comment in item.doc_comments() {
        ranges.push(comment.syntax().text_range());
        combined.push_str(comment.text());
        combined.push('\n');
    }
    for attr in item.attrs() {
        if attr.simple_name().as_deref() == Some("doc") {
            ranges.push(attr.syntax().text_range());
            combined.push_str(&attr.syntax().text().to_string());
            combined.push('\n');
        }
    }
    let first_range = ranges.first()?;
    let last_range = ranges.last()?;
    let range = TextRange::new(first_range.start(), last_range.end());
    let tier = if combined.contains("[DOC-TIER: A]") {
        DocTier::TierA
    } else {
        DocTier::TierBC
    };
    let line_start = line_index.line_col(range.start()).line;
    let line_end = line_index.line_col(range.end()).line;
    let line_count = line_end.saturating_sub(line_start).saturating_add(1) as usize;
    Some(DocCommentInfo {
        tier,
        line_count,
        has_req_anchor: combined.contains("[REQ-"),
        has_fls_anchor: combined.contains("[FLS: fls_"),
        text_range: range,
        byte_len: range.len(),
    })
}

/// Synthesizes a compact, anti-bloat Tier-B/C docstring adhering to the `<= 4`-line limit.
/// [REQ-0605] [FLS: fls_build_tier_bc]
pub fn build_tier_bc_docstring(
    req_id: &str,
    fls_id: &str,
    summary: &str,
) -> String {
    let clean_summary = summary.trim();
    let clean_req = req_id.trim();
    let clean_fls = fls_id.trim();
    format!("/// {clean_summary}\n/// [{clean_req}] [FLS: {clean_fls}]\n")
}

/// Plans surgical byte-span docstring edits over a parsed source file without overlap.
/// [REQ-0605] [FLS: fls_plan_edits]
pub fn plan_docstring_surgical_edits(
    source: &str,
    file_path: &Utf8Path,
    compact_tier_bc: bool,
) -> Result<DocstringSynthesisPlan, String> {
    let normalized = source.replace("\r\n", "\n");
    let parsed = SourceFile::parse(&normalized);
    let line_index = LineIndex::new(&normalized);
    let mut edits = Vec::new();
    for item in parsed.tree().items() {
        if let Some(info) = extract_item_doc_info(&item, &line_index) {
            if compact_tier_bc && info.tier == DocTier::TierBC && info.line_count > MAX_TIER_BC_DOC_LINES {
                let compact = build_tier_bc_docstring("REQ-0605", "fls_compacted", "Compacted Tier-B/C docstring.");
                edits.push((info.text_range, compact));
            }
        }
    }
    Ok(DocstringSynthesisPlan {
        target_path: file_path.to_path_buf(),
        abs_path: AbsPathBuf::try_from(file_path.to_path_buf()).ok(),
        rel_path: RelPathBuf::try_from(file_path.as_str()).ok(),
        file_id: None,
        edits,
        tier_bc_compacted: compact_tier_bc,
    })
}

/// Applies a `DocstringSynthesisPlan` atomically to source text using `TextEditBuilder`.
/// [REQ-0605] [FLS: fls_apply_synthesis]
pub fn apply_docstring_synthesis(
    source: &str,
    plan: &DocstringSynthesisPlan,
) -> Result<String, String> {
    if plan.edits.is_empty() {
        return Ok(source.to_string());
    }
    let mut builder = TextEditBuilder::default();
    for (range, replacement) in &plan.edits {
        builder.replace(*range, replacement.clone());
    }
    let edit: TextEdit = builder.finish();
    let mut output = source.to_string();
    edit.apply(&mut output);
    Ok(output)
}

/// Audits a source file for Check 36 completeness, tier constraints, and traceability anchors.
/// [DOC-TIER: A] [REQ-0605] [FLS: fls_audit_docstrings]
pub fn audit_source_file_docstrings(
    file_path: &Utf8Path,
    source: &str,
) -> Result<Vec<DocstringGateViolation>, String> {
    let normalized = source.replace("\r\n", "\n");
    let parsed = SourceFile::parse(&normalized);
    let line_index = LineIndex::new(&normalized);
    let mut violations = Vec::new();
    for item in parsed.tree().items() {
        let item_ident = item.syntax().first_token().map(|t| t.text().to_string()).unwrap_or_else(|| "unknown".into());
        let line_num = line_index.line_col(item.syntax().text_range().start()).line as usize + 1;
        match extract_item_doc_info(&item, &line_index) {
            None => {
                violations.push(DocstringGateViolation {
                    file_path: file_path.to_path_buf(),
                    line_number: line_num,
                    item_ident,
                    detail: "missing required safety docstring on production item".into(),
                });
            }
            Some(info) => {
                if info.tier == DocTier::TierBC && info.line_count > MAX_TIER_BC_DOC_LINES {
                    violations.push(DocstringGateViolation {
                        file_path: file_path.to_path_buf(),
                        line_number: line_num,
                        item_ident: item_ident.clone(),
                        detail: format!("Tier-B/C docstring line count {} exceeds max ceiling {}", info.line_count, MAX_TIER_BC_DOC_LINES),
                    });
                }
                if !info.has_req_anchor && !info.has_fls_anchor {
                    violations.push(DocstringGateViolation {
                        file_path: file_path.to_path_buf(),
                        line_number: line_num,
                        item_ident,
                        detail: "docstring lacks mandatory [REQ-*] or [FLS: fls_*] anchor".into(),
                    });
                }
            }
        }
    }
    Ok(violations)
}
```

### 6.2 Three-Layer Definition of Done Closure

Every implementation task derived from **BP-05** (**REQ-0605**) under **Contract A** and **Contract B** closes all three architectural Definition of Done layers across `< 380`-SLOC submodules with zero `N/A` exemptions:

| DoD Layer | Architectural Responsibility | Concrete Module & Interface Realization | Verification Gate |
| :--- | :--- | :--- | :--- |
| **(1) Domain State & Data Model** | Lossless CST docstring models (`rowan::TextRange`, `TextSize`), VFS/path descriptors (`ra_ap_vfs::FileId`, `ra_ap_paths::AbsPathBuf`, `camino::Utf8PathBuf`), synthesis plans, doc tiers (`[DOC-TIER: A]` vs Tier-B/C), and structured violation diagnostics | `DocTier`, `DocCommentInfo`, `DocstringSynthesisPlan`, `DocstringGateViolation` in `< 380`-SLOC `doc_edit.rs` and `docstring_tier_rules.rs` (`crates/deap-core/src/lib.rs:1-40`, `crates/verify-baseline/src/lib.rs:1-40`) | **AC-01**, **AC-02** (`cargo test -p deap-core`) |
| **(2) Logic & State Management** | Lossless CST traversal (`ra_ap_syntax::ast::{HasDocComments, HasAttrs}`), `ignore::WalkBuilder` + `ra_ap_vfs::Vfs` indexing, `ra_ap_line_index::LineIndex` + `ra_ap_text_edit::TextEditBuilder` non-overlapping surgical edit generation, `--compact-tier-bc` compaction, and `<= 4`-line Tier-B/C rule enforcement | `< 380`-SLOC submodules `deap_core::rust_cst::doc_edit`, `compile_sysml::docstring_cst` (`plan_docstring_surgical_edits`, `apply_docstring_synthesis`), and `verify_baseline::checks::{docstring_cst_gate, docstring_tier_rules}` (`crates/compile-sysml/src/lib.rs:1-40`) | **AC-02**, **AC-03**, **AC-04** (`cargo test -p compile-sysml -p verify-baseline`) |
| **(3) Presentation & Actuator Interface Binding** | CLI flag actuation (`compile-sysml --inject-safety-docstrings`, `--compact-tier-bc`) and Check 36 baseline gate report emission with explicit `[DOC-TIER: A]` entrypoint bindings | `crates/compile-sysml/src/main.rs:1-40` and `crates/verify-baseline/src/main.rs:1-40` binding to terminal outputs and non-zero exit codes | **AC-01**, **AC-04**, **AC-05** (`./target/release/verify-baseline . --no-domain`) |

### 6.3 Dogfooding Pipeline Ingestion & Verification Protocol

To verify that this blueprint is both structurally compliant with all 12 baseline quality gates and dogfood-ingestible by the Level 0 Markdown AST translator without mutating upstream clean landing zones (`schema/` and `.pipeline/schema.sysml`), execute:

```bash
./target/release/verify-baseline check-blueprint \
  docs/designs/blueprint-05-deterministic-safety-docstring-synthesizer.md \
  --dry-run

./target/release/verify-baseline . \
  --only docs/designs/blueprint-05-deterministic-safety-docstring-synthesizer.md \
  --no-domain
```

---

## 7. Source References

- **Workspace Manifest & Crate Ecosystem (`Contract A`)**: `Cargo.toml:1-40` (`ra_ap_syntax`, `ra_ap_text_edit`, `ra_ap_line_index`, `ra_ap_vfs`, `ra_ap_paths`, `rowan`, `camino`, `ignore`, and `[workspace.lints.clippy]` forbid gates)
- **Core Lossless CST Library Root (`deap-core`)**: `crates/deap-core/src/lib.rs:1-40` (Canonical `ra_ap_syntax::ast::{HasDocComments, HasAttrs}`, `rowan::TextRange`, and `ra_ap_vfs` extraction host)
- **Forward Docstring Synthesizer Library Root (`compile-sysml`)**: `crates/compile-sysml/src/lib.rs:1-40` (`-4,900+` SLOC legacy scanner deletion and `< 380`-SLOC surgical `ra_ap_text_edit` docstring injection host)
- **Forward Docstring Synthesizer CLI Entrypoint (`compile-sysml`)**: `crates/compile-sysml/src/main.rs:1-40` (`--inject-safety-docstrings` and `--compact-tier-bc` `[DOC-TIER: A]` actuator entrypoint)
- **Check 36 Docstring Completeness Verifier Library Root (`verify-baseline`)**: `crates/verify-baseline/src/lib.rs:1-40` (`-3,300+` SLOC legacy scanner deletion and `< 380`-SLOC lossless CST Check 36 audit host)
- **Check 36 & Blueprint Verifier CLI Entrypoint (`verify-baseline`)**: `crates/verify-baseline/src/main.rs:1-40` (Check 36 tier compliance and `check-blueprint` 12-gate `[DOC-TIER: A]` CLI actuator entrypoint)
- **Level 0 Markdown & SysML v2 Ingestion Entrypoints (`ingest-sysml`)**: `crates/ingest-sysml/src/lib.rs:1-40`, `crates/ingest-sysml/src/main.rs:1-40` (Bijective Markdown-to-SysML v2 AST translation and `.sysml` emission roundtrip)
- **Tracked Issues**: `#498, #500, #501, #508, #510, #742`

