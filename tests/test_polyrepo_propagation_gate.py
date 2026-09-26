#!/usr/bin/env python3
"""
Automated Cross-Repository Integration Test and Polyrepo Rollout Gate Suite.

Remediates Issue #372:
"Missing cross-repository integration tests and automated propagation gates"

Asserts that scripts/install_pipeline.sh:
1. Correctly provisions a downstream GitLab sandbox repository with working
   dual-provider issue creation (`create_issue.sh` in GitLab mode).
2. Accurately passes `--provider gitlab` with `--domain-url` preserving GitHub
   upstream domain repositories and avoiding invalid GitLab URL synthesis.
3. Enforces downstream baseline verification readiness in GitLab mode.

Primary Commercial Toolchain Integration Context:
  MATLAB / Simulink / Stateflow / Embedded Coder (Model-Based Design, Control Law Synthesis,
  DO-178C C/SPARK Ada code generation).

Pure Schema-Driven Invariant:
  Zero hardcoded domain concepts. All test runs operate in isolated temporary directories
  outside the workspace repository and leave zero residual state.
"""

import json
import os
import shutil
import stat
import subprocess
import sys
import tempfile
import unittest

PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
INSTALL_SCRIPT = os.path.join(PROJECT_ROOT, "scripts", "install_pipeline.sh")
BASELINE_SCRIPT_NAME = "verify_downstream_baseline.py"
CREATE_ISSUE_REL_PATH = os.path.join("skills", "spec-orchestrator", "scripts", "create_issue.sh")


