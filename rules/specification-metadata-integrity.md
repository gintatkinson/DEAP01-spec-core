<!-- Copyright Gint Atkinson, gint.atkinson@gmail.com -->

# Rule: Specification Metadata Integrity & Native Markdown Metadata Tables

**ALWAYS enforce:** All engineering specification files across the repository (`docs/epics/`, `docs/features/`, `docs/user-stories/`, and `docs/use-cases/`) MUST start at lines 1–10 with a native CommonMark two-column Markdown Metadata Table. Raw `--- ... ---` YAML frontmatter blocks are strictly forbidden in specification markdown documents.

## Scope and Normative Authority

**This file is the single normative home for specification metadata integrity, frontmatter elimination, and document-relative schema traceability across all backlog artifacts in the Digital Engineering Autonomous Pipeline (DEAP).**

This standard applies unconditionally to all specification documents residing under:
- `docs/epics/`
- `docs/features/`
- `docs/user-stories/`
- `docs/use-cases/`

## Hard Constraints

1. **Strict Prohibition of Raw YAML Frontmatter**:
   - Raw `--- ... ---` YAML frontmatter blocks are **strictly forbidden** in repository specification markdown files.
   - Frontmatter blocks render as unstructured raw text dumps or unformatted blocks in many web file viewers, GitLab Work Items, GitHub issue previews, and downstream documentation renders.
   - All metadata previously carried in YAML frontmatter blocks MUST be converted to native CommonMark two-column tables.

2. **Native CommonMark Two-Column Metadata Table**:
   - Every specification document MUST begin within lines 1–10 (optionally preceded only by copyright comments) with a native CommonMark two-column table with the exact header:
     ```markdown
     | Attribute | Specification Detail |
     | :--- | :--- |
     ```
   - The table MUST contain all mandatory attributes defined for its specification type.
   - Column 1 contains the attribute name; Column 2 contains the corresponding specification value.

3. **Valid Document-Relative Specification Source Locators**:
   - The `Specification Source` attribute MUST use a valid document-relative path (e.g., `../../schema/SystemModel.sysml` or `[schema/SystemModel.sysml](../../schema/SystemModel.sysml)`).
   - Absolute filesystem paths, bare repository-root paths without relative navigation, or dangling references that do not resolve from the specification's filesystem directory are strictly prohibited.
   - This ensures offline traceability and navigation directly from file browsers and markdown viewers.

## Mandatory Attributes per Specification Type

Each specification artifact type MUST include its designated mandatory attributes in the metadata table:

### 1. Epics (`docs/epics/`)

Epics define high-level system packages, major structural assemblies, or top-level subsystems.

- **Mandatory Attributes**:
  - `Issue ID`: Tracker issue identifier or integer ID.
  - `Title`: Full epic title (e.g., `EPIC-001: Core System Architecture`).
  - `Type`: Literal string `epic`.
  - `Package`: Formal SysML v2 package or namespace (e.g., `SystemModelDefinitions`).
  - `Subsystem`: Subsystem classification name.
  - `Generation Mode`: Mode of generation (e.g., `subagent` or `spec-orchestrator`).
  - `Specification Source`: Document-relative link to the authoritative source model (e.g., `[schema/SystemModel.sysml](../../schema/SystemModel.sysml)`).

#### Epic Metadata Table Example:
```markdown
| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | 1 |
| **Title** | EPIC-001: Core System Architecture |
| **Type** | epic |
| **Package** | SystemModelDefinitions |
| **Subsystem** | Core Subsystems |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/SystemModel.sysml](../../schema/SystemModel.sysml) |
```

---

### 2. Features (`docs/features/`)

Features define modular subsystem components, structural part definitions (`part def`), or data item definitions (`item def`).

- **Mandatory Attributes**:
  - `Issue ID`: Tracker issue identifier or integer ID.
  - `Title`: Full feature title (e.g., `feat-001a-subsystem-structural-mounting`).
  - `Type`: Literal string `feature`.
  - `Parent Epic`: Identifier or relative link to parent Epic (e.g., `EPIC-001` or `[EPIC-001](../epics/EPIC-001.md)`).
  - `Interface Type`: Interface classification (e.g., `Physical / Structural / Umbilical`, `CAN / PWM / Serial`, `Ethernet / IP`).
  - `Schema Containers`: SysML block or container definitions (e.g., `SubsystemAssembly`, `SystemModelDefinitions::Subsystem`).
  - `Generation Mode`: Mode of generation (e.g., `subagent`).
  - `Specification Source`: Document-relative link to the source model (e.g., `[schema/SystemModel.sysml](../../schema/SystemModel.sysml)`).

