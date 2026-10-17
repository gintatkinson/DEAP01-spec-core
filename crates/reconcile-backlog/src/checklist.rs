//! Markdown checklist parsing, cross-linking, and reconciliation.
//!
//! Realises: [Zero-Placeholder Invariant, Strict Bidirectional Checklist Sync]

/// Represents a parsed checklist item.
#[derive(Debug, Clone, PartialEq)]
pub struct ChecklistItem {
    pub line_number: usize,
    pub checked: bool,
    pub issue_number: Option<u64>,
    pub text: String,
    pub full_line: String,
}

/// Internal helper representing a parsed checklist line.
struct ParsedCheckline {
    checked: bool,
    issue_number: Option<u64>,
    text: String,
    mark_pos: usize,
}

/// Parses a line into checklist components if it matches `- [ ]` or `- [x]`.
fn parse_checklist_line(line: &str) -> Option<ParsedCheckline> {
    let trimmed = line.trim_start();
    let indent_len = line.len() - trimmed.len();

    // Check for checkbox prefix with str::strip_prefix
    let (checked, after_box) = if let Some(rest) = trimmed.strip_prefix("- [ ] ") {
        (false, rest)
    } else if let Some(rest) = trimmed.strip_prefix("- [x] ").or_else(|| trimmed.strip_prefix("- [X] ")) {
        (true, rest)
    } else if let Some(rest) = trimmed.strip_prefix("- [ ]") {
        (false, rest)
    } else if let Some(rest) = trimmed.strip_prefix("- [x]").or_else(|| trimmed.strip_prefix("- [X]")) {
        (true, rest)
    } else {
        // Fallback for optional whitespace between '-' and '['
        let after_dash = trimmed.strip_prefix('-')?;
        let after_dash_trimmed = after_dash.trim_start();
        if let Some(rest) = after_dash_trimmed.strip_prefix("[ ] ") {
            (false, rest)
        } else if let Some(rest) = after_dash_trimmed.strip_prefix("[x] ").or_else(|| after_dash_trimmed.strip_prefix("[X] ")) {
            (true, rest)
        } else if let Some(rest) = after_dash_trimmed.strip_prefix("[ ]") {
            (false, rest)
        } else {
            let rest = after_dash_trimmed
                .strip_prefix("[x]")
                .or_else(|| after_dash_trimmed.strip_prefix("[X]"))?;
            (true, rest)
        }
    };

    // Calculate exact byte position of the checkbox mark character in `line`
    let bracket_offset = trimmed.find('[')?;
    let mark_pos = indent_len + bracket_offset + 1;

    // Parse issue number (#123) and clean title from `after_box`
    let (issue_number, text) = extract_checklist_title_and_issue(after_box);

    Some(ParsedCheckline {
        checked,
        issue_number,
        text,
        mark_pos,
    })
}

/// Helper function to clean markdown formatting, bold prefixes, and annotations from specification titles.
fn clean_title_formatting(raw: &str) -> String {
    let mut t = raw.trim();

    // 1. Strip trailing parenthetical annotations like `(clauses ...)`, `(refs #...)`, `(#...)`, `(justification ...)`
    if t.ends_with(')') {
        if let Some(open_paren) = t.rfind('(') {
            let inside = t[open_paren + 1..t.len() - 1].trim();
            if inside.starts_with('#')
                || inside.starts_with("refs")
                || inside.starts_with("clauses")
                || inside.starts_with("justification")
                || inside.contains('#')
            {
                t = t[..open_paren].trim();
            }
        }
    }

    // 2. If there is a bold prefix like `**Prefix:** Title` or `**Prefix**: Title`, extract the title after bold prefix
    if t.starts_with("**") {
        if let Some(close_bold) = t[2..].find("**") {
            let inside_bold = &t[2..2 + close_bold];
            let after_bold = t[2 + close_bold + 2..].trim();
            if inside_bold.ends_with(':') || inside_bold.ends_with('-') {
                if !after_bold.is_empty() {
                    t = after_bold;
                }
            } else if let Some(rest) = after_bold.strip_prefix(':').or_else(|| after_bold.strip_prefix('-')) {
                let candidate = rest.trim();
                if !candidate.is_empty() {
                    t = candidate;
                }
            }
        }
    }

    // Strip enclosing bold/italic formatting
    while (t.starts_with("**") && t.ends_with("**")) && t.len() >= 4 {
        t = t[2..t.len() - 2].trim();
    }
    while (t.starts_with('*') && t.ends_with('*')) && t.len() >= 2 {
        t = t[1..t.len() - 1].trim();
    }
    while (t.starts_with("__") && t.ends_with("__")) && t.len() >= 4 {
        t = t[2..t.len() - 2].trim();
    }
    while (t.starts_with('_') && t.ends_with('_')) && t.len() >= 2 {
        t = t[1..t.len() - 1].trim();
    }

    // Strip trailing separator punctuation
    t = t.trim_end_matches(':').trim_end_matches('-').trim();

    t.to_string()
}

