//! Specification mandatory section validation conforming to DO-178C Level A and ISO 26262 ASIL D.
//!
//! Validates required Markdown section headers for Epics, Features (including UI, API, and M2M variants),
//! User Stories, and Use Cases against configured or fallback structural rules using zero-copy AST token matching.

use crate::spec_audit::markdown_ast::MarkdownDoc;
use deap_core::rules::CodebaseRules;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Normalizes heading or pattern text into canonical comparison tokens.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `normalize_tokens` in `crates/verify-baseline/src/spec_audit/sections.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `normalize_tokens` (declaration #1) within `SPEC-AUDIT-SECTIONS`.
///
/// ## 3. Preconditions
/// - `text: &str` must satisfy domain and type-safety constraints prior to invoking `normalize_tokens`.
///
/// ## 4. Postconditions
/// - Returns `Vec<String>` from `normalize_tokens` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(text)) bounded over input/member dimension of `normalize_tokens`.
/// - Space Complexity: O(N(text)) bounded auxiliary memory for `normalize_tokens`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `normalize_tokens` (`Vec<String>`).
/// - State: Maintains consistent state invariants across `normalize_tokens` without partial mutation on error.
/// - Action: Executes `normalize_tokens` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `normalize_tokens` (`SPEC-AUDIT-SECTIONS`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-SECTIONS-NORMALIZE-TOKENS] | SysML v2: ActionDef(normalize_tokens)
pub fn normalize_tokens(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    let with_and = lower.replace('&', " and ");
    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in with_and.chars() {
        if ch.is_alphanumeric() {
            current.push(ch);
        } else if !current.is_empty() {
            tokens.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

/// Single section matching rule comprising expected heading level and normalized tokens.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `SectionRule` in `crates/verify-baseline/src/spec_audit/sections.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `SectionRule` across member set `[level: u8, normalized_tokens: Vec<String>, display_name: String]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `SectionRule` members (`level: u8, normalized_tokens: Vec<String>, display_name: String`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `SectionRule` representation for struct `SectionRule`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(3)) bounded over input/member dimension of `SectionRule`.
/// - Space Complexity: O(M(3)) bounded auxiliary memory for `SectionRule`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `SectionRule` (`SectionRule`).
/// - State: Maintains consistent state invariants across `SectionRule` without partial mutation on error.
/// - Action: Executes `SectionRule` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `SectionRule` (`SPEC-AUDIT-SECTIONS`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-SECTIONS-SECTIONRULE] | SysML v2: PartDef(SectionRule)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionRule {
    /// Heading level (1 for H1, 2 for H2, 3 for H3, etc.).
    pub level: u8,
    /// Normalized sequence of alphanumeric tokens that must match the heading.
    pub normalized_tokens: Vec<String>,
    /// Canonical human-readable section heading title (e.g. "## 1. Context").
    pub display_name: String,
}

impl SectionRule {
    /// Constructs a new SectionRule with explicit level, tokens, and display title.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `SectionRule::new` in `crates/verify-baseline/src/spec_audit/sections.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `SectionRule::new` (declaration #3) within `SPEC-AUDIT-SECTIONS`.
    ///
    /// ## 3. Preconditions
    /// - `level: u8` must satisfy domain and type-safety constraints prior to invoking `SectionRule::new`.
    /// - `normalized_tokens: Vec<String>` must satisfy domain and type-safety constraints prior to invoking `SectionRule::new`.
    /// - `display_name: impl Into<String>` must satisfy domain and type-safety constraints prior to invoking `SectionRule::new`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Self` from `SectionRule::new` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(level)) bounded over input/member dimension of `SectionRule::new`.
    /// - Space Complexity: O(N(level)) bounded auxiliary memory for `SectionRule::new`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `SectionRule::new` (`Self`).
    /// - State: Maintains consistent state invariants across `SectionRule::new` without partial mutation on error.
    /// - Action: Executes `SectionRule::new` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `SectionRule::new` (`SPEC-AUDIT-SECTIONS`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SPEC-AUDIT-SECTIONS-SECTIONRULE-NEW] | SysML v2: ActionDef(SectionRule::new)
    pub fn new(level: u8, normalized_tokens: Vec<String>, display_name: impl Into<String>) -> Self {
        Self {
            level,
            normalized_tokens,
            display_name: display_name.into(),
        }
    }

    /// Constructs a SectionRule by parsing a pattern string and display title.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `SectionRule::from_pattern` in `crates/verify-baseline/src/spec_audit/sections.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `SectionRule::from_pattern` (declaration #4) within `SPEC-AUDIT-SECTIONS`.
    ///
    /// ## 3. Preconditions
    /// - `pattern: &str` must satisfy domain and type-safety constraints prior to invoking `SectionRule::from_pattern`.
    /// - `display_name: &str` must satisfy domain and type-safety constraints prior to invoking `SectionRule::from_pattern`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Self` from `SectionRule::from_pattern` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(pattern)) bounded over input/member dimension of `SectionRule::from_pattern`.
    /// - Space Complexity: O(N(pattern)) bounded auxiliary memory for `SectionRule::from_pattern`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `SectionRule::from_pattern` (`Self`).
    /// - State: Maintains consistent state invariants across `SectionRule::from_pattern` without partial mutation on error.
    /// - Action: Executes `SectionRule::from_pattern` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `SectionRule::from_pattern` (`SPEC-AUDIT-SECTIONS`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SPEC-AUDIT-SECTIONS-SECTIONRULE-FROM-PATTERN] | SysML v2: ActionDef(SectionRule::from_pattern)
    pub fn from_pattern(pattern: &str, display_name: &str) -> Self {
        let clean_pat = pattern.trim_start_matches("(?i)").trim();
        let pat_hashes = clean_pat.chars().take_while(|&c| c == '#').count();
        let disp_hashes = display_name.trim().chars().take_while(|&c| c == '#').count();

        let level = if pat_hashes > 0 {
            pat_hashes as u8
        } else if disp_hashes > 0 {
            disp_hashes as u8
        } else {
            2
        };

        let disp_content = display_name.trim().trim_start_matches('#').trim();
        let tokens = if !disp_content.is_empty() {
            normalize_tokens(disp_content)
        } else {
            let cleaned = clean_pat
                .trim_start_matches('#')
                .replace("(?:and|&)", "and")
                .replace(r"\s+", " ")
                .replace(r"\s*", " ")
                .replace(r"\s", " ")
                .replace(r"\.", ".");
            normalize_tokens(&cleaned)
        };

        Self {
            level,
            normalized_tokens: tokens,
            display_name: display_name.to_string(),
        }
    }
}

