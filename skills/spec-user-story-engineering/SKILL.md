<!-- Copyright Gint Atkinson, gint.atkinson@gmail.com -->

---
name: spec-user-story-engineering
description: "Extracts BDD User Stories derived from SysML v2 interaction def, action def, state def, and port def AST nodes and normative specification documents using OOA/OOD modeling. User Stories parse SysML v2 interaction AST blocks to generate sequence diagram lifelines, message flows, and Stateflow transition triggers, while Acceptance Criteria BDD Scenarios generate and link to formal SysML test case def declarations with verify requirement bindings."
compatibility: "Requires issue tracker CLI and git. Works with modern agentic development environments."
metadata:
  title: "Specification User Story Engineering (Behavioral Extraction)"
  category: architecture
  risk: low
  source: custom
  version: "2.1"
---

# Specification User Story Engineering (Behavioral Extraction)

This skill enables a sub-agent to autonomously derive pure Behavior-Driven Development (BDD) User Stories modeled according to Object-Oriented Analysis and Design (OOA/OOD) principles directly from SysML v2 behavioral elements (`interaction def`, `action def`, `state def`, `port def`, `test case def`) and normative specification documents. 

In accordance with [`rules/sysml-ssot-completeness.md`](rules/sysml-ssot-completeness.md), SysML v2 is the 100% Single Source of Truth (SSOT) for all computational actions, lifecycle states, interface ports, interaction message flows, and verification test cases. User Stories provide behavioral realization feeding downstream Stateflow statecharts and control law synthesis in the Primary Tier-1 Commercial Toolchain Context (**MATLAB / Simulink / Stateflow / Embedded Coder** for DO-178C C / SPARK Ada generation).

## Execution Trigger
You should invoke this skill ONLY after the structural Features have been extracted using the `schema-specification-engineering` skill.

### SysML Interaction AST Parsing Trigger (Mandatory - Issue #40 / Check 20)
You MUST parse SysML v2 `interaction` AST blocks (`interaction def` / `interaction <Name> { lifeline ...; message ...; trigger ...; }`) to derive the sequence diagram lifelines, message flows, and Stateflow transition triggers. All internal participant lifelines in the User Story MUST bind to valid SysML `part def`s, and message flows must bind to formal SysML interaction messages or part action/operation definitions.

### Algorithmic & Calculation Story Extraction Trigger (Mandatory)
In addition to standard deployment scenarios, you MUST scan the SysML v2 model (`action def`) and schemas for any derived, computed, or calculated values (e.g. performing unit conversions, coordinate transformations, validation ranges, formulas, or elapsed time checks). For every calculated or derived value identified, you MUST extract a dedicated, mandatory User Story that details the calculations, formulas, or algorithmic transformations required, ensuring that these dynamic behaviors are fully captured.

### Temporal & Lifecycle Expiration Story Extraction Trigger (Mandatory)
In addition to standard deployment scenarios, you MUST scan the SysML v2 model (`state def`) and schemas for any temporal/lifecycle expirations, state-decay lifecycles, or timeout transitions (e.g. token expiration, data staleness, status-based data access rules, or lifecycle decay). For every temporal or lifecycle expiration identified, you MUST extract a dedicated, mandatory User Story detailing the transition to the expired state and any postconditions for accessing data in that state.

### Acceptance Criteria Test Case Binding Trigger (Mandatory - Issue #42 / Check 22)
For every BDD scenario and acceptance criteria set, you MUST generate and link to a formal SysML `test case def` declaration in the AST containing explicit `subject <Part>`, `verify requirement <RequirementID>`, `objective "<Objective>"`, and ordered `step <action>` definitions, ensuring formal traceability back to parent safety requirements.

