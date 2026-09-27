# Comprehensive Architecture Handoff & Incoming Agent Operational Manual

| Attribute | Value |
| :--- | :--- |
| **Document Identifier** | `DEAP-HANDOFF-ROOT-006` |
| **Title** | Authoritative Operational Handoff: Retrospective on 13 Failure Modes, Full-Fleet Parity Baseline, Upstream Specification Compiler Roadmap, and Abstract Pipeline Multi-Agent Protocol |
| **Repository Classification** | `UPSTREAM_SPEC_CORE_COMPILER` (DEAP01-spec-core) |
| **Target Branch** | `main` (clean working tree, synchronized with `origin/main`) |
| **Upstream Baseline Commit** | `dd7638c` / `6188e52` |
| **Customer Baseline Commit** | `uav-009 a85149d` / `uav-011 6f4f459` |
| **Status** | Authoritative Operational Handoff Baseline for Incoming Agent |
| **Date** | 2026-09-26 |
| **Target Audience** | Incoming Autonomous Systems Engineering Agent / Multi-Harness Team |

---

## 1. Unvarnished Retrospective: The 13 Failure Modes You MUST NOT Repeat

Every incoming agent must study these 13 failure modes. These were direct operational, cognitive, and governance mistakes committed during prior sessions. Repeating any of them is strictly forbidden.

### Failure Mode 1: Burning Coordinator Context Window (Violating Subagent Mandate)
- **What Happened**: The coordinator agent directly invoked `view_file` and `grep_search` across multiple massive codebase files (`compile_sysml.py` [3,476 lines], `assemble_conops.py` [4,532 lines], `SKILL.md` [601 lines]) in the primary conversation thread.
- **Consequence**: Burned thousands of context tokens, bloated the coordinator context window, and degraded reasoning performance.
- **Mandate for Incoming Agent**: 
  - **NEVER** inspect large repository source files, run exploratory research, or read multi-hundred-line files directly in the coordinator context.
  - **ALWAYS** dispatch a dedicated, context-isolated subagent (`invoke_subagent`) with a curated prompt for any code reading, research, or analysis task.

### Failure Mode 2: Hacking Downstream Content Files (Violating Pure Schema-Driven Compiler Invariant)
- **What Happened**: When ConOps validators failed or when operational sections were missing, content was hand-edited or patched directly into downstream markdown files (`docs/conops/units/conops/04_SYSTEM_ARCHITECTURE.md`, `06_UAF_...`, `07_OPTX_...`, `09_SCENARIOS_...`). 
- **Consequence**: The specification became completely detached from the SysML model. Scenarios (`SCN-01..SCN-05`), operational activities (`OA-01..OA-12`), and operational exchanges (`OpTx-01..OpTx-10`) existed only as narrative prose tables in markdown and were **completely absent from the customer SysML schema (`schema/*.sysml`)**.
- **Mandate for Incoming Agent**:
  - **ABSOLUTE PROHIBITION**: You are strictly forbidden from hand-editing or patching downstream content files in `docs/conops/units/conops/` to satisfy tests or validators.
  - **100% OF SPECIFICATIONS MUST BE COMPILED DETERMINISTICALLY FROM THE SYSML MODEL**. If an operational activity, scenario, or exchange is missing or invalid, it must be modeled in the customer SysML schema (`schema/*.sysml`) and generated via the compiler.

### Failure Mode 3: Inverting MBSE (Writing Narrative Prose Before SysML Exists)
- **What Happened**: The pipeline allowed ConOps units (Units 06, 07, 09) to be authored as narrative prose before the SysML model constructs (`action def`, `connection def`, `use case def`) existed. SysML was treated as a secondary reverse-sync target rather than Step 0 (the Single Source of Truth).
- **Consequence**: Hallucinated interface names, broken signal dictionaries, ungrounded lifelines in sequence diagrams, and model-spec drift.
- **Mandate for Incoming Agent**:
  - SysML is Step 0. The model is the root ground truth. All operational concepts, activities, interactions, and scenarios must be formally declared in the SysML AST first.