class PolyrepoSandboxHelper:
    """Helper methods for constructing isolated polyrepo test sandboxes."""

    @staticmethod
    def init_git_repo(repo_dir: str, remote_url: str = "https://gitlab.com/uas-defense/uav-011.git", branch: str = "main"):
        """Initializes a clean git repository with user config and origin remote."""
        subprocess.run(["git", "init", "-b", branch], cwd=repo_dir, check=True, capture_output=True)
        subprocess.run(["git", "config", "user.email", "test-runner@deap.local"], cwd=repo_dir, check=True, capture_output=True)
        subprocess.run(["git", "config", "user.name", "DEAP Test Runner"], cwd=repo_dir, check=True, capture_output=True)
        if remote_url:
            subprocess.run(["git", "remote", "add", "origin", remote_url], cwd=repo_dir, check=True, capture_output=True)

    @staticmethod
    def run_installer(*args, cwd=None):
        """Runs scripts/install_pipeline.sh with specified arguments."""
        cmd = ["bash", INSTALL_SCRIPT] + list(args)
        return subprocess.run(
            cmd,
            cwd=cwd or PROJECT_ROOT,
            capture_output=True,
            text=True
        )

    @staticmethod
    def install_mock_glab(bin_dir: str, log_file: str, body_capture_file: str = None):
        """Installs an executable mock glab CLI that logs invocations and simulates responses."""
        os.makedirs(bin_dir, exist_ok=True)
        mock_glab_path = os.path.join(bin_dir, "glab")
        body_cap = body_capture_file or os.path.join(os.path.dirname(log_file), "glab_body.md")
        script = f"""#!/usr/bin/env python3
import sys, json, os

log_file = r"{log_file}"
calls = []
if os.path.exists(log_file):
    try:
        with open(log_file, "r", encoding="utf-8") as f:
            calls = json.load(f)
    except Exception:
        calls = []
calls.append(sys.argv[1:])
with open(log_file, "w", encoding="utf-8") as f:
    json.dump(calls, f)

args = sys.argv[1:]
if len(args) >= 2 and args[0] == "issue" and args[1] == "list":
    if os.environ.get("MOCK_GLAB_AUTH_FAILURE"):
        sys.stderr.write("error: unauthenticated - please run glab auth login\\n")
        sys.exit(1)
    existing = os.environ.get("MOCK_GLAB_EXISTING")
    if existing:
        print(existing)
        sys.exit(0)
    sys.exit(0)

if len(args) >= 2 and args[0] == "label" and args[1] == "list":
    if os.environ.get("MOCK_GLAB_MISSING_LABEL"):
        sys.exit(0)
    print("101\\tfeature\\tFeature specifications\\t#0366d6")
    sys.exit(0)

if len(args) >= 2 and args[0] == "label" and args[1] == "create":
    sys.exit(0)

if len(args) >= 2 and args[0] == "issue" and args[1] == "create":
    body_capture = r"{body_cap}"
    for i, a in enumerate(args):
        if a == "--description-file" and i + 1 < len(args):
            desc_path = args[i + 1]
            if os.path.exists(desc_path):
                with open(desc_path, "r", encoding="utf-8") as src, open(body_capture, "w", encoding="utf-8") as dst:
                    dst.write(src.read())
        elif a == "--description" and i + 1 < len(args):
            with open(body_capture, "w", encoding="utf-8") as dst:
                dst.write(args[i + 1])
    print("https://gitlab.com/uas-defense/uav-011/-/issues/101")
    sys.exit(0)

if len(args) >= 2 and args[0] == "issue" and args[1] == "reopen":
    sys.exit(0)

if len(args) >= 2 and args[0] == "issue" and (args[1] == "note" or args[1] == "comment"):
    sys.exit(0)

sys.exit(0)
"""
        with open(mock_glab_path, "w", encoding="utf-8") as f:
            f.write(script)
        os.chmod(mock_glab_path, os.stat(mock_glab_path).st_mode | stat.S_IEXEC | stat.S_IXGRP | stat.S_IXOTH)
        return mock_glab_path

    @staticmethod
    def install_mock_gh(bin_dir: str, log_file: str):
        """Installs an executable mock gh CLI that logs any unintended calls."""
        os.makedirs(bin_dir, exist_ok=True)
        mock_gh_path = os.path.join(bin_dir, "gh")
        script = f"""#!/usr/bin/env python3
import sys, json, os

log_file = r"{log_file}"
calls = []
if os.path.exists(log_file):
    try:
        with open(log_file, "r", encoding="utf-8") as f:
            calls = json.load(f)
    except Exception:
        calls = []
calls.append(sys.argv[1:])
with open(log_file, "w", encoding="utf-8") as f:
    json.dump(calls, f)
sys.exit(0)
"""
        with open(mock_gh_path, "w", encoding="utf-8") as f:
            f.write(script)
        os.chmod(mock_gh_path, os.stat(mock_gh_path).st_mode | stat.S_IEXEC | stat.S_IXGRP | stat.S_IXOTH)
        return mock_gh_path

    @staticmethod
    def create_valid_spec_corpus(sandbox_dir: str, gitlab_base_url: str = "https://gitlab.com/uas-defense/uav-011"):
        """Creates a fully valid SysML model, Epic, and Feature specification in the sandbox."""
        schema_dir = os.path.join(sandbox_dir, "schema")
        epics_dir = os.path.join(sandbox_dir, "docs", "epics")
        features_dir = os.path.join(sandbox_dir, "docs", "features")

        os.makedirs(schema_dir, exist_ok=True)
        os.makedirs(epics_dir, exist_ok=True)
        os.makedirs(features_dir, exist_ok=True)

        # 1. SysML v2 AST Model
        schema_file = os.path.join(schema_dir, "model.sysml")
        with open(schema_file, "w", encoding="utf-8") as f:
            f.write("""package AutonomousUAS_SSOT {
    doc /* SSOT for Autonomous UAS Infrastructure Safety */
    part def FlightGuidanceComponent {
        doc /* Subsystem component container */
        attribute componentId : String;
    }
    part def FlightGuidanceComputer {
        doc /* Flight guidance and supervisory controller */
        attribute activeRouteId : String;
    }
}
""")

        # 2. Epic Specification
        epic_file = os.path.join(epics_dir, "epic-01-flight-guidance.md")
        with open(epic_file, "w", encoding="utf-8") as f:
            f.write(f"""---
title: "Flight Guidance Subsystem"
type: "epic"
package: "FlightGuidance"
generation_mode: "subagent"
version: "1.0"
date: "2026-09-26"
release_date: "2026-09-26"
spec_source: "Project Constitution"
---

# Epic: Flight Guidance Subsystem

## 1. Context
Flight guidance and supervisory trajectory control subsystem.

## 2. Requirements & Checklist
- [ ] #101 - [Flight Guidance Computer]({gitlab_base_url}/-/blob/main/docs/features/feat-01-flight-guidance-computer.md) Primary flight guidance computer

### Associated Use Cases & User Stories

#### Associated Use Cases
- [ ] #102 - [Trajectory Deconfliction]({gitlab_base_url}/-/blob/main/docs/use-cases/uc-01-trajectory-deconfliction.md) Flight trajectory deconfliction

#### Associated User Stories
- [ ] #103 - [Avoid Obstacle]({gitlab_base_url}/-/blob/main/docs/user-stories/us-01-avoid-obstacle.md) Emergency avoidance maneuver

## 3. Architecture

### Subsystem Capability Allocations
| Capability Name | Subsystem Package | Description / Objective |
| --- | --- | --- |
| AutonomousCollisionAvoidance | FlightGuidance | Executes real-time detect-and-avoid trajectory deconfliction |

### Subsystem Component Definition
```mermaid
classDiagram
    FlightGuidanceComponent "1" *-- "1" FlightGuidanceComputer : contains
    class FlightGuidanceComponent {{
        +String componentId
    }}
```

## System-Level UML Class Diagram
```mermaid
classDiagram
    FlightGuidanceComponent "1" *-- "1" FlightGuidanceComputer : contains
    class FlightGuidanceComponent {{
        +String componentId
    }}
    class FlightGuidanceComputer {{
        +String activeRouteId
    }}
```

## State Machine Definitions

## System State Machine Diagram
```mermaid
stateDiagram-v2
    [*] --> Standby
```

## 4. Operational Considerations
Real-time embedded execution.

## 5. Security & Governance
DO-178C Level A assurance.

## Specification Context
Normative schema definition.

## 6. Source References
Structural Schema: [model.sysml]({gitlab_base_url}/-/blob/main/schema/model.sysml) (Clause: 3.1)
""")

        # 3. Feature Specification
        feat_file = os.path.join(features_dir, "feat-01-flight-guidance-computer.md")
        with open(feat_file, "w", encoding="utf-8") as f:
            f.write(f"""---
title: "Flight Guidance Computer"
type: "feature"
interface_type: "backend"
generation_mode: "subagent"
version: "1.0"
date: "2026-09-26"
release_date: "2026-09-26"
spec_source: "Project Constitution"
schema_containers:
  - path: "AutonomousUAS_SSOT:AutonomousUAS_SSOT/FlightGuidanceComputer"
    node_type: container
---

# Feature: Flight Guidance Computer

## Parent Epic
- [ ] #100 - [Flight Guidance Subsystem]({gitlab_base_url}/-/blob/main/docs/epics/epic-01-flight-guidance.md) Subsystem parent

## Description
Flight guidance controller component synthesized via MATLAB / Simulink / Stateflow / Embedded Coder.

## UML Class Diagram
```mermaid
classDiagram
    FlightGuidanceComponent "1" *-- "1" FlightGuidanceComputer : contains
    class FlightGuidanceComponent {{
        +String componentId "[1]"
    }}
    class FlightGuidanceComputer {{
        +String activeRouteId "[1]"
    }}
```

## Interface Requirements

### 1. Test Data Shape
```json
{{
  "activeRouteId": "ROUTE_ALPHA"
}}
```

### 2. Validation & Constraints
- ThermalLimit: coreTemperature <= 85.0

### 3. Logical Operations & Interface Messages
- `+ExecuteManeuver(in targetHeading : Float)`: Executes maneuver.

### 4. Interactive Flow & States
- State transition to guidance active when route initialized.

### 5. Logical Exception States & Validation Failures
- Thermal throttling triggers on coreTemperature > 85.0.

## Given-When-Then Acceptance Criteria
- Given flight guidance computer in navigation mode, When obstacle detected, Then ExecuteManeuver commands trajectory change.

## Source References
Structural Schema: [model.sysml]({gitlab_base_url}/-/blob/main/schema/model.sysml) (Clause: 3.2)
- Realizes: [Epic 01 Flight Guidance](../epics/epic-01-flight-guidance.md)

## Logical UI & Interface Bindings
| Interface Channel | Category | Target Component / Handler | Target Container / Endpoint | Data Source Binding |
| --- | --- | --- | --- | --- |
| mcp | M2M API | MCPToolHandler | /guidance/controller | /AutonomousUAS_SSOT:AutonomousUAS_SSOT/FlightGuidanceComputer |
""")
        return feat_file


