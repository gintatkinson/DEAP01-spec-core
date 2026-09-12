#!/usr/bin/env python3
"""
scripts/file_defect.py

Mechanical pre-submission schema validator and issue filer for DEAP01-spec-core.
Validates defect dossiers strictly against the 7-section Adversarial Audit schema
before submitting them to GitHub (via `gh issue create`) or GitLab (via `glab issue create`).

Enforces:
1. Exactly 7 section headers (`## 1.` to `## 7.`).
2. Mandatory `## Audit Source` with `SEVERITY: [Critical|Important|Suggestion|Nitpick]` and `FILE_LOCATION:`.
3. Exactly 5 Whys in Section 2 (`1. **Why ...?** Because ...`).
4. Section 1 three-bullet structure (`**File**:`, `**Pillar**:`, `**Symptom**:`)
5. Section 4 Mermaid diagram validation for Critical/Important findings (offline syntax check)
   or `N/A -- ` declaration for Suggestion/Nitpick findings.
6. Balanced code blocks.
7. No ASCII art UML arrows outside code fences.

Usage:
    python3 scripts/file_defect.py --title "[AUDIT] [file.ext]: [description]" --body-file /path/to/dossier.md --repo gintatkinson/DEAP01-spec-core
    python3 scripts/file_defect.py --body-file /path/to/dossier.md --validate-only
"""

import argparse
import json
import os
import re
import subprocess
import sys
from typing import Any, Dict, List, Optional, Set, Tuple

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PARITY_AUDITOR_SRC = os.path.join(REPO_ROOT, "skills", "spec-orchestrator", "parity_auditor", "src")
if PARITY_AUDITOR_SRC not in sys.path:
    sys.path.insert(0, PARITY_AUDITOR_SRC)

try:
    from parity_auditor.validators.mermaid_syntax_validator import check_mermaid_text
except ImportError:
    check_mermaid_text = None


VALID_SEVERITIES = ("Critical", "Important", "Suggestion", "Nitpick")


