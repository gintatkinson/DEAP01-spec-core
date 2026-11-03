//! SysML model discovery and target scan directory taxonomy (`TargetScanDir`, `PathPrefix`) for parity gates.
//!
//! Safety & Traceability: Realises: [REQ-SYSML-MODEL-DISCOVERY], [REQ-SYSML-ARCH-MODEL-DISCOVERY]
//! Contract: Discovers SysML v2 and architectural schema texts across `schema/`, `docs/designs/`, and `.pipeline/schema.sysml`.
//! Bounds & Faults: Deterministic sorted path scanning; pure non-panicking string and AST inspection.

use std::fs;
use std::path::{Path, PathBuf};

/// Target directory enumeration for recursive specification scanning.
/// Realises: [REQ-TARGET-SCAN-DIR-ENUM]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum TargetScanDir {
    Conops,
    Safety,
    Interfaces,
    Icd,
    Architecture,
    Designs,
    Epics,
    Features,
    UserStories,
    UseCases,
    Reports,
    Management,
}

impl TargetScanDir {
    /// Complete deterministic slice of all 12 specification target scan directories.
    /// Realises: [REQ-TARGET-SCAN-DIR-ALL]
    pub(crate) const ALL: &'static [TargetScanDir] = &[
        TargetScanDir::Conops,
        TargetScanDir::Safety,
        TargetScanDir::Interfaces,
        TargetScanDir::Icd,
        TargetScanDir::Architecture,
        TargetScanDir::Designs,
        TargetScanDir::Epics,
        TargetScanDir::Features,
        TargetScanDir::UserStories,
        TargetScanDir::UseCases,
        TargetScanDir::Reports,
        TargetScanDir::Management,
    ];

    /// Maps the enum variant to its relative directory name under `docs/`.
    /// Realises: [REQ-TARGET-SCAN-DIR-SUBDIR]
    pub(crate) const fn docs_subdir(self) -> &'static str {
        match self {
            Self::Conops => "conops",
            Self::Safety => "safety",
            Self::Interfaces => "interfaces",
            Self::Icd => "icd",
            Self::Architecture => "architecture",
            Self::Designs => "designs",
            Self::Epics => "epics",
            Self::Features => "features",
            Self::UserStories => "user-stories",
            Self::UseCases => "use-cases",
            Self::Reports => "reports",
            Self::Management => "management",
        }
    }

    /// Resolves the target directory relative to the given repository root.
    /// Realises: [REQ-TARGET-SCAN-DIR-RESOLVE]
    pub(crate) fn resolve(&self, repo_root: &Path) -> PathBuf {
        repo_root.join("docs").join(self.docs_subdir())
    }

    /// Indicates whether specifications in this directory require Check 23 factual grounding.
    /// Realises: [REQ-TARGET-SCAN-DIR-FACTUAL-GROUNDING]
    pub(crate) const fn requires_factual_grounding(self) -> bool {
        match self {
            Self::Conops
            | Self::Safety
            | Self::Interfaces
            | Self::Icd
            | Self::Architecture
            | Self::Designs
            | Self::Epics
            | Self::Features
            | Self::UserStories
            | Self::UseCases
            | Self::Reports => true,
            Self::Management => false,
        }
    }
}

/// Allowed path prefixes for authoritative citation extraction.
/// Realises: [REQ-PATH-PREFIX-ENUM]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PathPrefix {
    Schema,
    Docs,
}

impl PathPrefix {
    /// Returns the canonical string prefix representation.
    /// Realises: [REQ-PATH-PREFIX-STR]
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Schema => "schema/",
            Self::Docs => "docs/",
        }
    }

    /// Returns all canonical path prefix variants.
    /// Realises: [REQ-PATH-PREFIX-ALL]
    pub(crate) fn all() -> [PathPrefix; 2] {
        [PathPrefix::Schema, PathPrefix::Docs]
    }
}

