//! Check 17: Safety Integrity, STPA 8-Pillars, and Regulatory Objectives Gate.

use deap_core::rules::EXCLUDED_DIRS;
use deap_core::workspace::is_upstream_compiler;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Outcome of the safety integrity check.
#[derive(Debug, PartialEq, Eq)]
pub enum SafetyCheckOutcome {
    UpstreamClean,
    DownstreamPendingOrClean,
    DownstreamVerified,
}

/// Parsed FMECA table information.
#[derive(Debug, Default)]
pub struct FmecaData {
    pub total_rows: usize,
    pub has_rpn: bool,
    pub ssot_basis_count: usize,
    pub derived_basis_count: usize,
    pub rows: Vec<FmecaRow>,
}

#[derive(Debug, Default)]
pub struct FmecaRow {
    pub id: String,
    pub component: String,
    pub failure_mode: String,
    pub s: Option<i32>,
    pub o: Option<i32>,
    pub d: Option<i32>,
    pub rpn: Option<i32>,
    pub basis: Option<String>,
}

/// Helper checking if a markdown table row is an FMECA header.
fn is_fmeca_header(line: &str) -> bool {
    let lower = line.to_lowercase();
    let has_mode = lower.contains("failure mode")
        || (lower.contains("failure") && (lower.contains("mode") || lower.contains("mechanism")));
    let has_fmeca_col = lower.contains("fmeca id") || lower.contains("fmeca-id");
    let has_fmeca_general = lower.contains("fmeca")
        && (lower.contains("component")
            || lower.contains("subsystem")
            || lower.contains("item")
            || lower.contains("rpn")
            || lower.contains("severity")
            || lower.contains("criticality"));
    has_mode || has_fmeca_col || has_fmeca_general
}

/// Parse FMECA markdown tables from content.
pub fn parse_fmeca_table(content: &str) -> FmecaData {
    let mut data = FmecaData::default();
    let lines: Vec<&str> = content.lines().collect();
    let n = lines.len();
    let mut i = 0;

    while i < n {
        let line = lines[i].trim();
        if line.starts_with('|') && is_fmeca_header(line) {
            // Must be followed by a table separator line containing '---'
            let next_line = if i + 1 < n { lines[i + 1].trim() } else { "" };
            if !next_line.starts_with('|') || !next_line.contains("---") {
                i += 1;
                continue;
            }

            let headers: Vec<String> = line
                .split('|')
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.trim().to_lowercase())
                .collect();

            // Locate columns
            let id_idx = headers.iter().position(|h| {
                h.contains("id") || h.contains("code") || h.starts_with("fm#") || h.starts_with("item#")
            });
            let comp_idx = headers.iter().position(|h| {
                (h.contains("component") || h.contains("subsystem") || h.contains("part") || h.contains("unit"))
                    && !h.contains("effect")
                    && !h.contains("mode")
            }).or_else(|| {
                headers.iter().position(|h| h.contains("item") && !h.contains("id"))
            });
            let mode_idx = headers.iter().position(|h| {
                (h.contains("failure mode") || h.contains("failure mechanism") || h.contains("mode"))
                    && !h.contains("dimension")
                    && !h.contains("effect")
            }).or_else(|| {
                headers.iter().position(|h| h.contains("failure") && !h.contains("id") && !h.contains("effect"))
            });
            let s_idx = headers.iter().position(|h| h == "s" || h.starts_with("sev") || h.contains("severity"));
            let o_idx = headers.iter().position(|h| h == "o" || h.starts_with("occ") || h.contains("occurrence"));
            let d_idx = headers.iter().position(|h| h == "d" || h.starts_with("det") || h.contains("detection"));
            let rpn_idx = headers.iter().position(|h| h == "rpn" || h.contains("priority"));
            let basis_idx = headers.iter().position(|h| h.contains("basis") || h.contains("derivation") || h.contains("provenance"));

            if rpn_idx.is_some() || headers.iter().any(|h| h.contains("rpn")) {
                data.has_rpn = true;
            }

            // Skip separator line (|---|---|...)
            i += 1;
            while i < n && lines[i].trim().starts_with('|') && lines[i].contains("---") {
                i += 1;
            }

            // Parse body rows
            while i < n && lines[i].trim().starts_with('|') {
                let row_line = lines[i].trim();
                let cells: Vec<String> = row_line
                    .split('|')
                    .map(|s| s.trim().to_string())
                    .collect();

                // Strip leading/trailing empty elements from split on |
                let cells: Vec<String> = if cells.len() >= 2 && cells[0].is_empty() && cells[cells.len() - 1].is_empty() {
                    cells[1..cells.len() - 1].to_vec()
                } else {
                    cells
                };

                if !cells.is_empty() {
                    let mut row = FmecaRow::default();
                    if let Some(idx) = id_idx {
                        if idx < cells.len() {
                            row.id = cells[idx].clone();
                        }
                    }
                    if let Some(idx) = comp_idx {
                        if idx < cells.len() {
                            row.component = cells[idx].clone();
                        }
                    }
                    if let Some(idx) = mode_idx {
                        if idx < cells.len() {
                            row.failure_mode = cells[idx].clone();
                        }
                    }

                    // Fallbacks if columns were not matched by header
                    if row.component.is_empty() {
                        if id_idx == Some(0) && cells.len() > 1 {
                            row.component = cells[1].clone();
                        } else if cells.len() > 1 && (cells[0].to_uppercase().starts_with("FM") || cells[0].to_uppercase().starts_with("FMECA")) {
                            row.component = cells[1].clone();
                        }
                    }
                    if row.failure_mode.is_empty() {
                        if comp_idx == Some(1) && cells.len() > 2 {
                            row.failure_mode = cells[2].clone();
                        } else if cells.len() > 2 && (cells[0].to_uppercase().starts_with("FM") || cells[0].to_uppercase().starts_with("FMECA")) {
                            row.failure_mode = cells[2].clone();
                        }
                    }

                    if let Some(idx) = s_idx {
                        if idx < cells.len() {
                            row.s = cells[idx].parse::<i32>().ok();
                        }
                    }
                    if let Some(idx) = o_idx {
                        if idx < cells.len() {
                            row.o = cells[idx].parse::<i32>().ok();
                        }
                    }
                    if let Some(idx) = d_idx {
                        if idx < cells.len() {
                            row.d = cells[idx].parse::<i32>().ok();
                        }
                    }
                    if let Some(idx) = rpn_idx {
                        if idx < cells.len() {
                            row.rpn = cells[idx].parse::<i32>().ok();
                        }
                    }

                    // Check basis
                    let mut found_basis = None;
                    if let Some(idx) = basis_idx {
                        if idx < cells.len() {
                            let cell_upper = cells[idx].to_uppercase();
                            if cell_upper.contains("SSOT") {
                                found_basis = Some("SSOT".to_string());
                            } else if cell_upper.contains("DERIVED") {
                                found_basis = Some("Derived".to_string());
                            }
                        }
                    }
                    if found_basis.is_none() {
                        let full_row = cells.join(" ").to_uppercase();
                        if full_row.contains("SSOT") {
                            found_basis = Some("SSOT".to_string());
                        } else if full_row.contains("DERIVED") {
                            found_basis = Some("Derived".to_string());
                        }
                    }

                    if let Some(ref b) = found_basis {
                        if b == "SSOT" {
                            data.ssot_basis_count += 1;
                        } else if b == "Derived" {
                            data.derived_basis_count += 1;
                        }
                    }
                    row.basis = found_basis;

                    data.total_rows += 1;
                    data.rows.push(row);
                }
                i += 1;
            }
        }
        i += 1;
    }

    data
}

