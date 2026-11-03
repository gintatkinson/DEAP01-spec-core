//! Specification mandatory section validation conforming to DO-178C Level A and ISO 26262 ASIL D.
//!
//! Validates required Markdown section headers for Epics, Features (including UI, API, and M2M variants),
//! User Stories, and Use Cases against configured or fallback structural rules using zero-copy AST token matching.

use crate::spec_audit::markdown_ast::MarkdownDoc;
pub use crate::spec_audit::sections_boilerplate::validate_use_case_anti_boilerplate;
use deap_core::rules::CodebaseRules;
use std::collections::BTreeMap;
use std::sync::LazyLock;

/// Normalizes heading or pattern text into canonical comparison tokens.
/// Realises: [REQ-SPEC-AUDIT-SECTIONS-NORMALIZE-TOKENS]
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
/// Realises: [REQ-SPEC-AUDIT-SECTIONS-SECTIONRULE]
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
    /// Realises: [REQ-SPEC-AUDIT-SECTIONS-SECTIONRULE-NEW]
    pub fn new(level: u8, normalized_tokens: Vec<String>, display_name: impl Into<String>) -> Self {
        Self {
            level,
            normalized_tokens,
            display_name: display_name.into(),
        }
    }

    /// Constructs a SectionRule by parsing a pattern string and display title.
    /// Realises: [REQ-SPEC-AUDIT-SECTIONS-SECTIONRULE-FROM-PATTERN]
    pub fn from_pattern(pattern: &str, display_name: &str) -> Self {
        let clean_pat = pattern.trim_start_matches("(?i)").trim();
        let pat_hashes = clean_pat.chars().take_while(|&c| c == '#').count();
        let disp_hashes = display_name
            .trim()
            .chars()
            .take_while(|&c| c == '#')
            .count();

        let level = if pat_hashes > 0 {
            u8::try_from(pat_hashes).unwrap_or(u8::MAX)
        } else if disp_hashes > 0 {
            u8::try_from(disp_hashes).unwrap_or(u8::MAX)
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

/// Inserts a slice of `(pattern, display)` section pairs under `key`.
/// Realises: [REQ-SPEC-AUDIT-SECTIONS-INSERT-RULE-PAIRS]
fn insert_rule_pairs(
    rules: &mut BTreeMap<String, Vec<SectionRule>>,
    key: &str,
    pairs: &[(&str, &str)],
) {
    let entry = rules.entry(key.to_string()).or_default();
    for (pat, disp) in pairs {
        entry.push(SectionRule::from_pattern(pat, disp));
    }
}

/// Populates default Epic and Feature variant rules into `rules`.
/// Realises: [REQ-SPEC-AUDIT-SECTIONS-POPULATE-EPIC-FEATURE]
fn populate_epic_and_feature_rules(rules: &mut BTreeMap<String, Vec<SectionRule>>) {
    insert_rule_pairs(
        rules,
        "epic",
        &[
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
        ],
    );
    insert_rule_pairs(
        rules,
        "feature",
        &[
            ("## UML Class Diagram", "## UML Class Diagram"),
            ("## Interface Requirements", "## Interface Requirements"),
            ("### 1. Test Data Shape", "### 1. Test Data Shape"),
            ("### 4. Interactive Flow & States", "### 4. Interactive Flow & States"),
        ],
    );
    insert_rule_pairs(
        rules,
        "feature_ui",
        &[
            ("## UML Class Diagram", "## UML Class Diagram"),
            ("## Interface Requirements", "## Interface Requirements"),
            ("### 1. Test Data Shape", "### 1. Test Data Shape"),
            ("### 3. Visual Layout & Arrangement", "### 3. Visual Layout & Arrangement"),
            ("### 4. Interactive Flow & States", "### 4. Interactive Flow & States"),
        ],
    );
    let api_m2m_sections = [
        ("## UML Class Diagram", "## UML Class Diagram"),
        ("## Interface Requirements", "## Interface Requirements"),
        ("### 1. Payload Schema", "### 1. Payload Schema"),
        ("### 3. Logical Operations & Interface Messages", "### 3. Logical Operations & Interface Messages"),
        ("### 4. Logical Exception States & Validation Failures", "### 4. Logical Exception States & Validation Failures"),
    ];
    insert_rule_pairs(rules, "feature_api", &api_m2m_sections);
    insert_rule_pairs(rules, "feature_m2m", &api_m2m_sections);
}

/// Populates default User Story and Use Case rules into `rules`.
/// Realises: [REQ-SPEC-AUDIT-SECTIONS-POPULATE-STORY-USECASE]
fn populate_story_and_usecase_rules(rules: &mut BTreeMap<String, Vec<SectionRule>>) {
    insert_rule_pairs(
        rules,
        "user_story",
        &[
            ("## UML Sequence Diagram", "## UML Sequence Diagram"),
            ("## Required Features", "## Required Features"),
        ],
    );
    insert_rule_pairs(
        rules,
        "use_case",
        &[
            ("## UML Diagrams", "## UML Diagrams"),
            ("## 1. Actors", "## 1. Actors"),
            ("## 2. Preconditions", "## 2. Preconditions"),
            ("## 3. Trigger", "## 3. Trigger"),
            ("## 4. Main Success Scenario", "## 4. Main Success Scenario"),
            ("## 5. Alternate and Exception Flows", "## 5. Alternate and Exception Flows"),
            ("## 6. Postconditions", "## 6. Postconditions"),
            ("## 8. Realization Matrix", "## 8. Realization Matrix"),
        ],
    );
}

/// Collection of mandatory section rules indexed by specification type.
/// Realises: [REQ-SPEC-AUDIT-SECTIONS-SECTIONRULES]
#[derive(Debug, Clone)]
pub struct SectionRules {
    /// Map of specification type key to its required section rules.
    pub rules: BTreeMap<String, Vec<SectionRule>>,
}

impl Default for SectionRules {
    /// Returns the default specification section rules.
    /// Realises: [REQ-SPEC-AUDIT-SECTIONS-SECTIONRULES-DEFAULT]
    fn default() -> Self {
        Self::default_rules()
    }
}

impl SectionRules {
    /// Constructs default section rules covering Epics, Features, User Stories, and Use Cases.
    /// Realises: [REQ-SPEC-AUDIT-SECTIONS-SECTIONRULES-DEFAULT-RULES]
    pub fn default_rules() -> Self {
        let mut rules: BTreeMap<String, Vec<SectionRule>> = BTreeMap::new();
        populate_epic_and_feature_rules(&mut rules);
        populate_story_and_usecase_rules(&mut rules);
        Self { rules }
    }

    /// Ingests required sections configuration from parsed `CodebaseRules`.
    /// Realises: [REQ-SPEC-AUDIT-SECTIONS-SECTIONRULES-FROM-CODEBASE-RULES]
    pub fn from_codebase_rules(codebase_rules: &CodebaseRules) -> Self {
        let mut section_rules = Self::default_rules();
        if let Some(val_rules) = &codebase_rules.validation_rules {
            if let Some(req_sections) = &val_rules.required_sections {
                for (spec_type, arr_val) in req_sections {
                    Self::ingest_json_rule_array(&mut section_rules.rules, spec_type, arr_val);
                }
            }
        }
        if let Some(spec_rules) = codebase_rules.extra.get("spec_rules") {
            if let Some(req_sections) = spec_rules
                .get("required_sections")
                .and_then(|v| v.as_object())
            {
                for (spec_type, arr_val) in req_sections {
                    Self::ingest_json_rule_array(&mut section_rules.rules, spec_type, arr_val);
                }
            }
        }
        section_rules
    }

    /// Parses a JSON array of `[pattern, display_name]` pairs into `rules`.
    /// Realises: [REQ-SPEC-AUDIT-SECTIONS-INGEST-JSON-ARRAY]
    fn ingest_json_rule_array(
        rules: &mut BTreeMap<String, Vec<SectionRule>>,
        spec_type: &str,
        arr_val: &serde_json::Value,
    ) {
        if let Some(arr) = arr_val.as_array() {
            let mut parsed_rules = Vec::new();
            for item in arr {
                if let Some(pair) = item.as_array() {
                    if let (Some(pat_str), Some(disp_str)) = (
                        pair.first().and_then(|v| v.as_str()),
                        pair.get(1).and_then(|v| v.as_str()),
                    ) {
                        parsed_rules.push(SectionRule::from_pattern(pat_str, disp_str));
                    }
                }
            }
            if !parsed_rules.is_empty() {
                rules.insert(spec_type.to_string(), parsed_rules);
            }
        }
    }

    /// Resolves the effective rule lookup key for a specification.
    /// Realises: [REQ-SPEC-AUDIT-SECTIONS-SECTIONRULES-RESOLVE-RULE-KEY]
    fn resolve_rule_key(&self, spec_type: &str, doc: &MarkdownDoc) -> String {
        let norm_type = spec_type.to_lowercase();
        match norm_type.as_str() {
            "epic" | "epics" => "epic".to_string(),
            "user_story" | "user-story" | "user_stories" | "user-stories" => {
                "user_story".to_string()
            }
            "use_case" | "use-case" | "use_cases" | "use-cases" => "use_case".to_string(),
            "feature" | "features" => {
                let iface_opt = doc.table_metadata.get("interface_type").or_else(|| {
                    doc.frontmatter
                        .as_ref()
                        .and_then(|fm| fm.get("interface_type"))
                });
                if let Some(iface) = iface_opt {
                    let iface_lower = iface.to_lowercase();
                    if (iface_lower.contains("ui") || iface_lower.contains("gui"))
                        && self.rules.contains_key("feature_ui")
                    {
                        return "feature_ui".to_string();
                    } else if (iface_lower.contains("api") || iface_lower.contains("mcp"))
                        && self.rules.contains_key("feature_api")
                    {
                        return "feature_api".to_string();
                    } else if (iface_lower.contains("m2m") || iface_lower.contains("hardware"))
                        && self.rules.contains_key("feature_m2m")
                    {
                        return "feature_m2m".to_string();
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
    /// Realises: [REQ-SPEC-AUDIT-SECTIONS-SECTIONRULES-VALIDATE]
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
                errors.push(format!(
                    "{}: missing section '{}'",
                    rel_path, rule.display_name
                ));
            }
        }

        errors
    }
}

static GLOBAL_DEFAULT_SECTION_RULES: LazyLock<SectionRules> =
    LazyLock::new(SectionRules::default_rules);

/// Validates that a specification document contains all mandatory section headings.
/// Realises: [REQ-SPEC-AUDIT-SECTIONS-VALIDATE-SECTIONS]
pub fn validate_sections(spec_type: &str, doc: &MarkdownDoc, rel_path: &str) -> Vec<String> {
    GLOBAL_DEFAULT_SECTION_RULES.validate(spec_type, doc, rel_path)
}

#[cfg(test)]
#[path = "sections_tests.rs"]
mod tests;
