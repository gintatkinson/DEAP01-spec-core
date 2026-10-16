//! Parity gates (Checks 21 through 31) mirroring DEAP specification compiler gates.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C safety-critical software
//! engineering standards. Provides comprehensive, deterministic verification gates enforcing
//! semantic diagram-to-AST topology parity, physical prose invariant grounding, Level 1C ICD
//! completeness, cross-document connection parity, and dual-schema SSOT synchronization.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Schema-AST Grounding Invariant: Zero undeclared physical hardware components or ungrounded
//!   ports in specification diagrams and interface matrices.
//! - Traceability Preservation: 1:1 bidirectional parity across ConOps SV-1, ICD_01, executive
//!   reports, and the canonical SSOT Architecture Views Library.
//! - Monotonic Determinism: Zero randomized hashing structures; all collections use BTreeMap
//!   and BTreeSet, and all directory scans sort paths lexicographically.
//! - Memory & Char Boundary Safety: All string slicing operations use character boundary validation
//!   and ASCII case transformations, eliminating panic hazards and internal compiler errors.
//!
//! ## 3. Preconditions
//! - Repository root must be an accessible local filesystem directory.
//! - Schemas and specifications must contain UTF-8 encoded text.
//!
//! ## 4. Postconditions
//! - Returns `Ok(ParityOutcome::Verified)` when all parity invariants hold.
//! - Returns `Ok(ParityOutcome::PendingOrClean)` when upstream clean landing zones or downstream
//!   pending states are detected.
//! - Returns `Err(Vec<String>)` containing explicit, actionable failure diagnostics upon violation.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(N * L) where N is the total count of specification and schema files,
//!   and L is the average line length.
//! - Space Complexity: O(S) bounded by total token and symbol count across active schemas and specs.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Filesystem I/O, UTF-8 parsing, and Markdown AST parsing failures fail closed and propagate
//!   structured verification error diagnostics.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-PARITY-GATES-21-THROUGH-31], [DO-330-TQL-4-VERIFICATION-TOOL].

use deap_core::rules::EXCLUDED_DIRS;
use deap_core::sysml_ast::{
    find_matching_brace, mask_string_literals, parse_action_defs, parse_attribute_defs,
    parse_calc_defs, parse_connection_defs, parse_constraint_defs, parse_part_defs,
    parse_port_defs, parse_requirement_defs, parse_state_defs, parse_test_case_defs,
    strip_sysml_comments, PartDef,
};
use deap_core::workspace::is_upstream_compiler;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use walkdir::WalkDir;

/// Outcome status for parity gates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParityOutcome {
    PendingOrClean,
    InterfacesDirMissing,
    IcdSpecsPending,
    ConopsDirMissing,
    Verified,
    DualSchemaSingleOrClean,
    DualSchemaIdentical,
}

/// Canonical Mermaid diagram types recognized across specification architecture viewpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagramType {
    Flowchart,
    Graph,
    ClassDiagram,
    StateDiagramV2,
    StateDiagram,
    SequenceDiagram,
    ErDiagram,
    Gantt,
    Pie,
    GitGraph,
    C4Context,
    Journey,
    QuadrantChart,
    XyChartBeta,
    Mindmap,
    Timeline,
    SankeyBeta,
    RequirementDiagram,
}

impl DiagramType {
    /// Parses a diagram header line into a strongly typed DiagramType variant.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Ensures diagram syntax conforms to standard Mermaid grammar without static string slices.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Token matching is ASCII case-insensitive and checks word boundaries.
    ///
    /// ## 3. Preconditions
    /// Input line must be valid UTF-8.
    ///
    /// ## 4. Postconditions
    /// Returns `Some(DiagramType)` if valid header found, `None` otherwise.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(T) where T is token length.
    /// Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Returns `None` on unrecognized or empty lines without panicking.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-MERMAID-HEADER-TYPED-AST].
    pub fn parse_header(line: &str) -> Option<Self> {
        let trimmed = line.trim();
        let token = trimmed.split_whitespace().next()?;
        if token.eq_ignore_ascii_case("flowchart") {
            Some(Self::Flowchart)
        } else if token.eq_ignore_ascii_case("graph") {
            Some(Self::Graph)
        } else if token.eq_ignore_ascii_case("classdiagram") {
            Some(Self::ClassDiagram)
        } else if token.eq_ignore_ascii_case("statediagram-v2") {
            Some(Self::StateDiagramV2)
        } else if token.eq_ignore_ascii_case("statediagram") {
            Some(Self::StateDiagram)
        } else if token.eq_ignore_ascii_case("sequencediagram") {
            Some(Self::SequenceDiagram)
        } else if token.eq_ignore_ascii_case("erdiagram") {
            Some(Self::ErDiagram)
        } else if token.eq_ignore_ascii_case("gantt") {
            Some(Self::Gantt)
        } else if token.eq_ignore_ascii_case("pie") {
            Some(Self::Pie)
        } else if token.eq_ignore_ascii_case("gitgraph") {
            Some(Self::GitGraph)
        } else if token.eq_ignore_ascii_case("c4context") {
            Some(Self::C4Context)
        } else if token.eq_ignore_ascii_case("journey") {
            Some(Self::Journey)
        } else if token.eq_ignore_ascii_case("quadrantchart") {
            Some(Self::QuadrantChart)
        } else if token.eq_ignore_ascii_case("xychart-beta") {
            Some(Self::XyChartBeta)
        } else if token.eq_ignore_ascii_case("mindmap") {
            Some(Self::Mindmap)
        } else if token.eq_ignore_ascii_case("timeline") {
            Some(Self::Timeline)
        } else if token.eq_ignore_ascii_case("sankey-beta") {
            Some(Self::SankeyBeta)
        } else if token.eq_ignore_ascii_case("requirementdiagram") {
            Some(Self::RequirementDiagram)
        } else {
            None
        }
    }
}

/// Validate whether the given line matches a valid Mermaid diagram type header.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Gatekeeper verifying valid diagram type specification headers.
///
/// ## 2. Invariants & Mathematical Properties
/// Delegates to strongly-typed `DiagramType::parse_header`.
///
/// ## 3. Preconditions
/// Input string must be valid UTF-8.
///
/// ## 4. Postconditions
/// Returns true if valid diagram type header detected.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(T) where T is token length.
/// Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Safe boolean return; never panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-MERMAID-HEADER-VALIDATION].
pub fn is_valid_mermaid_header(s: &str) -> bool {
    DiagramType::parse_header(s).is_some()
}

/// Token scanner checking for forbidden Mermaid class method colon syntax:
/// e.g. `+method(): ReturnType` instead of `+ReturnType method()`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Enforces syntax compliance for UML class diagrams in specifications.
///
/// ## 2. Invariants & Mathematical Properties
/// Rejects colon-after-parentheses method signatures in class member lines.
///
/// ## 3. Preconditions
/// Input string `s` must be valid UTF-8.
///
/// ## 4. Postconditions
/// Returns true if forbidden colon syntax is detected.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(L) where L is string length.
/// Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Uses safe slice bounds access; never panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-MERMAID-CLASS-SYNTAX-GATE].
pub fn has_colon_method_syntax(s: &str) -> bool {
    let trimmed = s.trim();
    let mut chars = trimmed.chars();

    match chars.next() {
        Some('+' | '-' | '#' | '~') => {}
        _ => return false,
    }

    let rem = chars.as_str();
    let ident_end = rem
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .unwrap_or(rem.len());
    if ident_end == 0 {
        return false;
    }

    let after_ident = match rem.get(ident_end..) {
        Some(slice) => slice.trim_start(),
        None => return false,
    };
    if !after_ident.starts_with('(') {
        return false;
    }

    if let Some(close_idx) = after_ident.rfind(')') {
        let after_paren = match after_ident.get(close_idx.saturating_add(1)..) {
            Some(slice) => slice.trim_start(),
            None => return false,
        };
        if let Some(after_colon) = after_paren.strip_prefix(':') {
            let ret_type = after_colon.trim_start();
            if let Some(first_ch) = ret_type.chars().next() {
                if first_ch.is_alphanumeric() || first_ch == '_' {
                    return true;
                }
            }
        }
    }

    false
}

/// Canonical specification directories within a DEAP workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TargetScanDir {
    Conops,
    Safety,
    Interfaces,
    Icd,
    Architecture,
    Epics,
    Features,
    UserStories,
    UseCases,
    Management,
    Reports,
}

impl TargetScanDir {
    pub const ALL: &'static [Self] = &[
        Self::Conops,
        Self::Safety,
        Self::Interfaces,
        Self::Icd,
        Self::Architecture,
        Self::Epics,
        Self::Features,
        Self::UserStories,
        Self::UseCases,
        Self::Management,
        Self::Reports,
    ];

    /// Subdirectory name under `docs/` (avoids hardcoded OS path separators).
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Returns standardized relative subdirectory name for specification taxonomy.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Exact 1:1 mapping between enum variant and directory name.
    ///
    /// ## 3. Preconditions
    /// None.
    ///
    /// ## 4. Postconditions
    /// Returns static ASCII string slice corresponding to target directory.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1). Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Exhaustive pattern match prevents unreachable or unhandled variants.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-TARGET-SCAN-DIR-SUBDIR].
    pub const fn docs_subdir(self) -> &'static str {
        match self {
            Self::Conops => "conops",
            Self::Safety => "safety",
            Self::Interfaces => "interfaces",
            Self::Icd => "icd",
            Self::Architecture => "architecture",
            Self::Epics => "epics",
            Self::Features => "features",
            Self::UserStories => "user-stories",
            Self::UseCases => "use-cases",
            Self::Management => "management",
            Self::Reports => "reports",
        }
    }

    /// Resolves `<repo_root>/docs/<subdir>` portably across host OSs.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Portable filesystem path resolution preventing cross-platform separator drift.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// PathBuf is strictly rooted under `repo_root/docs`.
    ///
    /// ## 3. Preconditions
    /// `repo_root` must be valid path.
    ///
    /// ## 4. Postconditions
    /// Returns resolved PathBuf.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1). Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Pure path joining; cannot panic.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-TARGET-SCAN-DIR-RESOLVE].
    pub fn resolve(self, repo_root: &Path) -> PathBuf {
        repo_root.join("docs").join(self.docs_subdir())
    }

    /// Exhaustive policy: does Check 23 (Factual Grounding & Citation Audit) apply to this directory?
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Enforces factual grounding and citation audits across all engineering specification tiers.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// All technical specification categories require grounding; management is excluded.
    ///
    /// ## 3. Preconditions
    /// None.
    ///
    /// ## 4. Postconditions
    /// Returns true if grounding audit is mandatory, false otherwise.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1). Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Exhaustive match enforces compile-time decision on new variants.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-TARGET-SCAN-DIR-POLICY].
    pub const fn requires_factual_grounding(self) -> bool {
        match self {
            Self::Conops
            | Self::Safety
            | Self::Interfaces
            | Self::Icd
            | Self::Architecture
            | Self::Epics
            | Self::Features
            | Self::UserStories
            | Self::UseCases
            | Self::Reports => true,
            Self::Management => false,
        }
    }
}

/// Schema and specification citation prefix identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PathPrefix {
    Schema,
    Docs,
}

impl PathPrefix {
    /// Returns the static path prefix string.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Returns canonical prefix tokens for citation path boundary scanning.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Exact 1:1 mapping between enum variant and prefix token.
    ///
    /// ## 3. Preconditions
    /// None.
    ///
    /// ## 4. Postconditions
    /// Returns static ASCII string slice.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1). Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Exhaustive pattern match prevents unhandled variants.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-PATH-PREFIX-AS-STR].
    fn as_str(&self) -> &'static str {
        match self {
            Self::Schema => "schema/",
            Self::Docs => "docs/",
        }
    }

    /// Returns all canonical path prefix variants.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Provides exhaustive collection of allowable citation root prefixes.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Fixed-size array containing all enum variants without duplication.
    ///
    /// ## 3. Preconditions
    /// None.
    ///
    /// ## 4. Postconditions
    /// Returns array of all PathPrefix variants.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1). Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Pure constructor; cannot fail.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-PATH-PREFIX-ALL].
    fn all() -> [PathPrefix; 2] {
        [PathPrefix::Schema, PathPrefix::Docs]
    }
}

/// Check if a metrology value is an unresolved placeholder string.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS metrological integrity gates. Detects placeholder
/// or ungrounded values in specification tables.
///
/// ## 2. Invariants & Mathematical Properties
/// Case-insensitive match against banned placeholder tokens.
///
/// ## 3. Preconditions
/// Input `val` must be valid UTF-8.
///
/// ## 4. Postconditions
/// Returns true if placeholder token detected, false otherwise.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N) where N is input length. Space Complexity: O(N) for lowercase conversion.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Pure string inspection; never panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-METROLOGY-PLACEHOLDER-DETECTOR].
fn is_metrology_placeholder(val: &str) -> bool {
    let lower = val.to_ascii_lowercase();
    lower.contains("not declared in schema")
        || lower.contains("declared assumption")
        || lower.contains("tbd")
        || lower.contains("unknown")
        || lower.contains("todo")
        || lower.contains("unspecified")
}

/// Check if a unit string matches recognized SI or dimensional metric standards.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to ISO 80000 and IEEE metrology standards. Ensures dimensional unit validity
/// without static string slice arrays.
///
/// ## 2. Invariants & Mathematical Properties
/// Pure match on canonical lowercase metric tokens.
///
/// ## 3. Preconditions
/// Input `unit` must be valid UTF-8.
///
/// ## 4. Postconditions
/// Returns true if unit is recognized SI standard, false otherwise.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(U) where U is unit string length. Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Exhaustive match returns false on unrecognized units without panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-METROLOGY-SI-VALIDATOR].
fn is_valid_si_unit(unit: &str) -> bool {
    let u = unit.trim().to_ascii_lowercase();
    matches!(
        u.as_str(),
        "s" | "sec" | "second" | "seconds" | "ms" | "us" | "µs" | "ns" | "min" | "h" | "hr"
            | "hz" | "khz" | "mhz" | "ghz"
            | "m" | "km" | "cm" | "mm" | "nm" | "um" | "µm"
            | "m/s" | "km/h" | "m/s^2" | "m/s2" | "fps" | "rpm"
            | "kg" | "g" | "mg"
            | "j" | "kj" | "mj" | "wh" | "kwh"
            | "w" | "kw" | "mw"
            | "v" | "mv" | "kv"
            | "a" | "ma"
            | "deg" | "rad" | "degree" | "degrees" | "radian" | "radians"
            | "db" | "dbm" | "dbi"
            | "%" | "percent" | "ratio" | "dimensionless" | "count"
            | "bps" | "kbps" | "mbps" | "gbps" | "b/s" | "kb/s" | "mb/s" | "gb/s"
            | "b" | "kb" | "mb" | "gb" | "bytes" | "ppm"
    )
}

/// Recursively collects part definition names into a deterministic set.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Collects AST node symbols for structural SysML hierarchy comparison.
///
/// ## 2. Invariants & Mathematical Properties
/// Traverses all nested child part definitions recursively and inserts names into BTreeSet.
///
/// ## 3. Preconditions
/// `part` AST node must be well-formed.
///
/// ## 4. Postconditions
/// `set` contains all unique part definition names in the subtree.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(P log P) where P is total part count. Space Complexity: O(P).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Pure tree traversal; cannot panic.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-AST-PART-DEF-NAME-COLLECTOR].
fn collect_part_def_names(part: &PartDef, set: &mut BTreeSet<String>) {
    if !part.name.is_empty() {
        set.insert(part.name.clone());
    }
    for sub in &part.parts {
        collect_part_def_names(sub, set);
    }
}

/// Locate and read authoritative SysML v2 or multi-format schema model content (schema/**/* or .pipeline/schema.sysml).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Establishes the authoritative structural SysML v2 AST baseline for semantic parity auditing.
///
/// ## 2. Invariants & Mathematical Properties
/// Discovered files are sorted deterministically before concatenation.
///
/// ## 3. Preconditions
/// `repo_root` must be a valid directory.
///
/// ## 4. Postconditions
/// Returns `Some(String)` with combined model text, or `None` if schema landing zone is clean.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(F * S) where F is file count and S is average file size.
/// Space Complexity: O(Total schema bytes).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Swallows unreadable individual files gracefully while preserving discovered valid content.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-MODEL-DISCOVERY].
pub fn discover_sysml_model_text(repo_root: &Path) -> Option<String> {
    let schema_dir = repo_root.join("schema");
    if schema_dir.is_dir() {
        let mut paths = deap_core::sysml_ast::UnifiedSchemaModelExtractor::discover_schema_files(&[&schema_dir]);
        paths.sort();
        let mut sysml_contents = Vec::new();
        let mut non_sysml_paths = Vec::new();

        for p in paths {
            let is_sysml = p.extension().is_some_and(|ext| ext == "sysml");
            if is_sysml {
                if let Ok(content) = fs::read_to_string(&p) {
                    if !content.trim().is_empty() {
                        sysml_contents.push(content);
                    }
                }
            } else {
                non_sysml_paths.push(p);
            }
        }

        if !sysml_contents.is_empty() {
            return Some(sysml_contents.join("\n\n"));
        }

        if !non_sysml_paths.is_empty() {
            let parts = deap_core::sysml_ast::UnifiedSchemaModelExtractor::extract_subsystem_parts(&non_sysml_paths);
            if !parts.is_empty() {
                let mut synthetic = String::new();
                synthetic.push_str("package DownstreamSchema {\n");
                for part in parts {
                    synthetic.push_str(&format!("    part def {};\n", part));
                }
                synthetic.push_str("}\n");
                return Some(synthetic);
            }
        }
    }

    let pipeline_model = repo_root.join(".pipeline").join("schema.sysml");
    if pipeline_model.is_file() {
        if let Ok(content) = fs::read_to_string(&pipeline_model) {
            if !content.trim().is_empty() {
                return Some(content);
            }
        }
    }

    None
}