def validate_defect_body(
    body_text: str,
    title: Optional[str] = None,
    source_name: str = "<input>",
) -> List[str]:
    """
    Validates a defect body string against the 7-section Adversarial Audit schema.
    Returns a list of error strings. Empty list indicates full compliance.
    """
    errors: List[str] = []

    if not body_text or not isinstance(body_text, str) or not body_text.strip():
        errors.append("Defect body is empty or whitespace-only.")
        return errors

    # Check 1: Exactly 7 numbered section headers (## 1. to ## 7.)
    header_matches = list(re.finditer(r"^##\s+([1-7])\.\s+(.*)$", body_text, re.MULTILINE))
    found_numbers = [int(m.group(1)) for m in header_matches]
    if found_numbers != [1, 2, 3, 4, 5, 6, 7]:
        errors.append(
            f"Section headers error: expected exactly 7 section headers numbered '## 1.' through '## 7.' in order. Found: {found_numbers}"
        )

    # Check 2: Audit Source header
    audit_source_match = re.search(r"^##\s+Audit Source\b", body_text, re.MULTILINE | re.IGNORECASE)
    if not audit_source_match:
        errors.append("Missing mandatory '## Audit Source' section header.")

    # Check 3: SEVERITY line
    severity_match = re.search(
        r"^SEVERITY:\s*(Critical|Important|Suggestion|Nitpick)\s*$",
        body_text,
        re.MULTILINE,
    )
    severity = severity_match.group(1) if severity_match else None
    if not severity:
        # Check if severity is present with invalid value
        invalid_sev = re.search(r"^SEVERITY:\s*(\S+.*)$", body_text, re.MULTILINE)
        if invalid_sev:
            errors.append(
                f"Invalid SEVERITY '{invalid_sev.group(1).strip()}'. Must be one of: {', '.join(VALID_SEVERITIES)}."
            )
        else:
            errors.append(
                f"Missing mandatory 'SEVERITY:' line. Must match 'SEVERITY: ({'|'.join(VALID_SEVERITIES)})'."
            )

    # Check 4: FILE_LOCATION line
    file_loc_match = re.search(r"^FILE_LOCATION:\s*(\S+.*)$", body_text, re.MULTILINE)
    if not file_loc_match or not file_loc_match.group(1).strip():
        errors.append("Missing or empty mandatory 'FILE_LOCATION:' line.")

    # Section-specific slices if headers are present
    if len(header_matches) == 7:
        sec1 = body_text[header_matches[0].start() : header_matches[1].start()]
        sec2 = body_text[header_matches[1].start() : header_matches[2].start()]
        sec3 = body_text[header_matches[2].start() : header_matches[3].start()]
        sec4 = body_text[header_matches[3].start() : header_matches[4].start()]
        sec5 = body_text[header_matches[4].start() : header_matches[5].start()]
        sec6 = body_text[header_matches[5].start() : header_matches[6].start()]
        sec7_end = audit_source_match.start() if audit_source_match else len(body_text)
        sec7 = body_text[header_matches[6].start() : sec7_end]

        # Check 5: Section 1 three bold bullet items (**File**:, **Pillar**:, **Symptom**:)
        has_file = bool(re.search(r"^\s*[-*]\s+\*\*File\*\*:\s*.+", sec1, re.MULTILINE))
        has_pillar = bool(re.search(r"^\s*[-*]\s+\*\*Pillar\*\*:\s*.+", sec1, re.MULTILINE))
        has_symptom = bool(re.search(r"^\s*[-*]\s+\*\*Symptom\*\*:\s*.+", sec1, re.MULTILINE))
        if not (has_file and has_pillar and has_symptom):
            missing_bullets = []
            if not has_file:
                missing_bullets.append("- **File**:")
            if not has_pillar:
                missing_bullets.append("- **Pillar**:")
            if not has_symptom:
                missing_bullets.append("- **Symptom**:")
            errors.append(
                f"Section 1 missing mandatory bullet point(s): {', '.join(missing_bullets)}."
            )

        # Check 6: Section 2 (5 Whys)
        why_matches = list(
            re.finditer(
                r"^\s*([1-5])\.\s+\*\*Why\s+.*?\?\*\*\s+Because\s+.+$",
                sec2,
                re.MULTILINE,
            )
        )
        why_numbers = [int(m.group(1)) for m in why_matches]
        if why_numbers != [1, 2, 3, 4, 5]:
            errors.append(
                f"Section 2 must contain exactly 5 'Why ...? Because ...' entries numbered 1 to 5. Found: {why_numbers}"
            )

        # Check 7 & 8: Section 4 UML Diagrams
        if severity in ("Critical", "Important"):
            has_mermaid = "```mermaid" in sec4
            if not has_mermaid:
                errors.append(
                    f"Section 4 must contain a ```mermaid code block for {severity} findings."
                )
            else:
                if check_mermaid_text:
                    mermaid_findings = check_mermaid_text(sec4, source=source_name)
                    for f in mermaid_findings:
                        errors.append(f"Mermaid syntax error in Section 4: {f}")
        elif severity in ("Suggestion", "Nitpick"):
            has_na = bool(re.search(r"N/A\s*(?:--|-)", sec4, re.IGNORECASE))
            if not has_na:
                errors.append(
                    f"Section 4 must declare 'N/A -- {severity} severity.' for {severity} findings."
                )

    # Check 9: Balanced code blocks
    fence_count = len(re.findall(r"^\s*```", body_text, re.MULTILINE))
    if fence_count % 2 != 0:
        errors.append(f"Unbalanced code blocks: found odd number ({fence_count}) of ``` fences.")

    # Check 10: No ASCII art UML arrows outside code fences
    # Strip all code blocks
    non_code_text = re.sub(r"```.*?```", "", body_text, flags=re.DOTALL)
    ascii_arrows = re.findall(r"(->>|-->|→)", non_code_text)
    if ascii_arrows:
        errors.append(
            f"Found ASCII art arrow(s) {set(ascii_arrows)} outside fenced code blocks. Use formal Mermaid diagrams in Section 4."
        )

    # Check 11: Title format if provided
    if title is not None:
        title_stripped = title.strip()
        if not title_stripped:
            errors.append("Title cannot be empty.")

    return errors


STOPWORDS = {
    "the", "a", "an", "and", "or", "for", "with", "from", "that", "this", "in", "on", "at",
    "by", "to", "of", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
    "do", "does", "did", "audit", "bug", "defect", "issue", "file", "error", "fails", "failed",
    "failing", "missing", "pillar", "critical", "important", "suggestion", "nitpick", "src",
    "scripts", "tests", "validators", "core", "parity_auditor"
}


def extract_file_location(body_text: str) -> Optional[str]:
    """Extracts raw FILE_LOCATION line value from defect body."""
    if not body_text:
        return None
    m = re.search(r"^FILE_LOCATION:\s*(\S+.*)$", body_text, re.MULTILINE)
    return m.group(1).strip() if m else None


def normalize_file_target(file_loc: Optional[str]) -> Tuple[str, str]:
    """
    Returns (normalized_full_path, base_filename) from FILE_LOCATION.
    Strips line number annotations (e.g. ':1137-1141' or ':56-61').
    """
    if not file_loc:
        return "", ""
    clean = file_loc.split(":")[0].strip().strip("`* ")
    norm_path = os.path.normpath(clean).lower()
    base_name = os.path.basename(clean).lower()
    return norm_path, base_name