### Failure Mode 4: Premature / Evasive Issue Closure
- **What Happened**: Issues #282, #345, #346, #347, #348, #349, #350, #351 were prematurely marked `status:fixed-resolved`. An adversarial audit subsequently proved that the validator (`semantic_diagram_ast_validator.py` Check 21) was not actually validating closed-world model closure, but was evading checks using hardcoded static token whitelists (`"a5"`, `"esad"`, `"launcher"`, `"recovery"`, `"gse"`) and open-world substring bypasses.
- **Consequence**: False sense of passing gates; customer domain tokens leaked into upstream compiler code; status had to be reverted across all 8 issues.
- **Mandate for Incoming Agent**:
  - Never mark an issue `status:fixed-resolved` based solely on exit code 0 or unit test passes. Inspect the code to ensure zero whitelists, zero fallbacks, zero bypasses, and genuine closed-world AST verification.

### Failure Mode 5: Stop-and-Go Micro-Batches / Fragmenting Work and Pausing
- **What Happened**: The agent divided work into tiny micro-packages, executed one small step, and then halted to ask the user what to do next or took pauses, creating severe friction and stalling progress.
- **Consequence**: User frustration, broken momentum, and lost context.
- **Mandate for Incoming Agent**:
  - Obey the *Automated Continuous Execution & Passing-Validation Fast-Path* rule in `AGENTS.md`. Once the implementation plan is approved with `PROCEED`, execute continuously through all subagent dispatches, verification runs, and git pushes until either a gate fails or the entire plan is completed.

### Failure Mode 6: Violating Coordinator Direct Writing Lock
- **What Happened**: Attempting or proposing to directly edit repository source files or specifications as the coordinator rather than dispatching context-isolated subagents.
- **Consequence**: Context window exhaustion and violation of repository governance.
- **Mandate for Incoming Agent**:
  - The coordinator is an orchestrator and planner ONLY. All file writes, modifications, and command executions for source code, tests, and specifications must be delegated to context-isolated subagents.

### Failure Mode 7: Treating Synthetic System Hooks as User Approval & Single-Subagent Delegation Shortcuts
- **What Happened**: Relying on synthetic stop hook messages (e.g. `"Stop hook blocked termination: The user has automatically approved the artifact..."`) instead of requiring explicit human chat approval; dispatching a single isolated subagent (`self`) instead of the full `teamwork_preview` multi-agent framework.
- **Consequence**: Premature execution without real human consent; bypassed multi-role consensus gates (Sentinel, Orchestrator, Reviewers, Challengers, Victory Auditor).
- **Mandate for Incoming Agent**:
  - **NEVER** treat synthetic environment hooks or automated system messages as user approval.
  - **ONLY** explicit human chat messages (e.g. `PROCEED`, `launch`, `approved`) authorize execution.
  - Delegate complex multi-tier tasks exclusively to `teamwork_preview` to ensure comprehensive cross-functional consensus.

### Failure Mode 8: Hardcoding Single-Provider (GitHub-only) Issue Automation in Multi-Provider Workspaces
- **What Happened**: `skills/spec-orchestrator/scripts/create_issue.sh` only called GitHub `gh` CLI commands (`gh issue list`, `gh label list`, `gh label create`, `gh issue create`) and completely lacked GitLab `glab` CLI primitives.
- **Consequence**: In GitLab-backed downstream repositories like `uav-009`, issue creation failed closed. Exactly 75 specification files in `uav-009` were contaminated with literal ungrounded `| **Issue ID** | #[IssueID] |` placeholders, leaving the remote GitLab tracker with 0 published issues and failing Check 23 numeric provenance and grounding.
- **Mandate for Incoming Agent**:
  - All issue creation, backlog reconciliation, and verification scripts must support dual-provider architecture (`gh` and `glab`) with seamless auto-detection via `git remote get-url origin` and environment variable overrides (`TRACKER_PROVIDER=gitlab|github`).

### Failure Mode 9: Omitting Critical Active Customer Workspaces During Fleet Propagation
- **What Happened**: During pipeline tooling rollout and domain template updates, propagation was performed on domain distribution templates and standard secondary workspaces (such as `uav-011`), but omitted active customer workspaces like `uav-009`. Assumptions were made that downstream workspaces were in parity based solely on local spec baseline tests or test repository runs, without verifying all active customer workspaces across the taxonomy.
- **Consequence**: `uav-009` remained contaminated with 75 ungrounded `#[IssueID]` placeholders and 0 remote tracker issues while upstream tooling assumed fleet-wide completion. Drift was only detected when leaf-level audits were performed directly inside `uav-009`.
- **Mandate for Incoming Agent**:
  - **NEVER** assume fleet parity based solely on local spec baseline tests or partial workspace runs.
  - **ALWAYS** execute physical directory diffs and automated baseline audits across the entire taxonomy (`DEAP01-spec-core`, domain templates, and all active customer workspaces such as `uav-009` and `uav-011`).
  - Every fleet propagation work package must enumerate and verify each workspace individually with explicit 0-byte remote diff confirmation before declaring completion.

