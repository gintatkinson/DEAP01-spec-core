"""
Validator that enforces profile compliance rules (e.g. UML traceability tags
/// Realises: [SpecName/ClassName], @immutable annotations on domain classes,
Result<T> return signatures, and DartDoc /// member documentation) across target codebase source files.
"""

import os
import re
from typing import List
from .base import IValidator
from ..core.findings import Finding
from ..core.workspace import WorkspaceRepository


class ProfileComplianceValidator(IValidator):
    """Validator that checks profile compliance and domain engineering standards in source code."""

    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[Finding]:
        """
        Scans target codebase directories for public Dart declarations to ensure:
        1. UML traceability tags (/// Realises: [SpecName/ClassName]) are present on public declarations.
        2. @immutable annotations are present on domain classes.
        3. Result<T> return signatures are used for fallible domain operations.
        4. DartDoc /// comments are present on public members.
        """
        errors = []
        workspace_dir = repo.workspace_dir
        rules = repo.get_codebase_rules()
        target_dirs = rules.target_directories if rules else None

        exclusions = {
            ".git", ".agents", ".pipeline", "build", "dist", "node_modules",
            ".dart_tool", "vendor", ".venv", "venv", "__pycache__", "coverage"
        }

        def get_source_files(target_dir_name: str, extensions: List[str]):
            if not target_dir_name:
                return []
            full_path = os.path.join(workspace_dir, target_dir_name)
            if not os.path.exists(full_path):
                return []
            matched = []
            for root, dirs, files in os.walk(full_path):
                dirs[:] = [d for d in dirs if d not in exclusions]
                for file in files:
                    if any(file.endswith(ext) for ext in extensions):
                        matched.append(os.path.join(root, file))
            return matched

        flutter_exts = getattr(rules.flutter_rules, "file_extensions", [".dart"]) if (rules and rules.flutter_rules) else [".dart"]
        flutter_dir = getattr(target_dirs, "flutter", None) if target_dirs else None
        
        flutter_files = get_source_files(flutter_dir, flutter_exts)

        if not flutter_files:
            candidate_dirs = ["app_flutter", "lib"]
            for cdir in candidate_dirs:
                cpath = os.path.join(workspace_dir, cdir)
                if os.path.exists(cpath):
                    for root, dirs, files in os.walk(cpath):
                        dirs[:] = [d for d in dirs if d not in exclusions]
                        for file in files:
                            if file.endswith(".dart"):
                                flutter_files.append(os.path.join(root, file))

        for filepath in flutter_files:
            rel_path = os.path.relpath(filepath, workspace_dir)
            norm_path = rel_path.replace("\\", "/")
            if "/test/" in norm_path or "/integration_test/" in norm_path or filepath.endswith("_test.dart"):
                continue

            try:
                with open(filepath, "r", encoding="utf-8", errors="ignore") as f:
                    lines = f.readlines()
            except Exception:
                continue

            if filepath.endswith(".dart"):
                self._check_dart_traceability_tags(lines, rel_path, errors)
                self._check_domain_immutable_annotations(lines, rel_path, errors)
                self._check_domain_result_signatures(lines, rel_path, errors)
                self._check_dart_member_docstrings(lines, rel_path, errors)

        return errors

    def _check_dart_traceability_tags(self, lines: List[str], rel_path: str, errors: List[Finding]):
        """Checks missing UML traceability tags on public class declarations in Dart files."""
        decl_pattern = re.compile(
            r'^\s*(?:@\w+(?:\([^)]*\))?\s*)*(?:abstract\s+|base\s+|final\s+|interface\s+|sealed\s+|mixin\s+)*'
            r'(?:class|enum|mixin|extension|typedef)\s+([A-Z]\w*)'
        )
        for idx, line in enumerate(lines):
            match = decl_pattern.match(line)
            if match:
                name = match.group(1)
                if name.startswith("_"):
                    continue

                has_traceability_tag = False
                prev_idx = idx - 1
                doc_lines = []
                while prev_idx >= 0:
                    prev_line = lines[prev_idx].strip()
                    if prev_line.startswith("///") or prev_line.startswith("/**") or prev_line.startswith("*") or prev_line.endswith("*/"):
                        doc_lines.append(prev_line)
                        prev_idx -= 1
                    elif prev_line.startswith("@"):
                        prev_idx -= 1
                        continue
                    elif not prev_line:
                        prev_idx -= 1
                        continue
                    else:
                        break

                full_doc = " ".join(doc_lines)
                if "Realises:" in full_doc:
                    has_traceability_tag = True

                if not has_traceability_tag:
                    errors.append(Finding(
                        "uml-traceability-tag-missing",
                        f"Missing UML traceability tag (/// Realises: [SpecName/ClassName]) for public declaration '{name}' at {rel_path}:{idx + 1}",
                        location=f"{rel_path}:{idx + 1}"
                    ))

    def _check_domain_immutable_annotations(self, lines: List[str], rel_path: str, errors: List[Finding]):
        """Checks missing @immutable annotations on public concrete domain classes."""
        norm_path = rel_path.replace("\\", "/")
        if "domain/" not in norm_path:
            return

        # Match concrete domain classes (excluding abstract classes/interfaces)
        class_pattern = re.compile(
            r'^\s*(?:base\s+|final\s+|sealed\s+)?class\s+([A-Z]\w*)'
        )
        for idx, line in enumerate(lines):
            match = class_pattern.match(line)
            if match:
                name = match.group(1)
                if name.startswith("_"):
                    continue

                has_immutable = False
                prev_idx = idx - 1
                while prev_idx >= 0:
                    prev_line = lines[prev_idx].strip()
                    if "@immutable" in prev_line:
                        has_immutable = True
                        break
                    elif prev_line.startswith("///") or prev_line.startswith("/**") or prev_line.startswith("*") or prev_line.endswith("*/") or prev_line.startswith("@") or not prev_line:
                        prev_idx -= 1
                        continue
                    else:
                        break

                if not has_immutable:
                    errors.append(Finding(
                        "domain-immutable-annotation-missing",
                        f"Missing @immutable annotation for public domain class '{name}' at {rel_path}:{idx + 1}",
                        location=f"{rel_path}:{idx + 1}"
                    ))

    def _check_domain_result_signatures(self, lines: List[str], rel_path: str, errors: List[Finding]):
        """Checks domain operation method signatures to ensure Result<T> return types."""
        norm_path = rel_path.replace("\\", "/")
        if "domain/" not in norm_path:
            return

        method_pattern = re.compile(
            r'^(?:  |\t)(?:Future(?:<[^>]+>)?|Stream(?:<[^>]+>)?|[A-Z]\w*(?:<[^>]+>)?)\s+([a-z]\w*)\s*\('
        )
        for idx, line in enumerate(lines):
            match = method_pattern.match(line)
            if match:
                name = match.group(1)
                if name.startswith("_") or name in ("toString", "noSuchMethod", "copyWith"):
                    continue
                if "get " in line or "set " in line or "operator" in line or "factory" in line or "const" in line:
                    continue

                if "Result<" not in line and "Result " not in line and "const Result." not in line:
                    errors.append(Finding(
                        "domain-result-signature-missing",
                        f"Missing Result<T> return signature for domain operation '{name}' at {rel_path}:{idx + 1}",
                        location=f"{rel_path}:{idx + 1}"
                    ))

    def _check_dart_member_docstrings(self, lines: List[str], rel_path: str, errors: List[Finding]):
        """Checks missing DartDoc comments on public members in Dart files."""
        # Only class-level members (indented by 0 or 2 spaces)
        member_pattern = re.compile(
            r'^(?:  |\t)?(?:final\s+|const\s+|static\s+|factory\s+|late\s+)*(?:[A-Z]\w*<[^>]+>|[A-Z]\w*|[a-z]\w*)\s+([a-z]\w*)\s*[\(;=]'
        )
        keywords = {
            "return", "if", "for", "while", "switch", "case", "break", "continue",
            "yield", "await", "var", "throw", "rethrow", "assert", "print", "else",
            "import", "export", "part"
        }

        for idx, line in enumerate(lines):
            stripped = line.strip()
            if stripped.startswith("@override") or (idx > 0 and "@override" in lines[idx-1]):
                continue

            match = member_pattern.match(line)
            if match:
                name = match.group(1)
                if name.startswith("_") or name in ("toString", "noSuchMethod", "copyWith", "hashCode") or name in keywords:
                    continue

                has_doc = False
                prev_idx = idx - 1
                while prev_idx >= 0:
                    prev_line = lines[prev_idx].strip()
                    if prev_line.startswith("///") or prev_line.startswith("/**") or prev_line.endswith("*/"):
                        has_doc = True
                        break
                    elif prev_line.startswith("@"):
                        prev_idx -= 1
                        continue
                    elif not prev_line:
                        prev_idx -= 1
                        continue
                    else:
                        break

                if not has_doc:
                    errors.append(Finding(
                        "public-member-docstring-missing",
                        f"Missing DartDoc /// for public member '{name}' at {rel_path}:{idx + 1}",
                        location=f"{rel_path}:{idx + 1}"
                    ))