const INTERFACE_KEYWORDS: &[&str] = &[
    "interface", "port", "bus", "signal", "protocol", "packet", "message", "frame",
    "channel", "link", "crc", "timeout", "io", "input", "output", "data",
    "transceiver", "receiver", "uplink", "downlink", "telemetry", "transient", "transients",
    "γ", "\\gamma", "gamma",
];

const STATE_KEYWORDS: &[&str] = &[
    "state", "mode", "transition", "deadlock", "latch", "phase", "statechart",
    "fsm", "sync", "synchronization", "desync", "drift", "stuck", "uninitialized",
    "freeze", "lockup", "trip", "abort", "corruption", "disagreement",
    "φ", "\\phi", "phi",
];

const ACTION_KEYWORDS: &[&str] = &[
    "action", "command", "execution", "operation", "control", "timing", "deadline",
    "compute", "calculation", "process", "logic", "omission", "commission", "latency",
    "jitter", "delay", "rate", "clamping", "limiter", "saturation", "step",
    "overshoot", "schedule", "task", "authority", "miss",
    "ω", "\\omega", "omega",
];

const RESOURCE_KEYWORDS: &[&str] = &[
    "resource", "memory", "cpu", "buffer", "power", "energy", "battery", "thermal",
    "heat", "overheat", "overload", "bandwidth", "storage", "capacity", "stack",
    "heap", "overflow", "underflow", "brownout", "voltage", "current", "load",
    "fault", "short", "sag", "circuit", "crowbar", "degradation", "flash", "ram",
    "supply", "undervoltage", "overvoltage", "seizure", "winding", "windings",
    "wiper", "hardware", "bearing", "dielectric", "squib", "fuse", "fusing",
    "ψ", "\\psi", "psi",
];

fn matches_dimension_lower(lower: &str, keywords: &[&str]) -> bool {
    for &kw in keywords {
        let kw_len = kw.len();
        let mut start = 0;
        while let Some(pos) = lower[start..].find(kw) {
            let actual_pos = start + pos;
            let end_pos = actual_pos + kw_len;

            let prev_ok = if actual_pos == 0 {
                true
            } else {
                let prev_char = lower[..actual_pos].chars().next_back().unwrap();
                !prev_char.is_alphanumeric() && prev_char != '_'
            };

            let next_ok = if end_pos == lower.len() {
                true
            } else {
                let next_char = lower[end_pos..].chars().next().unwrap();
                !next_char.is_alphanumeric() && next_char != '_'
            };

            if prev_ok && next_ok {
                return true;
            }
            start = actual_pos + 1;
            while start < lower.len() && !lower.is_char_boundary(start) {
                start += 1;
            }
        }
    }
    false
}

/// Verify that failure modes across the FMECA table span the 4 universal failure dimensions:
/// Interface (Γ), State (Φ), Action (Ω), Resource (Ψ).
pub fn check_failure_dimension_coverage(content: &str, fmeca: &FmecaData) -> Vec<String> {
    let mut found_interface = false;
    let mut found_state = false;
    let mut found_action = false;
    let mut found_resource = false;

    // Check rows first
    for row in &fmeca.rows {
        let combined = format!("{} {} {}", row.failure_mode, row.component, row.basis.as_deref().unwrap_or(""));
        let combined_lower = combined.to_lowercase();
        if !found_interface && matches_dimension_lower(&combined_lower, INTERFACE_KEYWORDS) {
            found_interface = true;
        }
        if !found_state && matches_dimension_lower(&combined_lower, STATE_KEYWORDS) {
            found_state = true;
        }
        if !found_action && matches_dimension_lower(&combined_lower, ACTION_KEYWORDS) {
            found_action = true;
        }
        if !found_resource && matches_dimension_lower(&combined_lower, RESOURCE_KEYWORDS) {
            found_resource = true;
        }
    }

    // Fallback: check whole content if FMECA is described in text
    let content_lower = content.to_lowercase();
    if !found_interface && matches_dimension_lower(&content_lower, INTERFACE_KEYWORDS) {
        found_interface = true;
    }
    if !found_state && matches_dimension_lower(&content_lower, STATE_KEYWORDS) {
        found_state = true;
    }
    if !found_action && matches_dimension_lower(&content_lower, ACTION_KEYWORDS) {
        found_action = true;
    }
    if !found_resource && matches_dimension_lower(&content_lower, RESOURCE_KEYWORDS) {
        found_resource = true;
    }

    let mut missing = Vec::new();
    if !found_interface {
        missing.push("Interface (Γ)".to_string());
    }
    if !found_state {
        missing.push("State (Φ)".to_string());
    }
    if !found_action {
        missing.push("Action (Ω)".to_string());
    }
    if !found_resource {
        missing.push("Resource (Ψ)".to_string());
    }
    missing
}

/// Verify that all 4 STPA UCA guide words / categories are covered in content.
pub fn check_uca_categories(content: &str) -> Vec<String> {
    let mut missing = Vec::new();
    let lower = content.to_lowercase();

    // 1. Not providing causes hazard
    let has_not_providing = lower.contains("not providing")
        || lower.contains("not provided")
        || lower.contains("omission")
        || lower
            .split_whitespace()
            .collect::<Vec<_>>()
            .windows(2)
            .any(|w| w[0] == "not" && (w[1].starts_with("provid") || w[1] == "providing" || w[1] == "provided"));
    if !has_not_providing {
        missing.push("1. Not providing causes hazard".to_string());
    }

    // 2. Providing causes hazard
    let has_providing_explicit = lower.contains("commission")
        || lower.contains("providing causes")
        || lower.contains("providing incorrectly");
    let has_providing = if has_providing_explicit {
        true
    } else {
        let mut found = false;
        let mut start = 0;
        while let Some(pos) = lower[start..].find("providing") {
            let actual_pos = start + pos;
            let end_pos = actual_pos + "providing".len();

            let prev_ok = actual_pos == 0 || {
                let prev_char = lower[..actual_pos].chars().next_back().unwrap();
                !prev_char.is_alphanumeric() && prev_char != '_'
            };
            let next_ok = end_pos == lower.len() || {
                let next_char = lower[end_pos..].chars().next().unwrap();
                !next_char.is_alphanumeric() && next_char != '_'
            };

            if prev_ok && next_ok {
                let after = lower[end_pos..].trim_start();
                let is_providing_too = after.starts_with("too") && (after.len() == 3 || {
                    let after_too = after["too".len()..].chars().next().unwrap();
                    !after_too.is_alphanumeric() && after_too != '_'
                });
                if !is_providing_too {
                    found = true;
                    break;
                }
            }
            start = actual_pos + 1;
            while start < lower.len() && !lower.is_char_boundary(start) {
                start += 1;
            }
        }
        found
    };
    if !has_providing {
        missing.push("2. Providing causes hazard".to_string());
    }

    // 3. Providing too early, too late, or out of order
    let has_timing = lower.contains("too early")
        || lower.contains("too late")
        || lower.contains("out of order")
        || lower.contains("timing")
        || lower.contains("early/late");
    if !has_timing {
        missing.push("3. Providing too early, too late, or out of order".to_string());
    }

    // 4. Stopped too soon or applied too long
    let has_duration = lower.contains("stopped too soon")
        || lower.contains("applied too long")
        || lower.contains("duration")
        || lower.contains("stopped early");
    if !has_duration {
        missing.push("4. Stopped too soon or applied too long".to_string());
    }

    missing
}

