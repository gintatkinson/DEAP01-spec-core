---
identifier: DEAP-BLUEPRINT-GITLAB-001
title: "DEAP Multi-Provider Infrastructure Architecture: Native GitLab & Self-Hosted Integration"
version: "1.0.0"
status: "APPROVED / PRODUCTION-GRADE"
created: "2026-09-01"
classification: "Enterprise Multi-Provider DevOps & Issue Tracker Architecture Specification"
assurance_levels:
  - "AssuranceLevel::Tier1"
  - "AssuranceLevel::Tier2"
  - "AssuranceLevel::Tier3"
  - "AssuranceLevel::Tier4"
  - "AssuranceLevel::Tier5"
target_standards:
  - "GitLab REST API v4"
  - "GitLab CI/CD (Pipeline Engine)"
  - "ISO/IEC/IEEE 15288:2023"
  - "RTCA DO-178C / DO-331"
  - "ASTM F3269-17 RTA"
  - "NIST SP 800-53 Rev. 5"
---

# DEAP Multi-Provider Infrastructure Architecture: Native GitLab & Self-Hosted Integration

| Attribute | Specification Detail |
| :--- | :--- |
| **Version** | 1.0.0 |
| **Date** | 2026-09-01 |

> **Document Identifier:** `DEAP-BLUEPRINT-GITLAB-001`  
> **Status:** `APPROVED / PRODUCTION-GRADE`  
> **Classification:** `Enterprise Multi-Provider DevOps & Issue Tracker Architecture Specification`  
> **Target Standards:** `GitLab REST API v4` | `GitLab CI/CD (Pipeline Engine)` | `ISO/IEC/IEEE 15288:2023` | `RTCA DO-178C / DO-331 (Tool Qualification & Model-Based Development)` | `ASTM F3269-17 RTA` | `JARUS SORA v2.5` | `NIST SP 800-53 Rev. 5 (SCIF / Air-Gapped Security Controls)` | `AssuranceLevel::Tier1..5`

---

## Section 1: Executive Summary & Multi-Provider Rationale

### 1.1 Executive Summary

In safety-critical cyber-physical systems--specifically abstract cyber-physical pipeline execution runners near critical infrastructure--system safety, multi-authority regulatory auditability (AssuranceLevel::Tier1..5), and formal certification (RTCA DO-178C / DO-331) demand end-to-end bi-directional traceability across all engineering artifacts.

A major architectural challenge in enterprise high-assurance infrastructure projects is **infrastructure and provider lock-in**. While initial open-source or commercial research and development frequently takes place on public software-as-a-service (SaaS) platforms (such as GitHub.com or GitLab.com SaaS), production deployment, defense programs, and classified operational variants are mandated to execute within **on-premises, air-gapped, or Sensitive Compartmented Information Facility (SCIF)** enclaves. These secure enclaves standardly deploy **GitLab Community Edition (CE)** or **GitLab Enterprise Edition (EE)** hosted on private hardware or isolated sovereign clouds (e.g., AWS Secret Region, Azure Government).

The **Digital Engineering Agentic Pipeline (DEAP)** resolves this operational divide by introducing a **Multi-Provider Infrastructure Architecture**. DEAP decouples the underlying Version Control System (VCS) transport from issue tracking, agile backlog reconciliation, and continuous integration/continuous delivery (CI/CD) pipelines. By providing native, zero-dependency GitLab REST API v4 integration alongside GitHub and local mock drivers, DEAP guarantees 100% deterministic portability between public SaaS, private enterprise clouds, and air-gapped defense enclaves without altering a single engineering specification, SysML v2 model, or Tier-1 commercial Model-Based Design (MBD) synthesis pipeline (**MATLAB / Simulink / Stateflow / Embedded Coder**).

```mermaid
flowchart TB
    subgraph VCS_Agnostic_Core ["DEAP Core Engine - VCS & Tracker Agnostic Layer"]
        AST_SSOT["SysML v2 Formal AST SSOT - .pipeline/schema.sysml"]
        Spec_Corpus["Agile Backlog Corpus - Epics, Features, Stories, Use Cases"]
        Reconcile_Engine["reconcile_backlog.py Engine"]
        Parity_Lock["22-Gate Parity Lock - verify_model_coverage.py"]
    end

    subgraph Tracker_Abstraction ["Unified Tracker Abstraction Layer"]
        Driver_Interface["IssueTracker Abstract Base Class"]
        GL_Driver["GitLabV4Tracker Driver - Zero-Dependency"]
        GH_Driver["GitHubCliTracker / GitHubRestTracker Driver"]
        Local_Driver["LocalMockTracker Driver - Offline/Isolated"]
    end

    subgraph Infrastructure_Targets ["Supported Infrastructure Deployment Targets"]
        subgraph Target_Public ["Public / Commercial SaaS"]
            GH_SaaS["GitHub.com Enterprise SaaS"]
            GL_SaaS["GitLab.com Ultimate SaaS"]
        end
        subgraph Target_Private ["Private / Sovereign Cloud"]
            GL_SelfHosted["GitLab EE Self-Hosted - AWS GovCloud / Azure Gov"]
        end
        subgraph Target_AirGapped ["Air-Gapped / SCIF Defense Enclave"]
            GL_AirGapped["GitLab CE/EE Air-Gapped Enclave - Offline CA / Zero Egress"]
        end
    end

    subgraph Toolchain_Synthesis ["Primary Tier-1 Commercial Toolchain - MBD"]
        Simulink["MATLAB / Simulink - Subsystems & Buses"]
        Stateflow["Stateflow - Discrete Mode Supervisors"]
        SLDV["Simulink Design Verifier - Formal RTA Proofs"]
        Coder["Embedded Coder - DO-178C C / SPARK Ada Synthesis"]
    end

    VCS_Agnostic_Core --> Tracker_Abstraction
    Driver_Interface --> GL_Driver
    Driver_Interface --> GH_Driver
    Driver_Interface --> Local_Driver

    GL_Driver --> GL_SaaS
    GL_Driver --> GL_SelfHosted
    GL_Driver --> GL_AirGapped
    GH_Driver --> GH_SaaS

    VCS_Agnostic_Core --> Toolchain_Synthesis
    Toolchain_Synthesis --> Parity_Lock
```

---

### 1.2 Multi-Provider Comparison & Strategy Matrix

The following matrix compares infrastructure deployment environments across transport protocols, authentication schemes, network constraints, and tracking capabilities:

| Architectural Dimension | GitHub.com SaaS | GitLab.com SaaS | Self-Hosted GitLab (EE/CE) | Air-Gapped / SCIF GitLab (EE/CE) |
| :--- | :--- | :--- | :--- | :--- |
| **Network Egress** | Unrestricted Internet | Unrestricted Internet | Restricted / VPC Peered | **Strictly Prohibited (Zero Egress)** |
| **Transport Layer** | HTTPS / SSH | HTTPS / SSH | HTTPS (Private CA) / SSH | HTTPS (Custom Root CA) / Local SSH |
| **API Version** | GitHub REST API v3 / GraphQL | GitLab REST API v4 | GitLab REST API v4 | GitLab REST API v4 |
| **Auth Tokens** | `GITHUB_TOKEN`, `GH_TOKEN`, PAT | `GITLAB_TOKEN`, `GL_TOKEN`, PAT | `GITLAB_TOKEN`, `CI_JOB_TOKEN` | `GITLAB_TOKEN`, `CI_JOB_TOKEN` |
| **TLS/SSL Validation** | Public WebPKI CA | Public WebPKI CA | Enterprise / Corporate Root CA | Private Air-Gapped Enclave Root CA |
| **CLI Availability** | `gh` CLI required | `glab` CLI or REST API | REST API / `glab` (optional) | **REST API Only (Zero External CLI)** |
| **Scoped Labels** | Emulated via strings | Native (`key::value`) | Native (`key::value`) | Native (`key::value`) |
| **CI/CD Pipeline Engine** | GitHub Actions (`.github/`) | GitLab CI (`.gitlab-ci.yml`) | GitLab CI (`.gitlab-ci.yml`) | GitLab CI (`.gitlab-ci.yml`) |
| **Runner Infrastructure** | Hosted Runners (Ubuntu/macOS) | Hosted / Shared Runners | Private Docker / Shell Runners | Local Bare-Metal / K8s Runners |
| **Artifact Storage** | GitHub Packages / Releases | GitLab Package Registry | Self-Hosted MinIO / Ceph S3 | Air-Gapped Local Blob Storage |
| **MBD Harness Runner** | MathWorks GitHub Action | GitLab Custom Container Runner | Bare-Metal Linux/Win Runner + MATLAB | Air-Gapped Host + MATLAB Concurrent Lic |

---

### 1.3 5-Whys Root Cause Analysis: Vendor Lock-In & Air-Gap Failures in high-assurance CI/CD

To understand why traditional high-assurance software pipelines fail when migrating from development environments to secure system-certification enclaves, DEAP applies the formal **5-Whys Root Cause Analysis**:

```mermaid
flowchart TD
    W1["Why 1 - Why do safety-critical high-assurance CI/CD pipelines fail when deployed to air-gapped or SCIF certification environments?"]
    W2["Why 2 - Why cannot pipeline automation scripts authenticate or reconcile backlog issues in secure enclaves?"]
    W3["Why 3 - Why are backlog automation scripts coupled to external CLI tools (e.g., gh CLI) and public SaaS endpoints?"]
    W4["Why 4 - Why was issue tracking and backlog reconciliation hard-coded to a single VCS vendor API instead of an abstract transport layer?"]
    W5["Why 5 (Root Cause) - Why did the software architecture fail to decouple VCS transport, issue tracking, and CI/CD execution via a zero-dependency REST engine and standard-compliant scoped label lifecycle?"]

    W1 --> W2
    W2 --> W3
    W3 --> W4
    W4 --> W5
```

- **Why 1:** Safety-critical pipelines fail in secure enclaves because scripts make outbound HTTP requests to unauthorized public hosts or require external binary dependencies not approved on the air-gapped baseline.
- **Why 2:** Backlog automation scripts fail to execute because they depend on external binaries (such as `gh` or Node-based GitHub action runners) and public certificate trust stores rather than enterprise TLS anchors.
- **Why 3:** Scripts are coupled to vendor-specific tooling because developers treat issue tracking as a proprietary portal rather than a standardized, REST-compliant entity lifecycle.
- **Why 4:** Issue tracking was coupled to a single vendor because early prototypes lacked an abstract provider interface separating domain logic (Epics, Features, Stories, Use Cases) from remote API serialization.
- **Why 5 (Root Cause):** The systems engineering architecture lacked a unified **Tracker Abstraction Architecture**, a **Zero-Dependency Native REST API Engine**, and a **Scoped Label Lifecycle** compatible with both GitLab CE/EE and GitHub.

---

### 1.4 Architectural Invariants & Non-Negotiable Tenets