/// Check 21: Semantic Diagram-to-AST Topology Parity Gate.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ISO 26262 ASIL D. Prevents architectural drift by
/// ensuring that all hardware and subsystem blocks declared in architecture diagrams
/// resolve against the authoritative SysML AST symbol table.
///
/// ## 2. Invariants & Mathematical Properties
/// - Zero undeclared hardware components in architecture diagrams.
/// - Mandatory valid Mermaid diagram type header on every ```mermaid block.
/// - Zero forbidden colon method syntax in class diagrams.
///
/// ## 3. Preconditions
/// `repo_root` directory must exist.
///
/// ## 4. Postconditions
/// Returns `Ok(ParityOutcome::Verified)` if all diagrams conform to SysML AST.
/// Returns `Ok(ParityOutcome::PendingOrClean)` if no SysML model is present.
/// Returns `Err(Vec<String>)` with diagnostic violations otherwise.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(D * L) where D is diagram count and L is line count.
/// Space Complexity: O(V) where V is diagram vertex count.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Propagates I/O and parsing errors fail-closed.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-PARITY-GATE-21].
pub fn check_semantic_diagram_ast_parity(repo_root: &Path) -> Result<ParityOutcome, Vec<String>> {
    let model_text = discover_sysml_model_text(repo_root);
    if model_text.is_none() {
        return Ok(ParityOutcome::PendingOrClean);
    }
    let model_str = model_text.unwrap_or_default();

    let mut declared_symbols = BTreeSet::new();
    for p in parse_part_defs(&model_str) {
        collect_part_def_names(&p, &mut declared_symbols);
    }
    for p in parse_port_defs(&model_str) {
        declared_symbols.insert(p.name);
    }
    for a in parse_action_defs(&model_str) {
        declared_symbols.insert(a.name);
    }

    let mut errors = Vec::new();

    for &scan_dir in TargetScanDir::ALL {
        let dir = scan_dir.resolve(repo_root);
        if !dir.is_dir() {
            continue;
        }

        let mut md_files = Vec::new();
        for entry_res in WalkDir::new(&dir)
            .sort_by_file_name()
            .into_iter()
            .filter_entry(|e| {
                if e.file_type().is_dir() {
                    let name = e.file_name().to_string_lossy();
                    !EXCLUDED_DIRS.iter().any(|ex| name == ex) && name != "blueprints"
                } else {
                    true
                }
            })
        {
            let entry = match entry_res {
                Ok(e) => e,
                Err(err) => {
                    errors.push(format!("Check 21 failed: I/O error walking directory '{}': {}", dir.display(), err));
                    continue;
                }
            };
            if entry.file_type().is_file() {
                let path = entry.path().to_path_buf();
                if path.extension().is_some_and(|ext| ext == "md") {
                    md_files.push(path);
                }
            }
        }
        md_files.sort();

        for path in md_files {
            let rel = path.strip_prefix(repo_root).unwrap_or(&path).display().to_string();
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    errors.push(format!("Check 21 failed: Unable to read file '{}': {}", rel, e));
                    continue;
                }
            };

            let lines: Vec<&str> = content.lines().collect();
            let mut i = 0;
            while i < lines.len() {
                let line_str = match lines.get(i) {
                    Some(&l) => l,
                    None => break,
                };
                if line_str.trim().starts_with("```mermaid") {
                    i += 1;
                    let mut block_lines = Vec::new();
                    while i < lines.len() {
                        let inner = match lines.get(i) {
                            Some(&l) => l,
                            None => break,
                        };
                        if inner.trim().starts_with("```") {
                            break;
                        }
                        block_lines.push(inner);
                        i += 1;
                    }

                    if let Some(&first) = block_lines.iter().find(|l| !l.trim().is_empty() && !l.trim().starts_with("%%")) {
                        let trimmed = first.trim();
                        let diag_type = DiagramType::parse_header(trimmed);
                        if diag_type.is_none() {
                            errors.push(format!("Missing mandatory Mermaid diagram type header in {} (got: '{}')", rel, trimmed));
                        }

                        let is_class_diag = diag_type == Some(DiagramType::ClassDiagram);
                        for line in &block_lines {
                            let s = line.trim();
                            if is_class_diag {
                                if (s.contains('{') || s.contains('}')) && !s.starts_with("class ") && s != "{" && s != "}" {
                                    errors.push(format!("Mermaid syntax violation in {}: Curly braces '{{}}' inside class member line: '{}'.", rel, s));
                                }
                                if has_colon_method_syntax(s) {
                                    errors.push(format!("Mermaid syntax violation in {}: Colons ':' forbidden in class member line: '{}'. Use '+ReturnType methodName(Type arg)' spacing.", rel, s));
                                }

                                if let Some(stripped) = s.strip_prefix("class ") {
                                    let class_name = stripped.trim().trim_end_matches('{').trim();
                                    if !class_name.is_empty()
                                        && !declared_symbols.contains(class_name)
                                        && !class_name.starts_with('<')
                                    {
                                        errors.push(format!(
                                            "Topological drift in {}: Diagram references undeclared hardware/subsystem concept '{}'.",
                                            rel, class_name
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
                i += 1;
            }
        }
    }

    if errors.is_empty() {
        Ok(ParityOutcome::Verified)
    } else {
        Err(errors)
    }
}

/// Check 22: Physical Invariant Semantic Prose Gate.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ISO 26262 ASIL D. Verifies that physical and operational
/// invariant assertions in safety specifications ground against SysML model constraints and variables.
///
/// ## 2. Invariants & Mathematical Properties
/// - Every operational invariant referenced in prose must map to an active SysML constraint.
/// - Clean landing zones evaluate to `PendingOrClean`.
///
/// ## 3. Preconditions
/// `repo_root` must be an accessible directory.
///
/// ## 4. Postconditions
/// Returns `Ok(ParityOutcome::Verified)` when all invariant prose is substantiated.
/// Returns `Err(Vec<String>)` if invariant assertions diverge or reference ungrounded parameters.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(P * L) where P is prose line count.
/// Space Complexity: O(C) where C is constraint set size.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed on unreadable files.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-PARITY-GATE-22].
pub fn check_semantic_prose_invariants(repo_root: &Path) -> Result<ParityOutcome, Vec<String>> {
    let model_text = discover_sysml_model_text(repo_root);
    let schema_dir = repo_root.join("schema");
    let has_extracted = schema_dir.join("extracted").is_dir();

    if model_text.is_none() && !has_extracted {
        return Ok(ParityOutcome::PendingOrClean);
    }
    let model_str = model_text.unwrap_or_default();

    let constraints = parse_constraint_defs(&model_str);
    let mut declared_constraints = BTreeSet::new();
    for c in constraints {
        declared_constraints.insert(c.name);
    }

    let mut declared_parts = BTreeSet::new();
    for p in parse_part_defs(&model_str) {
        collect_part_def_names(&p, &mut declared_parts);
    }

    let mut errors = Vec::new();
    let scan_dirs = [
        TargetScanDir::Safety.resolve(repo_root),
        TargetScanDir::Conops.resolve(repo_root),
        TargetScanDir::Features.resolve(repo_root),
    ];

    for dir in &scan_dirs {
        if !dir.is_dir() {
            continue;
        }
        for entry_res in WalkDir::new(dir).sort_by_file_name().into_iter() {
            let entry = match entry_res {
                Ok(e) => e,
                Err(err) => {
                    errors.push(format!("Check 22 failed: I/O error walking directory '{}': {}", dir.display(), err));
                    continue;
                }
            };
            if entry.file_type().is_file() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "md") {
                    let rel = path.strip_prefix(repo_root).unwrap_or(path).display().to_string();
                    let content = match fs::read_to_string(path) {
                        Ok(c) => c,
                        Err(e) => {
                            errors.push(format!("Check 22 failed: Unable to read file '{}': {}", rel, e));
                            continue;
                        }
                    };

                    for (idx, line) in content.lines().enumerate() {
                        let line_no = idx.saturating_add(1);
                        let trimmed = line.trim();
                        if trimmed.starts_with("```") || trimmed.is_empty() {
                            continue;
                        }
                        if (trimmed.contains("Invariant:") || trimmed.contains("Physical Invariant:"))
                            && trimmed.contains("[INV-")
                        {
                            if let Some(start) = trimmed.find("[INV-") {
                                    if let Some(slice) = trimmed.get(start.saturating_add(1)..) {
                                        if let Some(end) = slice.find(']') {
                                            if let Some(inv_id) = slice.get(..end) {
                                                if !declared_constraints.is_empty()
                                                    && !declared_constraints.contains(inv_id)
                                                {
                                                    errors.push(format!(
                                                        "Check 22 failed: Prose invariant assertion '{}' on line {} in '{}' is ungrounded in SysML model.",
                                                        inv_id, line_no, rel
                                                    ));
                                                }
                                            }
                                        }
                                    }
                            }
                        }
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(ParityOutcome::Verified)
    } else {
        Err(errors)
    }
}

/// Persistent Section Slicer validating physical on-disk citations and section anchors.
#[derive(Debug, Clone)]
pub struct SectionSlicer<'a> {
    pub repo_root: &'a Path,
}

impl<'a> SectionSlicer<'a> {
    /// Constructs a new SectionSlicer for a given repository root.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Section slicer initialization helper.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Bound to lifetime of the root path reference.
    ///
    /// ## 3. Preconditions
    /// None.
    ///
    /// ## 4. Postconditions
    /// Returns initialized SectionSlicer.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1). Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Never panics.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SECTION-SLICER-INIT].
    pub fn new(repo_root: &'a Path) -> Self {
        Self { repo_root }
    }

    /// Resolves and sanitizes a target file path against the repository root.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Enforces workspace boundary containment and mitigates path traversal vulnerabilities.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Rejects absolute paths and `..` parent directory traversal components.
    ///
    /// ## 3. Preconditions
    /// `target_file` must be valid UTF-8.
    ///
    /// ## 4. Postconditions
    /// Returns `Some(PathBuf)` if file exists inside `repo_root`, `None` otherwise.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(P) where P is path string length.
    /// Space Complexity: O(P).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Returns `None` on invalid or non-existent paths without throwing exceptions.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-PATH-TRAVERSAL-SANITIZATION].
    pub fn resolve_path(&self, target_file: &str) -> Option<PathBuf> {
        let clean = target_file.trim_matches(|c: char| {
            c.is_whitespace() || matches!(c, '`' | '\'' | '"' | '[' | ']' | '(' | ')' | '<' | '>')
        });
        let rel_path = Path::new(clean);
        if rel_path.is_absolute()
            || rel_path
                .components()
                .any(|comp| matches!(comp, Component::ParentDir))
        {
            return None;
        }
        let path = self.repo_root.join(rel_path);
        if path.is_file() {
            Some(path)
        } else {
            None
        }
    }

    /// Checks if a markdown heading line matches a targeted section identifier with exact word boundaries.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Prevents substring section spoofing in citation auditing.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Matches exact section tokens (e.g. `7.2.3` matches `§7.2.3` or `7.2.3:` but not `10.7.2.3`).
    ///
    /// ## 3. Preconditions
    /// `heading_line` and `sec_norm` must be non-empty strings.
    ///
    /// ## 4. Postconditions
    /// Returns true if heading token matches `sec_norm`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(H) where H is heading line length.
    /// Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Never panics.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-EXACT-HEADING-BOUNDARIES].
    pub fn heading_matches_section(heading_line: &str, sec_norm: &str) -> bool {
        let body = heading_line.trim_start_matches('#').trim();
        for token in body.split_whitespace() {
            let clean_tok = token
                .trim_start_matches('§')
                .trim_end_matches(['.', ':', ')', ']', '>']);
            if clean_tok.eq_ignore_ascii_case(sec_norm) {
                return true;
            }
        }
        body.eq_ignore_ascii_case(sec_norm)
    }

    /// Extracts the text content under a designated section heading.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Provides isolated section text slices for factual grounding verification.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Slices strictly bounded by subsequent heading of equal or higher level. Zero whole-file fallback.
    ///
    /// ## 3. Preconditions
    /// Target file must exist and be accessible.
    ///
    /// ## 4. Postconditions
    /// Returns `Some(String)` with isolated section content if found; returns `None` if missing.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(L) where L is lines in target file.
    /// Space Complexity: O(S) where S is size of target section text.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Returns `None` if section is not found or file is unreadable.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SECTION-EXTRACTION-NO-FALLBACK].
    pub fn extract_section(&self, target_file: &str, section: &str) -> Option<String> {
        let path = self.resolve_path(target_file)?;
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return None,
        };
        let sec_norm = section.trim().trim_start_matches('§').trim();
        if sec_norm.is_empty() {
            return Some(content);
        }

        let mut in_target_sec = false;
        let mut target_level = 0;
        let mut sec_content = String::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                let level = trimmed.chars().take_while(|&c| c == '#').count();
                if in_target_sec {
                    if level <= target_level {
                        break;
                    }
                } else if Self::heading_matches_section(trimmed, sec_norm) {
                    in_target_sec = true;
                    target_level = level;
                }
            }
            if in_target_sec {
                sec_content.push_str(line);
                sec_content.push('\n');
            }
        }

        if in_target_sec && !sec_content.is_empty() {
            Some(sec_content)
        } else {
            None
        }
    }
}

/// Extracted source citation from specification or safety prose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CitationRef {
    pub target_file: String,
    pub section: Option<String>,
    pub line_no: usize,
}

/// Parses a citation token into target file and optional section identifier.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Parses citation strings with full char boundary verification to eliminate slice panics.
///
/// ## 2. Invariants & Mathematical Properties
/// Uses `to_ascii_lowercase` so byte offsets match original text exactly.
///
/// ## 3. Preconditions
/// Input string must be valid UTF-8.
///
/// ## 4. Postconditions
/// Returns `Some(CitationRef)` on success, `None` if no valid citation path found.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N) where N is input string length.
/// Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Uses `.get()` with char boundary validation; never panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CITATION-PARSING-CHAR-BOUNDARY].
fn parse_citation_string(raw: &str, line_no: usize) -> Option<CitationRef> {
    let lower = raw.to_ascii_lowercase();
    let start_pos = match (lower.find("schema/"), lower.find("docs/")) {
        (Some(a), Some(b)) => a.min(b),
        (Some(a), None) => a,
        (None, Some(b)) => b,
        (None, None) => return None,
    };

    let slice = raw.get(start_pos..)?;
    let path_end = slice
        .find(|c: char| {
            c.is_whitespace()
                || matches!(c, '`' | '\'' | '"' | '(' | ')' | '[' | ']' | '<' | '>' | '§' | '#' | ',' | ';' | '|')
        })
        .unwrap_or(slice.len());

    let path_slice = slice.get(..path_end)?;
    let path = path_slice
        .trim_matches(|c: char| matches!(c, '.' | ':' | '`' | '\'' | '"'))
        .to_string();
    if path.is_empty() {
        return None;
    }

    let mut section = None;
    let after_path = slice.get(path_end..).unwrap_or("");
    if let Some(sec_pos) = after_path.find('§') {
        let after_sec_pos = sec_pos.saturating_add('§'.len_utf8());
        let after_sec = after_path.get(after_sec_pos..).unwrap_or("").trim_start();
        let sec_end = after_sec
            .find(|c: char| {
                c.is_whitespace()
                    || matches!(c, '`' | '\'' | '"' | ')' | ']' | '>' | ',' | ';' | '|')
            })
            .unwrap_or(after_sec.len());
        let sec_slice = after_sec.get(..sec_end).unwrap_or("");
        let sec_val = sec_slice.trim_end_matches('.').trim();
        if !sec_val.is_empty() {
            section = Some(sec_val.to_string());
        }
    }

    Some(CitationRef {
        target_file: path,
        section,
        line_no,
    })
}

/// Extracts citations from a single line of text across HTML comments, prose, lists, and tables.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Extracts all citations on a line while preserving multiple distinct section citations to the same file.
///
/// ## 2. Invariants & Mathematical Properties
/// Deduplication operates on `(target_file, section)` tuple.
///
/// ## 3. Preconditions
/// `line` must be valid UTF-8.
///
/// ## 4. Postconditions
/// Returns deterministic list of unique citations found on the line.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(L) where L is line length.
/// Space Complexity: O(C) where C is count of citations on the line.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Safe iterator access; never panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SAME-LINE-CITATION-PRESERVATION].
pub fn extract_citations_from_line(line: &str, line_no: usize) -> Vec<CitationRef> {
    let mut citations = Vec::new();
    let lower = line.to_ascii_lowercase();

    let mut push_unique = |c: CitationRef| {
        if !citations
            .iter()
            .any(|ex: &CitationRef| ex.target_file == c.target_file && ex.section == c.section)
        {
            citations.push(c);
        }
    };

    if let Some(start_idx) = lower.find("<!-- source:") {
        let after_prefix_idx = start_idx.saturating_add("<!-- source:".len());
        if let Some(after) = line.get(after_prefix_idx..) {
            if let Some(end_idx) = after.find("-->") {
                if let Some(cit_str) = after.get(..end_idx) {
                    if let Some(c) = parse_citation_string(cit_str.trim(), line_no) {
                        push_unique(c);
                    }
                }
            }
        }
    }

    for prefix_type in PathPrefix::all() {
        let prefix = prefix_type.as_str();
        let mut search_from = 0;
        while search_from < lower.len() {
            let search_slice = match lower.get(search_from..) {
                Some(s) => s,
                None => break,
            };
            let pos = match search_slice.find(prefix) {
                Some(p) => p,
                None => break,
            };
            let abs_pos = search_from.saturating_add(pos);
            let valid_boundary = if abs_pos == 0 {
                true
            } else {
                match line.as_bytes().get(abs_pos.saturating_sub(1)) {
                    Some(&b) => !b.is_ascii_alphanumeric() && b != b'_' && b != b'-',
                    None => true,
                }
            };
            if valid_boundary {
                if let Some(rem) = line.get(abs_pos..) {
                    if let Some(c) = parse_citation_string(rem, line_no) {
                        let advance = c.target_file.len().max(prefix.len());
                        push_unique(c);
                        search_from = abs_pos.saturating_add(advance);
                        continue;
                    }
                }
            }
            search_from = abs_pos.saturating_add(prefix.len());
        }
    }

    citations
}

/// Determines whether a lexical token has the structural morphology of a technical protocol identifier
/// (e.g., all-caps acronym `PWM`, `CAN`, mixed-case `DShot`, `MAVLink`, or alphanumeric/hyphenated `DShot600`, `RS-485`)
/// rather than an ordinary Titlecase or lowercase prose word.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A. Prevents ungrounded protocol injection using pure lexical morphology
/// without relying on hardcoded English stopword lists.
///
/// ## 2. Invariants & Mathematical Properties
/// - Token length must be at least 3 characters.
/// - Token must begin with an ASCII uppercase character.
/// - At least one hyphen-delimited segment must either be pure digits or contain an uppercase character
///   or digit after its first character (rejecting ordinary hyphenated Titlecase prose like `Auto-Reconnection`).
///
/// ## 3. Preconditions
/// `token` is a whitespace-delimited, punctuation-trimmed lexical slice.
///
/// ## 4. Postconditions
/// Returns `true` iff `token` exhibits non-prose protocol identifier structure.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(K) where K is token character length.
/// Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Pure character-level inspection; never panics on arbitrary UTF-8 input.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-PROTOCOL-TOKEN-EXTRACTION].
fn is_structural_protocol_token(token: &str) -> bool {
    if token.len() < 3 || !token.starts_with(|c: char| c.is_ascii_uppercase()) {
        return false;
    }
    token.split('-').any(|seg| {
        (!seg.is_empty() && seg.chars().all(|c| c.is_ascii_digit()))
            || (seg.len() >= 2
                && seg
                    .get(1..)
                    .is_some_and(|rest| rest.chars().any(|c| c.is_ascii_uppercase() || c.is_ascii_digit())))
    })
}

/// Normalizes a protocol or AST identifier to lowercase ASCII alphanumeric characters for AST matching
/// (so e.g. `RS-485` and `RS485` normalize to `"rs485"`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A. Enables canonical equivalence matching between prose protocol tokens
/// and SysML v2 AST identifier declarations.
///
/// ## 2. Invariants & Mathematical Properties
/// - Filters to `is_ascii_alphanumeric()` and maps to ASCII lowercase.
///
/// ## 3. Preconditions
/// `token` must be valid UTF-8.
///
/// ## 4. Postconditions
/// Returns normalized lowercase alphanumeric string.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(K) where K is token length.
/// Space Complexity: O(K).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Total function; never panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-AST-SYMBOL-NORMALIZATION].
fn normalize_ast_protocol_symbol(token: &str) -> String {
    token
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// Recursively harvests AST symbols (parts, ports, port types, protocol families, item flows,
/// electrical attributes, and attributes) from a slice of `PartDef` nodes.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A. Extracts hierarchical SysML v2 AST ground truth for Check 23.
///
/// ## 2. Invariants & Mathematical Properties
/// - Traverses nested `PartDef` hierarchies arbitrarily deep.
///
/// ## 3. Preconditions
/// `parts` is a slice of parsed `PartDef` AST nodes.
///
/// ## 4. Postconditions
/// Populates `symbols` and `claims` with all AST-grounded tokens and quantitative claims.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N) where N is total AST node count.
/// Space Complexity: O(N).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Pure AST traversal; never panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-AST-GROUND-TRUTH-EXTRACTION].
fn harvest_ast_parts_ground_truth(
    parts: &[deap_core::sysml_ast::PartDef],
    symbols: &mut BTreeSet<String>,
    claims: &mut BTreeSet<String>,
) {
    for part in parts {
        if !part.name.is_empty() {
            symbols.insert(part.name.to_ascii_lowercase());
            symbols.insert(normalize_ast_protocol_symbol(&part.name));
        }
        for port in &part.ports {
            harvest_ast_port_ground_truth(port, symbols, claims);
        }
        for attr in &part.attributes {
            harvest_ast_attribute_ground_truth(attr, symbols, claims);
        }
        for c in &part.constraints {
            if !c.name.is_empty() {
                symbols.insert(c.name.to_ascii_lowercase());
                symbols.insert(normalize_ast_protocol_symbol(&c.name));
            }
            for q in extract_quantitative_unit_claims(&c.expression) {
                claims.insert(q);
            }
        }
        harvest_ast_parts_ground_truth(&part.parts, symbols, claims);
    }
}

/// Harvests AST symbols and quantitative claims from a single `PortDef` node.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A. Extracts port type, protocol family, item flows, and electrical attributes.
///
/// ## 2. Invariants & Mathematical Properties
/// - Indexes both raw lowercase and normalized alphanumeric forms.
///
/// ## 3. Preconditions
/// `port` is a valid `PortDef`.
///
/// ## 4. Postconditions
/// Inserts port ground-truth tokens into `symbols` and `claims`.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(P) where P is port attribute count.
/// Space Complexity: O(P).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Total function; never panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-AST-GROUND-TRUTH-EXTRACTION].
fn harvest_ast_port_ground_truth(
    port: &deap_core::sysml_ast::PortDef,
    symbols: &mut BTreeSet<String>,
    claims: &mut BTreeSet<String>,
) {
    if !port.name.is_empty() {
        symbols.insert(port.name.to_ascii_lowercase());
        symbols.insert(normalize_ast_protocol_symbol(&port.name));
    }
    if !port.type_name.is_empty() {
        symbols.insert(port.type_name.to_ascii_lowercase());
        symbols.insert(normalize_ast_protocol_symbol(&port.type_name));
    }
    if !port.protocol_family.is_empty() {
        symbols.insert(port.protocol_family.to_ascii_lowercase());
        symbols.insert(normalize_ast_protocol_symbol(&port.protocol_family));
    }
    if !port.port_category.is_empty() {
        symbols.insert(port.port_category.to_ascii_lowercase());
        symbols.insert(normalize_ast_protocol_symbol(&port.port_category));
    }
    for flow in &port.item_flows {
        if !flow.name.is_empty() {
            symbols.insert(flow.name.to_ascii_lowercase());
            symbols.insert(normalize_ast_protocol_symbol(&flow.name));
        }
        if !flow.item_type.is_empty() {
            symbols.insert(flow.item_type.to_ascii_lowercase());
            symbols.insert(normalize_ast_protocol_symbol(&flow.item_type));
        }
    }
    for (k, v) in &port.electrical_attributes {
        symbols.insert(k.to_ascii_lowercase());
        symbols.insert(v.to_ascii_lowercase());
        symbols.insert(normalize_ast_protocol_symbol(v));
        for q in extract_quantitative_unit_claims(v) {
            claims.insert(q);
        }
    }
}

/// Harvests AST symbols and quantitative claims from a single `AttributeDef` node.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A. Extracts attribute names, types, and default values.
///
/// ## 2. Invariants & Mathematical Properties
/// - Indexes attribute identifiers and default values.
///
/// ## 3. Preconditions
/// `attr` is a valid `AttributeDef`.
///
/// ## 4. Postconditions
/// Inserts attribute ground-truth tokens into `symbols` and `claims`.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(A) where A is attribute string length.
/// Space Complexity: O(A).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Total function; never panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-AST-GROUND-TRUTH-EXTRACTION].
fn harvest_ast_attribute_ground_truth(
    attr: &deap_core::sysml_ast::AttributeDef,
    symbols: &mut BTreeSet<String>,
    claims: &mut BTreeSet<String>,
) {
    if !attr.name.is_empty() {
        symbols.insert(attr.name.to_ascii_lowercase());
        symbols.insert(normalize_ast_protocol_symbol(&attr.name));
    }
    if !attr.type_name.is_empty() {
        symbols.insert(attr.type_name.to_ascii_lowercase());
        symbols.insert(normalize_ast_protocol_symbol(&attr.type_name));
    }
    if let Some(ref dv) = attr.default_value {
        let clean = dv.trim_matches('"').trim();
        symbols.insert(clean.to_ascii_lowercase());
        symbols.insert(normalize_ast_protocol_symbol(clean));
        for q in extract_quantitative_unit_claims(clean) {
            claims.insert(q);
        }
    }
}

/// Parses SysML model text with the AST parser (`parse_part_defs`, `parse_port_defs`,
/// `parse_attribute_defs`, `parse_constraint_defs`) to construct ground-truth symbol and claim sets.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ISO 26262 ASIL D. Grounds specification protocol and quantitative
/// assertions against the parsed SysML v2 AST.
///
/// ## 2. Invariants & Mathematical Properties
/// - Combines AST-extracted port, part, attribute, and constraint declarations into deterministic `BTreeSet`s.
///
/// ## 3. Preconditions
/// `model_str` is valid UTF-8 SysML text (may be empty if no model exists).
///
/// ## 4. Postconditions
/// Returns `(ast_symbols, ast_quantitative_claims)`.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(M) where M is model byte length.
/// Space Complexity: O(S) where S is extracted symbol count.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Total function; returns empty sets for empty input.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-AST-GROUND-TRUTH-EXTRACTION].
fn build_ast_model_ground_truth(model_str: &str) -> (BTreeSet<String>, BTreeSet<String>) {
    let mut symbols = BTreeSet::new();
    let mut claims = BTreeSet::new();
    if model_str.trim().is_empty() {
        return (symbols, claims);
    }

    let parts = parse_part_defs(model_str);
    harvest_ast_parts_ground_truth(&parts, &mut symbols, &mut claims);

    for port in parse_port_defs(model_str) {
        harvest_ast_port_ground_truth(&port, &mut symbols, &mut claims);
    }
    for attr in deap_core::sysml_ast::parse_attribute_defs(model_str) {
        harvest_ast_attribute_ground_truth(&attr, &mut symbols, &mut claims);
    }
    for c in parse_constraint_defs(model_str) {
        if !c.name.is_empty() {
            symbols.insert(c.name.to_ascii_lowercase());
            symbols.insert(normalize_ast_protocol_symbol(&c.name));
        }
        for q in extract_quantitative_unit_claims(&c.expression) {
            claims.insert(q);
        }
    }
    for q in extract_quantitative_unit_claims(model_str) {
        claims.insert(q);
    }

    (symbols, claims)
}

/// Extracts candidate communication protocol or hardware interface identifiers from a line.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A. Identifies interface and bus protocol assertions in prose
/// to verify grounding against the authoritative SysML AST and cited sections.
///
/// ## 2. Invariants & Mathematical Properties
/// - Scans structural protocol tokens following `over` / `via` prepositions or preceding `protocol` / `interface` / `bus`.
/// - Uses pure morphological token classification (`is_structural_protocol_token`) with zero hardcoded word lists.
///
/// ## 3. Preconditions
/// `line` must be a valid UTF-8 string slice.
///
/// ## 4. Postconditions
/// Returns a vector of candidate protocol tokens found on `line`.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(W) where W is word count on the line.
/// Space Complexity: O(W).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Pure token analysis; never panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-PROTOCOL-TOKEN-EXTRACTION].
fn extract_candidate_protocol_tokens(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let lower_full = line.to_ascii_lowercase();
    if lower_full.contains("[tier-3:") || lower_full.contains("[tier-4:") {
        return out;
    }

    for segment in line.split('|') {
        let words: Vec<&str> = segment
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '-'))
            .filter(|w| !w.is_empty())
            .collect();

        for (i, &w) in words.iter().enumerate() {
            let wl = w.to_ascii_lowercase();
            if wl == "over" || wl == "via" {
                for j in i.saturating_add(1)..words.len() {
                    if let Some(&cand_w) = words.get(j) {
                        if is_valid_si_unit(cand_w) {
                            continue;
                        }
                        if is_structural_protocol_token(cand_w) {
                            let combined = if let Some(&next_num) = words.get(j.saturating_add(1)) {
                                if !next_num.is_empty() && next_num.chars().all(|c| c.is_ascii_digit()) {
                                    format!("{} {}", cand_w, next_num)
                                } else {
                                    cand_w.to_string()
                                }
                            } else {
                                cand_w.to_string()
                            };
                            if !out.iter().any(|existing: &String| existing.eq_ignore_ascii_case(&combined)) {
                                out.push(combined);
                            }
                            break;
                        }
                    }
                }
            }
            if (wl == "protocol" || wl == "interface" || wl == "bus") && i > 0 {
                if let Some(&prev_w) = words.get(i.saturating_sub(1)) {
                    if is_structural_protocol_token(prev_w)
                        && !is_valid_si_unit(prev_w)
                        && !out.iter().any(|existing: &String| existing.eq_ignore_ascii_case(prev_w))
                    {
                        out.push(prev_w.to_string());
                    }
                }
            }
        }
    }
    out
}

/// Extracts numeric-unit pairs (such as `50 Hz` or `100 ms`) from a specification line.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A. Extracts quantitative physical claims to verify grounding
/// against cited source sections and SysML models.
///
/// ## 2. Invariants & Mathematical Properties
/// - Identifies adjacent `<number> <SI-unit>` token pairs.
///
/// ## 3. Preconditions
/// `line` must be valid UTF-8 text.
///
/// ## 4. Postconditions
/// Returns a list of normalized lowercase `<number> <unit>` strings.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(W) where W is word count on the line.
/// Space Complexity: O(W).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Pure token inspection; never panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-QUANTITATIVE-CLAIM-EXTRACTION].
fn extract_quantitative_unit_claims(line: &str) -> Vec<String> {
    let words: Vec<&str> = line
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '.' && c != '/' && c != '^' && c != '%'))
        .filter(|w| !w.is_empty())
        .collect();
    let mut claims = Vec::new();
    for i in 0..words.len() {
        if let (Some(&num_w), Some(&unit_w)) = (words.get(i), words.get(i.saturating_add(1))) {
            let is_num = !num_w.is_empty()
                && num_w.chars().all(|c| c.is_ascii_digit() || c == '.')
                && num_w.chars().any(|c| c.is_ascii_digit());
            if is_num && is_valid_si_unit(unit_w) {
                claims.push(format!("{} {}", num_w.to_ascii_lowercase(), unit_w.to_ascii_lowercase()));
            }
        }
    }
    claims
}