/// Extracts the human-readable specification title and any issue reference number (`#<digits>`)
/// from checklist text, stripping Markdown links, bold markers, and trailing annotations.
pub fn extract_checklist_title_and_issue(raw_text: &str) -> (Option<u64>, String) {
    let trimmed = raw_text.trim();

    // 1. Check for prefix issue: #<digits>
    let mut found_issue = None;
    let mut title_candidate = trimmed;

    if let Some(after_hash) = trimmed.strip_prefix('#') {
        let digits_len = after_hash.chars().take_while(|c| c.is_ascii_digit()).count();
        if digits_len > 0 {
            if let Ok(num) = after_hash[..digits_len].parse::<u64>() {
                found_issue = Some(num);
                let after_digits = &after_hash[digits_len..];
                let after_digits_trimmed = after_digits.trim_start();
                let after_sep = if let Some(rest) = after_digits_trimmed.strip_prefix('-') {
                    rest
                } else if let Some(rest) = after_digits_trimmed.strip_prefix(':') {
                    rest
                } else {
                    after_digits_trimmed
                };
                title_candidate = after_sep.trim_start();
            }
        }
    }

    // 2. If no prefix issue, scan raw_text for inline #<digits>
    if found_issue.is_none() {
        let bytes = trimmed.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'#' {
                let start = i + 1;
                let mut end = start;
                while end < bytes.len() && bytes[end].is_ascii_digit() {
                    end += 1;
                }
                if end > start {
                    if let Ok(num) = trimmed[start..end].parse::<u64>() {
                        found_issue = Some(num);
                        break;
                    }
                }
            }
            i += 1;
        }
    }

    // 3. Extract title:
    // If text has Markdown link syntax [...](...), extract the text between outer [ and ],
    // ignoring the (...) URL target and any trailing annotations.
    let title = if let Some(open_b) = title_candidate.find('[') {
        if let Some(sep) = title_candidate[open_b..].find("](") {
            let inside_link = &title_candidate[open_b + 1..open_b + sep];
            clean_title_formatting(inside_link)
        } else {
            clean_title_formatting(title_candidate)
        }
    } else {
        clean_title_formatting(title_candidate)
    };

    (found_issue, title)
}

/// Identifies if a string contains prohibited unpopulated fallback placeholders.
pub fn contains_unpopulated_placeholders(text: &str) -> bool {
    text.to_ascii_lowercase().contains("to be populated")
}

/// Replaces prohibited placeholder blocks with compliant allocated notes or removes them.
pub fn clean_unpopulated_placeholders(content: &str) -> String {
    clean_unpopulated_placeholders_counted(content).0
}

/// Replaces prohibited placeholder blocks with compliant allocated notes and returns the count of purged blocks.
pub fn clean_unpopulated_placeholders_counted(content: &str) -> (String, usize) {
    let target = "to be populated";
    let replacement = "- None allocated (structural subsystem component)";
    let mut result = String::with_capacity(content.len());
    let mut rest = content;
    let mut purged_count = 0;

    while let Some(pos) = rest.to_ascii_lowercase().find(target) {
        // Forward matching: check for optional trailing phase specifier e.g. " after phase [digits/alnum]"
        let mut cur_offset = target.len();
        let after_target = &rest[pos + cur_offset..];
        let after_lower = after_target.to_ascii_lowercase();

        if after_lower.starts_with(" after phase") {
            cur_offset += " after phase".len();
            // Skip optional spaces/tabs between "after phase" and phase identifier
            let rem = &rest[pos + cur_offset..];
            let rem_trimmed = rem.trim_start_matches(|c: char| c == ' ' || c == '\t');
            cur_offset += rem.len() - rem_trimmed.len();
            // Consume optional alphanumeric phase identifier (e.g., "1", "2", "3", "IV")
            let id_len = rem_trimmed.chars().take_while(|c| c.is_ascii_alphanumeric()).count();
            cur_offset += id_len;
        }

        // Forward scanning: consume trailing closing envelope on the line
        // Envelope characters: spaces, tabs, closing parenthesis ')', markdown emphasis '*', '_'
        let forward_slice = &rest[pos + cur_offset..];
        let forward_bytes = forward_slice.as_bytes();
        let mut i = 0;
        // Skip horizontal spaces/tabs
        while i < forward_bytes.len() && (forward_bytes[i] == b' ' || forward_bytes[i] == b'\t') {
            i += 1;
        }
        // Skip optional ')'
        if i < forward_bytes.len() && forward_bytes[i] == b')' {
            i += 1;
        }
        // Skip horizontal spaces/tabs
        while i < forward_bytes.len() && (forward_bytes[i] == b' ' || forward_bytes[i] == b'\t') {
            i += 1;
        }
        // Skip closing markdown emphasis: '*' or '_'
        while i < forward_bytes.len() && (forward_bytes[i] == b'*' || forward_bytes[i] == b'_') {
            i += 1;
        }
        // Skip optional ')' (in case formatted like (*...*))
        if i < forward_bytes.len() && forward_bytes[i] == b')' {
            i += 1;
        }
        // Skip trailing horizontal spaces/tabs up to newline
        while i < forward_bytes.len() && (forward_bytes[i] == b' ' || forward_bytes[i] == b'\t') {
            i += 1;
        }
        let end_idx = pos + cur_offset + i;

        // Backward scanning: check line preceding the placeholder in `rest`
        let before = &rest[..pos];
        let line_start = before.rfind('\n').map(|idx| idx + 1).unwrap_or(0);
        let line_prefix = &rest[line_start..pos];

        // Measure leading indentation on this line
        let indent_len = line_prefix.len() - line_prefix.trim_start_matches(|c: char| c == ' ' || c == '\t').len();
        let after_indent = &line_prefix[indent_len..];

        // Check if after_indent contains only markdown markers/envelope:
        // bullets ('-', '*', '+'), emphasis ('*', '_'), parentheses ('('), whitespace (' ', '\t')
        let is_pure_envelope = !after_indent.chars().any(|c| {
            c != '-' && c != '*' && c != '+' && c != '_' && c != '(' && c != ' ' && c != '\t'
        });

        let start_idx = if is_pure_envelope {
            line_start + indent_len
        } else {
            // If there was preceding non-envelope text, peel only trailing envelope from `before`
            let mut j = pos;
            while j > line_start {
                let prev_char = rest[..j].chars().next_back().unwrap();
                if prev_char == ' ' || prev_char == '\t' || prev_char == '*' || prev_char == '_' || prev_char == '(' {
                    j -= prev_char.len_utf8();
                } else {
                    break;
                }
            }
            j
        };

        result.push_str(&rest[..start_idx]);
        result.push_str(replacement);
        rest = &rest[end_idx..];
        purged_count += 1;
    }

    result.push_str(rest);
    (result, purged_count)
}

/// Parses all `- [ ]` or `- [x]` checklist items from markdown content.
pub fn parse_checklists(content: &str) -> Vec<ChecklistItem> {
    let mut items = Vec::new();

    for (idx, line) in content.lines().enumerate() {
        if let Some(parsed) = parse_checklist_line(line) {
            items.push(ChecklistItem {
                line_number: idx + 1,
                checked: parsed.checked,
                issue_number: parsed.issue_number,
                text: parsed.text,
                full_line: line.to_string(),
            });
        }
    }

    items
}