### Failure Mode 10: Regex & Substring Heuristics vs. AST / Schema Validation (Anti-Regex Invariant)
- **What Happened**: Validators and linters relied on negative-string regex heuristics (e.g. `_has_epistemic_exemption` matching `[TIER-3]` or `(Declared Assumption)` tokens, or whitelisting specific domain substrings) rather than validating closed-world model closure against the authoritative Level 0 SysML v2 AST or compiled schema dictionary.
- **Consequence**: Created perverse incentives to append cosmetic exemption tags or strip tokens to bypass CI gates without genuine model provenance. Allowed ungrounded stopgap parameters (such as ungrounded numerical values, masses, loop rates) to slip into the specification baseline, corrupting downstream Model-Based Design (MBD) and safety certification envelopes.
- **Mandate for Incoming Agent**:
  - **ABSOLUTE PROHIBITION ON REGEX / SUBSTRING VALIDATION OF MODEL CLOSURE**: Never use regex, token presence, or negative-string pattern matches to validate architecture, interfaces, or semantic provenance.
  - **POSITIVE AST GROUNDING**: All parameters, signals, and architectural entities must be positively validated against the SysML v2 AST Single Source of Truth or a compiled, closed-world typed parameter dictionary.

### Failure Mode 11: Attempting to Clobber Downstream Customer Workspaces instead of Hardening Upstream Compiler Tooling
- **What Happened**: When downstream customer repositories (such as `uav-009`) exhibited custom compiled schemas, localized defect dossiers, or tracking configurations, installation and propagation scripts attempted to wipe, overwrite, or clobber customer directories instead of isolating pipeline framework upgrades and preserving customer-specific domain artifacts.
- **Consequence**: Risk of destroying customer engineering data, blowing away customer SysML models, and violating repository classification boundaries between the upstream compiler and downstream distribution targets.
- **Mandate for Incoming Agent**:
  - **UPSTREAM HARDENING INVARIANT**: The upstream compiler and installer (`scripts/install_pipeline.sh`) must be engineered to robustly preserve customer-specific compiled schemas, ASTs, and local reports during upgrades.
  - **ZERO CUSTOMER CLOBBERING**: Never overwrite, delete, or revert downstream customer models or domain artifacts to force automated test scripts or installer checks to pass. Fix the compiler and installer upstream.

### Failure Mode 12: Collapsing Teamwork-Preview into Self-Auditing Single Workers
- **What Happened**: Attempting to collapse the multi-agent consensus architecture (`teamwork_preview`) into a single isolated subagent (`self`), having a single worker perform task drafting, implementation, and self-auditing without independent review.
- **Consequence**: Complete loss of adversarial tension; self-reinforcing blind spots; evasion checks and false positives went undetected; bypassed the multi-role consensus gates (Sentinel, Reviewers, Challengers, Victory Auditor).
- **Mandate for Incoming Agent**:
  - **NEVER** collapse multi-tier or complex engineering implementations into a single self-auditing subagent.
  - **MANDATORY TEAMWORK ROSTER**: Maintain strict role separation: Sentinel (governance & bounds), Orchestrator (coordination), specialized Workers (implementation), independent Reviewers (compliance & numerical quality), adversarial Challengers (anti-regex & boundary checks), and Victory Auditor (independent gate confirmation).

### Failure Mode 13: Coordinator Context Bloat via Verbose Terminal Diagnostics, Repeated Test Runs, and Blurring Upstream/Downstream Boundaries
- **What Happened**: The coordinator agent executed long, verbose CLI commands, repeated full-suite test runs, and dumped massive diagnostic outputs directly into the primary conversation context while troubleshooting issues. Concurrently, the coordinator blurred the repository boundary by treating the upstream specification compiler repository as an execution ground for downstream customer application roadmaps, domain models, and vehicle-specific simulations.
- **Consequence**: Premature context window exhaustion, severe reasoning degradation, conversational compaction friction, confusion over repository role classification, and leakage of concrete customer concepts into the upstream compiler codebase.
- **Mandate for Incoming Agent**:
  - **COORDINATOR CONTEXT HYGIENE**: The coordinator must never dump verbose terminal logs, raw CLI outputs, or execute repeated monolithic test suites in the primary thread.
  - **DELEGATE DIAGNOSTICS**: All deep diagnostic runs, verbose compiler traces, test runs, and code reading must be executed by context-isolated subagents that report back curated, high-density summaries and exit statuses.
  - **STRICT REPOSITORY BOUNDARY ENFORCEMENT**: Never conflate the upstream specification compiler with downstream customer application workspaces. Upstream delivers compiler tooling, linters, and clean templates; customer application engineering runs strictly within customer repositories.