## Step 1: Context Ingestion (SysML v2 AST, Schemas & Operational Text)
1. Ingest the canonical SysML v2 model (`.pipeline/schema.sysml`), `.pipeline/schema-digest.json`, target normative specification document, AND structural schemas.
2. **Scan the SysML v2 AST definitions and structural schema nodes** (specifically `interaction def`, `action def`, `state def`, `port def`, `test case def`, `constraint def`, `assert constraint`, node descriptions, comments, type restrictions, and validation constraints) to identify:
   - Any interaction sequences (`interaction def`), lifelines, message flows, and triggers.
   - Any derived, calculated, or computed data fields.
   - Any mathematical formulas, equations, unit conversions, or derivations.
   - Any temporal attributes, state lifecycles, or transition guards.
   - Any verification test cases (`test case def`) and verified safety requirements (`verify requirement`).
3. Target and analyze the following operational chapters of the normative specification:
   - Introduction & Applicability
   - Deployment Scenarios
   - Operational Considerations
   - Security Considerations
   - Algorithmic, Calculation, or Derivation clauses

## Step 2: Isolated User Story Modeling (Subagent Dispatch Loop)

1. **Identify Scenarios & Triggers:** Analyze the specification chapters and structural schemas to determine all required deployment scenarios, calculations/derivations, interaction flows, and temporal/state lifecycles. Compile the list of target User Stories to be engineered.
2. **Dispatch User Story Subagent:** For each identified User Story, invoke a **new, fresh subagent with an isolated context**. Pass ONLY the specific operational text, relevant schema definitions, related Feature specs, SysML interaction definitions, and the User Story template. The subagent must have no visibility or knowledge of other User Stories.
3. **Execution within Subagent Context:**
   - **Compliance Table Mandate:** Before writing the file, you MUST output a structured compliance table checking for lifeline aliasing (e.g. 'actorName : Classifier'), open return arrows ('-->'), return value assignment signatures (no method call format), Given-When-Then BDD scenarios, SysML interaction binding, and SysML test case definition bindings.
   - **Behavioral Modeling:** Model the scenario as a formal User Story integrated with OOA/OOD principles:
     - Identify the Actor/Role (the object or entity initiating the action).
     - Formulate the core scenario using strict BDD syntax mapped to object interactions (`Given`/`When`/`Then` or `As a`/`I want to`/`So that`).
     - Map the story to specific Domain Objects (the structural schema entities affected).
     - **UML Sequence Diagram & SysML Interaction Binding (Check 20)**: Include a **UML Sequence Diagram** (using Mermaid `sequenceDiagram`) illustrating the dynamic interaction between the Actor and specific Domain Objects.
       - *SysML Interaction Realization*: Parse SysML `interaction` AST blocks to construct lifelines, message sequences, and triggers.
       - *Lifeline Notation*: All sequence diagrams must use the standard UML lifeline notation `name : Classifier` or `: Classifier` (using Mermaid alias syntax: `actor userActor as "userActor : UserActor"` or `participant systemService as "systemService : SystemService"`).
       - *Lifeline Part Binding*: Every internal `participant` classifier MUST resolve to a valid SysML `part def` declared in the SysML AST.
       - *Actor vs Participant (enforced — issue #277)*: The choice of keyword is semantic, not cosmetic, and determines whether the classifier must exist in a Feature class diagram / SysML part definition.
         - Declare a lifeline `actor` **only** when it represents an entity **outside the system boundary** — a human role, or a third-party system you do not model. An `actor` classifier is **exempt** from the structural-definition requirement, because external entities are correctly absent from the structural models.
         - Declare a lifeline `participant` for every **internal** object. A `participant` classifier **MUST** be defined as a class in some Feature's UML Class Diagram and as a SysML `part def`, and every message sent to it must map to a public operation on that class/part.
       - *Open Return Arrow*: Return/reply messages must use the open arrowhead (`-->` in Mermaid) instead of the filled/closed arrowhead (`-->>`).
       - *Return Value Signatures*: Return messages must represent assignments/return values (e.g. `isValid : Boolean`) rather than method/operation calls.
       - *Operation Matching*: Every call/message in a sequence diagram must map to a public operation/method (with camelCase signature and typed arguments) on the receiver lifeline's classifier in the class diagrams and SysML part actions/operations.
       - *Combined Fragment Guards*: Guards on conditional/looping blocks (e.g. `alt`, `loop`, `opt`) must be enclosed in standard UML square brackets `[guard]`.
       - *Validation Loops/Conditional Blocks*: Use Mermaid `alt` or `loop` blocks to explicitly illustrate input validation loops.
       - *Helper/Calculator Object Delegation*: Do not model the main container handling complex computations directly; delegate to specialized helper or utility objects.
     - **UML State Machine Diagram**: Include state transitions, guards, events, and actions using Mermaid `stateDiagram-v2` (mandatory if the story involves state transitions or lifecycle expirations).
       - *Notation*: States must be in PascalCase. Transitions must be annotated with `event [guard] / action` on the transition arrow. Use `[*]` for entry/exit points. Use `-. label .->` syntax for dotted links.
     - **SysML Test Case & Verification Binding (Check 22)**:
       - Every BDD scenario MUST declare and bind to a formal SysML `test case def` block.
       - The `test case def` MUST declare a `verify requirement` binding pointing to the parent safety requirement (e.g. `verify requirement REQ_SAF_001;`).
       - Specify the test case subject part, objective statement, and execution steps.
   - **The Cross-Cutting Matrix (Feature Linking):**
     - Inspect the provided structural features to determine exactly which of those `#IssueID`s are prerequisites for the current User Story.
     - Construct the `## Required Features` matrix containing a markdown tasklist of these intersecting links referencing BOTH the Issue ID and the absolute URL of the feature document.
     - Every checklist item in the matrix MUST include a concise parenthetical justification explaining the semantic linkage.
    - **Formatting of Alphanumeric Identifiers & Math Expressions**: Ensure all requirement references, hazard tags, and SORA SAIL codes use standard bold text (`**SC-01**`, `**H-1**`, `**OSO-11**`) rather than inline LaTeX math mode (`$SC-01$`). Non-mathematical alphanumeric tokens must NEVER be wrapped in `$...$` math delimiters per `rules/latex-katex-integrity.md`.
    - **Pure Symbolic Mathematical Separation Rule**: When specifying mathematical derivations, calculations, kinematic equations, or control laws:
      - Display math blocks (`$$ \begin{aligned} ... \end{aligned} $$`) MUST express **pure symbolic equations only**.
      - Strictly prohibit embedding physical unit macros (`\text{ ms}`, `\text{ kg}`, `\text{ m/s}`, etc.) inside display math equations.
      - Mandate that all physical values, numerical limits, constants, calibration thresholds, and engineering units are defined in the accompanying "- Parameter Definitions & Engineering Units:" text section immediately following the equation.
      - Strictly prohibit dangling operators (such as `/` with no denominator) and unescaped underscores inside `\text{}` (mandating `\text{yaw-disturbance}` or structured subscripts like `\Delta v_{\text{yaw,dist}}`).
      - Multi-line aligned equations MUST use `\begin{aligned} ... \end{aligned}` inside `$$` delimiters on dedicated lines.
    - **Tandem Elaboration & Zero Model Drift:** Any newly derived operations, algorithmic methods, state transitions, port interactions, or verification test cases identified during User Story modeling MUST be reflected back into the SysML v2 model (`.pipeline/schema.sysml`) as `interaction def`, `action def`, `state def`, `port def`, or `test case def` elements per `rules/sysml-ssot-completeness.md`.
    - **Markdown Generation:** Write the User Story as a local markdown file (e.g., `docs/user-stories/us-01-register-entity.md`).
4. **Return Control:** The subagent completes the task and returns control to the worker agent.

## Step 4: Markdown Generation
Create a new file in `docs/user-stories/us-[XX]-[name].md` (zero-padded, dash-separated, e.g., `us-01-register-entity.md`).
> **Unified Slugification Mandate:** When generating filenames from titles (e.g., `us-29-fiber-cable-and-strand-inventory.md`), you MUST preserve all stop-words (like 'and', 'the', 'of', etc.) consistently. Do NOT strip stop-words when converting titles to lowercase hyphen-separated slugs.

Format strictly:

````markdown
| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #[IssueID] |
| **Title** | [User Story Title] |
| **Type** | user-story |
| **Parent Epic** | #[EpicIssueID] - [Epic Title](../epics/epic-XX-name.md) |
| **SysML Interaction** | `[SysMLInteractionName]` |
| **SysML Test Case** | `[SysMLTestCaseName]` |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/...](../../schema/...) |