def extract_core_title_tokens(title: str) -> Set[str]:
    """
    Extracts meaningful core keyword tokens from an issue title,
    stripping bracket tags like [AUDIT], file extensions, numbers, and stopwords.
    """
    if not title:
        return set()
    clean_title = re.sub(r'\[[^\]]*\]', ' ', title)
    raw_tokens = re.findall(r'[a-zA-Z0-9_]{3,}', clean_title.lower())
    return {tok for tok in raw_tokens if tok not in STOPWORDS and not tok.isdigit()}


def find_duplicate_issue(
    candidate_title: str,
    candidate_body: str,
    existing_issues: List[Dict[str, Any]],
) -> Optional[Dict[str, Any]]:
    """
    Searches across existing issues (regardless of state: open, closed, status:fixed-resolved)
    to find duplicates based on exact title match, or matching FILE_LOCATION and core title tokens.
    """
    cand_file_raw = extract_file_location(candidate_body)
    cand_norm_path, cand_base_name = normalize_file_target(cand_file_raw)
    cand_tokens = extract_core_title_tokens(candidate_title)
    cand_title_clean = re.sub(r'\[[^\]]*\]', '', candidate_title).strip().lower()

    for issue in existing_issues:
        issue_title = str(issue.get("title", "")).strip()
        issue_body = str(issue.get("body", "") or issue.get("description", ""))
        issue_title_clean = re.sub(r'\[[^\]]*\]', '', issue_title).strip().lower()

        # 1. Exact or normalized title match
        if cand_title_clean and (cand_title_clean == issue_title_clean or candidate_title.strip().lower() == issue_title.lower()):
            return issue

        # 2. FILE_LOCATION matching + core title token overlap
        ex_file_raw = extract_file_location(issue_body)
        ex_norm_path, ex_base_name = normalize_file_target(ex_file_raw)

        same_file = False
        if cand_norm_path and ex_norm_path:
            if cand_norm_path == ex_norm_path or cand_base_name == ex_base_name:
                same_file = True
        elif cand_base_name and (cand_base_name in issue_title.lower() or cand_base_name in issue_body.lower()):
            same_file = True
        elif ex_base_name and (ex_base_name in candidate_title.lower() or ex_base_name in candidate_body.lower()):
            same_file = True

        if same_file:
            ex_tokens = extract_core_title_tokens(issue_title)
            common_tokens = cand_tokens & ex_tokens
            if len(common_tokens) >= 2:
                return issue
            if cand_tokens and ex_tokens:
                jaccard = len(common_tokens) / len(cand_tokens | ex_tokens)
                if jaccard >= 0.25:
                    return issue
                if len(cand_tokens) <= 2 and len(common_tokens) >= 1:
                    return issue

    return None


def fetch_existing_issues(
    repo: str,
    provider: str = "github",
) -> List[Dict[str, Any]]:
    """Fetches all existing issues across all states (open, closed) from tracker CLI."""
    prov = provider.lower()
    if prov == "github":
        cmd = [
            "gh",
            "issue",
            "list",
            "--repo",
            repo,
            "--state",
            "all",
            "--limit",
            "200",
            "--json",
            "number,title,labels,body,state",
        ]
        try:
            res = subprocess.run(cmd, capture_output=True, text=True)
            if res.returncode == 0 and res.stdout.strip():
                return json.loads(res.stdout)
        except Exception:
            pass
    elif prov == "gitlab":
        cmd = [
            "glab",
            "issue",
            "list",
            "--repo",
            repo,
            "--all",
            "--per-page",
            "100",
            "--output",
            "json",
        ]
        try:
            res = subprocess.run(cmd, capture_output=True, text=True)
            if res.returncode == 0 and res.stdout.strip():
                return json.loads(res.stdout)
        except Exception:
            pass

    return []


def resolve_label(severity: Optional[str], provider: str = "github", explicit_label: Optional[str] = None) -> str:
    """Resolve issue label from finding severity and provider target."""
    if explicit_label:
        return explicit_label

    prov = provider.lower()
    sev = severity.capitalize() if severity else "Important"

    if sev in ("Critical", "Important"):
        return "type::bug" if prov == "gitlab" else "bug"
    else:  # Suggestion, Nitpick
        return "type::feature" if prov == "gitlab" else "enhancement"