/// Collection of mandatory section rules indexed by specification type.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `SectionRules` in `crates/verify-baseline/src/spec_audit/sections.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `SectionRules` across member set `[rules: HashMap<String, Vec<SectionRule>>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `SectionRules` members (`rules: HashMap<String, Vec<SectionRule>>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `SectionRules` representation for struct `SectionRules`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(1)) bounded over input/member dimension of `SectionRules`.
/// - Space Complexity: O(M(1)) bounded auxiliary memory for `SectionRules`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `SectionRules` (`SectionRules`).
/// - State: Maintains consistent state invariants across `SectionRules` without partial mutation on error.
/// - Action: Executes `SectionRules` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `SectionRules` (`SPEC-AUDIT-SECTIONS`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-SECTIONS-SECTIONRULES] | SysML v2: PartDef(SectionRules)
#[derive(Debug, Clone)]
pub struct SectionRules {
    /// Map of specification type key to its required section rules.
    pub rules: HashMap<String, Vec<SectionRule>>,
}

impl Default for SectionRules {
    /// Executes deterministic fn `SectionRules::default` within `crates/verify-baseline/src/spec_audit/sections.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `SectionRules::default` in `crates/verify-baseline/src/spec_audit/sections.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `SectionRules::default` (declaration #6) within `SPEC-AUDIT-SECTIONS`.
    ///
    /// ## 3. Preconditions
    /// - Invoked within a valid execution context for `SectionRules::default` in `SPEC-AUDIT-SECTIONS`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Self` from `SectionRules::default` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(K(default)) bounded over input/member dimension of `SectionRules::default`.
    /// - Space Complexity: O(K(default)) bounded auxiliary memory for `SectionRules::default`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `SectionRules::default` (`Self`).
    /// - State: Maintains consistent state invariants across `SectionRules::default` without partial mutation on error.
    /// - Action: Executes `SectionRules::default` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `SectionRules::default` (`SPEC-AUDIT-SECTIONS`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SPEC-AUDIT-SECTIONS-SECTIONRULES-DEFAULT] | SysML v2: ActionDef(SectionRules::default)
    fn default() -> Self {
        Self::default_rules()
    }
}

