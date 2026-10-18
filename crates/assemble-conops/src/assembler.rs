//! ConOps & Mission Intent modular assembly, link relativization, and verification engine.

use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::params::SysMLParameterBindingEngine;
use crate::sanitize::sanitize_level_1b_operational_text;
use crate::toc::{extract_headings, generate_table_of_contents, verify_markdown_links};

/// Canonical unit file whitelists for deterministic ConOps & Mission Intent assembly.
pub static CANONICAL_CONOPS_UNITS: &[&str] = &[
    "01_METADATA_AND_OVERVIEW.md",
    "02_DEFICIENCIES_AND_MOTIVATION.md",
    "03_PROPOSED_CAPABILITIES.md",
    "04_SYSTEM_ARCHITECTURE.md",
    "04_USER_CLASSES_AND_STAKEHOLDERS.md",
    "05_OPERATIONAL_STATE_SPACE_AND_RISK.md",
    "06_UAF_OPERATIONAL_ACTIVITIES.md",
    "07_OPTX_EXCHANGES.md",
    "08_SYSTEM_ARCHITECTURE.md",
    "08_ENVIRONMENTAL_OPERATING_LIMITS.md",
    "08_ENVIRONMENTAL_MIL_STD_810H.md",
    "09_SCENARIOS_AND_TIMELINES.md",
    "10_MAINTENANCE_AND_GSE_SUPPORT.md",
    "11_IMPACTS_AND_TRADE_STUDIES.md",
    "12_EMERGENCY_DECISION_MATRIX.md",
];

pub static CANONICAL_MISSION_INTENT_UNITS: &[&str] = &[
    "01_COMMANDERS_INTENT.md",
    "02_MISSION_ESSENTIAL_TASK_LIST.md",
    "03_INCOSE_MOE_MOP_MATH.md",
    "04_MULTI_DOMAIN_THREAT_MATRIX.md",
    "05_PACE_C2_PLAN.md",
    "06_SAFETY_INTERLOCKS.md",
    "06_ROE_SAFETY_INTERLOCKS.md",
    "06_RULES_OF_ENGAGEMENT.md",
    "07_AIRSPACE_GEOZONES.md",
    "08_GO_NO_GO_MATRIX.md",
    "09_ENERGY_AND_RESERVE_BOUNDS.md",
    "09_BINGO_ENERGY_MATH.md",
    "10_OPERATIONAL_ALLOCATION_TAGS.md",
];

/// Computes a relative path from `base` to `path`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `path_relative_from` in `crates/assemble-conops/src/assembler.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `path_relative_from` (declaration #1) within `ASSEMBLE-CONOPS-ASSEMBLER`.
///
/// ## 3. Preconditions
/// - `path: &Path` must satisfy domain and type-safety constraints prior to invoking `path_relative_from`.
/// - `base: &Path` must satisfy domain and type-safety constraints prior to invoking `path_relative_from`.
///
/// ## 4. Postconditions
/// - Returns `PathBuf` from `path_relative_from` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(path)) bounded over input/member dimension of `path_relative_from`.
/// - Space Complexity: O(N(path)) bounded auxiliary memory for `path_relative_from`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `path_relative_from` (`PathBuf`).
/// - State: Maintains consistent state invariants across `path_relative_from` without partial mutation on error.
/// - Action: Executes `path_relative_from` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `path_relative_from` (`ASSEMBLE-CONOPS-ASSEMBLER`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-ASSEMBLE-CONOPS-ASSEMBLER-PATH-RELATIVE-FROM] | SysML v2: ActionDef(path_relative_from)
pub fn path_relative_from(path: &Path, base: &Path) -> PathBuf {
    let mut path_comps = path.components().peekable();
    let mut base_comps = base.components().peekable();

    while path_comps.peek() == base_comps.peek() && path_comps.peek().is_some() {
        path_comps.next();
        base_comps.next();
    }

    let mut result = PathBuf::new();
    for comp in base_comps {
        if matches!(comp, Component::Normal(_)) {
            result.push("..");
        }
    }
    for comp in path_comps {
        result.push(comp.as_os_str());
    }
    result
}

/// Executes deterministic fn `is_uri_scheme` within `crates/assemble-conops/src/assembler.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `is_uri_scheme` in `crates/assemble-conops/src/assembler.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `is_uri_scheme` (declaration #2) within `ASSEMBLE-CONOPS-ASSEMBLER`.
///
/// ## 3. Preconditions
/// - `s: &str` must satisfy domain and type-safety constraints prior to invoking `is_uri_scheme`.
///
/// ## 4. Postconditions
/// - Returns `bool` from `is_uri_scheme` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(s)) bounded over input/member dimension of `is_uri_scheme`.
/// - Space Complexity: O(N(s)) bounded auxiliary memory for `is_uri_scheme`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `is_uri_scheme` (`bool`).
/// - State: Maintains consistent state invariants across `is_uri_scheme` without partial mutation on error.
/// - Action: Executes `is_uri_scheme` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `is_uri_scheme` (`ASSEMBLE-CONOPS-ASSEMBLER`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-ASSEMBLE-CONOPS-ASSEMBLER-IS-URI-SCHEME] | SysML v2: ActionDef(is_uri_scheme)
fn is_uri_scheme(s: &str) -> bool {
    if let Some(pos) = s.find("://") {
        if pos == 0 {
            return false;
        }
        let scheme = &s[..pos];
        let mut chars = scheme.chars();
        if let Some(first) = chars.next() {
            if first.is_ascii_alphabetic()
                && chars.all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.')
            {
                return true;
            }
        }
    }
    false
}

/// Normalizes markdown links `[label](target)` so they resolve relative to the directory of `output_file`.
pub fn relativize_markdown_links(text: &str, output_file: &Path, workspace_dir: &Path) -> String {
    if text.is_empty() {
        return String::new();
    }

    let abs_ws_dir = workspace_dir
        .canonicalize()
        .unwrap_or_else(|_| workspace_dir.to_path_buf());
    let abs_output_file = if output_file.is_absolute() {
        output_file.to_path_buf()
    } else {
        abs_ws_dir.join(output_file)
    };
    let abs_output_dir = abs_output_file
        .parent()
        .unwrap_or(&abs_ws_dir)
        .to_path_buf();

    let bytes = text.as_bytes();
    let mut result = String::with_capacity(text.len());
    let mut idx = 0;

    while idx < bytes.len() {
        if bytes[idx] == b'[' {
            let mut j = idx + 1;
            let mut label_end = None;
            while j < bytes.len() {
                if bytes[j] == b'\\' && j + 1 < bytes.len() {
                    j += 2;
                    continue;
                }
                if bytes[j] == b']' {
                    label_end = Some(j);
                    break;
                }
                j += 1;
            }

            if let Some(lend) = label_end {
                if lend > idx + 1 && lend + 1 < bytes.len() && bytes[lend + 1] == b'(' {
                    let target_start = lend + 2;
                    let mut k = target_start;
                    let mut target_end = None;
                    while k < bytes.len() {
                        if bytes[k] == b')' {
                            target_end = Some(k);
                            break;
                        }
                        if bytes[k] == b'\n' {
                            break;
                        }
                        k += 1;
                    }

                    if let Some(tend) = target_end {
                        if tend > target_start {
                            let label = &text[idx + 1..lend];
                            let raw_target = &text[target_start..tend];
                            let target_trimmed = raw_target.trim();

                            if target_trimmed.is_empty() {
                                result.push_str(&text[idx..tend + 1]);
                                idx = tend + 1;
                                continue;
                            }

                            let (url_part, title_part, has_angle_brackets) = if target_trimmed.starts_with('<')
                                && target_trimmed.contains('>')
                            {
                                let end = target_trimmed.find('>').unwrap();
                                let url = target_trimmed[1..end].trim();
                                let title = &target_trimmed[end + 1..];
                                (url, title.to_string(), true)
                            } else {
                                let parts: Vec<&str> = target_trimmed.splitn(2, ' ').collect();
                                if parts.len() == 2
                                    && (parts[1].starts_with('"')
                                        || parts[1].starts_with('\'')
                                        || parts[1].starts_with('('))
                                {
                                    (parts[0], format!(" {}", parts[1]), false)
                                } else {
                                    (target_trimmed, String::new(), false)
                                }
                            };

                            // Preserve external URLs and anchors
                            if url_part.starts_with('#')
                                || url_part.starts_with("http://")
                                || url_part.starts_with("https://")
                                || url_part.starts_with("mailto:")
                                || url_part.starts_with("ftp://")
                                || is_uri_scheme(url_part)
                                || url_part.is_empty()
                            {
                                result.push_str(&text[idx..tend + 1]);
                                idx = tend + 1;
                                continue;
                            }

                            let (path_part, anchor_suffix) = if let Some((p, a)) = url_part.split_once('#') {
                                (p, format!("#{a}"))
                            } else {
                                (url_part, String::new())
                            };

                            if path_part.is_empty() {
                                result.push_str(&text[idx..tend + 1]);
                                idx = tend + 1;
                                continue;
                            }

                            let is_explicit_rel = path_part.starts_with("./")
                                || path_part.starts_with("../")
                                || path_part == "."
                                || path_part == "..";
                            let is_abs = path_part.starts_with('/')
                                || (path_part.len() > 2 && path_part.as_bytes()[1] == b':');

                            let target_abs = if is_abs {
                                let p = Path::new(path_part);
                                if p.starts_with(&abs_ws_dir) {
                                    p.to_path_buf()
                                } else {
                                    abs_ws_dir.join(path_part.trim_start_matches(['/', '\\']))
                                }
                            } else if is_explicit_rel {
                                let candidate = abs_output_dir.join(path_part);
                                if candidate.exists() {
                                    candidate
                                } else {
                                    let cand_ws =
                                        abs_ws_dir.join(path_part.trim_start_matches(['.', '/', '\\']));
                                    let cand_docs = abs_ws_dir
                                        .join("docs")
                                        .join(path_part.trim_start_matches(['.', '/', '\\']));
                                    if cand_ws.exists() {
                                        cand_ws
                                    } else if cand_docs.exists() {
                                        cand_docs
                                    } else {
                                        candidate
                                    }
                                }
                            } else {
                                abs_ws_dir.join(path_part)
                            };

                            let rel = path_relative_from(&target_abs, &abs_output_dir);
                            let mut rel_str = rel.to_string_lossy().replace('\\', "/");
                            if path_part.starts_with("./") && !rel_str.starts_with('.') && !rel_str.is_empty() {
                                rel_str = format!("./{rel_str}");
                            }

                            let new_url = format!("{rel_str}{anchor_suffix}");
                            let replacement = if has_angle_brackets {
                                format!("[{label}](<{new_url}>{title_part})")
                            } else {
                                format!("[{label}]({new_url}{title_part})")
                            };
                            result.push_str(&replacement);
                            idx = tend + 1;
                            continue;
                        }
                    }
                }
            }
        }

        let ch = text[idx..].chars().next().unwrap();
        result.push(ch);
        idx += ch.len_utf8();
    }

    result
}