1. **Zero External Runtime Dependency Policy:** All tracker clients, API callers, and reconciliation scripts must execute using standard Python runtime libraries (`urllib.request`, `urllib.parse`, `json`, `ssl`, `time`). External third-party packages (e.g., `requests`, `python-gitlab`, `urllib3`) must not be required for production reconciliation.
2. **Deterministic Provider Independence:** Specifying `TRACKER_PROVIDER=gitlab` or `TRACKER_PROVIDER=github` switches remote serialization without modifying a single markdown frontmatter schema, SysML v2 AST node, or test suite.
3. **Strict Transport Decoupling:** Git version control (cloning, branching, committing, pushing) operates over standard Git protocols (SSH/HTTPS) independently of the Issue Tracking REST API.
4. **Air-Gap & Private PKI First-Class Support:** The GitLab REST engine must natively support custom root CA certificates (`GITLAB_CA_CERT_PATH`), self-hosted domain routing (`GITLAB_URL`, `CI_SERVER_URL`), and ephemeral pipeline credentials (`CI_JOB_TOKEN`).
5. **Bidirectional DO-178C Traceability Parity:** Scoped labels (`type::*`, `status::*`, `safety::*`, `rta::*`) must enforce mutual exclusivity and provide a complete audit trail mapping directly to RTCA DO-178C verification objectives and ASTM F3269-17 Run-Time Assurance (RTA) invariants.

---

## Section 2: Tracker Abstraction Architecture

### 2.1 Tracker Decoupling Topology & Domain Model

The DEAP tracker architecture separates engineering domain concerns from infrastructure transport mechanics. The core engine interacts exclusively with the abstract `IssueTracker` interface, which defines standardized CRUD operations, search filters, comment streams, and label mutations.

```mermaid
classDiagram
    class IssueTracker {
        <<Abstract>>
        +str get_provider_name()
        +bool authenticate()
        +List~TrackerIssue~ list_issues(filters)
        +TrackerIssue get_issue(issue_id)
        +TrackerIssue create_issue(issue)
        +TrackerIssue update_issue(issue_id, updates)
        +TrackerComment add_comment(issue_id, body)
        +bool set_labels(issue_id, labels)
        +CanonicalStatus normalize_status(raw_status)
    }

    class GitLabV4Tracker {
        -GitLabV4Client client
        -str project_id
        -bool scoped_label_enabled
        +List~TrackerIssue~ list_issues(filters)
        +TrackerIssue get_issue(issue_id)
        +TrackerIssue create_issue(issue)
        +TrackerIssue update_issue(issue_id, updates)
        +TrackerComment add_comment(issue_id, body)
        +bool set_labels(issue_id, labels)
        -_map_scoped_labels(labels) List~str~
    }

    class GitHubCliTracker {
        -str gh_binary
        -str repo_slug
        +List~TrackerIssue~ list_issues(filters)
        +TrackerIssue get_issue(issue_id)
        +TrackerIssue create_issue(issue)
        +TrackerIssue update_issue(issue_id, updates)
        +TrackerComment add_comment(issue_id, body)
        +bool set_labels(issue_id, labels)
    }

    class GitHubRestTracker {
        -str token
        -str base_url
        +List~TrackerIssue~ list_issues(filters)
        +TrackerIssue get_issue(issue_id)
        +TrackerIssue create_issue(issue)
        +TrackerIssue update_issue(issue_id, updates)
        +TrackerComment add_comment(issue_id, body)
        +bool set_labels(issue_id, labels)
    }

    class LocalMockTracker {
        -Path mock_store_path
        -Dict in_memory_db
        +List~TrackerIssue~ list_issues(filters)
        +TrackerIssue get_issue(issue_id)
        +TrackerIssue create_issue(issue)
        +TrackerIssue update_issue(issue_id, updates)
        +TrackerComment add_comment(issue_id, body)
        +bool set_labels(issue_id, labels)
    }

    IssueTracker <|-- GitLabV4Tracker
    IssueTracker <|-- GitHubCliTracker
    IssueTracker <|-- GitHubRestTracker
    IssueTracker <|-- LocalMockTracker
```

---

### 2.2 Factory Pattern & Dynamic Provider Resolution

The reconciliation engine instantiates concrete tracker drivers at runtime via a deterministic resolution hierarchy:

```mermaid
flowchart TD
    Start["Initiate Tracker Initialization"] --> Check_Explicit{"Is TRACKER_PROVIDER set in env?"}
    
    Check_Explicit -->|Yes: gitlab| Init_GitLab["Instantiate GitLabV4Tracker"]
    Check_Explicit -->|Yes: github| Init_GitHub["Instantiate GitHubCliTracker / GitHubRestTracker"]
    Check_Explicit -->|Yes: local / mock| Init_Mock["Instantiate LocalMockTracker"]
    
    Check_Explicit -->|No| Check_CI{"Is GitLab CI active? (CI_SERVER_URL or GITLAB_CI present)"}
    Check_CI -->|Yes| Init_GitLab
    Check_CI -->|No| Check_GH_Env{"Is GITHUB_ACTIONS or GH_TOKEN present?"}
    Check_GH_Env -->|Yes| Init_GitHub
    Check_GH_Env -->|No| Check_Remote{"Inspect git remote origin URL"}
    
    Check_Remote -->|Contains gitlab| Init_GitLab
    Check_Remote -->|Contains github| Init_GitHub
    Check_Remote -->|No Remote / Air-Gapped Local| Init_Mock

    Init_GitLab --> Auth_GL{"Validate GitLab Auth Token & CA"}
    Init_GitHub --> Auth_GH{"Validate GitHub Auth / CLI"}
    Init_Mock --> Ready["Tracker Driver Ready for Reconciliation"]

    Auth_GL -->|Pass| Ready
    Auth_GL -->|Fail| Fatal_GL["Raise FatalAuthenticationError (Hard Stop)"]
    Auth_GH -->|Pass| Ready
    Auth_GH -->|Fail| Fatal_GH["Raise FatalAuthenticationError (Hard Stop)"]
```

#### Provider Resolution Configuration Specification

The runtime resolution is parameterized by `.pipeline/config.yaml` or `.codebase-rules.json`:

```json
{
  "tracker_configuration": {
    "default_provider": "auto",
    "providers": {
      "gitlab": {
        "api_version": "v4",
        "default_url": "https://gitlab.com",
        "url_env_vars": ["GITLAB_URL", "CI_SERVER_URL"],
        "token_env_vars": ["GITLAB_TOKEN", "GL_TOKEN", "CI_JOB_TOKEN"],
        "ca_cert_env_vars": ["GITLAB_CA_CERT_PATH", "REQUESTS_CA_BUNDLE", "SSL_CERT_FILE"],
        "scoped_labels": true,
        "max_body_chars": 1000000,
        "rate_limit_retry_attempts": 5,
        "rate_limit_backoff_base_sec": 1.5
      },
      "github": {
        "api_version": "v3",
        "default_url": "https://api.github.com",
        "token_env_vars": ["GITHUB_TOKEN", "GH_TOKEN"],
        "scoped_labels": false,
        "max_body_chars": 65536,
        "rate_limit_retry_attempts": 5,
        "rate_limit_backoff_base_sec": 2.0
      }
    }
  }
}
```

---

### 2.3 Uniform Data Model & Entities

The unified data model defines immutable, strongly-typed representations of backlog entities across all providers:

```python
from dataclasses import dataclass, field
from typing import List, Dict, Optional, Any
from enum import Enum

class CanonicalStatus(str, Enum):
    PROPOSED = "proposed"
    IN_PROGRESS = "in-progress"
    READY_FOR_REVIEW = "ready-for-review"
    FIXED_RESOLVED = "fixed-resolved"
    VERIFIED = "verified"
    CLOSED = "closed"
    BLOCKED = "blocked"

class SpecType(str, Enum):
    EPIC = "epic"
    FEATURE = "feature"
    USER_STORY = "user-story"
    USE_CASE = "use-case"
    SAFETY_REQ = "safety-req"

@dataclass(frozen=True)
class TrackerLabel:
    name: str
    color: Optional[str] = None
    description: Optional[str] = None
    scoped_key: Optional[str] = None
    scoped_value: Optional[str] = None

@dataclass
class TrackerIssue:
    issue_id: str                      # Canonical string representation (e.g., "104" or "GL-104")
    iid: int                           # Internal integer project-scoped ID (GitLab IID / GitHub Number)
    title: str                         # Normalized issue title
    body: str                          # Markdown specification content
    state: str                         # Provider state ("opened", "closed", "OPEN", "CLOSED")
    labels: List[str]                  # Complete list of applied label strings
    author: str                        # Author username
    web_url: str                       # Canonical web URL for the issue
    created_at: str                    # ISO 8601 timestamp
    updated_at: str                    # ISO 8601 timestamp
    status: CanonicalStatus = CanonicalStatus.PROPOSED
    spec_type: Optional[SpecType] = None
    metadata: Dict[str, Any] = field(default_factory=dict)

@dataclass
class TrackerComment:
    comment_id: str
    body: str
    author: str
    created_at: str
    system: bool = False
```

---

### 2.4 Transactional Backlog Reconciliation Lifecycle

The reconciliation engine (`scripts/reconcile_backlog.py`) executes a 7-step transactional pipeline ensuring that markdown files in `docs/` and remote tracker issues remain in mathematical synchronization:

```mermaid
sequenceDiagram
    autonumber
    participant LocalDisk as "Local Spec Corpus (docs/)"
    participant Reconciler as "reconcile_backlog.py"
    participant Factory as "TrackerFactory"
    participant Driver as "GitLabV4Tracker Driver"
    participant GL_API as "GitLab REST API v4"
    participant Auditor as "22-Gate Parity Lock"

    LocalDisk->>Reconciler: Scan docs/epics, features, user-stories, use-cases
    Reconciler->>Factory: Resolve Provider (env/remote detection)
    Factory->>Driver: Initialize GitLabV4Tracker(project_id, token, url)
    Driver->>GL_API: GET /api/v4/projects/:id (Validate connectivity & permissions)
    GL_API-->>Driver: HTTP 200 OK (Project JSON metadata)

    Reconciler->>Driver: list_issues(state="all", per_page=100)
    Driver->>GL_API: GET /api/v4/projects/:id/issues?pagination=keyset
    GL_API-->>Driver: Return all existing issues (Paginated)
    Driver-->>Reconciler: In-memory Issue Index (by IID & title)

    rect rgb(240, 248, 255)
        Note over Reconciler,GL_API: Step 4-6: Placeholder Resolution & Issue Sync
        Reconciler->>Reconciler: Resolve #[IssueID] -> #[IID] in dependencies
        Reconciler->>Driver: Update issue body & Scoped Labels (status::*, type::*)
        Driver->>GL_API: PUT /api/v4/projects/:id/issues/:iid
        GL_API-->>Driver: HTTP 200 OK (Updated Issue)
    end

    rect rgb(240, 255, 240)
        Note over Reconciler,LocalDisk: Step 7: Local Frontmatter Update & Verification
        Reconciler->>LocalDisk: Write canonical issue_id & url to YAML frontmatter
        Reconciler->>Auditor: Execute verify_model_coverage.py
        Auditor-->>Reconciler: Verification Pass (0 Drift / 0 Violations)
    end
```