/// Helper checking whether text contains an STPA identifier with prefix, e.g. "l-", "h-", "ls-", "sc-", "uca-".
/// Matches "L-\d+", "$L-\d+", etc.
fn has_stpa_id(lower: &str, prefix: &str) -> bool {
    // Fast path: scan word tokens
    for raw_token in lower.split_whitespace() {
        let token = raw_token.trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '$');
        let clean = token.strip_prefix('$').unwrap_or(token);
        if let Some(rest) = clean.strip_prefix(prefix) {
            let digits = rest.trim_end_matches(|c: char| !c.is_ascii_digit());
            if !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()) {
                return true;
            }
        }
    }
    // Substring boundary fallback
    let mut start = 0;
    while let Some(pos) = lower[start..].find(prefix) {
        let actual_pos = start + pos;
        let prev_ok = if actual_pos == 0 {
            true
        } else {
            let prev_char = lower[..actual_pos].chars().next_back().unwrap();
            !prev_char.is_alphanumeric() && prev_char != '_'
        };
        let after_prefix = &lower[actual_pos + prefix.len()..];
        let mut digits_count = 0;
        for ch in after_prefix.chars() {
            if ch.is_ascii_digit() {
                digits_count += 1;
            } else {
                break;
            }
        }
        if prev_ok && digits_count > 0 {
            let after_digits = &after_prefix[digits_count..];
            let next_ok = if after_digits.is_empty() {
                true
            } else {
                let next_char = after_digits.chars().next().unwrap();
                !next_char.is_alphanumeric() && next_char != '_'
            };
            if next_ok {
                return true;
            }
        }
        start = actual_pos + prefix.len();
        while start < lower.len() && !lower.is_char_boundary(start) {
            start += 1;
        }
    }
    false
}