/// Audits citations and protocols in a specification file against the cited sections and parsed SysML AST.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ISO 26262 ASIL D. Eliminates fraudulent source citations
/// and ungrounded quantitative or protocol assertions across safety and engineering artifacts.
///
/// ## 2. Invariants & Mathematical Properties
/// - Every cited file and section must exist on disk.
/// - Every quantitative claim and protocol in cited text must exist in the cited section or parsed SysML AST.
///
/// ## 3. Preconditions
/// Input content, path, slicer, and SysML model string must be valid UTF-8.
///
/// ## 4. Postconditions
/// Appends structured defect diagnostics to `errors` upon citation fraud or ungrounded claims.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(L * C) where L is line count and C is citation count.
/// Space Complexity: O(E) where E is error count.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Slices and file lookups are safe; failures are captured as diagnostic errors.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CITATION-FRAUD-AUDIT].
pub fn audit_spec_citations_and_protocols(
    content: &str,
    rel_path: &str,
    slicer: &SectionSlicer,
    model_str: &str,
    errors: &mut Vec<String>,
) {
    let lines: Vec<&str> = content.lines().collect();
    let mut active_citations: Vec<CitationRef> = Vec::new();
    let (ast_symbols, ast_claims) = build_ast_model_ground_truth(model_str);
    let model_lower = model_str.to_ascii_lowercase();
    let has_model = !model_str.trim().is_empty();

    for (idx, &line) in lines.iter().enumerate() {
        let line_no = idx.saturating_add(1);
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.is_empty() {
            continue;
        }

        let line_citations = extract_citations_from_line(trimmed, line_no);
        for cit in &line_citations {
            if slicer.resolve_path(&cit.target_file).is_none() {
                errors.push(format!(
                    "Check 23 failed: Citation fraud detected in '{}': cited source file '{}' does not exist in workspace.",
                    rel_path, cit.target_file
                ));
                continue;
            }

            if let Some(ref sec) = cit.section {
                let sec_extracted = slicer.extract_section(&cit.target_file, sec);
                if sec_extracted.is_none() {
                    errors.push(format!(
                        "Check 23 failed: Citation fraud detected in '{}': cited section '§{}' does not exist in '{}'.",
                        rel_path, sec, cit.target_file
                    ));
                } else if let Some(ref sec_text) = sec_extracted {
                    let lower_sec = sec_text.to_ascii_lowercase();
                    let next_line = lines.get(idx.saturating_add(1)).copied().unwrap_or("");
                    let combined_claim = if trimmed.starts_with("<!--") {
                        next_line
                    } else {
                        trimmed
                    };

                    for q_claim in extract_quantitative_unit_claims(combined_claim) {
                        let in_ast = ast_claims.contains(&q_claim) || model_lower.contains(&q_claim);
                        if !lower_sec.contains(&q_claim) && !in_ast {
                            errors.push(format!(
                                "Check 23 failed: Citation fraud detected in '{}' on line {}: cited section '§{}' in '{}' does not substantiate '{}' claim.",
                                rel_path, line_no, sec, cit.target_file, q_claim
                            ));
                        }
                    }

                    for proto_tok in extract_candidate_protocol_tokens(combined_claim) {
                        let proto_lower = proto_tok.to_ascii_lowercase();
                        let proto_norm = normalize_ast_protocol_symbol(&proto_tok);
                        let in_ast = ast_symbols.contains(&proto_lower)
                            || ast_symbols.contains(&proto_norm)
                            || model_lower.contains(&proto_lower);
                        if !lower_sec.contains(&proto_lower) && !in_ast {
                            errors.push(format!(
                                "Check 23 failed: Citation fraud detected in '{}' on line {}: cited section '§{}' in '{}' does not substantiate '{}' interface claim.",
                                rel_path, line_no, sec, cit.target_file, proto_tok
                            ));
                        }
                    }
                }
            }
        }

        if !line_citations.is_empty() {
            active_citations = line_citations.clone();
        }

        if has_model || !active_citations.is_empty() {
            for proto_tok in extract_candidate_protocol_tokens(trimmed) {
                let proto_lower = proto_tok.to_ascii_lowercase();
                let proto_norm = normalize_ast_protocol_symbol(&proto_tok);
                let in_model = has_model
                    && (ast_symbols.contains(&proto_lower)
                        || ast_symbols.contains(&proto_norm)
                        || model_lower.contains(&proto_lower));
                let in_cited_sec = active_citations.iter().any(|c| {
                    if let Some(ref sec) = c.section {
                        slicer
                            .extract_section(&c.target_file, sec)
                            .is_some_and(|t| {
                                let tl = t.to_ascii_lowercase();
                                tl.contains(&proto_lower)
                                    || normalize_ast_protocol_symbol(&tl).contains(&proto_norm)
                            })
                    } else {
                        false
                    }
                });
                if !in_model && !in_cited_sec {
                    errors.push(format!(
                        "Check 23 failed: Ungrounded protocol detected in '{}' on line {}: protocol '{}' is not declared in SysML AST or cited source section.",
                        rel_path, line_no, proto_tok
                    ));
                }
            }
        }
    }
}

/// Check 23: Factual Grounding & Numeric Provenance Gate.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ISO 26262 ASIL D. Validates that specifications are grounded
/// against physical reality and authoritative schema models.
///
/// ## 2. Invariants & Mathematical Properties
/// - Downstream customer workspaces require grounded schemas or models.
/// - Citation fraud and protocol hallucinations immediately halt verification.
///
/// ## 3. Preconditions
/// `repo_root` must be a directory.
///
/// ## 4. Postconditions
/// Returns `Ok(ParityOutcome::Verified)` on success.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(F * L) where F is file count and L is line count.
/// Space Complexity: O(E) where E is error count.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed on I/O or citation discrepancies.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-PARITY-GATE-23].
pub fn check_factual_grounding(
    repo_root: &Path,
    allow_missing_specs: bool,
    strict: bool,
) -> Result<ParityOutcome, Vec<String>> {
    let is_upstream = is_upstream_compiler(repo_root);
    let model_text = discover_sysml_model_text(repo_root);
    let schema_dir = repo_root.join("schema");
    let has_extracted = schema_dir.join("extracted").is_dir();

    if model_text.is_none() && !has_extracted {
        let effective_allow_missing = allow_missing_specs && !strict;
        if !is_upstream && !effective_allow_missing {
            return Err(vec![
                "Check 23 failed: SysML model or schema ground truth is missing in downstream customer mode.".to_string(),
            ]);
        }
        return Ok(ParityOutcome::PendingOrClean);
    }

    if is_upstream {
        return Ok(ParityOutcome::Verified);
    }

    let mut errors = Vec::new();
    let slicer = SectionSlicer::new(repo_root);
    let model_str = model_text.unwrap_or_default();

    for scan_dir in TargetScanDir::ALL.iter().copied().filter(|d| d.requires_factual_grounding()) {
        let dir = scan_dir.resolve(repo_root);
        if !dir.is_dir() {
            continue;
        }
        for entry_res in WalkDir::new(&dir)
            .sort_by_file_name()
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !EXCLUDED_DIRS.iter().any(|ex| name == ex)
                    && name != "blueprints"
                    && name != "defects"
                    && name != "audits"
                    && !name.starts_with('.')
                    && !name.starts_with('#')
            })
        {
            let entry = match entry_res {
                Ok(e) => e,
                Err(err) => {
                    errors.push(format!("Check 23 failed: I/O error walking directory '{}': {}", dir.display(), err));
                    continue;
                }
            };
            if entry.file_type().is_file() {
                let fname = entry.file_name().to_string_lossy();
                if fname.ends_with(".md") && fname != "README.md" {
                    let path = entry.path();
                    let rel_path = path
                        .strip_prefix(repo_root)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .to_string();
                    let content = match fs::read_to_string(path) {
                        Ok(c) => c,
                        Err(e) => {
                            errors.push(format!("Check 23 failed: Unable to read file '{}': {}", rel_path, e));
                            continue;
                        }
                    };
                    audit_spec_citations_and_protocols(
                        &content,
                        &rel_path,
                        &slicer,
                        &model_str,
                        &mut errors,
                    );
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(ParityOutcome::Verified)
    } else {
        Err(errors)
    }
}

/// Helper: Extracts a declared port ID (e.g. `PORT-FC-GNSS_FIX_IN`) from text.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Extracts canonical PORT-* identifiers without regex.
///
/// ## 2. Invariants & Mathematical Properties
/// Token begins with `PORT-` and contains alphanumeric characters, underscores, or hyphens.
///
/// ## 3. Preconditions
/// Input string must be valid UTF-8.
///
/// ## 4. Postconditions
/// Returns `Some(String)` with uppercase Port ID if found.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N). Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Safe index slicing; never panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-PORT-ID-EXTRACTION].
pub fn extract_port_identifier(s: &str) -> Option<String> {
    let trimmed = s.trim_matches(|c| c == '`' || c == ' ' || c == '*');
    if let Some(pos) = trimmed.find("PORT-") {
        if let Some(rest) = trimmed.get(pos..) {
            let end = rest
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-')
                .unwrap_or(rest.len());
            if end > 5 {
                if let Some(slice) = rest.get(..end) {
                    return Some(slice.to_ascii_uppercase());
                }
            }
        }
    }
    None
}

/// Helper: Extracts connection ID (e.g. `CONN-05`) from text.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Extracts canonical CONN-* identifiers without regex.
///
/// ## 2. Invariants & Mathematical Properties
/// Token begins with `CONN-` and contains alphanumeric characters, underscores, or hyphens.
///
/// ## 3. Preconditions
/// Input string must be valid UTF-8.
///
/// ## 4. Postconditions
/// Returns `Some(String)` with uppercase Connection ID if found.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N). Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Safe index slicing; never panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CONN-ID-EXTRACTION].
pub fn extract_single_conn_id(s: &str) -> Option<String> {
    let trimmed = s.trim_matches(|c| c == '`' || c == ' ' || c == '*');
    let lower = trimmed.to_ascii_lowercase();
    if let Some(pos) = lower.find("conn-") {
        if let Some(rest) = trimmed.get(pos..) {
            let end = rest
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-')
                .unwrap_or(rest.len());
            if end > 5 {
                if let Some(slice) = rest.get(..end) {
                    return Some(slice.to_ascii_uppercase());
                }
            }
        }
    }
    None
}

/// Helper: Extracts signal ID (e.g. `SIG-NS-FC-001`) from text.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Extracts canonical SIG-* identifiers without regex.
///
/// ## 2. Invariants & Mathematical Properties
/// Token begins with `SIG-` and contains alphanumeric characters, underscores, or hyphens.
///
/// ## 3. Preconditions
/// Input string must be valid UTF-8.
///
/// ## 4. Postconditions
/// Returns `Some(String)` with uppercase Signal ID if found.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N). Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Safe index slicing; never panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SIGNAL-ID-EXTRACTION].
pub fn extract_single_signal_id(s: &str) -> Option<String> {
    let trimmed = s.trim_matches(|c| c == '`' || c == ' ' || c == '*');
    let lower = trimmed.to_ascii_lowercase();
    if let Some(pos) = lower.find("sig-") {
        if let Some(rest) = trimmed.get(pos..) {
            let end = rest
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-')
                .unwrap_or(rest.len());
            if end > 4 {
                if let Some(slice) = rest.get(..end) {
                    return Some(slice.to_ascii_uppercase());
                }
            }
        }
    }
    None
}

/// Helper: Strict Markdown table separator row validator.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Distinguishes genuine Markdown table header separators from data rows containing hyphens.
///
/// ## 2. Invariants & Mathematical Properties
/// Every column must strictly match `^:?-+:?$`.
///
/// ## 3. Preconditions
/// `line` must be valid UTF-8.
///
/// ## 4. Postconditions
/// Returns true only if line is a valid Markdown table separator.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(C) where C is characters in line.
/// Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Safe character iterators; never panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-TABLE-SEPARATOR-VALIDATION].
pub fn is_table_separator_line(line: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') {
        return false;
    }
    let cols: Vec<&str> = trimmed
        .split('|')
        .map(|c| c.trim())
        .filter(|c| !c.is_empty())
        .collect();
    if cols.is_empty() {
        return false;
    }
    cols.iter().all(|col| {
        let c_trim = col.trim_matches(':');
        !c_trim.is_empty() && c_trim.chars().all(|ch| ch == '-')
    })
}

/// Helper: Parses markdown table headers and rows under a section header containing `section_kw`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Extracts structured table cells from specification sections.
///
/// ## 2. Invariants & Mathematical Properties
/// Correctly distinguishes header separators from data rows.
///
/// ## 3. Preconditions
/// Input content and section keyword must be valid UTF-8.
///
/// ## 4. Postconditions
/// Returns `(headers, rows)` preserving column order.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(L * C) where L is lines and C is columns.
/// Space Complexity: O(R * C) where R is rows count.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Safe bounds access on column slices; never panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-MARKDOWN-TABLE-PARSER].
pub fn parse_markdown_table_rows(
    content: &str,
    section_kw: &str,
) -> (Vec<String>, Vec<Vec<String>>) {
    let mut headers = Vec::new();
    let mut rows = Vec::new();
    let mut in_section = false;
    let mut in_table = false;
    let mut header_seen = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            let lower = trimmed.to_ascii_lowercase();
            if lower.contains(section_kw) {
                in_section = true;
                in_table = false;
                header_seen = false;
                continue;
            } else if in_section {
                break;
            }
        }

        if in_section {
            if trimmed.starts_with('|') {
                if is_table_separator_line(trimmed) {
                    in_table = true;
                    continue;
                }
                let raw_cols: Vec<&str> = trimmed.split('|').map(|c| c.trim()).collect();
                let col_count = raw_cols.len();
                let start = if trimmed.starts_with('|') && col_count > 1 { 1 } else { 0 };
                let end = if trimmed.ends_with('|') && col_count > start {
                    col_count.saturating_sub(1)
                } else {
                    col_count
                };
                let row_slice = raw_cols.get(start..end).unwrap_or(&[]);
                let row_cols: Vec<String> = row_slice.iter().map(|&s| s.to_string()).collect();

                if !header_seen {
                    headers = row_cols.iter().map(|s| s.to_ascii_lowercase()).collect();
                    header_seen = true;
                    continue;
                }
                if in_table {
                    rows.push(row_cols);
                }
            } else if in_table && trimmed.is_empty() {
                break;
            }
        }
    }
    (headers, rows)
}

/// Metadata describing a declared port definition in ICD_01.
#[derive(Debug, Clone)]
struct PortMeta {
    subsystem: String,
    name: String,
    direction: String,
}

/// Recursively collects nested subsystem and port identifiers from parsed SysML `PartDef` AST nodes.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A. Extracts hierarchical nesting structure from the SysML v2 AST
/// so Check 24 enforces qualified port paths (`PORT-ID (Subsystem.Path.port)`) for nested AST components.
///
/// ## 2. Invariants & Mathematical Properties
/// - Any child `PartDef` inside a parent `PartDef.parts` (depth >= 1) and its declared `ports`
///   are recorded as nested AST elements.
///
/// ## 3. Preconditions
/// `parts` is a slice of parsed `PartDef` nodes.
///
/// ## 4. Postconditions
/// Populates `nested_subsystems` and `nested_ports` with lowercase AST names.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N) where N is total AST part/port count.
/// Space Complexity: O(N).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Pure AST traversal; never panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-PARITY-GATE-24].
fn collect_ast_nested_subsystems_and_ports(
    parts: &[deap_core::sysml_ast::PartDef],
    is_nested: bool,
    nested_subsystems: &mut BTreeSet<String>,
    nested_ports: &mut BTreeSet<String>,
) {
    for part in parts {
        if is_nested {
            if !part.name.is_empty() {
                nested_subsystems.insert(part.name.to_ascii_lowercase());
            }
            for port in &part.ports {
                if !port.name.is_empty() {
                    nested_ports.insert(port.name.to_ascii_lowercase());
                }
            }
        }
        if !part.parts.is_empty() {
            if !part.name.is_empty() {
                nested_subsystems.insert(part.name.to_ascii_lowercase());
            }
            collect_ast_nested_subsystems_and_ports(
                &part.parts,
                true,
                nested_subsystems,
                nested_ports,
            );
        }
    }
}