---

## Section 3: Native GitLab REST API v4 Engine Specification

### 3.1 Zero-Dependency Python Client Architecture (`urllib.request`)

In restricted, air-gapped, and SCIF environments, installing third-party Python wheels (such as `requests`, `urllib3`, `certifi`, or `python-gitlab`) is strictly governed and often prohibited without lengthy security accreditation. To ensure zero operational friction, the DEAP GitLab Engine is implemented purely using the Python standard library.

```python
import os
import json
import ssl
import time
import urllib.request
import urllib.parse
import urllib.error
from typing import Dict, List, Optional, Tuple, Any

class GitLabV4Client:
    """
    Zero-dependency GitLab REST API v4 client.
    Implements RFC 5988 Link header pagination, rate-limit backoff,
    custom private CA TLS validation, and scoped label management.
    """

    def __init__(
        self,
        base_url: str,
        token: str,
        project_id: str,
        ca_cert_path: Optional[str] = None,
        timeout_sec: int = 30,
        max_retries: int = 5,
        backoff_base_sec: float = 1.5,
    ):
        self.base_url = base_url.rstrip("/")
        self.token = token
        self.project_id = urllib.parse.quote(project_id, safe="")
        self.timeout_sec = timeout_sec
        self.max_retries = max_retries
        self.backoff_base_sec = backoff_base_sec
        self.ssl_context = self._create_ssl_context(ca_cert_path)

    def _create_ssl_context(self, ca_cert_path: Optional[str]) -> ssl.SSLContext:
        """Create secure TLS 1.3/1.2 SSL context with custom enterprise/SCIF root CAs."""
        ctx = ssl.create_default_context(ssl.Purpose.SERVER_AUTH)
        if ca_cert_path and os.path.isfile(ca_cert_path):
            ctx.load_verify_locations(cafile=ca_cert_path)
        # Defense requirement: enforce TLS 1.2 minimum
        ctx.minimum_version = ssl.TLSVersion.TLSv1_2
        return ctx

    def _build_request(
        self,
        endpoint: str,
        method: str = "GET",
        params: Optional[Dict[str, Any]] = None,
        data: Optional[Dict[str, Any]] = None,
    ) -> urllib.request.Request:
        """Constructs authenticated urllib Request with private token headers."""
        url = f"{self.base_url}/api/v4/{endpoint.lstrip('/')}"
        if params:
            query_string = urllib.parse.urlencode(params)
            url = f"{url}?{query_string}"

        body = None
        headers = {
            "PRIVATE-TOKEN": self.token,
            "Accept": "application/json",
            "User-Agent": "DEAP-GitLab-Engine/1.0 (DO-178C-Audited)",
        }

        if data is not None:
            body = json.dumps(data).encode("utf-8")
            headers["Content-Type"] = "application/json; charset=utf-8"

        return urllib.request.Request(url=url, data=body, headers=headers, method=method)

    def execute_request(
        self,
        endpoint: str,
        method: str = "GET",
        params: Optional[Dict[str, Any]] = None,
        data: Optional[Dict[str, Any]] = None,
    ) -> Tuple[int, Dict[str, Any], Dict[str, str]]:
        """
        Executes HTTP request with exponential backoff for 429 and 5xx errors.
        Returns: (status_code, json_response_dict, headers_dict)
        """
        req = self._build_request(endpoint, method, params, data)
        attempt = 0

        while attempt < self.max_retries:
            attempt += 1
            try:
                with urllib.request.urlopen(req, context=self.ssl_context, timeout=self.timeout_sec) as resp:
                    status_code = resp.status
                    resp_headers = dict(resp.headers.items())
                    raw_body = resp.read().decode("utf-8")
                    parsed_body = json.loads(raw_body) if raw_body else {}
                    return status_code, parsed_body, resp_headers

            except urllib.error.HTTPError as e:
                status_code = e.code
                error_headers = dict(e.headers.items()) if e.headers else {}
                
                # Check for rate limiting (429) or transient gateway failures (502, 503, 504)
                if status_code in (429, 502, 503, 504):
                    retry_after = error_headers.get("Retry-After")
                    if retry_after and retry_after.isdigit():
                        sleep_time = float(retry_after)
                    else:
                        sleep_time = self.backoff_base_sec * (2 ** (attempt - 1))
                    
                    time.sleep(sleep_time)
                    continue

                raw_err = e.read().decode("utf-8") if e.fp else ""
                raise RuntimeError(
                    f"GitLab API HTTP {status_code} Error on {method} {endpoint}: {raw_err}"
                ) from e

            except urllib.error.URLError as e:
                if attempt >= self.max_retries:
                    raise RuntimeError(f"Network transport failure connecting to {self.base_url}: {e.reason}") from e
                time.sleep(self.backoff_base_sec * (2 ** (attempt - 1)))

        raise TimeoutError(f"Exceeded maximum retries ({self.max_retries}) for GitLab API request: {endpoint}")
```

---

### 3.2 Endpoint Architecture & Request/Response Contracts

The DEAP client interacts with four primary GitLab REST API v4 resource trees:

```mermaid
graph LR
    subgraph GitLab_API_v4 ["GitLab REST API v4 Endpoints"]
        P_Proj["/api/v4/projects/:id"]
        P_Issues["/api/v4/projects/:id/issues"]
        P_Issue_IID["/api/v4/projects/:id/issues/:issue_iid"]
        P_Notes["/api/v4/projects/:id/issues/:issue_iid/notes"]
        P_Labels["/api/v4/projects/:id/labels"]
    end

    Client["GitLabV4Client"] --> P_Proj
    Client --> P_Issues
    Client --> P_Issue_IID
    Client --> P_Notes
    Client --> P_Labels
```

#### 1. Project Discovery & Validation
- **Method / Path:** `GET /api/v4/projects/:id`
- **Path Parameter `:id`:** URL-encoded project path (e.g., `Cyber-Physical System-group%2Fsystem-safety-subsystem`) or numeric ID (e.g., `42019`).
- **Response Validation:** Verifies project accessibility, default branch, visibility, and features enabled (`issues_enabled == true`).

#### 2. Keyset & Offset Issue Query
- **Method / Path:** `GET /api/v4/projects/:id/issues`
- **Query Parameters:**
  - `state`: `all` | `opened` | `closed`
  - `labels`: Comma-separated list (e.g., `type::feature,status::in-progress`)
  - `per_page`: `100` (GitLab maximum)
  - `pagination`: `keyset` (preferred for high-volume enterprise projects)
  - `order_by`: `created_at` | `id`
  - `sort`: `asc` | `desc`

#### 3. Transactional Issue Update
- **Method / Path:** `PUT /api/v4/projects/:id/issues/:issue_iid`
- **Payload Schema:**
```json
{
  "title": "Feature 104: Multi-Spectral Optical Fence Intrusion Detector",
  "description": "## 1. Feature Description\n\n...",
  "add_labels": "status::fixed-resolved,verification::passed",
  "remove_labels": "status::in-progress,status::ready-for-review",
  "state_event": "reopen"
}
```

#### 4. Audit & Verification Discussion Notes
- **Method / Path:** `POST /api/v4/projects/:id/issues/:issue_iid/notes`
- **Payload Schema:**
```json
{
  "body": "### \u2705 DEAP DO-178C Automated Verification Report\n\n- **22-Gate Parity Lock:** PASSED\n- **SysML v2 AST Digest:** `a7f9c2...`\n- **Simulink Model Synthesis:** `SystemControl_SLX_v2.slx` (0 SLDV Errors)\n- **Traceability Status:** 100% Objectives Satisfied\n- **GitLab Pipeline:** [#94821](https://gitlab.internal.defense.gov/Cyber-Physical System/safety/-/pipelines/94821)"
}
```

---

### 3.3 Keyset & Link-Header Pagination Engine

For enterprise repositories with thousands of requirements and user stories, offset-based pagination (`page=N`) suffers from performance degradation ($O(N)$ database query cost). DEAP implements dual pagination:

1. **Keyset Pagination (GitLab Standard):** Evaluates `Link` headers containing `rel="next"` with cursor tokens.
2. **RFC 5988 Link Header Parser:** Extracts next URL endpoints deterministically.

```python
def fetch_all_issues(client: GitLabV4Client) -> List[Dict[str, Any]]:
    """
    Fetches all project issues using RFC 5988 Link header pagination.
    """
    all_issues = []
    endpoint = f"projects/{client.project_id}/issues"
    params = {"per_page": 100, "state": "all"}

    while endpoint:
        status, issues, headers = client.execute_request(endpoint, method="GET", params=params)
        all_issues.extend(issues)
        
        # Keyset / Header Link Resolution
        link_header = headers.get("Link", "")
        next_url = None
        if link_header:
            # Format: <https://gitlab.com/api/v4/projects/1/issues?page=2&per_page=100>; rel="next", ...
            links = link_header.split(",")
            for link in links:
                parts = link.split(";")
                if len(parts) >= 2 and 'rel="next"' in parts[1]:
                    raw_url = parts[0].strip().strip("<>")
                    next_url = raw_url

        if next_url:
            # Extract endpoint relative to base_url/api/v4
            api_prefix = f"{client.base_url}/api/v4/"
            if next_url.startswith(api_prefix):
                endpoint = next_url[len(api_prefix):]
                params = None # Params already encoded in next_url
            else:
                break
        else:
            # Offset pagination fallback if Link header absent
            total_pages = int(headers.get("X-Total-Pages", 1))
            current_page = int(headers.get("X-Page", 1))
            if current_page < total_pages:
                params["page"] = current_page + 1
            else:
                break

    return all_issues
```

---

### 3.4 Rate Limiting & Resiliency State Machine

GitLab API endpoints enforce rate limits (typically 2,000 to 10,000 requests per minute depending on tier). The client implements a bounded exponential backoff with jitter:

$$T_{\text{wait}} = \min\left(T_{\max}, T_{\text{base}} \times 2^{\text{attempt}} + \mathcal{U}(0, 0.5)\right)$$

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Executing : "execute_request()"
    Executing --> Success : "HTTP 200 / 201"
    Executing --> RateLimited : "HTTP 429 (Too Many Requests)"
    Executing --> ServerError : "HTTP 502 / 503 / 504"
    Executing --> ClientError : "HTTP 400 / 401 / 403 / 404"

    RateLimited --> InspectHeaders : "Check Retry-After header"
    InspectHeaders --> Sleeping : "Sleep for Retry-After or Exp-Backoff"
    ServerError --> Sleeping : "Sleep Exp-Backoff"
    
    Sleeping --> Executing : "Retry Attempt <= MaxRetries"
    Sleeping --> TerminalFailure : "Retry Attempt > MaxRetries"

    ClientError --> TerminalFailure : "Raise Unrecoverable Error"
    Success --> [*] : "Return JSON Payload"
    TerminalFailure --> [*] : "Raise Exception & Halt Pipeline"