/// Reconciles checklist items in markdown content using a mapping of issue numbers or normalized titles to resolved status.
pub fn reconcile_checklist_content<F>(
    content: &str,
    is_item_resolved: F,
) -> (String, usize)
where
    F: Fn(Option<u64>, &str) -> Option<bool>,
{
    let mut new_lines = Vec::new();
    let mut modified_count = 0;

    for line in content.lines() {
        if let Some(parsed) = parse_checklist_line(line) {
            if let Some(target_resolved) = is_item_resolved(parsed.issue_number, parsed.text.trim()) {
                if target_resolved != parsed.checked {
                    let mark = if target_resolved { "x" } else { " " };
                    let new_line = format!("{}{}{}", &line[..parsed.mark_pos], mark, &line[parsed.mark_pos + 1..]);
                    new_lines.push(new_line);
                    modified_count += 1;
                    continue;
                }
            }
        }
        new_lines.push(line.to_string());
    }

    (new_lines.join("\n"), modified_count)
}

pub type SpecDocument = crate::LoadedSpec;

fn is_spec_epic(s: &crate::LoadedSpec) -> bool {
    s.frontmatter.spec_type.eq_ignore_ascii_case("epic")
        || s.rel_path.starts_with("docs/epics")
        || s.rel_path.starts_with("epics")
}

fn is_spec_feature(s: &crate::LoadedSpec) -> bool {
    s.frontmatter.spec_type.eq_ignore_ascii_case("feature")
        || s.rel_path.starts_with("docs/features")
        || s.rel_path.starts_with("features")
}

fn is_spec_use_case(s: &crate::LoadedSpec) -> bool {
    s.frontmatter.spec_type.eq_ignore_ascii_case("use_case")
        || s.frontmatter.spec_type.eq_ignore_ascii_case("use-case")
        || s.rel_path.starts_with("docs/use-cases")
        || s.rel_path.starts_with("use-cases")
}

fn is_spec_user_story(s: &crate::LoadedSpec) -> bool {
    s.frontmatter.spec_type.eq_ignore_ascii_case("user_story")
        || s.frontmatter.spec_type.eq_ignore_ascii_case("user-story")
        || s.rel_path.starts_with("docs/user-stories")
        || s.rel_path.starts_with("user-stories")
}

fn get_spec_stem(path: &str) -> String {
    let fname = std::path::Path::new(path)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or(path);
    fname.strip_suffix(".md").unwrap_or(fname).to_string()
}

fn spec_matches_epic(epic: &crate::LoadedSpec, child: &crate::LoadedSpec, total_epics: usize) -> bool {
    if child.rel_path == epic.rel_path {
        return false;
    }
    if total_epics <= 1 {
        return true;
    }
    let epic_stem = get_spec_stem(&epic.rel_path);
    let epic_file = std::path::Path::new(&epic.rel_path)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("");
    let child_file = std::path::Path::new(&child.rel_path)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("");

    if let Some(ref ep) = child.frontmatter.epic {
        if ep.contains(&epic_stem)
            || (!epic_file.is_empty() && ep.contains(epic_file))
            || (!epic.frontmatter.title.is_empty() && ep.contains(&epic.frontmatter.title))
        {
            return true;
        }
    }

    if let (Some(sub_c), Some(sub_p)) = (&child.frontmatter.subsystem, &epic.frontmatter.subsystem) {
        if !sub_c.is_empty() && sub_c.eq_ignore_ascii_case(sub_p) {
            return true;
        }
    }

    if (!child_file.is_empty() && epic.content.contains(child_file))
        || (!epic_file.is_empty() && child.content.contains(epic_file))
    {
        return true;
    }

    false
}

fn spec_matches_feature(feat: &crate::LoadedSpec, child: &crate::LoadedSpec, total_features: usize) -> bool {
    if child.rel_path == feat.rel_path {
        return false;
    }
    if total_features <= 1 {
        return true;
    }
    let feat_stem = get_spec_stem(&feat.rel_path);
    let feat_file = std::path::Path::new(&feat.rel_path)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("");

    if !feat_file.is_empty() && child.content.contains(feat_file) {
        return true;
    }
    if !feat_stem.is_empty() && child.content.contains(&feat_stem) {
        return true;
    }
    if !feat.frontmatter.title.is_empty() && child.content.contains(&feat.frontmatter.title) {
        return true;
    }

    if let Some(feat_ref) = child.frontmatter.raw_fields.get("feature") {
        if feat_ref.contains(&feat_stem) || feat_ref.contains(&feat.frontmatter.title) {
            return true;
        }
    }

    if let (Some(sub_c), Some(sub_f)) = (&child.frontmatter.subsystem, &feat.frontmatter.subsystem) {
        if !sub_c.is_empty() && sub_c.eq_ignore_ascii_case(sub_f) {
            return true;
        }
    }

    if let (Some(part_c), Some(part_f)) = (&child.frontmatter.part, &feat.frontmatter.part) {
        if !part_c.is_empty() && part_c.eq_ignore_ascii_case(part_f) {
            return true;
        }
    }

    false
}

