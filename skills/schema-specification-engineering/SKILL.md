<!-- Copyright Gint Atkinson, gint.atkinson@gmail.com -->

---
name: schema-specification-engineering
description: "Transforms structural schemas and normative specification documents into Agile Epics and Features. Use when you need to extract platform-independent feature specifications from structural schemas with exhaustive constraint parsing and Given-When-Then acceptance criteria."
compatibility: "Requires issue tracker CLI and git. Works with modern agentic development environments."
metadata:
  title: "Schema Specification Engineering (Structural Extraction)"
  risk: medium
  source: custom
  version: "2.0"
---

# Schema Specification Engineering

Use this as the single canonical workflow for translating structural schemas, SysML v2 AST models, and normative specification documents into highly rigorous, implementation-ready Agile specifications for sub-agents. 

In accordance with [`rules/sysml-ssot-completeness.md`](rules/sysml-ssot-completeness.md), SysML v2 is the 100% Single Source of Truth (SSOT) for all system architecture and structural components. Epics derive from architectural `package`s and Features derive from formal `part def` (components) and `item def` (data payloads) AST elements. All structural specifications feed downstream Model-Based Design (MBD) and code synthesis in the Primary Tier-1 Commercial Toolchain Context (**MATLAB / Simulink / Stateflow / Embedded Coder** for DO-178C C / SPARK Ada generation).

> [!TIP]
> This skill operates in the spirit of the `andrej-karpathy` methodology: focus deeply on the fundamentals, enforce exhaustive structural rigor, leave absolutely zero ambiguity in the acceptance criteria, and instrument the outputs flawlessly into project tracking systems.

> [!IMPORTANT]
> **EXHAUSTIVE SEMANTIC MODELING MANDATE**
> Do NOT blindly map every isolated schema attribute (e.g., `x`, `y`, `z`) to a separate Feature. You MUST semantically model the schema by grouping cohesive properties into a single logical Feature (e.g., "Cartesian Coordinates"). However, "zero abstraction" still applies: within that grouped Feature, you MUST exhaustively document EVERY underlying attribute/node, capturing its exact data type, mathematical constraints (value ranges, units), defaults, and verbatim specification text. No constraint detail may be lost or summarized away.

## Closed-Loop Payload Verification Gate & Anti-Complacency Rule
- **Exit code 0 is NEVER sufficient proof of success.**
- After modifying or publishing any GitHub issue or document, the agent MUST run `gh issue view <ID>` or `gh api` to fetch the live published payload and inspect links, Mermaid headers, and syntax.
- **Optimism bias is prohibited**: agents must cite empirical output of live payload inspection before declaring completion.

## Step 1: Forensic Audit & Module Decomposition

> [!IMPORTANT]
> **MANDATORY PRE-EXECUTION INGESTION GATE (SysML v2 SSOT)**
> Before initiating Phase 1 decomposition, you MUST execute `sysmlv2_ingest.py` to convert input specification schemas (OMG IDL, AUTOSAR ARXML, Protobuf, OpenAPI) into canonical SysML v2 textual models (`.pipeline/schema.sysml`) and generate `.pipeline/schema-digest.json` per [`rules/sysml-ssot-completeness.md`](rules/sysml-ssot-completeness.md):
> ```bash
> python3 skills/spec-orchestrator/scripts/sysmlv2_ingest.py --schema <schema-file-or-dir> --format auto --out .pipeline/schema.sysml --digest .pipeline/schema-digest.json
> ```
> All Epics and Features MUST be derived directly from the resulting SysML v2 AST nodes (`package`, `part def`, `item def`), and any downstream structural refinements must maintain 100% bidirectional parity with the SysML v2 model.

1. **Parse the Schema & SysML v2 AST:** Read the SysML v2 model (`.pipeline/schema.sysml`), the primary structural schema file, and its imports.
2. **Categorize the Module (Utility vs. Functional)**:
   - Identify if the module contains only type helpers (`typedef`, `identity`, `grouping` definitions without concrete `container` or `list` data nodes).
   - If it is a **utility module** (e.g., `ietf-yang-types`), catalog its types into a Shared Type Registry and parse `grouping` definitions as Reusable Component Features or UML DataTypes linked via composition (`*--`). Do NOT skip specification generation.
   - If it contains concrete data nodes (`config true/false`), classify it as a **functional module** and proceed to decomposition.
3. **Determine Bounded Context (Epic) Boundaries**:
   - Do NOT use a rigid "one schema file = one Epic" rule.
   - **Augment Target Path Resolution Mandate**: Before partitioning, you MUST resolve all `augment` XPaths (e.g., `augment "/nw:networks/nw:network"`) to their target containers and inject the augmented nodes into the target container's tree so they are captured in the target Feature specification.
   - **SysML v2 Subsystem Allocation & Capability Mandate**: Epics MUST derive directly from architectural `package` subsystem boundaries in the SysML AST. Every Epic MUST extract and represent formal `capability def` declarations declared within its subsystem package.
   - If a functional module is small (total leaf count <= 40 and depth <= 3), map it to exactly **1 Epic**.
   - If a functional module is massive (leaf count > 40 or depth > 3), partition the schema graph by major top-level subtrees. Create **1 Epic per partition** representing a logical Bounded Context / Subsystem.