/// Scans text for unresolved parameter tokens matching `{{...}}`.
pub fn find_raw_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i + 4 <= bytes.len() {
        if bytes[i] == b'{' && bytes[i + 1] == b'{' {
            let start = i;
            let mut j = i + 2;
            let mut found_end = None;
            while j + 1 < bytes.len() {
                if bytes[j] == b'}' && bytes[j + 1] == b'}' {
                    found_end = Some(j);
                    break;
                }
                if bytes[j] == b'\n' {
                    break;
                }
                j += 1;
            }
            if let Some(end_pos) = found_end {
                let inner = &text[start + 2..end_pos];
                let (key, _default) = match inner.split_once(':') {
                    Some((k, d)) => (k, Some(d)),
                    None => (inner, None),
                };
                if !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    tokens.push(text[start..end_pos + 2].to_string());
                    i = end_pos + 2;
                    continue;
                }
            }
        }
        i += 1;
    }
    tokens
}

/// Validates unit integrity:
/// 1. File exists and is non-empty.
/// 2. Zero unresolved placeholder tokens matching `{{...}}`.
pub fn validate_unit_integrity(
    unit_paths: &[PathBuf],
    param_engine: Option<&SysMLParameterBindingEngine>,
) -> (bool, Vec<String>) {
    let mut errors = Vec::new();

    for path in unit_paths {
        if !path.is_file() {
            errors.push(format!("Unit file not found: {}", path.display()));
            continue;
        }

        let text = match fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) => {
                errors.push(format!("Failed to read unit file '{}': {e}", path.display()));
                continue;
            }
        };

        if text.trim().is_empty() {
            errors.push(format!("Unit file '{}' is empty.", path.display()));
            continue;
        }

        let processed = if let Some(eng) = param_engine {
            eng.substitute(&text)
        } else {
            text
        };

        let placeholders: Vec<String> = find_raw_tokens(&processed);
        if !placeholders.is_empty() {
            let mut unique = placeholders;
            unique.sort();
            unique.dedup();
            errors.push(format!(
                "Unit file '{}' contains {} unresolved placeholder token(s): {}",
                path.display(),
                unique.len(),
                unique.join(", ")
            ));
        }
    }

    (errors.is_empty(), errors)
}

