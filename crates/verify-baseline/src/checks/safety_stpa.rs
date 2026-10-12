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
    if lower.contains("fmeca") || lower.contains("failure mode") {
        return true;
    }
    let words: Vec<&str> = lower.split_whitespace().collect();
    words.windows(2).any(|w| w[0].contains("failure") && w[1].contains("mode"))
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
            let headers: Vec<String> = line
                .split('|')
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.trim().to_lowercase())
                .collect();

            // Locate columns
            let id_idx = headers.iter().position(|h| h.contains("id") || h.contains("item"));
            let comp_idx = headers.iter().position(|h| h.contains("component") || h.contains("item") || h.contains("subsystem"));
            let mode_idx = headers.iter().position(|h| h.contains("failure mode") || h.contains("failure") || h.contains("mode"));
            let s_idx = headers.iter().position(|h| h == "s" || h.starts_with("sev") || h.contains("severity"));
            let o_idx = headers.iter().position(|h| h == "o" || h.starts_with("occ") || h.contains("occurrence"));
            let d_idx = headers.iter().position(|h| h == "d" || h.starts_with("det") || h.contains("detection"));
            let rpn_idx = headers.iter().position(|h| h == "rpn" || h.contains("priority"));
            let basis_idx = headers.iter().position(|h| h.contains("basis") || h.contains("derivation"));

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
                        !EXCLUDED_DIRS.iter().any(|&ex| name == ex)
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
                !EXCLUDED_DIRS.iter().any(|&ex| name == ex)
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
    let combined_lower = combined_content.to_lowercase();

    // Pillar 1: System Losses (L-1..N)
    let has_loss = combined_lower.contains("loss") || combined_lower.contains("losses");
    let has_l_id = has_stpa_id(&combined_lower, "l-");
    if !(has_loss && has_l_id) {
        errors.push("Pillar 1 violation: Missing System Losses ($L-1..N$) identification.".to_string());
    }

    // Pillar 2: System Hazards (H-1..N)
    let has_hazard = combined_lower.contains("hazard") || combined_lower.contains("hazards");
    let has_h_id = has_stpa_id(&combined_lower, "h-");
    if !(has_hazard && has_h_id) {
        errors.push("Pillar 2 violation: Missing System Hazards ($H-1..N$) identification.".to_string());
    }

    // Pillar 3: Control Structure Topology
    let has_control_struct = combined_lower.contains("control structure");
    let has_controller = combined_lower.contains("controller");
    let has_actuator = combined_lower.contains("actuator");
    if !(has_control_struct || (has_controller && has_actuator)) {
        errors.push("Pillar 3 violation: Missing Hierarchical Control Structure Topology.".to_string());
    }

    // Pillar 4: Unsafe Control Actions (UCA-1..N)
    let has_uca_kw = combined_lower.contains("unsafe control action")
        || combined_lower.contains("unsafe control actions");
    let has_uca_id = has_stpa_id(&combined_lower, "uca-");
    if !(has_uca_kw || has_uca_id) {
        errors.push("Pillar 4 violation: Missing Unsafe Control Actions ($UCA-1..N$).".to_string());
    }
    let missing_uca_cats = check_uca_categories(&combined_content);
    if !missing_uca_cats.is_empty() {
        errors.push(format!(
            "Pillar 4 violation: Missing UCA failure mode categories: {}.",
            missing_uca_cats.join(", ")
        ));
    }

    // Pillar 5: Loss Scenarios (LS-1..N)
    let has_ls_kw = combined_lower.contains("loss scenario")
        || combined_lower.contains("loss scenarios")
        || combined_lower.contains("causal scenario")
        || combined_lower.contains("causal scenarios");
    let has_ls_id = has_stpa_id(&combined_lower, "ls-");
    if !(has_ls_kw && has_ls_id) {
        errors.push("Pillar 5 violation: Missing Loss Scenarios ($LS-1..N$) & Causal Factors.".to_string());
    }

    // Pillar 6: Formal Safety Constraints (SC-1..N)
    let has_sc_kw = combined_lower.contains("safety constraint")
        || combined_lower.contains("safety constraints");
    let has_sc_id = has_stpa_id(&combined_lower, "sc-");
    if !(has_sc_kw && has_sc_id) {
        errors.push("Pillar 6 violation: Missing Formal Safety Constraints ($SC-1..N$).".to_string());
    }

    // Pillar 7: FMECA Criticality Matrix
    let has_fmeca = combined_lower.contains("fmeca") || combined_lower.contains("failure mode");
    if !has_fmeca {
        errors.push("Pillar 7 violation: Missing FMECA Criticality Matrix.".to_string());
    } else {
        let fmeca = parse_fmeca_table(&combined_content);
        if fmeca.total_rows == 0 {
            errors.push("Pillar 7 violation: FMECA Criticality Matrix contains 0 rows.".to_string());
        } else if fmeca.total_rows < 15 {
            // Log notice or check row count >= 15 as declared in Pillar 7 baseline
            errors.push(format!(
                "Pillar 7 violation: FMECA Criticality Matrix contains {} rows (minimum 15+ rows required).",
                fmeca.total_rows
            ));
        }

        let has_rpn = fmeca.has_rpn
            || combined_lower.contains("risk priority number")
            || matches_dimension_lower(&combined_lower, &["rpn"]);
        if !has_rpn {
            errors.push("Pillar 7 violation: FMECA table missing RPN (Risk Priority Number) calculation.".to_string());
        }

        let total_basis = fmeca.ssot_basis_count + fmeca.derived_basis_count;
        if fmeca.total_rows > 0 && total_basis == 0 {
            errors.push("Pillar 7 violation: FMECA Criticality Matrix missing explicit Derivation Basis classification ('SSOT' / 'Derived').".to_string());
        }

        if fmeca.total_rows > 0 {
            let missing_dims = check_failure_dimension_coverage(&combined_content, &fmeca);
            if !missing_dims.is_empty() {
                errors.push(format!(
                    "Pillar 7 violation: FMECA table missing coverage for universal failure dimension(s): {} (expected Interface (Γ), State (Φ), Action (Ω), Resource (Ψ)).",
                    missing_dims.join(", ")
                ));
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
    let has_pillar_8_kw = combined_lower.contains("pillar 8")
        || combined_lower.contains("regulatory objectives")
        || combined_lower.contains("sora sail");
    let has_heading_8 = combined_content.lines().any(|l| {
        let trimmed = l.trim();
        let after_hashes = trimmed.trim_start_matches('#');
        let hash_count = trimmed.len() - after_hashes.len();
        (2..=4).contains(&hash_count) && after_hashes.trim_start().starts_with("8.")
    });
    if !(has_pillar_8_kw || has_heading_8) {
        errors.push("Pillar 8 violation: Missing Pillar 8 (Domain Regulatory Objectives & Integrity Mapping / Regulatory Objectives & Risk Mitigations).".to_string());
    }

    // Commercial Toolchain Hooks (MATLAB / Simulink / Stateflow)
    let has_toolchain = combined_lower.contains("matlab")
        || combined_lower.contains("simulink")
        || combined_lower.contains("stateflow")
        || combined_lower.contains("embedded coder")
        || combined_lower.contains("sldv");
    if !has_toolchain {
        errors.push("Commercial Toolchain violation: Missing MATLAB / Simulink / Stateflow / Embedded Coder integration hooks.".to_string());
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
}
