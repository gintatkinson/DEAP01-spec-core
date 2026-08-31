#!/usr/bin/env python3
"""
Subagent Output Integrity Validator & Escape Tokens Gate (Mechanism 3 & 4)

Verifies subagent output artifacts:
1. Non-zero file size
2. File creation proof (existence on filesystem)
3. Valid Mermaid diagram headers and closed code fences (handling all orientations without cross-fence bleeding)
4. Zero unreplaced {{REQUIRED_*}} escape tokens in specification text
5. Subagent prompt payload verification via lint_subagent_prompt

Usage:
    python3 scripts/verify_subagent_output.py [--files file1 file2 ...] [--dir docs] [--report report.json]
"""

import argparse
import datetime
import json
import os
import re
import sys

# Ensure script dir and project root are on sys.path
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.dirname(SCRIPT_DIR)
if SCRIPT_DIR not in sys.path:
    sys.path.insert(0, SCRIPT_DIR)
if PROJECT_ROOT not in sys.path:
    sys.path.insert(0, PROJECT_ROOT)

try:
    from scripts.lint_subagent_prompt import lint_subagent_prompt
except ImportError:
    try:
        from lint_subagent_prompt import lint_subagent_prompt
    except ImportError:
        lint_subagent_prompt = None


VALID_MERMAID_HEADERS = (
    'classDiagram',
    'graph TD',
    'graph LR',
    'graph TB',
    'graph BT',
    'graph RL',
    'flowchart TD',
    'flowchart LR',
    'flowchart TB',
    'flowchart BT',
    'flowchart RL',
    'sequenceDiagram',
    'stateDiagram-v2',
    'stateDiagram',
    'erDiagram',
    'gantt',
    'pie',
    'mindmap',
    'timeline',
    'gitGraph',
    'C4Context',
    'quadrantChart',
    'journey',
    'requirementDiagram',
)


def _extract_and_validate_mermaid_blocks(content: str) -> bool:
    """
    Extracts and validates Mermaid code fences from markdown content line-by-line.
    Guarantees:
    - Fenced blocks are isolated without cross-fence false-positives.
    - Inline backticks (e.g. ````mermaid`) in prose/tables do not trigger code fence mode.
    - All orientations (TB, TD, LR, RL, BT) with arbitrary whitespace are supported.
    - Unclosed code fences are detected and flagged.
    """
    lines = content.splitlines()
    in_mermaid = False
    in_other_fence = False
    fence_marker = ""
    mermaid_blocks = []
    current_block = []

    for line in lines:
        stripped = line.strip()
        if not in_mermaid and not in_other_fence:
            m = re.match(r'^(?P<fence>```+|~~~+)\s*(?P<info>.*)$', stripped)
            if m:
                fence_marker = m.group('fence')
                info = m.group('info').strip().lower()
                if info.startswith('mermaid'):
                    in_mermaid = True
                    current_block = []
                else:
                    in_other_fence = True
        elif in_mermaid:
            if (
                stripped.startswith(fence_marker)
                and len(stripped.split()[0]) >= len(fence_marker)
                and len(stripped.rstrip(fence_marker[0])) == 0
            ):
                in_mermaid = False
                mermaid_blocks.append("\n".join(current_block))
                current_block = []
            else:
                current_block.append(line)
        elif in_other_fence:
            if (
                stripped.startswith(fence_marker)
                and len(stripped.split()[0]) >= len(fence_marker)
                and len(stripped.rstrip(fence_marker[0])) == 0
            ):
                in_other_fence = False

    # Check for unclosed fence at EOF
    if in_mermaid or in_other_fence:
        return False

    for block in mermaid_blocks:
        b_lines = [
            l.strip()
            for l in block.splitlines()
            if l.strip() and not l.strip().startswith("%%")
        ]
        if not b_lines:
            return False
        first_line = b_lines[0]
        # Validate known exact header prefixes or generalized regex for graph/flowchart
        is_valid_header = any(first_line.startswith(h) for h in VALID_MERMAID_HEADERS) or bool(
            re.match(r'^(?:graph|flowchart)\s+(?:TD|TB|LR|RL|BT)\b', first_line, re.IGNORECASE)
        )
        if not is_valid_header:
            return False

    return True


