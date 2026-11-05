#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Checks 10 and 12 (SysML v2 Compilation/Roundtrip and Closed-Loop GitHub SSOT Publishing) for `blueprint_gate`.
//! Realises: [REQ-BLUEPRINT-CHECK-10-SYSML-ROUNDTRIP], [REQ-BLUEPRINT-CHECK-12-PUBLISH-SSOT]

use std::path::{Path, PathBuf};

/// Maximum permitted SysML AST package and part nesting depth during bijective roundtrip normalization.
/// Realises: [REQ-BLUEPRINT-CHECK-10-AST-NORMALIZE]
pub(crate) const MAX_NORMALIZE_DEPTH: usize = 64;

/// Typed error returned when SysML AST nesting exceeds `MAX_NORMALIZE_DEPTH` during normalization.
/// Realises: [REQ-BLUEPRINT-CHECK-10-AST-NORMALIZE]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NormalizeDepthError {
    MaxDepthExceeded { depth: usize, max_depth: usize },
}

impl std::fmt::Display for NormalizeDepthError {
    /// Formats the normalization depth bound violation diagnostic for fail-closed reporting.
    /// Realises: [REQ-BLUEPRINT-CHECK-10-AST-NORMALIZE]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MaxDepthExceeded { depth, max_depth } => write!(
                f,
                "SysML AST normalization depth {depth} exceeds maximum bound {max_depth}"
            ),
        }
    }
}

impl std::error::Error for NormalizeDepthError {}

/// Validates iteratively that a `PartDef` subtree does not exceed `MAX_NORMALIZE_DEPTH` prior to mutation.
/// Realises: [REQ-BLUEPRINT-CHECK-10-PART-NORMALIZE]
fn validate_part_normalize_depth(
    part: &deap_core::sysml_ast::PartDef,
    depth: usize,
) -> Result<(), NormalizeDepthError> {
    let mut stack: Vec<(&deap_core::sysml_ast::PartDef, usize)> = vec![(part, depth)];
    while let Some((current, cur_depth)) = stack.pop() {
        if cur_depth > MAX_NORMALIZE_DEPTH {
            return Err(NormalizeDepthError::MaxDepthExceeded {
                depth: cur_depth,
                max_depth: MAX_NORMALIZE_DEPTH,
            });
        }
        let next_depth = cur_depth.saturating_add(1);
        for child in current.parts.iter().rev() {
            stack.push((child, next_depth));
        }
    }
    Ok(())
}

/// Validates iteratively that a `PackageDef` hierarchy and its `PartDef` subtrees do not exceed `MAX_NORMALIZE_DEPTH`.
/// Realises: [REQ-BLUEPRINT-CHECK-10-AST-NORMALIZE]
fn validate_package_normalize_depth(
    pkg: &deap_core::sysml_ast::PackageDef,
    depth: usize,
) -> Result<(), NormalizeDepthError> {
    let mut stack: Vec<(&deap_core::sysml_ast::PackageDef, usize)> = vec![(pkg, depth)];
    while let Some((current_pkg, cur_depth)) = stack.pop() {
        if cur_depth > MAX_NORMALIZE_DEPTH {
            return Err(NormalizeDepthError::MaxDepthExceeded {
                depth: cur_depth,
                max_depth: MAX_NORMALIZE_DEPTH,
            });
        }
        let next_depth = cur_depth.saturating_add(1);
        for part in &current_pkg.part_defs {
            validate_part_normalize_depth(part, next_depth)?;
        }
        for sub_pkg in current_pkg.packages.iter().rev() {
            stack.push((sub_pkg, next_depth));
        }
    }
    Ok(())
}

/// Promotes the first attribute or first port `doc` comment onto `part.doc` for deterministic roundtrip.
/// Realises: [REQ-BLUEPRINT-CHECK-10-PART-NORMALIZE]
fn promote_leading_member_doc_to_part(part: &mut deap_core::sysml_ast::PartDef) {
    if let Some(first_attr) = part.attributes.first_mut() {
        if let Some(attr_doc) = first_attr.doc.take() {
            match part.doc {
                Some(ref mut p_doc) => {
                    p_doc.push_str(" | ");
                    p_doc.push_str(&attr_doc);
                }
                None => part.doc = Some(attr_doc),
            }
        }
    } else if let Some(first_port) = part.ports.first_mut() {
        if let Some(port_doc) = first_port.doc.take() {
            match part.doc {
                Some(ref mut p_doc) => {
                    p_doc.push_str(" | ");
                    p_doc.push_str(&port_doc);
                }
                None => part.doc = Some(port_doc),
            }
        }
    }
}