class TestPolyrepoGitLabProvisioning(unittest.TestCase):
    """Verifies that install_pipeline.sh provisions a downstream GitLab sandbox repository cleanly."""

    def setUp(self):
        self.assertTrue(os.path.isfile(INSTALL_SCRIPT), f"Installer script missing at {INSTALL_SCRIPT}")
        self.temp_dir = tempfile.TemporaryDirectory()
        self.sandbox_dir = self.temp_dir.name

    def tearDown(self):
        self.temp_dir.cleanup()

    def test_provision_downstream_gitlab_sandbox_structure(self):
        """Asserts scripts/install_pipeline.sh provisions complete pipeline infrastructure in GitLab sandbox."""
        PolyrepoSandboxHelper.init_git_repo(self.sandbox_dir, "https://gitlab.com/uas-defense/uav-011.git")

        proc = PolyrepoSandboxHelper.run_installer(
            self.sandbox_dir,
            "--provider", "gitlab",
            "--domain-url", "https://github.com/gintatkinson/DEAP-uas-infrastructure-safety.git"
        )
        self.assertEqual(proc.returncode, 0, f"Installer failed: {proc.stderr}\n{proc.stdout}")

        # Assert mandatory files and directories are provisioned
        expected_paths = [
            os.path.join(self.sandbox_dir, ".pipeline", "constitution.md"),
            os.path.join(self.sandbox_dir, ".pipeline", "ACTIVE_RULES_BUNDLE.md"),
            os.path.join(self.sandbox_dir, "README.md"),
            os.path.join(self.sandbox_dir, "CLAUDE.md"),
            os.path.join(self.sandbox_dir, "AGENTS.md"),
            os.path.join(self.sandbox_dir, "scripts", BASELINE_SCRIPT_NAME),
            os.path.join(self.sandbox_dir, CREATE_ISSUE_REL_PATH),
            os.path.join(self.sandbox_dir, "skills", "spec-orchestrator", "scripts", "reconcile_backlog.py"),
            os.path.join(self.sandbox_dir, ".gitignore"),
        ]
        for path in expected_paths:
            self.assertTrue(os.path.exists(path), f"Mandatory pipeline path missing: {path}")

        # Assert clean landing zone: .pipeline/upstream must NOT be present in downstream sandbox
        upstream_sentinel = os.path.join(self.sandbox_dir, ".pipeline", "upstream")
        self.assertFalse(os.path.exists(upstream_sentinel), "Downstream sandbox must not contain .pipeline/upstream")

        # Assert all 6 safety fixtures are provisioned
        safety_fixture_names = [
            "complete_stpa_matrix.md",
            "truncated_uca_matrix.md",
            "missing_guideword_matrix.md",
            "incomplete_osos.md",
            "proof_missing_derivation.md",
            "complete_proof.md"
        ]
        for fix_name in safety_fixture_names:
            fix_path = os.path.join(self.sandbox_dir, "tests", "fixtures", "safety", fix_name)
            self.assertTrue(os.path.isfile(fix_path), f"Safety fixture missing: {fix_path}")

        # Assert .gitignore exists and contains no OS metadata files
        gitignore_path = os.path.join(self.sandbox_dir, ".gitignore")
        with open(gitignore_path, "r", encoding="utf-8") as f:
            gitignore_content = f.read()
        self.assertIn(".DS_Store", gitignore_content)

    def test_downstream_baseline_readiness_gitlab_mode(self):
        """Asserts downstream baseline verification executes cleanly (exit code 0) in GitLab mode."""
        PolyrepoSandboxHelper.init_git_repo(self.sandbox_dir, "https://gitlab.com/uas-defense/uav-011.git")

        proc = PolyrepoSandboxHelper.run_installer(
            self.sandbox_dir,
            "--provider", "gitlab",
            "--domain-url", "https://github.com/gintatkinson/DEAP-uas-infrastructure-safety.git"
        )
        self.assertEqual(proc.returncode, 0, f"Installer failed: {proc.stderr}")

        # Execute scripts/verify_downstream_baseline.py in sandbox
        baseline_script = os.path.join(self.sandbox_dir, "scripts", BASELINE_SCRIPT_NAME)
        self.assertTrue(os.path.isfile(baseline_script), "verify_downstream_baseline.py missing from scripts/")

        env = dict(os.environ, DEAP_REPOSITORY_TYPE="UPSTREAM_SPEC_CORE_COMPILER")
        base_proc = subprocess.run(
            [sys.executable, baseline_script, "--no-domain", "."],
            cwd=self.sandbox_dir,
            env=env,
            capture_output=True,
            text=True
        )
        self.assertEqual(
            base_proc.returncode, 0,
            f"Downstream baseline verification failed in GitLab mode:\nSTDOUT:\n{base_proc.stdout}\nSTDERR:\n{base_proc.stderr}"
        )
        self.assertIn("Check 10 verified", base_proc.stdout)
        self.assertIn("Check 14 verified", base_proc.stdout)
        self.assertIn("Check 15 verified", base_proc.stdout)

    def test_downstream_baseline_fail_closed_when_rules_bundle_missing(self):
        """Asserts that downstream baseline verification fails closed if mandatory governance is missing."""
        PolyrepoSandboxHelper.init_git_repo(self.sandbox_dir, "https://gitlab.com/uas-defense/uav-011.git")

        proc = PolyrepoSandboxHelper.run_installer(self.sandbox_dir, "--provider", "gitlab")
        self.assertEqual(proc.returncode, 0, f"Installer failed: {proc.stderr}")

        # Remove the mandatory rules entrypoint
        rule_path = os.path.join(self.sandbox_dir, "rules", "sysml-ssot-completeness.md")
        if os.path.exists(rule_path):
            os.remove(rule_path)

        baseline_script = os.path.join(self.sandbox_dir, "scripts", BASELINE_SCRIPT_NAME)
        base_proc = subprocess.run(
            [sys.executable, baseline_script, "--no-domain", "."],
            cwd=self.sandbox_dir,
            capture_output=True,
            text=True
        )
        self.assertNotEqual(
            base_proc.returncode, 0,
            "Baseline verification should have failed closed when governance was missing"
        )