#### Feature Metadata Table Example:
```markdown
| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | 10 |
| **Title** | feat-001a-subsystem-structural-mounting |
| **Type** | feature |
| **Parent Epic** | [EPIC-001](../epics/EPIC-001.md) |
| **Interface Type** | Physical / Structural / Umbilical |
| **Schema Containers** | `SystemModelDefinitions::SubsystemAssembly` |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/SystemModel.sysml](../../schema/SystemModel.sysml) |
```

---

### 3. User Stories (`docs/user-stories/`)

User Stories define behavioral interactions, operational workflows, control actions (`action def`), or state transitions (`state def`).

- **Mandatory Attributes**:
  - `Issue ID`: Tracker issue identifier or integer ID.
  - `Title`: Full user story title (e.g., `us-001-subsystem-coupling-verification`).
  - `Type`: Literal string `user-story`.
  - `Parent Epic`: Identifier or relative link to parent Epic.
  - `SysML Interaction`: Behavioral action or state definition (e.g., `VerifySubsystemInterface`, `ExecuteControlAllocation`).
  - `SysML Test Case`: Associated formal verification or test case element (e.g., `TestInterfaceCouplingTorque`).
  - `Generation Mode`: Mode of generation (e.g., `subagent`).
  - `Specification Source`: Document-relative link to the source model (e.g., `[schema/SystemModel.sysml](../../schema/SystemModel.sysml)`).

#### User Story Metadata Table Example:
```markdown
| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | 30 |
| **Title** | us-001-subsystem-coupling-verification |
| **Type** | user-story |
| **Parent Epic** | [EPIC-001](../epics/EPIC-001.md) |
| **SysML Interaction** | `SystemModelDefinitions::VerifySubsystemInterface` |
| **SysML Test Case** | `SystemModelDefinitions::TestInterfaceCouplingTorque` |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/SystemModel.sysml](../../schema/SystemModel.sysml) |
```

---

### 4. Use Cases (`docs/use-cases/`)

Use Cases define formal operational capabilities (`use case def`), subject boundaries, and actor interactions.

- **Mandatory Attributes**:
  - `Issue ID`: Tracker issue identifier or integer ID.
  - `Title`: Full use case title (e.g., `uc-001-assemble-and-verify-subsystem`).
  - `Type`: Literal string `use-case`.
  - `Parent Epic`: Identifier or relative link to parent Epic.
  - `Schema Containers`: SysML use case element or container definition (e.g., `AssembleAndVerifySubsystem`).
  - `Generation Mode`: Mode of generation (e.g., `subagent`).
  - `Specification Source`: Document-relative link to the source model (e.g., `[schema/SystemModel.sysml](../../schema/SystemModel.sysml)`).

#### Use Case Metadata Table Example:
```markdown
| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | 50 |
| **Title** | uc-001-assemble-and-verify-subsystem |
| **Type** | use-case |
| **Parent Epic** | [EPIC-001](../epics/EPIC-001.md) |
| **Schema Containers** | `SystemModelDefinitions::AssembleAndVerifySubsystem` |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/SystemModel.sysml](../../schema/SystemModel.sysml) |
```

---

## Tooling & Parsing Guidelines

- **Markdown-First Parsing**: Tooling, validators, and issue trackers that ingest specification metadata MUST parse the initial CommonMark table directly.
- **Regex & AST Extraction**: Automated tools can extract key-value pairs from lines matching `\|\s*\*{0,2}(?P<key>[^|*]+?)\*{0,2}\s*\|\s*(?P<val>[^|]+?)\s*\|`.
- **Bidirectional Traceability**: The `Specification Source` relative path allows automated integrity scanners (e.g., `link_validator.py` and document reference tests) to verify that the source SysML file exists and contains the referenced containers.

## Why

1. **Tracker & Web Portal Rendering**: Web-based project management interfaces (such as GitLab Work Items, GitHub Issues, and standard static site generators) often do not render YAML frontmatter blocks cleanly in body markdown views, resulting in unsightly raw YAML headers (`--- ... ---`).
2. **Standardized Native Presentation**: Two-column CommonMark tables render natively across all markdown parsers, web browsers, and document converters without requiring frontmatter extensions.
3. **Rigorous Traceability**: Enforcing required attributes per specification type guarantees that every Epic, Feature, User Story, and Use Case remains strictly linked to its parent Epic, formal SysML definitions, and relative source models.
