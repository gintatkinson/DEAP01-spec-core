//! Check 17: Safety Integrity, STPA 8-Pillars, and Regulatory Objectives Gate.

use deap_core::rules::EXCLUDED_DIRS;
use deap_core::workspace::is_upstream_compiler;
use regex::Regex;
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

/// Parse FMECA markdown tables from content.
pub fn parse_fmeca_table(content: &str) -> FmecaData {
    let mut data = FmecaData::default();
    let lines: Vec<&str> = content.lines().collect();
    let n = lines.len();
    let mut i = 0;

    let fmeca_header_re = Regex::new(r"(?i)failure\s+mode|fmeca").unwrap();

    while i < n {
        let line = lines[i].trim();
        if line.starts_with('|') && fmeca_header_re.is_match(line) {
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

/// Verify that failure modes across the FMECA table span the 4 universal failure dimensions:
/// Interface (Γ), State (Φ), Action (Ω), Resource (Ψ).
pub fn check_failure_dimension_coverage(content: &str, fmeca: &FmecaData) -> Vec<String> {
    let interface_re = Regex::new(r"(?i)\b(?:Interface|Port|Bus|Signal|Protocol|Packet|Message|Frame|Channel|Link|CRC|Timeout|IO|Input|Output|Data|Transceiver|Receiver|Uplink|Downlink|Telemetry|Transients?|Γ|\\Gamma)\b").unwrap();
    let state_re = Regex::new(r"(?i)\b(?:State|Mode|Transition|Deadlock|Latch|Phase|Statechart|FSM|Sync|Synchronization|Desync|Drift|Stuck|Uninitialized|Freeze|Lockup|Trip|Abort|Corruption|Disagreement|Φ|\\Phi)\b").unwrap();
    let action_re = Regex::new(r"(?i)\b(?:Action|Command|Execution|Operation|Control|Timing|Deadline|Compute|Calculation|Process|Logic|Omission|Commission|Latency|Jitter|Delay|Rate|Clamping|Limiter|Saturation|Step|Overshoot|Schedule|Task|Authority|Miss|Ω|\\Omega)\b").unwrap();
    let resource_re = Regex::new(r"(?i)\b(?:Resource|Memory|CPU|Buffer|Power|Energy|Battery|Thermal|Heat|Overheat|Overload|Bandwidth|Storage|Capacity|Stack|Heap|Overflow|Underflow|Brownout|Voltage|Current|Load|Fault|Short|Sag|Circuit|Crowbar|Degradation|Flash|RAM|Supply|Undervoltage|Overvoltage|Seizure|Windings?|Wiper|Hardware|Bearing|Dielectric|Squib|Fuse|Fusing|Ψ|\\Psi)\b").unwrap();

    let mut found_interface = false;
    let mut found_state = false;
    let mut found_action = false;
    let mut found_resource = false;

    // Check rows first
    for row in &fmeca.rows {
        let combined = format!("{} {} {}", row.failure_mode, row.component, row.basis.as_deref().unwrap_or(""));
        if interface_re.is_match(&combined) {
            found_interface = true;
        }
        if state_re.is_match(&combined) {
            found_state = true;
        }
        if action_re.is_match(&combined) {
            found_action = true;
        }
        if resource_re.is_match(&combined) {
            found_resource = true;
        }
    }

    // Fallback: check whole content if FMECA is described in text
    if !found_interface && interface_re.is_match(content) {
        found_interface = true;
    }
    if !found_state && state_re.is_match(content) {
        found_state = true;
    }
    if !found_action && action_re.is_match(content) {
        found_action = true;
    }
    if !found_resource && resource_re.is_match(content) {
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

    // 1. Not providing causes hazard
    let re_not_providing = Regex::new(r"(?i)\b(?:not\s+provid(?:ing|ed)|omission)\b").unwrap();
    if !re_not_providing.is_match(content) {
        missing.push("1. Not providing causes hazard".to_string());
    }

    // 2. Providing causes hazard
    let re_providing_explicit = Regex::new(r"(?i)\b(?:providing\s+(?:causes|incorrectly)|commission)\b").unwrap();
    let re_providing_any = Regex::new(r"(?i)\bproviding\b").unwrap();
    let re_providing_too = Regex::new(r"(?i)\bproviding\s+too\b").unwrap();
    let has_providing = re_providing_explicit.is_match(content)
        || (re_providing_any.is_match(content) && !re_providing_too.is_match(content));
    if !has_providing {
        missing.push("2. Providing causes hazard".to_string());
    }

    // 3. Providing too early, too late, or out of order
    let re_timing = Regex::new(r"(?i)\b(?:too\s+early|too\s+late|out\s+of\s+order|timing|early/late)\b").unwrap();
    if !re_timing.is_match(content) {
        missing.push("3. Providing too early, too late, or out of order".to_string());
    }

    // 4. Stopped too soon or applied too long
    let re_duration = Regex::new(r"(?i)\b(?:stopped\s+too\s+soon|applied\s+too\s+long|duration|stopped\s+early)\b").unwrap();
    if !re_duration.is_match(content) {
        missing.push("4. Stopped too soon or applied too long".to_string());
    }

    missing
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

    // Pillar 1: System Losses (L-1..N)
    let re_loss = Regex::new(r"(?i)loss(?:es)?").unwrap();
    let re_l_id = Regex::new(r"(?i)\bL-\d+\b|\$L-\d+").unwrap();
    if !(re_loss.is_match(&combined_content) && re_l_id.is_match(&combined_content)) {
        errors.push("Pillar 1 violation: Missing System Losses ($L-1..N$) identification.".to_string());
    }

    // Pillar 2: System Hazards (H-1..N)
    let re_hazard = Regex::new(r"(?i)hazard(?:s)?").unwrap();
    let re_h_id = Regex::new(r"(?i)\bH-\d+\b|\$H-\d+").unwrap();
    if !(re_hazard.is_match(&combined_content) && re_h_id.is_match(&combined_content)) {
        errors.push("Pillar 2 violation: Missing System Hazards ($H-1..N$) identification.".to_string());
    }

    // Pillar 3: Control Structure Topology
    let re_control_struct = Regex::new(r"(?i)control\s+structure").unwrap();
    let re_controller = Regex::new(r"(?i)controller").unwrap();
    let re_actuator = Regex::new(r"(?i)actuator").unwrap();
    if !(re_control_struct.is_match(&combined_content)
        || (re_controller.is_match(&combined_content) && re_actuator.is_match(&combined_content)))
    {
        errors.push("Pillar 3 violation: Missing Hierarchical Control Structure Topology.".to_string());
    }

    // Pillar 4: Unsafe Control Actions (UCA-1..N)
    let re_uca = Regex::new(r"(?i)unsafe\s+control\s+actions?|\bUCA-\d+\b").unwrap();
    if !re_uca.is_match(&combined_content) {
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
    let re_ls = Regex::new(r"(?i)loss\s+scenarios?|causal\s+scenarios?").unwrap();
    let re_ls_id = Regex::new(r"(?i)\bLS-\d+\b|\$LS-\d+").unwrap();
    if !(re_ls.is_match(&combined_content) && re_ls_id.is_match(&combined_content)) {
        errors.push("Pillar 5 violation: Missing Loss Scenarios ($LS-1..N$) & Causal Factors.".to_string());
    }

    // Pillar 6: Formal Safety Constraints (SC-1..N)
    let re_sc = Regex::new(r"(?i)safety\s+constraints?").unwrap();
    let re_sc_id = Regex::new(r"(?i)\bSC-\d+\b|\$SC-\d+").unwrap();
    if !(re_sc.is_match(&combined_content) && re_sc_id.is_match(&combined_content)) {
        errors.push("Pillar 6 violation: Missing Formal Safety Constraints ($SC-1..N$).".to_string());
    }

    // Pillar 7: FMECA Criticality Matrix
    let re_fmeca = Regex::new(r"(?i)fmeca|failure\s+mode").unwrap();
    if !re_fmeca.is_match(&combined_content) {
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

        if !fmeca.has_rpn && !Regex::new(r"(?i)\bRPN\b|Risk\s+Priority\s+Number").unwrap().is_match(&combined_content) {
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
    let re_pillar_8 = Regex::new(r"(?i)pillar\s+8|regulatory\s+objectives|sora\s+sail").unwrap();
    if !re_pillar_8.is_match(&combined_content) && !Regex::new(r"(?m)^#{2,4}\s*8\.\s+").unwrap().is_match(&combined_content) {
        errors.push("Pillar 8 violation: Missing Pillar 8 (Domain Regulatory Objectives & Integrity Mapping / Regulatory Objectives & Risk Mitigations).".to_string());
    }

    // Commercial Toolchain Hooks (MATLAB / Simulink / Stateflow)
    let re_toolchain = Regex::new(r"(?i)matlab|simulink|stateflow|embedded\s+coder|sldv").unwrap();
    if !re_toolchain.is_match(&combined_content) {
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