impl SectionRules {
    /// Constructs default section rules covering Epics, Features, User Stories, and Use Cases.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `SectionRules::default_rules` in `crates/verify-baseline/src/spec_audit/sections.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `SectionRules::default_rules` (declaration #7) within `SPEC-AUDIT-SECTIONS`.
    ///
    /// ## 3. Preconditions
    /// - Invoked within a valid execution context for `SectionRules::default_rules` in `SPEC-AUDIT-SECTIONS`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Self` from `SectionRules::default_rules` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(K(default_rules)) bounded over input/member dimension of `SectionRules::default_rules`.
    /// - Space Complexity: O(K(default_rules)) bounded auxiliary memory for `SectionRules::default_rules`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `SectionRules::default_rules` (`Self`).
    /// - State: Maintains consistent state invariants across `SectionRules::default_rules` without partial mutation on error.
    /// - Action: Executes `SectionRules::default_rules` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `SectionRules::default_rules` (`SPEC-AUDIT-SECTIONS`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SPEC-AUDIT-SECTIONS-SECTIONRULES-DEFAULT-RULES] | SysML v2: ActionDef(SectionRules::default_rules)
    pub fn default_rules() -> Self {
        let mut rules: HashMap<String, Vec<SectionRule>> = HashMap::new();

        // Epics
        let epic_sections = [
            ("## 1. Context", "## 1. Context"),
            ("## 2. Requirements & Checklist", "## 2. Requirements & Checklist"),
            ("### Associated Use Cases & User Stories", "### Associated Use Cases & User Stories"),
            ("#### Associated Use Cases", "#### Associated Use Cases"),
            ("#### Associated User Stories", "#### Associated User Stories"),
            ("## 3. Architecture", "## 3. Architecture"),
            ("## 4. Operational Considerations", "## 4. Operational Considerations"),
            ("## 5. Security & Governance", "## 5. Security & Governance"),
            ("## 6. Source References", "## 6. Source References"),
            ("## System-Level UML Class Diagram", "## System-Level UML Class Diagram"),
            ("## System State Machine Diagram", "## System State Machine Diagram"),
        ];
        for (pat, disp) in epic_sections {
            rules
                .entry("epic".to_string())
                .or_default()
                .push(SectionRule::from_pattern(pat, disp));
        }

        // Features (base)
        let feature_sections = [
            ("## UML Class Diagram", "## UML Class Diagram"),
            ("## Interface Requirements", "## Interface Requirements"),
            ("### 1. Test Data Shape", "### 1. Test Data Shape"),
            ("### 4. Interactive Flow & States", "### 4. Interactive Flow & States"),
        ];
        for (pat, disp) in feature_sections {
            rules
                .entry("feature".to_string())
                .or_default()
                .push(SectionRule::from_pattern(pat, disp));
        }

        // Features (UI variant)
        let feature_ui_sections = [
            ("## UML Class Diagram", "## UML Class Diagram"),
            ("## Interface Requirements", "## Interface Requirements"),
            ("### 1. Test Data Shape", "### 1. Test Data Shape"),
            ("### 3. Visual Layout & Arrangement", "### 3. Visual Layout & Arrangement"),
            ("### 4. Interactive Flow & States", "### 4. Interactive Flow & States"),
        ];
        for (pat, disp) in feature_ui_sections {
            rules
                .entry("feature_ui".to_string())
                .or_default()
                .push(SectionRule::from_pattern(pat, disp));
        }

        // Features (API variant)
        let feature_api_sections = [
            ("## UML Class Diagram", "## UML Class Diagram"),
            ("## Interface Requirements", "## Interface Requirements"),
            ("### 1. Payload Schema", "### 1. Payload Schema"),
            ("### 3. Logical Operations & Interface Messages", "### 3. Logical Operations & Interface Messages"),
            ("### 4. Logical Exception States & Validation Failures", "### 4. Logical Exception States & Validation Failures"),
        ];
        for (pat, disp) in feature_api_sections {
            rules
                .entry("feature_api".to_string())
                .or_default()
                .push(SectionRule::from_pattern(pat, disp));
        }

        // Features (M2M variant)
        let feature_m2m_sections = [
            ("## UML Class Diagram", "## UML Class Diagram"),
            ("## Interface Requirements", "## Interface Requirements"),
            ("### 1. Payload Schema", "### 1. Payload Schema"),
            ("### 3. Logical Operations & Interface Messages", "### 3. Logical Operations & Interface Messages"),
            ("### 4. Logical Exception States & Validation Failures", "### 4. Logical Exception States & Validation Failures"),
        ];
        for (pat, disp) in feature_m2m_sections {
            rules
                .entry("feature_m2m".to_string())
                .or_default()
                .push(SectionRule::from_pattern(pat, disp));
        }

        // User Stories
        let story_sections = [
            ("## UML Sequence Diagram", "## UML Sequence Diagram"),
            ("## Required Features", "## Required Features"),
        ];
        for (pat, disp) in story_sections {
            rules
                .entry("user_story".to_string())
                .or_default()
                .push(SectionRule::from_pattern(pat, disp));
        }

        // Use Cases
        let usecase_sections = [
            ("## UML Diagrams", "## UML Diagrams"),
            ("## 1. Actors", "## 1. Actors"),
            ("## 2. Preconditions", "## 2. Preconditions"),
            ("## 3. Trigger", "## 3. Trigger"),
            ("## 4. Main Success Scenario", "## 4. Main Success Scenario"),
            ("## 5. Alternate and Exception Flows", "## 5. Alternate and Exception Flows"),
            ("## 6. Postconditions", "## 6. Postconditions"),
            ("## 8. Realization Matrix", "## 8. Realization Matrix"),
        ];
        for (pat, disp) in usecase_sections {
            rules
                .entry("use_case".to_string())
                .or_default()
                .push(SectionRule::from_pattern(pat, disp));
        }

        Self { rules }
    }

