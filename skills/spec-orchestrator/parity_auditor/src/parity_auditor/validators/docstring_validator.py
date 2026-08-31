"""
Validator that enforces public member documentation (docstrings / DartDoc / JSDoc)
presence across target codebase source files.
"""

import os
import re
from typing import List
from .base import IValidator
from ..core.findings import Finding
from ..core.workspace import WorkspaceRepository


class DocstringValidator(IValidator):
    """Validator that checks presence of docstrings on public members in source code."""

    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[str]:
        """
        Scans target codebase directories for public classes, interfaces, methods,
        functions, and public properties to ensure they have docstrings.

        - Dart: DartDoc /// or /** */
        - Python: Docstrings ''' or \"\"\" or # comments
        - JS/TS: JSDoc /** */ or ///
        """
        errors = []
        workspace_dir = repo.workspace_dir
        rules = repo.get_codebase_rules()
        target_dirs = rules.target_directories

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

        flutter_exts = getattr(rules.flutter_rules, "file_extensions", [".dart"]) if rules.flutter_rules else [".dart"]
        react_exts = getattr(rules.react_rules, "file_extensions", [".ts", ".tsx", ".js", ".jsx"]) if rules.react_rules else [".ts", ".tsx", ".js", ".jsx"]
        python_exts = getattr(rules.python_rules, "file_extensions", [".py"]) if rules.python_rules else [".py"]

        flutter_files = get_source_files(getattr(target_dirs, "flutter", None), flutter_exts)
        react_files = get_source_files(getattr(target_dirs, "react", None), react_exts)
        python_files = get_source_files(getattr(target_dirs, "python", None), python_exts)

        all_files = flutter_files + react_files + python_files

        if not all_files:
            candidate_dirs = ["app_flutter", "web_react", "src"]
            for cdir in candidate_dirs:
                cpath = os.path.join(workspace_dir, cdir)
                if os.path.exists(cpath):
                    for root, dirs, files in os.walk(cpath):
                        dirs[:] = [d for d in dirs if d not in exclusions]
                        for file in files:
                            if file.endswith((".dart", ".ts", ".tsx", ".js", ".jsx", ".py")):
                                all_files.append(os.path.join(root, file))

        for filepath in all_files:
            rel_path = os.path.relpath(filepath, workspace_dir)
            try:
                with open(filepath, "r", encoding="utf-8", errors="ignore") as f:
                    lines = f.readlines()
            except Exception:
                continue

            if filepath.endswith(".dart"):
                self._check_dart_docstrings(lines, rel_path, errors)
            elif filepath.endswith((".ts", ".tsx", ".js", ".jsx")):
                self._check_ts_docstrings(lines, rel_path, errors)
            elif filepath.endswith(".py"):
                self._check_py_docstrings(lines, rel_path, errors)

        return errors

    def _check_dart_docstrings(self, lines: List[str], rel_path: str, errors: List[str]):
        """Checks missing DartDoc comments on public declarations in Dart files."""
        decl_pattern = re.compile(
            r'^\s*(?:@\w+(?:\([^)]*\))?\s*)*(?:abstract\s+|base\s+|final\s+|interface\s+|sealed\s+|mixin\s+)*'
            r'(?:class|enum|mixin|extension|typedef)\s+([A-Z]\w*)'
        )
        for idx, line in enumerate(lines):
            match = decl_pattern.match(line)
            if match:
                name = match.group(1)
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
                        f"Missing DartDoc /// for public declaration '{name}' at {rel_path}:{idx + 1}",
                        location=f"{rel_path}:{idx + 1}"
                    ))

    def _check_ts_docstrings(self, lines: List[str], rel_path: str, errors: List[str]):
        """Checks missing JSDoc comments on exported declarations in TS/JS files."""
        export_pattern = re.compile(
            r'^\s*export\s+(?:default\s+)?(?:class|interface|function|enum|type|const|let|var)\s+([A-Za-z0-9_]+)'
        )
        for idx, line in enumerate(lines):
            match = export_pattern.match(line)
            if match:
                name = match.group(1)
                has_doc = False
                prev_idx = idx - 1
                while prev_idx >= 0:
                    prev_line = lines[prev_idx].strip()
                    if prev_line.startswith("/**") or prev_line.startswith("///") or prev_line.endswith("*/"):
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
                        f"Missing JSDoc /** */ for exported declaration '{name}' at {rel_path}:{idx + 1}",
                        location=f"{rel_path}:{idx + 1}"
                    ))

    def _check_py_docstrings(self, lines: List[str], rel_path: str, errors: List[str]):
        """Checks missing docstrings on public classes/functions in Python files."""
        decl_pattern = re.compile(r'^\s*(?:async\s+)?(?:def|class)\s+([A-Za-z0-9_]+)')
        for idx, line in enumerate(lines):
            match = decl_pattern.match(line)
            if match:
                name = match.group(1)
                if name.startswith("_") and not name.startswith("__"):
                    continue
                
                has_doc = False
                next_idx = idx + 1
                while next_idx < len(lines):
                    next_line = lines[next_idx].strip()
                    if not next_line:
                        next_idx += 1
                        continue
                    if next_line.startswith(('"""', "'''", 'r"""', "r'''", 'f"""', "f'''")):
                        has_doc = True
                    break
                
                if not has_doc:
                    prev_idx = idx - 1
                    while prev_idx >= 0 and not lines[prev_idx].strip():
                        prev_idx -= 1
                    if prev_idx >= 0 and lines[prev_idx].strip().startswith("#"):
                        has_doc = True
                
                if not has_doc:
                    errors.append(Finding(
                        "public-member-docstring-missing",
                        f"Missing docstring for public declaration '{name}' at {rel_path}:{idx + 1}",
                        location=f"{rel_path}:{idx + 1}"
                    ))
