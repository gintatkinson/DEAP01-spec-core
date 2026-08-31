import os
import re
from typing import List
from .base import IValidator
from ..core.findings import Finding
from ..core.workspace import WorkspaceRepository

class DependencyValidator(IValidator):
    def validate_epic_prerequisite_links(self, repo: WorkspaceRepository, **kwargs) -> List[str]:
        rules = repo.get_codebase_rules()
        backlog_dirs = rules.backlog_directories
        
        schema_dir = kwargs.get("schema_dir")
        if not schema_dir:
            schema_dir = os.path.join(repo.workspace_dir, backlog_dirs.schemas)
            
        epics_dir = os.path.join(repo.workspace_dir, backlog_dirs.epics)
        
        if not os.path.exists(schema_dir) or not os.path.exists(epics_dir):
            return []
            
        imports = set()
        for filename in os.listdir(schema_dir):
            filepath = os.path.join(schema_dir, filename)
            if os.path.isdir(filepath):
                continue
            ext = os.path.splitext(filename)[1].lower()
            if ext != ".yang":
                continue
                
            try:
                with open(filepath, "r", encoding="utf-8", errors="ignore") as f:
                    content = f.read()
                for m in re.finditer(r'\bimport\s+([a-zA-Z0-9_\-]+)', content):
                    imports.add(m.group(1))
            except Exception:
                pass
                
        schema_files = {os.path.splitext(f)[0] for f in os.listdir(schema_dir) if f.endswith(".yang")}
        project_imports = imports.intersection(schema_files)
        
        if not project_imports:
            return []
            
        errors = []
        for filename in sorted(os.listdir(epics_dir)):
            if not filename.endswith(".md"):
                continue
            filepath = os.path.join(epics_dir, filename)
            try:
                with open(filepath, "r", encoding="utf-8", errors="ignore") as f:
                    content = f.read()
            except Exception:
                continue
                
            for imp in sorted(project_imports):
                if re.search(r'\b' + re.escape(imp) + r'\b', content, re.IGNORECASE):
                    has_parent_link = bool(
                        re.search(r'Parent Epic.*\[.*?\]\(.*?\)', content, re.IGNORECASE) or 
                        re.search(r'\[.*?Parent Epic.*?\]\(.*?\)', content, re.IGNORECASE)
                    )
                    if not has_parent_link:
                        errors.append(Finding(
                            "epic-imported-schema-prerequisite-link-missing",
                            f"Epic '{filename}' references imported schema '{imp}' without linking to prerequisite parent Epic.",
                            location=imp,
                        ))
                        
        return errors

    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[str]:
        rules = repo.get_codebase_rules()
        backlog_dirs = rules.backlog_directories
        
        schema_dir = kwargs.get("schema_dir")
        if not schema_dir:
            schema_dir = os.path.join(repo.workspace_dir, backlog_dirs.schemas)
            
        epics_dir = os.path.join(repo.workspace_dir, backlog_dirs.epics)
        features_dir = os.path.join(repo.workspace_dir, backlog_dirs.features)
        
        errors = []
        
        if not os.path.exists(schema_dir):
            return []
            
        # 1. Collect all schema imports
        imports = set()
        for filename in os.listdir(schema_dir):
            filepath = os.path.join(schema_dir, filename)
            if os.path.isdir(filepath):
                continue
            ext = os.path.splitext(filename)[1].lower()
            if ext != ".yang":
                continue
                
            try:
                with open(filepath, "r", encoding="utf-8", errors="ignore") as f:
                    content = f.read()
                for m in re.finditer(r'\bimport\s+([a-zA-Z0-9_\-]+)', content):
                    imports.add(m.group(1))
            except Exception:
                pass
                
        # Filter imports to only keep those that exist as files in schema_dir
        schema_files = {os.path.splitext(f)[0] for f in os.listdir(schema_dir) if f.endswith(".yang")}
        project_imports = imports.intersection(schema_files)
        
        if project_imports:
            # 2. Scan all local spec files to see which dependencies they cover
            spec_contents = []
            spec_filenames = []
            
            def read_specs(directory):
                if not os.path.exists(directory):
                    return
                for filename in os.listdir(directory):
                    if not filename.endswith(".md"):
                        continue
                    filepath = os.path.join(directory, filename)
                    try:
                        with open(filepath, "r", encoding="utf-8", errors="ignore") as f:
                            content = f.read()
                        spec_contents.append(content)
                        spec_filenames.append(filename)
                    except Exception:
                        pass
                        
            read_specs(epics_dir)
            read_specs(features_dir)
            
            # 3. Verify each import dependency has at least one active specification file
            for dep in sorted(project_imports):
                covered = False
                for filename, content in zip(spec_filenames, spec_contents):
                    if dep.lower() in filename.lower():
                        covered = True
                        break
                    if re.search(r'\b' + re.escape(dep) + r'\b', content, re.IGNORECASE):
                        covered = True
                        break
                        
                if not covered:
                    errors.append(Finding(
                        "schema-import-dependency-unspecified",
                        f"Missing specification files for imported schema dependency '{dep}'. Please ensure the dependent specifications are kept in the workspace.",
                        location=dep,
                    ))

        # 4. Check Epic parent prerequisite links for imported schemas
        errors.extend(self.validate_epic_prerequisite_links(repo, **kwargs))
                
        return errors