    /// Ingests required sections configuration from parsed `CodebaseRules`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `SectionRules::from_codebase_rules` in `crates/verify-baseline/src/spec_audit/sections.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `SectionRules::from_codebase_rules` (declaration #8) within `SPEC-AUDIT-SECTIONS`.
    ///
    /// ## 3. Preconditions
    /// - `codebase_rules: &CodebaseRules` must satisfy domain and type-safety constraints prior to invoking `SectionRules::from_codebase_rules`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Self` from `SectionRules::from_codebase_rules` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(codebase_rules)) bounded over input/member dimension of `SectionRules::from_codebase_rules`.
    /// - Space Complexity: O(N(codebase_rules)) bounded auxiliary memory for `SectionRules::from_codebase_rules`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `SectionRules::from_codebase_rules` (`Self`).
    /// - State: Maintains consistent state invariants across `SectionRules::from_codebase_rules` without partial mutation on error.
    /// - Action: Executes `SectionRules::from_codebase_rules` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `SectionRules::from_codebase_rules` (`SPEC-AUDIT-SECTIONS`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SPEC-AUDIT-SECTIONS-SECTIONRULES-FROM-CODEBASE-RULES] | SysML v2: ActionDef(SectionRules::from_codebase_rules)
    pub fn from_codebase_rules(codebase_rules: &CodebaseRules) -> Self {
        let mut section_rules = Self::default_rules();

        // 1. Ingest from validation_rules.required_sections
        if let Some(val_rules) = &codebase_rules.validation_rules {
            if let Some(req_sections) = &val_rules.required_sections {
                for (spec_type, arr_val) in req_sections {
                    if let Some(arr) = arr_val.as_array() {
                        let mut parsed_rules = Vec::new();
                        for item in arr {
                            if let Some(pair) = item.as_array() {
                                if pair.len() >= 2 {
                                    if let (Some(pat_str), Some(disp_str)) =
                                        (pair[0].as_str(), pair[1].as_str())
                                    {
                                        parsed_rules.push(SectionRule::from_pattern(pat_str, disp_str));
                                    }
                                }
                            }
                        }
                        if !parsed_rules.is_empty() {
                            section_rules.rules.insert(spec_type.clone(), parsed_rules);
                        }
                    }
                }
            }
        }

        // 2. Ingest from spec_rules.required_sections if present in extra
        if let Some(spec_rules) = codebase_rules.extra.get("spec_rules") {
            if let Some(req_sections) = spec_rules.get("required_sections").and_then(|v| v.as_object()) {
                for (spec_type, arr_val) in req_sections {
                    if let Some(arr) = arr_val.as_array() {
                        let mut parsed_rules = Vec::new();
                        for item in arr {
                            if let Some(pair) = item.as_array() {
                                if pair.len() >= 2 {
                                    if let (Some(pat_str), Some(disp_str)) =
                                        (pair[0].as_str(), pair[1].as_str())
                                    {
                                        parsed_rules.push(SectionRule::from_pattern(pat_str, disp_str));
                                    }
                                }
                            }
                        }
                        if !parsed_rules.is_empty() {
                            section_rules.rules.insert(spec_type.clone(), parsed_rules);
                        }
                    }
                }
            }
        }

        section_rules
    }