class TestPolyrepoDomainUrlPreservation(unittest.TestCase):
    """Verifies that --provider gitlab with --domain-url preserves GitHub domain remote URLs."""

    def setUp(self):
        self.temp_dir = tempfile.TemporaryDirectory()
        self.sandbox_dir = self.temp_dir.name

    def tearDown(self):
        self.temp_dir.cleanup()

    def test_gitlab_sandbox_with_github_domain_url_preserves_remote(self):
        """Asserts that passing --domain-url with --provider gitlab avoids invalid GitLab URL synthesis."""
        PolyrepoSandboxHelper.init_git_repo(self.sandbox_dir, "https://gitlab.com/uas-defense/uav-011.git")

        domain_remote = "https://github.com/gintatkinson/DEAP-uas-infrastructure-safety.git"
        proc = PolyrepoSandboxHelper.run_installer(
            self.sandbox_dir,
            "--provider", "gitlab",
            "--role", "domain-template",
            "--domain-url", domain_remote
        )
        self.assertEqual(proc.returncode, 0, f"Installer failed: {proc.stderr}")

        readme_path = os.path.join(self.sandbox_dir, "README.md")
        with open(readme_path, "r", encoding="utf-8") as f:
            readme_content = f.read()

        # Customer onboarding clone command must reference the explicit GitHub domain URL
        expected_clone = (
            f"git clone {domain_remote} ./.tmp-pipeline && "
            "bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline"
        )
        self.assertIn(expected_clone, readme_content, "Expected clone command with GitHub domain URL not found in README.md")

        # Zero occurrences of invalid synthesized gitlab.com domain template URLs
        invalid_synthesized = "gitlab.com/gintatkinson/DEAP-uas-infrastructure-safety"
        self.assertNotIn(
            invalid_synthesized,
            readme_content,
            f"Forbidden synthesized GitLab URL found in downstream README: {invalid_synthesized}"
        )

    def test_gitlab_sandbox_with_domain_name_preserves_github_host(self):
        """Asserts that specifying --domain-name with --provider gitlab resolves to GitHub domain template."""
        PolyrepoSandboxHelper.init_git_repo(self.sandbox_dir, remote_url=None)

        proc = PolyrepoSandboxHelper.run_installer(
            self.sandbox_dir,
            "--provider", "gitlab",
            "--role", "domain-template",
            "--domain-name", "DEAP-uas-infrastructure-safety"
        )
        self.assertEqual(proc.returncode, 0, f"Installer failed: {proc.stderr}")

        readme_path = os.path.join(self.sandbox_dir, "README.md")
        with open(readme_path, "r", encoding="utf-8") as f:
            readme_content = f.read()

        expected_clone = (
            "git clone https://github.com/gintatkinson/DEAP-uas-infrastructure-safety.git ./.tmp-pipeline && "
            "bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline"
        )
        self.assertIn(expected_clone, readme_content)
        self.assertNotIn("gitlab.com/gintatkinson/DEAP-uas-infrastructure-safety", readme_content)

    def test_gitlab_sandbox_with_custom_domain_forge_url(self):
        """Asserts that passing an arbitrary domain forge URL is preserved verbatim."""
        PolyrepoSandboxHelper.init_git_repo(self.sandbox_dir, "https://gitlab.com/uas-defense/uav-011.git")

        custom_url = "https://forge.internal.defense.gov/deap/domain-uas.git"
        proc = PolyrepoSandboxHelper.run_installer(
            self.sandbox_dir,
            "--provider", "gitlab",
            "--role", "domain-template",
            "--domain-url", custom_url
        )
        self.assertEqual(proc.returncode, 0, f"Installer failed: {proc.stderr}")

        readme_path = os.path.join(self.sandbox_dir, "README.md")
        with open(readme_path, "r", encoding="utf-8") as f:
            readme_content = f.read()

        self.assertIn(custom_url, readme_content)