# User Story: [Title]

## Parent Epic
- [ ] #[EpicIssueID] - [Epic Title](../epics/epic-XX-name.md) (semantic linkage justification)

## Domain Object Mapping
- **Primary Domain Objects:** [List affected structural schema entities / SysML parts]
- **Actor/Role:** [The object/entity initiating the action]

## BDD Scenario (OOA/OOD Realization)
**Given** [Initial system/object state]
**When** [Triggering action/event/message]
**Then** [Resulting system/object state]

*(Alternatively)*
**As a** [Actor]
**I want to** [Action]
**So that** [Outcome/State Change]

## UML Sequence Diagram
```mermaid
sequenceDiagram
    autonumber
    actor userActor as "userActor : UserActor"
    participant domainRegistry as "domainRegistry : DomainRegistry"
    participant businessLogicService as "businessLogicService : BusinessLogicService"

    userActor->>domainRegistry: operationName(attributeName: DataType)
    alt [payloadIsValid == true]
        domainRegistry->>businessLogicService: validateBounds(attributeName: DataType)
        businessLogicService-->domainRegistry: isValid : Boolean
        alt [isValid == true]
            Note over domainRegistry: Store value
            domainRegistry-->userActor: status : Status
        else [isValid == false]
            domainRegistry-->userActor: "status : Status"
        end
    else [payloadIsValid == false]
        domainRegistry-->userActor: status : Status
    end
```