4. **Dispatch Epic Subagents:** For each identified Bounded Context/Subsystem Epic:
   - Invoke a **new, fresh subagent with an isolated context**.
   - Pass only the specific schema nodes/attributes for this subsystem, the SysML AST `package` and `capability def` declarations, and the Epic template.
   - The subagent drafts the Epic markdown file (e.g., `docs/epics/epic-01-name.md`) containing:
     - Formal **Subsystem Capability Allocations** mapping every `capability def` block declared within the architectural `package` to the Epic's subsystem.
     - An overarching **System-Level UML Class Diagram** illustrating the subsystem's classes and their relationships.
     - A **UML Component** representing the subsystem, specifying its provided/required interfaces and operations.
     - A **System State Machine Diagram** representing the macro-level domain, combining the individual structures and lifecycles that will be broken down into child features.

## Step 2: Isolated Feature Extraction (Subagent Dispatch Loop)

For each Bounded Context, partition its subtree into cohesive functional feature groups:
1. **Apply Structural Weight Heuristics for Feature Boundaries**:
   - For any candidate subtree node N, compute its **Structural Weight (SW)**:
     **Pre-Processing Mandate**: Before calculating $SW(N)$, you MUST flatten the schema by recursively inlining all `grouping` definitions wherever `uses` occurs.
     $$SW(N) = L_{immediate}(N) + \sum_{C \in Containers(N)} L_{immediate}(C) + \sum_{U \in Uses(N)} L_{expanded}(U)$$
     where $L_{immediate}(X)$ is the count of leaf and leaf-list nodes directly under node X, excluding any nested list elements, and $L_{expanded}(U)$ is the count of leaf and leaf-list nodes expanded from `uses` statements.
   - **1:1 Container-to-Feature Mapping Mandate:** Every distinct schema `container` MUST be extracted into its own separate Feature file. Do NOT consolidate multiple containers into a single Feature file regardless of structural weight. However, `choice` and `case` branches MUST be kept inside the parent container's Feature file and modeled as polymorphic abstract classes (`<|--`). Attributes within a single container may be grouped within that container's Feature file.
   - **SysML AST Action & Operation Binding Mandate**: Feature specifications MUST extract and bind formal `action def` operations with typed parameter signatures (`in`, `out`, parameter types) from the SysML AST for the corresponding `part def` / `item def`.
   - If $SW(N) > 20$ or has nested lists, partition it:
     - *Sibling lists*: Split each list into its own Feature.
     - *Nested lists*: Split nested lists with >= 5 leaves into child Features.
     - *Complex container*: Split the container by its immediate child containers.
   - **Operational Statements**: Group RPCs, actions, and notifications directly into the Feature containing the target entity they operate on. For top-level `rpc` and `notification` statements without a target entity, extract them into API/M2M Feature files mapped to the module's System Component.
   - **Schema Import Prerequisite Links**: When a schema module `import`s another module that is itself specified in this workspace, the importing Epic MUST carry a `Parent Epic` markdown link to the Epic that specifies the imported module, and every imported module MUST have at least one Epic or Feature specifying it. An import is a hard prerequisite — the importing specification cannot be implemented before the imported one exists — and an unlinked import leaves that ordering constraint recorded nowhere. Enforced by `dependency_validator`.
   - **Container Traceability**: Every Feature MUST declare exactly one schema container in its metadata table `Schema Containers` attribute using the fully-qualified schema container path format: `<module-prefix>:<root-container>/[parent-containers]/[choice/case-wrappers]/<target-node>` (e.g., `ietf-geo-location:geo-location/reference-frame/geodetic-system` or `ietf-geo-location:geo-location/location/ellipsoid`). All intermediate parent containers and choice/case wrapper nodes MUST be preserved in the path. Multi-container Features are forbidden — subagents must split consolidated containers into separate Feature files before the linter gate.