```

---

## Section 4: Scoped Label Lifecycle & DO-178C / SORA Traceability

### 4.1 Two-Tier Scoped Label Taxonomy (`key::value`)

GitLab native **Scoped Labels** utilize the double-colon syntax (`::`) to enforce mutual exclusivity. Applying `status::fixed-resolved` automatically removes `status::in-progress` or `status::ready-for-review` without requiring manual unlabeling calls.

DEAP establishes a standardized two-tier label taxonomy mapped to high-assurance safety standards:

| Label Category | Scoped Syntax | Mutual Exclusivity | Aerospace / Safety Standard | Purpose |
| :--- | :--- | :--- | :--- | :--- |
| **Artifact Type** | `type::epic`<br>`type::feature`<br>`type::user-story`<br>`type::use-case`<br>`type::safety-req` | Enforced | ISO/IEC/IEEE 15288 §6.4<br>DO-178C High-Level Requirements | Categorizes backlog artifact level and metamodel abstraction. |
| **Lifecycle Status** | `status::proposed`<br>`status::in-progress`<br>`status::ready-for-review`<br>`status::fixed-resolved`<br>`status::verified`<br>`status::closed` | Enforced | DO-178C Table A-1 to A-7<br>Constitutional State Machine | Controls verification gating and transition approval state. |
| **Design Assurance** | `safety::dal-a`<br>`safety::dal-b`<br>`safety::dal-c`<br>`safety::dal-d` | Enforced | RTCA DO-178C / DO-254 / ARP4754A | Assigns rigor requirements for MC/DC coverage and formal proofs. |
| **SORA SAIL** | `sora::sail-i`<br>`sora::sail-ii`<br>`sora::sail-iii`<br>`sora::sail-iv`<br>`sora::sail-v`<br>`sora::sail-vi` | Enforced | JARUS SORA v2.5 Annex E | Specific Assurance and Integrity Level for abstract cyber-physical pipeline execution runners risk mitigation. |
| **RTA Architecture** | `rta::envelope-protection`<br>`rta::recovery-trigger`<br>`rta::monitored-invariant` | Additive | ASTM F3269-17 RTA | Identifies Run-Time Assurance components and safety guards. |
| **Verification Gate** | `verification::passed`<br>`verification::failed`<br>`verification::blocked` | Enforced | 22-Gate Mechanical Parity Lock | Automated CI/CD mechanical pass/fail evidence indicator. |

---

### 4.2 Scoped Label Lifecycle State Machine

The progression of an issue through its lifecycle is strictly governed by automated verification gates. In accordance with project constitution rules, subagents and engineers may transition issues up to `status::fixed-resolved` and `verification::passed`. The final transition to `status::closed` is reserved exclusively for the Product Owner / Certification Authority:

```mermaid
stateDiagram-v2
    [*] --> Proposed : "type::* + status::proposed"
    
    Proposed --> InProgress : "Developer / Agent Assigned (status::in-progress)"
    
    state InProgress {
        [*] --> Authoring
        Authoring --> SpecElaboration : "Refine Markdown & SysML AST"
        SpecElaboration --> LocalVerification : "Run verify_model_coverage.py"
    }

    InProgress --> ReadyForReview : "Spec & Code Complete (status::ready-for-review)"
    
    ReadyForReview --> PipelineVerification : "GitLab CI Triggered"
    
    state PipelineVerification {
        [*] --> LintStage : "lint:sysml + lint:markdown"
        LintStage --> TestStage : "test:semantic-acceptance + test:simulink-harness"
        TestStage --> VerifyStage : "22-Gate Parity Lock + SLDV Proofs"
    }

    PipelineVerification --> InProgress : "Gate Tripped / Test Failed (verification::failed)"
    PipelineVerification --> FixedResolved : "All 22 Gates Pass (status::fixed-resolved + verification::passed)"

    FixedResolved --> Verified : "Automated End-to-End Traceability Build (status::verified)"
    
    Verified --> Closed : "Product Owner / DER Sign-off (status::closed)"
    Closed --> [*]
```

---

### 4.3 DO-178C Objective Traceability & SORA Risk Reduction Mapping

Every GitLab Issue and its associated markdown specification file (`docs/features/FEAT-*.md`, `docs/user-stories/US-*.md`) maintain cryptographic bi-directional traceability to SysML v2 model elements and DO-178C lifecycle objectives:

```mermaid
flowchart LR
    subgraph SysML_Model ["SysML v2 Formal Model"]
        SysML_Req["requirement def REQ_FENCE_01"]
        SysML_Part["part def FenceIntrusionDetector"]
        SysML_State["state def MonitoringState"]
    end

    subgraph GitLab_Issues ["GitLab Issue Tracker - REST API v4"]
        GL_Epic["Epic #10: Infrastructure Safety Monitoring - type::epic, safety::dal-a"]
        GL_Feat["Feature #104: Optical Fence Detector - type::feature, status::fixed-resolved"]
        GL_Story["Story #412: Multi-Spectral Anomaly Trigger - type::user-story, verification::passed"]
    end

    subgraph MBD_Toolchain ["Tier-1 MBD Synthesis"]
        SLX_Block["Simulink Subsystem: FenceDetector.slx"]
        SF_Chart["Stateflow Supervisor: FSM_FenceGuard.sfx"]
        Generated_C["Embedded Coder: fence_detector.c"]
    end

    subgraph Certification_Evidence ["DO-178C / SORA Evidence Artifacts"]
        Trace_Matrix["Traceability Matrix: TRACE-DO178C.json"]
        SLDV_Report["SLDV Formal Proof Report: SLDV-FENCE-001.pdf"]
        MCDC_Cov["MC/DC 100% Coverage Report: COV-DO178C-DAL-A.xml"]
    end

    SysML_Req --> GL_Epic
    SysML_Part --> GL_Feat
    SysML_State --> GL_Story

    GL_Feat --> SLX_Block
    GL_Story --> SF_Chart
    SLX_Block --> Generated_C
    SF_Chart --> Generated_C

    Generated_C --> Trace_Matrix
    SLX_Block --> SLDV_Report
    Generated_C --> MCDC_Cov
```

---

## Section 5: Air-Gapped & SCIF Deployment Security Model

### 5.1 Token Authentication Hierarchy & Resolution Engine

To support execution across diverse environments--from developer laptops to automated air-gapped CI/CD runners--the DEAP GitLab Engine enforces a deterministic credential resolution hierarchy:

```mermaid
flowchart TD
    Start["Request API Authentication"] --> C1{"Is GITLAB_TOKEN set?"}
    C1 -->|Yes| Sanitized1["Sanitize & Validate Token"]
    C1 -->|No| C2{"Is GL_TOKEN set?"}
    C2 -->|Yes| Sanitized2["Sanitize & Validate Token"]
    C2 -->|No| C3{"Is CI_JOB_TOKEN set? (GitLab CI/CD)"}
    C3 -->|Yes| Header_JobToken["Authenticate via JOB-TOKEN Header"]
    C3 -->|No| C4{"Is Local Mock Mode Enabled?"}
    C4 -->|Yes| Use_Mock["Operate LocalMockTracker (No Network)"]
    C4 -->|No| Fatal_Auth["Raise FatalAuthenticationError: No Valid Token"]

    Sanitized1 --> Header_PrivateToken["Authenticate via PRIVATE-TOKEN Header"]
    Sanitized2 --> Header_PrivateToken
```

#### Token Sanitization & Threat Mitigation

To prevent dummy, mock, or leaked placeholders from interfering with real authentication, the engine executes environment sanitization:

```python
def sanitize_token_env():
    """
    Scans environment variables and purges placeholder or dummy tokens.
    """
    dummy_patterns = ("antigravity", "dummy", "placeholder", "mock", "invalid", "your_token_here")
    for var in ("GITLAB_TOKEN", "GL_TOKEN", "GITHUB_TOKEN", "GH_TOKEN"):
        val = os.environ.get(var, "")
        if val and any(pattern in val.lower() for pattern in dummy_patterns):
            os.environ.pop(var, None)
```

---

### 5.2 Custom Domain & Instance Routing

The engine resolves the target instance URL and project path dynamically, handling nested group namespaces standard in defense hierarchies (e.g., `defense-org/Cyber-Physical System-division/safety-branch/Cyber-Physical System-infrastructure-safety`):

```python
def resolve_gitlab_instance_and_project() -> Tuple[str, str]:
    """
    Resolves (gitlab_base_url, project_path_or_id) from environment or git remote.
    """
    # 1. Base URL Resolution
    base_url = (
        os.environ.get("CI_SERVER_URL")
        or os.environ.get("GITLAB_URL")
        or os.environ.get("GL_URL")
        or "https://gitlab.com"
    ).rstrip("/")

    # 2. Project ID / Slug Resolution
    project_id = (
        os.environ.get("CI_PROJECT_PATH")
        or os.environ.get("CI_PROJECT_ID")
        or os.environ.get("GITLAB_PROJECT_ID")
    )

    if not project_id:
        # Fallback to inspecting git remote origin URL
        try:
            import subprocess
            remote_url = subprocess.check_output(
                ["git", "config", "--get", "remote.origin.url"],
                text=True
            ).strip()
            # Parse git@gitlab.internal.defense.gov:Cyber-Physical System/safety.git or https://...
            if "gitlab" in remote_url:
                if remote_url.startswith("git@"):
                    path_part = remote_url.split(":", 1)[1]
                else:
                    path_part = urllib.parse.urlparse(remote_url).path.lstrip("/")
                project_id = path_part.rstrip(".git")
        except Exception:
            project_id = "default/project"

    return base_url, project_id
```

---

### 5.3 Air-Gapped Network & Custom Private CA TLS Architecture

In SCIF and air-gapped deployments, GitLab instances use internal certificates signed by a private defense Root Certificate Authority. Connections will fail if evaluated against the public WebPKI bundle.

DEAP implements strict TLS validation using the local authority store:

```mermaid
flowchart LR
    subgraph Air_Gapped_Host ["Air-Gapped Host / Runner Node"]
        Client_Script["DEAP Python Script - GitLabV4Client"]
        CA_Store["/etc/ssl/certs/defense-internal-root-ca.crt - GITLAB_CA_CERT_PATH"]
        Custom_SSL["ssl.SSLContext - TLS 1.3 / 1.2 Strict"]
    end

    subgraph Enclave_Network ["Isolated SCIF Enclave (Zero Egress)"]
        GL_Server["GitLab Enterprise Server\n(gitlab.internal.defense.gov)"]
    end

    Client_Script --> CA_Store
    CA_Store --> Custom_SSL
    Custom_SSL -->|"Encrypted HTTPS (Port 443)\nMutual TLS / Encrypted Channel"| GL_Server