class TestPolyrepoDualProviderIssueCreation(unittest.TestCase):
    """Verifies dual-provider issue creation execution in a provisioned downstream GitLab sandbox."""

    def setUp(self):
        self.temp_dir = tempfile.TemporaryDirectory()
        self.sandbox_dir = self.temp_dir.name
        self.bin_dir = os.path.join(self.sandbox_dir, "mock_bin")
        self.glab_log = os.path.join(self.sandbox_dir, "glab_calls.json")
        self.gh_log = os.path.join(self.sandbox_dir, "gh_calls.json")
        self.glab_body_capture = os.path.join(self.sandbox_dir, "glab_body_captured.md")

        PolyrepoSandboxHelper.init_git_repo(self.sandbox_dir, "https://gitlab.com/uas-defense/uav-011.git")

        # Provision downstream sandbox
        proc = PolyrepoSandboxHelper.run_installer(
            self.sandbox_dir,
            "--provider", "gitlab",
            "--domain-url", "https://github.com/gintatkinson/DEAP-uas-infrastructure-safety.git"
        )
        self.assertEqual(proc.returncode, 0, f"Installer failed: {proc.stderr}")

        # Install mock binaries
        PolyrepoSandboxHelper.install_mock_glab(self.bin_dir, self.glab_log, self.glab_body_capture)
        PolyrepoSandboxHelper.install_mock_gh(self.bin_dir, self.gh_log)

        # Create valid specification corpus in the sandbox
        self.feat_file = PolyrepoSandboxHelper.create_valid_spec_corpus(self.sandbox_dir)
        self.feature_title = "Flight Guidance Computer"

        # Commit files so git index and HEAD are consistent
        subprocess.run(["git", "add", "."], cwd=self.sandbox_dir, check=True, capture_output=True)
        subprocess.run(
            ["git", "commit", "--no-verify", "-m", "chore: initialize baseline corpus (refs #1)"],
            cwd=self.sandbox_dir,
            check=True,
            capture_output=True
        )

    def tearDown(self):
        self.temp_dir.cleanup()

    def _get_glab_calls(self):
        if not os.path.exists(self.glab_log):
            return []
        with open(self.glab_log, "r", encoding="utf-8") as f:
            return json.load(f)

    def _get_gh_calls(self):
        if not os.path.exists(self.gh_log):
            return []
        with open(self.gh_log, "r", encoding="utf-8") as f:
            return json.load(f)

    def _run_provisioned_create_issue(self, args, extra_env=None):
        create_script = os.path.join(self.sandbox_dir, CREATE_ISSUE_REL_PATH)
        env = os.environ.copy()
        env["PATH"] = f"{self.bin_dir}:{env.get('PATH', '')}"
        env["DEAP_REPOSITORY_TYPE"] = "UPSTREAM_SPEC_CORE_COMPILER"
        env.pop("TRACKER_PROVIDER", None)
        env.pop("PROVIDER", None)
        env.pop("GITLAB_CI", None)

        if extra_env:
            env.update(extra_env)

        cmd = ["bash", create_script] + list(args)
        return subprocess.run(
            cmd,
            cwd=self.sandbox_dir,
            env=env,
            capture_output=True,
            text=True
        )

    def test_create_issue_gitlab_provider_dispatch(self):
        """Asserts create_issue.sh in provisioned sandbox invokes glab with --description-file."""
        res = self._run_provisioned_create_issue([self.feat_file, "feature", self.feature_title])
        self.assertEqual(res.returncode, 0, f"create_issue.sh failed in GitLab sandbox:\nSTDOUT:\n{res.stdout}\nSTDERR:\n{res.stderr}")

        glab_calls = self._get_glab_calls()
        gh_calls = self._get_gh_calls()

        self.assertGreater(len(glab_calls), 0, "glab was not invoked by provisioned create_issue.sh")
        self.assertEqual(len(gh_calls), 0, "gh was unexpectedly invoked in GitLab sandbox")

        # Verify glab issue create arguments
        create_calls = [c for c in glab_calls if c[:2] == ["issue", "create"]]
        self.assertEqual(len(create_calls), 1, "glab issue create should have been executed once")
        call_args = create_calls[0]
        self.assertIn("--description-file", call_args, "glab issue create must use --description-file to prevent ARG_MAX overflow")
        self.assertNotIn("--description", call_args, "glab issue create must not use inline --description")
        self.assertIn("--title", call_args)
        self.assertIn(self.feature_title, call_args)

    def test_create_issue_gitlab_idempotency_prevents_duplicate(self):
        """Asserts create_issue.sh detects existing issue on GitLab and skips creation."""
        # GitLab TSV format: id<TAB>title<TAB>labels... (title is column 2)
        existing_tsv = f"101\t{self.feature_title}\tfeature\topen"
        res = self._run_provisioned_create_issue(
            [self.feat_file, "feature", self.feature_title],
            extra_env={"MOCK_GLAB_EXISTING": existing_tsv}
        )
        self.assertEqual(res.returncode, 0, f"create_issue.sh failed: {res.stderr}\n{res.stdout}")
        self.assertIn("[IDEMPOTENT] Issue #101 already carries the title", res.stdout)

        glab_calls = self._get_glab_calls()
        create_calls = [c for c in glab_calls if c[:2] == ["issue", "create"]]
        self.assertEqual(len(create_calls), 0, "glab issue create should NOT be called for existing issue")

    def test_create_issue_gitlab_link_expansion_uses_dash_blob(self):
        """Asserts create_issue.sh expands relative markdown links using GitLab /-/blob/ syntax."""
        res = self._run_provisioned_create_issue([self.feat_file, "feature", self.feature_title])
        self.assertEqual(res.returncode, 0, f"create_issue.sh failed: {res.stderr}\n{res.stdout}")

        self.assertTrue(os.path.isfile(self.glab_body_capture), "Captured glab body file was not written")
        with open(self.glab_body_capture, "r", encoding="utf-8") as f:
            captured_body = f.read()

        # In GitLab mode, relative markdown links should expand to /-/blob/
        self.assertIn("/-/blob/", captured_body, "GitLab body should expand relative links using /-/blob/ syntax")
        # Ensure GitHub /blob/ syntax is not used for GitLab repo links
        for line in captured_body.splitlines():
            if "gitlab.com" in line and "/blob/" in line:
                self.assertIn("/-/blob/", line, f"Found GitHub /blob/ syntax in GitLab link: {line}")


