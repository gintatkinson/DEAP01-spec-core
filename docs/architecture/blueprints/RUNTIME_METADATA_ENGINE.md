---
identifier: DEAP-BLUEPRINT-METADATA-001
title: "DEAP Runtime Metadata Engine & Dynamic AST Projection Architecture Blueprint"
version: "1.0.0"
status: "APPROVED / PRODUCTION-GRADE"
created: "2026-09-01"
last_updated: "2026-10-19"
classification: "UPSTREAM_SPEC_CORE_COMPILER"
target_regulatory_frameworks:
  - AssuranceLevel::Tier1 (Zero Hardcoded Domain Logic)
  - AssuranceLevel::Tier2 (Dynamic Model-Driven Architecture)
  - High Assurance Model-Based Systems Engineering
---

# DEAP Runtime Metadata Engine & Dynamic AST Projection Architecture Blueprint

| Attribute | Specification Detail |
| :--- | :--- |
| **Document Identifier** | `DEAP-BLUEPRINT-METADATA-001` |
| **Title** | Runtime Metadata Engine & Dynamic AST Projection Architecture Blueprint |
| **Version** | `1.0.0` |
| **Date** | `2026-10-19` |
| **Status** | `APPROVED / PRODUCTION-GRADE` |
| **Classification** | `UPSTREAM_SPEC_CORE_COMPILER` |
| **Assurance Level** | `AssuranceLevel::Tier1` (Zero Hardcoded Domain Logic & Dynamic Schema Discovery) |
| **Target Technology Profile** | Modular Rust Cargo Workspace (`deap-core`, `verify-baseline`, `compile-sysml`, `ingest-sysml`, `assemble-conops`, `reconcile-backlog`) |

---

## 1. Executive Summary & Pure Schema-Driven Invariant

The **DEAP Runtime Metadata Engine** eliminates hardcoded domain classes, static layout assumptions, and compile-time data structures across both compiler pipelines and client presentation tiers. In strict adherence to the **Pure Schema-Driven Compiler Invariant**:
- The application engine and UI adapters possess ZERO hardcoded domain concepts at compile time.
- All entity types, structural fields, relations, visual symbols, validation invariants, and section hierarchies are discovered dynamically at runtime from the connected data sources and SysML v2 AST models (`crates/deap-core/src/sysml_ast/mod.rs`).
- All system models are expressed via purely symbolic computer science primitives: `PartDef_Alpha`, `Subsystem_01`, `Port_A`, `Action_01`, `Constraint_Invariant`.
- Static layout files with hardcoded field enumerations are prohibited; schemas are ingested dynamically from SysML v2 AST, Protocol Buffers reflection (`FileDescriptorSet`), and relational schema dictionaries via `UnifiedSchemaModelExtractor` (`crates/deap-core/src/sysml_ast/mod.rs`).

```mermaid
flowchart TD
    subgraph ModelSource ["Authoritative Model Sources"]
        direction TB
        SysMLAST["SysML v2 AST<br/>(schema/ -> .pipeline/)"]
        ProtoReflect["Protocol Buffers<br/>(FileDescriptorSet / gRPC)"]
        RelationalDict["Relational Schema<br/>(SQLite Dictionary)"]
    end

    subgraph RustWorkspace ["Rust Cargo Workspace Core"]
        direction TB
        IngestCrate["ingest-sysml<br/>(Level 0 Ingestion CLI)"]
        CoreCrate["deap-core<br/>(SysML AST & Metadata Core)"]
        CompileCrate["compile-sysml<br/>(Parser, Sync & STPA Engine)"]
        VerifyCrate["verify-baseline<br/>(Parity & Baseline Gates)"]
    end

    subgraph ClientAdapters ["Dynamic Presentation Adapters"]
        direction TB
        TreeWidget["HierarchyTree<br/>(Dynamic Part Relations)"]
        GridWidget["PropertyGrid<br/>(Dynamic Attribute Fields)"]
        TableWidget["DensityTable<br/>(Schema-Driven Columns)"]
        MapWidget["TopologyMap<br/>(Port & Connection Graph)"]
    end

    ModelSource --> IngestCrate
    IngestCrate --> CoreCrate
    CoreCrate --> CompileCrate
    CompileCrate --> VerifyCrate
    CoreCrate --> ClientAdapters
```

---

## 2. Modular Cargo Workspace Architecture