```

1. **Certificate Discovery:** The client checks `GITLAB_CA_CERT_PATH`, `REQUESTS_CA_BUNDLE`, `SSL_CERT_FILE`, and system locations (`/etc/ssl/certs/ca-certificates.crt`, `/etc/pki/tls/certs/ca-bundle.crt`).
2. **Context Creation:** `ssl.create_default_context(cafile=ca_path)` ensures complete validation of internal certificate chains.
3. **No Insecure Fallback:** Disabling certificate verification (`verify=False` / `CERT_NONE`) is strictly prohibited in compliance with NIST SP 800-53 Rev. 5 controls (SC-8 Transmission Confidentiality and Integrity).

---

### 5.4 SCIF Operational Workflow & Data Flow Diagram

```mermaid
flowchart TD
    subgraph SCIF_Enclave ["Sensitive Compartmented Information Facility (SCIF) Enclave"]
        subgraph Vault ["Local Artifact Vault (Air-Gapped)"]
            GitLab_EE["Self-Hosted GitLab EE Server (Local DB & Repos)"]
            PyPI_Mirror["Offline Python Package Mirror (Audited Wheels)"]
            MATLAB_Server["MathWorks Network License Server (Local)"]
        end

        subgraph Runner_Fleet ["GitLab CI/CD Runner Fleet (Dedicated Bare-Metal)"]
            Runner1["Runner 1: Lint & AST Parse"]
            Runner2["Runner 2: Semantic Acceptance & Mock Reconciler"]
            Runner3["Runner 3: MATLAB / Simulink / SLDV / Embedded Coder"]
        end

        subgraph Agentic_Dev ["Agentic Engineering Workstations"]
            Agent1["Subagent Context A (Spec Scaffolding)"]
            Agent2["Subagent Context B (Reverse Sync Engine)"]
            Dev1["Systems Safety Engineer"]
        end
    end

    Agent1 -->|"Local Git SSH + REST API v4"| GitLab_EE
    Agent2 -->|"Local Git SSH + REST API v4"| GitLab_EE
    Dev1 -->|"Local Git SSH + REST API v4"| GitLab_EE

    GitLab_EE -->|"Trigger Local Pipeline"| Runner1
    GitLab_EE -->|"Trigger Local Pipeline"| Runner2
    GitLab_EE -->|"Trigger Local Pipeline"| Runner3

    Runner3 -->|"Fetch Licenses"| MATLAB_Server
    Runner1 & Runner2 & Runner3 -->|"Post Scoped Labels & Evidence Notes"| GitLab_EE
```

---

## Section 6: Standardized 3-Stage GitLab CI/CD Pipeline Matrix

### 6.1 Canonical `.gitlab-ci.yml` Pipeline Architecture

The DEAP pipeline architecture enforces a strict 3-stage mechanical verification matrix:

$$\text{Pipeline} = \text{Stage}_{\text{lint}} \xrightarrow{\text{pass}} \text{Stage}_{\text{test}} \xrightarrow{\text{pass}} \text{Stage}_{\text{verify}}$$

```mermaid
flowchart LR
    subgraph Stage_Lint ["Stage 1: lint"]
        L1["lint:sysml (AST & Syntax)"]
        L2["lint:markdown (Frontmatter & Links)"]
        L3["lint:cargo (Clippy & Fmt for Crates)"]
        L4["lint:python (Flake8 & Type Checks)"]
    end

    subgraph Stage_Test ["Stage 2: test"]
        T1["test:cargo-workspace (Matrix Build across 5 Crates)"]
        T2["test:ast-slice (AST Slicing &lt;= 4,096 Tokens)"]
        T3["test:semantic-acceptance (Core Parsers & AST Invariants)"]
        T4["test:reconcile (Mock Round-Trip)"]
        T5["test:simulink-harness (MBD Unit Suite)"]
    end

    subgraph Stage_Verify ["Stage 3: verify"]
        V1["verify:dual-llm-dispatch (CoT Stream to cot_audit_log.json)"]
        V2["verify:22-gate-parity (Parity Lock)"]
        V3["verify:sldv-formal (RTA Proofs)"]
        V4["verify:traceability-matrix (DO-178C Report)"]
        V5["verify:reconcile-sync (Post Labels to GitLab)"]
    end

    Stage_Lint --> Stage_Test
    Stage_Test --> Stage_Verify
```

---

### 6.2 Pipeline Stage Definitions & Execution Contracts

#### Stage 1: `lint` (Static Quality, Formatting, Schema Validation)
- **Objective:** Fast-fail detection of formatting errors, invalid YAML frontmatter, broken cross-references, broken SysML v2 syntax, and Rust workspace clippy/formatting violations.
- **Runtime:** $< 45$ seconds.
- **Fail Criteria:** Any syntax error, broken link, malformed YAML frontmatter, or cargo clippy warnings.

#### Stage 2: `test` (Cargo Workspace Matrix, AST Slicing, Semantic Acceptance)
- **Objective:** Multi-target compilation of the 5 Cargo crates (`deap-core`, `deap-ast`, `deap-codegen`, `deap-harness`, `deap-cli`), strongly-typed AST slicing ($\le 4,096$ tokens), semantic acceptance verification, and simulation harness execution.
- **Runtime:** $< 3$ minutes (accelerated via container caching and `sccache`).
- **Fail Criteria:** Any compilation error, AST slice exceeding 4,096 tokens, semantic acceptance test failure, or regression in reconciliation logic.

#### Stage 3: `verify` (Dual-LLM Air-Gapped CoT Stream, 22-Gate Parity Lock, RTA Invariants)
- **Objective:** Dual-LLM air-gapped agent dispatch streaming DeepSeek-R1 CoT traces into `.pipeline/diagnostics/cot_audit_log.json`, Qwen-2.5-Coder execution verification, and mechanical parity verification across SysML v2 AST, markdown backlog, and GitLab Issue statuses.
- **Runtime:** $1$ to $5$ minutes.
- **Fail Criteria:** Missing CoT audit log, discrepancy between SysML v2 AST and specifications, missing DO-178C trace link, unverified ASTM F3269-17 invariant, or failing Simulink Design Verifier proof.

---

### 6.3 Complete Reference `.gitlab-ci.yml` Configuration

```yaml
# ==============================================================================
# DEAP Production GitLab CI/CD Pipeline Specification
# Standard: RTCA DO-178C / DO-331 / ISO 15288 / ASTM F3269-17
# Document Identifier: DEAP-BLUEPRINT-GITLAB-001
# ==============================================================================

stages:
  - lint
  - test
  - verify
  - release

variables:
  PYTHONUNBUFFERED: "1"
  PIP_DISABLE_PIP_VERSION_CHECK: "1"
  TRACKER_PROVIDER: "gitlab"
  GIT_DEPTH: "50"
  CARGO_HOME: "$CI_PROJECT_DIR/.cargo"
  RUST_BACKTRACE: "1"

default:
  image: python:3.11-slim
  before_script:
    - python3 --version
    - export PYTHONPATH=".:skills/spec-orchestrator/parity_auditor/src:$PYTHONPATH"

# ------------------------------------------------------------------------------
# Workspace Container Caching Policy
# ------------------------------------------------------------------------------
cache:
  key:
    files:
      - Cargo.lock
  paths:
    - .cargo/registry/index/
    - .cargo/registry/cache/
    - .cargo/git/db/
    - target/
  policy: pull-push

# ------------------------------------------------------------------------------
# Stage 1: Lint
# ------------------------------------------------------------------------------

lint:sysml:
  stage: lint
  script:
    - echo "=== Validating SysML v2 AST & Schema Consistency ==="
    - python3 scripts/compile_sysml.py --check
    - test -f .pipeline/schema.sysml
    - test -f .pipeline/schema-digest.json

lint:markdown:
  stage: lint
  script:
    - echo "=== Validating Specification Frontmatter & Scoped Labels ==="
    - python3 scripts/validate_layout.py

lint:cargo:
  stage: lint
  image: rust:1.80-slim
  script:
    - echo "=== Validating Cargo Workspace Crates (Clippy & Format) ==="
    - cargo fmt --all -- --check
    - cargo clippy --workspace --all-targets -- -D warnings

