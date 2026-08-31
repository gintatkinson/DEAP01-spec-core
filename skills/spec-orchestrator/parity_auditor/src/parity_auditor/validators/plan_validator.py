import json
import os
import re
from typing import List
from .base import IValidator
from ..core.findings import Finding
from ..core.workspace import WorkspaceRepository


class PlanValidator(IValidator):
    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[str]:
        digest_path = kwargs.get(
            "digest_path",
            os.path.join(repo.workspace_dir, ".pipeline", "schema-digest.json")
        )
        plan_path = kwargs.get(
            "plan_path",
            os.path.join(repo.workspace_dir, "implementation_plan.md")
        )

        errors = []

        if not os.path.exists(digest_path):
            return []

        try:
            with open(digest_path, "r", encoding="utf-8") as f:
                digest_data = json.load(f)
        except Exception as e:
            return [Finding("schema-digest-invalid-json", f"Failed to parse schema digest: {e}")]

        digest_nodes = set(digest_data.get("schema_nodes", []))
        if not digest_nodes:
            return []

        if not os.path.exists(plan_path):
            return [Finding("implementation-plan-missing", f"implementation_plan.md does not exist to verify schema node coverage.")]

        try:
            with open(plan_path, "r", encoding="utf-8") as f:
                plan_content = f.read()
        except Exception as e:
            return [Finding("implementation-plan-unreadable", f"Failed to read implementation_plan.md: {e}")]

        # Extract mapped schema nodes from plan
        mapped_nodes = set()
        for node in digest_nodes:
            if re.search(r'\b' + re.escape(node) + r'\b', plan_content):
                mapped_nodes.add(node)

        unmapped_nodes = digest_nodes - mapped_nodes
        if unmapped_nodes:
            coverage = len(mapped_nodes) / len(digest_nodes) * 100.0
            missing_str = ", ".join(sorted(list(unmapped_nodes))[:10])
            errors.append(Finding(
                "plan-schema-coverage-must-be-100-percent",
                f"Plan schema coverage is {coverage:.1f}% (< 100%). Unmapped schema nodes ({len(unmapped_nodes)}): {missing_str}",
                location=os.path.basename(plan_path)
            ))

        return errors
