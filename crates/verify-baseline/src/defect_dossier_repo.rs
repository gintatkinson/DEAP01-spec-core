//! Title validation, repository slug resolution, and scoped validation orchestration for defect dossiers.
//! Realises: [REQ-VERIFY-BASELINE-DEFECT-DOSSIER], [DO-330-TQL-4-DEFECT-GATE]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::defect_dossier::{
    domain_cleanliness_gate, extract_file_basename, is_unfilled_placeholder,
    validate_section_6_code_and_target_grounding, DefectSchemaValidation, DossierSection,
};

/// Validates that a defect issue title conforms to Step D Check 11 (`[AUDIT] [<file.ext>]: <description>`).
/// Realises: [REQ-DEFECT-DOSSIER-TITLE-VALIDATOR]
pub(crate) fn validate_defect_title_impl(title: &str, file_location: &str) -> Result<(), String> {
    let trimmed = title.trim();
    if trimmed.contains('\u{2014}') {
        return Err(
            "Defect title contains forbidden Unicode em dash (\\u2014); use ASCII '--' instead."
                .to_string(),
        );
    }
    let Some(after_prefix) = trimmed.strip_prefix("[AUDIT] [") else {
        return Err(format!(
            "Invalid title '{}': must match '[AUDIT] [<file.ext>]: <description>' (Check 11).",
            trimmed
        ));
    };
    let Some((file_part, desc_part)) = after_prefix.split_once("]: ") else {
        return Err(format!(
            "Invalid title '{}': missing ']: ' separator in '[AUDIT] [<file.ext>]: <description>' (Check 11).",
            trimmed
        ));
    };
    let file_token = file_part.trim();
    let desc_token = desc_part.trim();
    if file_token.is_empty() || is_unfilled_placeholder(file_token) {
        return Err(format!(
            "Invalid title '{}': '<file.ext>' bracket token must be non-empty (Check 11).",
            trimmed
        ));
    }
    if desc_token.is_empty() || is_unfilled_placeholder(desc_token) {
        return Err(format!(
            "Invalid title '{}': '<description>' after ']: ' must be non-empty and not a placeholder (Check 11).",
            trimmed
        ));
    }
    let expected_basename = extract_file_basename(file_location);
    if !expected_basename.is_empty() && file_token != expected_basename {
        return Err(format!(
            "Title filename mismatch (Check 11): title specifies '[{}]' but FILE_LOCATION basename is '{}'.",
            file_token, expected_basename
        ));
    }
    Ok(())
}

/// Validates a markdown defect dossier with explicit target repository and workspace root scoping.
/// Realises: [REQ-VALIDATE-DEFECT-DOSSIER-SCOPED]
pub(crate) fn validate_defect_dossier_scoped_impl(
    content: &str,
    target_repo: Option<&str>,
    workspace_root: Option<&Path>,
) -> Result<DefectSchemaValidation, Vec<String>> {
    let mut errors = Vec::new();
    let lines: Vec<String> = content.lines().map(str::to_string).collect();
    let is_upstream = match target_repo {
        Some(repo) => {
            let repo_clean = repo.trim().to_ascii_lowercase();
            repo_clean.contains("deap01-spec-core")
                || repo_clean == "upstream"
                || repo_clean == "spec-core"
        }
        None => crate::defect_dossier_grounding::detect_is_upstream_compiler(workspace_root),
    };
    match lines.iter().map(|l| l.trim()).find(|l| !l.is_empty()) {
        None => {
            errors.push("Defect dossier body is empty.".to_string());
            return Err(errors);
        }
        Some(first_line) if first_line.starts_with("# ") => {
            errors.push(format!("Forbidden leading H1 header '{}' on first line: dossier must begin directly with '## 1. Context and References'.", first_line));
        }
        Some(first_line) if first_line != DossierSection::ContextAndReferences.header() => {
            errors.push(format!("First non-empty line must be '{}' (found '{}').", DossierSection::ContextAndReferences.header(), first_line));
        }
        Some(_) => {}
    }

    let (section_indices, audit_source_idx) =
        crate::defect_dossier_grounding::locate_and_verify_dossier_sections(&lines, &mut errors);
    let get_section_slice = |target_sec: DossierSection| -> &[String] {
        let Some((_, start_idx)) = section_indices.iter().find(|(s, _)| *s == target_sec) else {
            return &[];
        };
        let mut end_idx = lines.len();
        for &(_, other_idx) in &section_indices {
            if other_idx > *start_idx && other_idx < end_idx {
                end_idx = other_idx;
            }
        }
        if let Some(a_idx) = audit_source_idx {
            if a_idx > *start_idx && a_idx < end_idx {
                end_idx = a_idx;
            }
        }
        lines.get(start_idx.saturating_add(1)..end_idx).unwrap_or(&[])
    };

    let (parsed_severity, severity_str, file_location) =
        crate::defect_dossier_grounding::extract_severity_and_file_location(&lines, &mut errors);
    let (symptom, test_target) = crate::defect_dossier_sections::validate_section_1_context(
        get_section_slice(DossierSection::ContextAndReferences), &mut errors,
    );
    crate::defect_dossier_sections::validate_section_2_whys(
        get_section_slice(DossierSection::RootCauseAnalysis), &mut errors,
    );
    crate::defect_dossier_sections::validate_section_4_uml(
        get_section_slice(DossierSection::UmlDiagrams), content, parsed_severity, &mut errors,
    );
    let sec6_slice = get_section_slice(DossierSection::ProposedCorrection);
    crate::defect_dossier_sections::validate_fences_ascii_art_and_em_dash(sec6_slice, content, &mut errors);
    validate_section_6_code_and_target_grounding(sec6_slice, &test_target, is_upstream, &mut errors);
    let _ = domain_cleanliness_gate;

    if !errors.is_empty() {
        Err(errors)
    } else {
        Ok(DefectSchemaValidation {
            severity: severity_str,
            file_location,
            suggested_label: parsed_severity.map_or_else(|| "enhancement".to_string(), |s| s.github_label().to_string()),
            symptom,
            test_target,
        })
    }
}

