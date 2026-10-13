//! Defect dossier schema validator.
//!
//! Validates defect reports against the 7-section Adversarial Audit schema:
//! 1. Sections ## 1. through ## 7.
//! 2. ## Audit Source header
//! 3. SEVERITY line (Critical | Important | Suggestion | Nitpick)
//! 4. FILE_LOCATION line
//! 5. Balanced code fence count (```)

use std::path::PathBuf;

#[derive(clap::Args, Debug, Clone)]
pub struct DefectDossierArgs {
    /// Path to markdown defect dossier body file
    #[arg(long = "body-file")]
    pub body_file: Option<PathBuf>,

    /// Title of the defect issue
    #[arg(long = "title")]
    pub title: Option<String>,

    /// Repository identifier (e.g. gintatkinson/DEAP01-spec-core)
    #[arg(long = "repo", default_value = "gintatkinson/DEAP01-spec-core")]
    pub repo: String,

    /// Label to apply
    #[arg(long = "label")]
    pub label: Option<String>,

    /// Issue tracker provider (github or gitlab)
    #[arg(long = "provider", default_value = "github")]
    pub provider: String,

    /// Validate defect dossier schema without submitting
    #[arg(long = "dry-run", alias = "validate-only")]
    pub dry_run: bool,

    /// Positional body file path
    #[arg(value_name = "BODY_FILE")]
    pub target: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefectSchemaValidation {
    pub severity: String,
    pub file_location: String,
    pub suggested_label: String,
}

pub fn validate_defect_dossier(content: &str) -> Result<DefectSchemaValidation, Vec<String>> {
    let mut errors = Vec::new();

    // 1. Check sections ## 1. through ## 7.
    for i in 1..=7 {
        let prefix = format!("{}.", i);
        let mut found = false;
        for line in content.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("##") {
                let rest = trimmed[2..].trim_start();
                if rest.starts_with(&prefix) {
                    found = true;
                    break;
                }
            }
        }
        if !found {
            errors.push(format!("Missing mandatory section header '## {}.'", i));
        }
    }

    // 2. Check ## Audit Source header
    let mut found_audit_source = false;
    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("##") {
            let rest = trimmed[2..].trim_start();
            if rest.to_ascii_lowercase().starts_with("audit source") {
                found_audit_source = true;
                break;
            }
        }
    }
    if !found_audit_source {
        errors.push("Missing mandatory '## Audit Source' section header.".to_string());
    }

    // 3. Check SEVERITY line
    let mut severity_opt = None;
    for line in content.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("SEVERITY:") {
            let val = rest.trim();
            for valid in &["Critical", "Important", "Suggestion", "Nitpick"] {
                if val.starts_with(valid) {
                    severity_opt = Some((*valid).to_string());
                    break;
                }
            }
            if severity_opt.is_some() {
                break;
            }
        }
    }
    let severity = match severity_opt {
        Some(s) => s,
        None => {
            errors.push(
                "Missing or invalid 'SEVERITY:' line (must be Critical, Important, Suggestion, or Nitpick)."
                    .to_string(),
            );
            String::new()
        }
    };

    // 4. Check FILE_LOCATION line
    let mut location_opt = None;
    for line in content.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("FILE_LOCATION:") {
            let val = rest.trim();
            if !val.is_empty() {
                location_opt = Some(val.to_string());
                break;
            }
        }
    }
    let file_location = match location_opt {
        Some(loc) => loc,
        None => {
            errors.push("Missing or empty mandatory 'FILE_LOCATION:' line.".to_string());
            String::new()
        }
    };

    // 5. Check balanced code blocks
    let fence_count = content.lines().filter(|l| l.contains("```")).count();
    if fence_count % 2 != 0 {
        errors.push(format!(
            "Unbalanced code blocks: found odd number ({}) of ``` fences.",
            fence_count
        ));
    }

    if !errors.is_empty() {
        Err(errors)
    } else {
        let suggested_label = match severity.as_str() {
            "Critical" | "Important" => "bug".to_string(),
            _ => "enhancement".to_string(),
        };

        Ok(DefectSchemaValidation {
            severity,
            file_location,
            suggested_label,
        })
    }
}

pub fn run_check_defect(args: &DefectDossierArgs) -> Result<(), i32> {
    let body_path = args
        .body_file
        .as_ref()
        .or(args.target.as_ref());

    let body_file = match body_path {
        Some(p) => p,
        None => {
            eprintln!("Error: --body-file is required and must exist.");
            return Err(1);
        }
    };

    if !body_file.is_file() {
        eprintln!("Error: Defect body file does not exist: {}", body_file.display());
        return Err(1);
    }

    let content = match std::fs::read_to_string(body_file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading body file {}: {}", body_file.display(), e);
            return Err(1);
        }
    };

    match validate_defect_dossier(&content) {
        Ok(validation) => {
            let label = args.label.clone().unwrap_or_else(|| {
                if args.provider == "gitlab" {
                    if validation.suggested_label == "bug" {
                        "type::bug".to_string()
                    } else {
                        "type::feature".to_string()
                    }
                } else {
                    validation.suggested_label.clone()
                }
            });

            let file_name = body_file
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "dossier.md".to_string());
            let title = args
                .title
                .clone()
                .unwrap_or_else(|| format!("[AUDIT] {}", file_name));

            println!("Defect dossier validation PASSED for {}.", body_file.display());
            println!("[VALIDATION] Target payload:");
            println!("  Provider: {}", args.provider);
            println!("  Repo: {}", args.repo);
            println!("  Title: {}", title);
            println!("  Severity: {}", validation.severity);
            println!("  File Location: {}", validation.file_location);
            println!("  Label: {}", label);
            println!("  Body file: {}", body_file.display());

            Ok(())
        }
        Err(errors) => {
            eprintln!(
                "Defect dossier validation FAILED for {} with {} violation(s):",
                body_file.display(),
                errors.len()
            );
            for err in errors {
                eprintln!("  - {}", err);
            }
            Err(1)
        }
    }
}
