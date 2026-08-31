import os
import re
from typing import List
from .base import IValidator
from ..core.findings import Finding
from ..core.workspace import WorkspaceRepository

MD_CONSTRUCTS_INVALID_IN_MERMAID = re.compile(
    r'^(#|> |[\*\-\+]\s|\[[^\]]+\]\(|`[^`]|!\[|\|)'
)

class DocsValidator(IValidator):
    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[str]:
        errors = []
        workspace_dir = repo.workspace_dir
        
        doc_files = [
            "README.md",
            ".pipeline/constitution.md",
            ".pipeline/profiles/flutter.md"
        ]
        
        obsolete_patterns = [
            (r"color\.alarm\b", "color.alarm (obsolete token namespace)"),
            (r"alarm\.cleared\b", "alarm.cleared (obsolete token namespace)"),
            (r"alarm\.minor\b", "alarm.minor (obsolete token namespace)"),
            (r"alarm\.critical\b", "alarm.critical (obsolete token namespace)"),
        ]
        
        rules = repo.get_codebase_rules()
        backlog_dirs = rules.backlog_directories
        backlog_paths = []
        if backlog_dirs:
            for field_name in ["epics", "features", "user_stories", "use_cases"]:
                val = getattr(backlog_dirs, field_name, None)
                if val:
                    backlog_paths.append(val)
        
        # Build list of all files to scan
        all_docs = []
        for rel_path in doc_files:
            all_docs.append((rel_path, os.path.join(workspace_dir, rel_path)))
            
        for dir_rel in backlog_paths:
            dir_path = os.path.join(workspace_dir, dir_rel)
            if not os.path.exists(dir_path):
                continue
            for root, _, files in os.walk(dir_path):
                for file in files:
                    if file.endswith(".md"):
                        full_path = os.path.join(root, file)
                        rel_path = os.path.relpath(full_path, workspace_dir)
                        all_docs.append((rel_path, full_path))
                        
        forbidden_standards = rules.spec_rules.forbidden_standards_blocklist
        
        # Patterns for standard/platform leaks in backlog specs
        leak_checks = [
            (r"\bCode Realization Table\b", "Code Realization Table (belongs in Tier 3 walkthroughs only)"),
            (r"\bError Handling & Codes\b", "Error Handling & Codes header (violates Tier 1 standard-agnostic design)"),
            (r"\bProtocol & Endpoint Definitions\b", "Protocol & Endpoint Definitions header (violates Tier 1 standard-agnostic design)"),
            (r"\b(400 Bad Request|422 Unprocessable Entity|404 Not Found|500 Internal Server Error)\b", "standard HTTP status code"),
            (r"\b\w+\.(py|tsx|ts|dart|cs)\b", "implementation platform source file reference")
        ]
        
        for rel_path, filepath in all_docs:
            if not os.path.exists(filepath):
                continue
            try:
                with open(filepath, "r", encoding="utf-8", errors="ignore") as f:
                    content = f.read()
            except Exception:
                continue
                
            for pattern, name in obsolete_patterns:
                if re.search(pattern, content, re.IGNORECASE):
                    errors.append(Finding(
                        "doc-obsolete-token-namespace-reference",
                        f"Documentation file '{rel_path}' contains obsolete reference '{name}'. Please update it to standard-agnostic status mappings.",
                        location=rel_path,
                    ))
                    
            for standard in forbidden_standards:
                if standard in content:
                    errors.append(Finding(
                        "doc-hardcoded-standard-reference",
                        f"Documentation file '{rel_path}' contains hardcoded reference to '{standard}'. Target profiles and READMEs must remain strictly standard-agnostic.",
                        location=rel_path,
                    ))
                    
            # Check for standard/platform leaks only in backlog specifications
            is_backlog_spec = any(rel_path.startswith(dir_rel) for dir_rel in backlog_paths if dir_rel)
            if is_backlog_spec:
                for pattern, desc in leak_checks:
                    if re.search(pattern, content, re.IGNORECASE):
                        errors.append(Finding(
                            "spec-standard-or-platform-leak",
                            f"Backlog specification '{rel_path}' contains standard/platform leak: '{desc}'. Tier 1 documents must be purely functional and platform-independent.",
                            location=rel_path,
                        ))

            # Code block integrity check (Mermaid block leakage & stray fences)
            lines = content.splitlines()
            in_code_block = False
            code_block_start = 0
            is_mermaid_block = False
            mermaid_fence_count = 0
            for idx, line in enumerate(lines, 1):
                stripped = line.strip()
                if stripped.startswith("```"):
                    if in_code_block:
                        in_code_block = False
                        if is_mermaid_block:
                            mermaid_fence_count += 1
                        is_mermaid_block = False
                    else:
                        in_code_block = True
                        code_block_start = idx
                        if stripped.startswith("```mermaid"):
                            is_mermaid_block = True
                            mermaid_fence_count += 1
                elif in_code_block and is_mermaid_block:
                    if MD_CONSTRUCTS_INVALID_IN_MERMAID.match(stripped):
                        errors.append(Finding(
                            "spec-markdown-construct-inside-mermaid-block",
                            f"Specification '{rel_path}' has non-mermaid content "
                            f"'{stripped[:60]}' inside an unclosed Mermaid block "
                            f"starting at line {code_block_start}.",
                            location=f"{rel_path}:{code_block_start}",
                        ))
                        in_code_block = False
                        is_mermaid_block = False
            if in_code_block:
                errors.append(Finding(
                    "spec-code-fence-must-be-closed",
                    f"Specification '{rel_path}' has an unclosed code block (or stray code fence) starting at line {code_block_start}.",
                    location=f"{rel_path}:{code_block_start}",
                ))
            if mermaid_fence_count % 2 != 0:
                errors.append(Finding(
                    "spec-mermaid-fence-count-must-be-even",
                    f"Specification '{rel_path}' has unbalanced mermaid fences ({mermaid_fence_count} opening/closing found). A closing ``` may be missing after a mermaid block.",
                    location=rel_path,
                ))
                        
        return errors