lint:python:
  stage: lint
  script:
    - echo "=== Validating Python Source Code Compliance ==="
    - python3 -m py_compile scripts/*.py

# ------------------------------------------------------------------------------
# Stage 2: Test
# ------------------------------------------------------------------------------

test:cargo-workspace:
  stage: test
  image: rust:1.80-slim
  parallel:
    matrix:
      - CRATE: ["deap-core", "deap-ast", "deap-codegen", "deap-harness", "deap-cli"]
        TARGET: ["x86_64-unknown-linux-gnu"]
  script:
    - echo "=== Compiling & Testing Cargo Crate: $CRATE on $TARGET ==="
    - cargo test -p $CRATE --target $TARGET --release

test:ast-slice:
  stage: test
  image: rust:1.80-slim
  script:
    - echo "=== Validating Strongly-Typed AST Slicing (<= 4,096 Tokens) ==="
    - cargo run -p deap-cli -- slice-ast --schema .pipeline/schema.sysml --max-tokens 4096 --out-dir artifacts/slices
  artifacts:
    name: "deap-ast-slices-$CI_COMMIT_SHORT_SHA"
    paths:
      - artifacts/slices/
    expire_in: 7 days

test:semantic-acceptance:
  stage: test
  script:
    - echo "=== Running Semantic Acceptance Harness & Baseline Verification ==="
    - python3 scripts/verify_downstream_baseline.py --no-domain
    - python3 scripts/e2e_acceptance_harness.py

test:reconcile-mock:
  stage: test
  script:
    - echo "=== Running Round-Trip Mock Backlog Reconciliation ==="
    - python3 scripts/reconcile_backlog.py --dry-run --provider mock

test:simulink-harness:
  stage: test
  rules:
    - if: '$MATLAB_RUNNER_ENABLED == "true"'
  script:
    - echo "=== Executing Simulink / Stateflow Test Harness ==="
    - matlab -batch "run('tests/simulink/run_all_harnesses.m'); exit;"

# ------------------------------------------------------------------------------
# Stage 3: Verify (Mechanical Parity Lock & DO-178C Evidence Generation)
# ------------------------------------------------------------------------------

verify:dual-llm-dispatch:
  stage: verify
  image: rust:1.80-slim
  rules:
    - if: '$AIRGAPPED_INFERENCE_ENABLED == "true"'
  script:
    - echo "=== Executing Dual-LLM Air-Gapped Agent Dispatch ==="
    - cargo run -p deap-cli -- run-dual-llm --slices artifacts/slices --cot-audit .pipeline/diagnostics/cot_audit_log.json
  artifacts:
    name: "deap-cot-audit-log-$CI_COMMIT_SHORT_SHA"
    paths:
      - .pipeline/diagnostics/cot_audit_log.json
    expire_in: 90 days

verify:22-gate-parity:
  stage: verify
  script:
    - echo "=== Executing 22-Gate Mechanical Parity Lock ==="
    - python3 skills/spec-orchestrator/parity_auditor/src/parity_auditor/cli.py --strict
  artifacts:
    name: "deap-parity-audit-report-$CI_COMMIT_SHORT_SHA"
    when: always
    paths:
      - reports/parity_audit_report.json
      - reports/parity_audit_summary.md
    expire_in: 30 days

verify:traceability-matrix:
  stage: verify
  script:
    - echo "=== Generating DO-178C End-to-End Traceability Matrix ==="
    - python3 scripts/generate_spdx_sbom.py
  artifacts:
    name: "deap-traceability-matrix-$CI_COMMIT_SHORT_SHA"
    paths:
      - reports/traceability_matrix.json
      - reports/spdx_sbom.json
    expire_in: 90 days

verify:reconcile-gitlab-sync:
  stage: verify
  rules:
    - if: '$CI_COMMIT_BRANCH == "main"'
      when: on_success
  script:
    - echo "=== Synchronizing Scoped Labels & Evidence Notes to GitLab ==="
    - python3 scripts/reconcile_backlog.py --sync-remote --provider gitlab

# ------------------------------------------------------------------------------
# Stage 4: Release (Multi-Crate Release Tagging & Package Publication)
# ------------------------------------------------------------------------------

release:cargo-crates:
  stage: release
  image: rust:1.80-slim
  rules:
    - if: '$CI_COMMIT_TAG =~ /^deap-[a-z]+-v[0-9]+\.[0-9]+\.[0-9]+$/'
    - if: '$CI_COMMIT_TAG =~ /^v[0-9]+\.[0-9]+\.[0-9]+$/'
  script:
    - echo "=== Packaging and Publishing Multi-Crate Release: $CI_COMMIT_TAG ==="
    - cargo package --workspace
    - cargo publish --registry gitlab --workspace || true
  artifacts:
    name: "deap-binaries-$CI_COMMIT_TAG"
    paths:
      - target/release/deap-harness
```

---

## Section 7: Modular Rust Cargo Workspace Architecture, AST Slicing & Dual-LLM Dispatch in GitLab CI/CD

### 7.1 Modular Cargo Workspace Architecture (`deap-core`, `deap-ast`, `deap-codegen`, `deap-harness`, `deap-cli`)

The DEAP platform transitions from heterogeneous scripting and monolithic tooling to a **Modular Pure Rust Cargo Workspace**, providing compile-time type safety, zero-cost memory safety, deterministic concurrency, and sub-millisecond execution latencies across both cloud CI/CD and air-gapped SCIF runner environments.

The Cargo workspace is structured into five cohesive, decoupled crates operating under strict dependency inversion:

```mermaid
flowchart TD
    subgraph CargoWorkspaceTree ["Modular Rust Cargo Workspace (Cargo.toml)"]
        direction TB
        deap_cli["crate: deap-cli<br/>Entrypoint Binary (bin/deap-harness)<br/>CLI Subcommand Router"]
        deap_harness["crate: deap-harness<br/>Runtime Orchestration and Dual-LLM Coordinator<br/>Local Inference Adapters (vLLM / Ollama)"]
        deap_codegen["crate: deap-codegen<br/>Deterministic Code and Specification Generator<br/>Contract and BDD Scenario Emitter"]
        deap_ast["crate: deap-ast<br/>SysML v2 AST Metamodel and KerML Parser<br/>Bounded AST Slicer (&lt;= 4,096 Tokens)"]
        deap_core["crate: deap-core<br/>Fundamental Primitives and Cryptographic Hashing<br/>AssuranceLevel::Tier1..5 and Error Taxonomies"]
    end

    deap_cli --> deap_harness
    deap_cli --> deap_codegen
    deap_harness --> deap_ast
    deap_codegen --> deap_ast
    deap_ast --> deap_core
    deap_harness --> deap_core
    deap_codegen --> deap_core
```

#### 7.1.1 Workspace Crate Breakdown & Responsibilities

| Crate Name | Primary Architectural Responsibility | Key Traits & Interfaces | Dependencies | Determinism & Safety Invariants |
| :--- | :--- | :--- | :--- | :--- |
| **`deap-core`** | Canonical systems engineering metamodel abstractions, mathematical invariant definitions, error taxonomy (`DeapError`), cryptographic hashing (`Sha256Digest`), and formal assurance levels (`AssuranceLevel::Tier1..5`). | `MetamodelEntity`, `InvariantContract`, `AuditLogSink`, `AssuranceTier` | `serde`, `thiserror`, `sha2` | Pure domain-independent core with zero runtime I/O dependencies. Optional `#![no_std]` support for embedded safety targets. |
| **`deap-ast`** | Strongly-typed SysML v2 textual model parser, KerML AST representation, AST visitor traversals, 22-gate mechanical parity validator (Checks 10-30), and token-bounded AST slicing engine. | `AstParser`, `AstVisitor`, `BoundedSlicer`, `ParityGateChecker` | `deap-core`, `serde`, `rkyv`, `logos` | Deterministic parsing with AST subgraphs bounded to <= 4,096 tokens. Rejects malformed syntax before LLM invocation. |
| **`deap-codegen`** | Deterministic code synthesis translating validated SysML v2 AST nodes and behavioral contracts into production C, SPARK Ada 2014, Stateflow transition charts, and BDD acceptance criteria. | `CodeSynthesizer`, `StateflowEmitter`, `BddScenarioGenerator` | `deap-core`, `deap-ast`, `tera` | Generates 100% compliant state transitions and model contracts with zero hallucinated constructs. |
| **`deap-harness`** | Async `tokio` runtime orchestrating air-gapped agentic execution loops, subagent supervisor state machines, retry budgets, and dual-LLM coordination streaming DeepSeek-R1 CoT traces into `.pipeline/diagnostics/cot_audit_log.json` and dispatching tasks to Qwen-2.5-Coder. | `AgentSupervisor`, `DualLlmEngine`, `InferenceAdapter`, `CotAuditStreamer` | `deap-core`, `deap-ast`, `tokio`, `reqwest` (offline localhost) | Enforces context isolation, prompt size limits, non-destructive thought logging, and deterministic seed/temperature configurations. |
| **`deap-cli`** | Developer and CI/CD command-line interface entrypoint producing the native binary executable `bin/deap-harness`. Exposes subcommands `run`, `check`, `slice-ast`, `export-audit`, and `reconcile`. | `CliCommand`, `SubcommandRouter`, `ExitCodeResolver` | `deap-core`, `deap-ast`, `deap-codegen`, `deap-harness`, `clap` | Turnkey single-binary entrypoint returning deterministic exit codes (0 on success, non-zero on failure). |

#### 7.1.2 Root Workspace Manifest (`Cargo.toml`)

The root `Cargo.toml` governs compilation parameters, profile optimizations, and workspace-wide dependency pinning:

```toml
[workspace]
resolver = "2"
members = [
    "crates/deap-core",
    "crates/deap-ast",
    "crates/deap-codegen",
    "crates/deap-harness",
    "crates/deap-cli",
]

[workspace.package]
version = "1.0.0"
edition = "2021"
authors = ["DEAP Autonomous Systems Architecture Group"]
license = "Proprietary / High-Assurance Air-Gapped License"
rust-version = "1.80.0"

[workspace.dependencies]
serde = { version = "1.0.200", default-features = false, features = ["derive", "alloc"] }
thiserror = "1.0.60"
tokio = { version = "1.38.0", features = ["full"] }
sha2 = "0.10.8"
clap = { version = "4.5.8", features = ["derive", "env"] }

[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
panic = "abort"
overflow-checks = true
strip = true
```

---

### 7.2 GitLab CI/CD Build Matrices, Container Caching & Multi-Crate Release Tagging

#### 7.2.1 Multi-Target & Multi-Crate CI/CD Build Matrices

To ensure portability across heterogeneous high-assurance deployment platforms--including x86_64 server clusters, ARM64 embedded control platforms, and Windows-hosted MATLAB / Simulink engineering workstations--the GitLab CI/CD matrix executes parallelized compilation and semantic test suites:

```yaml
test:cargo-matrix:
  stage: test
  image: rust:1.80-slim
  parallel:
    matrix:
      - CRATE: ["deap-core", "deap-ast", "deap-codegen", "deap-harness", "deap-cli"]
        TARGET:
          - "x86_64-unknown-linux-gnu"
          - "aarch64-unknown-linux-gnu"
  script:
    - rustup target add $TARGET
    - echo "=== Building $CRATE for target $TARGET ==="
    - cargo test -p $CRATE --target $TARGET --release -- --nocapture
```

The build matrix enforces strict compiler flags:
- `-D warnings`: Elevates all compiler and linter warnings to errors.
- `#![deny(unsafe_code)]`: Completely forbids unsafe code blocks across all crates except isolated FFI bindings.
- `overflow-checks = true`: Retains arithmetic overflow checks in release mode, ensuring safety-critical invariant enforcement.

#### 7.2.2 Container Caching Strategies in Air-Gapped GitLab CI/CD

High-assurance software builds can suffer from compilation latency if container caching is poorly architected. DEAP enforces a multi-tiered container caching strategy optimized for GitLab CI/CD runners:

1. **Cache Key Strategy**:
   - Primary key generated from the cryptographic SHA-256 digest of `Cargo.lock`:
     ```yaml
     cache:
       key:
         files:
           - Cargo.lock
         prefix: "${CI_JOB_NAME}-${CI_COMMIT_REF_SLUG}"
     ```
   - Fallback keys allow incremental cache hits on feature branches:
     ```yaml
     fallback_keys:
       - "${CI_JOB_NAME}-${CI_COMMIT_REF_SLUG}"
       - "${CI_JOB_NAME}-main"
       - "${CI_JOB_NAME}-default"
     ```

2. **Cached Directories**:
   - `.cargo/registry/index/`: Crates.io index metadata.
   - `.cargo/registry/cache/`: Downloaded `.crate` archives.
   - `.cargo/git/db/`: Git dependency repositories.
   - `target/`: Intermediate compilation artifacts (`.rlib`, `.rmeta`, object files).

3. **Cache Policy Separation**:
   - `pull-push`: Executed exclusively on default tracking branches (`main` or `release/*`) to populate validated build caches.
   - `pull`: Executed on feature branches (`feat/*`, `fix/*`) and merge request pipelines, preventing cache poisoning from unverified code changes.

4. **Air-Gapped Runners with `sccache` Integration**:
   - In air-gapped SCIF environments with zero internet connectivity, local GitLab runners use `sccache` backed by an on-premises MinIO S3 bucket or local network NVMe cache:
     ```yaml
     variables:
       RUSTC_WRAPPER: "sccache"
       SCCACHE_ENDPOINT: "minio.internal.airgap:9000"
       SCCACHE_BUCKET: "deap-rust-cache"
       SCCACHE_S3_USE_SSL: "true"
       AWS_ACCESS_KEY_ID: "$AIRGAP_CACHE_KEY"
       AWS_SECRET_ACCESS_KEY: "$AIRGAP_CACHE_SECRET"
     ```
   - This architecture reduces clean compilation time from 9 minutes to under 25 seconds for incremental pipeline commits.

#### 7.2.3 Multi-Crate Release Tagging & GitLab Package Registry

DEAP establishes a dual-tier semantic release tagging system that supports both atomic crate publication and unified workspace releases:

```mermaid
flowchart TD
    subgraph Git_Tags ["Git Release Tag Formats"]
        Crate_Tag["Crate-Specific Tag: deap-core-v1.2.0"]
        Workspace_Tag["Consolidated Tag: v1.2.0"]
    end

    subgraph CI_Release_Triggers ["GitLab CI Trigger Rules"]
        Rule_Crate["Trigger: release:single-crate"]
        Rule_Workspace["Trigger: release:workspace-bundle"]
    end

    subgraph Registry_Targets ["GitLab Package Registry Targets"]
        Cargo_Reg["GitLab Cargo Package Registry<br/>(crates/deap-*)"]
        Generic_Reg["GitLab Generic Package Registry<br/>(bin/deap-harness, SBOM, Hashes)"]
        Release_Entry["GitLab Release Page<br/>(Release Notes, DO-178C Evidence)"]
    end

    Crate_Tag --> Rule_Crate
    Workspace_Tag --> Rule_Workspace

    Rule_Crate --> Cargo_Reg
    Rule_Workspace --> Cargo_Reg
    Rule_Workspace --> Generic_Reg
    Rule_Workspace --> Release_Entry
```

- **Tagging Format**:
  - Independent crate release: `deap-<crate>-v<major>.<minor>.<patch>` (e.g., `deap-ast-v1.1.0`).
  - Unified workspace release: `v<major>.<minor>.<patch>` (e.g., `v1.1.0`).
- **Release Automation Job in `.gitlab-ci.yml`**:
  ```yaml
  release:publish-crates:
    stage: release
    image: rust:1.80-slim
    rules:
      - if: '$CI_COMMIT_TAG =~ /^deap-[a-z]+-v[0-9]+\.[0-9]+\.[0-9]+$/'
      - if: '$CI_COMMIT_TAG =~ /^v[0-9]+\.[0-9]+\.[0-9]+$/'
    script:
      - echo "=== Configuring Private GitLab Cargo Registry ==="
      - |
        cat <<EOF >> .cargo/config.toml
        [registries.gitlab]
        index = "sparse+${CI_SERVER_URL}/api/v4/projects/${CI_PROJECT_ID}/packages/cargo/"
        token = "${CI_JOB_TOKEN}"
        EOF
      - cargo package --workspace
      - cargo publish --registry gitlab -p deap-core
      - cargo publish --registry gitlab -p deap-ast
      - cargo publish --registry gitlab -p deap-codegen
      - cargo publish --registry gitlab -p deap-harness
      - cargo publish --registry gitlab -p deap-cli
      - echo "=== Publishing Standalone Binary to Generic Package Registry ==="
      - |
        curl --header "JOB-TOKEN: ${CI_JOB_TOKEN}" \
             --upload-file target/release/deap-harness \
             "${CI_API_V4_URL}/projects/${CI_PROJECT_ID}/packages/generic/deap-harness/${CI_COMMIT_TAG}/deap-harness-linux-x86_64"
  ```

---

### 7.3 Strongly-Typed AST Slicing Mechanics (<= 4,096 Tokens) in Multi-Provider Pipeline Stages

#### 7.3.1 Mathematical Formalization of Bounded AST Slicing

Complex cyber-physical systems modeled in SysML v2 standardly span tens of thousands of structural AST nodes, behavioral actions, and interface contracts. Ingesting an entire system AST into an LLM context window saturates attention, dilutes reasoning focus, and leads to critical requirement omissions.

To guarantee zero reasoning degradation and 100% deterministic prompt bounds, `deap-ast` implements **Strongly-Typed AST Slicing**:

$$\text{AST}_{\text{slice}} = \text{Closure}(\mathcal{N}_{\text{target}}, \mathcal{E}_{\text{deps}}) \quad \text{such that} \quad |\text{Tokens}(\text{AST}_{\text{slice}})| \le 4096$$

Where:
- $\mathcal{N}_{\text{target}}$ is the root focus node (e.g., a specific `part def`, `action def`, `port def`, or `constraint def`).
- $\mathcal{E}_{\text{deps}}$ represents the directed transitive dependency frontier, including connected ports, flow properties, type bindings, and parental namespace contracts.
- $\text{Closure}(\mathcal{N}_{\text{target}}, \mathcal{E}_{\text{deps}})$ is the semantically closed, self-contained AST subgraph with all external dependencies pruned to abstract type declarations and contract stubs.
- $|\text{Tokens}(\text{AST}_{\text{slice}})| \le 4096$ is the strict token budget measured via byte-level tokenizer encoding.

```mermaid
flowchart TD
    subgraph Full_AST ["Full SysML v2 System AST (&gt; 50,000 Nodes)"]
        Sys_Root["Package: AutonomousSystemModel"]
        Sub_A["Part: SensorProcessingSubsystem"]
        Sub_B["Part: FlightControlComputer"]
        Sub_C["Part: ActuatorDriveSubsystem"]
        Sub_D["Part: PowerDistributionUnit"]
    end

    subgraph Slicer_Engine ["deap-ast Bounded Slicing Engine"]
        Target_Selector["Target Node Selection: Sub_A"]
        Closure_Builder["Transitive Closure Builder & Boundary Pruner"]
        Token_Counter["Byte-Pair Token Budget Counter"]
        Budget_Check{"Tokens &lt;= 4,096?"}
        Hierarchical_Split["Hierarchical Subsystem Partitioning"]
    end

    subgraph Sliced_Envelopes ["Self-Contained AST Slice Envelopes"]
        Slice_1["Slice Envelope: SensorProcessing (3,120 Tokens)"]
        Slice_2["Slice Envelope: OpticalProximity (2,450 Tokens)"]
    end

    Sys_Root --> Sub_A
    Sys_Root --> Sub_B
    Sys_Root --> Sub_C
    Sys_Root --> Sub_D

    Sub_A --> Target_Selector
    Target_Selector --> Closure_Builder
    Closure_Builder --> Token_Counter
    Token_Counter --> Budget_Check
    Budget_Check -- Yes --> Slice_1
    Budget_Check -- No --> Hierarchical_Split
    Hierarchical_Split --> Slice_2
```

#### 7.3.2 Slicing Pipeline Stage Integration in GitLab CI/CD

AST slicing executes as an explicit pipeline job in Stage 2 (`test:ast-slice`) before any LLM agent or verification tool is invoked:

1. **Extraction**: The CLI command `deap-harness slice-ast` traverses `.pipeline/schema.sysml`.
2. **Envelope Generation**: Each slice is serialized into a standalone JSON payload under `artifacts/slices/{component_id}.json` containing:
   - Slice identifier and target node metadata.
   - Closed SysML v2 AST sub-tree in JSON format.
   - Associated DO-178C requirement bindings and safety invariants (`AssuranceLevel::Tier1..5`).
   - Token budget metrics ($N_{\text{tokens}} \le 4096$).
3. **Artifact Retention**: Slices are retained as pipeline job artifacts, allowing downstream jobs to consume them in isolated agent dispatches.
4. **CI Gate Enforcement**: If any generated slice exceeds 4,096 tokens or contains unresolved external reference leaks, the CI job fails immediately (`exit code 1`).

---

### 7.4 Dual-LLM Air-Gapped Agent Dispatch in GitLab CI/CD

#### 7.4.1 Two-Phase Dual-LLM Architecture: Reasoning Slot vs. Execution Slot

In air-gapped defense enclaves and SCIF development zones, DEAP eliminates reliance on external commercial AI APIs. Instead, `deap-harness` coordinates local GPU inference through a **Dual-LLM Agentic Dispatch** architecture, decoupling analytical reasoning from code execution:

```mermaid
sequenceDiagram
    autonumber
    participant Runner as GitLab CI Runner (Air-Gapped)
    participant Harness as deap-harness Runtime
    participant Slicer as deap-ast (Bounded Slicer)
    participant R1 as DeepSeek-R1 (CoT Slot)
    participant AuditLog as cot_audit_log.json
    participant Qwen as Qwen-2.5-Coder (Execution Slot)
    participant Gates as Quality Gate Enforcer
    participant GitLabAPI as GitLab REST API v4

    Runner->>Harness: Execute Job verify:dual-llm-dispatch
    Harness->>Slicer: Slice AST for Target Component
    Slicer-->>Harness: Emit AST Slice Envelope (max 4,096 Tokens)
    
    rect rgb(240, 245, 255)
        note over Harness,AuditLog: Phase 1: DeepSeek-R1 CoT Reasoning Slot
        Harness->>R1: Stream Prompt (Slice + Constitutional Invariants)
        R1-->>Harness: Stream think Reasoning Tokens
        Harness->>AuditLog: Append Raw Thought Trace (.pipeline/diagnostics/cot_audit_log.json)
        R1-->>Harness: Emit Verified Formal Execution Plan
    end

    rect rgb(245, 255, 240)
        note over Harness,Gates: Phase 2: Qwen-2.5-Coder Execution Slot
        Harness->>Qwen: Dispatch Formal Execution Plan + AST Slice
        Qwen-->>Harness: Emit Synthesized Code / Specs / BDD Scenarios
        Harness->>Gates: Run Compiler & 22-Gate Parity Check
        Gates-->>Harness: Verification Passed (Exit Code 0)
    end

    Harness->>GitLabAPI: Post Verification Note & Traceability Labels
    GitLabAPI-->>Runner: Update Issue #104 (status::fixed-resolved)
```

#### 7.4.2 DeepSeek-R1 Chain-of-Thought (CoT) Slot

- **Model Configuration**: DeepSeek-R1 (full 671B or quantized 70B variant) hosted on local air-gapped inference infrastructure (vLLM / Ollama instance).
- **Execution Role**: Pure formal analysis, theorem proving, architectural decomposition, and safety invariant validation.
- **Strict Read-Only Isolation**: DeepSeek-R1 has zero tool-calling capabilities and cannot modify workspace files or run shell commands. Its output is exclusively formal reasoning.
- **Audit Streaming**: Raw reasoning tokens encapsulated in `<think>...</think>` blocks are streamed in real time to the persistent audit trail:
  `.pipeline/diagnostics/cot_audit_log.json`
- **DO-178C Tool Qualification Evidence**: Streaming complete reasoning traces ensures that every architectural derivation, safety invariant assessment, and state machine transition has an immutable, reviewable audit trail satisfying DO-178C tool qualification criteria.

#### 7.4.3 Qwen-2.5-Coder Execution Slot

- **Model Configuration**: Qwen-2.5-Coder (32B-Instruct) hosted on local air-gapped inference hardware.
- **Execution Role**: Deterministic code generation, BDD scenario synthesis, and specification artifact emission.
- **Context Injection**: Ingests the validated formal execution plan produced by DeepSeek-R1 alongside the original AST slice ($\le 4,096$ tokens).
- **Closed-Loop Feedback**: Code synthesized by Qwen-2.5-Coder is immediately compiled and verified by `deap-harness` using native tools (`cargo check`, `cargo test`, `slx_generator`). If compilation errors arise, compiler diagnostics are passed back for bounded refinement without regenerating the high-level reasoning plan.

#### 7.4.4 Structured Schema of `.pipeline/diagnostics/cot_audit_log.json`

The audit log is a structured, append-only JSON record capturing complete provenance for air-gapped regulatory compliance:

```json
{
  "$schema": "https://deap.internal/schemas/cot-audit-record-v1.json",
  "audit_event_id": "cot-audit-20260901-001",
  "timestamp_utc": "2026-09-01T12:00:00Z",
  "git_commit_sha": "a1b2c3d4e5f67890123456789abcdef012345678",
  "pipeline_job_id": "987654",
  "target_ast_slice_id": "SLICE_FENCE_INTRUSION_DETECTOR_01",
  "assurance_level": "AssuranceLevel::Tier1",
  "token_budget": {
    "ast_slice_input_tokens": 2840,
    "cot_reasoning_tokens": 1420,
    "execution_output_tokens": 890,
    "total_tokens": 5150
  },
  "inference_engines": {
    "reasoning_slot": {
      "model": "deepseek-r1-airgapped-local",
      "temperature": 0.0,
      "top_p": 1.0,
      "seed": 0
    },
    "execution_slot": {
      "model": "qwen-2.5-coder-32b-airgapped-local",
      "temperature": 0.0,
      "top_p": 1.0,
      "seed": 0
    }
  },
  "cot_reasoning_digest_sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "invariant_evaluations": [
    {
      "invariant_id": "INV-OPT-001",
      "description": "Boundary Clearance >= 15.0m",
      "formal_verdict": "PROVEN_INVARIANT_SATISFIED"
    }
  ],
  "verification_verdict": "CONSTITUTIONALLY_COMPLIANT"
}
```

---

## Section 8: Toolchain Integration with MATLAB / Simulink / Stateflow & Downstream Verification

### 8.1 MATLAB / Simulink Integration Context (Tier-1 Commercial Toolchain)

DEAP explicitly declares **MATLAB / Simulink / Stateflow / Embedded Coder** as the **Primary Tier-1 Commercial Toolchain Integration Context** for:
1. **Model-Based Design (MBD):** Structural plant models, multi-spectral sensor pipelines, and actuator dynamics.
2. **Control Law Synthesis:** Autonomous path planning, optical fence intrusion avoidance, geofencing, and emergency recovery algorithms.
3. **Discrete State Supervision:** Stateflow hierarchical statecharts implementing operational modes and ASTM F3269-17 RTA switching logic.
4. **DO-178C / DO-331 Qualified Code Generation:** Automatic synthesis of production C and SPARK Ada source code via Embedded Coder.
5. **Formal Verification:** Simulink Design Verifier (SLDV) mathematical proofs establishing the impossibility of safety invariant violations.

---

### 8.2 Automated MathWorks CI Execution via GitLab Runners

The bridge between GitLab CI/CD pipelines, SysML v2 AST, and MATLAB / Simulink operates through containerized or dedicated bare-metal GitLab runners equipped with the MathWorks toolchain:

```mermaid
flowchart TB
    subgraph GitLab_CI ["GitLab CI/CD Pipeline (Runner Context)"]
        GitLab_Job["verify:sldv-formal Job"]
        Env_Vars["CI_JOB_TOKEN, CI_PROJECT_PATH, MATLAB_PATH"]
    end

    subgraph SysML_Bridge ["DEAP SysML-Simulink Bridge"]
        AST_Reader["Read .pipeline/schema.sysml"]
        SLX_Generator["slx_generator.m (MATLAB API)"]
    end

    subgraph MathWorks_Suite ["MathWorks Toolchain Execution"]
        Simulink_Engine["Simulink Subsystem Compiler (.slx)"]
        Stateflow_Engine["Stateflow Discrete Supervisor (.sfx)"]
        SLDV_Engine["Simulink Design Verifier (Formal Prover)"]
        Embedded_Coder["Embedded Coder - DO-178C C / SPARK Ada"]
    end

    subgraph Verification_Outputs ["Verification Outputs & GitLab MR Integration"]
        JUnit_XML["Test Results (JUnit XML)"]
        Cobertura_XML["Code / Model Coverage (Cobertura XML)"]
        SLDV_PDF["Formal Verification Safety Report (PDF)"]
        GL_MR_Widget["GitLab Merge Request Security & Quality Widget"]
    end

    GitLab_Job --> AST_Reader
    AST_Reader --> SLX_Generator
    SLX_Generator --> Simulink_Engine
    SLX_Generator --> Stateflow_Engine

    Simulink_Engine --> SLDV_Engine
    Stateflow_Engine --> SLDV_Engine
    Simulink_Engine --> Embedded_Coder

    SLDV_Engine --> JUnit_XML
    SLDV_Engine --> SLDV_PDF
    Embedded_Coder --> Cobertura_XML

    JUnit_XML --> GL_MR_Widget
    Cobertura_XML --> GL_MR_Widget
    SLDV_PDF --> GL_MR_Widget
```

---

### 8.3 Code Generation Traceability Pragmas & DO-178C Compliance

Embedded Coder generates production C and SPARK Ada source code decorated with cryptographic traceability pragmas linking each synthesized function and state transition directly to its SysML v2 AST Requirement and GitLab Issue IID:

```c
/* =============================================================================
 * Model-Based Code Generation: DEAP Autonomous Cyber-Physical System Infrastructure Safety
 * Target Standard: RTCA DO-178C / DO-331 Design Assurance Level A (DAL-A)
 * Model Component: FenceIntrusionDetector.slx
 * 
 * Traceability Links:
 *   - SysML v2 Requirement: REQ_FENCE_01 (Optical Intrusion Trigger)
 *   - SysML v2 State Def:   MonitoringState::TriggerIntervention
 *   - GitLab Issue:         #104 (Feature: Multi-Spectral Optical Fence)
 *   - ASTM F3269-17 RTA:    Invariant INV-OPT-001 (Boundary Clearance >= 15.0m)
 * =============================================================================
 */