## UML State Machine Diagram
*(Mandatory if the story involves state transitions or lifecycle expirations)*
```mermaid
stateDiagram-v2
    [*] --> InitialState
    InitialState --> ActiveState : "activate [activationCodeIsValid == true] / initializeSession"
    ActiveState --> TerminatedState : "expire [timeElapsed >= timeoutLimit] / cleanupResources"
    TerminatedState --> [*]
```

## Formal SysML Test Case & Verification Binding
- **SysML Test Case Def:** `TC_[StoryName]_[ID]`
- **Subject Part:** `[PartName]`
- **Verified Safety Requirement:** `[REQ_SAF_XXX]`
- **Verification Objective:** "[Verification objective statement]"
- **Test Steps:**
  - `step inject_stimulus`
  - `step assert_safety_response`

```sysml
test case def TC_[StoryName]_[ID] {
    subject [PartName];
    verify requirement [REQ_SAF_XXX];
    objective "[Verification objective statement]";
    step inject_stimulus;
    step assert_safety_response;
}
```

## Operational Context
[Verbatim operational constraints or deployment scenarios quoted from the specification]

### Mathematical Formulations & Derivations (When Applicable)
$$
\begin{aligned}
E_{total} &= E_{kinetic} + E_{potential} \\
P_{mech} &= \tau \cdot \omega
\end{aligned}
$$

- Parameter Definitions & Engineering Units:
- $E_{total}$: Total mechanical energy of the system.
- $E_{kinetic}$: Kinetic energy ($\frac{1}{2} m v^2$).
- $E_{potential}$: Potential energy ($m g h$).
- $\tau$: Motor shaft torque.
- $\omega$: Angular velocity.

## Required Features Matrix
- [ ] #[IssueID] - [Feature Title](../features/feat-XX-name.md) (semantic linkage justification)
- [ ] #[IssueID] - [Feature Title](../features/feat-XX-name.md) (semantic linkage justification)

## Logical UI & Interface Bindings
*(Required for UI/LUMI features. Raw 'N/A' fallback strings and literal placeholder strings ('#X', 'Task Y') are strictly prohibited.)*
<!-- Single-Channel (Visual GUI) Format -->
- **Target LUI Component:** [Specify canonical LUI component e.g. StringInputField, TableView, PropertyGrid, OR 'Unbound (Deferred to Implementation Profile)']
- **Target Layout Container ID:** [Specify container ID from logical-layout.json, OR 'Unbound (Deferred to Implementation Profile)']
- **Data Source Bindings:** [Specify exact, authoritative schema path locator e.g. /nwi:network-inventory/nil:locations/nil:location/nil:geo-location/nil:reference-frame, OR 'Unbound (Deferred to Implementation Profile)']