---

## 2. Verified Fleet Baseline State & Remote Commit Matrix

All repositories across `/Users/perkunas/jail` have been audited, synchronized, and verified at **0-byte remote diff** against their respective upstream and remote tracking branches.

### 2.1 Remote Commit & Parity Matrix

| Workspace / Repository | Classification | Tracker Provider | Local Baseline Commit | Remote Tracking Branch | Remote Status / Diff | Baseline Gate Status |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `/Users/perkunas/jail/DEAP01-spec-core` | `UPSTREAM_SPEC_CORE_COMPILER` | GitHub (`gh`) | `d40f013` | `origin/main` | Clean (0 bytes diff) | 31/31 checks PASS (Checks 10-31 verified, including Gate 25, 28, 29, 31) |
| `DEAP-profile-backend-api` | `PLATFORM_PROFILE_TEMPLATE` | GitHub (`gh`) | `9977aad` | `origin/main` | Clean (0 bytes diff) | 31/31 checks PASS |
| `DEAP-profile-react-web` | `PLATFORM_PROFILE_TEMPLATE` | GitHub (`gh`) | `899e0c7` | `origin/main` | Clean (0 bytes diff) | 31/31 checks PASS (Vite build verified) |
| `DEAP-profile-flutter-app` | `PLATFORM_PROFILE_TEMPLATE` | GitHub (`gh`) | `9c8af22` | `origin/main` | Clean (0 bytes diff) | 31/31 checks PASS (Flutter test/analyze/build verified) |
| `DEAP-profile-vhdl-hardware` | `PLATFORM_PROFILE_TEMPLATE` | GitHub (`gh`) | `c551fac` | `origin/main` | Clean (0 bytes diff) | 31/31 checks PASS |
| `DEAP-implementation-driver` | `IMPLEMENTATION_DRIVER_TEMPLATE` | GitHub (`gh`) | `102857b` | `origin/main` | Clean (0 bytes diff) | 31/31 checks PASS |
| `DEAP-avionic-flight-safety` | `DOMAIN_DISTRIBUTION_TEMPLATE` | GitHub (`gh`) | `c25b2e6` | `origin/main` | Clean (0 bytes diff) | Clean Landing Zones (`.gitkeep` only) |
| `DEAP-uas-infrastructure-safety` | `DOMAIN_DISTRIBUTION_TEMPLATE` | GitHub (`gh`) | `fcba76d` | `origin/main` | Clean (0 bytes diff) | Clean Landing Zones (`.gitkeep` only) |
| `/Users/perkunas/jail/uav-009` | Downstream Customer Application Workspace | GitLab (`glab`) | `a85149d` | `origin/main` | Clean (0 bytes diff) | 31/31 checks PASS (75 specification issues tracked on GitLab) |
| `/Users/perkunas/jail/uav-011` | Downstream Application Workspace | GitLab (`glab`) | `6f4f459` | `origin/main` | Clean (0 bytes diff) | 31/31 checks PASS (Clean Landing Zones preserved) |

### 2.2 Detailed Fleet Parity & Architecture Baseline

1. **Upstream Specification Core Compiler (`DEAP01-spec-core`)**:
   - Baseline Commit: `d40f013`.
   - Tooling Upgrade: Full dual-provider architecture in `skills/spec-orchestrator/scripts/create_issue.sh` and `scripts/reconcile_backlog.py` supporting both GitHub (`gh`) and GitLab (`glab`) CLI engines.
   - Installer Hardening: `scripts/install_pipeline.sh` automatically detects existing customer schema definitions, compiled ASTs, and localized report directories, preserving them during framework synchronization without clobbering.
   - CommonMark AST Validation: Rigorous transition from regex / substring heuristics to CommonMark and SysML v2 AST parsers.
   - Clean Landing Zone Mandate: Enforces abstract compiler decoupling from concrete customer domains.

