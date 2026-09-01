import os
from typing import List
from .base import IValidator
from ..core.findings import Finding
from ..core.workspace import WorkspaceRepository

class ProfileScopingValidator(IValidator):
    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[str]:
        # Upstream compiler repository exemption: without an app_flutter/web_react
        # client codebase, an empty codebase is a configuration state, not a defect.
        if repo.is_upstream_compiler_repo() and not repo.has_configured_target_code_directories():
            print("Note: Upstream compiler repository mode. Skipping empty-codebase profile-scoping check.")
            return []

        rules = repo.get_codebase_rules()
        target_dirs = rules.target_directories
        
        flutter_dir_name = target_dirs.flutter
        flutter_dir = os.path.join(repo.workspace_dir, flutter_dir_name) if flutter_dir_name else None
        
        errors = []
        
        def find_files(directory, extensions, exclusions):
            if not directory or not os.path.exists(directory):
                return []
            matched = []
            for root, dirs, files in os.walk(directory):
                dirs[:] = [d for d in dirs if d not in exclusions]
                for file in files:
                    if any(file.endswith(ext) for ext in extensions):
                        matched.append(os.path.join(root, file))
            return matched
        
        flutter_rules = rules.flutter_rules
        flutter_files = find_files(flutter_dir, flutter_rules.file_extensions, set(flutter_rules.exclusions))
        
        if not flutter_files:
            return [Finding(
                "profile-scoping-requires-platform-sources",
                "Profile Compliance: Codebase is empty. No source files found for profile validation.",
                location=flutter_dir_name or "",
            )]

        if flutter_files:
            dart_contents = []
            for f in flutter_files:
                try:
                    with open(f, "r", encoding="utf-8", errors="ignore") as file:
                        dart_contents.append((f, file.read()))
                except Exception:
                    pass
                    
            for filepath, content in dart_contents:
                if "splitter" in filepath.lower() or "Splitter" in content:
                    if "Listener" not in content and "GestureDetector" not in content:
                        errors.append(Finding(
                            "flutter-splitter-requires-pointer-gesture-listener",
                            f"Profile Compliance: Flutter Splitter widget in '{os.path.basename(filepath)}' is missing pointer gesture event listeners (Listener or GestureDetector).",
                            location=os.path.basename(filepath),
                        ))
                        
        return errors