The Runtime Metadata Engine is organized within the standard DEAP pure Rust Cargo workspace (`Cargo.toml`) plus isolated Stage-2 MIR lint plugins (`./lints/deap-dylint-mir/`):

```
DEAP01-spec-core/
├── Cargo.toml                      # Workspace manifest (resolver = "2")
├── lints/
│   └── deap-dylint-mir/            # Isolated Stage-2 nightly MIR linter (#![feature(rustc_private)], rustc_middle::mir)
├── crates/
│   ├── deap-core/                  # Lossless CST/AST core (deap_core::sysml_cst, deap_core::rust_cst, deap_core::dart_cst), rules, diagnostics, markdown
│   ├── verify-baseline/            # 41-gate architectural, structural, CST, and 5-Stage formal verification runner
│   ├── compile-sysml/              # SysML v2.0 + KerML 1.0 logos+rowan+salsa compiler, SHA-256 digest, STPA, CST sync & syn/quote/prettyplease codegen
│   ├── ingest-sysml/               # Ground 0.0 multi-format Level 0 schema CST ingestion CLI
│   ├── assemble-conops/            # Deterministic Concept of Operations & Mission Intent assembler
│   └── reconcile-backlog/          # Bidirectional GitHub/GitLab Agile backlog & WBS reconciler
```

### 2.1 Crate Distribution & Official Compiler-Crate Stack (Contract B)

All workspace crates enforce a strict 3-Layer Definition of Done targeting `< 380`-SLOC submodules and explicit legacy hand-rolled scanner deletion (zero hand-rolled character/line scanners):

| Crate | Primary Modules | Responsibility in Metadata, CST & AST Architecture |
| :--- | :--- | :--- |
| **`crates/deap-core`** | `src/sysml_ast/mod.rs`, `deap_core::sysml_cst`, `deap_core::rust_cst`, `deap_core::dart_cst`, `src/rules.rs`, `src/diagnostics.rs`, `src/markdown.rs`, `src/workspace.rs` | Defines canonical 100% OMG SysML v2.0 + KerML 1.0 AST/HIR models (`src/sysml_ast/mod.rs`) and lossless CSTs (`deap_core::sysml_cst` via `logos` + `rowan` + `salsa` + `ariadne`), lossless Rust CST inspection & codemods (`deap_core::rust_cst` via `ra_ap_syntax` `ast::{MatchExpr, MatchArmList, CallExpr, MethodCallExpr, TryExpr, Fn, HasDocComments, HasAttrs}`, `ra_ap_vfs`, `ra_ap_paths`, `ra_ap_line_index`, `ra_ap_text_edit`, `rowan`, `camino`, `ignore`), lossless Dart CST (`deap_core::dart_cst` via `logos` + `rowan`), lossless CommonMark AST (`pulldown-cmark` + `pulldown-cmark-to-cmark`), Cargo workspace/manifest inspection (`cargo_metadata`, `cargo-lock`, `cargo-util-schemas`, `cargo-platform`, `toml_edit`), and multi-format schema discovery (`UnifiedSchemaModelExtractor`, `SupportedSchemaExtension`, `SupportedExtensionsCatalog`). |
| **`crates/compile-sysml`** | `src/lib.rs`, `src/parser/grammar/mod.rs`, `src/semantic/digest.rs`, `src/stpa/`, `src/sync/`, `src/modular_codegen/mod.rs`, `src/flutter_codegen.rs` | Parses 100% normative OMG SysML v2.0 + KerML 1.0 models via `logos` + `rowan` + `salsa` (`deap_core::sysml_cst`, `SysmlParser`, `parse_sysml_tree`), computes atomic SHA-256 digests (`SchemaDigest`, `FileDigestEntry`, `generate_digest`), synthesizes STPA/FMECA matrices, executes formatting-preserving CST-splice bidirectional sync, and emits deterministic Fixed-Point Rust/Flutter code (`syn` + `proc-macro2` + `quote` + `prettyplease`), `flutter_rust_bridge` Ring-2 guards, and Stage-4 `#[kani::proof]` harnesses (`crates/compile-sysml/src/modular_codegen/mod.rs`, `crates/compile-sysml/src/flutter_codegen.rs`). |
| **`crates/verify-baseline`** | `src/runner.rs`, `src/checks/parity_gates.rs`, `src/checks/domain_cleanliness_gate.rs`, `src/checks/codebase.rs` | Enforces 41 deterministic verification gates across AST/CST parity, schema cleanliness, traceability, `pulldown-cmark` Markdown/Mermaid/KaTeX rendering, defect dossiers, adversarial CST codebase checks 38-41 (`crates/verify-baseline/src/checks/codebase.rs`), and the 5-Stage Compiler & Ferrocene Verification Gatekeeper (Stage 1 `rustc`/`cargo clippy --message-format=json` -> Stage 2 `./lints/deap-dylint-mir/` -> Stage 3 `cargo miri` + `flutter_rust_bridge` -> Stage 4 `cargo kani` `#[kani::proof]` -> Stage 5 Ferrocene `criticalup.toml` + `[FLS: fls_*]` + `symbol-report`/`blanket` MC/DC + 3-Ring DAL-A..E partitioning). |
| **`crates/ingest-sysml`** | `src/lib.rs`, `src/main.rs`, `src/translators/mod.rs`, `ingest_sysml::cst::{idl_cst, yang_cst}` | Discovers and translates all `SupportedSchemaExtension` Level 0 specifications (`.sysml`, `.yaml`, `.yml`, `.json`, `.proto`, `.idl`, `.arxml`, `.md`, `.markdown`, `.yang`, plus polyglot `.rs` and `.dart` CSTs) into `.pipeline/schema.sysml` and `.pipeline/schema-digest.json` via `deap_core::sysml_cst`, `ingest_sysml::cst::{idl_cst, yang_cst}`, `pulldown-cmark` + `pulldown-cmark-to-cmark`, and typed `serde_json` OpenAPI/JSON-Schema CST/AST translators with zero hand-rolled scanners. |
| **`crates/assemble-conops`** | `src/lib.rs`, `src/main.rs` | Assembles hierarchical Concept of Operations and Tactical Mission Intent specification units from pure schema contracts using `pulldown-cmark` + `pulldown-cmark-to-cmark`. |
| **`crates/reconcile-backlog`** | `src/lib.rs`, `src/main.rs` | Synchronizes local Epics, Features, User Stories, Use Cases, and WBS registers with the remote issue tracker using lossless `pulldown-cmark` AST and `--message-format=json` diagnostics. |