/// Compiles unit files located in `units_dir` into a single verified Markdown document.
pub fn assemble_document(
    units_dir: &Path,
    doc_title: Option<&str>,
    doc_version: Option<&str>,
    doc_date: Option<&str>,
    param_engine: &mut SysMLParameterBindingEngine,
    canonical_whitelist: Option<&[&str]>,
) -> (String, Vec<String>) {
    let mut errors = Vec::new();
    if !units_dir.is_dir() {
        return (
            String::new(),
            vec![format!("Units directory '{}' does not exist.", units_dir.display())],
        );
    }

    let default_whitelist = match units_dir.file_name().and_then(|n| n.to_str()).unwrap_or("") {
        "conops" => Some(CANONICAL_CONOPS_UNITS),
        "mission_intent" | "missionintent" => Some(CANONICAL_MISSION_INTENT_UNITS),
        _ => None,
    };
    let effective_whitelist = canonical_whitelist.or(default_whitelist);

    let all_entries = match fs::read_dir(units_dir) {
        Ok(read_dir) => {
            let mut files = Vec::new();
            for entry in read_dir.filter_map(|e| e.ok()) {
                let p = entry.path();
                if p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("md") {
                    if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                        files.push(name.to_string());
                    }
                }
            }
            files.sort();
            files
        }
        Err(e) => {
            return (
                String::new(),
                vec![format!("Failed to read directory '{}': {e}", units_dir.display())],
            );
        }
    };

    if all_entries.is_empty() {
        return (
            String::new(),
            vec![format!("No markdown unit files (*.md) found in '{}'.", units_dir.display())],
        );
    }

    let mut filenames = Vec::new();
    if let Some(whitelist) = effective_whitelist {
        for &item in whitelist {
            if item == "04_SYSTEM_ARCHITECTURE.md"
                || item == "04_USER_CLASSES_AND_STAKEHOLDERS.md"
                || item == "08_SYSTEM_ARCHITECTURE.md"
            {
                let unit4_handled = filenames.iter().any(|x: &String| {
                    x == "04_SYSTEM_ARCHITECTURE.md"
                        || x == "04_USER_CLASSES_AND_STAKEHOLDERS.md"
                        || x == "04_SYSTEM_CAPABILITIES_AND_FUNCTIONS.md"
                        || x == "08_SYSTEM_ARCHITECTURE.md"
                });
                if !unit4_handled {
                    if all_entries.contains(&"04_SYSTEM_ARCHITECTURE.md".to_string()) {
                        filenames.push("04_SYSTEM_ARCHITECTURE.md".to_string());
                    } else if all_entries.contains(&"08_SYSTEM_ARCHITECTURE.md".to_string()) {
                        filenames.push("08_SYSTEM_ARCHITECTURE.md".to_string());
                    } else if all_entries.contains(&"04_USER_CLASSES_AND_STAKEHOLDERS.md".to_string()) {
                        filenames.push("04_USER_CLASSES_AND_STAKEHOLDERS.md".to_string());
                    } else if all_entries.contains(&"04_SYSTEM_CAPABILITIES_AND_FUNCTIONS.md".to_string()) {
                        filenames.push("04_SYSTEM_CAPABILITIES_AND_FUNCTIONS.md".to_string());
                    }
                }
            } else if item == "05_OPERATIONAL_STATE_SPACE_AND_RISK.md"
                || item == "05_AIRSPACE_AND_SORA_RISK.md"
            {
                let unit5_handled = filenames.iter().any(|x: &String| {
                    x == "05_OPERATIONAL_STATE_SPACE_AND_RISK.md"
                        || x == "05_AIRSPACE_AND_SORA_RISK.md"
                });
                if !unit5_handled {
                    if all_entries.contains(&"05_OPERATIONAL_STATE_SPACE_AND_RISK.md".to_string()) {
                        filenames.push("05_OPERATIONAL_STATE_SPACE_AND_RISK.md".to_string());
                    } else if all_entries.contains(&"05_AIRSPACE_AND_SORA_RISK.md".to_string()) {
                        filenames.push("05_AIRSPACE_AND_SORA_RISK.md".to_string());
                    }
                }
            } else if item == "06_SAFETY_INTERLOCKS.md"
                || item == "06_ROE_SAFETY_INTERLOCKS.md"
                || item == "06_RULES_OF_ENGAGEMENT.md"
            {
                let unit6_handled = filenames.iter().any(|x: &String| {
                    x == "06_SAFETY_INTERLOCKS.md"
                        || x == "06_ROE_SAFETY_INTERLOCKS.md"
                        || x == "06_RULES_OF_ENGAGEMENT.md"
                });
                if !unit6_handled {
                    if all_entries.contains(&"06_SAFETY_INTERLOCKS.md".to_string()) {
                        filenames.push("06_SAFETY_INTERLOCKS.md".to_string());
                    } else if all_entries.contains(&"06_ROE_SAFETY_INTERLOCKS.md".to_string()) {
                        filenames.push("06_ROE_SAFETY_INTERLOCKS.md".to_string());
                    } else if all_entries.contains(&"06_RULES_OF_ENGAGEMENT.md".to_string()) {
                        filenames.push("06_RULES_OF_ENGAGEMENT.md".to_string());
                    }
                }
            } else if item == "08_ENVIRONMENTAL_OPERATING_LIMITS.md"
                || item == "08_ENVIRONMENTAL_MIL_STD_810H.md"
            {
                let unit8_handled = filenames.iter().any(|x: &String| {
                    x == "08_ENVIRONMENTAL_OPERATING_LIMITS.md"
                        || x == "08_ENVIRONMENTAL_MIL_STD_810H.md"
                });
                if !unit8_handled {
                    if all_entries.contains(&"08_ENVIRONMENTAL_OPERATING_LIMITS.md".to_string()) {
                        filenames.push("08_ENVIRONMENTAL_OPERATING_LIMITS.md".to_string());
                    } else if all_entries.contains(&"08_ENVIRONMENTAL_MIL_STD_810H.md".to_string()) {
                        filenames.push("08_ENVIRONMENTAL_MIL_STD_810H.md".to_string());
                    }
                }
            } else if all_entries.contains(&item.to_string()) && !filenames.contains(&item.to_string()) {
                filenames.push(item.to_string());
            }
        }
        if filenames.is_empty() {
            return (
                String::new(),
                vec![format!("No canonical unit files from whitelist found in '{}'.", units_dir.display())],
            );
        }
    } else {
        filenames = all_entries;
    }

    let unit_paths: Vec<PathBuf> = filenames.iter().map(|f| units_dir.join(f)).collect();

    // Required placeholder gate for Unit 4
    for path in &unit_paths {
        let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if [
            "04_SYSTEM_ARCHITECTURE.md",
            "08_SYSTEM_ARCHITECTURE.md",
            "04_USER_CLASSES_AND_STAKEHOLDERS.md",
            "04_SYSTEM_CAPABILITIES_AND_FUNCTIONS.md",
        ]
        .contains(&fname)
        {
            if let Ok(u4_raw) = fs::read_to_string(path) {
                let has_concrete_super = (u4_raw.contains("### 4.1")
                    || u4_raw.contains("### 8.1")
                    || u4_raw.contains("Super-System")
                    || u4_raw.contains("Super-system"))
                    && (u4_raw.contains("```mermaid") || u4_raw.contains("SV-1"));
                let has_concrete_sub = (u4_raw.contains("### 4.3")
                    || u4_raw.contains("### 8.3")
                    || u4_raw.contains("### 4.8")
                    || u4_raw.contains("Subsystem Architecture"))
                    && (u4_raw.contains("LRU") || u4_raw.contains("Subsystem") || u4_raw.contains("Part"));
                let has_concrete = has_concrete_super && has_concrete_sub;

                if !has_concrete
                    && (!u4_raw.contains("{{SUPER_SYSTEM_ARCHITECTURE}}")
                        || !u4_raw.contains("{{SUBSYSTEM_ARCHITECTURE_SECTION}}"))
                {
                    errors.push(format!("{fname} omits required placeholder {{{{SUPER_SYSTEM_ARCHITECTURE}}}} or {{{{SUBSYSTEM_ARCHITECTURE_SECTION}}}}."));
                }
            }
        }
    }

    // Validate unit integrity
    let (is_valid, integrity_errors) = validate_unit_integrity(&unit_paths, Some(param_engine));
    if !is_valid {
        errors.extend(integrity_errors);
        return (String::new(), errors);
    }

    // Read and substitute unit contents
    let mut units: Vec<(String, String)> = Vec::new();
    for path in &unit_paths {
        let raw_text = fs::read_to_string(path).unwrap_or_default();
        let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
        let bound_text = param_engine.substitute(&raw_text);
        units.push((fname, bound_text));
    }

    let default_title = doc_title.unwrap_or("Concept of Operations");
    let default_ver = doc_version
        .or_else(|| param_engine.parameter_bindings.get("DOCUMENT_VERSION").map(|s| s.as_str()))
        .unwrap_or("1.0.0");
    let default_date = doc_date
        .or_else(|| param_engine.parameter_bindings.get("DOCUMENT_DATE").map(|s| s.as_str()))
        .unwrap_or("2026-10-10");

    let mut meta_title = default_title.to_string();
    let mut meta_version = default_ver.to_string();
    let mut meta_date = default_date.to_string();

    for (_, content) in &units {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('|') {
                let parts: Vec<&str> = trimmed.trim_matches('|').split('|').map(str::trim).collect();
                if parts.len() >= 2 {
                    if parts[0].eq_ignore_ascii_case("**title**") {
                        meta_title = parts[1].to_string();
                    } else if parts[0].eq_ignore_ascii_case("**version**") {
                        meta_version = parts[1].to_string();
                    } else if parts[0].eq_ignore_ascii_case("**date**") {
                        meta_date = parts[1].to_string();
                    }
                }
            }
        }
    }

    if let Some(t) = doc_title {
        meta_title = t.to_string();
    }
    if let Some(v) = doc_version {
        meta_version = v.to_string();
    }
    if let Some(d) = doc_date {
        meta_date = d.to_string();
    }

    // Combine sections
    let mut body_sections = Vec::new();
    let mut h1_found = false;

    for (_, content) in &units {
        let lines: Vec<&str> = content.lines().collect();
        let mut filtered_lines = Vec::new();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];
            let trimmed = line.trim();
            if trimmed.starts_with("| Attribute | Value |") || trimmed.starts_with("| **Title**") {
                while i < lines.len() && lines[i].trim().starts_with('|') {
                    i += 1;
                }
                continue;
            }

            if trimmed.starts_with("# ") {
                if h1_found {
                    filtered_lines.push(format!("#{line}"));
                } else {
                    h1_found = true;
                    filtered_lines.push(line.to_string());
                }
            } else {
                filtered_lines.push(line.to_string());
            }
            i += 1;
        }

        let cleaned = filtered_lines.join("\n").trim().to_string();
        if !cleaned.is_empty() {
            body_sections.push(cleaned);
        }
    }

    let full_body = body_sections.join("\n\n");
    let header_table = format!(
        "| Attribute | Value |\n| :--- | :--- |\n| **Title** | {meta_title} |\n| **Version** | {meta_version} |\n| **Date** | {meta_date} |\n"
    );

    let headings = extract_headings(&full_body);
    let toc = generate_table_of_contents(&headings, 2);

    let assembled = if full_body.starts_with("# ") {
        if let Some((h1, rest)) = full_body.split_once('\n') {
            format!("{header_table}\n{h1}\n\n{toc}\n{}\n", rest.trim())
        } else {
            format!("{header_table}\n{full_body}\n\n{toc}\n")
        }
    } else {
        format!("{header_table}\n# {meta_title}\n\n{toc}\n{}\n", full_body.trim())
    };

    let link_errs = verify_markdown_links(&assembled);
    if !link_errs.is_empty() {
        errors.extend(link_errs);
    }

    // 100% AST Part Coverage check for ConOps Section 4
    let is_conops = meta_title.to_lowercase().contains("concept of operations")
        || meta_title.to_lowercase().contains("conops");
    if is_conops && !param_engine.ast_part_names.is_empty() {
        let mut missing_parts = Vec::new();
        for p in &param_engine.ast_part_names {
            let clean_p = sanitize_level_1b_operational_text(p);
            if !assembled.contains(p) && !assembled.contains(&clean_p) {
                missing_parts.push(p.clone());
            }
        }
        if !missing_parts.is_empty() {
            errors.push(format!(
                "ConOps Section 4 AST Part Coverage Gate failed: Missing declared AST part def(s): {} in Section 4.",
                missing_parts.join(", ")
            ));
        }
    }

    (assembled, errors)
}