def file_defect_issue(
    title: str,
    body_file: str,
    repo: str,
    label: str,
    provider: str = "github",
    dry_run: bool = False,
    existing_issues: Optional[List[Dict[str, Any]]] = None,
) -> int:
    """Files a validated defect issue to GitHub or GitLab, preventing duplicates across all states."""
    prov = provider.lower()
    if prov not in ("github", "gitlab"):
        print(f"Error: Unsupported provider '{provider}'. Must be 'github' or 'gitlab'.", file=sys.stderr)
        return 1

    try:
        with open(body_file, "r", encoding="utf-8") as f:
            body_content = f.read()
    except OSError as exc:
        print(f"Error reading body file {body_file}: {exc}", file=sys.stderr)
        return 1

    # Deduplication check across all issue states
    if existing_issues is None and not dry_run:
        existing_issues = fetch_existing_issues(repo, provider=prov)

    if existing_issues:
        duplicate = find_duplicate_issue(title, body_content, existing_issues)
        if duplicate:
            issue_id = duplicate.get("number") or duplicate.get("iid") or duplicate.get("id") or "UNKNOWN"
            issue_title = duplicate.get("title", "")
            issue_state = duplicate.get("state", "")
            labels_raw = duplicate.get("labels", [])
            labels_str = ", ".join(
                [lbl.get("name", "") if isinstance(lbl, dict) else str(lbl) for lbl in labels_raw]
            )
            print(
                f"[DEDUPLICATION] Duplicate defect detected matching existing issue #{issue_id} "
                f"('{issue_title}') [state: {issue_state}, labels: [{labels_str}]]. Skipping duplicate filing."
            )
            return 0

    if dry_run:
        print("[DRY RUN] Defect validation PASSED. Target payload:")
        print(f"  Provider: {prov}")
        print(f"  Repo: {repo}")
        print(f"  Title: {title}")
        print(f"  Label: {label}")
        print(f"  Body file: {body_file}")
        return 0

    if prov == "github":
        cmd = [
            "gh",
            "issue",
            "create",
            "--repo",
            repo,
            "--title",
            title,
            "--label",
            label,
            "--body-file",
            body_file,
        ]
    else:  # gitlab
        cmd = [
            "glab",
            "issue",
            "create",
            "--repo",
            repo,
            "--title",
            title,
            "--label",
            label,
            "--description",
            body_content,
        ]

    print(f"Executing: {' '.join(cmd)}")
    res = subprocess.run(cmd, capture_output=True, text=True)
    if res.returncode == 0:
        if res.stdout:
            print(res.stdout.strip())
        return 0
    else:
        print(f"Error creating issue via {prov} CLI (exit code {res.returncode}):", file=sys.stderr)
        if res.stderr:
            print(res.stderr.strip(), file=sys.stderr)
        if res.stdout:
            print(res.stdout.strip(), file=sys.stderr)
        return res.returncode


def main():
    parser = argparse.ArgumentParser(
        description="Pre-submission schema validator & issue filer for DEAP01-spec-core"
    )
    parser.add_argument("--title", default=None, help="Issue title (required unless --validate-only/--dry-run)")
    parser.add_argument("--body-file", required=True, help="Path to defect dossier markdown file")
    parser.add_argument("--repo", default="gintatkinson/DEAP01-spec-core", help="Target repository (e.g. owner/repo)")
    parser.add_argument("--label", default=None, help="Issue label (optional, resolved from severity if omitted)")
    parser.add_argument("--provider", default="github", choices=["github", "gitlab"], help="Issue tracker provider")
    parser.add_argument("--dry-run", "--validate-only", dest="dry_run", action="store_true", help="Validate body schema without calling issue create")

    args = parser.parse_args()

    if not os.path.isfile(args.body_file):
        print(f"Error: Body file not found: {args.body_file}", file=sys.stderr)
        sys.exit(1)

    try:
        with open(args.body_file, "r", encoding="utf-8") as f:
            body_content = f.read()
    except OSError as exc:
        print(f"Error reading body file {args.body_file}: {exc}", file=sys.stderr)
        sys.exit(1)

    if not args.dry_run and not args.title:
        print("Error: --title is required when submitting an issue.", file=sys.stderr)
        sys.exit(1)

    errors = validate_defect_body(body_content, title=args.title, source_name=args.body_file)
    if errors:
        print(f"Defect dossier validation FAILED for {args.body_file} with {len(errors)} violation(s):", file=sys.stderr)
        for idx, err in enumerate(errors, 1):
            print(f"  {idx}. {err}", file=sys.stderr)
        sys.exit(1)

    # Extract severity for label resolution
    sev_match = re.search(r"^SEVERITY:\s*(Critical|Important|Suggestion|Nitpick)\s*$", body_content, re.MULTILINE)
    severity = sev_match.group(1) if sev_match else None
    resolved_lbl = resolve_label(severity, provider=args.provider, explicit_label=args.label)

    exit_code = file_defect_issue(
        title=args.title or f"[AUDIT] {os.path.basename(args.body_file)}",
        body_file=args.body_file,
        repo=args.repo,
        label=resolved_lbl,
        provider=args.provider,
        dry_run=args.dry_run,
    )
    sys.exit(exit_code)


if __name__ == "__main__":
    main()