fn format_canonical_checklist_item(
    spec: &crate::LoadedSpec,
    repo_slug: &str,
    default_branch: &str,
    issue_index: &crate::tracker::IssueIndex,
    default_clause: Option<&str>,
) -> String {
    let issue_num = spec.frontmatter.issue_id.or_else(|| {
        issue_index
            .find_issue_for_type(None, Some(&spec.frontmatter.spec_type), &spec.frontmatter.title)
            .map(|i| i.number)
    });

    let (is_resolved, issue_ref) = match issue_num {
        Some(num) => {
            let resolved = issue_index.by_number.get(&num).map(|i| i.is_resolved()).unwrap_or(false);
            (resolved, format!("#{}", num))
        }
        None => (false, "#[IssueID]".to_string()),
    };

    let mark = if is_resolved { "x" } else { " " };

    let title = if !spec.frontmatter.title.trim().is_empty() {
        spec.frontmatter.title.trim()
    } else {
        let fname = std::path::Path::new(&spec.rel_path)
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or(&spec.rel_path);
        fname.strip_suffix(".md").unwrap_or(fname)
    };

    let clean_repo = crate::clean_repo_name(repo_slug);
    let branch = if default_branch.trim().is_empty() {
        "main"
    } else {
        default_branch.trim()
    };

    let doc_path = if spec.rel_path.starts_with("docs/") {
        spec.rel_path.clone()
    } else {
        format!("docs/{}", spec.rel_path)
    };

    let link = if !clean_repo.is_empty() {
        format!("https://github.com/{}/blob/{}/{}", clean_repo, branch, doc_path)
    } else {
        let stripped = spec.rel_path.strip_prefix("docs/").unwrap_or(&spec.rel_path);
        format!("../{}", stripped)
    };

    let clause_suffix = if let Some(clauses) = spec.frontmatter.raw_fields.get("clauses") {
        format!(" ({})", clauses)
    } else if let Some(justification) = spec.frontmatter.raw_fields.get("justification") {
        format!(" ({})", justification)
    } else if let Some(default_c) = default_clause {
        format!(" ({})", default_c)
    } else {
        String::new()
    };

    format!("- [{}] {} - [{}]({}){}", mark, issue_ref, title, link, clause_suffix)
}

fn contains_spec_reference(line: &str, spec: &crate::LoadedSpec) -> bool {
    let spec_file = std::path::Path::new(&spec.rel_path)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("");
    let spec_stem = get_spec_stem(&spec.rel_path);

    if !spec_file.is_empty() && line.contains(spec_file) {
        return true;
    }
    if !spec_stem.is_empty() && line.contains(&spec_stem) {
        return true;
    }
    if !spec.frontmatter.title.is_empty() && line.contains(&spec.frontmatter.title) {
        return true;
    }
    if let Some(id) = spec.frontmatter.issue_id {
        if line.contains(&format!("#{}", id)) {
            return true;
        }
    }
    false
}

fn reconcile_or_inject_section(
    content: &str,
    heading: &str,
    new_items: &[String],
    specs: &[&crate::LoadedSpec],
    none_fallback: &str,
    next_delimiters: &[&str],
) -> String {
    if let Some(heading_pos) = content.find(heading) {
        let after_heading_pos = heading_pos + heading.len();
        let rest = &content[after_heading_pos..];

        // Find end of section
        let mut section_len = rest.len();
        for &delim in next_delimiters {
            if let Some(d_pos) = rest.find(delim) {
                if d_pos < section_len {
                    section_len = d_pos;
                }
            }
        }

        let section_body = &rest[..section_len];
        let after_section = &rest[section_len..];

        let mut lines: Vec<String> = Vec::new();
        let is_placeholder_or_empty = section_body.trim().is_empty()
            || contains_unpopulated_placeholders(section_body)
            || section_body.contains("- None allocated")
            || section_body.contains("- None directly allocated");

        if new_items.is_empty() {
            if is_placeholder_or_empty {
                lines.push(none_fallback.to_string());
            } else {
                for line in section_body.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        lines.push(line.to_string());
                    }
                }
            }
        } else if is_placeholder_or_empty {
            for item in new_items {
                lines.push(item.clone());
            }
        } else {
            // Keep existing non-placeholder lines
            for line in section_body.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty()
                    || contains_unpopulated_placeholders(trimmed)
                    || trimmed.starts_with("- None allocated")
                    || trimmed.starts_with("- None directly allocated")
                {
                    continue;
                }
                lines.push(line.to_string());
            }

            // Append any missing child items
            for (idx, item) in new_items.iter().enumerate() {
                if let Some(spec) = specs.get(idx) {
                    let already_present = lines.iter().any(|l| contains_spec_reference(l, spec));
                    if !already_present {
                        lines.push(item.clone());
                    }
                }
            }
        }

        let mut new_body = String::new();
        if !lines.is_empty() {
            new_body.push('\n');
            new_body.push_str(&lines.join("\n"));
            new_body.push('\n');
        }

        format!("{}{}{}{}", &content[..after_heading_pos], new_body, if after_section.starts_with('\n') { "" } else { "\n" }, after_section)
    } else {
        content.to_string()
    }
}