2. **Dispatch Feature Subagent:** For each identified feature group, invoke a **new, fresh subagent with an isolated context** to draft the feature specification. Pass the schema nodes, the SysML AST `part def`/`action def` elements, and properties for this specific feature group, AND the Bounded Context's Epic identity (local file prefix and/or pre-assigned tracker Issue ID if available). The subagent must have no visibility into other features.
3. **Execution within Subagent Context:**
   - **Compliance Table Mandate:** Before writing the file, you MUST output a structured compliance table checking for standard UML primitives, return multiplicities, no curly braces in Mermaid, and no isolated classes.
   - **Platform Independence:** Feature specifications MUST be purely functional and platform-independent. Describe *what* the system must do (data to store, validations to enforce, information to display) — never *how* (no framework-specific components, no platform-specific patterns).
   - **Exhaustive Constraint Parsing & 100% Parameter Type Coverage**: For EVERY attribute and operation within the grouped feature, analyze and record all structural and interface constraints:
     - conditional clauses
     - type definitions (value ranges, string patterns, references)
     - units and default values
     - read-only vs configurable access control
     - **100% Typed Parameter Coverage (Embedded Coder / DO-178C Synthesis)**: All operations originating from SysML `action def` or `operation def` MUST define typed signatures with explicit parameter directions (`in`, `out`) and standard data types (e.g., `Float`, `Integer`, `String`, `Boolean`, `Vector3D`, `StatusEnum`). No untyped or implicit parameter declarations are permitted.
     - **SysML AST Constraint Invariant Binding**: All `constraint def` and `assert constraint` expressions from the SysML AST (e.g., safety envelope limits, thermal thresholds) MUST be exhaustively parsed and bound into feature acceptance criteria.
   - **UML Class Diagram:** Every Feature specification MUST include a **UML Class Diagram** (using Mermaid `classDiagram`).
     - *UML Classifier Mapping*: Feature specifications must map to a primary UML Class or DataType representing the schema entity. To satisfy strict linter verification, the diagram MUST include class nodes for ALL ancestor containers along the fully-qualified schema container path (from the root container down to the target node) and illustrate composition or aggregation relationships (e.g. composition `*--` or aggregation `o--`) between every adjacent pair of classes in the hierarchy, ensuring no isolated classes exist. Classes that will cross serialization boundaries (Web Workers via `structuredClone`, Flutter Isolates via `SendPort`) MUST be modeled as pure data classes (DTOs) without methods. Service methods (e.g. `save()`, `validate()`) MUST be placed in separate service/repository classes that are NOT transferred across threads.
     - *Choice/Case Representation*: Model schema alternative structures as abstract classes or classes with the `<<choice>>` stereotype, and their constituent choices as classes inheriting (`<|--`) from the choice class.
     - *Feature Guard vs. Data Leaf Disambiguation*: YANG `feature` declarations and `if-feature` substatements (e.g., `if-feature "alternate-systems"`) are conditional compilation guards (`<<feature_guard>>`), NOT data leaves or class attributes. They MUST be extracted as stereotyped constraint notes or class notes, and MUST NOT be rendered as data leaf attributes on UML class diagrams.
     - *UML Standard Primitive Types*: All attributes in class diagrams must use standard capitalized UML primitives (`String`, `Integer`, `Real`, `Boolean`) instead of format-specific or custom types.
     - *Visibility & Multiplicity*: Every attribute/operation must use visibility indicators (`+`/`-`) and standard multiplicities (e.g. `[1]`, `[0..1]`, `[0..*]`).
     - *UML Constraints*: Schema-level constraints must map to standard text notes or separate tables. Curly braces '{}' inside class member lines are strictly prohibited due to Mermaid parse conflicts (they crash GitHub and Mermaid CLI renderers). Use parentheses '(default: earth)' or simple brackets '[default: earth]' if constraints must be inline.
     - *Multiplicity Bracket Rendering*: Note that unquoted brackets `[0..1]` inside Mermaid class bodies may cause rendering failures in some engines (GitHub, Mermaid CLI). Represent multiplicity on relationship lines instead.
     - *Double-Declaration Redundancy*: Do NOT list object-typed attributes inside the class body if they are already represented as named relationship lines.
   - **Interface Requirements:** Every feature spec MUST explicitly include a `## Interface Requirements` section divided into dynamic structured sub-sections based on the `Interface Type` (defined in the metadata table as `ui`, `api`, `m2m`, or interface classification):
     - *For UI Interfaces (`Interface Type: ui` or GUI classification)*:
       - `1. Test Data Shape (JSON Payload Example)`: A concrete, copy-pasteable JSON payload schema example block.
       - `2. Validation & Constraints`: Exhaustive list of ranges, regex patterns, mandatory fields, and conditions.
       - `3. Visual Layout & Arrangement`: Detailed, platform-independent description of the visual layout and hierarchy. Mandate CSS resets (box-sizing), scoped naming (CSS Modules/BEM) to avoid specificity conflicts, layout containment parameters (restricting containment to outer layout splitters and forbidding it on scrollable child panels), and valid DOM nesting for tree structures (recursive lists nested inside parent list-items).
       - `4. Interactive Flow & States`: System states (read-only, edit, empty, loading, error highlighting). Mandate computed-style assertions (such as verifying scroll dimensions or highlight colors) in the test guidelines for components with visual, selection, or highlight states.
     - *For API or M2M Interfaces (`Interface Type: api` or `m2m` or hardware classification)*:
       - `1. Payload Schema (JSON Schema/Protobuf)`: Target request/response payload definition.
       - `2. Validation & Constraints`: Schema field constraints, type validations, and logical conditions.
       - `3. Logical Operations & Interface Messages`: Abstract definitions of logical endpoints, methods (GET/POST/Publish/Subscribe or read/write operations), logical paths, or routing channels.
       - `4. Logical Exception States & Validation Failures`: Expected logical error states, exception/failure flows, and timeouts.
   - **Acceptance Criteria Translation:** Transform these programmatic constraints and interface requirements into exhaustive Given-When-Then Logical Acceptance Criteria. Criteria MUST be platform-independent.
   - **Specification Context Injection (Verbatim):** Embed the exact paragraphs and sections from the canonical normative text explaining the behavioral logic of this specific structural container under a `## Specification Context (Verbatim)` section.
   - **Formatting of Alphanumeric Identifiers & Math Expressions**: Ensure all requirement references, hazard tags, and SORA SAIL codes use standard bold text (`**SC-01**`, `**H-1**`, `**OSO-11**`) rather than inline LaTeX math mode (`$SC-01$`). Non-mathematical alphanumeric tokens must NEVER be wrapped in `$...$` math delimiters per `rules/latex-katex-integrity.md`.
   - **Pure Symbolic Mathematical Separation Rule**: When extracting or specifying mathematical equations, physical dynamics, kinematics, or control laws:
      - Display math blocks (`$$ \begin{aligned} ... \end{aligned} $$`) MUST express **pure symbolic equations only**.
      - Strictly prohibit embedding physical unit macros (`\text{ ms}`, `\text{ kg}`, `\text{ m/s}`, `\text{ bar}`, `\text{ V}`, etc.) inside display math equations.
      - Mandate that all physical values, numerical limits, constants, calibration thresholds, and engineering units are defined in the accompanying "Where and Operational Parameters:" text section immediately following the equation block.
      - Strictly prohibit dangling operators (such as `/` with no denominator) and unescaped underscores inside `\text{}` (mandating `\text{yaw-disturbance}` or structured subscripts like `\Delta v_{\text{yaw,dist}}`).
      - Multi-line aligned equations MUST use `\begin{aligned} ... \end{aligned}` inside `$$` delimiters on dedicated lines.
    - **Draft the Feature Spec File:** Write the Feature as a local markdown file (e.g., `docs/features/feat-01-name.md`).