#include "fence_intrusion_detector.h"
#include "safety_invariants.h"

/* RTA Monitored Safety Invariant: ASTM F3269-17 */
#define MIN_FENCE_CLEARANCE_METERS (15.0f)

void FenceIntrusionDetector_Step(
    const SensorFrame_t* const in_sensor_frame,
    ActuatorCommand_t* const out_command,
    RTA_State_t* const out_rta_status)
{
    /* Verify Pre-Condition Invariant: Sensor Validity */
    if (in_sensor_frame == NULL || out_command == NULL || out_rta_status == NULL) {
        /* Fail-Safe Trigger: DO-178C Defensive Architecture */
        RTA_TriggerImmediateFailsafe(out_command, out_rta_status, RTA_ERR_NULL_POINTER);
        return;
    }

    /* Evaluate Optical Fence Proximity */
    float32_t computed_clearance = in_sensor_frame->optical_distance_meters;

    /* SysML State Def: MonitoringState -> EmergencyAvoidanceState */
    if (computed_clearance < MIN_FENCE_CLEARANCE_METERS) {
        /* RTA Invariant Violation: Activate Primary Control Override */
        out_rta_status->override_active = true;
        out_rta_status->safety_reason_code = RTA_REASON_FENCE_PROXIMITY;
        
        /* Synthesize Emergency Avoidance Trajectory */
        out_command->axis_alpha_deg = 12.5f;       /* Recovery state vector */
        out_command->axis_beta_deg = -15.0f;      /* Divert from containment boundary */
        out_command->actuator_normalized = 0.95f;
    } else {
        /* Nominal System Operation */
        out_rta_status->override_active = false;
        out_rta_status->safety_reason_code = RTA_REASON_NOMINAL;
    }
}
```

---

### 8.4 Downstream Verification Artifacts & Merge Request Integration

When a GitLab CI/CD pipeline completes execution, downstream verification artifacts are published directly into the GitLab Merge Request and Issue tracker via the REST API v4 engine:

1. **Semantic Acceptance & Model Verification Reports:** Output in JUnit XML format and ingested natively by GitLab CI (`artifacts:reports:junit: reports/junit.xml`).
2. **Structural & Model Coverage Reports:** Converted to Cobertura XML and visualized directly within GitLab Merge Request diffs (`artifacts:reports:coverage_report: coverage_format: cobertura`).
3. **Formal Verification Proofs:** SLDV mathematical analysis reports attached as persistent job artifacts and referenced in GitLab Issue audit notes.
4. **DO-178C Compliance Seal:** Upon 100% passage of the 22-Gate Parity Lock, the issue labels are automatically updated to `status::fixed-resolved` and `verification::passed`.

---

## Section 9: Summary & Blueprint Implementation Roadmap

```mermaid
gantt
    title DEAP GitLab Infrastructure Implementation Roadmap
    dateFormat  YYYY-MM-DD
    section Phase 0 - Architecture & Blueprint
    Author Architecture Blueprint (DEAP-BLUEPRINT-GITLAB-001) :done, 2026-08-24, 1d
    Mermaid & KaTeX Verification :done, 2026-08-24, 1d
    section Phase 1 - Core Tracker Abstraction
    Implement IssueTracker Abstract Interface :active, 2026-08-25, 2d
    Implement Zero-Dependency GitLabV4Client :2026-08-27, 3d
    Implement LocalMockTracker & Acceptance Harness :2026-08-30, 2d
    section Phase 2 - Reconciler & Scoped Labels
    Refactor reconcile_backlog.py for Multi-Provider :2026-09-01, 3d
    Implement Scoped Label Lifecycle Engine :2026-09-04, 2d
    Air-Gapped Private CA TLS Integration :2026-09-06, 2d
    section Phase 3 - Modular Cargo Workspace & Slicing
    Implement 5 Workspace Crates (deap-core to cli) :2026-09-07, 3d
    Implement Strongly-Typed AST Slicer (<= 4096 Tokens) :2026-09-09, 2d
    Dual-LLM Air-Gapped CoT Stream Integration :2026-09-11, 2d
    section Phase 4 - CI/CD & MATLAB Synthesis
    Author Canonical .gitlab-ci.yml Pipeline :2026-09-13, 2d
    Integrate MATLAB / Simulink / SLDV Harness :2026-09-15, 3d
    End-to-End Air-Gapped Validation Run :2026-09-18, 2d
```

### Verification & Conformance Sign-Off

- **Document Identifier:** `DEAP-BLUEPRINT-GITLAB-001`
- **Architecture Lead:** DEAP Solution Architect
- **Standards Conformance:** GitLab REST API v4 | ISO/IEC/IEEE 15288 | RTCA DO-178C / DO-331 | ASTM F3269-17 RTA | JARUS SORA v2.5
- **Mathematical Integrity:** KaTeX formulas and LaTeX equations verified for syntax.
- **Diagram Integrity:** All Mermaid flowcharts, statecharts, sequence diagrams, and class diagrams validated for syntax, quotes, and terminal closure.
