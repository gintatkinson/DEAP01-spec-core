<!-- Copyright Gint Atkinson, gint.atkinson@gmail.com -->

# Rule: UML and Backlog Document Model Integrity

**ALWAYS enforce:** every Epic, Feature, User Story and Use Case must be structurally
complete, and every UML diagram it carries must be a valid model rather than a picture.

## Scope and normative home

**This file is the single normative home for the model-integrity constraints enforced by
`parity_auditor/validators/uml.py`.** They span all four backlog document types, so
stating them in any one worker skill would fragment them across four files with disjoint
subsets — the failure issue #289 fixed for the Mermaid rules by designating one home.
The worker skills own the *templates*; this file owns the *rules* the templates exist to
satisfy.

Three rules enforced here are **not** restated below, because they already have a
normative home and restating them is prohibited by
`rules/platform-independence.md` § *Normative home & enforcement*:

- braces inside class members — *Mermaid Class Member Brace Rules*
- colons inside note strings — *Mermaid Note Rules*
- stereotypes on relationship lines — *Mermaid Relationship Rules*

The subagent generation-mode marker likewise keeps its existing home in
`skills/spec-orchestrator/SKILL.md`.

Every rule below was enforced before issue #304 and stated in no document. Section and
diagram requirements are read from `codebase_rules.json`, so the *rule* is that the
configured requirement is met; the configuration supplies which sections and diagrams
those are.

## Corpus and configuration

- **Epic Directory Must Exist When Configured**: if `backlog_directories.epics` is set,
  the directory must exist. Absent, Epic class diagrams are silently dropped from the
  cross-document class registry, and every User Story lifeline that resolves against an
  Epic-defined class is then reported as undefined for the wrong reason.
- **Validator Configuration Must Be Complete**: `required_sections` and
  `required_diagrams` must be present for each document type. A missing key is reported
  rather than treated as "nothing required", because an empty requirement set makes every
  document trivially compliant.
- **Backlog Documents Must Be Readable**: a document that cannot be read is reported, not
  skipped. Skipping shrinks the audited corpus without saying so.

## Document structure

- **Documents Must Carry Their Configured Sections**: every document must contain each
  section header configured for its type.
- **Documents Must Carry Their Configured Diagrams**: every document must contain a valid
  diagram of each type configured for it. A document may describe behaviour in prose and
  still fail, which is intended: the diagram is the machine-checkable artefact.
- **Features Must Carry A Test Data Payload Example**: a Feature must include a payload
  example block under Test Data Shape. Without one the described shape cannot be
  exercised, so the Feature specifies a contract nothing can test against.
- **User Stories Must Carry A BDD Scenario**: a User Story must contain either
  Given-When-Then or the As-a / I-want-to / So-that form. A story with neither states no
  observable behaviour.
- **User Stories Must Carry A Required Features Matrix**: a User Story must contain the
  `## Required Features Matrix` section, which is how behaviour is traced back to
  structure.
- **The Features Matrix Must Reference At Least One Feature**: an empty matrix records no
  traceability while looking like it does.
- **Use Case Filenames Must Follow The Naming Convention**: use case files must match the
  configured naming pattern, so ordinals and titles remain machine-derivable.
- **Use Cases Must Carry Alternate And Exception Flows**: a Use Case must contain at least
  as many detailed alternate or exception flows as the referenced Features declare schema
  validation constraints. Happy-path-only Use Cases are how validation requirements get
  specified and never implemented.
- **Alternate Flows Must Be Detailed**: each alternate flow must contain at least the
  configured number of numbered steps. A one-line flow is a heading, not a flow.
- **Use Cases Must Carry An Alternate Flows Block**: the configured alternate-flows header
  must be present and non-empty.
- **Use Cases Must Carry A Complete Realization Matrix**: both the required User Stories
  header and `### Required Features` must be present under the Realization Matrix.
- **The Realization Matrix Must Carry Checklist Entries**: both sections must contain at
  least one checkbox. A present-but-empty matrix asserts traceability that does not exist.

## Traceability links

- **Checklist Items Must Carry An Absolute URL**: every checklist item in a Required
  Features Matrix or Realization Matrix must be a markdown link to an absolute URL.
  Relative paths break the moment the content is published to the tracker, which
  `rules/tracker-source-of-truth.md` makes the authoritative view.
- **Checklist Items Must Carry A Semantic Justification**: each checklist item must end
  with a parenthetical stating why the referenced item is required. A bare link records
  that two documents are related and not how.
- **Epic Checklist Items Must Link To A Feature File**: an Epic's checklist entries must
  be valid markdown links pointing at the referenced Feature's absolute URL.
- **Specifications Must Not Contain Template Placeholders**: unpopulated template stubs
  must not survive into a registered document. A placeholder that reaches the tracker
  reads as a specification and contains none.
