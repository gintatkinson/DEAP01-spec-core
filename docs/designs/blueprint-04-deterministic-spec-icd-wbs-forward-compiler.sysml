doc /* Translated from Level 0 OEM Markdown specification */
package Deterministic_Specification_ICD_and_WBS_Forward_Compiler_Blueprint {
    attribute Blueprint_ID : String = "BP-04";
    attribute Requirement_ID : String = "REQ-0604";
    attribute Title : String = "Deterministic Specification, ICD, and WBS Forward Compiler Blueprint";
    attribute Status : String = "Approved";
    attribute Target_Crates : String = "crates/compile-sysml, crates/reconcile-backlog, crates/ingest-sysml, crates/assemble-conops, crates/verify-baseline";
    attribute Tracked_Issues : String = "#497, #506, #507";
    attribute Generation_Mode : String = "subagent";
    attribute Specification_Source : String = "crates/compile-sysml/src/sync/forward.rs:1585-2078 / #507";
    doc /* UUIDv5:  | Complexity: Class P | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100 */
    requirement def REQ_0604_Deterministic_Specification_ICD_and_WBS_Forward_Compiler_Blueprint {
        id = "REQ-0604";
        text = "The Digital Engineering Agent Platform (DEAP) compiler pipeline SHALL deterministically project all Agile functional specifications (docs/epics/, docs/features/, docs/user-stories/, docs/use-cases/), Logical Interface Control Documents (docs/interfaces/ICD_01_SYSTEM_INTERFACE_MATRIX.md,";
        attribute uuidv5 : String = "";
        attribute complexity_class : String = "Class P";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100";
        attribute ac_01_deterministic_multi_story_use_case_and_conops_parent_epic_projection : String = "Diagnostic: E0100).";
        attribute ac_02_deterministic_logical_icd_matrix_and_signal_dictionary_synthesis : String = "Diagnostic: E0200).";
        attribute ac_03_deterministic_mil_std_881e_wbs_and_enterprise_export_synthesis : String = "Diagnostic: E0300).";
        attribute ac_04_fail_closed_stateful_html_comment_frontmatter_guard_and_parent_inclusive_checklist_reconciliation : String = "Diagnostic: E0300).";
        attribute ac_05_fail_closed_subpackage_scoped_reverse_sync_and_dogfood_blueprint_ingestion_parity : String = "Diagnostic: E0400).";
        doc /* \begin{aligned} \text{Invariant}_{\text{FWD}} &\iff (N_{\text{subagent}} = 0) \land (\text{Coverage}_{\text{AST}} = 100) \land (V_{\text{E1..E5}} = 0) \land (\Delta_{\text{sync}} = 0) \end{aligned} */
        constraint def Invariant_REQ_0604_SC_01;
        doc /* \begin{aligned} T_{\text{forward}}(P, C, I, U) &= O(P \log P + C^{2} + I \cdot \log P + U \cdot I) \\ S_{\text{forward}}(P, C, I, U) &= O(P + C^{2} + I + U) \\ \Delta_{\text{roundtrip}}(\mathcal{M}) &= H(\text{ReverseSync}(\text{ForwardSync}(\mathcal{M}))) \oplus H(\mathcal{M}) = 0 \end{aligned} */
        constraint def Invariant_REQ_0604_SC_02;
        require Invariant_REQ_0604_SC_01;
        require Invariant_REQ_0604_SC_02;
        verify by AC_01_Deterministic_Multi_Story_Use_Case_and_ConOps_Parent_Epic_Projection;
        verify by AC_02_Deterministic_Logical_ICD_Matrix_and_Signal_Dictionary_Synthesis;
        verify by AC_03_Deterministic_MIL_STD_881E_WBS_and_Enterprise_Export_Synthesis;
        verify by AC_04_Fail_Closed_Stateful_HTML_Comment_Frontmatter_Guard_and_Parent_Inclusive_Checklist_Reconciliation;
        verify by AC_05_Fail_Closed_Subpackage_Scoped_Reverse_Sync_and_Dogfood_Blueprint_Ingestion_Parity;
        satisfy by AgileSpecForwardProjector;
    }
    doc /* Maximum deterministic execution bound for `--forward-sync` across all package subtrees. [unit: ms] */
    part def AgileSpecForwardProjector {
        attribute maxForwardLatencyMs : Integer = 25;
        doc /* Enforces symmetric `in` and `out` `ActionDef` parameter emission across Mermaid and Markdown bullets. [unit: flag] */
        attribute preserveTypedActionParams : Boolean = true;
        doc /* Receives ordered `PackageDef`, `PartDef`, `InteractionDef`, and `UseCaseDef` AST slices. [protocol: MemoryBus] */
        port inout packageAstInPort : String;
        doc /* Emits Check 13B E1-E5 compliant Epics, Features, User Stories, and Use Cases. [protocol: AtomicFsWrite] */
        port inout specMarkdownOutPort : String;
    }
    doc /* Tracks multi-line `in_html_comment` state to prevent commented example tables from spoofing frontmatter issue IDs. [unit: flag] */
    part def BidirectionalBacklogReconciler {
        attribute trackHtmlCommentState : Boolean = true;
        doc /* Computes parent-inclusive Epic count so child-only slices never trigger single-epic wildcard matching. [unit: flag] */
        attribute parentInclusiveEpics : Boolean = true;
        doc /* Reads and reconciles CommonMark metadata tables and bidirectional Epic/Feature checklists under `docs/`. [protocol: AtomicFsWrite] */
        port inout specDocsInOutPort : String;
        doc /* Merges reverse-synchronized specification entities into their owning subsystem `PackageDef` subpackages. [protocol: AtomicFsWrite] */
        port inout subpackageSyncOutPort : String;
    }
    doc /* Total number of generative LLM subagents invoked during specification, ICD, and WBS forward compilation (`C-FWD-01 == 0`). [unit: count] */
    part def DeterministicSpecIcdWbsForwardCompiler {
        attribute llm_subagent_invocation_count : Integer = 0;
        doc /* Percentage of declared SysML v2 AST packages, parts, ports, flows, interactions, and use cases projected into specifications (`C-FWD-02 == 100.0`). [unit: percent] */
        attribute ast_node_coverage_percent : Real = 100;
        doc /* Count of Check 13B E1-E5 Mermaid syntax or layout violations across all compiler-synthesized diagrams (`C-FWD-03 == 0`). [unit: count] */
        attribute mermaid_e1_e5_violation_count : Integer = 0;
        doc /* Count of AST structural or subpackage placement divergences after a forward-then-reverse synchronization cycle (`C-FWD-04 == 0`). [unit: count] */
        attribute forward_reverse_drift_count : Integer = 0;
        doc /* Receives the parsed, type-checked `SysmlPackage` (`PackageDef`) AST and SHA-256 `SchemaDigest` from `deap-core`. [protocol: MemoryBus] */
        port inout SysmlPackageAstInputPort : String;
        doc /* Emits deterministic `SpecSuiteBundle` Markdown specifications across `docs/epics/`, `docs/features/`, `docs/user-stories/`, and `docs/use-cases/`. [protocol: AtomicFsWrite] */
        port inout MarkdownSpecSuiteOutputPort : String;
        doc /* Emits `IcdDocumentPair` (`ICD_01_SYSTEM_INTERFACE_MATRIX.md` and `ICD_02_MASTER_SIGNAL_DICTIONARY.md`) under `docs/interfaces/`. [protocol: AtomicFsWrite] */
        port inout IcdMatrixArtifactPort : String;
        doc /* Emits `WbsExportBundle` (`WBS_01`..`WBS_03` Markdown artifacts and RFC 4180 CSV / JSON AST exports) under `docs/management/`. [protocol: AtomicFsWrite] */
        port inout WbsEnterpriseExportPort : String;
    }
    doc /* Maximum deterministic execution bound for `--emit-icd` interface matrix and signal dictionary compilation. [unit: ms] */
    part def LogicalIcdMatrixCompiler {
        attribute maxIcdLatencyMs : Integer = 10;
        doc /* Orders all `PartDef` matrix axes and `ConnectionDef` channels via `BTreeMap` deterministic traversal. [unit: flag] */
        attribute lexicographicalSort : Boolean = true;
        doc /* Receives `PartDef`, `PortDef`, `ConnectionDef`, `FlowDef`, and `ItemDef` AST nodes. [protocol: MemoryBus] */
        port inout interconnectAstInPort : String;
        doc /* Emits `ICD_01_SYSTEM_INTERFACE_MATRIX.md` and `ICD_02_MASTER_SIGNAL_DICTIONARY.md`. [protocol: AtomicFsWrite] */
        port inout icdMarkdownOutPort : String;
    }
    doc /* Canonical 4-level MIL-STD-881E decomposition depth (System, Subsystem, Work Package, Deliverable Task). [unit: levels] */
    part def MilStd881eWbsExporter {
        attribute wbsHierarchyDepth : Integer = 4;
        doc /* Maximum deterministic execution bound for `--emit-wbs` Markdown and CSV/JSON export generation. [unit: ms] */
        attribute maxWbsLatencyMs : Integer = 15;
        doc /* Receives `PackageDef` AST hierarchy and synchronized `docs/` specification frontmatter metadata. [protocol: MemoryBus] */
        port inout wbsAstMetadataInPort : String;
        doc /* Emits `WBS_01`..`WBS_03` Markdown registers and enterprise CSV/JSON AST exports. [protocol: AtomicFsWrite] */
        port inout wbsArtifactsOutPort : String;
    }
}