class TestPolyrepoRolloutGateContracts(unittest.TestCase):
    """Static and structural assertions ensuring polyrepo rollout gate invariants across installer and tooling."""

    def test_installer_polyrepo_contract_invariants(self):
        """Asserts installer script contains all required polyrepo parameters, flags, and cleanup rules."""
        with open(INSTALL_SCRIPT, "r", encoding="utf-8") as f:
            content = f.read()

        # Check options support
        self.assertIn("--provider", content)
        self.assertIn("--domain-url", content)
        self.assertIn("--domain-name", content)
        self.assertIn("--role", content)

        # Check active rules bundle generation
        self.assertIn("ACTIVE_RULES_BUNDLE.md", content)

        # Check exclusion of upstream compiler sentinels
        self.assertIn('rm -rf "$TARGET_DIR/.pipeline/upstream"', content)
        self.assertIn('rm -rf "$TARGET_DIR/.pipeline/diagnostics"', content)

        # Check safety integrity fixtures copy and verification
        self.assertIn("tests/fixtures", content)
        self.assertIn("complete_stpa_matrix.md", content)
        self.assertIn("truncated_uca_matrix.md", content)

        # Check OS metadata cleanup
        self.assertIn('.DS_Store', content)

    def test_create_issue_script_polyrepo_contract_invariants(self):
        """Asserts create_issue.sh adheres to dual-provider contracts and buffer safety invariants."""
        create_script_path = os.path.join(PROJECT_ROOT, CREATE_ISSUE_REL_PATH)
        with open(create_script_path, "r", encoding="utf-8") as f:
            content = f.read()

        # Environment export for title awk matching
        self.assertIn("export TITLE", content)

        # Buffer overflow prevention (Issue #374)
        self.assertIn('--description-file "$TMP_EXPANDED_BODY"', content)
        self.assertIn('--body-file "$TMP_EXPANDED_BODY"', content)

        # Dual-provider title column indexing (Issue #373)
        self.assertIn('$2 == ENVIRON["TITLE"]', content, "GitLab duplicate check must index column 2 (Title)")
        self.assertIn('$3 == ENVIRON["TITLE"]', content, "GitHub duplicate check must index column 3 (Title)")

        # Link expansion integration via reconcile_backlog
        self.assertIn("expand_relative_links_for_tracker", content)

        reconcile_script_path = os.path.join(PROJECT_ROOT, "skills", "spec-orchestrator", "scripts", "reconcile_backlog.py")
        with open(reconcile_script_path, "r", encoding="utf-8") as f:
            rec_content = f.read()
        self.assertIn("/-/blob/", rec_content, "GitLab relative link expansion must use /-/blob/")
        self.assertIn("/blob/", rec_content, "GitHub relative link expansion must use /blob/")


if __name__ == "__main__":
    unittest.main()