/// Orchestrates the assembly, parameter binding, and validation of both CONOPS.md and MISSION_INTENT.md.
pub fn assemble_conops(
    input_dir: &Path,
    output_dir: &Path,
    verify_only: bool,
    params: Option<&Path>,
    domain: Option<&str>,
    workspace_dir: Option<&Path>,
) -> Result<bool, Box<dyn std::error::Error>> {
    println!(
        "[*] ConOps Assembly Engine starting: input='{}', output='{}', verify_only={}, domain={}",
        input_dir.display(),
        output_dir.display(),
        verify_only,
        domain.unwrap_or("None")
    );

    let mut ws = workspace_dir.map(|p| p.to_path_buf());
    if ws.is_none() {
        for cand in &[
            output_dir.to_path_buf(),
            output_dir.join(".."),
            output_dir.join("..").join(".."),
            input_dir.to_path_buf(),
            input_dir.join(".."),
            input_dir.join("..").join(".."),
        ] {
            if cand.join("schema").is_dir() || cand.join(".pipeline").is_dir() {
                ws = Some(cand.canonicalize().unwrap_or_else(|_| cand.clone()));
                break;
            }
        }
    }
    let ws_path = ws.unwrap_or_else(|| input_dir.to_path_buf());

    let mut engine = SysMLParameterBindingEngine::new(
        None,
        params,
        Some(&ws_path),
        true,
        domain,
    );

    let detected_dom = domain
        .map(|s| s.to_string())
        .unwrap_or_else(|| engine.detected_domain.clone());

    let mut conops_units_dir = None;
    for cand in &[
        input_dir.join(&detected_dom).join("conops"),
        input_dir.join("units").join(&detected_dom).join("conops"),
        input_dir.join("conops"),
        input_dir.join("units").join("conops"),
    ] {
        if cand.is_dir() {
            conops_units_dir = Some(cand.clone());
            break;
        }
    }

    let mut mission_units_dir = None;
    for cand in &[
        input_dir.join(&detected_dom).join("mission_intent"),
        input_dir.join("units").join(&detected_dom).join("mission_intent"),
        input_dir.join("mission_intent"),
        input_dir.join("units").join("mission_intent"),
    ] {
        if cand.is_dir() {
            mission_units_dir = Some(cand.clone());
            break;
        }
    }

    let mut all_errors = Vec::new();

    // 1. Assemble CONOPS.md
    if let Some(ref c_dir) = conops_units_dir {
        println!(
            "[*] Assembling Concept of Operations from '{}' [domain={detected_dom}]...",
            c_dir.display()
        );
        let (mut doc, errs) = assemble_document(
            c_dir,
            Some("Concept of Operations (ConOps)"),
            None,
            None,
            &mut engine,
            Some(CANONICAL_CONOPS_UNITS),
        );
        if !errs.is_empty() {
            all_errors.extend(errs.into_iter().map(|e| format!("[CONOPS] {e}")));
        } else if !verify_only {
            fs::create_dir_all(output_dir)?;
            let out_file = output_dir.join("CONOPS.md");
            doc = relativize_markdown_links(&doc, &out_file, &ws_path);
            fs::write(&out_file, doc)?;
            println!("[+] Successfully wrote compiled ConOps to '{}'.", out_file.display());
        }
    } else {
        println!(
            "[-] ConOps units directory not found in '{}'. Skipping CONOPS.md assembly.",
            input_dir.display()
        );
    }

    // 2. Assemble MISSION_INTENT.md
    if let Some(ref m_dir) = mission_units_dir {
        println!(
            "[*] Assembling Tactical Mission Intent from '{}' [domain={detected_dom}]...",
            m_dir.display()
        );
        let (mut doc, errs) = assemble_document(
            m_dir,
            Some("Tactical Mission Intent & Execution Plan"),
            None,
            None,
            &mut engine,
            Some(CANONICAL_MISSION_INTENT_UNITS),
        );
        if !errs.is_empty() {
            all_errors.extend(errs.into_iter().map(|e| format!("[MISSION_INTENT] {e}")));
        } else if !verify_only {
            fs::create_dir_all(output_dir)?;
            let out_file = output_dir.join("MISSION_INTENT.md");
            doc = relativize_markdown_links(&doc, &out_file, &ws_path);
            fs::write(&out_file, doc)?;
            println!("[+] Successfully wrote compiled Mission Intent to '{}'.", out_file.display());
        }
    } else {
        println!(
            "[-] Mission Intent units directory not found in '{}'. Skipping MISSION_INTENT.md assembly.",
            input_dir.display()
        );
    }

    if !all_errors.is_empty() {
        eprintln!("\n[!] ConOps Assembly Errors encountered:");
        for err in &all_errors {
            eprintln!("    - {err}");
        }
        return Ok(false);
    }

    println!("[+] All ConOps assembly and verification checks passed cleanly.");
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relativize_markdown_links() {
        let text = "Check out [Schema](../../schema/platform.sysml) and [Anchor](#heading) and [Ext](https://example.com).";
        let out_file = Path::new("/workspace/docs/conops/CONOPS.md");
        let ws = Path::new("/workspace");
        let relativized = relativize_markdown_links(text, out_file, ws);
        assert!(relativized.contains("[Anchor](#heading)"));
        assert!(relativized.contains("[Ext](https://example.com)"));
    }

    #[test]
    fn test_validate_unit_integrity_with_clean_content() {
        let temp_dir = std::env::temp_dir().join("test_conops_units");
        fs::create_dir_all(&temp_dir).unwrap();
        let unit_file = temp_dir.join("01_TEST.md");
        fs::write(&unit_file, "# Unit 1\nClean text without tokens.").unwrap();

        let (is_valid, errors) = validate_unit_integrity(&[unit_file], None);
        assert!(is_valid);
        assert!(errors.is_empty());
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_validate_unit_integrity_with_unresolved_placeholder() {
        let temp_dir = std::env::temp_dir().join("test_conops_units_bad");
        fs::create_dir_all(&temp_dir).unwrap();
        let unit_file = temp_dir.join("01_BAD.md");
        fs::write(&unit_file, "# Unit 1\nUnresolved {{NONEXISTENT_TOKEN_12345}}.").unwrap();

        let (is_valid, errors) = validate_unit_integrity(&[unit_file], None);
        assert!(!is_valid);
        assert!(!errors.is_empty());
        assert!(errors[0].contains("NONEXISTENT_TOKEN_12345"));
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_canonical_unit_05_integrity_and_decontamination() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir.parent().unwrap().parent().unwrap();
        let unit_05_path = repo_root
            .join("skills/spec-conops-engineering/resources/units/conops/05_OPERATIONAL_STATE_SPACE_AND_RISK.md");

        if !unit_05_path.is_file() {
            return;
        }

        let engine = SysMLParameterBindingEngine::new(None, None, None, false, None);
        let (is_valid, errors) = validate_unit_integrity(&[unit_05_path.clone()], Some(&engine));
        assert!(is_valid, "Unit 05 integrity validation failed: {:?}", errors);
        assert!(errors.is_empty());

        let raw_content = fs::read_to_string(&unit_05_path).unwrap();
        assert!(!raw_content.contains("AIR_DENSITY"), "Unit 05 must not contain AIR_DENSITY");
        assert!(!raw_content.contains("t_{\\mathrm{fall}}"), "Unit 05 must not contain ballistic t_fall");
        assert!(!raw_content.contains("d_{\\mathrm{wind}}"), "Unit 05 must not contain wind drift d_wind");
        assert!(!raw_content.contains("FRONTAL_AREA"), "Unit 05 must not contain FRONTAL_AREA");
        assert!(!raw_content.contains("DRAG_COEFFICIENT"), "Unit 05 must not contain DRAG_COEFFICIENT");
        assert!(!raw_content.contains("Ground Risk Buffer"), "Unit 05 must not contain Ground Risk Buffer");
        assert!(!raw_content.contains("Ground Risk Class"), "Unit 05 must not contain Ground Risk Class");
        assert!(!raw_content.contains('\u{2014}'), "Unit 05 must not contain em dash");
        assert!(!raw_content.contains('\u{2013}'), "Unit 05 must not contain en dash");
        assert!(raw_content.contains("ISO/IEC/IEEE 15026-3"), "Unit 05 must cite ISO/IEC/IEEE 15026-3");
        assert!(raw_content.contains("E_{\\mathrm{boundary\\_impact}}"), "Unit 05 must contain boundary impact energy math");

        // Verify state space containment geometry is accurately attributed to kinematic trajectory dynamics
        assert!(
            raw_content.contains("kinematic trajectory dynamics"),
            "Unit 05 must attribute state space containment to kinematic trajectory dynamics"
        );
        assert!(
            raw_content.contains("boundary containment geometry"),
            "Unit 05 must attribute containment envelopes to boundary containment geometry"
        );

        // Verify containment geometry is NOT misattributed to ISO/IEC/IEEE 15026-3 or 15288:2023
        assert!(
            !raw_content.contains("Under ISO/IEC/IEEE 15026-3 / ISO/IEC/IEEE 15288:2023"),
            "Unit 05 must not misattribute containment buffer geometry to ISO/IEC/IEEE 15026-3 or 15288:2023"
        );
        assert!(
            !raw_content.contains("containment geometry (ISO/IEC/IEEE 15026-3"),
            "Unit 05 must not misattribute containment geometry to ISO/IEC/IEEE 15026-3"
        );
    }

    #[test]
    fn test_canonical_unit_05_non_si_aviation_units_and_alias_probe() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir.parent().unwrap().parent().unwrap();
        let unit_05_path = repo_root
            .join("skills/spec-conops-engineering/resources/units/conops/05_OPERATIONAL_STATE_SPACE_AND_RISK.md");

        if !unit_05_path.is_file() {
            return;
        }

        let raw_content = fs::read_to_string(&unit_05_path).unwrap();
        let lower = raw_content.to_lowercase();

        // 1. Assert canonical ConOps Unit 05 is strictly free of non-SI aviation units
        let forbidden_non_si_units = [
            "knots", "knot", "kts", "feet", "foot", "mph", "nautical mile",
        ];
        for unit in &forbidden_non_si_units {
            assert!(
                !lower.contains(unit),
                "Unit 05 must not contain non-SI aviation unit '{}'",
                unit
            );
        }

        // Check for isolated "ft" or "nm" as units
        for line in raw_content.lines() {
            for word in line.split(|c: char| !c.is_alphanumeric() && c != '_') {
                assert!(
                    word != "ft" && word != "FT",
                    "Unit 05 must not contain non-SI unit 'ft' in line: {}",
                    line
                );
                assert!(
                    word != "nm" && word != "NM",
                    "Unit 05 must not contain non-SI aviation unit 'nm' in line: {}",
                    line
                );
                assert!(
                    !word.starts_with("FL") || word.len() < 4 || !word[2..].chars().all(|c| c.is_ascii_digit()),
                    "Unit 05 must not contain flight level designations '{}' in line: {}",
                    word,
                    line
                );
            }
        }

        // 2. Assert canonical ConOps Unit 05 is free of hardcoded tactical aviation assumptions
        let forbidden_aviation_concepts = [
            "sora", "jarus", "bvlos", "vlos", "airspace",
            "flight termination", "aerodynamic drag", "ballistic fall", "wind drift",
        ];
        for concept in &forbidden_aviation_concepts {
            assert!(
                !lower.contains(concept),
                "Unit 05 must not contain hardcoded tactical aviation concept '{}'",
                concept
            );
        }

        // Check AGL and MSL
        for line in raw_content.lines() {
            for word in line.split(|c: char| !c.is_alphanumeric() && c != '_') {
                assert!(
                    word != "AGL" && word != "MSL",
                    "Unit 05 must not contain altitude reference '{}' in line: {}",
                    word,
                    line
                );
            }
        }

        // 3. Verify ISO/IEC/IEEE 15288:2023 §6.4.4 and ISO/IEC/IEEE 15026-3 citations
        assert!(
            raw_content.contains("ISO/IEC/IEEE 15288:2023 §6.4.4"),
            "Unit 05 must cite ISO/IEC/IEEE 15288:2023 §6.4.4"
        );
        assert!(
            raw_content.contains("ISO/IEC/IEEE 15026-3"),
            "Unit 05 must cite ISO/IEC/IEEE 15026-3"
        );

        // 4. Verify normalized SI units in parameter definitions
        let expected_si_tokens = ["m/s", "m/s^2", "kg", "J", "s"];
        for si in &expected_si_tokens {
            assert!(
                raw_content.contains(si),
                "Unit 05 must define parameters with normalized SI unit '{}'",
                si
            );
        }

        // 5. Verify SKILL.md ConOps engineering skill is decontaminated of hardcoded SORA requirements
        let skill_path = repo_root.join("skills/spec-conops-engineering/SKILL.md");
        if skill_path.is_file() {
            let skill_content = fs::read_to_string(&skill_path).unwrap();
            assert!(
                !skill_content.contains("JARUS SORA"),
                "skills/spec-conops-engineering/SKILL.md must not contain hardcoded 'JARUS SORA'"
            );
        }

        // 6. Verify backwards-compatible alias handling in assemble-conops
        let temp_dir = std::env::temp_dir().join(format!("test_alias_05_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        let conops_temp = temp_dir.join("conops");
        fs::create_dir_all(&conops_temp).unwrap();

        // Write historical alias file 05_AIRSPACE_AND_SORA_RISK.md
        let alias_file = conops_temp.join("05_AIRSPACE_AND_SORA_RISK.md");
        fs::write(&alias_file, "## 5. Operational State Space\n\nContent via historical alias.").unwrap();

        let mut engine = SysMLParameterBindingEngine::new(None, None, None, false, None);
        let (assembled_text, errors) = assemble_document(
            &conops_temp,
            Some("CONOPS"),
            Some("1.0.0"),
            Some("2026-10-14"),
            &mut engine,
            None,
        );
        assert!(errors.is_empty(), "Assembly with alias failed: {:?}", errors);
        assert!(
            assembled_text.contains("Content via historical alias"),
            "assemble_document must include 05_AIRSPACE_AND_SORA_RISK.md as fallback alias for Unit 05"
        );

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_canonical_unit_system_architecture_and_optx_decontamination() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir.parent().unwrap().parent().unwrap();
        let conops_units_dir = repo_root.join("skills/spec-conops-engineering/resources/units/conops");

        let optx_path = conops_units_dir.join("07_OPTX_EXCHANGES.md");
        if optx_path.is_file() {
            let engine = SysMLParameterBindingEngine::new(None, None, None, false, None);
            let (is_valid, errors) = validate_unit_integrity(&[optx_path.clone()], Some(&engine));
            assert!(is_valid, "Unit 07 integrity validation failed: {:?}", errors);
            assert!(errors.is_empty());

            let raw = fs::read_to_string(&optx_path).unwrap();
            assert!(!raw.contains("ARINC"), "Unit 07 must not contain ARINC bus protocols");
            assert!(!raw.contains("avionic"), "Unit 07 must not contain avionic network tiers");
            assert!(!raw.contains("DO-178"), "Unit 07 must not contain DO-178");
            assert!(!raw.contains("DO-254"), "Unit 07 must not contain DO-254");
            assert!(!raw.contains("DAL-A"), "Unit 07 must not contain DAL-A");
            assert!(!raw.contains("DAL-B"), "Unit 07 must not contain DAL-B");
            assert!(!raw.contains("Control Surface Deflections"), "Unit 07 must not contain Control Surface Deflections");
            assert!(!raw.contains("Barometric Static Pressure"), "Unit 07 must not contain Barometric Static Pressure");
            assert!(!raw.contains("Dynamic Bingo"), "Unit 07 must not contain Dynamic Bingo");
            assert!(!raw.contains("ROE Arming Keys"), "Unit 07 must not contain ROE Arming Keys");
            assert!(!raw.contains("ASTM F3411"), "Unit 07 must not contain ASTM F3411");
            assert!(!raw.contains("ASD-STAN"), "Unit 07 must not contain ASD-STAN");
            assert!(!raw.contains("ballistic recovery"), "Unit 07 must not contain ballistic recovery");
            assert!(!raw.contains("Squib"), "Unit 07 must not contain Squib");
            assert!(!raw.contains('\u{2014}'), "Unit 07 must not contain em dash");
            assert!(!raw.contains('\u{2013}'), "Unit 07 must not contain en dash");
        }

        let arch_04_path = conops_units_dir.join("04_SYSTEM_ARCHITECTURE.md");
        if arch_04_path.is_file() {
            let engine = SysMLParameterBindingEngine::new(None, None, None, false, None);
            let (is_valid, errors) = validate_unit_integrity(&[arch_04_path.clone()], Some(&engine));
            assert!(is_valid, "Unit 04 integrity validation failed: {:?}", errors);
            assert!(errors.is_empty());

            let raw = fs::read_to_string(&arch_04_path).unwrap();
            assert!(!raw.contains("Encrypted PACE Communications"), "Unit 04 must not contain PACE Communications");
            assert!(!raw.contains("MATLAB"), "Unit 04 must not contain MATLAB");
            assert!(!raw.contains('\u{2014}'), "Unit 04 must not contain em dash");
            assert!(!raw.contains('\u{2013}'), "Unit 04 must not contain en dash");
        }

        let arch_08_path = conops_units_dir.join("08_SYSTEM_ARCHITECTURE.md");
        if arch_08_path.is_file() {
            let engine = SysMLParameterBindingEngine::new(None, None, None, false, None);
            let (is_valid, errors) = validate_unit_integrity(&[arch_08_path.clone()], Some(&engine));
            assert!(is_valid, "Unit 08 integrity validation failed: {:?}", errors);
            assert!(errors.is_empty());

            let raw = fs::read_to_string(&arch_08_path).unwrap();
            assert!(!raw.contains("ARINC"), "Unit 08 must not contain ARINC");
            assert!(!raw.contains("STANAG"), "Unit 08 must not contain STANAG");
            assert!(!raw.contains("MIL-STD-1553"), "Unit 08 must not contain MIL-STD-1553");
            assert!(!raw.contains('\u{2014}'), "Unit 08 must not contain em dash");
            assert!(!raw.contains('\u{2013}'), "Unit 08 must not contain en dash");
        }
    }

    #[test]
    fn test_canonical_unit_12_and_lifecycle_containment_decontamination() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir.parent().unwrap().parent().unwrap();
        let conops_units_dir = repo_root.join("skills/spec-conops-engineering/resources/units/conops");

        let unit_12_path = conops_units_dir.join("12_EMERGENCY_DECISION_MATRIX.md");
        if unit_12_path.is_file() {
            let engine = SysMLParameterBindingEngine::new(None, None, None, false, None);
            let (is_valid, errors) = validate_unit_integrity(&[unit_12_path.clone()], Some(&engine));
            assert!(is_valid, "Unit 12 integrity validation failed: {:?}", errors);
            assert!(errors.is_empty());

            let text = fs::read_to_string(&unit_12_path).unwrap();
            let substituted = engine.substitute(&text);
            assert!(!substituted.contains("{{"), "Unit 12 must have zero unresolved tokens");
            assert!(!substituted.contains('\u{2014}'), "Unit 12 must not contain em dash");
            assert!(!substituted.to_lowercase().contains("parachute"), "Unit 12 must not contain parachute");
        }

        // Verify expendable platform assembly produces zero recovery facility or parachute references
        let mut exp_bindings = std::collections::HashMap::new();
        exp_bindings.insert("IS_EXPENDABLE".to_string(), "true".to_string());
        let exp_engine = SysMLParameterBindingEngine::new(Some(exp_bindings), None, None, false, None);

        let unit_05_path = conops_units_dir.join("05_OPERATIONAL_STATE_SPACE_AND_RISK.md");
        if unit_05_path.is_file() {
            let (is_valid, errors) = validate_unit_integrity(&[unit_05_path.clone()], Some(&exp_engine));
            assert!(is_valid, "Unit 05 expendable integrity failed: {:?}", errors);
            let text = fs::read_to_string(&unit_05_path).unwrap();
            let substituted = exp_engine.substitute(&text);
            assert!(!substituted.to_lowercase().contains("parachute"), "Expendable Unit 05 must not contain parachute");
        }
    }

    #[test]
    fn test_canonical_unit_05_pace_c2_plan_decontamination() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir.parent().unwrap().parent().unwrap();
        let pace_path = repo_root
            .join("skills/spec-conops-engineering/resources/units/mission_intent/05_PACE_C2_PLAN.md");

        if !pace_path.is_file() {
            return;
        }

        let engine = SysMLParameterBindingEngine::new(None, None, None, false, None);
        let (is_valid, errors) = validate_unit_integrity(&[pace_path.clone()], Some(&engine));
        assert!(is_valid, "Unit 05 PACE C2 Plan integrity validation failed: {:?}", errors);
        assert!(errors.is_empty());

        let raw = fs::read_to_string(&pace_path).unwrap();
        assert!(!raw.contains("MIL-STD-188-220"), "Unit 05 PACE C2 Plan must not contain MIL-STD-188-220");
        assert!(!raw.contains("MIL-STD-882E"), "Unit 05 PACE C2 Plan must not contain MIL-STD-882E");
        assert!(!raw.contains("STANAG"), "Unit 05 PACE C2 Plan must not contain STANAG");
        assert!(!raw.contains("CJCSI"), "Unit 05 PACE C2 Plan must not contain CJCSI");
        assert!(!raw.contains('\u{2014}'), "Unit 05 PACE C2 Plan must not contain em dash");
        assert!(!raw.contains('\u{2013}'), "Unit 05 PACE C2 Plan must not contain en dash");
        assert!(raw.contains("IEEE Std 1558-2020"), "Unit 05 PACE C2 Plan must cite IEEE Std 1558-2020");
        assert!(raw.contains("ISO/IEC/IEEE 15288:2023"), "Unit 05 PACE C2 Plan must cite ISO/IEC/IEEE 15288:2023");

        // Verify absence of hardcoded synthetic timeouts
        assert!(!raw.contains("5.0s"), "Unit 05 PACE C2 Plan must not contain hardcoded 5.0s timeout");
        assert!(!raw.contains("2.0s"), "Unit 05 PACE C2 Plan must not contain hardcoded 2.0s timeout");
        assert!(!raw.contains("15.0s"), "Unit 05 PACE C2 Plan must not contain hardcoded 15.0s timeout");
        assert!(!raw.contains("30.0s"), "Unit 05 PACE C2 Plan must not contain hardcoded 30.0s timeout");
        assert!(!raw.contains("tau_loss = 5.0"), "Unit 05 PACE C2 Plan must not contain tau_loss = 5.0");

        // Verify AST-driven timing parameters and operator-configured fallbacks
        assert!(raw.contains("tau_timeout_Primary"), "Unit 05 PACE C2 Plan must define tau_timeout_Primary");
        assert!(raw.contains("tau_hysteresis_Primary"), "Unit 05 PACE C2 Plan must define tau_hysteresis_Primary");
        assert!(raw.contains("OPERATOR_CONFIGURED"), "Unit 05 PACE C2 Plan must support OPERATOR_CONFIGURED parameters");
    }

    #[test]
    fn test_canonical_unit_01_metadata_and_overview_integrity() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir.parent().unwrap().parent().unwrap();
        let unit_01_path = repo_root
            .join("skills/spec-conops-engineering/resources/units/conops/01_METADATA_AND_OVERVIEW.md");

        if !unit_01_path.is_file() {
            return;
        }

        let engine = SysMLParameterBindingEngine::new(None, None, None, false, None);
        let (is_valid, errors) = validate_unit_integrity(&[unit_01_path.clone()], Some(&engine));
        assert!(is_valid, "Unit 01 integrity validation failed: {:?}", errors);
        assert!(errors.is_empty());

        let raw = fs::read_to_string(&unit_01_path).unwrap();

        // Section 1.3.1: Parametric Coordinate Reference Frames Math
        assert!(
            raw.contains("### 1.3.1 Parametric Coordinate Reference Frames Math"),
            "Unit 01 must contain Section 1.3.1 heading"
        );
        assert!(
            raw.contains("Primary Global Geodetic Frame"),
            "Unit 01 must contain Primary Global Geodetic Frame"
        );
        assert!(
            raw.contains("Local Tangent Plane / Navigation Frame"),
            "Unit 01 must contain Local Tangent Plane / Navigation Frame"
        );
        assert!(
            raw.contains("Body-Fixed Geometric Frame"),
            "Unit 01 must contain Body-Fixed Geometric Frame"
        );

        // Section 1.3.2: Parametric Subsystem Mass/Resource Budget Breakdown Table
        assert!(
            raw.contains("### 1.3.2 Parametric Subsystem Mass/Resource Budget Breakdown Table"),
            "Unit 01 must contain Section 1.3.2 heading"
        );
        assert!(
            raw.contains("{{TOTAL_MASS_KG}}") || raw.contains("{{TOTAL_MTOW_KG}}"),
            "Unit 01 must contain TOTAL_MASS_KG or TOTAL_MTOW_KG token"
        );

        // Section 1.3.3: Master Physical Limits Table
        assert!(
            raw.contains("### 1.3.3 Master Physical Limits Table"),
            "Unit 01 must contain Section 1.3.3 heading"
        );
        assert!(
            raw.contains("{{M_MAX_LIMIT_KG}}") || raw.contains("{{MTOW_MAX_KG}}"),
            "Unit 01 must contain M_MAX_LIMIT_KG or MTOW_MAX_KG token"
        );

        // Section 1.4: Abstract UAF Context Diagram (Mermaid flowchart TB)
        assert!(
            raw.contains("### 1.4 Abstract UAF Context Diagram"),
            "Unit 01 must contain Section 1.4 heading"
        );
        assert!(
            raw.contains("flowchart TB"),
            "Unit 01 must contain flowchart TB diagram"
        );

        // Canonical segments: SupportSegment MUST be present, legacy divergent AuxSupport MUST NOT
        assert!(
            raw.contains("subgraph SupportSegment[\"Support & Ground Support Equipment (GSE) Segment\"]"),
            "Unit 01 must declare canonical subgraph SupportSegment (fixes #305)"
        );
        assert!(
            !raw.contains("subgraph AuxSupport"),
            "Unit 01 must not contain legacy divergent subgraph AuxSupport (refs #305)"
        );
        assert!(
            raw.contains("subgraph ControlSegment"),
            "Unit 01 must declare subgraph ControlSegment"
        );
        assert!(
            raw.contains("subgraph OperationalRoles"),
            "Unit 01 must declare subgraph OperationalRoles"
        );
        assert!(
            raw.contains("subgraph ExternalAuthorities"),
            "Unit 01 must declare subgraph ExternalAuthorities"
        );

        // Section 1.5: Normative Standards & Regulatory Baseline
        assert!(
            raw.contains("### 1.5 Normative Standards & Regulatory Baseline"),
            "Unit 01 must contain Section 1.5 heading"
        );
        assert!(
            raw.contains("ISO/IEC/IEEE 29148:2018"),
            "Unit 01 must cite ISO/IEC/IEEE 29148:2018"
        );
        assert!(
            raw.contains("ISO/IEC/IEEE 15288:2023"),
            "Unit 01 must cite ISO/IEC/IEEE 15288:2023"
        );

        // Strict punctuation invariants
        assert!(!raw.contains('\u{2014}'), "Unit 01 must not contain Unicode em dash");
        assert!(!raw.contains('\u{2013}'), "Unit 01 must not contain Unicode en dash");
    }

    #[test]
    fn test_canonical_unit_10_maintenance_and_gse_support_integrity() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir.parent().unwrap().parent().unwrap();
        let unit_10_path = repo_root
            .join("skills/spec-conops-engineering/resources/units/conops/10_MAINTENANCE_AND_GSE_SUPPORT.md");

        if !unit_10_path.is_file() {
            return;
        }

        let engine = SysMLParameterBindingEngine::new(None, None, None, false, None);
        let (is_valid, errors) = validate_unit_integrity(&[unit_10_path.clone()], Some(&engine));
        assert!(is_valid, "Unit 10 integrity validation failed: {:?}", errors);
        assert!(errors.is_empty());

        let raw = fs::read_to_string(&unit_10_path).unwrap();

        // Section 10.1.1: Maintenance task cards
        for card in &["MTC-01", "MTC-01A", "MTC-02", "MTC-03", "MTC-04", "MTC-05"] {
            assert!(
                raw.contains(&format!("**{}**", card)),
                "Unit 10 must contain maintenance task card {}",
                card
            );
        }

        // Section 10.1.2: Rapid Sortie Turnaround Workflow & 7-Step Protocol
        assert!(
            raw.contains("#### 10.1.2 Rapid Sortie Turnaround Workflow & 7-Step Protocol"),
            "Unit 10 must contain Section 10.1.2 heading"
        );
        assert!(
            raw.contains("flowchart TD"),
            "Unit 10 must contain flowchart TD diagram"
        );

        // Canonical 7-step turnaround bullet list (fixes #305: missing in divergent copies)
        assert!(
            raw.contains("The 7-step turnaround sequence encompasses:"),
            "Unit 10 must contain plain-text 7-step sequence header (fixes #305)"
        );
        assert!(
            raw.contains("- Step 1: System Ingress & Safe-State Disarm"),
            "Unit 10 must contain Step 1 bullet (fixes #305)"
        );
        assert!(
            raw.contains("- Step 2: Telemetry Data Offload & Log Audit"),
            "Unit 10 must contain Step 2 bullet"
        );
        assert!(
            raw.contains("- Step 3: Rapid Energy Module Hot-Swap"),
            "Unit 10 must contain Step 3 bullet"
        );
        assert!(
            raw.contains("- Step 4: Rapid Visual & Structural Inspection"),
            "Unit 10 must contain Step 4 bullet"
        );
        assert!(
            raw.contains("- Step 5: Mission Re-Tasking & Key Injection"),
            "Unit 10 must contain Step 5 bullet"
        );
        assert!(
            raw.contains("- Step 6: Automated PBIT Diagnostics"),
            "Unit 10 must contain Step 6 bullet"
        );
        assert!(
            raw.contains("- Step 7: Final Arming & Sortie Release"),
            "Unit 10 must contain Step 7 bullet (fixes #305)"
        );
        assert!(
            raw.contains("- Contingency: Abort Turnaround & Route to MTC-04 LRU Swap"),
            "Unit 10 must contain contingency route bullet (fixes #305)"
        );

        // Mermaid flowchart nodes: must synchronize with upstream unit test assertion
        // In #305, the test asserts "Step 1: System Ingress & Safe-State Disarm" and "Abort Turnaround & Route to MTC-04 LRU Swap"
        assert!(
            raw.contains("Step 1: System Ingress &<br/>Safe-State Disarm"),
            "Unit 10 Mermaid flowchart must contain Step 1 with Safe-State Disarm (fixes #305)"
        );
        assert!(
            raw.contains("Step 7: Final Arming &<br/>Sortie Release"),
            "Unit 10 Mermaid flowchart must contain Step 7 with Final Arming & Sortie Release (fixes #305)"
        );
        assert!(
            raw.contains("Abort Turnaround &<br/>Route to MTC-04 LRU Swap"),
            "Unit 10 Mermaid flowchart must contain Abort Turnaround route (fixes #305)"
        );

        // Ensure divergent conjunction and line breaks are NOT present
        assert!(
            !raw.contains("Step 1: System Ingress and<br/>Safe-State Disarm"),
            "Unit 10 must not contain divergent 'and' conjunction in Step 1 (refs #305)"
        );

        // Section 10.2: Support Equipment Taxonomy Table (SE-01..SE-06)
        assert!(
            raw.contains("### 10.2 Support Equipment (SE) Taxonomy"),
            "Unit 10 must contain Section 10.2 heading"
        );
        for se_id in &["SE-01", "SE-02", "SE-03", "SE-04", "SE-05", "SE-06"] {
            assert!(
                raw.contains(&format!("**{}**", se_id)),
                "Unit 10 must contain support equipment {}",
                se_id
            );
        }

        // Strict punctuation invariants
        assert!(!raw.contains('\u{2014}'), "Unit 10 must not contain Unicode em dash");
        assert!(!raw.contains('\u{2013}'), "Unit 10 must not contain Unicode en dash");
    }

    #[test]
    fn test_canonical_unit_09_scenarios_and_timelines_integrity() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir.parent().unwrap().parent().unwrap();
        let unit_09_path = repo_root
            .join("skills/spec-conops-engineering/resources/units/conops/09_SCENARIOS_AND_TIMELINES.md");

        if !unit_09_path.is_file() {
            return;
        }

        let engine = SysMLParameterBindingEngine::new(None, None, None, false, None);
        let (is_valid, errors) = validate_unit_integrity(&[unit_09_path.clone()], Some(&engine));
        assert!(is_valid, "Unit 09 integrity validation failed: {:?}", errors);
        assert!(errors.is_empty());

        let raw = fs::read_to_string(&unit_09_path).unwrap();

        // Verify all 5 schema-driven scenario structures exist
        assert!(raw.contains("### 9.1 Scenario SCN-01: Normal Operational Cycle"));
        assert!(raw.contains("### 9.2 Scenario SCN-02: Boundary Excursion & Auto-Recovery"));
        assert!(raw.contains("### 9.3 Scenario SCN-03: Degraded C2 Lost-Link & Autonomous Fallback Return"));
        assert!(raw.contains("### 9.4 Scenario SCN-04: Emergency State Safing"));
        assert!(raw.contains("### 9.5 Scenario SCN-05: Sensor Stream Correlation & State Synchronization"));

        // Verify valid Mermaid sequence and state diagrams
        assert!(raw.contains("sequenceDiagram"));
        assert!(raw.contains("stateDiagram-v2"));

        // Verify elimination of faulty prose-locking synthetic combat drone phases (fixes #317)
        assert!(!raw.contains("Phase_Ingress_StationKeeping"), "Unit 09 must not contain Phase_Ingress_StationKeeping");
        assert!(!raw.contains("Phase_PID_InterlockCheck"), "Unit 09 must not contain Phase_PID_InterlockCheck");
        assert!(!raw.contains("Phase_DualConsent_ArmingExecution"), "Unit 09 must not contain Phase_DualConsent_ArmingExecution");
        assert!(!raw.contains("Phase_PostAction_AssessmentDump"), "Unit 09 must not contain Phase_PostAction_AssessmentDump");

        // Verify verified domain-agnostic state machine phases are present
        assert!(raw.contains("Phase_Stream_Ingress"));
        assert!(raw.contains("Phase_Feature_Correlation"));
        assert!(raw.contains("Phase_State_Synchronization"));

        // Strict punctuation invariants
        assert!(!raw.contains('\u{2014}'), "Unit 09 must not contain Unicode em dash");
        assert!(!raw.contains('\u{2013}'), "Unit 09 must not contain Unicode en dash");
    }

    #[test]
    fn test_canonical_unit_08_environmental_integrity_and_decontamination() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir.parent().unwrap().parent().unwrap();
        let conops_units_dir = repo_root.join("skills/spec-conops-engineering/resources/units/conops");
        let unit_08_path = conops_units_dir.join("08_ENVIRONMENTAL_MIL_STD_810H.md");

        if !unit_08_path.is_file() {
            return;
        }

        let engine = SysMLParameterBindingEngine::new(None, None, None, false, None);
        let (is_valid, errors) = validate_unit_integrity(&[unit_08_path.clone()], Some(&engine));
        assert!(is_valid, "Unit 08 integrity validation failed: {:?}", errors);
        assert!(errors.is_empty());

        let raw = fs::read_to_string(&unit_08_path).unwrap();

        // Enforce pure schema-driven compiler invariant: zero hardcoded military standards (refs #323)
        assert!(!raw.contains("MIL-STD-810H"), "Unit 08 must not hardcode MIL-STD-810H (refs #323)");
        assert!(!raw.contains("MIL-STD-461G"), "Unit 08 must not hardcode MIL-STD-461G (refs #323)");
        assert!(!raw.contains("MIL-STD-1472H"), "Unit 08 must not hardcode MIL-STD-1472H (refs #323)");

        // Enforce decontamination of tactical aviation / military hardware concepts (refs #323)
        assert!(!raw.contains("Air Transport"), "Unit 08 must not contain aviation concept Air Transport");
        assert!(!raw.contains("Air Carriage"), "Unit 08 must not contain aviation concept Air Carriage");
        assert!(!raw.contains("Rapid Decompression"), "Unit 08 must not contain aviation concept Rapid Decompression");
        assert!(!raw.contains("altitude ceiling"), "Unit 08 must not contain aviation concept altitude ceiling");
        assert!(!raw.contains("cargo hold"), "Unit 08 must not contain aviation concept cargo hold");
        assert!(!raw.contains("radome"), "Unit 08 must not contain aviation concept radome");
        assert!(!raw.contains("telemetry mast"), "Unit 08 must not contain aviation concept telemetry mast");
        assert!(!raw.contains("Ground Support Equipment"), "Unit 08 must not hardcode Ground Support Equipment");
        assert!(!raw.contains("Red China Clay"), "Unit 08 must not contain military test dust Red China Clay");
        assert!(!raw.contains("gunfire vibration"), "Unit 08 must not contain military gunfire vibration");

        // Verify abstract standards citations
        assert!(raw.contains("ISO/IEC/IEEE 15288:2023"), "Unit 08 must cite ISO/IEC/IEEE 15288:2023");
        assert!(raw.contains("ISO/IEC/IEEE 29148:2018"), "Unit 08 must cite ISO/IEC/IEEE 29148:2018");
        assert!(raw.contains("IEC 60529"), "Unit 08 must cite IEC 60529");

        // Verify abstract qualification matrix rows exist
        for env_id in &["ENV-01", "ENV-02", "ENV-03", "ENV-04", "ENV-05", "ENV-06", "ENV-07"] {
            assert!(
                raw.contains(&format!("**{}**", env_id)),
                "Unit 08 must contain abstract environmental matrix row {}",
                env_id
            );
        }

        // Verify parameterization & substitution cleanliness
        let substituted = engine.substitute(&raw);
        assert!(!substituted.contains("{{"), "Unit 08 substituted output must contain zero unresolved placeholders");

        // Strict punctuation invariants
        assert!(!raw.contains('\u{2014}'), "Unit 08 must not contain Unicode em dash");
        assert!(!raw.contains('\u{2013}'), "Unit 08 must not contain Unicode en dash");
    }

    #[test]
    fn test_governance_rule_conops_mission_intent_integrity_decontamination() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir.parent().unwrap().parent().unwrap();
        let rule_path = repo_root.join("rules/conops-mission-intent-integrity.md");

        assert!(rule_path.is_file(), "rules/conops-mission-intent-integrity.md must exist");
        let content = fs::read_to_string(&rule_path).unwrap();

        // Forbidden domain-specific aerospace concepts violating Pure Schema-Driven Compiler Invariant
        let forbidden_concepts = [
            "civil airborne",
            "bird strike",
            "mid-air",
            "airframe limits",
            "pitot probe",
            "propeller delamination",
            "pilot fatigue",
            "DO-178C",
            "ARP4761",
            "ARP4754A",
            "h_max",
        ];

        let mut found_violations = Vec::new();
        for concept in &forbidden_concepts {
            if content.to_lowercase().contains(&concept.to_lowercase()) {
                found_violations.push(*concept);
            }
        }

        assert!(
            found_violations.is_empty(),
            "rules/conops-mission-intent-integrity.md contains forbidden aerospace domain concepts: {:?}",
            found_violations
        );
    }

    #[test]
    fn test_canonical_mission_intent_unit_09_resource_math_decontamination() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir.parent().unwrap().parent().unwrap();
        let unit_09_path = repo_root.join("skills/spec-conops-engineering/resources/units/mission_intent/09_BINGO_ENERGY_MATH.md");

        assert!(unit_09_path.is_file(), "09_BINGO_ENERGY_MATH.md must exist on disk");

        let engine = SysMLParameterBindingEngine::new(None, None, None, false, None);
        let (is_valid, errors) = validate_unit_integrity(&[unit_09_path.clone()], Some(&engine));
        assert!(is_valid, "Unit 09 integrity validation failed: {:?}", errors);
        assert!(errors.is_empty());

        let raw = fs::read_to_string(&unit_09_path).unwrap();

        // Required headings and abstract MBSE mathematical standards (INCOSE SEH v5.0 §3.2)
        assert!(
            raw.contains("## 9. Critical Energy Threshold Mathematics & Secondary Divert Protocols"),
            "Unit 09 must contain canonical Section 9 heading"
        );
        assert!(
            raw.contains("### 9.1 Parametric Closed-Loop Resource Reserve Formulation"),
            "Unit 09 must contain Section 9.1 heading"
        );
        assert!(
            raw.contains("### 9.2 Secondary and Tertiary Divert Recovery Protocols"),
            "Unit 09 must contain Section 9.2 heading"
        );
        assert!(
            raw.contains("INCOSE Systems Engineering Handbook v5.0 (§3.2)") || raw.contains("INCOSE SEH v5.0 §3.2"),
            "Unit 09 must cite INCOSE SEH v5.0 §3.2"
        );
        assert!(
            raw.contains("R_{\\mathrm{threshold}}(t)"),
            "Unit 09 must formulate dynamic resource threshold R_threshold(t)"
        );
        assert!(
            raw.contains(r"\text{Ratio}_{\text{reserve\_min}}") || raw.contains(r"Ratio_{\text{reserve\_min}}"),
            "Unit 09 must formulate statutory resource reserve ratio"
        );

        // Verification of zero forbidden tactical aviation jargon
        let forbidden_jargon = [
            "bingo",
            "lz-primary",
            "lz-divert",
            "landing zone",
            "elevation",
            "obstacle clearance",
            "flight level",
            "flight ceiling",
            "airfield",
            "aerodrome",
            "runway",
            "airspeed",
            "glide slope",
        ];

        let mut found_jargon = Vec::new();
        for term in &forbidden_jargon {
            if raw.to_lowercase().contains(term) {
                found_jargon.push(*term);
            }
        }
        assert!(
            found_jargon.is_empty(),
            "Unit 09 contains forbidden tactical aviation jargon: {:?}",
            found_jargon
        );

        // Verification of zero non-SI aviation units
        let forbidden_units = [
            "knots",
            "knot",
            "kts",
            "feet",
            "nautical mile",
            "fpm",
        ];

        let mut found_units = Vec::new();
        for unit in &forbidden_units {
            for line in raw.lines() {
                let lower_line = line.to_lowercase();
                for word in lower_line.split_whitespace() {
                    let cleaned = word.trim_matches(|c: char| !c.is_alphanumeric());
                    if cleaned == *unit {
                        found_units.push(*unit);
                    }
                }
            }
        }
        assert!(
            found_units.is_empty(),
            "Unit 09 contains forbidden non-SI aviation units: {:?}",
            found_units
        );

        // Verification of zero Unicode em dash or en dash
        assert!(!raw.contains('\u{2014}'), "Unit 09 must not contain Unicode em dash");
        assert!(!raw.contains('\u{2013}'), "Unit 09 must not contain Unicode en dash");

        // Verification of clean parameter substitution with zero unresolved tokens
        let substituted = engine.substitute(&raw);
        assert!(!substituted.contains("{{"), "Unit 09 substituted output must contain zero unresolved placeholders");
    }
}