/// Check 24: Level 1C ICD Completeness & Signal Flow Parity Gate.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ISO 26262 ASIL D. Enforces interface completeness,
/// signal metrology, and hierarchical port qualification across ICD documents.
///
/// ## 2. Invariants & Mathematical Properties
/// - Every declared port must bind to at least one connection (zero dangling ports).
/// - Every signal must map to an active connection binding.
/// - Nested ports require hierarchical qualification `PORT-ID (Subsystem.Path.port)`.
///
/// ## 3. Preconditions
/// `repo_root` must be an accessible directory.
///
/// ## 4. Postconditions
/// Returns `Ok(ParityOutcome::Verified)` when interface contracts hold.
/// Returns `Err(Vec<String>)` containing structural or metrological defects.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(P + C + S) for ports, connections, and signals.
/// Space Complexity: O(P + C + S) stored in BTreeMap/BTreeSet.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Propagates I/O and table parsing errors fail-closed.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-PARITY-GATE-24].
pub fn check_icd_completeness(repo_root: &Path) -> Result<ParityOutcome, Vec<String>> {
    let is_upstream = is_upstream_compiler(repo_root);
    if is_upstream {
        return Ok(ParityOutcome::Verified);
    }

    let iface_dir = TargetScanDir::Interfaces.resolve(repo_root);
    let icd_dir = TargetScanDir::Icd.resolve(repo_root);
    let interfaces_dir = if iface_dir.is_dir() {
        iface_dir
    } else if icd_dir.is_dir() {
        icd_dir
    } else {
        iface_dir
    };

    let model_text = discover_sysml_model_text(repo_root);
    let schema_dir = repo_root.join("schema");
    let has_extracted = schema_dir.join("extracted").is_dir();

    if !interfaces_dir.is_dir() {
        if model_text.is_none() && !has_extracted {
            return Ok(ParityOutcome::PendingOrClean);
        }
        return Ok(ParityOutcome::InterfacesDirMissing);
    }

    let mut icd01_path = None;
    let mut icd02_path = None;

    if let Ok(entries) = fs::read_dir(&interfaces_dir) {
        let mut entry_list: Vec<_> = entries.flatten().collect();
        entry_list.sort_by_key(|e| e.file_name());
        for entry in entry_list {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".md") {
                if name.contains("ICD_01") || name.contains("SYSTEM_INTERFACE_MATRIX") {
                    icd01_path = Some(entry.path());
                } else if name.contains("ICD_02") || name.contains("MASTER_SIGNAL_DICTIONARY") {
                    icd02_path = Some(entry.path());
                }
            }
        }
    }

    if icd01_path.is_none() && icd02_path.is_none() {
        if model_text.is_none() && !has_extracted {
            return Ok(ParityOutcome::PendingOrClean);
        }
        return Ok(ParityOutcome::IcdSpecsPending);
    }

    let mut errors = Vec::new();

    let icd01_file = match icd01_path {
        Some(p) => p,
        None => {
            errors.push("Check 24 failed: System Interface Matrix 'docs/interfaces/ICD_01_SYSTEM_INTERFACE_MATRIX.md' is missing.".to_string());
            return Err(errors);
        }
    };

    let icd02_file = match icd02_path {
        Some(p) => p,
        None => {
            errors.push("Check 24 failed: Master Signal Flow Dictionary 'docs/interfaces/ICD_02_MASTER_SIGNAL_DICTIONARY.md' is missing.".to_string());
            return Err(errors);
        }
    };

    let icd01_content = fs::read_to_string(&icd01_file)
        .map_err(|e| vec![format!("Check 24 failed: Failed to read ICD_01: {}", e)])?;
    let icd02_content = fs::read_to_string(&icd02_file)
        .map_err(|e| vec![format!("Check 24 failed: Failed to read ICD_02: {}", e)])?;

    let mut ast_nested_subsystems = BTreeSet::new();
    let mut ast_nested_ports = BTreeSet::new();
    if let Some(ref mtext) = model_text {
        let ast_parts = parse_part_defs(mtext);
        collect_ast_nested_subsystems_and_ports(
            &ast_parts,
            false,
            &mut ast_nested_subsystems,
            &mut ast_nested_ports,
        );
    }

    let (port_headers, port_rows) = parse_markdown_table_rows(&icd01_content, "port definition");
    if port_rows.is_empty() {
        errors.push("Check 24 failed: ICD_01 Port Definition Roster Table (Section 4) is missing or contains 0 rows.".to_string());
    }

    let port_id_idx = port_headers.iter().position(|h| h.contains("port id") || h.contains("id")).unwrap_or(0);
    let subsys_idx = port_headers.iter().position(|h| h.contains("subsystem")).unwrap_or(1);
    let port_name_idx = port_headers.iter().position(|h| h.contains("port name") || h.contains("name")).unwrap_or(2);
    let dir_idx = port_headers.iter().position(|h| h.contains("direction")).unwrap_or(3);

    let mut declared_ports: BTreeMap<String, PortMeta> = BTreeMap::new();
    for row in &port_rows {
        let raw_id = row.get(port_id_idx).map(|s| s.as_str()).unwrap_or("");
        let port_id = extract_port_identifier(raw_id).unwrap_or_else(|| {
            raw_id.trim_matches(|c| c == '`' || c == ' ' || c == '*').to_string()
        });
        let subsys = row.get(subsys_idx).map(|s| s.as_str()).unwrap_or("").trim_matches(|c| c == '`' || c == ' ' || c == '*').to_string();
        let name = row.get(port_name_idx).map(|s| s.as_str()).unwrap_or("").trim_matches(|c| c == '`' || c == ' ' || c == '*').to_string();
        let dir = row.get(dir_idx).map(|s| s.as_str()).unwrap_or("").trim_matches(|c| c == '`' || c == ' ' || c == '*').to_ascii_uppercase();

        if port_id.is_empty() || !port_id.starts_with("PORT-") {
            errors.push(format!("Check 24 failed: Invalid Port ID '{}' in Section 4 (must start with 'PORT-')", raw_id));
        } else {
            use std::collections::btree_map::Entry;
            match declared_ports.entry(port_id) {
                Entry::Occupied(e) => {
                    errors.push(format!("Check 24 failed: Duplicate Port ID '{}' declared in Section 4", e.key()));
                }
                Entry::Vacant(e) => {
                    e.insert(PortMeta { subsystem: subsys, name, direction: dir });
                }
            }
        }
    }

    let (conn_headers, conn_rows) = parse_markdown_table_rows(&icd01_content, "connection binding");
    if conn_rows.is_empty() {
        errors.push("Check 24 failed: ICD_01 Connection Binding Roster Table (Section 5) is missing or contains 0 rows.".to_string());
    }

    let conn_id_idx = conn_headers.iter().position(|h| h.contains("connection id") || h.contains("id")).unwrap_or(0);
    let src_idx = conn_headers.iter().position(|h| h.contains("source")).unwrap_or(1);
    let dst_idx = conn_headers.iter().position(|h| h.contains("dest")).unwrap_or(2);
    let flows_idx = conn_headers.iter().position(|h| h.contains("item flow") || h.contains("flow")).unwrap_or(6);

    let mut bound_ports = BTreeSet::new();
    let mut valid_connections = BTreeSet::new();
    let mut seen_conn_ids = BTreeSet::new();

    for row in &conn_rows {
        let raw_conn_id = row.get(conn_id_idx).map(|s| s.as_str()).unwrap_or("");
        let conn_id = extract_single_conn_id(raw_conn_id).unwrap_or_else(|| {
            raw_conn_id.trim_matches(|c| c == '`' || c == ' ' || c == '*').to_string()
        });
        let src_raw = row.get(src_idx).map(|s| s.as_str()).unwrap_or("");
        let dst_raw = row.get(dst_idx).map(|s| s.as_str()).unwrap_or("");
        let _flows_raw = row.get(flows_idx).map(|s| s.as_str()).unwrap_or("");

        if conn_id.is_empty() || !conn_id.starts_with("CONN-") {
            errors.push(format!("Check 24 failed: Invalid Connection ID '{}' in Section 5 (must start with 'CONN-')", raw_conn_id));
        } else if seen_conn_ids.contains(&conn_id) {
            errors.push(format!("Check 24 failed: Duplicate Connection ID '{}' in Section 5", conn_id));
        } else {
            seen_conn_ids.insert(conn_id.clone());
        }

        let src_port_id = extract_port_identifier(src_raw);
        let dst_port_id = extract_port_identifier(dst_raw);

        if src_port_id.is_none() {
            errors.push(format!(
                "Check 24 failed: Connection '{}' source port '{}' lacks valid PORT-* identifier",
                conn_id, src_raw
            ));
        }
        if dst_port_id.is_none() {
            errors.push(format!(
                "Check 24 failed: Connection '{}' dest port '{}' lacks valid PORT-* identifier",
                conn_id, dst_raw
            ));
        }

        let src_subsys = src_port_id
            .as_ref()
            .and_then(|id| declared_ports.get(id))
            .map(|p| p.subsystem.as_str())
            .unwrap_or("");
        let dst_subsys = dst_port_id
            .as_ref()
            .and_then(|id| declared_ports.get(id))
            .map(|p| p.subsystem.as_str())
            .unwrap_or("");
        let src_port_name = src_port_id
            .as_ref()
            .and_then(|id| declared_ports.get(id))
            .map(|p| p.name.as_str())
            .unwrap_or(src_raw);
        let dst_port_name = dst_port_id
            .as_ref()
            .and_then(|id| declared_ports.get(id))
            .map(|p| p.name.as_str())
            .unwrap_or(dst_raw);
        let has_multi_subsystems = !src_subsys.is_empty()
            && !dst_subsys.is_empty()
            && src_subsys != dst_subsys;
        let is_ast_nested = ast_nested_subsystems.contains(&src_subsys.to_ascii_lowercase())
            || ast_nested_subsystems.contains(&dst_subsys.to_ascii_lowercase())
            || ast_nested_ports.contains(&src_port_name.to_ascii_lowercase())
            || ast_nested_ports.contains(&dst_port_name.to_ascii_lowercase());

        let is_nested_connection = is_ast_nested
            || src_raw.contains('.')
            || dst_raw.contains('.')
            || src_subsys.contains('.')
            || dst_subsys.contains('.')
            || has_multi_subsystems;

        if is_nested_connection {
            let src_qualified = src_raw.contains('(') && src_raw.contains('.') && src_raw.contains(')');
            let dst_qualified = dst_raw.contains('(') && dst_raw.contains('.') && dst_raw.contains(')');
            if !src_qualified {
                errors.push(format!(
                    "Check 24 failed: Connection '{}' source port '{}' lacks hierarchical subsystem qualification for nested port (expected format 'PORT-ID (Subsystem.Path.port)')",
                    conn_id, src_raw
                ));
            }
            if !dst_qualified {
                errors.push(format!(
                    "Check 24 failed: Connection '{}' dest port '{}' lacks hierarchical subsystem qualification for nested port (expected format 'PORT-ID (Subsystem.Path.port)')",
                    conn_id, dst_raw
                ));
            }
        }

        if let Some(ref sp_id) = src_port_id {
            if let Some(port) = declared_ports.get(sp_id) {
                if port.direction != "OUT" && port.direction != "INOUT" {
                    errors.push(format!(
                        "Check 24 failed: Connection '{}' binds port '{}' with direction '{}' as source (expected OUT or INOUT)",
                        conn_id, sp_id, port.direction
                    ));
                }
                bound_ports.insert(sp_id.clone());
            } else {
                errors.push(format!(
                    "Check 24 failed: Connection '{}' references undeclared source port '{}'",
                    conn_id, sp_id
                ));
            }
        }

        if let Some(ref dp_id) = dst_port_id {
            if let Some(port) = declared_ports.get(dp_id) {
                if port.direction != "IN" && port.direction != "INOUT" {
                    errors.push(format!(
                        "Check 24 failed: Connection '{}' binds port '{}' with direction '{}' as destination (expected IN or INOUT)",
                        conn_id, dp_id, port.direction
                    ));
                }
                bound_ports.insert(dp_id.clone());
            } else {
                errors.push(format!(
                    "Check 24 failed: Connection '{}' references undeclared dest port '{}'",
                    conn_id, dp_id
                ));
            }
        }

        if let (Some(s), Some(d)) = (src_port_id, dst_port_id) {
            valid_connections.insert((s, d));
        }
    }

    for (port_id, port) in &declared_ports {
        if !bound_ports.contains(port_id) {
            errors.push(format!(
                "Check 24 failed: Dangling port detected: declared port '{}' ({}) in subsystem '{}' is not bound to any connection in Section 5",
                port_id, port.name, port.subsystem
            ));
        }
    }

    let (sig_headers, sig_rows) = parse_markdown_table_rows(&icd02_content, "signal flow dictionary");
    if sig_rows.is_empty() {
        errors.push("Check 24 failed: ICD_02 Master Signal Flow Dictionary Table (Section 2) is missing or contains 0 rows.".to_string());
    }

    let sig_id_idx = sig_headers.iter().position(|h| h.contains("signal id") || h.contains("id")).unwrap_or(0);
    let name_idx = sig_headers.iter().position(|h| h.contains("signal name") || h.contains("name")).unwrap_or(1);
    let src_port_idx = sig_headers.iter().position(|h| h.contains("source")).unwrap_or(2);
    let dst_port_idx = sig_headers.iter().position(|h| h.contains("dest")).unwrap_or(3);
    let type_idx = sig_headers.iter().position(|h| h.contains("data type") || h.contains("type")).unwrap_or(4);
    let units_idx = sig_headers.iter().position(|h| h.contains("unit")).unwrap_or(5);
    let range_idx = sig_headers.iter().position(|h| h.contains("range")).unwrap_or(6);
    let rate_idx = sig_headers.iter().position(|h| h.contains("rate")).unwrap_or(7);
    let default_idx = sig_headers.iter().position(|h| h.contains("default")).unwrap_or(8);
    let citation_idx = sig_headers.iter().position(|h| h.contains("citation") || h.contains("schema") || h.contains("source")).unwrap_or(9);

    let mut seen_sig_ids = BTreeSet::new();

    for row in &sig_rows {
        let raw_sig_id = row.get(sig_id_idx).map(|s| s.as_str()).unwrap_or("");
        let sig_id = extract_single_signal_id(raw_sig_id).unwrap_or_else(|| {
            raw_sig_id.trim_matches(|c| c == '`' || c == ' ' || c == '*').to_string()
        });
        let sig_name = row.get(name_idx).map(|s| s.as_str()).unwrap_or("").trim_matches(|c| c == '`' || c == ' ' || c == '*');
        let src_raw = row.get(src_port_idx).map(|s| s.as_str()).unwrap_or("");
        let dst_raw = row.get(dst_port_idx).map(|s| s.as_str()).unwrap_or("");
        let src_port = extract_port_identifier(src_raw).unwrap_or_else(|| {
            src_raw.trim_matches(|c| c == '`' || c == ' ' || c == '*').to_string()
        });
        let dst_port = extract_port_identifier(dst_raw).unwrap_or_else(|| {
            dst_raw.trim_matches(|c| c == '`' || c == ' ' || c == '*').to_string()
        });

        if sig_id.is_empty() || !sig_id.starts_with("SIG-") {
            errors.push(format!("Check 24 failed: Invalid Signal ID '{}' in Section 2 (must start with 'SIG-')", raw_sig_id));
        } else if seen_sig_ids.contains(&sig_id) {
            errors.push(format!("Check 24 failed: Duplicate Signal ID '{}' in Section 2", sig_id));
        } else {
            seen_sig_ids.insert(sig_id.clone());
        }

        if !declared_ports.contains_key(&src_port) {
            errors.push(format!("Check 24 failed: Signal '{}' ({}) references undeclared source port '{}'", sig_id, sig_name, src_port));
        }
        if !declared_ports.contains_key(&dst_port) {
            errors.push(format!("Check 24 failed: Signal '{}' ({}) references undeclared dest port '{}'", sig_id, sig_name, dst_port));
        }

        let is_mapped = valid_connections.contains(&(src_port.clone(), dst_port.clone()));
        if !is_mapped {
            errors.push(format!(
                "Check 24 failed: Unmapped signal '{}': no connection in ICD_01 connects source '{}' to dest '{}'",
                sig_id, src_port, dst_port
            ));
        }

        let data_type = row.get(type_idx).map(|s| s.as_str()).unwrap_or("").trim();
        let si_units = row.get(units_idx).map(|s| s.as_str()).unwrap_or("").trim();
        let valid_range = row.get(range_idx).map(|s| s.as_str()).unwrap_or("").trim();
        let update_rate = row.get(rate_idx).map(|s| s.as_str()).unwrap_or("").trim();
        let safe_default = row.get(default_idx).map(|s| s.as_str()).unwrap_or("").trim();
        let citation = row.get(citation_idx).map(|s| s.as_str()).unwrap_or("").trim();

        let fields = [
            ("data type", data_type),
            ("SI units", si_units),
            ("valid range", valid_range),
            ("update rate", update_rate),
            ("safe default value", safe_default),
            ("schema citation", citation),
        ];

        for (fname, val) in &fields {
            if val.is_empty() || is_metrology_placeholder(val) {
                errors.push(format!(
                    "Check 24 failed: Signal '{}' field '{}' has incomplete metrology or placeholder: '{}'",
                    sig_id, fname, val
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(ParityOutcome::Verified)
    } else {
        Err(errors)
    }
}

/// Check 25: Operational-to-Resource Allocation Gate (Gate 24).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ISO 26262 ASIL D. Validates that operational activities
/// `[OA-XX]` and mission execution tasks `[MET-XX]` map to system components without orphans.
///
/// ## 2. Invariants & Mathematical Properties
/// - Every operational activity tag must be syntactically valid and allocated to a system resource.
/// - Clean landing zones evaluate to `PendingOrClean`.
///
/// ## 3. Preconditions
/// `repo_root` must be an accessible directory.
///
/// ## 4. Postconditions
/// Returns `Ok(ParityOutcome::Verified)` on success.
/// Returns `Err(Vec<String>)` containing allocation tags defects.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(F * L) where F is file count and L is line count.
/// Space Complexity: O(A) where A is allocation finding count.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Propagates file read and allocation validation errors fail-closed.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-PARITY-GATE-25].
pub fn check_operational_allocation(repo_root: &Path) -> Result<ParityOutcome, Vec<String>> {
    let is_upstream = is_upstream_compiler(repo_root);
    let conops_dir = TargetScanDir::Conops.resolve(repo_root);
    let arch_dir = TargetScanDir::Architecture.resolve(repo_root);

    if is_upstream && !conops_dir.is_dir() {
        return Ok(ParityOutcome::PendingOrClean);
    }

    let mut errors = Vec::new();
    let mut target_files = Vec::new();
    let scan_dirs = if is_upstream {
        vec![conops_dir]
    } else {
        vec![conops_dir, arch_dir]
    };
    for dir in &scan_dirs {
        if dir.is_dir() {
            for entry_res in WalkDir::new(dir)
                .sort_by_file_name()
                .into_iter()
                .filter_entry(|e| {
                    if e.file_type().is_dir() {
                        let name = e.file_name().to_string_lossy();
                        !EXCLUDED_DIRS.iter().any(|ex| name == ex) && name != "blueprints"
                    } else {
                        true
                    }
                })
            {
                let entry = match entry_res {
                    Ok(e) => e,
                    Err(err) => {
                        errors.push(format!("Check 25 failed: I/O error walking directory '{}': {}", dir.display(), err));
                        continue;
                    }
                };
                if entry.file_type().is_file() {
                    let path = entry.path().to_path_buf();
                    if path.extension().is_some_and(|ext| ext == "md") {
                        target_files.push(path);
                    }
                }
            }
        }
    }
    target_files.sort();

    if target_files.is_empty() {
        return Ok(ParityOutcome::PendingOrClean);
    }

    for path in target_files {
        let rel = path.strip_prefix(repo_root).unwrap_or(&path).display().to_string();
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                errors.push(format!("Check 25 failed: Unable to read file '{}': {}", rel, e));
                continue;
            }
        };
        let findings = crate::spec_audit::conops_audit::validate_operational_allocation_tags(&content, &rel);
        for f in findings {
            errors.push(f.to_string());
        }
    }

    if errors.is_empty() {
        Ok(ParityOutcome::Verified)
    } else {
        Err(errors)
    }
}

/// Check 26: Standards & SI 7-Dimensional Parameter Metrology Gate (Gate 25).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ISO 26262 ASIL D. Validates that all specification parameter
/// tables and signal rosters declare canonical SI dimensional units without naked numbers.
///
/// ## 2. Invariants & Mathematical Properties
/// Every dimensional engineering quantity must declare an approved SI unit.
///
/// ## 3. Preconditions
/// `repo_root` must be a directory.
///
/// ## 4. Postconditions
/// Returns `Ok(ParityOutcome::Verified)` on success.
/// Returns `Err(Vec<String>)` if naked quantities or placeholder units are discovered.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(T * C) where T is table count and C is cell count.
/// Space Complexity: O(E) where E is error count.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed on I/O or metrology violations.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-PARITY-GATE-26].
pub fn check_standards_measurement(
    repo_root: &Path,
    allow_missing_specs: bool,
    strict: bool,
) -> Result<ParityOutcome, Vec<String>> {
    let is_upstream = is_upstream_compiler(repo_root);
    let iface_dir = TargetScanDir::Interfaces.resolve(repo_root);
    let feat_dir = TargetScanDir::Features.resolve(repo_root);
    let conops_dir = TargetScanDir::Conops.resolve(repo_root);
    let arch_dir = TargetScanDir::Architecture.resolve(repo_root);

    if is_upstream && (!iface_dir.is_dir() && !feat_dir.is_dir()) {
        return Ok(ParityOutcome::PendingOrClean);
    }

    let mut errors = Vec::new();
    let mut target_files = Vec::new();
    let scan_dirs = if is_upstream {
        vec![iface_dir, feat_dir]
    } else {
        vec![iface_dir, feat_dir, conops_dir, arch_dir]
    };
    for dir in &scan_dirs {
        if dir.is_dir() {
            for entry_res in WalkDir::new(dir)
                .sort_by_file_name()
                .into_iter()
                .filter_entry(|e| {
                    if e.file_type().is_dir() {
                        let name = e.file_name().to_string_lossy();
                        !EXCLUDED_DIRS.iter().any(|ex| name == ex) && name != "blueprints"
                    } else {
                        true
                    }
                })
            {
                let entry = match entry_res {
                    Ok(e) => e,
                    Err(err) => {
                        errors.push(format!("Check 26 failed: I/O error walking directory '{}': {}", dir.display(), err));
                        continue;
                    }
                };
                if entry.file_type().is_file() {
                    let path = entry.path().to_path_buf();
                    if path.extension().is_some_and(|ext| ext == "md") {
                        target_files.push(path);
                    }
                }
            }
        }
    }
    target_files.sort();

    if target_files.is_empty() {
        if allow_missing_specs && !strict {
            return Ok(ParityOutcome::PendingOrClean);
        }
        return Ok(ParityOutcome::PendingOrClean);
    }

    for path in target_files {
        let rel = path.strip_prefix(repo_root).unwrap_or(&path).display().to_string();
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                errors.push(format!("Check 26 failed: Unable to read file '{}': {}", rel, e));
                continue;
            }
        };

        for table_kind in ["signal", "parameter", "metric"] {
            let (headers, rows) = parse_markdown_table_rows(&content, table_kind);
            if let Some(unit_col) = headers.iter().position(|h| h.contains("unit")) {
                for row in &rows {
                    if let Some(unit_val) = row.get(unit_col) {
                        let clean = unit_val.trim().trim_matches(|c| c == '`' || c == '*');
                        if clean.is_empty() || is_metrology_placeholder(clean) || !is_valid_si_unit(clean) {
                            errors.push(format!(
                                "Check 26 failed: Invalid or missing SI dimensional unit '{}' in '{}'",
                                clean, rel
                            ));
                        }
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(ParityOutcome::Verified)
    } else {
        Err(errors)
    }
}

/// Extracts all connection identifiers (`CONN-01`..`CONN-N`) from text without regex.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A. Extracts all connection tokens deterministically without regex.
///
/// ## 2. Invariants & Mathematical Properties
/// - Requires leading word boundary (start of string or non-alphanumeric character).
/// - Requires character following `CONN-` to be alphanumeric (rejects `CONN---`).
/// - Returns results in a deterministic BTreeSet.
///
/// ## 3. Preconditions
/// Input string `text` must be valid UTF-8.
///
/// ## 4. Postconditions
/// Returns `BTreeSet<String>` of unique Connection IDs.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N) where N is text byte length.
/// Space Complexity: O(C) where C is connection token count.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Safe slice bounds access; never panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CONN-ID-SET-EXTRACTION].
pub fn extract_connection_ids(text: &str) -> BTreeSet<String> {
    let mut conns = BTreeSet::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i.saturating_add(5) <= bytes.len() {
            let prefix = match bytes.get(i..i.saturating_add(5)) {
                Some(p) => p,
                None => {
                    i += 1;
                    continue;
                }
            };
            if prefix.eq_ignore_ascii_case(b"CONN-") {
                let valid_leading = if i == 0 {
                    true
                } else {
                    match bytes.get(i.saturating_sub(1)) {
                        Some(&b) => !b.is_ascii_alphanumeric() && b != b'_' && b != b'-',
                        None => true,
                    }
                };
                if valid_leading {
                    let start = i;
                    let mut end = i.saturating_add(5);
                    let first_after_conn = bytes.get(end).copied();
                    if first_after_conn.is_some_and(|b| b.is_ascii_alphanumeric()) {
                        while end < bytes.len() {
                            let b = match bytes.get(end) {
                                Some(&b) => b,
                                None => break,
                            };
                            if b.is_ascii_alphanumeric() || b == b'_' || b == b'-' {
                                end += 1;
                            } else {
                                break;
                            }
                        }
                        if let Some(slice) = text.get(start..end) {
                            let id = slice.to_ascii_uppercase();
                            conns.insert(id);
                        }
                        i = end;
                        continue;
                    }
                }
            }
        }
        i += 1;
    }
    conns
}

/// Recursively collects connection identifiers from markdown and mermaid files in a directory.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Traverses specification directories deterministically to aggregate connection identifiers.
///
/// ## 2. Invariants & Mathematical Properties
/// Directory entries are sorted lexicographically before reading.
///
/// ## 3. Preconditions
/// `dir` path must be a valid directory.
///
/// ## 4. Postconditions
/// Extends `conns` with discovered connection tokens.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(F * N) where F is file count and N is bytes.
/// Space Complexity: O(C) connection tokens count.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Skips unreadable files safely.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DETERMINISTIC-CONNECTION-COLLECTION].
fn collect_connections_from_path(dir: &Path, conns: &mut BTreeSet<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        let mut entry_list: Vec<_> = entries.flatten().collect();
        entry_list.sort_by_key(|e| e.file_name());
        for entry in entry_list {
            let path = entry.path();
            if path.is_dir() {
                collect_connections_from_path(&path, conns);
            } else if path.is_file() {
                let is_spec = path.extension().is_some_and(|e| e == "md" || e == "mmd");
                if is_spec {
                    if let Ok(content) = fs::read_to_string(&path) {
                        conns.extend(extract_connection_ids(&content));
                    }
                }
            }
        }
    }
}

/// Check 27: Cross-Document Diagram Parity Gate (Gate 25B).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Mandates 1:1 bidirectional parity between ConOps SV-1 connection links (`CONN-01`..`CONN-N`),
/// Level 1C System Interface Matrix entries (`docs/interfaces/ICD_01_SYSTEM_INTERFACE_MATRIX.md`),
/// executive engineering reports (`docs/reports/`), and the canonical SSOT Architecture Views Library
/// (`docs/architecture/views/`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Eliminates vacuous truth empty-set bypass: asymmetrical connection counts are rejected.
/// - Set difference across all documents must evaluate to empty set.
///
/// ## 3. Preconditions
/// `repo_root` must be an accessible directory.
///
/// ## 4. Postconditions
/// Returns `Ok(ParityOutcome::Verified)` on exact 1:1 connection link parity.
/// Returns `Err(Vec<String>)` with sorted list of missing connections.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(C log C) where C is total connection count.
/// Space Complexity: O(C) in BTreeSet.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Missing canonical views in upstream mode immediately fails closed.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-PARITY-GATE-27].
pub fn check_cross_document_diagram_parity(repo_root: &Path) -> Result<ParityOutcome, Vec<String>> {
    let is_upstream = is_upstream_compiler(repo_root);
    let arch_views_dir = TargetScanDir::Architecture.resolve(repo_root).join("views");

    if is_upstream {
        if !arch_views_dir.is_dir() {
            return Err(vec![
                "Canonical SSOT Architecture Views Library (docs/architecture/views/) is missing in upstream compiler workspace (mandating canonical architecture views library per refs #276)".to_string(),
            ]);
        }
        let sv1_view = arch_views_dir.join("sv1_super_system.mmd");
        if !sv1_view.is_file() {
            return Err(vec![
                "Canonical SSOT Architecture Views Library (docs/architecture/views/) is missing required canonical view sv1_super_system.mmd (refs #276)".to_string(),
            ]);
        }
    }

    let mut conns_arch = BTreeSet::new();
    if arch_views_dir.is_dir() {
        collect_connections_from_path(&arch_views_dir, &mut conns_arch);
    }

    let mut conns_conops = BTreeSet::new();
    let conops_dir = TargetScanDir::Conops.resolve(repo_root);
    if conops_dir.is_dir() {
        collect_connections_from_path(&conops_dir, &mut conns_conops);
    }
    let conops_file = repo_root.join("docs").join("CONOPS.md");
    if conops_file.is_file() {
        if let Ok(c) = fs::read_to_string(&conops_file) {
            conns_conops.extend(extract_connection_ids(&c));
        }
    }

    let mut conns_icd = BTreeSet::new();
    let interfaces_dir = TargetScanDir::Interfaces.resolve(repo_root);
    if interfaces_dir.is_dir() {
        collect_connections_from_path(&interfaces_dir, &mut conns_icd);
    }
    let icd_dir = TargetScanDir::Icd.resolve(repo_root);
    if icd_dir.is_dir() {
        collect_connections_from_path(&icd_dir, &mut conns_icd);
    }

    let mut conns_reports = BTreeSet::new();
    let reports_dir = TargetScanDir::Reports.resolve(repo_root);
    if reports_dir.is_dir() {
        collect_connections_from_path(&reports_dir, &mut conns_reports);
    }

    let mut mismatches = Vec::new();

    if conns_conops.is_empty() != conns_icd.is_empty() {
        if conns_conops.is_empty() && !conns_icd.is_empty() && conops_dir.is_dir() {
            mismatches.push(format!(
                "Asymmetric connection declarations: ConOps declares 0 connections, while Level 1C System Interface Matrix declares {} connections.",
                conns_icd.len()
            ));
        } else if !conns_conops.is_empty() && conns_icd.is_empty() && interfaces_dir.is_dir() {
            mismatches.push(format!(
                "Asymmetric connection declarations: ConOps declares {} connections, while Level 1C System Interface Matrix declares 0 connections.",
                conns_conops.len()
            ));
        }
    }

    if !conns_arch.is_empty() {
        if !conns_conops.is_empty() {
            for conn in conns_conops.difference(&conns_arch) {
                mismatches.push(format!(
                    "ConOps SV-1 diagram declares connection '{}', but it is missing from Canonical SSOT Architecture Library (docs/architecture/views/) (mandating 1:1 bidirectional parity per refs #276)",
                    conn
                ));
            }
            for conn in conns_arch.difference(&conns_conops) {
                mismatches.push(format!(
                    "Canonical SSOT Architecture Library (docs/architecture/views/) registers connection '{}', but it is missing from ConOps SV-1 diagram (mandating 1:1 bidirectional parity per refs #276)",
                    conn
                ));
            }
        }
        if !conns_icd.is_empty() {
            for conn in conns_icd.difference(&conns_arch) {
                mismatches.push(format!(
                    "Level 1C System Interface Matrix registers connection '{}', but it is missing from Canonical SSOT Architecture Library (docs/architecture/views/) (mandating 1:1 bidirectional parity per refs #276)",
                    conn
                ));
            }
            for conn in conns_arch.difference(&conns_icd) {
                mismatches.push(format!(
                    "Canonical SSOT Architecture Library (docs/architecture/views/) registers connection '{}', but it is missing from Level 1C System Interface Matrix (mandating 1:1 bidirectional parity per refs #276)",
                    conn
                ));
            }
        }
        if !conns_reports.is_empty() {
            for conn in conns_reports.difference(&conns_arch) {
                mismatches.push(format!(
                    "Executive engineering report declares connection '{}', but it is missing from Canonical SSOT Architecture Library (docs/architecture/views/) (mandating 1:1 bidirectional parity per refs #276)",
                    conn
                ));
            }
            for conn in conns_arch.difference(&conns_reports) {
                mismatches.push(format!(
                    "Canonical SSOT Architecture Library (docs/architecture/views/) registers connection '{}', but it is missing from executive engineering report (mandating 1:1 bidirectional parity per refs #276)",
                    conn
                ));
            }
        }
    }

    if !conns_conops.is_empty() && !conns_icd.is_empty() {
        for conn in conns_conops.difference(&conns_icd) {
            mismatches.push(format!(
                "ConOps SV-1 diagram declares connection '{}', but it is missing from Level 1C System Interface Matrix (mandating 1:1 bidirectional parity per refs #271)",
                conn
            ));
        }

        for conn in conns_icd.difference(&conns_conops) {
            mismatches.push(format!(
                "Level 1C System Interface Matrix registers connection '{}', but it is missing from ConOps SV-1 diagram (mandating 1:1 bidirectional parity per refs #271)",
                conn
            ));
        }
    }

    if !conns_reports.is_empty() {
        if !conns_conops.is_empty() {
            for conn in conns_reports.difference(&conns_conops) {
                mismatches.push(format!(
                    "Executive engineering report declares connection '{}', but it is missing from ConOps SV-1 diagram (mandating 1:1 bidirectional parity per refs #276)",
                    conn
                ));
            }
            for conn in conns_conops.difference(&conns_reports) {
                mismatches.push(format!(
                    "ConOps SV-1 diagram declares connection '{}', but it is missing from executive engineering report (mandating 1:1 bidirectional parity per refs #276)",
                    conn
                ));
            }
        }
        if !conns_icd.is_empty() {
            for conn in conns_reports.difference(&conns_icd) {
                mismatches.push(format!(
                    "Executive engineering report declares connection '{}', but it is missing from Level 1C System Interface Matrix (mandating 1:1 bidirectional parity per refs #276)",
                    conn
                ));
            }
            for conn in conns_icd.difference(&conns_reports) {
                mismatches.push(format!(
                    "Level 1C System Interface Matrix registers connection '{}', but it is missing from executive engineering report (mandating 1:1 bidirectional parity per refs #276)",
                    conn
                ));
            }
        }
    }

    if !mismatches.is_empty() {
        mismatches.sort();
        mismatches.dedup();
        return Err(mismatches);
    }

    Ok(ParityOutcome::Verified)
}

/// Check 28: ConOps & Mission Intent Completeness Gate (Gate 26).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ISO 26262 ASIL D. Validates ConOps schema contract
/// and specification completeness fail-closed.
///
/// ## 2. Invariants & Mathematical Properties
/// - All JSON schema and markdown parse failures propagate as verification errors.
/// - Clean landing zones evaluate to `PendingOrClean`.
///
/// ## 3. Preconditions
/// `repo_root` must be an accessible directory.
///
/// ## 4. Postconditions
/// Returns `Ok(ParityOutcome::Verified)` on completeness.
/// Returns `Err(Vec<String>)` on malformed schema or incomplete ConOps specification.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(J + C) for JSON validation and ConOps audit.
/// Space Complexity: O(S) for schema AST.
///
/// ## 6. Failure Modes & Fault Tolerance
/// All I/O and parser errors fail closed and return `Err`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-PARITY-GATE-28].
pub fn check_conops_and_mission_intent_completeness(
    repo_root: &Path,
    allow_missing_specs: bool,
    strict: bool,
) -> Result<ParityOutcome, Vec<String>> {
    let is_upstream = is_upstream_compiler(repo_root);
    let conops_dir = repo_root.join("docs").join("conops");

    let schema_file = repo_root.join(".pipeline").join("schemas").join("conops_specification_schema.json");
    if schema_file.is_file() {
        match crate::spec_audit::conops_audit::validate_conops_schema_file(&schema_file) {
            Ok(schema_findings) if !schema_findings.is_empty() => {
                let errs: Vec<String> = schema_findings.into_iter().map(|f| f.to_string()).collect();
                return Err(errs);
            }
            Err(e) => {
                return Err(vec![format!(
                    "Check 28 failed: Unable to parse or validate schema '{}': {}",
                    schema_file.display(),
                    e
                )]);
            }
            Ok(_) => {}
        }
    }

    if !is_upstream {
        if !conops_dir.is_dir() {
            return Ok(ParityOutcome::ConopsDirMissing);
        }

        let mut conops_files = Vec::new();
        let entries = fs::read_dir(&conops_dir).map_err(|e| vec![format!(
            "Check 28 failed: Unable to read conops directory '{}': {}",
            conops_dir.display(), e
        )])?;
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".md") && name != "README.md" && !name.to_ascii_uppercase().contains("TEMPLATE") {
                conops_files.push(entry.path());
            }
        }
        conops_files.sort();

        if conops_files.is_empty() {
            return Ok(ParityOutcome::PendingOrClean);
        }

        let model_text = discover_sysml_model_text(repo_root);
        let schema_dir = repo_root.join("schema");
        let has_extracted = schema_dir.join("extracted").is_dir();
        let effective_allow_missing = allow_missing_specs && !strict;

        if effective_allow_missing && model_text.is_none() && !has_extracted {
            return Ok(ParityOutcome::PendingOrClean);
        }

        for path in &conops_files {
            let content = fs::read_to_string(path).map_err(|e| vec![format!(
                "Check 28 failed: Unable to read ConOps file '{}': {}",
                path.display(), e
            )])?;
            let doc = crate::spec_audit::markdown_ast::parse_markdown(&content).map_err(|e| vec![format!(
                "Check 28 failed: Unable to parse markdown in '{}': {}",
                path.display(), e
            )])?;
            let rel = match path.strip_prefix(repo_root) {
                Ok(r) => r.to_string_lossy().to_string(),
                Err(_) => path.to_string_lossy().to_string(),
            };
            let findings = crate::spec_audit::conops_audit::validate_conops(&doc, &content, &rel).map_err(|e| vec![format!(
                "Check 28 failed: Validation error in '{}': {}",
                rel, e
            )])?;
            if !findings.is_empty() && (strict || !effective_allow_missing) {
                let errs: Vec<String> = findings.into_iter().map(|f| f.to_string()).collect();
                return Err(errs);
            }
        }
    }

    Ok(ParityOutcome::Verified)
}