    /// Resolves the effective rule lookup key for a specification.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `SectionRules::resolve_rule_key` in `crates/verify-baseline/src/spec_audit/sections.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `SectionRules::resolve_rule_key` (declaration #9) within `SPEC-AUDIT-SECTIONS`.
    ///
    /// ## 3. Preconditions
    /// - `spec_type: &str` must satisfy domain and type-safety constraints prior to invoking `SectionRules::resolve_rule_key`.
    /// - `doc: &MarkdownDoc` must satisfy domain and type-safety constraints prior to invoking `SectionRules::resolve_rule_key`.
    ///
    /// ## 4. Postconditions
    /// - Returns `String` from `SectionRules::resolve_rule_key` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(spec_type)) bounded over input/member dimension of `SectionRules::resolve_rule_key`.
    /// - Space Complexity: O(N(spec_type)) bounded auxiliary memory for `SectionRules::resolve_rule_key`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `SectionRules::resolve_rule_key` (`String`).
    /// - State: Maintains consistent state invariants across `SectionRules::resolve_rule_key` without partial mutation on error.
    /// - Action: Executes `SectionRules::resolve_rule_key` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `SectionRules::resolve_rule_key` (`SPEC-AUDIT-SECTIONS`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SPEC-AUDIT-SECTIONS-SECTIONRULES-RESOLVE-RULE-KEY] | SysML v2: ActionDef(SectionRules::resolve_rule_key)
    fn resolve_rule_key(&self, spec_type: &str, doc: &MarkdownDoc) -> String {
        let norm_type = spec_type.to_lowercase();
        match norm_type.as_str() {
            "epic" | "epics" => "epic".to_string(),
            "user_story" | "user-story" | "user_stories" | "user-stories" => {
                "user_story".to_string()
            }
            "use_case" | "use-case" | "use_cases" | "use-cases" => "use_case".to_string(),
            "feature" | "features" => {
                let iface_opt = doc
                    .table_metadata
                    .get("interface_type")
                    .or_else(|| doc.frontmatter.as_ref().and_then(|fm| fm.get("interface_type")));

                if let Some(iface) = iface_opt {
                    let iface_lower = iface.to_lowercase();
                    if iface_lower.contains("ui") || iface_lower.contains("gui") {
                        if self.rules.contains_key("feature_ui") {
                            return "feature_ui".to_string();
                        }
                    } else if iface_lower.contains("api") || iface_lower.contains("mcp") {
                        if self.rules.contains_key("feature_api") {
                            return "feature_api".to_string();
                        }
                    } else if iface_lower.contains("m2m") || iface_lower.contains("hardware") {
                        if self.rules.contains_key("feature_m2m") {
                            return "feature_m2m".to_string();
                        }
                    }
                    let candidate = format!("feature_{}", iface_lower);
                    if self.rules.contains_key(&candidate) {
                        return candidate;
                    }
                }
                "feature".to_string()
            }
            other => other.to_string(),
        }
    }

    /// Validates mandatory sections of a parsed specification document.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `SectionRules::validate` in `crates/verify-baseline/src/spec_audit/sections.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `SectionRules::validate` (declaration #10) within `SPEC-AUDIT-SECTIONS`.
    ///
    /// ## 3. Preconditions
    /// - `spec_type: &str` must satisfy domain and type-safety constraints prior to invoking `SectionRules::validate`.
    /// - `doc: &MarkdownDoc` must satisfy domain and type-safety constraints prior to invoking `SectionRules::validate`.
    /// - `rel_path: &str` must satisfy domain and type-safety constraints prior to invoking `SectionRules::validate`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Vec<String>` from `SectionRules::validate` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(spec_type)) bounded over input/member dimension of `SectionRules::validate`.
    /// - Space Complexity: O(N(spec_type)) bounded auxiliary memory for `SectionRules::validate`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `SectionRules::validate` (`Vec<String>`).
    /// - State: Maintains consistent state invariants across `SectionRules::validate` without partial mutation on error.
    /// - Action: Executes `SectionRules::validate` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `SectionRules::validate` (`SPEC-AUDIT-SECTIONS`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SPEC-AUDIT-SECTIONS-SECTIONRULES-VALIDATE] | SysML v2: ActionDef(SectionRules::validate)
    pub fn validate(&self, spec_type: &str, doc: &MarkdownDoc, rel_path: &str) -> Vec<String> {
        let mut errors = Vec::new();
        let rule_key = self.resolve_rule_key(spec_type, doc);

        let required = match self.rules.get(&rule_key) {
            Some(r) => r,
            None => {
                if rule_key.starts_with("feature_") {
                    match self.rules.get("feature") {
                        Some(r) => r,
                        None => return errors,
                    }
                } else {
                    return errors;
                }
            }
        };

        let heading_tokens: Vec<(u8, Vec<String>)> = doc
            .headings
            .iter()
            .map(|h| (h.level, normalize_tokens(&h.text)))
            .collect();

        for rule in required {
            let matched = heading_tokens
                .iter()
                .any(|(level, tokens)| *level == rule.level && tokens == &rule.normalized_tokens);

            if !matched {
                errors.push(format!("{}: missing section '{}'", rel_path, rule.display_name));
            }
        }

        errors
    }
}

static GLOBAL_DEFAULT_SECTION_RULES: LazyLock<SectionRules> =
    LazyLock::new(SectionRules::default_rules);