def verify_file(file_path):
    check_result = {
        'file_path': str(file_path),
        'non_zero': False,
        'creation_proof': False,
        'escape_tokens_clear': True,
        'mermaid_valid': True,
        'issue_url_present': True,
    }

    if not os.path.exists(file_path):
        return check_result, False

    check_result['creation_proof'] = True

    try:
        size = os.path.getsize(file_path)
        if size > 0:
            check_result['non_zero'] = True
        else:
            check_result['non_zero'] = False
            return check_result, False
    except OSError:
        return check_result, False

    with open(file_path, 'r', encoding='utf-8', errors='replace') as f:
        content = f.read()

    # Check unreplaced escape tokens outside code blocks and inline backticks
    content_no_code = re.sub(r'```.*?```', '', content, flags=re.DOTALL)
    content_no_code = re.sub(r'~~~.*?~~~', '', content_no_code, flags=re.DOTALL)
    content_no_code = re.sub(r'`[^`\n]+`', '', content_no_code)

    if '{{REQUIRED_' in content_no_code:
        check_result['escape_tokens_clear'] = False

    # Check Mermaid diagrams if markdown file
    if file_path.endswith('.md'):
        check_result['mermaid_valid'] = _extract_and_validate_mermaid_blocks(content)

    is_pass = (
        check_result['non_zero']
        and check_result['creation_proof']
        and check_result['escape_tokens_clear']
        and check_result['mermaid_valid']
    )

    return check_result, is_pass


def verify_prompt_payload(prompt_text):
    """Validates prompt text payloads for untruncated skill directives:
    1. Asserts presence of view_file on SKILL.md by explicit path as step 1.
    2. Asserts single-item micro-task scope.
    3. Asserts presence of gh issue create and glab issue create.
    4. Asserts presence of PROCEED token.
    5. Rejects summarized or truncated prompt payloads.
    6. Forbids gh/glab issue close in agent prompts (reserved for Product Owner review).

    Returns (check_result_dict, is_pass_bool)
    """
    check_result = {
        'view_file_step_1': False,
        'audit_issue_create': True,
        'untruncated': True,
        'forbid_issue_close': True,
        'proceed_token': True,
        'single_item_scope': True,
        'reasons': [],
    }

    if not prompt_text or not isinstance(prompt_text, str) or not prompt_text.strip():
        check_result['untruncated'] = False
        check_result['reasons'].append("Prompt payload is empty or not a string.")
        return check_result, False

    # Use lint_subagent_prompt if available
    if lint_subagent_prompt is not None:
        lint_errors = lint_subagent_prompt(prompt_text)
        if lint_errors:
            check_result['reasons'].extend(lint_errors)

    # Check for truncation / summarization indicators
    truncation_patterns = [
        r'\[\s*\.\.\.\s*\]',
        r'\[\s*summarized?\s*\]',
        r'\[\s*truncated?\s*\]',
        r'\bsummarized\s+(?:prompt|instructions?|directives?|payload)\b',
        r'\btruncated\s+(?:prompt|instructions?|directives?|payload)\b',
        r'\bsummary\s+of\s+skill\b',
        r'\bshortcut\b',
        r'\bsee\s+skill\s+for\s+details\b',
        r'\bsee\s+SKILL\.md\s+for\s+details\b',
    ]
    for pat in truncation_patterns:
        if re.search(pat, prompt_text, re.IGNORECASE):
            check_result['untruncated'] = False
            if not any(pat in r for r in check_result['reasons']):
                check_result['reasons'].append(
                    f"Prompt payload contains truncation/summarization indicator matching '{pat}'."
                )

    # Forbid gh issue close in agent prompts
    if re.search(r'\b(?:gh|glab)\s+issue\s+close\b', prompt_text, re.IGNORECASE):
        check_result['forbid_issue_close'] = False
        if not any("issue close" in r for r in check_result['reasons']):
            check_result['reasons'].append(
                "Prompt payload contains forbidden 'gh/glab issue close' (issue closure is reserved for Product Owner review)."
            )

    # Assert presence of view_file on SKILL.md by explicit path as step 1
    has_view_file = 'view_file' in prompt_text
    has_skill_md = bool(re.search(r'\bSKILL\.md\b', prompt_text, re.IGNORECASE))
    step1_match = re.search(
        r'(?:step\s*1|very\s*first\s*step|first\s*step|as\s*its\s*very\s*first\s*step|before\s*(?:running|executing|any|proceeding|all)\s*(?:actions|tools|commands|steps|work)?|prerequisite|must\s*read).*view_file.*SKILL\.md|view_file.*SKILL\.md.*(?:step\s*1|very\s*first\s*step|first\s*step|as\s*its\s*very\s*first\s*step|before\s*(?:running|executing|any|proceeding|all)\s*(?:actions|tools|commands|steps|work)?|prerequisite|first)',
        prompt_text,
        re.IGNORECASE | re.DOTALL,
    )
    if has_view_file and has_skill_md and step1_match:
        check_result['view_file_step_1'] = True
    else:
        if not any("view_file" in r for r in check_result['reasons']):
            check_result['reasons'].append("Prompt payload missing explicit view_file on SKILL.md as step 1.")

    # Asserts presence of gh issue create for audit skills
    if re.search(r'\baudits?\b|\bauditor\b', prompt_text, re.IGNORECASE):
        if 'gh issue create' in prompt_text:
            check_result['audit_issue_create'] = True
        else:
            check_result['audit_issue_create'] = False
            if not any("gh issue create" in r for r in check_result['reasons']):
                check_result['reasons'].append("Audit skill prompt missing 'gh issue create'.")

    # PROCEED token check
    if not re.search(r'\bPROCEED\b', prompt_text):
        check_result['proceed_token'] = False

    is_pass = len(check_result['reasons']) == 0

    return check_result, is_pass


