| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #767 |
| **Title** | Use Case 39: Deterministic B Tree Map Item Inventory and Atomic Module Extraction with Check 37 Path Rewriting |
| **Type** | use-case |
| **Primary Actor** | Operator |
| **Scope** | AirGappedCargoLockGovernor |
| **Level** | User Goal |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Use Case 39: Deterministic B Tree Map Item Inventory and Atomic Module Extraction with Check 37 Path Rewriting

## UML Diagrams

```mermaid
graph TD
    subgraph "AirGappedCargoLockGovernor System Boundary"
        UC_Main(["Deterministic B Tree Map Item Inventory and Atomic Module..."])
        UC_Step01(["compile-sysml --list-rust-items <name>.rs followed by com..."])
        UC_Step02(["--list-rust-items emits a deterministic BTreeMap-ordered ..."])
    end
    Operator(("Operator")) --- UC_Main
    UC_Main -. "<<include>>" .-> UC_Step01
    UC_Main -. "<<include>>" .-> UC_Step02
```

## 1. Actors
- `Operator`

## 2. Preconditions
- An existing workspace module `<name>.rs` referenced in `docs/architecture/blueprints/*.md` and `README.md`, containing multiple top-level structs and functions with leading `///` docblocks and outer attributes inside an isolated temporary workspace fixture.

## 3. Trigger
- compile-sysml --list-rust-items <name>.rs followed by compile-sysml --extract-rust-module --from <name>/mod.rs --to <name>/<sub>.rs --items <ItemA,ItemB> is executed after transitioning <name>.rs to <name>/mod.rs.

## 4. Main Success Scenario
1. `Operator` invokes `compile-sysml --list-rust-items <name>.rs` on `ZeroTokenRustSpanCodemodEngine` (`runtime_llm_token_count = 0`) via `codemod_command_in`, delegating `list_rust_items` to `LosslessRustCstAnalyzer`.
2. `LosslessRustCstAnalyzer` emits a deterministic `BTreeMap`-ordered `RustItemInventory` on `item_inventory_out` recording each item's kind, identifier, `DocTier`, and composite UTF-8 `TextRange` byte/line span (including leading `///` docblocks and outer attributes).
3. `Operator` invokes `compile-sysml --extract-rust-module --from <name>/mod.rs --to <name>/<sub>.rs --items <ItemA,ItemB>` on `ZeroTokenRustSpanCodemodEngine`, which executes `extract_rust_module` inside `< 380` exec SLOC submodules (`deap_core_rust_cst_extract_module`).
4. `ZeroTokenRustSpanCodemodEngine` slices the non-overlapping composite `TextRange` spans in reverse byte order, synthesizes `<name>/<sub>.rs` with its `//!` module header, inserts `mod <sub>;` and `pub use` re-exports in `<name>/mod.rs`, and re-parses both files via `ra_ap_syntax::SourceFile::parse` to verify zero CST syntax errors.
5. `ZeroTokenRustSpanCodemodEngine` formats both modules via `rustfmt --edition 2021`, atomically updates `<name>.rs` citations to `<name>/mod.rs` across `docs/architecture/blueprints/*.md` and `README.md` (`rewrite_check37_citations = true`), and emits `atomic_patch_out` with exit code 0.

## 5. Alternate and Exception Flows
- 3a. Requested item identifier in `--items <ItemA,ItemB>` does not exist in `<name>/mod.rs` (`Diagnostic: DEAP-E0801`):
  - `ZeroTokenRustSpanCodemodEngine` aborts extraction before any filesystem mutation, emits single-line JSON diagnostic `DEAP-E0801` on `codemod_error_out`, and exits fail-closed with code 1.
- 4a. Extracted `<name>/mod.rs` or `<name>/<sub>.rs` buffer fails post-splice `ra_ap_syntax::SourceFile::parse` verification (`Diagnostic: DEAP-E0803`):
  - `ZeroTokenRustSpanCodemodEngine` rolls back the in-memory splice transaction without writing to disk, emits `DEAP-E0803` on `codemod_error_out`, and terminates fail-closed with exit code 1.
- 5a. Blueprint or `README.md` citation path rewrite fails Check 37 existence verification:
  - `ZeroTokenRustSpanCodemodEngine` restores the pre-extraction workspace state, preserves prior verified state, and notifies `Operator`.

## 6. Postconditions
- `--list-rust-items` emits a deterministic `BTreeMap`-ordered inventory, `--extract-rust-module` atomically extracts `<name>/<sub>.rs` and updates `<name>/mod.rs` with zero syntax errors, and Check 37 citations in `docs/architecture/blueprints/*.md` and `README.md` resolve cleanly.

## 8. Realization Matrix
| Specification Item | Type | Link / Reference |
| :--- | :--- | :--- |
| **TBD** | User Story | [User Story 54: list rust items](../user-stories/US-54-list_rust_items.md) |
| **TBD** | Feature | [Feature 36: [BP 08 Lossless Rust CST Codemod Engine and Unified rustc Compatible JSON Diagnostics] AirGappedCargoLockGovernor](../features/FEAT-36-AirGappedCargoLockGovernor.md) |

## Source References
Use case operational flows and lifecycle activity references:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Operational use case realization and traceability links derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148.