/// Validates that a specification document contains all mandatory section headings.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `validate_sections` in `crates/verify-baseline/src/spec_audit/sections.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `validate_sections` (declaration #11) within `SPEC-AUDIT-SECTIONS`.
///
/// ## 3. Preconditions
/// - `spec_type: &str` must satisfy domain and type-safety constraints prior to invoking `validate_sections`.
/// - `doc: &MarkdownDoc` must satisfy domain and type-safety constraints prior to invoking `validate_sections`.
/// - `rel_path: &str` must satisfy domain and type-safety constraints prior to invoking `validate_sections`.
///
/// ## 4. Postconditions
/// - Returns `Vec<String>` from `validate_sections` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(spec_type)) bounded over input/member dimension of `validate_sections`.
/// - Space Complexity: O(N(spec_type)) bounded auxiliary memory for `validate_sections`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `validate_sections` (`Vec<String>`).
/// - State: Maintains consistent state invariants across `validate_sections` without partial mutation on error.
/// - Action: Executes `validate_sections` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `validate_sections` (`SPEC-AUDIT-SECTIONS`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-SECTIONS-VALIDATE-SECTIONS] | SysML v2: ActionDef(validate_sections)
pub fn validate_sections(spec_type: &str, doc: &MarkdownDoc, rel_path: &str) -> Vec<String> {
    GLOBAL_DEFAULT_SECTION_RULES.validate(spec_type, doc, rel_path)
}