---

## 3. Core AST Models & Runtime Metadata Projection Contracts

### 3.1 Contract A: 100% Normative OMG SysML v2.0 + KerML 1.0 Textual Grammar Coverage (`crates/deap-core/src/sysml_ast/mod.rs` & `deap_core::sysml_cst`)

The lossless `logos` + `rowan` + `salsa` + `ariadne` frontend (`deap_core::sysml_cst`) and lowered HIR metamodel (`crates/deap-core/src/sysml_ast/mod.rs`) enforce **100% normative OMG SysML v2.0 + KerML 1.0 textual grammar coverage** with full Definition & Usage symmetry across all normative clauses and **zero 16-construct artificial caps**:

1. **Namespaces & Metadata**: `package`, `library package`, `standard library package`, `import` (`all`, `::*`, `::**`, filter `[...]`), `alias`, `doc`, `comment`, `rep`, `dependency`, `metadata def` / `metadata` / `@Metadata` / `#user_keyword` (`#hazard`, `#risk`, `#capability`, `#rt_loop`, `#dal`, `#fls` via standard OMG SysML v2 `metadata def` keywords while preserving backward-compatible parsing of legacy `hazard def` / `risk def` / `capability def`).
2. **Structural & Data Types**: `attribute def`/`attribute`, `enum def`/`enum`, `item def`/`item`, `part def`/`part`, `port def`/`port` (`in`, `out`, `inout`, `~` conjugation), `connection def`/`connection` (`connect ... to ...`), `interface def`/`interface`, `flow def`/`flow` (`of ... from ... to ...`), `allocation def`/`allocation` (`allocate ... to ...`), `occurrence def`/`occurrence` (`individual`, `snapshot`, `timeslice`).
3. **Behavioral & State**: `action def`/`action` (`perform`, `accept`, `send`, `assign`, `if`/`else`, `while`/`loop`, `for`, `fork`, `join`, `merge`, `decide`, `first ... then ...` succession), `state def`/`state` (`exhibit`, `parallel`, `entry`, `do`, `exit`, `transition first ... accept ... if ... do ... then ...`).
4. **Calculations, Constraints & Requirements**: `calc def`/`calc` (`return`), `constraint def`/`constraint` (`assert`, `assume`, `require`), `requirement def`/`requirement` (`subject`, `actor`, `stakeholder`, `framed concern`, `satisfy ... by ...`), `concern def`/`concern`.
5. **Use Cases, Analysis, Verification & Views**: `use case def`/`use case` (`include use case`), `analysis def`/`analysis` (`objective`), `verification def`/`verification` (`verify requirement`), `test case def`/`test case`, `interaction def`/`interaction`, `view def`/`view`, `viewpoint def`/`viewpoint`, `rendering def`/`rendering` (`expose`, `filter`, `render`), `variation`/`variant`.
6. **KerML 1.0 Specialization, Multiplicity & Expressions**: `:` (`defined by`), `:>` (`subsets`), `:>>` (`redefines`), `::>` (`references`), `~` (`conjugates`), `ref`, `abstract`, `readonly`, `derived`, `end`, `default`, `:=`, `[lo..hi]`, `[*]`, `{ordered, nonunique}`, full KerML logical/conditional/relational/arithmetic/collection expressions (`implies`, `xor`, `??`, `->`, `collect`, `select`, `forAll`, `exists`), and `ISQ`/`SI`/`ScalarValues` standard libraries.