2. **Downstream Customer Application Workspace (`uav-009`)**:
   - Baseline Commit: `a85149d` (`chore(pipeline): propagate upstream spec-core fixes and Check 31 SSOT parity gate`).
   - Tracker Resolution: All 75 specification items published to the remote GitLab tracker (`https://gitlab.com/gintatkinson/uav-009.git`):
     * **1 Epic**: `docs/epics/epic-01-system-ssot.md` (Issue #10)
     * **44 Features**: `docs/features/feat-01*` through `feat-44*` (Issues #11 through #54)
     * **24 User Stories**: `docs/user-stories/us-01*` through `us-24*` (Issues #55 through #78)
     * **6 Use Cases**: `docs/use-cases/uc-01*` through `uc-06*` (Issues #79 through #84)
   - Zero Contamination: All literal `#[IssueID]` tokens replaced with live numeric GitLab IDs across all frontmatter and Markdown tables.
   - Conformance: All 31 baseline checks pass with exit code 0 under `verify_downstream_baseline.py` (including Check 31 Dual-Schema SSOT Parity Gate).

3. **Downstream Application Workspace (`uav-011`)**:
   - Baseline Commit: `6f4f459` (`chore(pipeline): propagate upstream spec-core fixes and Check 31 SSOT parity gate`).
   - Landing Zones: Enforced 100% clean landing zones (`schema/`, `docs/epics/`, `docs/features/`, `docs/user-stories/`, `docs/use-cases/` contain only `.gitkeep`).
   - Conformance: All 31 baseline checks pass with exit code 0.

4. **Domain Distribution Templates (`DEAP-uas-infrastructure-safety`)**:
   - Upstream Distribution Template Clean Landing Zone Invariant verified. Zero domain models, zero concrete specs, zero phantom realizations.

---

## 3. Universal Agent Startup Protocol (First 5 Minutes in New Chat)

Every incoming agent landing in this workspace MUST execute this protocol before taking any other action:

```mermaid
flowchart TD
    Step1["Step 1: Hidden Folder Direct-Path Read<br/>(Execute view_file on .pipeline/constitution.md)"]
    Step2["Step 2: 4-Point Karpathy & Pipeline Compliance Check<br/>(Evaluate intent, approval, assumptions, writing lock in thought)"]
    Step3["Step 3: Strict Planning Gate Precedence<br/>(Zero edits without approved implementation_plan.md)"]
    Step4["Step 4: Ingest Authoritative Rule Files<br/>(Read ACTIVE_RULES_BUNDLE.md, AGENTS.md, rules/)"]
    Step5["Step 5: Verify Clean Baseline<br/>(Run commit check, status check, verify_downstream_baseline.py)"]

    Step1 --> Step2 --> Step3 --> Step4 --> Step5
```

1. **Step 1: Mandatory Hidden Folder Direct-Path Read (CRITICAL FIRST STEP)**:
   - Execute `view_file` on `.pipeline/constitution.md` directly. Glob and ripgrep queries skip hidden directories; assuming `.pipeline/` is missing causes instant failure.
2. **Step 2: 4-Point Karpathy & Pipeline Compliance Check**:
   - Must evaluate in thought before every response:
     1. Is the message an inquiry or direct command?
     2. Has user explicitly approved file-write/command execution? (Yes/No)
     3. Am I making any silent assumptions about user intent?
     4. Does this turn write repo source/specs? (If yes, coordinator writing is locked; subagents required).
3. **Step 3: Strict Planning Gate Precedence**:
   - Obey the strict planning gate (`rules/user-authorization-lock.md`): you are strictly forbidden from creating, modifying, or deleting files in the workspace or dispatching subagents unless documented in an approved implementation plan. Even if the user prompt contains authorization keywords like `PROCEED`, an approved plan is mandatory. Writing or updating the implementation plan is the sole permitted write prior to approval.
4. **Step 4: Ingest Authoritative Rule Files**:
   - `.pipeline/constitution.md`: Pure Schema-Driven Compiler Invariant (§26), Three-Tier Platform Isolation.
   - `.pipeline/ACTIVE_RULES_BUNDLE.md`: Consolidated active governance bundle.
   - `AGENTS.md` / `.agents/AGENTS.md`: Subagent dispatch mandate, clean landing zones, remote synchronization mandate.
   - `rules/tracker-source-of-truth.md`: Non-closure invariant (neutral citations `(refs #<id>)` or `(#<id>)`, zero auto-closing verbs in commits).
   - `rules/user-authorization-lock.md`: Strict planning gate precedence.
   - `rules/dual-track-mbd-verification.md`: Track A (Simulink) and Track B (Python Twin) deliverables.
5. **Step 5: Verify Clean Baseline**:
   - Run `python3 scripts/verify_commit_messages.py --head` (Must pass, exit code 0).
   - Run `git status` (Clean tree on `main`, 0 bytes diff against `origin/main`).

---

## 4. Upstream Specification Compiler Roadmap & Architecture Deliverables

DEAP01-spec-core is the upstream Model-Based Systems Engineering (MBSE) specification compiler and verification framework. It systematically transforms abstract SysML v2 schemas and normative domain standards into complete, formally verified, tracker-grounded Agile backlogs (Epics, Features, User Stories, and Use Cases).

### 4.1 Upstream Compiler Core Invariants & Boundaries

1. **Pure Schema-Driven Compiler Invariant**:
   - The upstream compiler contains ZERO hardcoded domain concepts. It does not model aircraft, automobiles, medical devices, satellites, or any concrete physical systems.
   - All specification generation is strictly and deterministically derived from AST nodes (`package`, `part def`, `item def`, `action def`, `state def`, `port def`, `requirement def`, `use case def`) present in user-provided schemas in `schema/`.
2. **Upstream Distribution Template Clean Landing Zone Invariant**:
   - In upstream distribution template repositories (`DEAP-*`), the directories `schema/`, `docs/epics/`, `docs/features/`, `docs/user-stories/`, and `docs/use-cases/` must remain clean landing zones with ONLY `.gitkeep` files.
   - Committing concrete customer models, downstream specifications, or application source code to upstream templates is strictly forbidden.
3. **Downstream Application Workspace Boundary**:
   - Concrete engineering roadmaps (such as vehicle guidance, navigation, flight control laws, sensor models, or domain-specific Model-Based Design simulations) belong exclusively in downstream customer workspaces (e.g., `uav-009`).
   - Customer workspaces are provisioned and upgraded via `scripts/install_pipeline.sh`.

### 4.2 Key Upstream Compiler Deliverables

```mermaid
flowchart TD
    subgraph Compiler_Deliverables ["DEAP Upstream Specification Core Compiler Deliverables"]
        direction TB
        D1["Installer Hardening & Non-Clobbering Upgrade<br/>(scripts/install_pipeline.sh: Preserves Customer Schemas & ASTs)"]
        D2["Dual-Provider Architecture Engine<br/>(create_issue.sh & reconcile_backlog.py: gh + glab Support)"]
        D3["CommonMark & SysML v2 AST Validation Engine<br/>(Anti-Regex Invariant: Semantic Parity Gates 01-30)"]
        D4["Upstream Clean Landing Zone Enforcement<br/>(verify_downstream_baseline.py: .gitkeep Landing Zones)"]
        D5["Closed-Loop Model Parity & Reverse-Sync<br/>(compile_sysml.py --reverse-sync: 100% SSOT Bidirectional Sync)"]

        D1 --> D2 --> D3 --> D4 --> D5
    end
```

#### 1. Installer Hardening & Customer Data Preservation (`scripts/install_pipeline.sh`)
- **Preservation of Customer Artifacts**: The installer automatically detects existing customer schema definitions (`schema/*.sysml`, `schema/extracted/`), compiled ASTs (`.pipeline/schema.sysml`, `.pipeline/schema-digest.json`), localized defect dossiers, and customer reports during pipeline synchronization. It upgrades pipeline framework assets without wiping or clobbering customer models.
- **Robust Role Detection**: Distinguishes between `UPSTREAM_SPEC_CORE_COMPILER`, `DOMAIN_DISTRIBUTION_TEMPLATE`, and `DOWNSTREAM_CUSTOMER_PROJECT`, synthesizing appropriate README onboarding instructions and governance links.

#### 2. Dual-Provider Issue Automation & Tracker Synchronization
- **Universal Provider Support**: Both GitHub (`gh`) and GitLab (`glab`) CLI engines are first-class citizens across `skills/spec-orchestrator/scripts/create_issue.sh` and `scripts/reconcile_backlog.py`.
- **Automatic Provider Detection**: Automatically resolves tracker provider via `git remote get-url origin` or `TRACKER_PROVIDER` / `CI_SERVER_URL` environment variables.
- **GitLab Markdown Compatibility**: Rewrites GitHub-style markdown blob links (`/blob/main/`) to GitLab syntax (`/-/blob/main/`) automatically during issue publication.
- **Commit Non-Closure Enforcement**: Mandates neutral citations `(refs #<id>)` and `(#<id>)` to prevent server-side premature auto-closure across both GitHub and GitLab.

#### 3. CommonMark & SysML v2 AST Validation Engine (Anti-Regex Invariant)
- **Positive Model Closure**: Replaces brittle regex substring matching with rigorous CommonMark AST and SysML v2 AST parsing.
- **Elimination of Whitelist Bypasses**: Validates architectural entities, port interfaces, and signal dictionaries against the formal AST Single Source of Truth rather than open-world string exemptions.
- **Universal Mermaid Syntax Validation**: Enforces Check 7 syntax gates across all diagrams (explicit diagram headers, quoted node labels, closed code fences).

#### 4. Clean Landing Zone Enforcement & Domain Decoupling
- **Landing Zone Verification**: All template distributions enforce clean landing zones containing only `.gitkeep` files in `schema/` and `docs/`.
- **Platform Profile Separation**: Platform-specific implementation profiles (`.pipeline/profiles/`) decouple abstract specifications from concrete codebases (`app_flutter/`, `web_react/`, `embedded_c/`).

---

## 5. Abstract Pipeline Multi-Agent Protocol & Teamwork Orchestration

To maintain context hygiene, avoid token exhaustion, and enforce independent adversarial verification, all compiler enhancements and abstract specification workflows must be executed using context-isolated subagents.

### 5.1 Teamwork Multi-Agent Roster for Upstream Specification Compiler

| Role Name | Identifier | Responsibility & Gate |
| :--- | :--- | :--- |
| **Governance Sentinel** | `sentinel` | Enforces repository classification (`UPSTREAM_SPEC_CORE_COMPILER`), strict planning gate, mandatory hidden folder read of `.pipeline/constitution.md`, and human authorization verification. Halts unapproved edits. |
| **Master Orchestrator** | `orchestrator` | Coordinates multi-agent dispatches, phase sequencing, tracking milestones, and managing cross-worker artifacts. |
| **Compiler & AST Specialist** | `worker-compiler` | Maintains and hardens SysML v2 AST compilation, reverse-sync (`compile_sysml.py`), CommonMark AST validators, and schema parsers. |
| **Tooling & Multi-Provider Specialist** | `worker-tooling` | Maintains dual-provider scripts (`create_issue.sh`, `reconcile_backlog.py`), installer logic (`install_pipeline.sh`), and platform profiles. |
| **Specification Orchestration Worker** | `worker-spec` | Drives abstract schema-derived specification generation (Phases 1-3) using item-level context-isolated subagent dispatch loops. |
| **Reviewer 1 (Compiler & Schema Conformance)** | `reviewer-compiler` | Validates Pure Schema-Driven Compiler Invariant, clean landing zones, zero hardcoded domain tokens, and 100% schema AST coverage. |
| **Reviewer 2 (Code Hygiene & Quality Reviewer)** | `reviewer-hygiene` | Verifies code style, unit test passes, neutral citations `(refs #<id>)`, and non-closure invariants. |
| **Challenger 1 (Boundary & Isolation Challenger)** | `challenger-boundary` | Verifies zero leakage of customer concepts into upstream compiler; enforces upstream vs. downstream repository boundaries; verifies customer workspace preservation (zero clobbering). |
| **Challenger 2 (Anti-Regex & AST Provenance Challenger)** | `challenger-grounding` | Scans all generated code and specifications for regex shortcuts, heuristic exemptions, or ungrounded parameters. Validates positive AST provenance. |
| **Victory Auditor** | `victory-auditor` | Independently verifies that all verification gates pass (exit code 0), confirms `git diff origin/main` is empty, and issues formal `VICTORY CONFIRMED`. |

### 5.2 Abstract Teamwork Launch Command for Upstream Spec Core

To launch the multi-agent consensus team for upstream specification core compiler maintenance or abstract specification engineering, invoke the subagent dispatch block:

```json
{
  "Subagents": [
    {
      "TypeName": "teamwork_preview",
      "Role": "Autonomous Multi-Agent Consensus Team",
      "Model": "inherit",
      "Prompt": "Execute view_file on skills/spec-orchestrator/SKILL.md as your very first step before executing any file edits or commands, and strictly follow its formatting templates and instruction guidelines.\n\nRepository Classification: UPSTREAM_SPEC_CORE_COMPILER\nTarget Workspace: /Users/perkunas/jail/DEAP01-spec-core\nPrimary Role: Abstract MBSE Specification Compiler & Tooling Maintenance\n\nTask: Maintain and verify upstream specification compiler tooling, ensure installer hardening with customer preservation, maintain dual-provider tracker engines (gh/glab), and enforce clean landing zones across distribution templates.\n\nPROCEED"
    }
  ]
}
```

### 5.3 Operational Direction for Downstream Customer Application Work

- **Boundary Separation**: Incoming agents are explicitly instructed that downstream application development (such as vehicle guidance, navigation, flight control laws, sensor models, or hardware-in-the-loop simulation) **MUST NOT** be executed in this upstream compiler repository (`DEAP01-spec-core`).
- **Downstream Execution Protocol**:
  1. Navigate to the designated customer workspace (e.g. `/Users/perkunas/jail/uav-009`).
  2. Verify downstream pipeline installation via `python3 scripts/verify_downstream_baseline.py`.
  3. Execute feature-driven implementation and simulation using `skills/feature-driven-implementation/SKILL.md` within the customer workspace.

---

## 6. Inviolable Governance & Operational Rules for the Incoming Agent

1. **Pure Schema-Driven Compiler Invariant**: Never write code or specifications that assume hardcoded domain concepts. All specifications must derive deterministically from user schemas in `schema/`.
2. **Subagent Delegation Mandate**: The coordinator NEVER directly writes repository source code, specifications, or tests. All writes must be delegated to context-isolated subagents.
3. **Continuous Execution Fast-Path**: Once the implementation plan receives explicit human approval (`PROCEED`), execute all work packages continuously to completion without pausing for intermediate approval unless a gate fails (`exit code != 0`).
4. **Non-Closure Tracker Invariant**: Never use issue-closing verbs (`fix`, `fixes`, `close`, `closes`, `resolve`, `resolves`) preceding `#<id>` in git commit messages. Always use neutral citations `(refs #<id>)` or `(#<id>)`. Mechanical gate `verify_commit_messages.py` strictly enforces this.
5. **Issues Remain Open**: Applying `status:fixed-resolved` (or `status::fixed-resolved` on GitLab) does NOT close tracker issues. Issues stay open until validated and closed by the human Product Owner.
6. **Remote Synchronization Mandate**: No work package is complete until pushed to remote and verified with `git diff origin/<branch>` returning empty.
7. **Clean Landing Zone Invariant**: Upstream distribution templates (`DEAP-uas-infrastructure-safety`) must maintain 100% clean landing zones (`schema/`, `docs/epics/`, `docs/features/`, `docs/user-stories/`, `docs/use-cases/` contain ONLY `.gitkeep` files).
8. **Dual-Track MBD Verification Invariant**: In downstream control and guidance implementations, all features must provide dual-track deliverables—Track A (programmatic Simulink `.m`/`.sldd`) and Track B (headless Python twin `.py`)—with zero-license CI test harnesses and $\le 10^{-6}$ numerical equivalence per `rules/dual-track-mbd-verification.md`. The upstream compiler provides the abstract verification framework supporting both tracks.
9. **Universal Mermaid Syntax Rules**:
   - First non-comment line inside every ```` ```mermaid ```` fence MUST declare an explicit diagram type (`flowchart TD`, `sequenceDiagram`, etc.).
   - All subgraphs in flowcharts MUST declare explicit direction (`direction TB` / `direction LR`).
   - All node labels containing special characters, punctuation, or numbers MUST be double-quoted.
   - Code fences must be strictly closed with matching closing fences on a new line.
10. **Anti-Regex & AST Provenance Invariant**: Never use negative-string regex heuristics or substring bypasses to validate model closure, provenance, or architecture. All parameters, signals, and architectural entities must be positively validated against the SysML v2 AST Single Source of Truth or a compiled, closed-world typed parameter dictionary.
11. **Customer Workspace Preservation Invariant**: Upstream tools and installers must never clobber, overwrite, or delete downstream customer schemas, ASTs, or defect reports. All tool upgrades must respect downstream customer boundaries.
12. **Multi-Agent Consensus Integrity**: Complex engineering tasks must never be collapsed into single self-auditing subagents. Strict separation of roles across Sentinel, Orchestrator, Workers, Reviewers, Challengers, and Victory Auditor must be maintained.
13. **Coordinator Context Hygiene & Boundary Preservation Mandate**: The coordinator must never dump verbose terminal logs, raw CLI outputs, or execute repeated monolithic test suites in the primary conversation thread. Investigative execution and diagnostic runs must be delegated to context-isolated subagents. Concurrently, the coordinator must strictly preserve the boundary between the upstream compiler and downstream customer workspaces.