- **Specifications Must Not Contain Unresolved Registration Tokens**: unresolved
  identifier tokens such as `#[EpicID]` must be replaced with the registered issue number
  before the document is published. Every referenced item must be explicitly registered.
- **Epic Prohibit Unreplaced Placeholder Text**: Epic specifications must not contain literal '(semantic linkage justification)' or '[POPULATE:' placeholder tokens. All placeholders must be replaced with concise justifications.

## Sequence diagrams

- **Sequence Diagrams Must Parse**: a sequence diagram that does not parse is reported
  rather than skipped, so a malformed diagram cannot pass as an absent one.
- **Lifelines Must Declare A Name And A Classifier**: every lifeline label must use the
  `name : Classifier` form. The classifier is what ties behaviour to the structural model;
  a bare name ties it to nothing.
- **Lifeline Classifiers Must Be Defined**: every non-actor lifeline's classifier must be
  a class defined in some Feature or Epic class diagram. External actors, declared with
  the UML `actor` keyword, are outside the system boundary and are exempt.
- **Messages Must Carry An Operation Signature**: a message must name an operation, not
  prose. A sentence on an arrow cannot be checked against the receiver's interface.
- **Message Operations Must Exist On The Receiver**: the named operation must be defined
  on the receiving lifeline's class. This is the check that makes a sequence diagram a
  model of the class diagram rather than an independent drawing.
- **Message Operations Must Be Public**: an operation invoked across a lifeline boundary
  must carry `+` visibility. Calling a private operation from outside its class is not a
  legal interaction.
- **Return Messages Must Use A Reply Arrow**: returns must use the standard dashed reply
  arrow. A solid arrow back is a second call, and the diagram then describes a different
  interaction from the one intended.
- **Return Messages Must Not Look Like Calls**: a return message must be a simple value or
  description and must not contain parentheses, which would read as an operation call.
- **Combined Fragment Guards Must Be Bracketed**: the guard of an `alt`, `opt`, `loop` or
  `par` fragment must be enclosed in square brackets. Unbracketed, the guard is parsed as
  part of the fragment label and the condition is lost.

## Use case diagrams and SysML v2 use case definitions

- **Use Case Flowcharts Must Parse**: as for sequence diagrams, a malformed flowchart is
  reported rather than skipped.
- **Use Cases Must Declare A System Boundary Subgraph**: the diagram must contain a
  subgraph whose id or label identifies the system boundary. Without it, inside and
  outside the system are indistinguishable and the two placement rules below cannot be
  evaluated.
- **Actors Must Sit Outside The System Boundary**: an actor is by definition external. An
  actor drawn inside the boundary asserts the system contains its own user.
- **Use Case Nodes Must Sit Inside The System Boundary**: a use case is a service the
  system offers, so it must be within the boundary.
- **Use Case Nodes Must Use The Stadium Shape**: use case nodes must use the Mermaid
  stadium shape, which is the notation's oval. Shape is what distinguishes an actor from
  a use case when the diagram is read rather than parsed.
- **Actor Associations Must Be Undirected**: the connection between an actor and a use
  case is an association, not a dependency or a flow, and must use an undirected link.
- **SysML Use Case Definitions Must Declare A Subject**: every `use case def` in the SysML v2
  model must declare a `subject` binding to a valid structural component (`part def`) that
  provides the operational capability.
- **SysML Use Case Definitions Must Declare Actors**: every `use case def` must declare
  initiating and participating `actor` roles matching external actors outside the system boundary.
- **SysML Use Case Definitions Must Declare Objectives**: every `use case def` must specify an
  `objective` defining the formal goal and measurable success criteria of the use case.
- **SysML Use Case Includes And Extends Must Declare Conditions**: use case decomposition using
  `include` or `extend` must reference valid `use case def` nodes; conditional `extend`
  relationships must carry explicit guard conditions.
- **Use Case Document Parity**: every Use Case specification in the backlog must correspond to
  a declared `use case def` in the SysML v2 model with matching identifier, actor bindings,
  preconditions, and postconditions.

## Statechart and state machine transitions

- **Statecharts Must Form A Deterministic Automaton**: state machine diagrams (`stateDiagram-v2`
  in Mermaid and `state def` in SysML v2) must define an explicit initial state `[*]`, reachable
  named states, and valid terminal states where applicable. Isolated or unreachable states are
  prohibited.
- **State Transitions Must Declare Triggers And Bracketed Guards**: every conditional transition
  between states must declare a trigger event, an explicit guard expression enclosed in square
  brackets `[guard]`, and an optional effect action `/ action`. Unbracketed guards or ambiguous
  event triggers are prohibited.
- **State Transition Endpoints Must Exist**: every transition line must connect source and
  target states that are explicitly defined within the enclosing state machine or component.
  Transitions pointing to undeclared states are prohibited.
- **Stateflow Execution Parity**: hierarchical composite states, transition priority orderings,
  and orthogonal parallel states must maintain deterministic execution semantics compatible with
  MATLAB / Simulink / Stateflow model synthesis.