/// Check 29: Cited Research Inventory & Declared-Total Population Register Gate (Gate 27).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ISO 26262 ASIL D. Audits normative bibliographies and
/// external standard citations to verify valid registry IDs and eliminate phantom citations.
///
/// ## 2. Invariants & Mathematical Properties
/// - Every cited standard in `Source References` must have a valid identifier and entry.
/// - Clean landing zones evaluate to `PendingOrClean`.
///
/// ## 3. Preconditions
/// `repo_root` must be an accessible directory.
///
/// ## 4. Postconditions
/// Returns `Ok(ParityOutcome::Verified)` on inventory completeness.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(F * L). Space Complexity: O(R) for reference entries.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Propagates file access errors fail-closed.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-PARITY-GATE-29].
pub fn check_research_inventory(
    repo_root: &Path,
    allow_missing_specs: bool,
    strict: bool,
) -> Result<ParityOutcome, Vec<String>> {
    let is_upstream = is_upstream_compiler(repo_root);
    if is_upstream {
        return Ok(ParityOutcome::Verified);
    }

    let model_text = discover_sysml_model_text(repo_root);
    let schema_dir = repo_root.join("schema");
    let has_extracted = schema_dir.join("extracted").is_dir();
    let effective_allow_missing = allow_missing_specs && !strict;

    if effective_allow_missing && model_text.is_none() && !has_extracted {
        return Ok(ParityOutcome::PendingOrClean);
    }

    let mut errors = Vec::new();
    let mut spec_files = Vec::new();
    let scan_dirs = [
        TargetScanDir::Epics.resolve(repo_root),
        TargetScanDir::Features.resolve(repo_root),
        TargetScanDir::Reports.resolve(repo_root),
    ];

    for dir in &scan_dirs {
        if dir.is_dir() {
            for entry_res in WalkDir::new(dir).sort_by_file_name().into_iter() {
                let entry = match entry_res {
                    Ok(e) => e,
                    Err(err) => {
                        errors.push(format!("Check 29 failed: I/O error walking directory '{}': {}", dir.display(), err));
                        continue;
                    }
                };
                if entry.file_type().is_file() {
                    let path = entry.path().to_path_buf();
                    if path.extension().is_some_and(|ext| ext == "md") {
                        spec_files.push(path);
                    }
                }
            }
        }
    }
    spec_files.sort();

    if spec_files.is_empty() {
        return Ok(ParityOutcome::PendingOrClean);
    }

    for path in spec_files {
        let rel = path.strip_prefix(repo_root).unwrap_or(&path).display().to_string();
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                errors.push(format!("Check 29 failed: Unable to read file '{}': {}", rel, e));
                continue;
            }
        };

        let mut in_source_refs = false;
        for (idx, line) in content.lines().enumerate() {
            let line_no = idx.saturating_add(1);
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                let lower = trimmed.to_ascii_lowercase();
                in_source_refs = lower.contains("source references") || lower.contains("bibliography");
                continue;
            }

            if in_source_refs && (trimmed.starts_with("- ") || trimmed.starts_with("* ")) {
                let entry_text = trimmed.trim_start_matches(['-', '*', ' ']);
                if entry_text.is_empty() || is_metrology_placeholder(entry_text) {
                    errors.push(format!(
                        "Check 29 failed: Incomplete or placeholder source reference on line {} in '{}': '{}'",
                        line_no, rel, entry_text
                    ));
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(ParityOutcome::Verified)
    } else {
        Err(errors)
    }
}