The authoritative structural and behavioral metadata in `crates/deap-core/src/sysml_ast/mod.rs` is represented by strongly-typed, `serde`-serializable AST definitions:

```rust
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Represents an attribute definition or instance in SysML v2 (`crates/deap-core/src/sysml_ast/mod.rs`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AttributeDef {
    pub name: String,
    #[serde(default)]
    pub type_name: String,
    pub default_value: Option<String>,
    pub doc: Option<String>,
}

/// Represents a port definition or interface point in SysML v2 (`crates/deap-core/src/sysml_ast/mod.rs`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PortDef {
    pub name: String,
    pub type_name: String,
    pub direction: String,
    #[serde(default)]
    pub is_conjugated: bool,
    pub doc: Option<String>,
    pub port_category: String,
    #[serde(default)]
    pub protocol_family: String,
    #[serde(default)]
    pub electrical_attributes: BTreeMap<String, String>,
    #[serde(default)]
    pub flows: Vec<FlowDef>,
}

/// Represents a structural part definition or block in SysML v2 (`crates/deap-core/src/sysml_ast/mod.rs`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PartDef {
    pub name: String,
    pub doc: Option<String>,
    pub is_def: bool,
    pub type_name: Option<String>,
    pub attributes: Vec<AttributeDef>,
    pub ports: Vec<PortDef>,
    pub actions: Vec<ActionDef>,
    pub constraints: Vec<ConstraintDef>,
    pub requirements: Vec<RequirementDef>,
    pub satisfied_requirements: Vec<String>,
    pub parts: Vec<PartDef>,
    pub operations: Vec<OperationDef>,
    pub capabilities: Vec<CapabilityDef>,
    pub interactions: Vec<InteractionDef>,
    pub test_cases: Vec<TestCaseDef>,
    pub states: Vec<StateDef>,
    pub use_cases: Vec<UseCaseDef>,
    pub item_defs: Vec<ItemDef>,
    pub hazards: Vec<HazardDef>,
    pub risks: Vec<RiskDef>,
    pub connections: Vec<ConnectionDef>,
    pub calcs: Vec<CalcDef>,
}
```

### 3.2 Dynamic UI Projection Descriptors & DataSource Contract

Downstream presentation shells project `PackageDef` and `PartDef` AST nodes from `crates/deap-core/src/sysml_ast/mod.rs` into runtime `TypeDescriptor` and `FieldDescriptor` views without hardcoded domain classes:

```rust
use serde::{Deserialize, Serialize};

/// Describes one entity type projected dynamically from `PartDef` (`crates/deap-core/src/sysml_ast/mod.rs`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDescriptor {
    pub type_name: String,                      // e.g. "Subsystem_01", "PartDef_Alpha"
    pub display_name: String,                   // Human-readable symbolic label
    pub icon_symbol: String,                    // Symbolic visual token identifier
    pub fields: Vec<FieldDescriptor>,           // Projected from `PartDef::attributes`
    pub satisfied_requirements: Vec<String>,    // Projected from `PartDef::satisfied_requirements`
    pub child_relations: Vec<TypeRelationDescriptor>, // Projected from `PartDef::parts` and `ports`
    pub parent_relations: Vec<TypeRelationDescriptor>,// Upstream package/part containment
}

/// Describes one attribute or property projected from `AttributeDef` or `FlowDef`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDescriptor {
    pub key: String,                            // XPath or dot-notation identifier
    pub label: String,                          // Presentation label
    pub field_type: FieldType,                  // Typed data primitive mapped from `type_name`
    pub section_label: Option<String>,          // Grouping category (e.g. "Primary", "Control")
    pub section_order: i32,                     // Display sequence within group
    pub required: bool,                         // Mandatory validation constraint
    pub min_value: Option<f64>,                 // Numeric lower bound from `ConstraintDef`
    pub max_value: Option<f64>,                 // Numeric upper bound from `ConstraintDef`
    pub regex_pattern: Option<String>,          // Textual validation pattern
    pub enum_options: Option<Vec<String>>,      // Enumeration allowed symbols from `StateDef`
    pub default_value: Option<serde_json::Value>, // Initial state fallback from `AttributeDef::default_value`
    pub input_formatters: Vec<String>,          // Formatting filters (e.g. "uppercase")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldType {
    String,
    Integer,
    Float,
    Boolean,
    Enumeration,
    DateTime,
    ComplexAstNode,
}

/// Describes a directional relationship between two entity types (`PartDef::parts` / `ConnectionDef`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeRelationDescriptor {
    pub relation_name: String,                  // e.g. "contains", "connects_to", "realises"
    pub target_type_name: String,               // Target TypeDescriptor identifier
    pub display_label: String,                  // Navigation label
}
```

### 3.3 Pluggable DataSource Trait Contract

```rust
pub trait DataSource: Send + Sync {
    /// Returns the provider identifier (e.g. "sqlite", "sysml_ast", "grpc").
    fn provider_name(&self) -> &str;

    /// Discovers all entity types exposed by the data source (`PackageDef::get_all_parts`).
    fn discover_types(&self) -> Result<Vec<TypeDescriptor>, DataSourceError>;

    /// Fetches the TypeDescriptor for a single entity type by `PartDef::name`.
    fn type_for(&self, type_name: &str) -> Result<TypeDescriptor, DataSourceError>;

    /// Retrieves active properties for an entity instance.
    fn fetch_properties(
        &self,
        type_name: &str,
        instance_id: &str,
    ) -> Result<serde_json::Value, DataSourceError>;

    /// Persists updated properties for an entity instance.
    fn save_properties(
        &self,
        type_name: &str,
        instance_id: &str,
        data: serde_json::Value,
    ) -> Result<(), DataSourceError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataSourceError {
    TypeNotFound(String),
    ValidationFailed(String, String),
    Io(String),
}
```

---

## 4. Strongly-Typed AST Slicing Mechanics ($\le 4,096$ Tokens)

To enable context-isolated subagent execution and zero-bloat client data synchronization, the Runtime Metadata Engine projects AST subtrees extracted via `PackageDef::get_all_parts` and `PartDef::find_part_mut` (`crates/deap-core/src/sysml_ast/mod.rs`) through an **AST Slicing Window**:

### 4.1 Token Budget Invariant

For any target subsystem or component model $v_0$:

$$
\mathcal{V}_{\text{slice}} = \{v_0\} \cup \mathcal{N}_{\text{child}}(v_0) \cup \mathcal{N}_{\text{port}}(v_0) \cup \mathcal{N}_{\text{conn}}(v_0)
$$

The serialized token footprint satisfies:

$$
|\text{TokenCount}(\text{Serialize}(\mathcal{S}(v_0)))| \le 4096
$$

Any peripheral nodes exceeding the 4,096 token boundary are stubbed as opaque interface signatures (`ExternalInterfaceRef { id, port_signature }`), preventing context window overflow during LLM code synthesis.

---

## 5. Multi-Source Pluggable Data Source Engine