## Actor port bindings and interface integrity

- **Actor Messages Must Traverse Formal Ports**: messages and interactions passing between
  external actors and internal system classifiers in sequence diagrams or SysML interaction
  flows must traverse explicitly declared `port def` endpoints on the system boundary `part def`.
- **Port Interfaces Must Declare Flow Direction And Item Type**: every `port def` must define
  a flow direction (`in`, `out`, `inout`) and reference a valid `item def` or typed interface.
  Untyped or raw unstructured port attachments are prohibited.
- **Port Bindings Must Be Fully Connected**: every port declared on an internal subsystem
  component must connect (`connect`) to a parent boundary port or bind to a peer component port.
  Unconnected or dangling ports are prohibited.
- **Message Signatures Must Match Port Interface Contracts**: an operation or data payload
  sent across an actor port boundary must correspond to an operation or item type declared on
  the receiving port's interface.

## Class diagrams

- **Class Diagrams Must Parse**: an unparsable class diagram is reported, so it cannot
  contribute an empty class set to the cross-document registry and make every lifeline
  referencing it fail instead.
- **Class Diagrams Must Declare Relationships**: a diagram with no relationships at all is
  a list of names. Containment, inheritance or association must be illustrated.
- **Relationship Connectors Must Be Recognised**: relationships must use one of the
  configured connector formats, or the parser records no edge and the connectivity rules
  below silently see a disconnected graph.
- **Classes Must Not Be Isolated**: a class with zero relationships is prohibited. This is
  a narrower rule than the one above: the diagram may be richly connected and still leave
  one class attached to nothing.
- **Class Diagrams Must Be Connected**: every class must be reachable from the diagram's
  root. Two disconnected clusters are two models sharing a fence.
- **Attributes Must Declare A Type**: an untyped attribute cannot be checked against the
  schema node it realises.
- **Attribute Types Must Be UML Primitives**: attribute types must come from the UML
  primitive set. A platform type here is the Tier 1 contamination
  `rules/platform-independence.md` prohibits, reaching the model instead of the prose.
- **Choice Classes Must Have A Subclass**: a class realising a schema `choice` must have at
  least one subclass inheriting from it via generalization. A choice with no cases models
  a decision with no outcomes.
- **Members Must Declare A Visibility Prefix**: every attribute and method must carry a UML
  visibility prefix. Visibility is what the sequence-diagram public-operation rule above
  is checked against, so an unprefixed member makes that rule unevaluable.
- **Composition And Aggregation Relationships Must Declare A Multiplicity**: every
  composition or aggregation relationship must carry a multiplicity tag on at least one
  association end (`1`, `0..1`, `0..*`). This is distinct from the member rule below,
  which governs attributes and return signatures: a whole-part relationship with no
  cardinality states that a part belongs to a whole without saying how many, which is
  exactly the schema fact the relationship exists to carry.
- **Members Must Declare A Multiplicity**: every attribute, and every method return
  signature, must declare a multiplicity such as `[1]`, `[0..1]` or `[0..*]`. Optionality
  and cardinality are schema facts and are lost if they are not written down.
- **Subsystem Component Classes Must Declare Members**: a class representing a subsystem
  component must define at least one attribute or operation. An empty component names a
  boundary and specifies nothing behind it.

## Schema traceability in class diagrams

- **Class Diagrams Must Model The Schema Container Path**: every segment of a declared
  `schema_containers` path must appear as a class node in the diagram. This is what makes
  the diagram checkable against the schema rather than merely inspired by it.
- **Class Diagrams Must Model The Schema Containment Relationships**: consecutive segments
  of that path must be joined by a relationship representing containment. Present nodes
  with absent edges reproduce the schema's vocabulary without its structure.
- **SysML Nodes Must Be Extracted Into A Feature**: every node discovered in a SysML v2
  model must appear in at least one Feature specification. An unextracted node is a
  modelled element with no functional requirement behind it, which is the coverage gap
  `.pipeline/constitution.md` § *Data Model Integrity* prohibits ("Every schema
  definition, model node, data object ... MUST map to at least one Feature"). Stated here
  because this file is the normative home for the constraints `cardinality_validator.py`
  and `uml.py` enforce; it was previously anchored to a heading in `implementation_plan.md`
  that no longer exists, leaving the rule enforced and undocumented.
- **SysML Models Must Be Readable**: a SysML v2 model file must exist in the schemas directory and be readable as UTF-8 text for cardinality validation.
- **SysML Feature Specifications Must Be Readable**: feature specification markdown files must be readable as UTF-8 text when validating SysML model extraction coverage.

## Why

A diagram that renders is not a model. Every rule here exists because the corresponding
defect passes visual review: the diagram looks right, the document reads complete, and the
missing type, unbracketed guard, unreachable class or placeholder link is only discovered
when something downstream tries to use it.