/// Locates and reads authoritative SysML v2 or multi-format schema model content.
/// Realises: [REQ-SYSML-MODEL-DISCOVERY]
pub fn discover_sysml_model_text(repo_root: &Path) -> Option<String> {
    let schema_dir = repo_root.join("schema");
    if schema_dir.is_dir() {
        if let Some(model) = discover_schema_dir_model_text(&schema_dir) {
            return Some(model);
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

    let designs_dir = repo_root.join("docs").join("designs");
    if designs_dir.is_dir() {
        let mut paths =
            deap_core::sysml_cst::UnifiedSchemaModelExtractor::discover_schema_files(&[
                &designs_dir,
            ]);
        paths.sort();
        let mut design_sysml_contents = Vec::new();
        for p in paths {
            if p.extension().is_some_and(|ext| ext == "sysml") {
                if let Ok(content) = fs::read_to_string(&p) {
                    if !content.trim().is_empty() {
                        design_sysml_contents.push(content);
                    }
                }
            }
        }
        if !design_sysml_contents.is_empty() {
            return Some(design_sysml_contents.join("\n\n"));
        }
    }
    None
}

/// Reads `.sysml` or synthesized subsystem parts from `schema_dir`.
/// Realises: [REQ-SYSML-MODEL-DISCOVERY]
fn discover_schema_dir_model_text(schema_dir: &Path) -> Option<String> {
    let mut paths =
        deap_core::sysml_cst::UnifiedSchemaModelExtractor::discover_schema_files(&[schema_dir]);
    paths.sort();
    let mut sysml_contents = Vec::new();
    let mut non_sysml_paths = Vec::new();

    for p in paths {
        if p.extension().is_some_and(|ext| ext == "sysml") {
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
        let parts = deap_core::sysml_cst::UnifiedSchemaModelExtractor::extract_subsystem_parts(
            &non_sysml_paths,
        );
        if !parts.is_empty() {
            let mut synthetic = String::from("package DownstreamSchema {\n");
            for part in parts {
                synthetic.push_str(&format!("    part def {};\n", part));
            }
            synthetic.push_str("}\n");
            return Some(synthetic);
        }
    }
    None
}

/// Discovers architectural SysML v2 model text for system-level safety (Check 17 FMECA) verification.
/// Realises: [REQ-SYSML-ARCH-MODEL-DISCOVERY], [REQ-ISSUE-472-CHECK17-REALIZATION-CLOSURE]
pub fn discover_architectural_sysml_model_text(repo_root: &Path) -> Option<String> {
    let raw_text = resolve_raw_architectural_sysml_text(repo_root)?;
    let stripped = strip_realization_packages_from_sysml(&raw_text);
    if !deap_core::sysml_cst::parse_part_defs(&stripped).is_empty() {
        Some(stripped)
    } else {
        Some(raw_text)
    }
}

/// Resolves raw architectural SysML model text across `schema/`, `docs/designs/`, and `.pipeline/schema.sysml`.
/// Realises: [REQ-SYSML-ARCH-MODEL-DISCOVERY]
fn resolve_raw_architectural_sysml_text(repo_root: &Path) -> Option<String> {
    let schema_dir = repo_root.join("schema");
    let designs_dir = repo_root.join("docs").join("designs");
    let mut scan_roots: Vec<&Path> = Vec::new();
    if schema_dir.is_dir() {
        scan_roots.push(&schema_dir);
    }
    if designs_dir.is_dir() {
        scan_roots.push(&designs_dir);
    }
    let mut resolved: Option<String> = None;
    if !scan_roots.is_empty() {
        let mut paths =
            deap_core::sysml_cst::UnifiedSchemaModelExtractor::discover_schema_files(&scan_roots);
        paths.sort();
        let mut domain_sysml = Vec::new();
        let mut all_sysml = Vec::new();
        let mut non_sysml = Vec::new();

        for p in paths {
            if p.extension().is_some_and(|ext| ext == "sysml") {
                let is_realization = p
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .is_some_and(|stem| {
                        let lower = stem.to_ascii_lowercase();
                        lower == "realization" || lower.ends_with("_realization")
                    });
                if let Ok(content) = fs::read_to_string(&p) {
                    if !content.trim().is_empty() {
                        if !is_realization {
                            domain_sysml.push(content.clone());
                        }
                        all_sysml.push(content);
                    }
                }
            } else if p.starts_with(&schema_dir) {
                non_sysml.push(p);
            }
        }
        if !domain_sysml.is_empty() {
            resolved = Some(domain_sysml.join("\n\n"));
        } else if !all_sysml.is_empty() {
            resolved = Some(all_sysml.join("\n\n"));
        } else if !non_sysml.is_empty() {
            let parts = deap_core::sysml_cst::UnifiedSchemaModelExtractor::extract_subsystem_parts(
                &non_sysml,
            );
            if !parts.is_empty() {
                let mut synthetic = String::from("package DownstreamSchema {\n");
                for part in parts {
                    synthetic.push_str(&format!("    part def {};\n", part));
                }
                synthetic.push_str("}\n");
                resolved = Some(synthetic);
            }
        }
    }
    if resolved.is_none() {
        let pipeline_model = repo_root.join(".pipeline").join("schema.sysml");
        if pipeline_model.is_file() {
            if let Ok(content) = fs::read_to_string(&pipeline_model) {
                if !content.trim().is_empty() {
                    resolved = Some(content);
                }
            }
        }
    }
    resolved
}

/// Strips inline `src`, `lib`, and `*_realization` packages from raw SysML model text.
/// Realises: [REQ-ISSUE-472-CHECK17-REALIZATION-CLOSURE]
fn strip_realization_packages_from_sysml(raw_text: &str) -> String {
    let mut filtered_lines = Vec::new();
    let mut skip_depth: usize = 0;
    for line in raw_text.lines() {
        let trimmed = line.trim();
        if skip_depth == 0 {
            if let Some(after_pkg) = trimmed.strip_prefix("package ") {
                let pkg_ident = after_pkg
                    .split(|c: char| c.is_whitespace() || c == '{')
                    .next()
                    .unwrap_or("");
                let pkg_lower = pkg_ident.to_ascii_lowercase();
                if pkg_lower == "src" || pkg_lower == "lib" || pkg_lower.ends_with("_realization") {
                    let opens = line.bytes().filter(|&b| b == b'{').count();
                    let closes = line.bytes().filter(|&b| b == b'}').count();
                    skip_depth = opens.saturating_sub(closes);
                    continue;
                }
            }
            filtered_lines.push(line);
        } else {
            let opens = line.bytes().filter(|&b| b == b'{').count();
            let closes = line.bytes().filter(|&b| b == b'}').count();
            skip_depth = skip_depth.saturating_add(opens).saturating_sub(closes);
        }
    }
    filtered_lines.join("\n")
}