3. **Return Control:** The subagent completes the task and returns control to the worker agent.

## Step 3: Specification Context Injection (Verbatim)

1. **Locate Normative Text:** Find the canonical normative text document (e.g. specification standard documents) associated with the schema.
2. **Extract Line-by-Line Context:** Identify the exact paragraphs and sections that explain the behavioral logic of the specific structural container.
3. **Embed Context:** Inject this verbatim text directly into the feature specification under a `## Specification Context (Verbatim)` section. This guarantees that implementing sub-agents have ground-truth knowledge and are not hallucinating implementation details.

## Step 4: Output Formatting & Strict GitHub Instrumentation

> [!WARNING]
> You must strictly follow the operational sequencing below to ensure the `#IssueID` linkages are perfectly resolved.
> **Unified Slugification Mandate:** When generating filenames from titles (e.g., `feat-01-fiber-cable-and-strand-inventory.md`), you MUST preserve all stop-words (like 'and', 'the', 'of', etc.) consistently. Do NOT strip stop-words when converting titles to lowercase hyphen-separated slugs.


1. **Markdown Metadata Table:** Prepend a native CommonMark two-column metadata table to every `.md` file per `rules/specification-metadata-integrity.md`:
   ```markdown
   | Attribute | Specification Detail |
   | :--- | :--- |
   | **Issue ID** | #[IssueID] |
   | **Title** | [Feature Title] |
   | **Type** | feature |
   | **Parent Epic** | #[EpicIssueID] - [Epic Title](../epics/epic-XX-name.md) |
   | **Interface Type** | [Interface Type] |
   | **Schema Containers** | `[SchemaContainerPath]` |
   | **Generation Mode** | subagent |
   | **Specification Source** | [schema/...](../../schema/...) |
   ```
   > **Note:** No `platform` field. Features are functional specs. Platform targeting occurs at implementation time via `feature-driven-implementation` and the project's implementation profiles.
   > **Container Traceability:** Every Feature MUST declare its schema container in `Schema Containers` with the fully-qualified container path in the format `<module-prefix>:<root-container>/[parent-containers]/[choice/case-wrappers]/<target-node>` (e.g., `ietf-geo-location:geo-location/reference-frame/geodetic-system`). All intermediate parent containers and choice/case wrapper nodes MUST be preserved. Multi-container Features are forbidden — the linter gate will reject files without a valid container.