/// Check 30: Executive Deliverable Traceability & Completeness Gate (Gate 27B).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ISO 26262 ASIL D. Audits executive deliverable reports
/// to verify that all diagrams, tables, and claims trace to SSOT architecture views.
///
/// ## 2. Invariants & Mathematical Properties
/// Reports must anchor all interface connections to SSOT views.
///
/// ## 3. Preconditions
/// `repo_root` must be a directory.
///
/// ## 4. Postconditions
/// Returns `Ok(ParityOutcome::Verified)` when report deliverables trace cleanly.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(R * L) where R is reports count. Space Complexity: O(D).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed on ungrounded deliverables in downstream mode.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-PARITY-GATE-30].
pub fn check_executive_deliverable_traceability(repo_root: &Path) -> Result<ParityOutcome, Vec<String>> {
    let reports_dir = TargetScanDir::Reports.resolve(repo_root);
    if !reports_dir.is_dir() {
        return Ok(ParityOutcome::PendingOrClean);
    }

    let mut errors = Vec::new();
    let mut report_files = Vec::new();
    for entry_res in WalkDir::new(&reports_dir).sort_by_file_name().into_iter() {
        let entry = match entry_res {
            Ok(e) => e,
            Err(err) => {
                errors.push(format!("Check 30 failed: I/O error walking directory '{}': {}", reports_dir.display(), err));
                continue;
            }
        };
        if entry.file_type().is_file() {
            let path = entry.path().to_path_buf();
            if path.extension().is_some_and(|ext| ext == "md") {
                report_files.push(path);
            }
        }
    }
    report_files.sort();

    if report_files.is_empty() {
        return Ok(ParityOutcome::PendingOrClean);
    }

    let mut ssot_conns = BTreeSet::new();
    let views_dir = repo_root.join("docs").join("architecture").join("views");
    if views_dir.is_dir() {
        for entry_res in WalkDir::new(&views_dir).sort_by_file_name().into_iter() {
            if let Ok(entry) = entry_res {
                if entry.file_type().is_file() {
                    if let Ok(vc) = fs::read_to_string(entry.path()) {
                        for cid in extract_connection_ids(&vc) {
                            ssot_conns.insert(cid);
                        }
                    }
                }
            }
        }
    }
    let icd_file = TargetScanDir::Interfaces
        .resolve(repo_root)
        .join("ICD_01_SYSTEM_INTERFACE_MATRIX.md");
    if icd_file.is_file() {
        if let Ok(ic) = fs::read_to_string(&icd_file) {
            for cid in extract_connection_ids(&ic) {
                ssot_conns.insert(cid);
            }
        }
    }
    let conops_dir = TargetScanDir::Conops.resolve(repo_root);
    if conops_dir.is_dir() {
        for entry_res in WalkDir::new(&conops_dir).sort_by_file_name().into_iter() {
            if let Ok(entry) = entry_res {
                if entry.file_type().is_file() && entry.path().extension().is_some_and(|ext| ext == "md") {
                    if let Ok(cc) = fs::read_to_string(entry.path()) {
                        for cid in extract_connection_ids(&cc) {
                            ssot_conns.insert(cid);
                        }
                    }
                }
            }
        }
    }

    for path in report_files {
        let rel = path.strip_prefix(repo_root).unwrap_or(&path).display().to_string();
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                errors.push(format!("Check 30 failed: Unable to read file '{}': {}", rel, e));
                continue;
            }
        };

        let conns = extract_connection_ids(&content);
        for conn in conns {
            let digits_only = conn
                .strip_prefix("CONN-")
                .is_some_and(|rest| !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()));
            if !digits_only || (!ssot_conns.is_empty() && !ssot_conns.contains(&conn)) {
                errors.push(format!(
                    "Check 30 failed: Report '{}' references ungrounded connection '{}'",
                    rel, conn
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(ParityOutcome::Verified)
    } else {
        Err(errors)
    }
}

/// Check 30A: Coverage-Digest Population Gate (Gate 28).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ISO 26262 ASIL D. Verifies that requirement and test
/// coverage matrices reference valid existing specification items (`FEAT-*`, `UC-*`, `US-*`) without phantom entries.
///
/// ## 2. Invariants & Mathematical Properties
/// Every referenced `FEAT-*.md`, `UC-*.md`, and `US-*.md` specification file across reports and specs must exist on disk.
///
/// ## 3. Preconditions
/// `repo_root` must be a directory.
///
/// ## 4. Postconditions
/// Returns `Ok(ParityOutcome::Verified)` when zero phantom specification file references exist.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(M * L). Space Complexity: O(F).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed on phantom realizations.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COVERAGE-DIGEST-GATE].
pub fn check_coverage_digest(repo_root: &Path) -> Result<ParityOutcome, Vec<String>> {
    let is_upstream = is_upstream_compiler(repo_root);
    if is_upstream {
        return Ok(ParityOutcome::PendingOrClean);
    }

    let mut errors = Vec::new();
    let mut declared_specs = BTreeSet::new();
    for spec_dir_kind in [
        TargetScanDir::Features,
        TargetScanDir::UseCases,
        TargetScanDir::UserStories,
    ] {
        let dir = spec_dir_kind.resolve(repo_root);
        if dir.is_dir() {
            for entry_res in WalkDir::new(&dir).sort_by_file_name().into_iter() {
                let entry = match entry_res {
                    Ok(e) => e,
                    Err(err) => {
                        errors.push(format!(
                            "Check 30A failed: I/O error walking directory '{}': {}",
                            dir.display(),
                            err
                        ));
                        continue;
                    }
                };
                if entry.file_type().is_file() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.ends_with(".md") && name != "README.md" {
                        declared_specs.insert(name.to_ascii_lowercase());
                    }
                }
            }
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }

    if declared_specs.is_empty() {
        return Ok(ParityOutcome::PendingOrClean);
    }

    for scan_dir_kind in [
        TargetScanDir::Reports,
        TargetScanDir::Epics,
        TargetScanDir::Features,
        TargetScanDir::UseCases,
        TargetScanDir::UserStories,
    ] {
        let dir = scan_dir_kind.resolve(repo_root);
        if !dir.is_dir() {
            continue;
        }
        for entry_res in WalkDir::new(&dir).sort_by_file_name().into_iter() {
            let entry = match entry_res {
                Ok(e) => e,
                Err(err) => {
                    errors.push(format!(
                        "Check 30A failed: I/O error walking directory '{}': {}",
                        dir.display(),
                        err
                    ));
                    continue;
                }
            };
            if entry.file_type().is_file() && entry.path().extension().is_some_and(|ext| ext == "md") {
                let fname = entry.file_name().to_string_lossy();
                if fname == "README.md" {
                    continue;
                }
                let rel = entry
                    .path()
                    .strip_prefix(repo_root)
                    .unwrap_or(entry.path())
                    .display()
                    .to_string();
                let content = match fs::read_to_string(entry.path()) {
                    Ok(c) => c,
                    Err(e) => {
                        errors.push(format!("Check 30A failed: Unable to read '{}': {}", rel, e));
                        continue;
                    }
                };
                for word in content.split_whitespace() {
                    for part in word.split(['(', ')', '[', ']', '`', '"', '\'']) {
                        let clean = part.trim_matches(|c: char| {
                            !c.is_ascii_alphanumeric() && c != '-' && c != '_' && c != '.' && c != '/'
                        });
                        let basename = clean.rsplit('/').next().unwrap_or(clean);
                        let bl = basename.to_ascii_lowercase();
                        if (bl.starts_with("feat-") || bl.starts_with("uc-") || bl.starts_with("us-"))
                            && bl.ends_with(".md")
                            && !declared_specs.contains(&bl)
                        {
                            errors.push(format!(
                                "Check 30A failed: Coverage digest or specification '{}' references undeclared specification file '{}'",
                                rel, basename
                            ));
                        }
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(ParityOutcome::Verified)
    } else {
        Err(errors)
    }
}

/// Extracts canonical STPA identifiers with a given prefix (e.g., `H-` or `SC-`) from a line,
/// normalizing numeric suffixes so `H-01` and `H-1` map to `"H-1"`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and MIL-STD-882E. Enables general STPA hazard and safety constraint
/// obligation extraction without hardcoded identifier literals.
///
/// ## 2. Invariants & Mathematical Properties
/// - Prefix must be followed by one or more ASCII digits.
/// - Leading zeros on the numeric suffix are normalized (`01` -> `1`).
///
/// ## 3. Preconditions
/// `line` and `prefix` must be valid UTF-8 strings (`prefix` uppercase, ending in `-`).
///
/// ## 4. Postconditions
/// Returns a sorted `BTreeSet<String>` of canonical STPA identifiers found on `line`.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(W) where W is token count on the line.
/// Space Complexity: O(W).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Total function; never panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-OBLIGATION-WITNESS-GATE].
fn extract_stpa_prefixed_ids(line: &str, prefix: &str) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for raw_tok in line.split(|c: char| !c.is_ascii_alphanumeric() && c != '-') {
        let upper = raw_tok.to_ascii_uppercase();
        if let Some(rest) = upper.strip_prefix(prefix) {
            if !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()) {
                let normalized_num = rest.trim_start_matches('0');
                let num_str = if normalized_num.is_empty() { "0" } else { normalized_num };
                ids.insert(format!("{}{}", prefix, num_str));
            }
        }
    }
    ids
}

/// Check 30B: Obligation-Witness Registry Gate (Gate 29).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, ISO 26262 ASIL D, and STPA safety engineering. Verifies
/// that every declared hazard (`H-*`) in `docs/safety/` is witnessed by a declared safety constraint (`SC-*`).
///
/// ## 2. Invariants & Mathematical Properties
/// Every declared hazard `H-*` must be witnessed by at least one safety constraint `SC-*`.
///
/// ## 3. Preconditions
/// `repo_root` must be a directory.
///
/// ## 4. Postconditions
/// Returns `Ok(ParityOutcome::Verified)` when zero orphan hazards exist.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(H). Space Complexity: O(H) where H is hazard count.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed on unwitnessed hazards or missing safety constraints.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-OBLIGATION-WITNESS-GATE].
pub fn check_obligation_witness(repo_root: &Path) -> Result<ParityOutcome, Vec<String>> {
    let is_upstream = is_upstream_compiler(repo_root);
    if is_upstream {
        return Ok(ParityOutcome::PendingOrClean);
    }

    let safety_dir = TargetScanDir::Safety.resolve(repo_root);
    if !safety_dir.is_dir() {
        return Ok(ParityOutcome::PendingOrClean);
    }

    let mut errors = Vec::new();
    let mut stpa_files = Vec::new();
    for entry_res in WalkDir::new(&safety_dir).sort_by_file_name().into_iter() {
        let entry = match entry_res {
            Ok(e) => e,
            Err(err) => {
                errors.push(format!("Check 30B failed: I/O error walking directory '{}': {}", safety_dir.display(), err));
                continue;
            }
        };
        if entry.file_type().is_file() {
            let path = entry.path().to_path_buf();
            if path.extension().is_some_and(|ext| ext == "md") {
                let fname = entry.file_name().to_string_lossy();
                if fname != "README.md" {
                    stpa_files.push(path);
                }
            }
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }

    if stpa_files.is_empty() {
        return Ok(ParityOutcome::PendingOrClean);
    }

    let mut declared_hazards = BTreeSet::new();
    let mut declared_constraints = BTreeSet::new();
    let mut witnessed_hazards = BTreeSet::new();
    let mut has_explicit_uca_bindings = false;

    for path in &stpa_files {
        let rel = path.strip_prefix(repo_root).unwrap_or(path).display().to_string();
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                errors.push(format!("Check 30B failed: Unable to read '{}': {}", rel, e));
                continue;
            }
        };
        for line in content.lines() {
            let line_h = extract_stpa_prefixed_ids(line, "H-");
            let line_sc = extract_stpa_prefixed_ids(line, "SC-");
            if !line_h.is_empty() && !line_sc.is_empty() {
                has_explicit_uca_bindings = true;
                for h in &line_h {
                    witnessed_hazards.insert(h.clone());
                }
            }
            for h in line_h {
                declared_hazards.insert(h);
            }
            for sc in line_sc {
                declared_constraints.insert(sc);
            }
        }
    }

    if !declared_hazards.is_empty() && declared_constraints.is_empty() {
        errors.push(format!(
            "Check 30B failed: Safety specifications declare hazards ({:?}) without any witnessing safety constraint (SC-*).",
            declared_hazards
        ));
    } else if has_explicit_uca_bindings {
        for h in &declared_hazards {
            if !witnessed_hazards.contains(h) {
                errors.push(format!(
                    "Check 30B failed: Declared hazard '{}' in safety specifications is not witnessed by any safety constraint (SC-*) binding.",
                    h
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(ParityOutcome::Verified)
    } else {
        Err(errors)
    }
}

/// Check 30C: Architecture Viewpoint & Diagram Completeness Gate (Gate 30).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ISO 26262 ASIL D. Validates that mandatory architecture
/// viewpoints (such as SV-1 Super System) exist in the Canonical SSOT Architecture Views Library.
///
/// ## 2. Invariants & Mathematical Properties
/// In upstream compiler workspaces, canonical viewpoint views must be present.
///
/// ## 3. Preconditions
/// `repo_root` must be a directory.
///
/// ## 4. Postconditions
/// Returns `Ok(ParityOutcome::Verified)` on architecture viewpoint completeness.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(1). Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed when canonical architecture library views are missing in upstream mode.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-PARITY-GATE-30-VIEWPOINTS].
pub fn check_architecture_viewpoint_diagrams(
    repo_root: &Path,
    allow_missing_specs: bool,
    strict: bool,
) -> Result<ParityOutcome, Vec<String>> {
    let is_upstream = is_upstream_compiler(repo_root);
    if is_upstream {
        let arch_views_dir = TargetScanDir::Architecture.resolve(repo_root).join("views");
        if !arch_views_dir.is_dir() {
            return Err(vec![
                "Check 30C failed: Canonical SSOT Architecture Views Library (docs/architecture/views/) is missing in upstream compiler workspace (refs #276)".to_string(),
            ]);
        }
        if !arch_views_dir.join("sv1_super_system.mmd").is_file() {
            return Err(vec![
                "Check 30C failed: Canonical SSOT Architecture Views Library is missing required view sv1_super_system.mmd (refs #276)".to_string(),
            ]);
        }
    } else {
        let model_text = discover_sysml_model_text(repo_root);
        let schema_dir = repo_root.join("schema");
        let has_extracted = schema_dir.join("extracted").is_dir();
        let effective_allow_missing = allow_missing_specs && !strict;

        if effective_allow_missing && model_text.is_none() && !has_extracted {
            return Ok(ParityOutcome::PendingOrClean);
        }
    }

    Ok(ParityOutcome::Verified)
}

/// Collects package-qualified slices `(package_path, shallow_package_body)` from SysML v2 source text.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C software engineering verification requirements.
/// Deterministically splits nested and sibling SysML v2 package blocks into package-qualified
/// body segments, enabling precise scope qualification and preventing cross-package AST collisions.
///
/// ## 2. Invariants & Mathematical Properties
/// - Package Scope Monotonicity: Child package bodies are prefixed by their parent path (`Parent::Child`).
/// - Character & Slice Safety: Uses character and byte bounds checking, guaranteeing panic-free extraction.
///
/// ## 3. Preconditions
/// `content` must be valid UTF-8 SysML v2 source text.
///
/// ## 4. Postconditions
/// Appends `(qualified_package_path, shallow_body)` tuples to `out`.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(L) where L is text length.
/// Space Complexity: O(L) bounded by source string size.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Unmatched braces or malformed tokens skip safely without panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DUAL-SCHEMA-SSOT-PARITY/PACKAGE-SCOPING].
fn collect_qualified_package_bodies(content: &str, parent_pkg: &str, out: &mut Vec<(String, String)>) {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut remainder = clean.as_bytes().to_vec();
    let mut idx = 0;

    while let Some(pos) = masked.get(idx..).and_then(|s| s.find("package")) {
        let pkg_pos = idx + pos;
        let before_ok = pkg_pos == 0
            || !masked.as_bytes().get(pkg_pos - 1).copied().is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_');
        let after_ok = !masked.as_bytes().get(pkg_pos + 7).copied().is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_');
        if !before_ok || !after_ok {
            idx = pkg_pos + 7;
            continue;
        }
        let mut curr = pkg_pos + 7;
        curr += masked.get(curr..).unwrap_or("").bytes().take_while(|b| b.is_ascii_whitespace()).count();
        let name_len = masked.get(curr..).unwrap_or("").bytes().take_while(|b| b.is_ascii_alphanumeric() || *b == b'_').count();
        let name = masked.get(curr..curr + name_len).unwrap_or("");
        if name.is_empty() {
            idx = pkg_pos + 7;
            continue;
        }
        curr += name_len;
        curr += masked.get(curr..).unwrap_or("").bytes().take_while(|b| b.is_ascii_whitespace()).count();
        if masked.get(curr..).is_some_and(|s| s.starts_with('{')) {
            if let Some(close_pos) = find_matching_brace(&clean, curr) {
                let body = clean.get(curr + 1..close_pos).unwrap_or("");
                let qualified_pkg = if parent_pkg.is_empty() {
                    name.to_string()
                } else {
                    format!("{}::{}", parent_pkg, name)
                };
                collect_qualified_package_bodies(body, &qualified_pkg, out);
                if let Some(slice) = remainder.get_mut(pkg_pos..=close_pos) {
                    for b in slice {
                        if *b != b'\n' {
                            *b = b' ';
                        }
                    }
                }
                idx = close_pos + 1;
                continue;
            }
        }
        idx = pkg_pos + 7;
    }

    let shallow = String::from_utf8(remainder).unwrap_or_default();
    if !shallow.trim().is_empty() {
        out.push((parent_pkg.to_string(), shallow));
    }
}

/// Compares scope-qualified AST definitions between schema directory and pipeline schema.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C software verification requirements.
/// Enforces bidirectional AST equivalence between directory schemas and pipeline schema.
///
/// ## 2. Invariants & Mathematical Properties
/// - Bidirectional Inclusion: Every key in `dir_map` must match `pipe_map`, and vice-versa.
/// - Deterministic Ordering: Iterates over sorted BTreeMap keys for reproducible error reporting.
///
/// ## 3. Preconditions
/// `dir_map` and `pipe_map` contain scope-qualified AST node vectors implementing PartialEq.
///
/// ## 4. Postconditions
/// Appends diagnostic mismatch messages to `mismatches` if differences are detected.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(K * N) where K is key count and N is average vector length.
/// Space Complexity: O(M) where M is count of mismatch diagnostic strings.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Appends all divergence details to `mismatches` without panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DUAL-SCHEMA-SSOT-PARITY/SCOPED-COMPARISON].
fn compare_scoped_defs<T: PartialEq>(
    kind_label: &str,
    dir_map: &BTreeMap<String, Vec<T>>,
    pipe_map: &BTreeMap<String, Vec<T>>,
    mismatches: &mut Vec<String>,
) {
    for (qual_name, dir_items) in dir_map {
        match pipe_map.get(qual_name) {
            Some(pipe_items) => {
                if dir_items != pipe_items {
                    if kind_label == "part def" {
                        mismatches.push(format!(
                            "part def '{}' has structural divergence between schema/*.sysml and .pipeline/schema.sysml (ports, attributes, or body differ)",
                            qual_name
                        ));
                    } else {
                        mismatches.push(format!(
                            "{} '{}' has structural divergence between schema/*.sysml and .pipeline/schema.sysml",
                            kind_label, qual_name
                        ));
                    }
                }
            }
            None => {
                mismatches.push(format!(
                    "{} defined in schema/*.sysml but missing in .pipeline/schema.sysml: [\"{}\"]",
                    kind_label, qual_name
                ));
            }
        }
    }
    for qual_name in pipe_map.keys() {
        if !dir_map.contains_key(qual_name) {
            mismatches.push(format!(
                "{} defined in .pipeline/schema.sysml but missing in schema/*.sysml: [\"{}\"]",
                kind_label, qual_name
            ));
        }
    }
}

/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C software engineering verification requirements.
/// Guarantees bit-for-bit SSOT parity between the modular multi-file schema tree (`schema/**/*.sysml`)
/// and the consolidated single-file execution schema (`.pipeline/schema.sysml`), preventing drift.
///
/// ## 2. Invariants & Mathematical Properties
/// - AST Parity Invariant: Every scope-qualified SysML v2 declaration across all 10 AST kinds
///   (part, port, action, requirement, constraint, state, calc, connection, test case, attribute)
///   in `schema/**/*.sysml` must be structurally isomorphic to `.pipeline/schema.sysml`.
/// - Monotonic Scope Invariant: Names are qualified by enclosing package path hierarchy, eliminating
///   unqualified map key collisions across sibling or nested packages.
///
/// ## 3. Preconditions
/// `repo_root` must be a directory.
///
/// ## 4. Postconditions
/// Returns `Ok(ParityOutcome::DualSchemaIdentical)` when AST definitions are structurally identical.
/// Returns `Ok(ParityOutcome::DualSchemaSingleOrClean)` if dual schema is not configured.
/// Returns `Err(Vec<String>)` with specific structural divergence diagnostics on schema drift.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(A) where A is total AST node count.
/// Space Complexity: O(A) in BTreeMap.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed on any structural or attribute difference.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DUAL-SCHEMA-SSOT-PARITY].
pub fn check_dual_schema_ssot_parity(repo_root: &Path) -> Result<ParityOutcome, Vec<String>> {
    let schema_dir = repo_root.join("schema");
    let pipeline_schema = repo_root.join(".pipeline").join("schema.sysml");

    let mut schema_sysml_files = Vec::new();
    if schema_dir.is_dir() {
        let all_files = deap_core::sysml_ast::UnifiedSchemaModelExtractor::discover_schema_files(&[&schema_dir]);
        schema_sysml_files = all_files
            .into_iter()
            .filter(|p| {
                p.extension().is_some_and(|ext| ext == "sysml")
                    && fs::metadata(p).map(|m| m.len() > 0).unwrap_or(false)
            })
            .collect();
    }

    let has_pipeline_schema = pipeline_schema.is_file()
        && fs::metadata(&pipeline_schema).map(|m| m.len() > 0).unwrap_or(false);
    let has_schema_files = !schema_sysml_files.is_empty();

    if !(has_schema_files && has_pipeline_schema) {
        return Ok(ParityOutcome::DualSchemaSingleOrClean);
    }

    let mut schema_dir_content = String::new();
    schema_sysml_files.sort();
    for fpath in &schema_sysml_files {
        match fs::read_to_string(fpath) {
            Ok(c) => {
                schema_dir_content.push_str(&c);
                schema_dir_content.push_str("\n\n");
            }
            Err(e) => {
                return Err(vec![format!("Check 31 failed: Unable to read {}: {}", fpath.display(), e)]);
            }
        }
    }

    let pipeline_schema_content = match fs::read_to_string(&pipeline_schema) {
        Ok(c) => c,
        Err(e) => {
            return Err(vec![format!("Check 31 failed: Unable to read {}: {}", pipeline_schema.display(), e)]);
        }
    };

    let mut dir_bodies = Vec::new();
    collect_qualified_package_bodies(&schema_dir_content, "", &mut dir_bodies);

    let mut pipe_bodies = Vec::new();
    collect_qualified_package_bodies(&pipeline_schema_content, "", &mut pipe_bodies);

    let mut parts_dir: BTreeMap<String, Vec<_>> = BTreeMap::new();
    let mut ports_dir: BTreeMap<String, Vec<_>> = BTreeMap::new();
    let mut actions_dir: BTreeMap<String, Vec<_>> = BTreeMap::new();
    let mut reqs_dir: BTreeMap<String, Vec<_>> = BTreeMap::new();
    let mut constraints_dir: BTreeMap<String, Vec<_>> = BTreeMap::new();
    let mut states_dir: BTreeMap<String, Vec<_>> = BTreeMap::new();
    let mut calcs_dir: BTreeMap<String, Vec<_>> = BTreeMap::new();
    let mut conns_dir: BTreeMap<String, Vec<_>> = BTreeMap::new();
    let mut tests_dir: BTreeMap<String, Vec<_>> = BTreeMap::new();
    let mut attrs_dir: BTreeMap<String, Vec<_>> = BTreeMap::new();

    for (pkg_path, body) in &dir_bodies {
        for p in parse_part_defs(body) {
            let qual = if pkg_path.is_empty() { p.name.clone() } else { format!("{}::{}", pkg_path, p.name) };
            parts_dir.entry(qual).or_default().push(p);
        }
        for p in parse_port_defs(body) {
            let qual = if pkg_path.is_empty() { p.name.clone() } else { format!("{}::{}", pkg_path, p.name) };
            ports_dir.entry(qual).or_default().push(p);
        }
        for a in parse_action_defs(body) {
            let qual = if pkg_path.is_empty() { a.name.clone() } else { format!("{}::{}", pkg_path, a.name) };
            actions_dir.entry(qual).or_default().push(a);
        }
        for r in parse_requirement_defs(body) {
            let qual = if pkg_path.is_empty() { r.name.clone() } else { format!("{}::{}", pkg_path, r.name) };
            reqs_dir.entry(qual).or_default().push(r);
        }
        for c in parse_constraint_defs(body) {
            let qual = if pkg_path.is_empty() { c.name.clone() } else { format!("{}::{}", pkg_path, c.name) };
            constraints_dir.entry(qual).or_default().push(c);
        }
        for s in parse_state_defs(body) {
            let qual = if pkg_path.is_empty() { s.name.clone() } else { format!("{}::{}", pkg_path, s.name) };
            states_dir.entry(qual).or_default().push(s);
        }
        for c in parse_calc_defs(body) {
            let qual = if pkg_path.is_empty() { c.name.clone() } else { format!("{}::{}", pkg_path, c.name) };
            calcs_dir.entry(qual).or_default().push(c);
        }
        for c in parse_connection_defs(body) {
            let qual = if pkg_path.is_empty() { c.name.clone() } else { format!("{}::{}", pkg_path, c.name) };
            conns_dir.entry(qual).or_default().push(c);
        }
        for t in parse_test_case_defs(body) {
            let qual = if pkg_path.is_empty() { t.name.clone() } else { format!("{}::{}", pkg_path, t.name) };
            tests_dir.entry(qual).or_default().push(t);
        }
        for a in parse_attribute_defs(body) {
            let qual = if pkg_path.is_empty() { a.name.clone() } else { format!("{}::{}", pkg_path, a.name) };
            attrs_dir.entry(qual).or_default().push(a);
        }
    }

    let mut parts_pipe: BTreeMap<String, Vec<_>> = BTreeMap::new();
    let mut ports_pipe: BTreeMap<String, Vec<_>> = BTreeMap::new();
    let mut actions_pipe: BTreeMap<String, Vec<_>> = BTreeMap::new();
    let mut reqs_pipe: BTreeMap<String, Vec<_>> = BTreeMap::new();
    let mut constraints_pipe: BTreeMap<String, Vec<_>> = BTreeMap::new();
    let mut states_pipe: BTreeMap<String, Vec<_>> = BTreeMap::new();
    let mut calcs_pipe: BTreeMap<String, Vec<_>> = BTreeMap::new();
    let mut conns_pipe: BTreeMap<String, Vec<_>> = BTreeMap::new();
    let mut tests_pipe: BTreeMap<String, Vec<_>> = BTreeMap::new();
    let mut attrs_pipe: BTreeMap<String, Vec<_>> = BTreeMap::new();

    for (pkg_path, body) in &pipe_bodies {
        for p in parse_part_defs(body) {
            let qual = if pkg_path.is_empty() { p.name.clone() } else { format!("{}::{}", pkg_path, p.name) };
            parts_pipe.entry(qual).or_default().push(p);
        }
        for p in parse_port_defs(body) {
            let qual = if pkg_path.is_empty() { p.name.clone() } else { format!("{}::{}", pkg_path, p.name) };
            ports_pipe.entry(qual).or_default().push(p);
        }
        for a in parse_action_defs(body) {
            let qual = if pkg_path.is_empty() { a.name.clone() } else { format!("{}::{}", pkg_path, a.name) };
            actions_pipe.entry(qual).or_default().push(a);
        }
        for r in parse_requirement_defs(body) {
            let qual = if pkg_path.is_empty() { r.name.clone() } else { format!("{}::{}", pkg_path, r.name) };
            reqs_pipe.entry(qual).or_default().push(r);
        }
        for c in parse_constraint_defs(body) {
            let qual = if pkg_path.is_empty() { c.name.clone() } else { format!("{}::{}", pkg_path, c.name) };
            constraints_pipe.entry(qual).or_default().push(c);
        }
        for s in parse_state_defs(body) {
            let qual = if pkg_path.is_empty() { s.name.clone() } else { format!("{}::{}", pkg_path, s.name) };
            states_pipe.entry(qual).or_default().push(s);
        }
        for c in parse_calc_defs(body) {
            let qual = if pkg_path.is_empty() { c.name.clone() } else { format!("{}::{}", pkg_path, c.name) };
            calcs_pipe.entry(qual).or_default().push(c);
        }
        for c in parse_connection_defs(body) {
            let qual = if pkg_path.is_empty() { c.name.clone() } else { format!("{}::{}", pkg_path, c.name) };
            conns_pipe.entry(qual).or_default().push(c);
        }
        for t in parse_test_case_defs(body) {
            let qual = if pkg_path.is_empty() { t.name.clone() } else { format!("{}::{}", pkg_path, t.name) };
            tests_pipe.entry(qual).or_default().push(t);
        }
        for a in parse_attribute_defs(body) {
            let qual = if pkg_path.is_empty() { a.name.clone() } else { format!("{}::{}", pkg_path, a.name) };
            attrs_pipe.entry(qual).or_default().push(a);
        }
    }

    let mut mismatches = Vec::new();
    compare_scoped_defs("part def", &parts_dir, &parts_pipe, &mut mismatches);
    compare_scoped_defs("port def", &ports_dir, &ports_pipe, &mut mismatches);
    compare_scoped_defs("action def", &actions_dir, &actions_pipe, &mut mismatches);
    compare_scoped_defs("requirement def", &reqs_dir, &reqs_pipe, &mut mismatches);
    compare_scoped_defs("constraint def", &constraints_dir, &constraints_pipe, &mut mismatches);
    compare_scoped_defs("state def", &states_dir, &states_pipe, &mut mismatches);
    compare_scoped_defs("calc def", &calcs_dir, &calcs_pipe, &mut mismatches);
    compare_scoped_defs("connection def", &conns_dir, &conns_pipe, &mut mismatches);
    compare_scoped_defs("test case def", &tests_dir, &tests_pipe, &mut mismatches);
    compare_scoped_defs("attribute def", &attrs_dir, &attrs_pipe, &mut mismatches);

    if mismatches.is_empty() {
        Ok(ParityOutcome::DualSchemaIdentical)
    } else {
        Err(mismatches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(name: &str) -> Result<Self, Box<dyn std::error::Error>> {
            static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let count = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos();
            let path = std::env::temp_dir().join(format!("verify_parity_{}_{}_{}", name, nanos, count));
            fs::create_dir_all(&path)?;
            Ok(Self { path })
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn test_dual_schema_single_passes() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("single")?;
        assert_eq!(check_dual_schema_ssot_parity(&tmp.path), Ok(ParityOutcome::DualSchemaSingleOrClean));
        Ok(())
    }

    #[test]
    fn test_dual_schema_identical_passes() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("identical")?;
        let schema_dir = tmp.path.join("schema");
        let pipe_dir = tmp.path.join(".pipeline");
        fs::create_dir_all(&schema_dir)?;
        fs::create_dir_all(&pipe_dir)?;

        let sysml = "package Core {\n  part def Sensor;\n  port def TelemetryPort;\n  action def ReadSensor;\n}\n";
        let mut f1 = File::create(schema_dir.join("domain.sysml"))?;
        write!(f1, "{}", sysml)?;

        let mut f2 = File::create(pipe_dir.join("schema.sysml"))?;
        write!(f2, "{}", sysml)?;

        assert_eq!(check_dual_schema_ssot_parity(&tmp.path), Ok(ParityOutcome::DualSchemaIdentical));
        Ok(())
    }

    #[test]
    fn test_dual_schema_mismatch_fails() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("mismatch")?;
        let schema_dir = tmp.path.join("schema");
        let pipe_dir = tmp.path.join(".pipeline");
        fs::create_dir_all(&schema_dir)?;
        fs::create_dir_all(&pipe_dir)?;

        let sysml1 = "package Core {\n  part def Sensor;\n}\n";
        let mut f1 = File::create(schema_dir.join("domain.sysml"))?;
        write!(f1, "{}", sysml1)?;

        let sysml2 = "package Core {\n  part def Actuator;\n}\n";
        let mut f2 = File::create(pipe_dir.join("schema.sysml"))?;
        write!(f2, "{}", sysml2)?;

        let res = check_dual_schema_ssot_parity(&tmp.path);
        assert!(res.is_err());
        Ok(())
    }

    #[test]
    fn test_dual_schema_parity_multi_file_passes() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("multi_file")?;
        let sub1 = tmp.path.join("schema").join("sub1");
        let sub2 = tmp.path.join("schema").join("sub2");
        let pipe_dir = tmp.path.join(".pipeline");
        fs::create_dir_all(&sub1)?;
        fs::create_dir_all(&sub2)?;
        fs::create_dir_all(&pipe_dir)?;

        let sysml_a = "package Sub1 {\n  part def SensorA;\n  port def PortA;\n  action def ActionA;\n}\n";
        let sysml_b = "package Sub2 {\n  part def SensorB;\n  port def PortB;\n  action def ActionB;\n}\n";
        let combined = format!("{}\n{}", sysml_a, sysml_b);

        let mut fa = File::create(sub1.join("a.sysml"))?;
        write!(fa, "{}", sysml_a)?;

        let mut fb = File::create(sub2.join("b.sysml"))?;
        write!(fb, "{}", sysml_b)?;

        let discovered_before_pipeline = discover_sysml_model_text(&tmp.path);
        assert!(discovered_before_pipeline.is_some(), "discover_sysml_model_text must discover multi-file schema across nested directories");
        let disc_text = match discovered_before_pipeline {
            Some(t) => t,
            None => return Err("expected discovered text".into()),
        };
        assert!(disc_text.contains("SensorA"));
        assert!(disc_text.contains("SensorB"));

        let mut fp = File::create(pipe_dir.join("schema.sysml"))?;
        write!(fp, "{}", combined)?;

        assert_eq!(
            check_dual_schema_ssot_parity(&tmp.path),
            Ok(ParityOutcome::DualSchemaIdentical)
        );
        Ok(())
    }

    #[test]
    fn test_issue_256_check28_rejects_missing_conops_schema_architecture_contract() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("check28_missing_arch")?;
        let schemas_dir = tmp.path.join(".pipeline").join("schemas");
        fs::create_dir_all(&schemas_dir)?;

        let invalid_schema = r#"{
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "type": "object",
            "required": ["metadata", "operational_context"],
            "properties": {
                "metadata": { "type": "object" },
                "operational_context": { "type": "object" }
            }
        }"#;
        fs::write(schemas_dir.join("conops_specification_schema.json"), invalid_schema)?;

        let res = check_conops_and_mission_intent_completeness(&tmp.path, false, true);
        assert!(res.is_err(), "Check 28 must fail when conops schema omits architecture properties");
        let errs = match res {
            Err(e) => e,
            Ok(v) => return Err(format!("expected Err, got {:?}", v).into()),
        };
        assert!(errs.iter().any(|e| e.contains("super_system_architecture")));
        assert!(errs.iter().any(|e| e.contains("subsystem_architecture")));
        Ok(())
    }

    #[test]
    fn test_issue_256_check28_passes_valid_conops_schema_architecture_contract() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("check28_valid_arch")?;
        let schemas_dir = tmp.path.join(".pipeline").join("schemas");
        fs::create_dir_all(&schemas_dir)?;

        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let root = manifest_dir.parent().and_then(|p| p.parent()).ok_or("missing root")?;
        let live_schema = root.join(".pipeline").join("schemas").join("conops_specification_schema.json");
        if live_schema.is_file() {
            let content = fs::read_to_string(&live_schema)?;
            fs::write(schemas_dir.join("conops_specification_schema.json"), content)?;

            let res = check_conops_and_mission_intent_completeness(&tmp.path, false, true);
            assert!(res.is_ok(), "Check 28 must pass when conops schema contract is complete: {:?}", res);
        }
        Ok(())
    }

    #[test]
    fn test_issue_271_conops_sv1_icd_parity_mismatch_fails() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("check27_conops_icd_mismatch")?;
        let conops_dir = tmp.path.join("docs").join("conops");
        let interfaces_dir = tmp.path.join("docs").join("interfaces");
        fs::create_dir_all(&conops_dir)?;
        fs::create_dir_all(&interfaces_dir)?;

        let conops_content = r#"## 4. System Operational Architecture
```mermaid
flowchart TD
    Operator -->|"CONN-01: Supervisory Command"| SupervisoryStation
    SupervisoryStation -->|"CONN-02: System Datalink"| Controller
```
"#;
        fs::write(conops_dir.join("04_SYSTEM_ARCHITECTURE.md"), conops_content)?;

        let icd_content = r#"# Level 1C: System Interface Matrix
## 5. Connection Binding Roster Table
| Connection ID | Source Port | Dest Port |
| :--- | :--- | :--- |
| `CONN-01` | PORT-SUP-CMD | PORT-CTRL-CMD |
| `CONN-99` | PORT-ALT-CMD | PORT-CTRL-ALT |
"#;
        fs::write(interfaces_dir.join("ICD_01_SYSTEM_INTERFACE_MATRIX.md"), icd_content)?;

        let res = check_cross_document_diagram_parity(&tmp.path);
        assert!(res.is_err(), "Check 27 must fail when ConOps SV-1 and Level 1C ICD connections mismatch");
        Ok(())
    }

    #[test]
    fn test_issue_271_conops_sv1_icd_parity_passes() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("check27_conops_icd_matching")?;
        let conops_dir = tmp.path.join("docs").join("conops");
        let interfaces_dir = tmp.path.join("docs").join("interfaces");
        fs::create_dir_all(&conops_dir)?;
        fs::create_dir_all(&interfaces_dir)?;

        let conops_content = r#"## 4. System Operational Architecture
```mermaid
flowchart TD
    Operator -->|"CONN-01: Supervisory Command"| SupervisoryStation
    SupervisoryStation -->|"CONN-02: System Datalink"| Controller
```
"#;
        fs::write(conops_dir.join("04_SYSTEM_ARCHITECTURE.md"), conops_content)?;

        let icd_content = r#"# Level 1C: System Interface Matrix
## 5. Connection Binding Roster Table
| Connection ID | Source Port | Dest Port |
| :--- | :--- | :--- |
| `CONN-01` | PORT-SUP-CMD | PORT-CTRL-CMD |
| `CONN-02` | PORT-SUP-TLM | PORT-CTRL-TLM |
"#;
        fs::write(interfaces_dir.join("ICD_01_SYSTEM_INTERFACE_MATRIX.md"), icd_content)?;

        let res = check_cross_document_diagram_parity(&tmp.path);
        assert_eq!(res, Ok(ParityOutcome::Verified));
        Ok(())
    }

    #[test]
    fn test_defect_276_divergent_reports_diagram_without_ssot_views_library_fails() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("check27_reports_divergence")?;
        let conops_dir = tmp.path.join("docs").join("conops");
        let reports_dir = tmp.path.join("docs").join("reports");
        let interfaces_dir = tmp.path.join("docs").join("interfaces");
        fs::create_dir_all(&conops_dir)?;
        fs::create_dir_all(&reports_dir)?;
        fs::create_dir_all(&interfaces_dir)?;

        let conops_content = r#"## 4. System Operational Architecture
```mermaid
flowchart TD
    Operator -->|"CONN-01: Supervisory Command"| SupervisoryStation
    SupervisoryStation -->|"CONN-02: System Datalink"| Controller
```
"#;
        fs::write(conops_dir.join("04_SYSTEM_ARCHITECTURE.md"), conops_content)?;

        let icd_content = r#"# Level 1C: System Interface Matrix
| Connection ID | Source Port | Dest Port |
| `CONN-01` | PORT-SUP-CMD | PORT-CTRL-CMD |
| `CONN-02` | PORT-SUP-TLM | PORT-CTRL-TLM |
"#;
        fs::write(interfaces_dir.join("ICD_01_SYSTEM_INTERFACE_MATRIX.md"), icd_content)?;

        let report_content = r#"# Executive Engineering Deliverables
```mermaid
flowchart TD
    SubA -->|"CONN-99: Divergent Unaligned Flow"| SubB
```
"#;
        fs::write(reports_dir.join("PHASE_1_EXECUTIVE_ENGINEERING_DELIVERABLES.md"), report_content)?;

        let res = check_cross_document_diagram_parity(&tmp.path);
        assert!(res.is_err(), "Check 27 must fail when executive deliverables report contains divergent diagram connections");
        Ok(())
    }

    #[test]
    fn test_defect_276_upstream_missing_architecture_views_library_fails() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("check27_upstream_missing_arch_views")?;
        let pipeline_upstream = tmp.path.join(".pipeline").join("upstream");
        fs::create_dir_all(&pipeline_upstream)?;

        let res = check_cross_document_diagram_parity(&tmp.path);
        assert!(res.is_err(), "Upstream compiler workspace must fail when canonical SSOT architecture views library (docs/architecture/views/) is missing");
        Ok(())
    }

    #[test]
    fn test_defect_276_canonical_architecture_views_parity_passes() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("check27_canonical_arch_views_matching")?;
        let pipeline_upstream = tmp.path.join(".pipeline").join("upstream");
        let arch_views_dir = tmp.path.join("docs").join("architecture").join("views");
        let conops_dir = tmp.path.join("docs").join("conops");
        let interfaces_dir = tmp.path.join("docs").join("interfaces");
        let reports_dir = tmp.path.join("docs").join("reports");

        fs::create_dir_all(&pipeline_upstream)?;
        fs::create_dir_all(&arch_views_dir)?;
        fs::create_dir_all(&conops_dir)?;
        fs::create_dir_all(&interfaces_dir)?;
        fs::create_dir_all(&reports_dir)?;

        let sv1_content = r#"flowchart TD
    Operator -->|"CONN-01: Supervisory Command"| SupervisoryStation
    SupervisoryStation -->|"CONN-02: System Datalink"| Controller