The Metadata Engine supports three canonical data source implementations operating without compile-time domain code, backed by `UnifiedSchemaModelExtractor` (`crates/deap-core/src/sysml_ast/mod.rs`) across all `SupportedSchemaExtension` formats (`.sysml`, `.yaml`, `.yml`, `.json`, `.proto`, `.idl`, `.arxml`, `.md`, `.markdown`, `.yang`, plus polyglot source lowering for `.rs` and `.dart`) using the official compiler-crate CST/AST stack with **zero hand-rolled scanners**:
- **`.sysml`**: `logos` + `rowan` + `salsa` + `ariadne` (`deap_core::sysml_cst`) covering 100% of the normative OMG SysML v2.0 + KerML 1.0 textual grammar (Contract A);
- **`.idl` & `.yang`**: lossless `logos` + `rowan` CST frontends (`ingest_sysml::cst::{idl_cst, yang_cst}`);
- **`.json`, `.yaml`, `.yml` (OpenAPI 3.0/3.1 & JSON-Schema)**: typed `serde_json` AST deserialization;
- **`.md` & `.markdown`**: lossless CommonMark AST via `pulldown-cmark` + `pulldown-cmark-to-cmark`;
- **`.rs` & `.dart`**: lossless Rust CST (`deap_core::rust_cst` via `ra_ap_syntax`) and lossless Dart CST (`deap_core::dart_cst` via `logos` + `rowan`).

### 5.1 Relational Schema Dictionary (SQLite Backend)

```sql
CREATE TABLE type_definitions (
  type_name       TEXT PRIMARY KEY,
  display_name    TEXT NOT NULL,
  icon_symbol     TEXT NOT NULL
);

CREATE TABLE type_attributes (
  id              INTEGER PRIMARY KEY,
  type_name       TEXT NOT NULL REFERENCES type_definitions(type_name),
  key             TEXT NOT NULL,
  label           TEXT NOT NULL,
  field_type      TEXT NOT NULL,
  section_label   TEXT,
  section_order   INTEGER DEFAULT 0,
  is_required     INTEGER DEFAULT 0,
  min_value       REAL,
  max_value       REAL,
  regex_pattern   TEXT,
  enum_options    TEXT,
  default_value   TEXT,
  input_formatters TEXT
);

CREATE TABLE type_relations (
  id              INTEGER PRIMARY KEY,
  parent_type     TEXT NOT NULL REFERENCES type_definitions(type_name),
  relation_name   TEXT NOT NULL,
  child_type      TEXT NOT NULL REFERENCES type_definitions(type_name),
  display_label   TEXT NOT NULL
);
```

### 5.2 Protocol Buffers Reflection Adapter (gRPC Backend)
- Ingests standard `.proto` descriptor sets (`FileDescriptorSet`) and discovers service/message definitions via `UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".proto", ...)`.
- Custom proto annotations map field constraints (`min_value`, `max_value`, `regex_pattern`) directly into `FieldDescriptor` properties without generating static target code.

### 5.3 Direct SysML v2 AST Adapter (`crates/deap-core` & `crates/compile-sysml`)
- Ingests compiled SysML v2 AST from `.pipeline/schema.sysml` via `compile_sysml::parse_sysml_tree` and verifies SHA-256 integrity via `.pipeline/schema-digest.json` (`compile_sysml::semantic::digest::SchemaDigest`).
- Extracts `PartDef`, `PortDef`, `ActionDef`, and `ConstraintDef` nodes directly into the runtime `TypeDescriptor` pool.

### 5.4 Option 1C Typed InstanceRecord Views, `flutter_rust_bridge` & Zero-Trust C-ABI Bridge (`crates/compile-sysml/src/flutter_codegen.rs`)
To resolve the tension between dynamic SQLite metadata discovery and compile-time type safety:
- **Option 1C App Glue Architecture**: `crates/compile-sysml/src/flutter_codegen.rs` and `crates/compile-sysml/src/modular_codegen/mod.rs` (`syn` + `proc-macro2` + `quote` + `prettyplease`) deterministically synthesize `app_flutter/lib/data/seeds/generated_domain_seed.g.dart` (populating all 5 canonical SQLite tables `type_definitions`, `type_attributes`, `type_relations`, `properties`, and `instances` from `.pipeline/schema.sysml` and `topology_data.json`) and typed `InstanceRecord` views in `app_flutter/lib/models/generated_schema_models.g.dart`, realizing `DEAP-BLUEPRINT-APP-GLUE-001` (`docs/designs/option-1c-rust-app-glue-ui-contracts-and-anti-fabrication-blueprint.md`, GitHub Feature Issue `#489`).
- **Zero-Trust Deterministic Rust C-ABI & `flutter_rust_bridge` Ring-2 Bridge**: `compile-sysml` synthesizes `app_flutter/lib/data/generated/solver_bridge.g.dart` and `flutter_rust_bridge` Ring-2 FFI guards providing zero-trust `DeapByteBuffer` FFI bindings to native Ring-1 `#![no_std]` Rust solvers across 3-Ring DAL-A..E partitions, realizing `DEAP-BLUEPRINT-CABI-001` (`docs/designs/zero-trust-deterministic-rust-c-abi-blueprint.md`, GitHub Feature Issue `#483`).
- **Verification Gates**: Validated automatically in `verify-baseline` via `crates/verify-baseline/src/checks/codebase.rs` (Checks 38-41) and the 5-Stage Compiler & Ferrocene Gatekeeper (Stage 1 `rustc`/`cargo clippy --message-format=json` -> Stage 2 isolated `./lints/deap-dylint-mir/` -> Stage 3 `cargo miri` + `flutter_rust_bridge` -> Stage 4 `cargo kani` `#[kani::proof]` -> Stage 5 Ferrocene `criticalup.toml` + `[FLS: fls_*]` + `symbol-report`/`blanket` MC/DC) with cross-platform runtime `PATH` discovery (`std::env::consts::{DLL_PREFIX, DLL_SUFFIX, EXE_SUFFIX}`, `camino::Utf8Path`) across macOS, Linux, and Windows.