2. **Epic File Structure / Template:** Every Epic specification markdown file MUST follow this exact section structure and ordering:
    ````markdown
    | Attribute | Specification Detail |
    | :--- | :--- |
    | **Issue ID** | #[IssueID] |
    | **Title** | [Epic Title] |
    | **Type** | epic |
    | **Package** | [PackageName] |
    | **Subsystem** | [SubsystemName] |
    | **Generation Mode** | subagent |
    | **Specification Source** | [schema/...](../../schema/...) |

    # Epic: [Epic Title]

    ## 1. Context
    [High-level functional description and specification-engineering context of the schema module]

    ## 2. Requirements & Checklist
    - [ ] #[IssueID] - [Feature Title](../features/feat-XX-name.md) (semantic linkage justification)

    ### Associated Use Cases & User Stories

    #### Associated Use Cases
    - [ ] #[IssueID] - [Use Case Title](../use-cases/uc-XX-name.md) (semantic linkage justification)

    #### Associated User Stories
    - [ ] #[IssueID] - [User Story Title](../user-stories/us-XX-name.md) (semantic linkage justification)

    > [!IMPORTANT]
    > **EXPLICIT LINKAGE JUSTIFICATION MANDATE**
    > Subagents MUST provide concise, context-specific semantic justifications for all checklist items. Omitting justifications or leaving unreplaced placeholder text in generated Epic specifications is strictly prohibited and will trigger validator rejection.


    ## 3. Architecture

    ### Subsystem Capability Allocations
    Formal allocation of SysML v2 `capability def` declarations mapped to this subsystem package:
    | Capability Name | Subsystem Package | Description / Objective |
    | --- | --- | --- |
    | TelemetryProcessing | SystemController | Executes real-time telemetry processing and state machine transitions |

    ### Subsystem Component Definition
    Define the subsystem representing the Epic as a UML Component specifying provided/required interfaces and operations.
    ```mermaid
    classDiagram
        class SubsystemComponent {
            <<component>>
            +Boolean providedInterface()
            +String requiredInterface()
        }
    ```

    ## System-Level UML Class Diagram
    ```mermaid
    classDiagram
        class SubsystemComponent {
            <<component>>
        }
        class FeatureClassifier1 {
            +String attributeOne "[1]"
            -Boolean attributeTwo "[0..1]"
        }
        class FeatureClassifier2 {
            +Integer attributeThree "[0..*]"
            +Boolean operationOne(String input)
        }
        SubsystemComponent *-- FeatureClassifier1
        SubsystemComponent *-- FeatureClassifier2
    ```

    ## State Machine Definitions

    ## System State Machine Diagram
    ```mermaid
    stateDiagram-v2
        [*] --> InitialState
        InitialState --> [*] : "operationOne(input) / Action"
    ```

    ## 4. Operational Considerations
    [Operational considerations and deployment scenarios]

    ## 5. Security & Governance
    [Security, access control, and governance considerations]

    ## Specification Context
    [Verbatim schema grouping/container descriptions from the normative specification]

    ## 6. Source References
    Structural Schema: [Target Schema File](link-to-schema)
    Normative Specification: [Normative Specification](link-to-specification)
    ````