/// Check 17: Safety Integrity Quality Gate and Regulatory Objectives Completeness Verification.
pub fn check_safety_integrity(
    repo_root: &Path,
    allow_missing_specs: bool,
    strict: bool,
) -> Result<SafetyCheckOutcome, Vec<String>> {
    let safety_dir = repo_root.join("docs").join("safety");

    if is_upstream_compiler(repo_root) {
        if safety_dir.is_dir() {
            let allowed_files = [".gitkeep", "README.md"];
            let mut violations = Vec::new();
            for entry in WalkDir::new(&safety_dir)
                .into_iter()
                .filter_entry(|e| {
                    if e.file_type().is_dir() {
                        let name = e.file_name().to_string_lossy();
                        !EXCLUDED_DIRS.iter().any(|ex| name == ex)
                    } else {
                        true
                    }
                })
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    let fname = entry.file_name().to_string_lossy();
                    if !allowed_files.contains(&fname.as_ref()) {
                        let rel = entry
                            .path()
                            .strip_prefix(repo_root)
                            .unwrap_or(entry.path())
                            .display()
                            .to_string();
                        violations.push(rel);
                    }
                }
            }
            if !violations.is_empty() {
                return Err(vec![format!(
                    "Check 17 failed: Upstream distribution template safety landing zone contains concrete specification files: {}",
                    violations.join(", ")
                )]);
            }
        }
        return Ok(SafetyCheckOutcome::UpstreamClean);
    }

    // Downstream repository validation
    let effective_allow_missing = allow_missing_specs && !strict;

    if !safety_dir.is_dir() {
        if effective_allow_missing {
            return Ok(SafetyCheckOutcome::DownstreamPendingOrClean);
        } else {
            return Err(vec!["Check 17 failed: Safety specification directory 'docs/safety/' is missing.".to_string()]);
        }
    }

    let mut safety_files = Vec::new();
    for entry in WalkDir::new(&safety_dir)
        .into_iter()
        .filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                !EXCLUDED_DIRS.iter().any(|ex| name == ex)
                    && name != "defects"
                    && name != "audits"
                    && name != "decisions"
            } else {
                true
            }
        })
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            let fname = entry.file_name().to_string_lossy();
            if fname.ends_with(".md") && fname != "README.md" {
                safety_files.push(entry.path().to_path_buf());
            }
        }
    }

    if safety_files.is_empty() {
        if effective_allow_missing {
            return Ok(SafetyCheckOutcome::DownstreamPendingOrClean);
        } else {
            return Err(vec!["Check 17 failed: No safety specifications found in 'docs/safety/'.".to_string()]);
        }
    }

    let stpa_matrix_file = safety_dir.join("STPA_MATRIX.md");
    if !stpa_matrix_file.is_file() {
        return Err(vec!["Check 17 failed: Unified STPA matrix 'docs/safety/STPA_MATRIX.md' is missing.".to_string()]);
    }

    let stpa_content = match fs::read_to_string(&stpa_matrix_file) {
        Ok(c) => c,
        Err(e) => return Err(vec![format!("Check 17 failed: Failed to read {}: {}", stpa_matrix_file.display(), e)]),
    };

    let mut combined_content = String::new();
    for sf in &safety_files {
        match fs::read_to_string(sf) {
            Ok(c) => {
                combined_content.push_str(&c);
                combined_content.push_str("\n\n---\n\n");
            }
            Err(e) => {
                return Err(vec![format!("Check 17 failed: Failed to read {}: {}", sf.display(), e)]);
            }
        }
    }

    let mut errors = Vec::new();

    // Citation and protocol validation in safety files
    let slicer = crate::checks::parity_gates::SectionSlicer::new(repo_root);
    let model_text = crate::checks::parity_gates::discover_sysml_model_text(repo_root);
    let model_str = model_text.unwrap_or_default();

    for sf in &safety_files {
        let rel_path = sf
            .strip_prefix(repo_root)
            .unwrap_or(sf)
            .to_string_lossy()
            .to_string();
        if let Ok(c) = fs::read_to_string(sf) {
            crate::checks::parity_gates::audit_spec_citations_and_protocols(
                &c,
                &rel_path,
                &slicer,
                &model_str,
                &mut errors,
            );
        }
    }

    let stpa_lower = stpa_content.to_lowercase();

    // Pillar 1: System Losses (L-1..N)
    let has_loss = stpa_lower.contains("loss") || stpa_lower.contains("losses");
    let has_l_id = has_stpa_id(&stpa_lower, "l-");
    if !(has_loss && has_l_id) {
        errors.push("Pillar 1 violation: Missing System Losses ($L-1..N$) identification in STPA_MATRIX.md.".to_string());
    }

    // Pillar 2: System Hazards (H-1..N)
    let has_hazard = stpa_lower.contains("hazard") || stpa_lower.contains("hazards");
    let has_h_id = has_stpa_id(&stpa_lower, "h-");
    if !(has_hazard && has_h_id) {
        errors.push("Pillar 2 violation: Missing System Hazards ($H-1..N$) identification in STPA_MATRIX.md.".to_string());
    }

    // Pillar 3: Control Structure Topology
    let has_control_struct = stpa_lower.contains("control structure");
    let has_controller = stpa_lower.contains("controller");
    let has_actuator = stpa_lower.contains("actuator");
    if !(has_control_struct || (has_controller && has_actuator)) {
        errors.push("Pillar 3 violation: Missing Hierarchical Control Structure Topology in STPA_MATRIX.md.".to_string());
    }

    // Pillar 4: Unsafe Control Actions (UCA-1..N)
    let has_uca_kw = stpa_lower.contains("unsafe control action")
        || stpa_lower.contains("unsafe control actions");
    let has_uca_id = has_stpa_id(&stpa_lower, "uca-");
    if !(has_uca_kw || has_uca_id) {
        errors.push("Pillar 4 violation: Missing Unsafe Control Actions ($UCA-1..N$) in STPA_MATRIX.md.".to_string());
    }
    let missing_uca_cats = check_uca_categories(&stpa_content);
    if !missing_uca_cats.is_empty() {
        errors.push(format!(
            "Pillar 4 violation: Missing UCA failure mode categories in STPA_MATRIX.md: {}.",
            missing_uca_cats.join(", ")
        ));
    }

    // Pillar 5: Loss Scenarios (LS-1..N)
    let has_ls_kw = stpa_lower.contains("loss scenario")
        || stpa_lower.contains("loss scenarios")
        || stpa_lower.contains("causal scenario")
        || stpa_lower.contains("causal scenarios");
    let has_ls_id = has_stpa_id(&stpa_lower, "ls-");
    if !(has_ls_kw && has_ls_id) {
        errors.push("Pillar 5 violation: Missing Loss Scenarios ($LS-1..N$) & Causal Factors in STPA_MATRIX.md.".to_string());
    }

    // Pillar 6: Formal Safety Constraints (SC-1..N)
    let has_sc_kw = stpa_lower.contains("safety constraint")
        || stpa_lower.contains("safety constraints");
    let has_sc_id = has_stpa_id(&stpa_lower, "sc-");
    if !(has_sc_kw && has_sc_id) {
        errors.push("Pillar 6 violation: Missing Formal Safety Constraints ($SC-1..N$) in STPA_MATRIX.md.".to_string());
    }

    // Pillar 7: FMECA Criticality Matrix
    let has_fmeca = stpa_lower.contains("fmeca") || stpa_lower.contains("failure mode");
    if !has_fmeca {
        errors.push("Pillar 7 violation: Missing FMECA Criticality Matrix in STPA_MATRIX.md.".to_string());
    } else {
        let fmeca = parse_fmeca_table(&stpa_content);
        if fmeca.total_rows == 0 {
            errors.push("Pillar 7 violation: FMECA Criticality Matrix in STPA_MATRIX.md contains 0 rows.".to_string());
        } else if fmeca.total_rows < 15 {
            // Log notice or check row count >= 15 as declared in Pillar 7 baseline
            errors.push(format!(
                "Pillar 7 violation: FMECA Criticality Matrix in STPA_MATRIX.md contains {} rows (minimum 15+ rows required).",
                fmeca.total_rows
            ));
        }

        let has_rpn = fmeca.has_rpn
            || stpa_lower.contains("risk priority number")
            || matches_dimension_lower(&stpa_lower, &["rpn"]);
        if !has_rpn {
            errors.push("Pillar 7 violation: FMECA table missing RPN (Risk Priority Number) calculation.".to_string());
        }

        let total_basis = fmeca.ssot_basis_count + fmeca.derived_basis_count;
        if fmeca.total_rows > 0 && total_basis == 0 {
            errors.push("Pillar 7 violation: FMECA Criticality Matrix missing explicit Derivation Basis classification ('SSOT' / 'Derived').".to_string());
        } else if fmeca.total_rows > 0 && total_basis < fmeca.total_rows {
            errors.push(format!(
                "Pillar 7 violation: FMECA Criticality Matrix contains {} row(s) missing explicit Derivation Basis classification ('SSOT' / 'Derived').",
                fmeca.total_rows - total_basis
            ));
        }

        if fmeca.total_rows > 0 {
            let missing_dims = check_failure_dimension_coverage(&stpa_content, &fmeca);
            if !missing_dims.is_empty() {
                errors.push(format!(
                    "Pillar 7 violation: FMECA table missing coverage for universal failure dimension(s): {} (expected Interface (Γ), State (Φ), Action (Ω), Resource (Ψ)).",
                    missing_dims.join(", ")
                ));
            }

            // Check multi-failure-mode per component coverage (Issue #212)
            let mut component_modes: std::collections::BTreeMap<String, Vec<String>> = std::collections::BTreeMap::new();
            for row in &fmeca.rows {
                let comp_raw = row.component.trim();
                let clean_comp = if let Some(idx) = comp_raw.find(']') {
                    if comp_raw.starts_with('[') {
                        &comp_raw[1..idx]
                    } else {
                        comp_raw
                    }
                } else {
                    comp_raw
                };
                let comp = clean_comp.trim_matches(|c| c == '`' || c == '*').trim();
                if !comp.is_empty() {
                    let mode = row.failure_mode.trim();
                    if !mode.is_empty() {
                        component_modes
                            .entry(comp.to_string())
                            .or_default()
                            .push(mode.to_string());
                    }
                }
            }

            if !component_modes.is_empty() {
                let mut single_mode_components = Vec::new();
                for (comp, modes) in &component_modes {
                    let mut distinct_modes = modes.clone();
                    distinct_modes.sort_by_key(|m| m.to_lowercase());
                    distinct_modes.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
                    if distinct_modes.len() < 2 {
                        single_mode_components.push(comp.clone());
                    }
                }

                if single_mode_components.len() == component_modes.len() {
                    errors.push(format!(
                        "Pillar 7 violation: FMECA matrix exhibits degenerate single-failure-mode-per-component coverage (all {} components have only 1 failure mode). Multi-failure-mode coverage per component is required (minimum >= 2 distinct failure modes per component).",
                        component_modes.len()
                    ));
                } else if !single_mode_components.is_empty() {
                    errors.push(format!(
                        "Pillar 7 violation: FMECA component(s) with degenerate single-failure-mode coverage (minimum >= 2 distinct failure modes required): {}.",
                        single_mode_components.join(", ")
                    ));
                }
            } else {
                errors.push("Pillar 7 violation: FMECA Criticality Matrix contains rows but no valid components or failure modes could be resolved.".to_string());
            }

            // Bidirectional AST closure and negative AST constraint verification (Issue #249)
            let model_text = crate::checks::parity_gates::discover_sysml_model_text(repo_root);
            if let Some(ref text) = model_text {
                let model_parts = deap_core::sysml_ast::parse_part_defs(text);
                let expected_parts: Vec<String> = model_parts
                    .into_iter()
                    .map(|p| p.name)
                    .filter(|n| !n.starts_with('_'))
                    .collect();

                if !expected_parts.is_empty() {
                    let mut table_components: Vec<String> = Vec::new();
                    for row in &fmeca.rows {
                        let comp_raw = row.component.trim();
                        let clean_comp = if let Some(idx) = comp_raw.find(']') {
                            if comp_raw.starts_with('[') {
                                &comp_raw[1..idx]
                            } else {
                                comp_raw
                            }
                        } else {
                            comp_raw
                        };
                        let comp = clean_comp.trim_matches(|c| c == '`' || c == '*').trim();
                        if !comp.is_empty() && !table_components.iter().any(|c| c.eq_ignore_ascii_case(comp)) {
                            table_components.push(comp.to_string());
                        }
                    }

                    // 1. Missing declared AST part def components
                    let mut missing_parts = Vec::new();
                    for p in &expected_parts {
                        if !table_components.iter().any(|c| deap_core::sysml_ast::component_matches(c, p)) {
                            missing_parts.push(p.clone());
                        }
                    }
                    if !missing_parts.is_empty() {
                        missing_parts.sort();
                        errors.push(format!(
                            "Pillar 7 violation: FMECA table missing declared AST part def component(s): {}.",
                            missing_parts.join(", ")
                        ));
                    }

                    // 2. Undeclared phantom components not in AST
                    let mut undeclared_parts = Vec::new();
                    for c in &table_components {
                        if !expected_parts.iter().any(|p| deap_core::sysml_ast::component_matches(c, p)) {
                            undeclared_parts.push(c.clone());
                        }
                    }
                    if !undeclared_parts.is_empty() {
                        undeclared_parts.sort();
                        errors.push(format!(
                            "Pillar 7 violation: FMECA table references undeclared phantom component(s) not in AST: {}.",
                            undeclared_parts.join(", ")
                        ));
                    }

                    // 3. Negative safety model constraints enforcement
                    for c in &table_components {
                        if deap_core::sysml_ast::is_component_negated(c, text)
                            || deap_core::sysml_ast::is_component_negated(c, &combined_content)
                        {
                            errors.push(format!(
                                "Pillar 7 violation: FMECA table references component '{}' contradicted by negative AST model constraint.",
                                c
                            ));
                        }
                    }
                }
            }
        }

        // Validate S/O/D ratings and RPN = S * O * D
        for row in &fmeca.rows {
            if let (Some(s), Some(o), Some(d)) = (row.s, row.o, row.d) {
                if s < 1 || s > 10 || o < 1 || o > 10 || d < 1 || d > 10 {
                    errors.push(format!(
                        "Pillar 7 violation: FMECA row '{}' ratings out of range [1, 10] (S={}, O={}, D={}).",
                        row.id, s, o, d
                    ));
                }
                let expected_rpn = s * o * d;
                if let Some(rpn) = row.rpn {
                    if rpn != expected_rpn {
                        errors.push(format!(
                            "Pillar 7 violation: FMECA row '{}' has invalid RPN calculation -- expected S({}) * O({}) * D({}) = {}, but found RPN = {}.",
                            row.id, s, o, d, expected_rpn, rpn
                        ));
                    }
                }
            }
        }
    }

    // Pillar 8: Regulatory Objectives & Risk Mitigations
    let has_pillar_8_kw = stpa_lower.contains("pillar 8")
        || stpa_lower.contains("regulatory objectives")
        || stpa_lower.contains("sora sail");
    let has_heading_8 = stpa_content.lines().any(|l| {
        let trimmed = l.trim();
        let after_hashes = trimmed.trim_start_matches('#');
        let hash_count = trimmed.len() - after_hashes.len();
        (2..=4).contains(&hash_count) && after_hashes.trim_start().starts_with("8.")
    });
    if !(has_pillar_8_kw || has_heading_8) {
        errors.push("Pillar 8 violation: Missing Pillar 8 in STPA_MATRIX.md (Domain Regulatory Objectives & Integrity Mapping / Regulatory Objectives & Risk Mitigations).".to_string());
    }

    // Commercial Toolchain Hooks (MATLAB / Simulink / Stateflow)
    let has_toolchain = stpa_lower.contains("matlab")
        || stpa_lower.contains("simulink")
        || stpa_lower.contains("stateflow")
        || stpa_lower.contains("embedded coder")
        || stpa_lower.contains("sldv");
    if !has_toolchain {
        errors.push("Commercial Toolchain violation: Missing MATLAB / Simulink / Stateflow / Embedded Coder integration hooks in STPA_MATRIX.md.".to_string());
    }

    if errors.is_empty() {
        Ok(SafetyCheckOutcome::DownstreamVerified)
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    struct TempDir {
        path: std::path::PathBuf,
    }

    impl TempDir {
        fn new(name: &str) -> Self {
            static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let count = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!("verify_safety_{}_{}_{}", name, nanos, count));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn test_missing_safety_dir_allow_missing() {
        let tmp = TempDir::new("allow_missing");
        let res = check_safety_integrity(&tmp.path, true, false);
        assert_eq!(res, Ok(SafetyCheckOutcome::DownstreamPendingOrClean));
    }

    #[test]
    fn test_missing_safety_dir_strict() {
        let tmp = TempDir::new("strict");
        let res = check_safety_integrity(&tmp.path, true, true);
        assert!(res.is_err());
    }

    #[test]
    fn test_uca_categories() {
        let clean = "Not providing command causes hazard. Providing incorrect command. Too early or out of order. Stopped too soon or applied too long.";
        assert!(check_uca_categories(clean).is_empty());

        let incomplete = "Only omission causes hazard.";
        let missing = check_uca_categories(incomplete);
        assert_eq!(missing.len(), 3);
    }

    #[test]
    fn test_fmeca_single_failure_mode_per_component_rejected() {
        let tmp = TempDir::new("single_mode");
        let safety_dir = tmp.path.join("docs").join("safety");
        fs::create_dir_all(&safety_dir).unwrap();

        let mut fmeca_content = String::from(
            "# Safety Analysis\n\n\
             ## Losses\nL-1: Loss of vehicle\n\n\
             ## Hazards\nH-1: Hazard 1\n\n\
             ## Control Structure\nController and actuator interface.\n\n\
             ## Unsafe Control Actions\n\
             UCA-1: Not providing command causes hazard. Providing causes hazard. Too early or out of order. Stopped too soon or applied too long.\n\n\
             ## Loss Scenarios\nLS-1: Scenario\n\n\
             ## Safety Constraints\nSC-1: Constraint\n\n\
             ## Pillar 7: FMECA Criticality Matrix\n\n\
             | FMECA ID | Component | Failure Mode | Severity (S) | Occurrence (O) | Detection (D) | RPN | Derivation Basis |\n\
             |---|---|---|---|---|---|---|---|\n"
        );
        let components = [
            ("Comp1", "Interface bus loss"),
            ("Comp2", "State transition deadlock"),
            ("Comp3", "Action timing latency"),
            ("Comp4", "Resource power fault"),
            ("Comp5", "Interface frame dropout"),
            ("Comp6", "State desync lockup"),
            ("Comp7", "Action command overshoot"),
            ("Comp8", "Resource thermal overheat"),
            ("Comp9", "Interface signal noise"),
            ("Comp10", "State corruption error"),
            ("Comp11", "Action computation delay"),
            ("Comp12", "Resource memory overflow"),
            ("Comp13", "Interface packet timeout"),
            ("Comp14", "State abort trip"),
            ("Comp15", "Action rate limiting"),
            ("Comp16", "Resource battery sag"),
        ];
        for (i, (comp, mode)) in components.iter().enumerate() {
            fmeca_content.push_str(&format!(
                "| FM-{:03} | {} | {} | 2 | 2 | 2 | 8 | Derived |\n",
                i + 1, comp, mode
            ));
        }

        fmeca_content.push_str(
            "\n## Pillar 8: Regulatory Objectives\n\
             | Objective ID | Requirement | Allocation |\n\
             |---|---|---|\n\
             | OBJ-1 | Req 1 | Comp1 |\n\n\
             MATLAB Simulink Stateflow Embedded Coder hooks.\n"
        );

        fs::write(safety_dir.join("STPA_MATRIX.md"), fmeca_content).unwrap();

        let res = check_safety_integrity(&tmp.path, false, true);
        assert!(res.is_err(), "Single-failure-mode-per-component matrix must be rejected, but got: {:?}", res);
    }

    #[test]
    fn test_fmeca_multi_failure_mode_per_component_passes() {
        let tmp = TempDir::new("multi_mode");
        let safety_dir = tmp.path.join("docs").join("safety");
        fs::create_dir_all(&safety_dir).unwrap();

        let mut fmeca_content = String::from(
            "# Safety Analysis\n\n\
             ## Losses\nL-1: Loss of vehicle\n\n\
             ## Hazards\nH-1: Hazard 1\n\n\
             ## Control Structure\nController and actuator interface.\n\n\
             ## Unsafe Control Actions\n\
             UCA-1: Not providing command causes hazard. Providing causes hazard. Too early or out of order. Stopped too soon or applied too long.\n\n\
             ## Loss Scenarios\nLS-1: Scenario\n\n\
             ## Safety Constraints\nSC-1: Constraint\n\n\
             ## Pillar 7: FMECA Criticality Matrix\n\n\
             | FMECA ID | Component | Failure Mode | Severity (S) | Occurrence (O) | Detection (D) | RPN | Derivation Basis |\n\
             |---|---|---|---|---|---|---|---|\n"
        );
        let component_pairs = [
            ("Comp1", "Interface bus dropout", "State desync lockup"),
            ("Comp2", "Action timing latency", "Resource power fault"),
            ("Comp3", "Interface frame corruption", "State transition freeze"),
            ("Comp4", "Action command overshoot", "Resource thermal overheat"),
            ("Comp5", "Interface packet loss", "State corruption error"),
            ("Comp6", "Action rate limiting", "Resource battery sag"),
            ("Comp7", "Interface bus jitter", "State abort trip"),
            ("Comp8", "Action torque clamping", "Resource memory exhaustion"),
        ];
        let mut row_idx = 1;
        for (comp, mode1, mode2) in &component_pairs {
            fmeca_content.push_str(&format!(
                "| FM-{:03} | {} | {} | 2 | 2 | 2 | 8 | Derived |\n",
                row_idx, comp, mode1
            ));
            row_idx += 1;
            fmeca_content.push_str(&format!(
                "| FM-{:03} | {} | {} | 2 | 2 | 2 | 8 | Derived |\n",
                row_idx, comp, mode2
            ));
            row_idx += 1;
        }

        fmeca_content.push_str(
            "\n## Pillar 8: Regulatory Objectives\n\
             | Objective ID | Requirement | Allocation |\n\
             |---|---|---|\n\
             | OBJ-1 | Req 1 | Comp1 |\n\n\
             MATLAB Simulink Stateflow Embedded Coder hooks.\n"
        );

        fs::write(safety_dir.join("STPA_MATRIX.md"), fmeca_content).unwrap();

        let res = check_safety_integrity(&tmp.path, false, true);
        assert_eq!(res, Ok(SafetyCheckOutcome::DownstreamVerified));
    }

    #[test]
    fn test_fmeca_partial_single_mode_component_fails() {
        let tmp = TempDir::new("partial_single");
        let safety_dir = tmp.path.join("docs").join("safety");
        fs::create_dir_all(&safety_dir).unwrap();

        let mut fmeca_content = String::from(
            "# Safety Analysis\n\n\
             ## Losses\nL-1: Loss of vehicle\n\n\
             ## Hazards\nH-1: Hazard 1\n\n\
             ## Control Structure\nController and actuator interface.\n\n\
             ## Unsafe Control Actions\n\
             UCA-1: Not providing command causes hazard. Providing causes hazard. Too early or out of order. Stopped too soon or applied too long.\n\n\
             ## Loss Scenarios\nLS-1: Scenario\n\n\
             ## Safety Constraints\nSC-1: Constraint\n\n\
             ## Pillar 7: FMECA Criticality Matrix\n\n\
             | FMECA ID | Component | Failure Mode | Severity (S) | Occurrence (O) | Detection (D) | RPN | Derivation Basis |\n\
             |---|---|---|---|---|---|---|---|\n"
        );
        for i in 1..=7 {
            fmeca_content.push_str(&format!(
                "| FM-{:03} | Comp{} | Interface bus loss {} | 2 | 2 | 2 | 8 | Derived |\n\
                 | FM-{:03} | Comp{} | State desync lockup {} | 2 | 2 | 2 | 8 | Derived |\n",
                (i - 1) * 2 + 1, i, i,
                (i - 1) * 2 + 2, i, i
            ));
        }
        fmeca_content.push_str(
            "| FM-015 | Comp8 | Action command latency and resource power fault | 2 | 2 | 2 | 8 | Derived |\n"
        );

        fmeca_content.push_str(
            "\n## Pillar 8: Regulatory Objectives\n\
             | Objective ID | Requirement | Allocation |\n\
             |---|---|---|\n\
             | OBJ-1 | Req 1 | Comp1 |\n\n\
             MATLAB Simulink Stateflow Embedded Coder hooks.\n"
        );

        fs::write(safety_dir.join("STPA_MATRIX.md"), fmeca_content).unwrap();

        let res = check_safety_integrity(&tmp.path, false, true);
        assert!(res.is_err());
        let errs = res.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("Comp8") && e.contains("degenerate single-failure-mode coverage")));
    }

    #[test]
    fn test_fmeca_duplicate_mode_does_not_satisfy_multi_mode() {
        let tmp = TempDir::new("duplicate_mode");
        let safety_dir = tmp.path.join("docs").join("safety");
        fs::create_dir_all(&safety_dir).unwrap();

        let mut fmeca_content = String::from(
            "# Safety Analysis\n\n\
             ## Losses\nL-1: Loss of vehicle\n\n\
             ## Hazards\nH-1: Hazard 1\n\n\
             ## Control Structure\nController and actuator interface.\n\n\
             ## Unsafe Control Actions\n\
             UCA-1: Not providing command causes hazard. Providing causes hazard. Too early or out of order. Stopped too soon or applied too long.\n\n\
             ## Loss Scenarios\nLS-1: Scenario\n\n\
             ## Safety Constraints\nSC-1: Constraint\n\n\
             ## Pillar 7: FMECA Criticality Matrix\n\n\
             | FMECA ID | Component | Failure Mode | Severity (S) | Occurrence (O) | Detection (D) | RPN | Derivation Basis |\n\
             |---|---|---|---|---|---|---|---|\n"
        );
        fmeca_content.push_str(
            "| FM-001 | Comp1 | Interface bus dropout | 2 | 2 | 2 | 8 | Derived |\n\
             | FM-002 | Comp1 | Interface bus dropout | 2 | 2 | 2 | 8 | Derived |\n"
        );
        let pairs = [
            ("Comp2", "Action timing latency", "Resource power fault"),
            ("Comp3", "Interface frame corruption", "State transition freeze"),
            ("Comp4", "Action command overshoot", "Resource thermal overheat"),
            ("Comp5", "Interface packet loss", "State corruption error"),
            ("Comp6", "Action rate limiting", "Resource battery sag"),
            ("Comp7", "Interface bus jitter", "State abort trip"),
            ("Comp8", "Action torque clamping", "Resource memory exhaustion"),
        ];
        let mut row_idx = 3;
        for (comp, mode1, mode2) in &pairs {
            fmeca_content.push_str(&format!(
                "| FM-{:03} | {} | {} | 2 | 2 | 2 | 8 | Derived |\n",
                row_idx, comp, mode1
            ));
            row_idx += 1;
            fmeca_content.push_str(&format!(
                "| FM-{:03} | {} | {} | 2 | 2 | 2 | 8 | Derived |\n",
                row_idx, comp, mode2
            ));
            row_idx += 1;
        }

        fmeca_content.push_str(
            "\n## Pillar 8: Regulatory Objectives\n\
             | Objective ID | Requirement | Allocation |\n\
             |---|---|---|\n\
             | OBJ-1 | Req 1 | Comp1 |\n\n\
             MATLAB Simulink Stateflow Embedded Coder hooks.\n"
        );

        fs::write(safety_dir.join("STPA_MATRIX.md"), fmeca_content).unwrap();

        let res = check_safety_integrity(&tmp.path, false, true);
        assert!(res.is_err());
        let errs = res.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("Comp1")));
    }

    #[test]
    fn test_fmeca_missing_derivation_basis_fails() {
        let tmp = TempDir::new("missing_basis");
        let safety_dir = tmp.path.join("docs").join("safety");
        fs::create_dir_all(&safety_dir).unwrap();

        let mut fmeca_content = String::from(
            "# Safety Analysis\n\n\
             ## Losses\nL-1: Loss of vehicle\n\n\
             ## Hazards\nH-1: Hazard 1\n\n\
             ## Control Structure\nController and actuator interface.\n\n\
             ## Unsafe Control Actions\n\
             UCA-1: Not providing command causes hazard. Providing causes hazard. Too early or out of order. Stopped too soon or applied too long.\n\n\
             ## Loss Scenarios\nLS-1: Scenario\n\n\
             ## Safety Constraints\nSC-1: Constraint\n\n\
             ## Pillar 7: FMECA Criticality Matrix\n\n\
             | FMECA ID | Component | Failure Mode | Severity (S) | Occurrence (O) | Detection (D) | RPN | Derivation Basis |\n\
             |---|---|---|---|---|---|---|---|\n"
        );
        let pairs = [
            ("Comp1", "Interface bus dropout", "State desync lockup"),
            ("Comp2", "Action timing latency", "Resource power fault"),
            ("Comp3", "Interface frame corruption", "State transition freeze"),
            ("Comp4", "Action command overshoot", "Resource thermal overheat"),
            ("Comp5", "Interface packet loss", "State corruption error"),
            ("Comp6", "Action rate limiting", "Resource battery sag"),
            ("Comp7", "Interface bus jitter", "State abort trip"),
            ("Comp8", "Action torque clamping", "Resource memory exhaustion"),
        ];
        let mut row_idx = 1;
        for (comp, mode1, mode2) in &pairs {
            let basis1 = if row_idx == 1 { "" } else { "Derived" };
            fmeca_content.push_str(&format!(
                "| FM-{:03} | {} | {} | 2 | 2 | 2 | 8 | {} |\n",
                row_idx, comp, mode1, basis1
            ));
            row_idx += 1;
            fmeca_content.push_str(&format!(
                "| FM-{:03} | {} | {} | 2 | 2 | 2 | 8 | Derived |\n",
                row_idx, comp, mode2
            ));
            row_idx += 1;
        }

        fmeca_content.push_str(
            "\n## Pillar 8: Regulatory Objectives\n\
             | Objective ID | Requirement | Allocation |\n\
             |---|---|---|\n\
             | OBJ-1 | Req 1 | Comp1 |\n\n\
             MATLAB Simulink Stateflow Embedded Coder hooks.\n"
        );

        fs::write(safety_dir.join("STPA_MATRIX.md"), fmeca_content).unwrap();

        let res = check_safety_integrity(&tmp.path, false, true);
        assert!(res.is_err());
        let errs = res.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("missing explicit Derivation Basis classification")));
    }

    #[test]
    fn test_issue_249_check17_rejects_fmeca_with_undeclared_or_negated_component() {
        let temp = TempDir::new("check17_issue249");
        let safety_dir = temp.path.join("docs").join("safety");
        let schema_dir = temp.path.join("schema");
        fs::create_dir_all(&safety_dir).unwrap();
        fs::create_dir_all(&schema_dir).unwrap();

        // Model declares PrimaryProcessor and SensorUnit, and negates the backup unit: backupEnabled = 0; backupSystem = "No";
        let model_text = r#"package SystemSSOT {
    part def PrimaryProcessor;
    part def SensorUnit;
    attribute backupEnabled = 0;
    attribute backupSystem = "No";
}
"#;
        fs::write(schema_dir.join("model.sysml"), model_text).unwrap();

        // FMECA table includes undeclared / negated AuxiliaryBackupUnit
        let mut content = String::from("# System Safety & STPA Matrix\n\n");
        content.push_str("## 1. System Losses ($L-1..N$)\n- Loss L-1: Loss of System Function\n\n");
        content.push_str("## 2. System Hazards ($H-1..N$)\n- Hazard H-1: Loss of Control\n\n");
        content.push_str("## 3. Control Structure Topology\n```mermaid\ngraph TD\n  A --> B\n```\n\n");
        content.push_str("## 4. Unsafe Control Actions ($UCA-1..N$)\n- UCA-1: Providing Halt command too late\n- UCA-2: Not providing Halt command\n- UCA-3: Stopped too soon\n- UCA-4: Applied too long\n\n");
        content.push_str("## 5. Loss Scenarios ($LS-1..N$)\n- Scenario LS-1: Sensor failure causes loss\n\n");
        content.push_str("## 6. Formal Safety Constraints ($SC-1..N$)\n- SC-01: System shall remain within safe operating envelope\n\n");
        content.push_str("## 7. FMECA Criticality Matrix\n\n");
        content.push_str("| FMECA ID | Component | Failure Mode | Potential Effect | Severity (S) | Occurrence (O) | Detection (D) | RPN (S x O x D) | Mitigation | Derivation Basis |\n");
        content.push_str("| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n");
        // 14 rows of PrimaryProcessor / SensorUnit
        for i in 1..=8 {
            content.push_str(&format!("| FMECA-{:03} | PrimaryProcessor | Mode {} (Interface Γ, State Φ) | Effect | 5 | 2 | 2 | 20 | Mit | SSOT |\n", i, i));
        }
        for i in 9..=14 {
            content.push_str(&format!("| FMECA-{:03} | SensorUnit | Mode {} (Action Ω, Resource Ψ) | Effect | 5 | 2 | 2 | 20 | Mit | SSOT |\n", i, i));
        }
        // Rows 15 & 16: AuxiliaryBackupUnit (undeclared & negated by model, 2 distinct modes)
        content.push_str("| FMECA-015 | AuxiliaryBackupUnit | Activation Failure (Action Ω) | Loss of redundancy | 10 | 2 | 2 | 40 | Watchdog | Derived |\n");
        content.push_str("| FMECA-016 | AuxiliaryBackupUnit | Supply Depletion (Resource Ψ) | Loss of redundancy | 10 | 2 | 2 | 40 | Dual Supply | Derived |\n\n");
        content.push_str("## 8. Domain Regulatory Objectives & Integrity Mapping\n- Pillar 8 SafetyIntegrityLevel_2 objectives\n\n");
        content.push_str("## Commercial Toolchain Hooks\n- MATLAB / Simulink / Stateflow hooks\n");

        fs::write(safety_dir.join("STPA_MATRIX.md"), content).unwrap();

        let result = check_safety_integrity(&temp.path, false, true);
        assert!(result.is_err(), "Expected Check 17 to fail when FMECA references undeclared/negated AuxiliaryBackupUnit");
        let errs = result.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("Pillar 7 violation") && e.contains("'AuxiliaryBackupUnit' contradicted by negative AST model constraint")),
            "Expected Pillar 7 negation error referencing AuxiliaryBackupUnit, got: {:?}", errs);
        assert!(!errs.iter().any(|e| e.contains("contradicted by negative AST model constraint") && (e.contains("'PrimaryProcessor'") || e.contains("'SensorUnit'"))),
            "Declared, non-negated components must not be flagged as negated, got: {:?}", errs);
    }

    #[test]
    fn test_issue_249_check17_rejects_missing_declared_part() {
        let temp = TempDir::new("check17_missing_part");
        let safety_dir = temp.path.join("docs").join("safety");
        let schema_dir = temp.path.join("schema");
        fs::create_dir_all(&safety_dir).unwrap();
        fs::create_dir_all(&schema_dir).unwrap();

        // Model declares FlightComputer and ActuatorUnit
        let model_text = r#"package SystemSSOT {
    part def FlightComputer;
    part def ActuatorUnit;
}
"#;
        fs::write(schema_dir.join("model.sysml"), model_text).unwrap();

        // FMECA table covers FlightComputer across all 15 rows, but completely omits ActuatorUnit
        let mut content = String::from("# System Safety & STPA Matrix\n\n");
        content.push_str("## 1. System Losses ($L-1..N$)\n- Loss L-1: Loss of Vehicle\n\n");
        content.push_str("## 2. System Hazards ($H-1..N$)\n- Hazard H-1: Loss of Control\n\n");
        content.push_str("## 3. Control Structure Topology\n```mermaid\ngraph TD\n  A --> B\n```\n\n");
        content.push_str("## 4. Unsafe Control Actions ($UCA-1..N$)\n- UCA-1: Providing Land command too late\n- UCA-2: Not providing Land command\n- UCA-3: Stopped too soon\n- UCA-4: Applied too long\n\n");
        content.push_str("## 5. Loss Scenarios ($LS-1..N$)\n- Scenario LS-1: Sensor failure causes loss\n\n");
        content.push_str("## 6. Formal Safety Constraints ($SC-1..N$)\n- SC-01: Vehicle shall remain in safe corridor\n\n");
        content.push_str("## 7. FMECA Criticality Matrix\n\n");
        content.push_str("| FMECA ID | Component | Failure Mode | Potential Effect | Severity (S) | Occurrence (O) | Detection (D) | RPN (S x O x D) | Mitigation | Derivation Basis |\n");
        content.push_str("| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n");
        for i in 1..=15 {
            let dim = match i % 4 {
                0 => "Interface Γ",
                1 => "State Φ",
                2 => "Action Ω",
                _ => "Resource Ψ",
            };
            content.push_str(&format!("| FMECA-{:03} | FlightComputer | Mode {} ({}) | Effect | 5 | 2 | 2 | 20 | Mit | SSOT |\n", i, i, dim));
        }
        content.push_str("\n## 8. Domain Regulatory Objectives & Integrity Mapping\n- Pillar 8 SORA SAIL II objectives\n\n");
        content.push_str("## Commercial Toolchain Hooks\n- MATLAB / Simulink / Stateflow hooks\n");

        fs::write(safety_dir.join("STPA_MATRIX.md"), content).unwrap();

        let result = check_safety_integrity(&temp.path, false, true);
        assert!(result.is_err(), "Expected Check 17 to fail when declared ActuatorUnit is missing from FMECA");
        let errs = result.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("missing declared AST part def component(s): ActuatorUnit")),
            "Expected missing declared part error for ActuatorUnit, got: {:?}", errs);
    }

    #[test]
    fn test_issue_249_check17_valid_bidirectional_ast_closure_passes() {
        let temp = TempDir::new("check17_valid_ast");
        let safety_dir = temp.path.join("docs").join("safety");
        let schema_dir = temp.path.join("schema");
        fs::create_dir_all(&safety_dir).unwrap();
        fs::create_dir_all(&schema_dir).unwrap();

        // Model declares FlightComputer and SensorUnit with valid positive configuration
        let model_text = r#"package SystemSSOT {
    part def FlightComputer;
    part def SensorUnit;
}
"#;
        fs::write(schema_dir.join("model.sysml"), model_text).unwrap();

        let mut content = String::from("# System Safety & STPA Matrix\n\n");
        content.push_str("## 1. System Losses ($L-1..N$)\n- Loss L-1: Loss of Vehicle\n\n");
        content.push_str("## 2. System Hazards ($H-1..N$)\n- Hazard H-1: Loss of Control\n\n");
        content.push_str("## 3. Control Structure Topology\n```mermaid\ngraph TD\n  A --> B\n```\n\n");
        content.push_str("## 4. Unsafe Control Actions ($UCA-1..N$)\n- UCA-1: Providing Land command too late\n- UCA-2: Not providing Land command\n- UCA-3: Stopped too soon\n- UCA-4: Applied too long\n\n");
        content.push_str("## 5. Loss Scenarios ($LS-1..N$)\n- Scenario LS-1: Sensor failure causes loss\n\n");
        content.push_str("## 6. Formal Safety Constraints ($SC-1..N$)\n- SC-01: Vehicle shall remain in safe corridor\n\n");
        content.push_str("## 7. FMECA Criticality Matrix\n\n");
        content.push_str("| FMECA ID | Component | Failure Mode | Potential Effect | Severity (S) | Occurrence (O) | Detection (D) | RPN (S x O x D) | Mitigation | Derivation Basis |\n");
        content.push_str("| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n");
        // 8 rows for FlightComputer spanning dimensions
        for i in 1..=8 {
            let dim = match i % 4 {
                0 => "Interface Γ",
                1 => "State Φ",
                2 => "Action Ω",
                _ => "Resource Ψ",
            };
            content.push_str(&format!("| FMECA-{:03} | FlightComputer | Mode {} ({}) | Effect | 5 | 2 | 2 | 20 | Mit | SSOT |\n", i, i, dim));
        }
        // 8 rows for SensorUnit spanning dimensions
        for i in 9..=16 {
            let dim = match i % 4 {
                0 => "Interface Γ",
                1 => "State Φ",
                2 => "Action Ω",
                _ => "Resource Ψ",
            };
            content.push_str(&format!("| FMECA-{:03} | SensorUnit | Mode {} ({}) | Effect | 5 | 2 | 2 | 20 | Mit | Derived |\n", i, i, dim));
        }
        content.push_str("\n## 8. Domain Regulatory Objectives & Integrity Mapping\n- Pillar 8 SORA SAIL II objectives\n\n");
        content.push_str("## Commercial Toolchain Hooks\n- MATLAB / Simulink / Stateflow hooks\n");

        fs::write(safety_dir.join("STPA_MATRIX.md"), content).unwrap();

        let result = check_safety_integrity(&temp.path, false, true);
        assert!(result.is_ok(), "Expected Check 17 to pass with valid bidirectional AST closure, got: {:?}", result);
    }

    #[test]
    fn test_defect_333_missing_stpa_matrix_rejected() {
        let temp = TempDir::new("check17_missing_stpa");
        let safety_dir = temp.path.join("docs").join("safety");
        fs::create_dir_all(&safety_dir).unwrap();

        // Fragmented safety files present, but unified STPA_MATRIX.md missing
        fs::write(safety_dir.join("01_LOSSES.md"), "# Losses\nL-1: Loss\n").unwrap();
        fs::write(safety_dir.join("05_FMECA.md"), "# FMECA\n| FM | Comp | Mode |\n").unwrap();

        let result = check_safety_integrity(&temp.path, false, true);
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("Unified STPA matrix 'docs/safety/STPA_MATRIX.md' is missing")));
    }

    #[test]
    fn test_defect_333_hollow_stpa_matrix_rejected() {
        let temp = TempDir::new("check17_hollow_stpa");
        let safety_dir = temp.path.join("docs").join("safety");
        fs::create_dir_all(&safety_dir).unwrap();

        // STPA_MATRIX.md only contains UCA table, missing all other 7 pillars
        let hollow_content = "# STPA Cross-Traceability Matrix\n\n\
| UCA ID | Controller | Control Action | Guide Word |\n\
|---|---|---|---|\n\
| UCA-001 | Sys | Action | Not providing |\n";
        fs::write(safety_dir.join("STPA_MATRIX.md"), hollow_content).unwrap();

        let result = check_safety_integrity(&temp.path, false, true);
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("Pillar 1 violation")));
    }
}