/// Parses a `<owner>/<repo>` repository slug from a Git remote URL or explicit slug string.
/// Realises: [REQ-DEFECT-DOSSIER-PARSE-REPO-SLUG]
pub(crate) fn parse_github_repo_slug_impl(remote_url: &str) -> Result<String, String> {
    let trimmed = remote_url.trim();
    if trimmed.is_empty() {
        return Err("Repository remote URL or slug is empty.".to_string());
    }
    let without_git_suffix = trimmed
        .trim_end_matches('/')
        .strip_suffix(".git")
        .unwrap_or_else(|| trimmed.trim_end_matches('/'))
        .trim_end_matches('/');
    let slug_candidate = if let Some(after_scheme) = without_git_suffix
        .strip_prefix("https://")
        .or_else(|| without_git_suffix.strip_prefix("http://"))
        .or_else(|| without_git_suffix.strip_prefix("ssh://"))
    {
        after_scheme
            .split_once('/')
            .map(|(_, path)| path.trim_start_matches('/'))
            .ok_or_else(|| {
                format!(
                    "Invalid repository URL '{}': missing '<owner>/<repo>' path after host.",
                    trimmed
                )
            })?
    } else if let Some((_, path)) = without_git_suffix.split_once(':') {
        path.trim_start_matches('/')
    } else {
        without_git_suffix
    };
    let Some((owner, repo)) = slug_candidate.split_once('/') else {
        return Err(format!(
            "Invalid repository identifier '{}': expected '<owner>/<repo>' slug.",
            trimmed
        ));
    };
    let (owner_clean, repo_clean) = (owner.trim(), repo.trim().trim_end_matches('/'));
    if owner_clean.is_empty()
        || repo_clean.is_empty()
        || repo_clean.contains('/')
        || owner_clean.contains(':')
        || repo_clean.contains(':')
        || owner_clean.chars().any(char::is_whitespace)
        || repo_clean.chars().any(char::is_whitespace)
    {
        return Err(format!(
            "Invalid repository slug '{}' derived from '{}': expected '<owner>/<repo>'.",
            slug_candidate, trimmed
        ));
    }
    Ok(format!("{}/{}", owner_clean, repo_clean))
}

/// Resolves the target GitHub repository `<owner>/<repo>` slug from an explicit `--repo` argument
/// or by querying `git remote get-url origin` in the dossier's working directory.
/// Realises: [REQ-DEFECT-DOSSIER-RESOLVE-TARGET-REPO]
pub(crate) fn resolve_target_repo_impl(
    explicit_repo: Option<&str>,
    work_dir: &Path,
) -> Result<String, String> {
    if let Some(raw) = explicit_repo {
        if !raw.trim().is_empty() {
            return parse_github_repo_slug_impl(raw.trim());
        }
    }
    let effective_dir = if work_dir.as_os_str().is_empty() {
        Path::new(".")
    } else if work_dir.is_dir() {
        work_dir
    } else {
        work_dir
            .ancestors()
            .find(|p| !p.as_os_str().is_empty() && p.is_dir())
            .unwrap_or_else(|| Path::new("."))
    };
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(effective_dir)
        .output()
        .map_err(|e| {
            format!(
                "--repo was omitted and failed to execute 'git remote get-url origin' in '{}': {}",
                effective_dir.display(),
                e
            )
        })?;
    if !output.status.success() {
        return Err(format!(
            "--repo was omitted and 'git remote get-url origin' could not be resolved in '{}': {}",
            effective_dir.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_github_repo_slug_impl(stdout.trim()).map_err(|e| {
        format!(
            "--repo was omitted and 'git remote get-url origin' returned an unparseable URL '{}': {}",
            stdout.trim(),
            e
        )
    })
}

/// Ensures the validated defect dossier is persisted inside `.pipeline/defects/<filename>.md`.
/// Realises: [REQ-DEFECT-DOSSIER-DUAL-PERSISTENCE]
pub(crate) fn ensure_persisted_in_pipeline_defects_impl(
    body_file: &Path,
    content: &str,
) -> Result<PathBuf, String> {
    let normalized = body_file.to_string_lossy().replace('\\', "/");
    if normalized.starts_with(".pipeline/defects/") || normalized.contains("/.pipeline/defects/") {
        return Ok(body_file.to_path_buf());
    }
    let file_name = body_file
        .file_name()
        .map_or_else(|| "defect_dossier.md".to_string(), |n| n.to_string_lossy().to_string());
    let defects_dir = PathBuf::from(".pipeline").join("defects");
    fs::create_dir_all(&defects_dir)
        .map_err(|e| format!("Failed to create '{}': {}", defects_dir.display(), e))?;
    let dest_path = defects_dir.join(file_name);
    crate::write_atomic_file(&dest_path, content).map_err(|e| {
        format!(
            "Failed to persist defect dossier to '{}': {}",
            dest_path.display(),
            e
        )
    })?;
    Ok(dest_path)
}