/// Folds `protocol_family` and `electrical_attributes` into each port's `doc` comment.
/// Realises: [REQ-BLUEPRINT-CHECK-10-PART-NORMALIZE]
fn fold_part_port_metadata_into_docs(ports: &mut [deap_core::sysml_ast::PortDef]) {
    for port in ports {
        let proto = std::mem::take(&mut port.protocol_family);
        let clean_proto = proto.trim_matches(|c: char| c == '"' || c == '`');
        if !clean_proto.is_empty() {
            let suffix = format!(" [protocol: {clean_proto}]");
            match port.doc {
                Some(ref mut d) => d.push_str(&suffix),
                None => port.doc = Some(suffix.trim().to_string()),
            }
        }
        if !port.electrical_attributes.is_empty() {
            let proto_summary: Vec<String> = port
                .electrical_attributes
                .iter()
                .map(|(k, v)| {
                    let clean_v = v.trim_matches(|c: char| c == '"' || c == '`');
                    format!("{k}: {clean_v}")
                })
                .collect();
            let suffix = format!(" [{}]", proto_summary.join(", "));
            match port.doc {
                Some(ref mut d) => d.push_str(&suffix),
                None => port.doc = Some(suffix.trim().to_string()),
            }
            port.electrical_attributes.clear();
        }
        if let Some(first_flow) = port.item_flows.first_mut() {
            first_flow.doc = None;
        }
    }
}

/// Iteratively normalizes a `PartDef` subtree up to `MAX_NORMALIZE_DEPTH` for bijective roundtrip.
/// Realises: [REQ-BLUEPRINT-CHECK-10-PART-NORMALIZE]
pub(crate) fn normalize_part_for_bijective_roundtrip(
    part: &mut deap_core::sysml_ast::PartDef,
    depth: usize,
) -> Result<(), NormalizeDepthError> {
    validate_part_normalize_depth(part, depth)?;
    let mut stack: Vec<(&mut deap_core::sysml_ast::PartDef, usize)> = vec![(part, depth)];
    while let Some((current, cur_depth)) = stack.pop() {
        if cur_depth > MAX_NORMALIZE_DEPTH {
            return Err(NormalizeDepthError::MaxDepthExceeded {
                depth: cur_depth,
                max_depth: MAX_NORMALIZE_DEPTH,
            });
        }
        promote_leading_member_doc_to_part(current);
        fold_part_port_metadata_into_docs(&mut current.ports);
        let next_depth = cur_depth.saturating_add(1);
        for child in current.parts.iter_mut().rev() {
            stack.push((child, next_depth));
        }
    }
    Ok(())
}

/// Iteratively normalizes a `PackageDef` hierarchy up to `MAX_NORMALIZE_DEPTH` for bijective roundtrip.
/// Realises: [REQ-BLUEPRINT-CHECK-10-AST-NORMALIZE]
pub(crate) fn normalize_package_for_bijective_roundtrip(
    pkg: &mut deap_core::sysml_ast::PackageDef,
    depth: usize,
) -> Result<(), NormalizeDepthError> {
    validate_package_normalize_depth(pkg, depth)?;
    let mut stack: Vec<(&mut deap_core::sysml_ast::PackageDef, usize)> = vec![(pkg, depth)];
    while let Some((current_pkg, cur_depth)) = stack.pop() {
        if cur_depth > MAX_NORMALIZE_DEPTH {
            return Err(NormalizeDepthError::MaxDepthExceeded {
                depth: cur_depth,
                max_depth: MAX_NORMALIZE_DEPTH,
            });
        }
        let next_depth = cur_depth.saturating_add(1);
        for part in &mut current_pkg.part_defs {
            normalize_part_for_bijective_roundtrip(part, next_depth)?;
        }
        for port in &mut current_pkg.port_defs {
            port.protocol_family.clear();
            if !port.electrical_attributes.is_empty() {
                port.electrical_attributes.clear();
            }
        }
        for sub_pkg in current_pkg.packages.iter_mut().rev() {
            stack.push((sub_pkg, next_depth));
        }
    }
    Ok(())
}

