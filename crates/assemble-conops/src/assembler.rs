//! ConOps & Mission Intent modular assembly, link relativization, and verification engine.

use std::fs;
use std::path::{Component, Path, PathBuf};

use regex::Regex;

use crate::params::{SysMLParameterBindingEngine, RAW_TOKEN_FINDER};
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
    "06_RULES_OF_ENGAGEMENT.md",
    "06_ROE_SAFETY_INTERLOCKS.md",
    "07_AIRSPACE_GEOZONES.md",
    "08_GO_NO_GO_MATRIX.md",
    "09_ENERGY_AND_RESERVE_BOUNDS.md",
    "09_BINGO_ENERGY_MATH.md",
    "10_OPERATIONAL_ALLOCATION_TAGS.md",
];

/// Computes a relative path from `base` to `path`.
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

    let link_re = Regex::new(r"\[(?P<label>(?:\\\]|[^\]])+)\]\((?P<target>[^)\n]+)\)").unwrap();
    let scheme_re = Regex::new(r"^[a-zA-Z][a-zA-Z0-9+.-]*://").unwrap();

    link_re
        .replace_all(text, |caps: &regex::Captures| {
            let label = caps.name("label").map(|m| m.as_str()).unwrap_or("");
            let raw_target = caps.name("target").map(|m| m.as_str()).unwrap_or("");
            let target_trimmed = raw_target.trim();

            if target_trimmed.is_empty() {
                return caps.get(0).unwrap().as_str().to_string();
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
                || scheme_re.is_match(url_part)
                || url_part.is_empty()
            {
                return caps.get(0).unwrap().as_str().to_string();
            }

            let (path_part, anchor_suffix) = if let Some((p, a)) = url_part.split_once('#') {
                (p, format!("#{a}"))
            } else {
                (url_part, String::new())
            };

            if path_part.is_empty() {
                return caps.get(0).unwrap().as_str().to_string();
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
            if has_angle_brackets {
                format!("[{label}](<{new_url}>{title_part})")
            } else {
                format!("[{label}]({new_url}{title_part})")
            }
        })
        .to_string()
}

/// Validates unit integrity:
/// 1. File exists and is non-empty.
/// 2. Zero unresolved placeholder tokens matching `{{...}}`.
pub fn validate_unit_integrity(
    unit_paths: &[PathBuf],
    param_engine: Option<&SysMLParameterBindingEngine>,
) -> (bool, Vec<String>) {
    let mut errors = Vec::new();
    let token_re = Regex::new(RAW_TOKEN_FINDER).unwrap();

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

        let placeholders: Vec<String> = token_re
            .find_iter(&processed)
            .map(|m| m.as_str().to_string())
            .collect();
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
            if item == "04_SYSTEM_ARCHITECTURE.md" || item == "04_USER_CLASSES_AND_STAKEHOLDERS.md" {
                let unit4_handled = filenames.iter().any(|x: &String| {
                    x == "04_SYSTEM_ARCHITECTURE.md"
                        || x == "04_USER_CLASSES_AND_STAKEHOLDERS.md"
                        || x == "04_SYSTEM_CAPABILITIES_AND_FUNCTIONS.md"
                });
                if !unit4_handled {
                    if all_entries.contains(&"04_SYSTEM_ARCHITECTURE.md".to_string()) {
                        filenames.push("04_SYSTEM_ARCHITECTURE.md".to_string());
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
                    if all_entries.contains(&item.to_string()) {
                        filenames.push(item.to_string());
                    } else {
                        for alt in &[
                            "06_RULES_OF_ENGAGEMENT.md",
                            "06_SAFETY_INTERLOCKS.md",
                            "06_ROE_SAFETY_INTERLOCKS.md",
                        ] {
                            if all_entries.contains(&alt.to_string()) && !filenames.contains(&alt.to_string()) {
                                filenames.push(alt.to_string());
                                break;
                            }
                        }
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
            "04_USER_CLASSES_AND_STAKEHOLDERS.md",
            "04_SYSTEM_CAPABILITIES_AND_FUNCTIONS.md",
        ]
        .contains(&fname)
        {
            if let Ok(u4_raw) = fs::read_to_string(path) {
                let has_concrete_super = (u4_raw.contains("### 4.1")
                    || u4_raw.contains("Super-System")
                    || u4_raw.contains("Super-system"))
                    && (u4_raw.contains("```mermaid") || u4_raw.contains("SV-1"));
                let has_concrete_sub = (u4_raw.contains("### 4.3")
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

    let title_meta_re = Regex::new(r"(?i)\|\s*\*\*Title\*\*\s*\|\s*([^|]+)\|").unwrap();
    let ver_meta_re = Regex::new(r"(?i)\|\s*\*\*Version\*\*\s*\|\s*([^|]+)\|").unwrap();
    let date_meta_re = Regex::new(r"(?i)\|\s*\*\*Date\*\*\s*\|\s*([^|]+)\|").unwrap();

    for (_, content) in &units {
        for line in content.lines() {
            if let Some(caps) = title_meta_re.captures(line) {
                if let Some(m) = caps.get(1) {
                    meta_title = m.as_str().trim().to_string();
                }
            }
            if let Some(caps) = ver_meta_re.captures(line) {
                if let Some(m) = caps.get(1) {
                    meta_version = m.as_str().trim().to_string();
                }
            }
            if let Some(caps) = date_meta_re.captures(line) {
                if let Some(m) = caps.get(1) {
                    meta_date = m.as_str().trim().to_string();
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
}