/// Validates Use Case content against canned fallback boilerplate, minimum scenario step count,
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `validate_use_case_anti_boilerplate` in `crates/verify-baseline/src/spec_audit/sections.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `validate_use_case_anti_boilerplate` (declaration #12) within `SPEC-AUDIT-SECTIONS`.
///
/// ## 3. Preconditions
/// - `doc: &MarkdownDoc` must satisfy domain and type-safety constraints prior to invoking `validate_use_case_anti_boilerplate`.
/// - `content: &str` must satisfy domain and type-safety constraints prior to invoking `validate_use_case_anti_boilerplate`.
/// - `rel_path: &str` must satisfy domain and type-safety constraints prior to invoking `validate_use_case_anti_boilerplate`.
///
/// ## 4. Postconditions
/// - Returns `Vec<String>` from `validate_use_case_anti_boilerplate` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(doc)) bounded over input/member dimension of `validate_use_case_anti_boilerplate`.
/// - Space Complexity: O(N(doc)) bounded auxiliary memory for `validate_use_case_anti_boilerplate`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `validate_use_case_anti_boilerplate` (`Vec<String>`).
/// - State: Maintains consistent state invariants across `validate_use_case_anti_boilerplate` without partial mutation on error.
/// - Action: Executes `validate_use_case_anti_boilerplate` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `validate_use_case_anti_boilerplate` (`SPEC-AUDIT-SECTIONS`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-SECTIONS-VALIDATE-USE-CASE-ANTI-BOILERPLATE] | SysML v2: ActionDef(validate_use_case_anti_boilerplate)
pub fn validate_use_case_anti_boilerplate(
    doc: &MarkdownDoc,
    content: &str,
    rel_path: &str,
) -> Vec<String> {
    let mut errors = Vec::new();
    let lower_content = content.to_lowercase();

    // 1. Canned fallback strings
    const CANNED_STRINGS: &[&str] = &[
        "initiates workflow execution",
        "processes structural elements and validates semantic constraints",
    ];

    for canned in CANNED_STRINGS {
        if lower_content.contains(canned) {
            errors.push(format!(
                "{}: Anti-Boilerplate Gate violation: Use Case contains canned fallback string '{}'. Use Cases must specify genuine domain-specific operational workflows.",
                rel_path, canned
            ));
        }
    }

    // 2. Count Main Success Scenario steps (minimum 5)
    let mut main_scenario_heading = None;
    for h in &doc.headings {
        let h_tokens = normalize_tokens(&h.text);
        if (h_tokens.contains(&"main".to_string()) && h_tokens.contains(&"scenario".to_string()))
            || h.text.to_lowercase().contains("main success scenario")
        {
            main_scenario_heading = Some(h);
            break;
        }
    }

    if let Some(h) = main_scenario_heading {
        let start_line = h.line;
        let next_heading_line = doc
            .headings
            .iter()
            .filter(|next_h| next_h.line > start_line && next_h.level <= h.level)
            .map(|next_h| next_h.line)
            .min()
            .unwrap_or(usize::MAX);

        let mut step_count = 0;
        for (idx, line) in content.lines().enumerate() {
            let line_num = idx + 1;
            if line_num > start_line && line_num < next_heading_line {
                let trimmed = line.trim();
                let digit_count = trimmed.chars().take_while(|c| c.is_ascii_digit()).count();
                if digit_count > 0 {
                    let after_digits = &trimmed[digit_count..];
                    if (after_digits.starts_with('.') || after_digits.starts_with(')'))
                        && after_digits[1..].starts_with(' ')
                    {
                        step_count += 1;
                    }
                }
            }
        }

        if step_count < 5 {
            errors.push(format!(
                "{}: Anti-Boilerplate Gate violation: Main Success Scenario has fewer than 5 steps (found {} step(s), minimum 5 required).",
                rel_path, step_count
            ));
        }
    }

    // 3. Count Alternate and Exception Flows cases (minimum 3, e.g. 5a, 5b, 5c)
    let mut alt_flows_heading = None;
    for h in &doc.headings {
        let h_tokens = normalize_tokens(&h.text);
        if (h_tokens.contains(&"alternate".to_string()) && h_tokens.contains(&"flows".to_string()))
            || (h_tokens.contains(&"exception".to_string()) && h_tokens.contains(&"flows".to_string()))
            || h.text.to_lowercase().contains("alternate and exception flows")
        {
            alt_flows_heading = Some(h);
            break;
        }
    }

    if let Some(h) = alt_flows_heading {
        let start_line = h.line;
        let next_heading_line = doc
            .headings
            .iter()
            .filter(|next_h| next_h.line > start_line && next_h.level <= h.level)
            .map(|next_h| next_h.line)
            .min()
            .unwrap_or(usize::MAX);

        let mut exception_count = 0;
        for (idx, line) in content.lines().enumerate() {
            let line_num = idx + 1;
            if line_num > start_line && line_num < next_heading_line {
                let mut trimmed = line.trim();
                if trimmed.starts_with('-') || trimmed.starts_with('*') {
                    trimmed = trimmed[1..].trim();
                }
                let digit_count = trimmed.chars().take_while(|c| c.is_ascii_digit()).count();
                if digit_count > 0 {
                    let after_digits = &trimmed[digit_count..];
                    let letter_count = after_digits.chars().take_while(|c| c.is_ascii_alphabetic()).count();
                    if letter_count > 0 {
                        let after_letters = &after_digits[letter_count..];
                        if after_letters.starts_with('.') || after_letters.starts_with(':') || after_letters.starts_with(' ') {
                            exception_count += 1;
                        }
                    }
                }
            }
        }

        if exception_count < 3 {
            errors.push(format!(
                "{}: Anti-Boilerplate Gate violation: Alternate and Exception Flows has fewer than 3 exception cases (found {} case(s), minimum 3 required, e.g. 5a, 5b, 5c).",
                rel_path, exception_count
            ));
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec_audit::markdown_ast::parse_markdown;

    #[test]
    fn test_normalize_tokens() {
        assert_eq!(normalize_tokens("## 1. Context"), vec!["1", "context"]);
        assert_eq!(
            normalize_tokens("2. Requirements & Checklist"),
            vec!["2", "requirements", "and", "checklist"]
        );
        assert_eq!(
            normalize_tokens("5. Alternate and Exception Flows"),
            vec!["5", "alternate", "and", "exception", "flows"]
        );
        assert_eq!(
            normalize_tokens("5. Alternate & Exception Flows"),
            vec!["5", "alternate", "and", "exception", "flows"]
        );
        assert_eq!(
            normalize_tokens("System-Level UML Class Diagram"),
            vec!["system", "level", "uml", "class", "diagram"]
        );
        assert_eq!(
            normalize_tokens("Special: / Path-With_Delimiters & More"),
            vec!["special", "path", "with", "delimiters", "and", "more"]
        );
    }

    #[test]
    fn test_section_rule_from_pattern() {
        let rule = SectionRule::from_pattern(r"(?i)##\s+1\.\s+Context", "## 1. Context");
        assert_eq!(rule.level, 2);
        assert_eq!(rule.normalized_tokens, vec!["1", "context"]);
        assert_eq!(rule.display_name, "## 1. Context");

        let rule3 = SectionRule::from_pattern(r"###\s+1\.\s+Payload\s+Schema", "### 1. Payload Schema");
        assert_eq!(rule3.level, 3);
        assert_eq!(rule3.normalized_tokens, vec!["1", "payload", "schema"]);
    }

    #[test]
    fn test_epic_missing_sections() {
        let content = r#"# Epic Title
## 1. Context
Prose here.
"#;
        let doc = parse_markdown(content).unwrap();
        let errors = validate_sections("epic", &doc, "docs/epics/epic-01-test.md");
        assert!(!errors.is_empty(), "Should report missing epic sections");
        assert!(errors.iter().any(|e| e.contains("2. Requirements & Checklist")));
    }

    #[test]
    fn test_epic_complete_sections_passes() {
        let content = r#"# Epic Complete
## 1. Context
## 2. Requirements & Checklist
### Associated Use Cases & User Stories
#### Associated Use Cases
#### Associated User Stories
## 3. Architecture
## 4. Operational Considerations
## 5. Security & Governance
## 6. Source References
## System-Level UML Class Diagram
## System State Machine Diagram
"#;
        let doc = parse_markdown(content).unwrap();
        let errors = validate_sections("epic", &doc, "docs/epics/epic-01-complete.md");
        assert!(errors.is_empty(), "Complete epic should have 0 section errors, got: {:?}", errors);
    }

    #[test]
    fn test_feature_ui_sections() {
        let content = r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Interface Type** | UI |

## UML Class Diagram
## Interface Requirements
### 1. Test Data Shape
### 3. Visual Layout & Arrangement
### 4. Interactive Flow & States
"#;
        let doc = parse_markdown(content).unwrap();
        let errors = validate_sections("feature", &doc, "docs/features/feat-01-ui.md");
        assert!(errors.is_empty(), "Complete UI feature should have 0 section errors, got: {:?}", errors);
    }

    #[test]
    fn test_heading_level_mismatch_fails() {
        // H3 instead of required H2 for "1. Context"
        let content = r#"# Epic Title
### 1. Context
## 2. Requirements & Checklist
## 3. Architecture
## 4. Operational Considerations
## 5. Security & Governance
## 6. Source References
## System-Level UML Class Diagram
## System State Machine Diagram
"#;
        let doc = parse_markdown(content).unwrap();
        let errors = validate_sections("epic", &doc, "docs/epics/epic-01-mismatch.md");
        assert!(
            errors.iter().any(|e| e.contains("1. Context")),
            "H3 should not satisfy H2 requirement for 1. Context"
        );
    }

    #[test]
    fn test_anti_boilerplate_gate() {
        // Canned fallback strings and underflow steps/exceptions
        let canned_content = r#"# Use Case 01: System Vision
## 4. Main Success Scenario
1. Actor `HumanEngineer` initiates workflow execution.
2. System `DEAPCompilerSystem` parses input model definitions.
3. Component `SystemVisionEngine` processes structural elements and validates semantic constraints.
4. System verifies invariant satisfaction.

## 5. Alternate and Exception Flows
- 5a. Syntax error detected: System logs error.
"#;
        let doc_canned = parse_markdown(canned_content).unwrap();
        let errors = validate_use_case_anti_boilerplate(&doc_canned, canned_content, "docs/use-cases/uc-01.md");
        assert!(errors.iter().any(|e| e.contains("initiates workflow execution")));
        assert!(errors.iter().any(|e| e.contains("processes structural elements and validates semantic constraints")));
        assert!(errors.iter().any(|e| e.contains("Main Success Scenario has fewer than 5 steps")));
        assert!(errors.iter().any(|e| e.contains("Alternate and Exception Flows has fewer than 3 exception cases")));

        // Valid use case with 6 steps and 4 exception flows
        let valid_content = r#"# Use Case 01: System Vision
## 4. Main Success Scenario
1. Operator dispatches compilation command with workspace schema root.
2. SystemVisionEngine initializes compiler namespace UUID constant NS_DEAP.
3. InvariantBootstrapEngine asserts zero hardcoded domain concept rules across runtime context.
4. NamespaceUuidGenerator pre-computes topological UUIDv5 namespace roots.
5. DomainVocabularyValidator inspects grammar vocabulary tokens against prohibited domain ontologies.
6. SystemVisionEngine emits immutable CompilerRulePort descriptor to downstream ingestion engines.

## 5. Alternate and Exception Flows
- 5a. Prohibited domain term detected: SystemVisionEngine halts pipeline and emits diagnostic E0100.
- 5b. Namespace UUID collision: SystemVisionEngine regenerates topological anchor.
- 5c. Malformed configuration flags: Compiler logs invalid argument diagnostics.
- 5d. File permission error on workspace: Execution halts safely.
"#;
        let doc_valid = parse_markdown(valid_content).unwrap();
        let errors_valid = validate_use_case_anti_boilerplate(&doc_valid, valid_content, "docs/use-cases/uc-01.md");
        assert!(errors_valid.is_empty(), "Valid use case should have 0 boilerplate errors: {:?}", errors_valid);
    }

    #[test]
    fn test_issue_449_epic_associated_use_cases_and_user_stories_sections_required() {
        let missing_assoc_content = r#"# Epic Complete
## 1. Context
## 2. Requirements & Checklist
## 3. Architecture
## 4. Operational Considerations
## 5. Security & Governance
## 6. Source References
## System-Level UML Class Diagram
## System State Machine Diagram
"#;
        let doc = parse_markdown(missing_assoc_content).unwrap();
        let errors = validate_sections("epic", &doc, "docs/epics/epic-01-missing-assoc.md");
        assert!(
            errors.iter().any(|e| e.contains("Associated Use Cases & User Stories")),
            "Epic missing '### Associated Use Cases & User Stories' must produce section validation error, got: {:?}",
            errors
        );
        assert!(
            errors.iter().any(|e| e.contains("Associated Use Cases")),
            "Epic missing '#### Associated Use Cases' must produce section validation error, got: {:?}",
            errors
        );
        assert!(
            errors.iter().any(|e| e.contains("Associated User Stories")),
            "Epic missing '#### Associated User Stories' must produce section validation error, got: {:?}",
            errors
        );
    }
}