<!-- OR Multi-Channel (Multi-Interface) Format -->
| Interface Channel | Category | Target Component / Handler | Target Container / Endpoint | Data Source Binding |
| --- | --- | --- | --- | --- |
| gui | Visual GUI | StringInputField | elements_view | /schema:path |
| mcp | M2M API | MCPToolHandler | /mcp/tool | /schema:path |

## Source References
> [!IMPORTANT]
> **Dynamic Schema Locator**: You MUST inspect the active workspace directories (e.g. `schema/`) to build schema locators dynamically. Do NOT hardcode legacy paths like `standard/ietf/RFC/`.

Structural Schema: [Target Schema File](link-to-schema)
Normative Specification: [Normative Specification](link-to-specification)
````

> [!WARNING]
> **Mermaid Block Closing Constraints & Code Fence Integrity:**
> - Every Mermaid diagram MUST be strictly closed with ```` ``` ```` on a new line. Leaking Mermaid blocks (e.g. having headings like `##` inside an unclosed diagram) or stray/unclosed code fences will fail downstream validation checks.
> - Ensure there are no stray backticks or unmatched code fences in the document.
> - **All Mermaid syntax constraints are defined in `rules/platform-independence.md` and MUST be observed in full** — including the prohibition on semicolons in `Note` and message text, colons in class members and note strings, stereotypes on relationship lines, and curly braces in class member lines. Do not maintain a local subset here; subsets drift (issue #289).



## Step 5: Zero-Fault Backlog Synchronization
1. **Mandatory Local Validation Gate:** Before committing, pushing, or creating issues in the backlog, the subagent MUST execute the local validation check:
   ```bash
   ./skills/spec-orchestrator/scripts/verify_model_coverage.py --spec-only --allow-missing-specs
   ```
   If the linter fails (returns a non-zero exit code), the subagent MUST parse the errors, fix all generated User Story markdown files, and re-run the linter until it passes with exit code 0.
   Before committing the generated markdown files, the agent MUST run a check for untracked pipeline infrastructure files. If untracked files are found in `.pipeline/`, `skills/`, `rules/`, or `scripts/`, they must be staged and committed alongside the markdown files using `git add` to prevent remote divergence:
   ```bash
   UNTRACKED_INFRA=$(git ls-files --others --exclude-standard .pipeline/ skills/ rules/ scripts/)
   if [ -n "$UNTRACKED_INFRA" ]; then
     git add .pipeline/ skills/ rules/ scripts/
   fi
   ```
   Once the linter passes, commit and push the Markdown files to the remote repository.
2. Verify the `user-story` label exists in the tracker repository, bootstrapping it if necessary.
3. **Duplicate Detection:** Before creating, query the active tracker provider for all existing user story issues to check if an issue with an identical or semantically equivalent title already exists. If found, skip creation and reuse the existing Issue ID.
4. Register the User Story issue natively with the active tracker provider.
   - **Crucial Verification & Body Synchronization:**
     1. Backlog issues MUST be registered using the deterministic title extraction step:
        ```bash
        TITLE=$(awk -F'|' '/**Title**/ {print $3}' <local-md-file> | xargs)
        gh issue create --title "$TITLE" --body-file <local-md-file>
        ```
        (to ensure they start with the full markdown content, including diagrams and references).
     2. Immediately after placeholder resolution (when the live issue ID is injected back into the file), the subagent MUST execute `gh issue edit <ID> --body-file <local-md-file>` to sync the resolved ID body.
     3. The subagent MUST run a post-creation verification check:
        `gh issue view <ID> --json body | python3 -c "import sys,json; b=json.load(sys.stdin)['body']; assert 'Source References' in b or 'References' in b, 'Body is a stub'"`
        and retry/halt if this verification fails.
5. Verify the creation and return the generated issue URLs/IDs to the Orchestrator or User.