/// Synthesizes and reconciles bidirectional checklist hierarchy for Epic and Feature specifications.
///
/// Mechanically synthesizes `## 2. Requirements & Checklist` in Epic specifications from child
/// `FEAT-*` specifications, and `### Associated Use Cases & User Stories` (`#### Associated Use Cases`
/// and `#### Associated User Stories`) in Epic and Feature specifications from child `UC-*` and `US-*`
/// specifications, formatting each entry canonically as:
/// `- [x| ] #<IssueID> - [<Title>](https://github.com/<owner>/<repo>/blob/<branch>/docs/...) (<clauses>)`
///
/// # Arguments
/// * `parent_spec` - The parent specification document to reconcile.
/// * `child_specs` - Sibling and child specification documents available in the workspace.
/// * `repo_slug` - GitHub repository slug (e.g. `owner/repo`) or empty string for local relative links.
/// * `default_branch` - Default git branch name (e.g. `main`).
/// * `issue_index` - Queryable index of tracker issues used to resolve issue numbers and closure states.
///
/// # Returns
/// Updated Markdown content for the parent specification with canonical checklists synthesized and reconciled.
///
/// # Errors
/// This function operates infallibly on UTF-8 string slices without raising runtime errors.
///
/// # Invariants
/// * Deterministic Mechanical Backlog Traceability: Every child specification is deterministically linked.
/// * Zero Unpopulated Fallbacks: Prohibited fallback placeholders are replaced with canonical allocations.
/// * Pure Schema-Driven Hierarchy: Links strictly reflect AST child-parent ownership.
///
/// # Architecture Context
/// Part of `crates/reconcile-backlog`, bridging SysML v2 AST lowering with GitHub tracker hierarchies.
///
/// # Realises
/// [Strict Bidirectional Checklist Sync, Zero-Placeholder Invariant, Deterministic Mechanical Backlog Traceability]
pub fn synthesize_and_reconcile_hierarchy(
    parent_spec: &crate::LoadedSpec,
    child_specs: &[crate::LoadedSpec],
    repo_slug: &str,
    default_branch: &str,
    issue_index: &crate::tracker::IssueIndex,
) -> String {
    let is_epic = is_spec_epic(parent_spec);
    let is_feature = is_spec_feature(parent_spec);

    if !is_epic && !is_feature {
        return parent_spec.content.clone();
    }

    let mut updated = parent_spec.content.clone();

    if is_epic {
        let total_epics = child_specs.iter().filter(|s| is_spec_epic(s)).count() + 1;
        let mut child_features: Vec<&crate::LoadedSpec> = Vec::new();
        let mut child_use_cases: Vec<&crate::LoadedSpec> = Vec::new();
        let mut child_user_stories: Vec<&crate::LoadedSpec> = Vec::new();

        for child in child_specs {
            if is_spec_feature(child) && spec_matches_epic(parent_spec, child, total_epics) {
                child_features.push(child);
            } else if is_spec_use_case(child) && spec_matches_epic(parent_spec, child, total_epics) {
                child_use_cases.push(child);
            } else if is_spec_user_story(child) && spec_matches_epic(parent_spec, child, total_epics) {
                child_user_stories.push(child);
            }
        }

        child_features.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
        child_use_cases.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
        child_user_stories.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));

        let feat_items: Vec<String> = child_features
            .iter()
            .map(|f| format_canonical_checklist_item(f, repo_slug, default_branch, issue_index, None))
            .collect();
        let uc_items: Vec<String> = child_use_cases
            .iter()
            .map(|u| format_canonical_checklist_item(u, repo_slug, default_branch, issue_index, None))
            .collect();
        let us_items: Vec<String> = child_user_stories
            .iter()
            .map(|s| format_canonical_checklist_item(s, repo_slug, default_branch, issue_index, None))
            .collect();

        // 1. Reconcile or inject ## 2. Requirements & Checklist
        if updated.contains("## 2. Requirements & Checklist") {
            updated = reconcile_or_inject_section(
                &updated,
                "## 2. Requirements & Checklist",
                &feat_items,
                &child_features,
                "- None allocated (structural subsystem component)",
                &["\n### ", "\n## "],
            );
        } else if !feat_items.is_empty() {
            let mut sec = String::from("\n## 2. Requirements & Checklist\n");
            sec.push_str(&feat_items.join("\n"));
            sec.push('\n');

            if let Some(pos) = updated.find("\n## 3. Architecture") {
                updated.insert_str(pos, &sec);
            } else if let Some(pos) = updated.find("\n## 1. Context") {
                let after_pos = pos + "\n## 1. Context".len();
                let next_pos = updated[after_pos..].find("\n## ").map(|p| after_pos + p).unwrap_or(updated.len());
                updated.insert_str(next_pos, &sec);
            } else if let Some(pos) = updated.find("\n## ") {
                updated.insert_str(pos, &sec);
            } else {
                updated.push_str(&sec);
            }
        }

        // 2. Reconcile or inject ### Associated Use Cases & User Stories
        if updated.contains("### Associated Use Cases & User Stories") {
            updated = reconcile_or_inject_section(
                &updated,
                "#### Associated Use Cases",
                &uc_items,
                &child_use_cases,
                "- None directly allocated (operational behavior allocated at system ConOps level in Epic 02)",
                &["\n#### ", "\n### ", "\n## "],
            );
            updated = reconcile_or_inject_section(
                &updated,
                "#### Associated User Stories",
                &us_items,
                &child_user_stories,
                "- None directly allocated (operational behavior allocated at system ConOps level in Epic 02)",
                &["\n#### ", "\n### ", "\n## "],
            );
        } else if !uc_items.is_empty() || !us_items.is_empty() {
            let uc_text = if uc_items.is_empty() {
                "- None directly allocated (operational behavior allocated at system ConOps level in Epic 02)".to_string()
            } else {
                uc_items.join("\n")
            };
            let us_text = if us_items.is_empty() {
                "- None directly allocated (operational behavior allocated at system ConOps level in Epic 02)".to_string()
            } else {
                us_items.join("\n")
            };

            let assoc_block = format!(
                "\n### Associated Use Cases & User Stories\n\n#### Associated Use Cases\n{}\n\n#### Associated User Stories\n{}\n",
                uc_text, us_text
            );

            // Insert after ## 2. Requirements & Checklist
            if let Some(pos) = updated.find("## 2. Requirements & Checklist") {
                let rest_idx = pos + "## 2. Requirements & Checklist".len();
                let next_sec = updated[rest_idx..].find("\n## ").map(|p| rest_idx + p).unwrap_or(updated.len());
                updated.insert_str(next_sec, &assoc_block);
            } else if let Some(pos) = updated.find("\n## 3. Architecture") {
                updated.insert_str(pos, &assoc_block);
            } else {
                updated.push_str(&assoc_block);
            }
        }
    } else if is_feature {
        let total_features = child_specs.iter().filter(|s| is_spec_feature(s)).count() + 1;
        let mut child_use_cases: Vec<&crate::LoadedSpec> = Vec::new();
        let mut child_user_stories: Vec<&crate::LoadedSpec> = Vec::new();

        for child in child_specs {
            if is_spec_use_case(child) && spec_matches_feature(parent_spec, child, total_features) {
                child_use_cases.push(child);
            } else if is_spec_user_story(child) && spec_matches_feature(parent_spec, child, total_features) {
                child_user_stories.push(child);
            }
        }

        child_use_cases.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
        child_user_stories.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));

        let uc_items: Vec<String> = child_use_cases
            .iter()
            .map(|u| format_canonical_checklist_item(u, repo_slug, default_branch, issue_index, None))
            .collect();
        let us_items: Vec<String> = child_user_stories
            .iter()
            .map(|s| format_canonical_checklist_item(s, repo_slug, default_branch, issue_index, None))
            .collect();

        if updated.contains("### Associated Use Cases & User Stories")
            || updated.contains("#### Associated Use Cases")
            || updated.contains("#### Associated User Stories")
        {
            updated = reconcile_or_inject_section(
                &updated,
                "#### Associated Use Cases",
                &uc_items,
                &child_use_cases,
                "- None allocated (structural subsystem component)",
                &["\n#### ", "\n### ", "\n## "],
            );
            updated = reconcile_or_inject_section(
                &updated,
                "#### Associated User Stories",
                &us_items,
                &child_user_stories,
                "- None allocated (structural subsystem component)",
                &["\n#### ", "\n### ", "\n## "],
            );
        } else if !uc_items.is_empty() || !us_items.is_empty() {
            let uc_text = if uc_items.is_empty() {
                "- None allocated (structural subsystem component)".to_string()
            } else {
                uc_items.join("\n")
            };
            let us_text = if us_items.is_empty() {
                "- None allocated (structural subsystem component)".to_string()
            } else {
                us_items.join("\n")
            };

            let assoc_block = format!(
                "\n### Associated Use Cases & User Stories\n\n#### Associated Use Cases\n{}\n\n#### Associated User Stories\n{}\n",
                uc_text, us_text
            );

            if let Some(pos) = updated.find("\n## Acceptance Criteria") {
                updated.insert_str(pos, &assoc_block);
            } else if let Some(pos) = updated.find("\n## Given-When-Then Acceptance Criteria") {
                updated.insert_str(pos, &assoc_block);
            } else if let Some(pos) = updated.find("\n## Source References") {
                updated.insert_str(pos, &assoc_block);
            } else {
                updated.push_str(&assoc_block);
            }
        }
    }

    updated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_checklists() -> Result<(), Box<dyn std::error::Error>> {
        let md = r#"
## 2. Requirements & Checklist
- [ ] #10 - Feature 01: [ConOps] Human Engineer Interface
- [x] #11 - Feature 02: [ConOps] CI Runner
- [ ] Feature without issue
"#;
        let items = parse_checklists(md);
        assert_eq!(items.len(), 3);
        let first = items.first().ok_or("missing checklist item 0")?;
        assert!(!first.checked);
        assert_eq!(first.issue_number, Some(10));
        let second = items.get(1).ok_or("missing checklist item 1")?;
        assert!(second.checked);
        assert_eq!(second.issue_number, Some(11));
        let third = items.get(2).ok_or("missing checklist item 2")?;
        assert!(!third.checked);
        assert_eq!(third.issue_number, None);
        Ok(())
    }

    #[test]
    fn test_contains_unpopulated_placeholders() {
        let placeholder = "*To be populated after Phase 3*";
        assert!(contains_unpopulated_placeholders(placeholder));
        let clean = "- None allocated (structural subsystem component)";
        assert!(!contains_unpopulated_placeholders(clean));
    }

    #[test]
    fn test_clean_unpopulated_placeholders() {
        let md = r#"
#### Associated Use Cases
*To be populated after Phase 3*
"#;
        let cleaned = clean_unpopulated_placeholders(md);
        assert!(!cleaned.contains("To be populated after Phase 3"));
        assert!(cleaned.contains("- None allocated (structural subsystem component)"));
    }

    #[test]
    fn test_reconcile_checklist_content() {
        let md = r#"
- [ ] #10 - Feature 01: Test
- [ ] #11 - Feature 02: Another
"#;
        let (reconciled, count) = reconcile_checklist_content(md, |issue_num, _| {
            if issue_num == Some(10) {
                Some(true)
            } else {
                None
            }
        });
        assert_eq!(count, 1);
        assert!(reconciled.contains("- [x] #10 - Feature 01: Test"));
        assert!(reconciled.contains("- [ ] #11 - Feature 02: Another"));
    }

    #[test]
    fn test_issue_439_placeholder_purging_regression() {
        let canonical_replacement = "- None allocated (structural subsystem component)";

        // 1. All placeholder phase variants and formatting variations
        let test_cases = vec![
            // Phase variants
            "*To be populated after Phase 3*",
            "*To be populated after Phase 2*",
            "*To be populated after Phase 1*",
            "*To be populated after Phase*",
            "*To be populated*",
            "*(To be populated)*",
            "*(To be populated after Phase 3)*",
            // Formatting variations
            "**To be populated after Phase 3**",
            "_To be populated after Phase 3_",
            "- *To be populated after Phase 3*",
            "TO BE POPULATED AFTER PHASE 2",
        ];

        for raw in test_cases {
            // Precondition: detector identifies raw placeholder as unpopulated
            assert!(
                contains_unpopulated_placeholders(raw),
                "Precondition failed: detector missed placeholder variant '{}'",
                raw
            );

            let (cleaned, count) = clean_unpopulated_placeholders_counted(raw);
            assert_eq!(
                count, 1,
                "Expected count == 1 for raw placeholder '{}', got count = {}",
                raw, count
            );
            assert!(
                cleaned.contains(canonical_replacement),
                "Cleaned text did not contain canonical replacement for '{}':\n{}",
                raw, cleaned
            );
            // Postcondition assertion
            assert!(
                !contains_unpopulated_placeholders(&cleaned),
                "Postcondition failed: cleaned text still contains unpopulated placeholder for '{}':\n{}",
                raw, cleaned
            );
            // Syntax purity assertions
            assert!(
                !cleaned.contains("*- None"),
                "Syntax corruption '*- None' found for '{}':\n{}",
                raw, cleaned
            );
            assert!(
                !cleaned.contains("_- None"),
                "Syntax corruption '_- None' found for '{}':\n{}",
                raw, cleaned
            );
            assert!(
                !cleaned.contains("- - None"),
                "Syntax corruption '- - None' found for '{}':\n{}",
                raw, cleaned
            );

            // Idempotency: running cleaner on already-cleaned text produces count == 0 and identical output
            let (second_cleaned, second_count) = clean_unpopulated_placeholders_counted(&cleaned);
            assert_eq!(second_count, 0, "Idempotency failed: second pass count = {} on '{}'", second_count, raw);
            assert_eq!(second_cleaned, cleaned, "Idempotency failed: output changed on second pass for '{}'", raw);
        }

        // 2. Multi-block counting: 3 blocks in one string -> count == 3
        let multi_block = r#"
#### Block 1
*To be populated after Phase 1*

#### Block 2
*To be populated after Phase 2*

#### Block 3
*To be populated after Phase 3*
"#;
        let (cleaned_multi, multi_count) = clean_unpopulated_placeholders_counted(multi_block);
        assert_eq!(multi_count, 3, "Multi-block test failed: expected count 3, got {}", multi_count);
        assert!(
            !contains_unpopulated_placeholders(&cleaned_multi),
            "Postcondition failed on multi-block:\n{}",
            cleaned_multi
        );
        let occurrences = cleaned_multi.matches(canonical_replacement).count();
        assert_eq!(occurrences, 3, "Expected 3 occurrences of canonical replacement, got {}", occurrences);
        assert!(!cleaned_multi.contains("*- None"));
        assert!(!cleaned_multi.contains("_- None"));
        assert!(!cleaned_multi.contains("- - None"));

        // Idempotency on multi-block
        let (idempotent_multi, idempotent_count) = clean_unpopulated_placeholders_counted(&cleaned_multi);
        assert_eq!(idempotent_count, 0);
        assert_eq!(idempotent_multi, cleaned_multi);
    }

    #[test]
    fn test_issue_448_synthesize_and_reconcile_hierarchy_and_tracker_link_projection() {
        use crate::frontmatter::SpecFrontmatter;
        use crate::tracker::TrackerIssue;
        use crate::LoadedSpec;
        use std::path::PathBuf;

        // 1. Child feature specification
        let feat = LoadedSpec {
            path: PathBuf::from("docs/features/FEAT-01-Tracker.md"),
            rel_path: "docs/features/FEAT-01-Tracker.md".to_string(),
            frontmatter: SpecFrontmatter {
                title: "Feature 01: [ConOps] Radar Tracker".to_string(),
                spec_type: "feature".to_string(),
                issue_id: Some(10),
                epic: Some("EPIC-01-AirDefense".to_string()),
                subsystem: Some("Radar".to_string()),
                part: Some("Tracker".to_string()),
                ..Default::default()
            },
            content: r#"# Feature 01: [ConOps] Radar Tracker

### Associated Use Cases & User Stories
#### Associated Use Cases
*To be populated after Phase 3*

#### Associated User Stories
*To be populated after Phase 3*
"#.to_string(),
        };

        // 2. Child use case specification
        let uc = LoadedSpec {
            path: PathBuf::from("docs/use-cases/UC-01-ProcessRadarTracks.md"),
            rel_path: "docs/use-cases/UC-01-ProcessRadarTracks.md".to_string(),
            frontmatter: SpecFrontmatter {
                title: "Use Case 01: Process Radar Tracks".to_string(),
                spec_type: "use_case".to_string(),
                issue_id: Some(20),
                subsystem: Some("Radar".to_string()),
                ..Default::default()
            },
            content: r#"# Use Case 01: Process Radar Tracks
| **#10** | Feature | [Feature 01: [ConOps] Radar Tracker](../features/FEAT-01-Tracker.md) |
"#.to_string(),
        };

        // 3. Child user story specification
        let us = LoadedSpec {
            path: PathBuf::from("docs/user-stories/US-01-IngestRadarTelemetry.md"),
            rel_path: "docs/user-stories/US-01-IngestRadarTelemetry.md".to_string(),
            frontmatter: SpecFrontmatter {
                title: "User Story 01: Ingest Radar Telemetry".to_string(),
                spec_type: "user-story".to_string(),
                issue_id: Some(30),
                subsystem: Some("Radar".to_string()),
                ..Default::default()
            },
            content: r#"# User Story 01: Ingest Radar Telemetry
## Required Features
- [ ] [Feature 01: [ConOps] Radar Tracker](../features/FEAT-01-Tracker.md)
"#.to_string(),
        };

        // 4. Parent Epic without ## 2. Requirements & Checklist
        let epic = LoadedSpec {
            path: PathBuf::from("docs/epics/EPIC-01-AirDefense.md"),
            rel_path: "docs/epics/EPIC-01-AirDefense.md".to_string(),
            frontmatter: SpecFrontmatter {
                title: "Epic 01: Air Defense Subsystem".to_string(),
                spec_type: "epic".to_string(),
                issue_id: Some(5),
                subsystem: Some("Radar".to_string()),
                ..Default::default()
            },
            content: r#"# Epic 01: Air Defense Subsystem

## 1. Context
Air defense subsystem operations.

## 3. Architecture
Subsystem composition.
"#.to_string(),
        };

        let mut issue_index = crate::tracker::IssueIndex::default();
        issue_index.by_number.insert(10, TrackerIssue {
            number: 10,
            title: "Feature 01: [ConOps] Radar Tracker".to_string(),
            labels: vec![],
            state: "CLOSED".to_string(), // resolved!
            body: None,
            url: None,
        });
        issue_index.by_number.insert(20, TrackerIssue {
            number: 20,
            title: "Use Case 01: Process Radar Tracks".to_string(),
            labels: vec![],
            state: "OPEN".to_string(),
            body: None,
            url: None,
        });
        issue_index.by_number.insert(30, TrackerIssue {
            number: 30,
            title: "User Story 01: Ingest Radar Telemetry".to_string(),
            labels: vec![],
            state: "OPEN".to_string(),
            body: None,
            url: None,
        });

        let all_specs = vec![epic.clone(), feat.clone(), uc.clone(), us.clone()];

        // Test Epic hierarchy synthesis:
        let reconciled_epic = synthesize_and_reconcile_hierarchy(
            &epic,
            &all_specs,
            "org/repo",
            "main",
            &issue_index,
        );

        assert!(reconciled_epic.contains("## 2. Requirements & Checklist"), "Epic must synthesize ## 2. Requirements & Checklist");
        assert!(reconciled_epic.contains("- [x] #10 - [Feature 01: [ConOps] Radar Tracker](https://github.com/org/repo/blob/main/docs/features/FEAT-01-Tracker.md)"), "Epic checklist must canonically format child feature with [x] when issue is resolved");
        assert!(reconciled_epic.contains("### Associated Use Cases & User Stories"), "Epic must synthesize Associated Use Cases & User Stories");
        assert!(reconciled_epic.contains("#### Associated Use Cases"), "Epic must synthesize Associated Use Cases");
        assert!(reconciled_epic.contains("- [ ] #20 - [Use Case 01: Process Radar Tracks](https://github.com/org/repo/blob/main/docs/use-cases/UC-01-ProcessRadarTracks.md)"));
        assert!(reconciled_epic.contains("#### Associated User Stories"), "Epic must synthesize Associated User Stories");
        assert!(reconciled_epic.contains("- [ ] #30 - [User Story 01: Ingest Radar Telemetry](https://github.com/org/repo/blob/main/docs/user-stories/US-01-IngestRadarTelemetry.md)"));

        // Test Feature hierarchy synthesis:
        let reconciled_feat = synthesize_and_reconcile_hierarchy(
            &feat,
            &all_specs,
            "org/repo",
            "main",
            &issue_index,
        );

        assert!(!reconciled_feat.contains("To be populated"), "Feature must replace placeholder blocks");
        assert!(reconciled_feat.contains("- [ ] #20 - [Use Case 01: Process Radar Tracks](https://github.com/org/repo/blob/main/docs/use-cases/UC-01-ProcessRadarTracks.md)"));
        assert!(reconciled_feat.contains("- [ ] #30 - [User Story 01: Ingest Radar Telemetry](https://github.com/org/repo/blob/main/docs/user-stories/US-01-IngestRadarTelemetry.md)"));

        // Test project_body_for_tracker:
        let mut path_to_issue = std::collections::BTreeMap::new();
        path_to_issue.insert("docs/features/FEAT-01-Tracker.md".to_string(), 10);
        path_to_issue.insert("features/FEAT-01-Tracker.md".to_string(), 10);

        let input_body = "- [ ] [Feature 01: [ConOps] Radar Tracker](../features/FEAT-01-Tracker.md)";
        let projected = crate::project_body_for_tracker(
            input_body,
            Some("org/repo"),
            "main",
            &path_to_issue,
        );

        assert!(!projected.contains("](#10)"), "Projected body must NOT contain broken intra-page anchor link [Title](#10)!");
        assert!(projected.contains("- [ ] #10 - [Feature 01: [ConOps] Radar Tracker](https://github.com/org/repo/blob/main/docs/features/FEAT-01-Tracker.md)"), "Projected body must translate markdown link to canonical repo blob URL with issue prefix, found:\n{}", projected);
    }

    #[test]
    fn test_extract_checklist_title_and_issue_comprehensive() {
        // 1. Prefix issue with markdown link and clause annotations
        let (issue1, title1) = extract_checklist_title_and_issue("#10 - [Feature 01: [ConOps] Human Engineer Interface](https://github.com/org/repo/blob/main/docs/features/FEAT-01.md) (clauses REQ-01)");
        assert_eq!(issue1, Some(10));
        assert_eq!(title1, "Feature 01: [ConOps] Human Engineer Interface");

        // 2. Markdown link without prefix issue
        let (issue2, title2) = extract_checklist_title_and_issue("[Feature 01: Interface](docs/features/FEAT-01.md)");
        assert_eq!(issue2, None);
        assert_eq!(title2, "Feature 01: Interface");

        // 3. Markdown link with inline issue number
        let (issue3, title3) = extract_checklist_title_and_issue("[Feature 01: Interface](docs/features/FEAT-01.md) (#11)");
        assert_eq!(issue3, Some(11));
        assert_eq!(title3, "Feature 01: Interface");

        // 4. Bold title without issue number
        let (issue4, title4) = extract_checklist_title_and_issue("**Feature 01: Human Engineer**");
        assert_eq!(issue4, None);
        assert_eq!(title4, "Feature 01: Human Engineer");

        // 5. Bold title with inline issue number
        let (issue5, title5) = extract_checklist_title_and_issue("**Feature 01: Human Engineer** (#12)");
        assert_eq!(issue5, Some(12));
        assert_eq!(title5, "Feature 01: Human Engineer");

        // 6. Italic title with refs parenthetical
        let (issue6, title6) = extract_checklist_title_and_issue("*Feature 01: Human Engineer* (refs #13)");
        assert_eq!(issue6, Some(13));
        assert_eq!(title6, "Feature 01: Human Engineer");

        // 7. Colon-separated prefix issue
        let (issue7, title7) = extract_checklist_title_and_issue("#14: Feature 01: Plain Text");
        assert_eq!(issue7, Some(14));
        assert_eq!(title7, "Feature 01: Plain Text");
    }

    #[test]
    fn test_reconcile_checklist_content_markdown_links_and_inline_issues() {
        let content = r#"
## Requirements
- [ ] [Feature 01: Human Engineer Interface](../features/FEAT-01.md)
- [ ] [Feature 02: CI Runner](../features/FEAT-02.md) (#20)
- [x] **Feature 03: Radar Tracking** (#30)
"#;

        let (reconciled, count) = reconcile_checklist_content(content, |issue_num, item_title| {
            if issue_num == Some(20) {
                Some(true)
            } else if issue_num == Some(30) {
                Some(false)
            } else if item_title == "Feature 01: Human Engineer Interface" {
                Some(true)
            } else {
                None
            }
        });

        assert_eq!(count, 3);
        assert!(reconciled.contains("- [x] [Feature 01: Human Engineer Interface](../features/FEAT-01.md)"));
        assert!(reconciled.contains("- [x] [Feature 02: CI Runner](../features/FEAT-02.md) (#20)"));
        assert!(reconciled.contains("- [ ] **Feature 03: Radar Tracking** (#30)"));
    }
}