---

## 6. Reactive Telemetry & State Binding Architecture

To prevent high-frequency telemetry event storms from freezing user interfaces or saturating execution pipelines:

1. **Thread / Isolate Decoupling**: Raw binary streams (gRPC/Protobuf) are unpacked entirely inside background workers (Web Workers in React, Dart Isolates in Flutter, background threads in Rust).
2. **100ms Micro-Batching Buffer**: Updates are accumulated over a fixed **100ms** sliding window, deduplicated by key (retaining the latest value), and dispatched as a flat JSON delta.
3. **Event-Echo Elimination**: Programmatic property updates applied by incoming telemetry streams NEVER trigger UI output callbacks (`onChange`, `onSave`). Event emission is strictly locked to direct operator input.

---

## 7. Architectural Gap Remediation Matrix

| Legacy Anti-Pattern | Runtime Metadata Engine Remediation |
| :--- | :--- |
| Hardcoded Dart / TypeScript domain classes (`SubsystemA`, `PartModelB`) | Generic presentation adapters rendering `TypeDescriptor` collections projected from `PartDef` (`crates/deap-core/src/sysml_ast/mod.rs`). |
| Hardcoded `switch(node.id)` statements for icon mapping | Dynamic `TypeDescriptor.icon_symbol` resolution. |
| Hardcoded UI layout groupings in views | Declarative `FieldDescriptor.section_label` and `section_order`. |
| Hardcoded validation logic per field key | Declarative boundaries (`min_value`, `max_value`, `regex_pattern`, `required`) derived from `ConstraintDef`. |
| In-memory ephemeral configuration | Zero-ephemeral storage: all metadata definitions committed in `.pipeline/schema.sysml`, `.pipeline/schema-digest.json`, and `schema/`. |
| Hardcoded enum string maps | Dynamic `FieldDescriptor.enum_options` list derived from `StateDef`. |

---

## 8. Verification & Conformance Strategy

- **Semantic Acceptance Testing**: Verified via `./target/release/verify-baseline . --no-domain` with exit code 0 (`crates/verify-baseline/src/runner.rs`).
- **Native Rust Unit & Mutation Testing**: Verified via `cargo test --workspace` executing native Rust `#[test]` suites across `crates/deap-core`, `crates/compile-sysml`, `crates/ingest-sysml`, `crates/assemble-conops`, `crates/reconcile-backlog`, and `crates/verify-baseline`.
- **Zero Em Dash Invariant**: The document contains 0 Unicode em dashes (`\u2014`), using ASCII `--` or `-` exclusively.
- **Zero Prohibited Terms**: Contains 0 kinetic, munition, or regulatory agency citations.

---

## 9. Governance & Document Sign-Off

| Role | Domain Authority | Status | Verification Signature |
| :--- | :--- | :--- | :--- |
| **Systems Architecture Lead** | Dynamic Model Metamodel | APPROVED | `Arch. System, PE` |
| **Compiler Engine Lead** | AST Projection & Rust Workspace | APPROVED | `Lead. Compiler, SE` |
| **Forensic Quality Auditor** | Purity & Schema Invariants | APPROVED | `Audit. Teamwork, QA` |