/// Atomically writes `<stem>.sysml` and `<stem>.digest.json` and verifies non-empty files on disk.
/// Realises: [REQ-BLUEPRINT-CHECK-10-SYSML-ROUNDTRIP]
fn persist_and_verify_sysml_artifacts(
    blueprint_path: &Path,
    parsed_pkg: &deap_core::sysml_ast::PackageDef,
    roundtrip_sysml: &str,
) -> Result<(PathBuf, PathBuf), Vec<String>> {
    let parent_dir = blueprint_path.parent().unwrap_or_else(|| Path::new("."));
    let stem = blueprint_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("blueprint");
    let sysml_path = parent_dir.join(format!("{stem}.sysml"));
    let digest_path = parent_dir.join(format!("{stem}.digest.json"));

    let digest = compile_sysml::generate_digest(parsed_pkg, &sysml_path.to_string_lossy());
    compile_sysml::write_atomic(&sysml_path, roundtrip_sysml).map_err(|e| {
        vec![format!("Check 10 failed: atomic write of '{}' failed: {e}", sysml_path.display())]
    })?;
    compile_sysml::write_digest_atomic(&digest_path, &digest).map_err(|e| {
        vec![format!("Check 10 failed: atomic write of digest '{}' failed: {e}", digest_path.display())]
    })?;

    let sysml_meta = std::fs::metadata(&sysml_path).map_err(|e| {
        vec![format!("Check 10 failed: persisted SysML file '{}' missing on disk: {e}", sysml_path.display())]
    })?;
    if sysml_meta.len() == 0 {
        return Err(vec![format!("Check 10 failed: persisted SysML file '{}' is empty on disk", sysml_path.display())]);
    }
    let digest_meta = std::fs::metadata(&digest_path).map_err(|e| {
        vec![format!("Check 10 failed: persisted digest file '{}' missing on disk: {e}", digest_path.display())]
    })?;
    if digest_meta.len() == 0 {
        return Err(vec![format!("Check 10 failed: persisted digest file '{}' is empty on disk", digest_path.display())]);
    }
    Ok((sysml_path, digest_path))
}

/// Check 10 (Deterministic In-Repo SysML v2 Compilation & Bijective Roundtrip Gate).
/// Realises: [REQ-BLUEPRINT-CHECK-10-SYSML-ROUNDTRIP]
pub fn check_10_compile_and_persist_sysml(
    blueprint_path: &Path,
    content: &str,
) -> Result<(PathBuf, PathBuf), Vec<String>> {
    let bp_display = blueprint_path.to_string_lossy();
    let mut pkg = ingest_sysml::translate_markdown(content, &bp_display)
        .map_err(|e| vec![format!("Check 10 failed: Markdown-to-SysML translation error: {e}")])?;
    normalize_package_for_bijective_roundtrip(&mut pkg, 0)
        .map_err(|e| vec![format!("Check 10 failed: {e}")])?;

    let sysml_text = compile_sysml::to_sysml(&pkg);
    if sysml_text.trim().is_empty() {
        return Err(vec!["Check 10 failed: serialized SysML v2 output is empty".to_string()]);
    }

    let mut missing_constructs = Vec::new();
    for required_token in ["package ", "requirement def ", "part def ", "constraint def "] {
        if !sysml_text.contains(required_token) {
            missing_constructs.push(format!(
                "Check 10 failed: compiled SysML v2 output is missing required construct '{required_token}'"
            ));
        }
    }
    if !missing_constructs.is_empty() {
        return Err(missing_constructs);
    }

    let parsed_pkg = compile_sysml::parse_sysml(&sysml_text)
        .map_err(|e| vec![format!("Check 10 failed: compiled SysML v2 failed to re-parse: {e}")])?;

    let mut validator = compile_sysml::SemanticValidator::new(&parsed_pkg);
    let semantic_errors: Vec<String> = validator
        .validate()
        .iter()
        .filter(|d| d.severity == compile_sysml::DiagnosticSeverity::Error)
        .map(|d| format!("Check 10 failed: SysML semantic validation error: {d:?}"))
        .collect();
    if !semantic_errors.is_empty() {
        return Err(semantic_errors);
    }

    let roundtrip_sysml = compile_sysml::to_sysml(&parsed_pkg);
    if roundtrip_sysml != sysml_text {
        return Err(vec![
            "Check 10 failed: bijective SysML v2 roundtrip mismatch (re-serialized AST differs byte-for-byte from initial serialization)"
                .to_string(),
        ]);
    }

    persist_and_verify_sysml_artifacts(blueprint_path, &parsed_pkg, &roundtrip_sysml)
}