3. **Feature File Structure / Template:** Every feature specification markdown file MUST follow this exact section structure and ordering:
   ````markdown
   | Attribute | Specification Detail |
   | :--- | :--- |
   | **Issue ID** | #[IssueID] |
   | **Title** | [Feature Title] |
   | **Type** | feature |
   | **Parent Epic** | #[EpicIssueID] - [Epic Title](../epics/epic-XX-name.md) |
   | **Interface Type** | [Interface Type] |
   | **Schema Containers** | `[SchemaContainerPath]` |
   | **Generation Mode** | subagent |
   | **Specification Source** | [schema/...](../../schema/...) |

   # Feature: [Feature Title]

   ## Parent Epic
   - [ ] #[EpicIssueID] - [Epic Title](../epics/epic-XX-name.md) (semantic linkage justification)

   ## Description
   [Functional description of the feature]

   ### Mathematical Formulations of [Domain Dynamics / Control Laws] (When Applicable)
   $$
   \begin{aligned}
   \dot{x} &= f(x, u, t) \\
   y &= g(x, u, t)
   \end{aligned}
   $$

   Where and Operational Parameters:
   - $x$: State vector representing system state variables.
   - $u$: Control input vector.
   - $t$: Time variable ($t \ge 0.0\text{ s}$).
   - $y$: System output measurement.

   ## UML Class Diagram
     ```mermaid
     classDiagram
         class ParentContainer {
         }
         class FeatureClassifier {
             +String primaryAttribute "[1]"
             -Boolean optionalAttribute "[0..1]" (constraintText)
             +Integer listAttribute "[0..*]"
             +Boolean doSomething(String param)
             +Boolean ExecuteManeuver(Float in_targetHeading, StatusEnum out_maneuverStatus)
         }
         ParentContainer *-- FeatureClassifier : featureClassifier
     ```

     ## Interface Requirements

     <!-- For UI Interfaces (interface_type: ui) -->
     ### 1. Test Data Shape
     ```json
     {
       "primaryAttribute": "example_value",
       "optionalAttribute": true,
       "listAttribute": [1, 2, 3]
     }
     ```

     ### 2. Validation & Constraints
     - [Field constraints, ranges, patterns, protocol/payload limits]

     ### 3. Visual Layout & Arrangement
     - [For UI: abstract grouping, zoning, hierarchy guidelines. Enforce CSS resets (box-sizing), scoped naming (CSS Modules/BEM) to avoid specificity conflicts, layout containment parameters (restricting containment to outer layout splitters and forbidding it on scrollable child panels), and valid DOM nesting for tree structures (recursive lists nested inside parent list-items).]

     ### 4. Interactive Flow & States
     - [For UI: states, errors, loading. Mandate computed-style assertions (such as verifying scroll dimensions or highlight colors) in the test guidelines for visual or active selection states.]

     <!-- OR For API or M2M Interfaces (interface_type: api or m2m) -->
     ### 1. Payload Schema
     ```json
     {
       "primaryAttribute": "example_value",
       "optionalAttribute": true,
       "listAttribute": [1, 2, 3]
     }
     ```

     ### 2. Validation & Constraints
     - [Field constraints, ranges, patterns, protocol/payload limits]

     ### 3. Logical Operations & Interface Messages
     - [For API/M2M: logical methods, operations, abstract paths, or channels with 100% typed parameter signatures matching SysML AST action defs for Embedded Coder/DO-178C code synthesis]
     - `+ExecuteManeuver(in targetHeading: Float, out maneuverStatus: StatusEnum)`: Executes avoidance trajectory maneuvers.

     ### 4. Logical Exception States & Validation Failures
     - [For API/M2M: logical error states, timeouts, exception flows]

   ## Given-When-Then Acceptance Criteria
   [BDD scenarios]

   ## Specification Context (Verbatim)
   [Raw normative specification context paragraphs]

   ## Source References
   Structural Schema: [Target Schema File](link-to-schema)
   Normative Specification: [Normative Specification](link-to-specification)

   ## Logical UI & Interface Bindings
   <!-- Single-Channel (Visual GUI) Format -->
   - **Target LUI Component:** [Specify canonical LUI component e.g. StringInputField, TableView, PropertyGrid, OR 'Unbound (Deferred to Implementation Profile)']
   - **Target Layout Container ID:** [Specify container ID from logical-layout.json, OR 'Unbound (Deferred to Implementation Profile)']
   - **Data Source Binding:** [Specify exact schema path e.g. /schema:path, OR 'Unbound (Deferred to Implementation Profile)']

   <!-- Multi-Channel (Multi-Interface) Format -->
   | Interface Channel | Category | Target Component / Handler | Target Container / Endpoint | Data Source Binding |
   | --- | --- | --- | --- | --- |
   | gui | Visual GUI | StringInputField | elements_view | /schema:path |
   | mcp | M2M API | MCPToolHandler | /mcp/tool | /schema:path |
   ````

   > [!WARNING]
   > **Mermaid Block Closing Constraints & Code Fence Integrity:**
    > - **Mandatory Mermaid Diagram Header Rule**: The very first non-comment line inside EVERY Mermaid code fence (```` ```mermaid ````) MUST declare a valid diagram type header (e.g. classDiagram, graph TD, flowchart TD, sequenceDiagram, stateDiagram-v2). Omitting the header and beginning directly with relationships or member lines is strictly forbidden.
    > - Every Mermaid diagram MUST be strictly closed with ```` ``` ```` on a new line. Leaking Mermaid blocks (e.g. having headings like `##` inside an unclosed diagram) or stray/unclosed code fences will fail downstream validation checks.
    > - Ensure there are no stray backticks or unmatched code fences in the document.
    > - **All Mermaid syntax constraints are defined in `rules/platform-independence.md` and MUST be observed in full** — including the prohibition on curly braces in class member lines, colons in class members and note strings, stereotypes on relationship lines, and semicolons in `Note` and message text. Do not maintain a local subset here; subsets drift (issue #289).
    > - **Universal Angle Bracket Escaping**: Unquoted `<` and `>` characters are strictly forbidden across ALL diagram types (graph TD, flowchart TD, sequenceDiagram, stateDiagram-v2). Transitions, labels, or guards containing comparison operators, brackets, or guards MUST enclose the label in double quotes.
    > - **Use Case Node Label Quoting**: Mandate double quotes around graph TD/flowchart TD node labels containing slashes, colons, parentheses, or brackets (e.g. `Node["Save/Restore (Local DB)"]`).
    > - **Subgraph Title Quoting**: Mandate double quotes around subgraph titles with spaces or hyphens (e.g. `subgraph "System Boundary"`).

4. **Source References Block (CRITICAL):**
   - **Dynamic Schema Locator**: You MUST inspect the active workspace directories (e.g. `schema/`) to build schema locators dynamically. Do NOT hardcode legacy paths like `standard/ietf/RFC/`.
   - At the bottom of every feature markdown file, you MUST append a `## Source References` section containing dynamic references to the input structural schemas and specifications, formatted like this:
   ````markdown
   ## Source References
   Structural Schema: [Target Schema File](link-to-schema) (Clause: [Clause Number])
   Normative Specification: [Normative Specification](link-to-specification) (Clause: [Clause Number])
   ````
   - Inject the exact absolute URLs pointing to the authoritative structural schema and normative text document provided by the user. Do not omit this.

5. **Logical UI & Interface Bindings Block (MANDATORY):**
   - Every feature specification markdown file MUST contain a `## Logical UI & Interface Bindings` section at the end of the file (unless exempt as a non-UI feature).
   - Features may format interface bindings as a single-channel 3-bullet locator list or as a multi-channel Multi-Interface Binding Table (`| Interface Channel | Category | Target Component / Handler | Target Container / Endpoint | Data Source Binding |`).
   - You MUST map the feature's container and leaf nodes to:
     - The target LUI component or M2M/Hardware handler (e.g. `StringInputField`, `TableView`, `MCPToolHandler`, `RegisterBuffer`), or `Unbound (Deferred to Implementation Profile)`.
     - The specific target layout container ID in `logical-layout.json` or API endpoint path, or `Unbound (Deferred to Implementation Profile)`.
     - The data source bindings. **CRITICAL PATH DERIVATION RULE**: You are strictly forbidden from copy-pasting generic template or placeholder namespaces (such as `schema:generic-topology`). You MUST derive the data source path directly from the exact, authoritative schema path locator of the target schema container augmented in the network inventory model (e.g. `/nwi:network-inventory/nil:locations/nil:location/nil:geo-location/nil:reference-frame` or `/nwi:network-inventory/nil:locations/nil:racks/nil:rack`), or state `Unbound (Deferred to Implementation Profile)`. Literal placeholder strings (`#X`, `Task Y`) are strictly prohibited.
   - **Logical UI & Interface Binding Validation Rules**: the bindings block is checked mechanically
     by `parity_auditor/validators/logical_ui_validator.py`.
     - **Interface Bindings Section Required**: every Feature MUST carry the
       `## Logical UI & Interface Bindings` section. A Feature is exempt only if its
       metadata table declares non-UI interface types (`config`, `persistence`, `gate`, `cli`, `backend`).
     - **Feature Metadata Table Must Parse**: the metadata table MUST be well formed. It
       carries `Interface Type` (e.g. `Physical / Structural / Umbilical`, `gui`, `mcp`, `m2m`, `api`).
     - **Interface Channel Row Required**: every channel listed in the Interface Type MUST have a corresponding row in the Multi-Interface Binding Table.
     - **Raw N/A Fallback Strings Strictly Prohibited**: raw `N/A` fallback strings and literal placeholder strings (`#X`, `Task Y`) are strictly prohibited across all single-channel lists and multi-channel binding tables. Explicit binding or setting to `Unbound (Deferred to Implementation Profile)` MUST be used instead.
     - **Target Component Must Exist In The Layout or State Unbound**: raw `N/A` strings and literal placeholder strings (`#X`, `Task Y`) are strictly prohibited. The `Target LUI Component` MUST name a canonical component type actually instantiated in `logical-layout.json` or canonical LUMI dictionary (e.g. `StringInputField`, `TableView`, `MCPToolHandler`, `RegisterBuffer`), or set to `Unbound (Deferred to Implementation Profile)`.
     - **Target Container Must Exist In The Layout or State Unbound**: raw `N/A` strings and literal placeholder strings (`#X`, `Task Y`) are strictly prohibited. The `Target Layout Container ID` MUST name a container `id` present in `logical-layout.json` or target endpoint, or set to `Unbound (Deferred to Implementation Profile)`.
     - **Component Must Match Its Container Type**: where both are given, the component
       type MUST equal the declared type of the target container.
     - **Data Source Bindings Must Be Schema Paths or State Unbound**: raw `N/A` strings and literal placeholder strings (`#X`, `Task Y`) are strictly prohibited. Each entry MUST be an exact, authoritative schema path locator beginning with `/`, `schema:` or `provider:` and MUST NOT contain spaces or placeholder strings, OR set to `Unbound (Deferred to Implementation Profile)`.
     - **Data Source Bindings Must Omit Choice And Case Nodes**: a YANG `choice` or `case`
       node is a schema-modelling construct and does not appear in the data tree, so it
       MUST NOT appear in a data path.
     - **Augmented Nodes Must Carry Their Module Prefix**: once a path enters an augmented
       subtree, every following segment MUST carry the augmenting module's prefix (e.g.
       `nil:location`, not `location`).
     - **Spatial Features Must Bind A Spatial Component**: a Feature whose text carries
       geodetic or spatial attributes MUST bind to `TopologyMap`, `TopographicalView`,
       `GeoSpatialViewer`, `PropertyGrid` or `TableView`.
   - **Geolocation & Geodetic Semantic Mapping Rules**:
     - Geolocation and geodetic attributes (such as reference-frame, geodetic-system, coordinates, velocity, geo-location, geodetic, latitude, longitude, altitude, elevation, datum, position, and spatial) represent child properties of concrete parent components.
     - You MUST map these geodetic attributes to details panels and tables (such as `PropertyGrid` with container ID `properties_view`, or `TableView` with container ID `components_table`) that are actively instantiated in the layout.
     - You are strictly forbidden from mapping these inherited attributes as standalone visual topology viewports (`TopographicalView`, `TopologyMap`, `GeoSpatialViewer`), tree selectors (`HierarchyTreeSelector`, `HierarchyTree`), or topology container IDs (`topology_pane`, `resource_tree`, `navigation_tree`, `map_viewport`).

## Step 5: Local Validation & Backlog Synchronization

1. **Mandatory Local Validation Gate:** Before committing, pushing, or creating issues in the backlog, the subagent MUST execute the local validation check:
   ```bash
   ./skills/spec-orchestrator/scripts/verify_model_coverage.py --spec-only --allow-missing-specs
   ```
   If the linter fails (returns a non-zero exit code), the subagent MUST parse the errors, fix all generated Feature and Epic markdown files, and re-run the linter until it passes with exit code 0.
   Before committing the generated markdown files, the agent MUST run a check for untracked pipeline infrastructure files. If untracked files are found in `.pipeline/`, `skills/`, `rules/`, or `scripts/`, they must be staged and committed alongside the markdown files using `git add` to prevent remote divergence:
   ```bash
   UNTRACKED_INFRA=$(git ls-files --others --exclude-standard .pipeline/ skills/ rules/ scripts/)
   if [ -n "$UNTRACKED_INFRA" ]; then
     git add .pipeline/ skills/ rules/ scripts/
   fi
   ```

2. **Tracker Label Bootstrapping:** Invoke the issue tracker's label bootstrap interface (e.g. creating "epic" and "feature" labels in the configured provider).

3. **Duplicate Detection (Idempotency Check):**
   - Before creating any issue, query the active tracker provider for all existing backlog issues to check if an issue with an identical or semantically equivalent title already exists.
   - If a duplicate is found: skip creation, and reuse the existing Issue ID for Epic linkage.
   - This ensures the pipeline is safe to re-run without creating duplicate issues.

4. **Feature Backlog Creation FIRST:**
   - Register each Feature specification with the active tracker provider, capturing the returned Issue ID/URL from the tracker.
   - **Crucial Verification & Body Synchronization:**
     1. Backlog issues MUST be registered using `TITLE=$(awk -F'|' '/**Title**/ {print $3}' <local-md-file> | xargs); gh issue create --title "$TITLE" --body-file <local-md-file>` (to ensure they start with the full markdown content, including diagrams and references).
     2. Immediately after placeholder resolution (when the live issue ID is injected back into the file), the subagent MUST execute `gh issue edit <ID> --body-file <local-md-file>` to sync the resolved ID body.
     3. The subagent MUST run a post-creation verification check:
         `gh issue view <ID> --json body | python3 -c "import sys,json; b=json.load(sys.stdin)['body']; markers=['Source References','UML Class Diagram','Acceptance Criteria']; missing=[m for m in markers if m not in b]; assert not missing, f'Body incomplete: missing {missing}'"`
         and retry/halt if this verification fails.
     4. Before committing the generated feature markdown files, the agent MUST run a check for untracked pipeline infrastructure files. If untracked files are found in `.pipeline/`, `skills/`, `rules/`, or `scripts/`, they must be staged and committed alongside the markdown files using `git add` to prevent remote divergence:
        ```bash
        UNTRACKED_INFRA=$(git ls-files --others --exclude-standard .pipeline/ skills/ rules/ scripts/)
        if [ -n "$UNTRACKED_INFRA" ]; then
          git add .pipeline/ skills/ rules/ scripts/
        fi
        ```

5. **Epic Backlog Assembly:**
   - Now that you possess the actual live Issue IDs for all extracted features, inject them into the Epic's checklist.
   - Ensure the body of the Epic lists its child features as a tasklist referencing the Issue ID and canonical relative markdown links to the feature document (e.g., `../features/feat-01.md`). Link expansion to web blob URLs for tracker UI navigation is handled automatically by the backlog reconciler during tracker payload synchronization.

6. **Epic Backlog Creation LAST:**
   - Register the Epic specification containing the fully resolved tasklist with the active tracker provider.
   - **Crucial Verification & Body Synchronization:**
     1. Register the Epic issue using `TITLE=$(awk -F'|' '/**Title**/ {print $3}' <local-md-file> | xargs); gh issue create --title "$TITLE" --body-file <local-md-file>`.
     2. Immediately after placeholder resolution, the subagent MUST execute `gh issue edit <ID> --body-file <local-md-file>` to sync the resolved ID body.
     3. The subagent MUST run a post-creation verification check:
         `gh issue view <ID> --json body | python3 -c "import sys,json; b=json.load(sys.stdin)['body']; markers=['Source References','System-Level UML Class Diagram','Context']; missing=[m for m in markers if m not in b]; assert not missing, f'Body incomplete: missing {missing}'"`
          and retry/halt if this verification fails.
      4. Before committing the generated epic markdown files, the agent MUST run a check for untracked pipeline infrastructure files. If untracked files are found in `.pipeline/`, `skills/`, `rules/`, or `scripts/`, they must be staged and committed alongside the markdown files using `git add` to prevent remote divergence:
        ```bash
        UNTRACKED_INFRA=$(git ls-files --others --exclude-standard .pipeline/ skills/ rules/ scripts/)
        if [ -n "$UNTRACKED_INFRA" ]; then
          git add .pipeline/ skills/ rules/ scripts/
        fi
        ```

7. **Backfill Parent Epic into Features:** After Epic issue creation, capture the returned Issue ID. For each Feature file generated under this Epic:
   a. Verify the `## Parent Epic` checklist item references the correct Epic Issue ID.
   b. If mismatched, replace with the correct ID and re-commit the file.
   c. Execute `gh issue edit <FeatureID> --body-file <local-feature-md>` to sync the tracker body.