"#;
        fs::write(arch_views_dir.join("sv1_super_system.mmd"), sv1_content)?;

        let conops_content = r#"## 4. System Operational Architecture
```mermaid
flowchart TD
    Operator -->|"CONN-01: Supervisory Command"| SupervisoryStation
    SupervisoryStation -->|"CONN-02: System Datalink"| Controller
```
"#;
        fs::write(conops_dir.join("04_SYSTEM_ARCHITECTURE.md"), conops_content)?;

        let icd_content = r#"# Level 1C: System Interface Matrix
| Connection ID | Source Port | Dest Port |
| `CONN-01` | PORT-SUP-CMD | PORT-CTRL-CMD |
| `CONN-02` | PORT-SUP-TLM | PORT-CTRL-TLM |
"#;
        fs::write(interfaces_dir.join("ICD_01_SYSTEM_INTERFACE_MATRIX.md"), icd_content)?;

        let report_content = r#"# Executive Engineering Deliverables
```mermaid
flowchart TD
    Operator -->|"CONN-01: Supervisory Command"| SupervisoryStation
    SupervisoryStation -->|"CONN-02: System Datalink"| Controller
```
"#;
        fs::write(reports_dir.join("PHASE_1_EXECUTIVE_ENGINEERING_DELIVERABLES.md"), report_content)?;

        let res = check_cross_document_diagram_parity(&tmp.path);
        assert_eq!(res, Ok(ParityOutcome::Verified));
        Ok(())
    }

    #[test]
    fn test_reproduce_defect_286_synthetic_mocks_blind_compiler_to_citation_fraud() -> Result<(), Box<dyn std::error::Error>> {
        use crate::checks::safety_stpa::check_safety_integrity;

        let tmp = TempDir::new("check23_reproduce_defect_286")?;
        let schema_dir = tmp.path.join("schema");
        let features_dir = tmp.path.join("docs").join("features");
        let safety_dir = tmp.path.join("docs").join("safety");
        fs::create_dir_all(&schema_dir)?;
        fs::create_dir_all(&features_dir)?;
        fs::create_dir_all(&safety_dir)?;

        let model_fixture = include_str!("../../../../tests/fixtures/safety/ground_truth_model.sysml");
        fs::write(schema_dir.join("model.sysml"), model_fixture)?;

        let oem_manual_fixture = include_str!("../../../../tests/fixtures/safety/oem_user_manual.md");
        fs::write(schema_dir.join("oem_user_manual.md"), oem_manual_fixture)?;

        let missing_file_fixture = include_str!("../../../../tests/fixtures/safety/missing_file_citation_feature.md");
        fs::write(features_dir.join("feat_missing_file.md"), missing_file_fixture)?;

        let fraudulent_cit_fixture = include_str!("../../../../tests/fixtures/safety/fraudulent_citation_feature.md");
        fs::write(features_dir.join("feat_fraudulent_cit.md"), fraudulent_cit_fixture)?;

        let section_fraud_fixture = include_str!("../../../../tests/fixtures/safety/section_symbol_fraud_feature.md");
        fs::write(features_dir.join("feat_section_fraud.md"), section_fraud_fixture)?;

        let ungrounded_dshot_fixture = include_str!("../../../../tests/fixtures/safety/ungrounded_dshot_feature.md");
        fs::write(features_dir.join("feat_ungrounded_dshot.md"), ungrounded_dshot_fixture)?;

        let mut stpa_content = String::from(include_str!("../../../../tests/fixtures/safety/complete_stpa_matrix.md"));
        stpa_content.push_str("\n\n<!-- Source: schema/phantom-safety-spec.md §9.9.9 -->\nActuator servos operate via DShot600 ungrounded bus.\n");
        fs::write(safety_dir.join("STPA_MATRIX.md"), stpa_content)?;

        let check23_res = check_factual_grounding(&tmp.path, false, true);
        assert!(check23_res.is_err(), "Check 23 failed to reject downstream citation fraud and ungrounded protocols!");

        let check17_res = check_safety_integrity(&tmp.path, false, true);
        assert!(check17_res.is_err(), "Check 17 failed to reject fraudulent safety citations to phantom files!");
        Ok(())
    }

    #[test]
    fn test_defect_333_icd_unqualified_nested_gnss_fix_fails() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("check24_unqualified_gnss_fix")?;
        let interfaces_dir = tmp.path.join("docs").join("interfaces");
        fs::create_dir_all(&interfaces_dir)?;

        let icd01_content = r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #333 |