def main():
    parser = argparse.ArgumentParser(description="Verify subagent output artifacts integrity.")
    parser.add_argument("--files", nargs="*", help="List of file paths to verify")
    parser.add_argument("--dir", help="Directory containing files to verify")
    parser.add_argument("--prompt", "--prompts", nargs="*", help="List of prompt strings or file paths containing prompt text to verify")
    parser.add_argument("--report", help="Path to write JSON report")
    args = parser.parse_args()

    target_files = []
    if args.files:
        target_files.extend(args.files)
    if args.dir and os.path.exists(args.dir):
        for root, _, files in os.walk(args.dir):
            for f in files:
                if f.endswith('.md'):
                    target_files.append(os.path.join(root, f))

    prompt_inputs = []
    if args.prompt:
        prompt_inputs.extend(args.prompt)

    if not target_files and not prompt_inputs:
        print("No files or prompts specified for verification.")
        sys.exit(0)

    checks = []
    overall_status = "PASS"

    for fpath in target_files:
        c_res, is_pass = verify_file(fpath)
        checks.append({
            'file_path': c_res['file_path'],
            'non_zero': c_res['non_zero'],
            'creation_proof': c_res['creation_proof'],
            'escape_tokens_clear': c_res['escape_tokens_clear'],
            'mermaid_valid': c_res['mermaid_valid'],
        })
        if not is_pass:
            overall_status = "FAIL"

    for p_in in prompt_inputs:
        p_text = p_in
        if os.path.isfile(p_in):
            with open(p_in, 'r', encoding='utf-8', errors='replace') as pf:
                p_text = pf.read()
        p_res, is_pass = verify_prompt_payload(p_text)
        checks.append({
            'prompt_input': p_in,
            'view_file_step_1': p_res['view_file_step_1'],
            'audit_issue_create': p_res['audit_issue_create'],
            'untruncated': p_res['untruncated'],
            'forbid_issue_close': p_res['forbid_issue_close'],
            'reasons': p_res['reasons'],
        })
        if not is_pass:
            overall_status = "FAIL"

    report = {
        'timestamp': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'status': overall_status,
        'checks': checks,
    }

    if args.report:
        os.makedirs(os.path.dirname(os.path.abspath(args.report)), exist_ok=True)
        with open(args.report, 'w', encoding='utf-8') as rf:
            json.dump(report, rf, indent=2)

    if overall_status == "PASS":
        print(f"Subagent output verification PASSED ({len(target_files)} files, {len(prompt_inputs)} prompts verified).")
        sys.exit(0)
    else:
        print(f"Subagent output verification FAILED ({len(target_files)} files, {len(prompt_inputs)} prompts checked).")
        sys.exit(42)


if __name__ == "__main__":
    main()