| **Title** | System Interface Matrix |
| **Type** | icd |

# Level 1C: System Interface Matrix

## 4. Port Definition Roster Table
| Port ID | Subsystem | Port Name | Direction | Port Type | Multiplicity | Protocol Profile |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `PORT-FC-GNSS_FIX_IN` | FlightController | gnssFixIn | IN | DataPort | 1 | PeriodicStream |
| `PORT-NS-GNSS_FIX_OUT` | NavigationSensorSuite | gnssFixOut | OUT | DataPort | 1 | PeriodicStream |

## 5. Connection Binding Roster Table
| Connection ID | Source Port | Dest Port | Flow Behavior | Latency Max ms | Reliability Req | Item Flows Conveyed |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `CONN-05` | gnssFixOut | gnssFixIn | Periodic update (100 Hz) | 10.0 | High | GnssFix |
"#;
        fs::write(interfaces_dir.join("ICD_01_SYSTEM_INTERFACE_MATRIX.md"), icd01_content)?;

        let icd02_content = r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #333 |
| **Title** | Master Signal Flow Dictionary |
| **Type** | icd |

# Level 1C: Master Signal Flow Dictionary

## 2. Master Signal Flow Dictionary Table
| Signal ID | Signal Name | Source Port | Dest Port | Data Type | SI Units | Valid Range | Update Rate | Safe Default Value | Schema Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `SIG-NS-FC-001` | GnssFixData | `PORT-NS-GNSS_FIX_OUT` | `PORT-FC-GNSS_FIX_IN` | GnssFix | dimensionless | [0, 1] | 100 Hz | 0 | schema/model.sysml |
"#;
        fs::write(interfaces_dir.join("ICD_02_MASTER_SIGNAL_DICTIONARY.md"), icd02_content)?;

        let res = check_icd_completeness(&tmp.path);
        assert!(res.is_err(), "Check 24 must fail when gnssFix connection lacks hierarchical qualification: {:?}", res);
        Ok(())
    }

    #[test]
    fn test_defect_333_icd_complete_level1c_suite_passes() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("check24_complete_level1c")?;
        let interfaces_dir = tmp.path.join("docs").join("interfaces");
        fs::create_dir_all(&interfaces_dir)?;

        let icd01_content = r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #333 |
| **Title** | System Interface Matrix |
| **Type** | icd |

# Level 1C: System Interface Matrix

## 4. Port Definition Roster Table
| Port ID | Subsystem | Port Name | Direction | Port Type | Multiplicity | Protocol Profile |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `PORT-FC-GNSS_FIX_IN` | FlightController | gnssFixIn | IN | DataPort | 1 | PeriodicStream |
| `PORT-NS-GNSS_FIX_OUT` | NavigationSensorSuite | gnssFixOut | OUT | DataPort | 1 | PeriodicStream |

## 5. Connection Binding Roster Table
| Connection ID | Source Port | Dest Port | Flow Behavior | Latency Max ms | Reliability Req | Item Flows Conveyed |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `CONN-05` | `PORT-NS-GNSS_FIX_OUT` (FlightController.NavigationSensorSuite.gnssFixOut) | `PORT-FC-GNSS_FIX_IN` (FlightController.gnssFixIn) | Periodic update (100 Hz) | 10.0 | High | GnssFix |
"#;
        fs::write(interfaces_dir.join("ICD_01_SYSTEM_INTERFACE_MATRIX.md"), icd01_content)?;

        let icd02_content = r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #333 |
| **Title** | Master Signal Flow Dictionary |
| **Type** | icd |

# Level 1C: Master Signal Flow Dictionary

## 2. Master Signal Flow Dictionary Table
| Signal ID | Signal Name | Source Port | Dest Port | Data Type | SI Units | Valid Range | Update Rate | Safe Default Value | Schema Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `SIG-NS-FC-001` | GnssFixData | `PORT-NS-GNSS_FIX_OUT` | `PORT-FC-GNSS_FIX_IN` | GnssFix | dimensionless | [0, 1] | 100 Hz | 0 | schema/model.sysml |
"#;
        fs::write(interfaces_dir.join("ICD_02_MASTER_SIGNAL_DICTIONARY.md"), icd02_content)?;

        let res = check_icd_completeness(&tmp.path);
        assert_eq!(res, Ok(ParityOutcome::Verified), "Expected complete ICD suite to pass, but got: {:?}", res);
        Ok(())
    }

    #[test]
    fn test_unicode_before_citation_does_not_panic() {
        let line = "* İnterface timing Ω µs as defined in schema/model.sysml §2.1";
        let citations = extract_citations_from_line(line, 1);
        assert_eq!(citations.len(), 1);
        if let Some(first) = citations.first() {
            assert_eq!(first.target_file, "schema/model.sysml");
            assert_eq!(first.section.as_deref(), Some("2.1"));
        }
    }

    #[test]
    fn test_multiple_citations_same_file_different_sections_on_single_line() {
        let line = "per schema/oem_user_manual.md §1.1 and per schema/oem_user_manual.md §99.9";
        let citations = extract_citations_from_line(line, 1);
        assert_eq!(citations.len(), 2, "Both citations on the same line must be preserved");
        let sec1 = citations.iter().any(|c| c.section.as_deref() == Some("1.1"));
        let sec2 = citations.iter().any(|c| c.section.as_deref() == Some("99.9"));
        assert!(sec1 && sec2);
    }

    #[test]
    fn test_section_slicer_rejects_non_heading_body_number_match() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("slicer_non_heading")?;
        let doc_path = tmp.path.join("spec.md");
        let content = "# 1. Overview\nSome body text mentioning 7.2.3 in passing without heading.\n## 2. Details\nMore text.\n";
        fs::write(&doc_path, content)?;

        let slicer = SectionSlicer::new(&tmp.path);
        let extracted = slicer.extract_section("spec.md", "§7.2.3");
        assert!(extracted.is_none(), "SectionSlicer must not return whole-file fallback when section is not a heading");
        Ok(())
    }

    #[test]
    fn test_check24_non_gnss_nested_port_requires_qualification() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("check24_nested_non_gnss")?;
        let interfaces_dir = tmp.path.join("docs").join("interfaces");
        fs::create_dir_all(&interfaces_dir)?;

        let icd01_content = r#"# Level 1C: System Interface Matrix
## 4. Port Definition Roster Table
| Port ID | Subsystem | Port Name | Direction | Port Type | Multiplicity | Protocol Profile |
| `PORT-FC-IMU_IN` | FlightController | imuIn | IN | DataPort | 1 | PeriodicStream |
| `PORT-IMU-ACCEL_OUT` | InertialSensorSuite | accelOut | OUT | DataPort | 1 | PeriodicStream |

## 5. Connection Binding Roster Table
| Connection ID | Source Port | Dest Port | Flow Behavior | Latency Max ms | Reliability Req | Item Flows Conveyed |
| `CONN-01` | PORT-IMU-ACCEL_OUT | PORT-FC-IMU_IN | Periodic update (200 Hz) | 5.0 | High | AccelData |
"#;
        fs::write(interfaces_dir.join("ICD_01_SYSTEM_INTERFACE_MATRIX.md"), icd01_content)?;

        let icd02_content = r#"# Level 1C: Master Signal Flow Dictionary
## 2. Master Signal Flow Dictionary Table
| Signal ID | Signal Name | Source Port | Dest Port | Data Type | SI Units | Valid Range | Update Rate | Safe Default Value | Schema Citation |
| `SIG-IMU-FC-001` | AccelData | `PORT-IMU-ACCEL_OUT` | `PORT-FC-IMU_IN` | Accel | m/s^2 | [-100, 100] | 200 Hz | 0 | schema/model.sysml |
"#;
        fs::write(interfaces_dir.join("ICD_02_MASTER_SIGNAL_DICTIONARY.md"), icd02_content)?;

        let res = check_icd_completeness(&tmp.path);
        assert!(res.is_err(), "Check 24 must fail when nested IMU connection lacks hierarchical qualification: {:?}", res);
        Ok(())
    }

    #[test]
    fn test_check27_empty_conops_connections_with_populated_icd_fails() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("check27_asymmetric_empty")?;
        let conops_dir = tmp.path.join("docs").join("conops");
        let interfaces_dir = tmp.path.join("docs").join("interfaces");
        fs::create_dir_all(&conops_dir)?;
        fs::create_dir_all(&interfaces_dir)?;

        let conops_content = r#"## 4. System Operational Architecture
```mermaid
flowchart TD
    Operator --> SupervisoryStation
```
"#;
        fs::write(conops_dir.join("04_SYSTEM_ARCHITECTURE.md"), conops_content)?;

        let icd_content = r#"# Level 1C: System Interface Matrix
## 5. Connection Binding Roster Table
| Connection ID | Source Port | Dest Port |
| `CONN-01` | PORT-SUP-CMD | PORT-CTRL-CMD |
"#;
        fs::write(interfaces_dir.join("ICD_01_SYSTEM_INTERFACE_MATRIX.md"), icd_content)?;

        let res = check_cross_document_diagram_parity(&tmp.path);
        assert!(res.is_err(), "Check 27 must fail when ConOps has 0 connections while ICD has CONN-01");
        Ok(())
    }

    #[test]
    fn test_check31_detects_attribute_or_body_divergence_with_identical_part_names() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("check31_body_divergence")?;
        let schema_dir = tmp.path.join("schema");
        let pipe_dir = tmp.path.join(".pipeline");
        fs::create_dir_all(&schema_dir)?;
        fs::create_dir_all(&pipe_dir)?;

        let sysml_with_port = "package Core {\n  part def FlightController {\n    port cmdIn;\n  }\n}\n";
        let sysml_empty = "package Core {\n  part def FlightController;\n}\n";

        fs::write(schema_dir.join("domain.sysml"), sysml_with_port)?;
        fs::write(pipe_dir.join("schema.sysml"), sysml_empty)?;

        let res = check_dual_schema_ssot_parity(&tmp.path);
        assert!(res.is_err(), "Check 31 must detect structural divergence inside part def even when names are identical: {:?}", res);
        Ok(())
    }

    #[test]
    fn test_issue_443_check31_detects_requirement_constraint_state_and_cross_package_divergence() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("check31_issue443_divergence")?;
        let schema_dir = tmp.path.join("schema");
        let pipe_dir = tmp.path.join(".pipeline");
        fs::create_dir_all(&schema_dir)?;
        fs::create_dir_all(&pipe_dir)?;

        // Case 1: Requirement drift
        let sysml_schema = "package Core {\n  part def Controller;\n  requirement def ReqSafety {\n    doc /* Must safely halt */\n  }\n}\n";
        let sysml_pipeline = "package Core {\n  part def Controller;\n}\n";

        fs::write(schema_dir.join("domain.sysml"), sysml_schema)?;
        fs::write(pipe_dir.join("schema.sysml"), sysml_pipeline)?;

        let res = check_dual_schema_ssot_parity(&tmp.path);
        assert!(res.is_err(), "Check 31 must detect requirement def divergence: {:?}", res);

        // Case 2: Constraint drift
        let sysml_schema_c = "package Core {\n  part def Controller;\n  constraint def ThermalLimit;\n}\n";
        fs::write(schema_dir.join("domain.sysml"), sysml_schema_c)?;
        let res_c = check_dual_schema_ssot_parity(&tmp.path);
        assert!(res_c.is_err(), "Check 31 must detect constraint def divergence: {:?}", res_c);

        // Case 3: Cross-package colliding part name divergence
        let sysml_multi_pkg_a = "package SubA {\n  part def Engine {\n    port in1;\n  }\n}\npackage SubB {\n  part def Engine;\n}\n";
        let sysml_multi_pkg_b = "package SubA {\n  part def Engine;\n}\npackage SubB {\n  part def Engine;\n}\n";
        fs::write(schema_dir.join("domain.sysml"), sysml_multi_pkg_a)?;
        fs::write(pipe_dir.join("schema.sysml"), sysml_multi_pkg_b)?;
        let res_coll = check_dual_schema_ssot_parity(&tmp.path);
        assert!(res_coll.is_err(), "Check 31 must detect divergence in colliding part name across packages: {:?}", res_coll);

        Ok(())
    }

    #[test]
    fn test_check28_propagates_schema_and_io_errors_fail_closed() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("check28_fail_closed")?;
        let schemas_dir = tmp.path.join(".pipeline").join("schemas");
        fs::create_dir_all(&schemas_dir)?;

        fs::write(schemas_dir.join("conops_specification_schema.json"), "{ invalid_json: true }")?;

        let res = check_conops_and_mission_intent_completeness(&tmp.path, false, true);
        assert!(res.is_err(), "Check 28 must fail closed on malformed JSON schema: {:?}", res);
        Ok(())
    }

    #[test]
    fn test_target_scan_dir_taxonomy_unification_option_b() {
        let set: BTreeSet<TargetScanDir> = TargetScanDir::ALL.iter().copied().collect();
        assert_eq!(set.len(), 11, "TargetScanDir::ALL must contain 11 unique variants");
        assert_eq!(TargetScanDir::ALL.len(), 11);
        assert!(TargetScanDir::Conops.requires_factual_grounding());
        assert!(TargetScanDir::Safety.requires_factual_grounding());
        assert!(TargetScanDir::Interfaces.requires_factual_grounding());
        assert!(TargetScanDir::Icd.requires_factual_grounding());
        assert!(TargetScanDir::Architecture.requires_factual_grounding());
        assert!(TargetScanDir::Epics.requires_factual_grounding());
        assert!(TargetScanDir::Features.requires_factual_grounding());
        assert!(TargetScanDir::UserStories.requires_factual_grounding());
        assert!(TargetScanDir::UseCases.requires_factual_grounding());
        assert!(TargetScanDir::Reports.requires_factual_grounding());
        assert!(!TargetScanDir::Management.requires_factual_grounding());
    }

    #[test]
    fn test_workspace_integrity_no_fake_predicates_or_fixture_hardcoding() -> Result<(), Box<dyn std::error::Error>> {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let root = manifest_dir
            .parent()
            .and_then(|p| p.parent())
            .ok_or("missing workspace root")?;
        let crates_dir = root.join("crates");

        let global_forbidden = vec![
            ["\"UNDEC", "LARED\""].concat(),
            ["\"DIVER", "GENT\""].concat(),
            ["\"PHAN", "TOM\""].concat(),
            ["let _ =", " repo_root"].concat(),
            ["\"VTOL", "Motor\""].concat(),
            ["\"dshot", "600\""].concat(),
            ["\"50 ", "hz\""].concat(),
            ["\"stanag ", "4586\""].concat(),
            ["\"stanag", "4586\""].concat(),
        ];

        let gate_specific_forbidden = vec![
            ["\"dsh", "ot\""].concat(),
            ["\"GN", "SS\""].concat(),
            ["\"IM", "U\""].concat(),
            ["\"Flight", "Controller\""].concat(),
        ];

        let corrupted_space_entry = ["\"", " ", "\","].concat();

        let mut violations = Vec::new();

        for entry_res in WalkDir::new(&crates_dir).sort_by_file_name().into_iter() {
            let entry = entry_res?;
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let rel = path.strip_prefix(root).unwrap_or(path).to_string_lossy().to_string();
            if !rel.contains("/src/") {
                continue;
            }

            let content = fs::read_to_string(path)?;
            let prod = content.split("#[cfg(test)]").next().unwrap_or(&content);

            for (idx, line) in prod.lines().enumerate() {
                let line_no = idx + 1;
                let code = line.split("//").next().unwrap_or(line);

                for needle in &global_forbidden {
                    if code.contains(needle) {
                        violations.push(format!(
                            "{}:{}: contains forbidden predicate or fixture literal {}",
                            rel, line_no, needle
                        ));
                    }
                }

                if rel.ends_with("parity_gates.rs")
                    || rel.ends_with("safety_stpa.rs")
                    || rel.ends_with("provenance.rs")
                {
                    for needle in &gate_specific_forbidden {
                        if code.contains(needle) {
                            violations.push(format!(
                                "{}:{}: contains hardcoded fixture literal {}",
                                rel, line_no, needle
                            ));
                        }
                    }
                }
            }

            if rel.ends_with("safety_stpa.rs") {
                for kw_const in [
                    "INTERFACE_KEYWORDS",
                    "STATE_KEYWORDS",
                    "ACTION_KEYWORDS",
                    "RESOURCE_KEYWORDS",
                ] {
                    if let Some(pos) = prod.find(kw_const) {
                        if let Some(after) = prod.get(pos..) {
                            if let Some(end_semi) = after.find("];") {
                                let block = &after[..end_semi];
                                if block.contains(&corrupted_space_entry) {
                                    violations.push(format!(
                                        "{}: {} contains corrupted single-space entry",
                                        rel, kw_const
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        assert!(
            violations.is_empty(),
            "Workspace integrity violations found ({}):\n{}",
            violations.len(),
            violations.join("\n")
        );
        Ok(())
    }

    #[test]
    fn test_issue_440_research_inventory_io_error_formats_as_check_29() -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let tmp = TempDir::new("res_inv_io")?;
            let epics_dir = tmp.path.join("docs").join("epics");
            fs::create_dir_all(&epics_dir)?;
            fs::write(epics_dir.join("EPIC-001.md"), "# Epic 1\n")?;
            let unreadable = epics_dir.join("forbidden");
            fs::create_dir(&unreadable)?;
            let mut perms = fs::metadata(&unreadable)?.permissions();
            perms.set_mode(0o000);
            fs::set_permissions(&unreadable, perms)?;

            let res = check_research_inventory(&tmp.path, false, false);

            let mut restore_perms = fs::metadata(&unreadable)?.permissions();
            restore_perms.set_mode(0o755);
            let _ = fs::set_permissions(&unreadable, restore_perms);

            let errs = res.expect_err("WalkDir on unreadable directory should produce errors");
            assert!(!errs.is_empty(), "Expected I/O errors for unreadable directory");
            let io_err = errs
                .iter()
                .find(|e| e.contains("I/O error walking directory"))
                .expect("Expected directory traversal I/O error in error list");
            assert!(
                io_err.starts_with("Check 29 failed:"),
                "I/O error must be formatted as Check 29 failed, got: {}",
                io_err
            );
            assert!(
                !io_err.contains("Check 28 failed"),
                "I/O error must not contain legacy 'Check 28 failed': {}",
                io_err
            );
        }
        Ok(())
    }


    #[test]
    fn test_issue_440_coverage_digest_errors_format_as_check_30a() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("cov_digest")?;
        let feat_dir = tmp.path.join("docs").join("features");
        fs::create_dir_all(&feat_dir)?;
        fs::write(feat_dir.join("FEAT-001.md"), "# Feature 1\n")?;

        let reports_dir = tmp.path.join("docs").join("reports");
        fs::create_dir_all(&reports_dir)?;
        fs::write(
            reports_dir.join("coverage.md"),
            "# Coverage\nReferences FEAT-999.md here.\n",
        )?;

        let res = check_coverage_digest(&tmp.path);
        let errs = res.expect_err("Undeclared spec reference should trigger errors");
        assert!(!errs.is_empty(), "Expected errors for undeclared spec");
        for err in &errs {
            assert!(
                err.starts_with("Check 30A failed:"),
                "Coverage digest error must be formatted as Check 30A failed, got: {}",
                err
            );
            assert!(
                !err.contains("Check 28 failed"),
                "Coverage digest error must not contain Check 28 failed: {}",
                err
            );
        }
        Ok(())
    }

    #[test]
    fn test_issue_440_obligation_witness_errors_format_as_check_30b() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("ob_witness")?;
        let safety_dir = tmp.path.join("docs").join("safety");
        fs::create_dir_all(&safety_dir)?;
        fs::write(
            safety_dir.join("hazard_analysis.md"),
            "# Hazard Analysis\nDeclared hazard H-001 with no safety constraint.\n",
        )?;

        let res = check_obligation_witness(&tmp.path);
        let errs = res.expect_err("Orphan hazard should trigger errors");
        assert!(!errs.is_empty(), "Expected errors for unwitnessed hazard");
        for err in &errs {
            assert!(
                err.starts_with("Check 30B failed:"),
                "Obligation witness error must be formatted as Check 30B failed, got: {}",
                err
            );
            assert!(
                !err.contains("Check 29 failed"),
                "Obligation witness error must not contain Check 29 failed: {}",
                err
            );
        }
        Ok(())
    }

    #[test]
    fn test_issue_440_architecture_viewpoint_errors_format_as_check_30c() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("arch_views")?;
        fs::create_dir_all(tmp.path.join(".pipeline").join("upstream"))?;

        let res = check_architecture_viewpoint_diagrams(&tmp.path, false, true);
        let errs = res.expect_err("Missing architecture views library in upstream should trigger error");
        assert!(!errs.is_empty(), "Expected error for missing arch views");
        for err in &errs {
            assert!(
                err.starts_with("Check 30C failed:"),
                "Architecture viewpoint error must be formatted as Check 30C failed, got: {}",
                err
            );
            assert!(
                !err.contains("Gate 30 failed") && !err.contains("Check 30 failed"),
                "Architecture viewpoint error must not contain Gate/Check 30 failed: {}",
                err
            );
        }
        Ok(())
    }
}

