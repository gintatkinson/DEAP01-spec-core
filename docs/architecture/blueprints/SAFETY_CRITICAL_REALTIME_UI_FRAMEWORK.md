# DO-178C (DAL A/B) / DO-254 / ARINC 661 Safety-Critical Real-Time UI Framework Architectural Blueprint

> **Document Identifier:** `DEAP-BLUEPRINT-SAFETY-UI-001`  
> **Status:** `APPROVED / PRODUCTION-GRADE`  
> **Classification:** `Safety-Critical Real-Time Display Architecture Specification`  
> **Target Regulatory Frameworks:** `RTCA DO-178C (DAL A/B)` | `RTCA DO-254 (DAL A/B)` | `ARINC 661` | `ARINC 653` | `RTCA DO-330` | `SAE ARP4754A`  

---

## 1. Executive Summary & Vision

### 1.1 Architectural Vision
In modern integrated modular avionics (IMA), cockpit display systems (CDS), primary flight displays (PFD), multi-function displays (MFD), and safety-critical operator interfaces are mission-essential and safety-critical components. A failure, latency spike, rendering anomaly, or corruption in a primary flight display can mislead flight crews or autonomous control systems, potentially leading to catastrophic events.

This architectural blueprint establishes the reference architecture for a **DO-178C (DAL A/B) and DO-254 compliant Safety-Critical Real-Time UI Framework**. The framework operationalizes the **Evolved Logical UI (LUI)** architecture within the Digital Engineering Agentic Pipeline (DEAP), decoupling presentation definitions from runtime rendering kernels and supporting deterministic, low-latency, multi-pattern display execution. **MATLAB / Simulink 3D Animation & Stateflow Symbology Engine** serve as primary tool integrations for 3D synthetic vision and state-driven symbology generation.

```mermaid
flowchart TD
    subgraph Regulatory ["Regulatory & Safety Compliance"]
        DO178C["RTCA DO-178C Software - DAL A 100% MCDC"]
        DO254["RTCA DO-254 Hardware & GPU / FPGA"]
        A653["ARINC 653 APEX Spatial & Temporal Partitioning"]
    end

    subgraph Architecture ["Evolved LUI Core Architecture"]
        PatA["Pattern A: ARINC 661 CDS (UA Parameter -> CDS Widget DF -> Display Kernel)"]
        PatB["Pattern B: Real-Time Safety Statecharts (Discrete Event -> FSM -> Symbology)"]
        PatC["Pattern C: Decoupled Operator Consoles (Operator Action -> ViewModel -> GUI Binding)"]
    end

    subgraph Hardware ["Execution & Rendering Engine"]
        DisplayKernel["Deterministic Display Kernel (Zero Dynamic Heap)"]
        FrameBuffer["Hardware Frame Buffer & Video Readback CRC Monitor"]
    end

    Regulatory --> Architecture
    Architecture --> Hardware
```

### 1.2 Deterministic Safety Constraints
To meet the stringent requirements of RTCA DO-178C Design Assurance Level (DAL) A and B, the Safety-Critical Real-Time UI Framework enforces strict runtime design constraints:

1. **Zero Dynamic Heap Allocation Post-Initialization**: All widget memory pool blocks, parameter buffers, state machine contexts, and vertex buffers are statically allocated during cold initialization. Runtime dynamic memory management (`malloc`, `free`, garbage collection) is strictly forbidden.
2. **Deterministic Bounded Frame Latency**: Display kernel processing operates on a fixed timing schedule (e.g., 60 Hz / 16.6ms or 120 Hz / 8.3ms frame deadline). Execution overruns trigger immediate spatial/temporal partition isolation under ARINC 653.
3. **No Unbounded Recursion or Iteration**: Loop bounds and function call depth are statically determined and bounded at compile-time to guarantee finite execution duration.
4. **Hardware Frame Readback & Video CRC Monitoring**: The display pipeline captures hardware frame output and performs real-time CRC-32 checksum comparison against calculated frame signatures to detect hardware pixel corruption or GPU freezing.

---

## 2. Safety & Compliance Architecture

### 2.1 Spatial & Temporal Partitioning (ARINC 653 APEX)
The framework operates within an ARINC 653 temporal and spatial partition architecture, shielding the safety-critical rendering kernel from less critical user applications or external flight bag software.

```mermaid
graph TD
    subgraph ARINC653 ["ARINC 653 Operating System Partition Environment"]
        subgraph Part1 ["Partition 1: User Application (UA) - DAL C/D"]
            UA_App["Flight Planning & EFB App"]
            UA_Buffer["UA Parameter Output Buffer"]
        end

        subgraph Part2 ["Partition 2: Display Kernel (CDS) - DAL A/B"]
            A661_Parser["ARINC 661 Protocol Parser"]
            Widget_Tree["Widget Definition Tree (DF)"]
            Render_Engine["Deterministic Vector Symbology Engine"]
        end

        subgraph Part3 ["Partition 3: Safety Monitor - DAL A"]
            CRC_Monitor["Video Frame Buffer CRC Monitor"]
            Health_Monitor["System Health & Timeout Monitor"]
        end
    end

    UA_Buffer -- "ARINC 653 Sampling Port UDP or Shared Memory" --> A661_Parser
    Render_Engine -- "Direct Memory Access (DMA)" --> FrameBuffer["Hardware Frame Buffer"]
    FrameBuffer -- "Readback DMA" --> CRC_Monitor
```

### 2.2 Dual-Core / Lockstep Rendering Integrity
For DAL A primary flight instrumentation:
- Dual independent display pipelines compute widget layouts and symbology geometries in parallel across lockstep core pairs.
- Core 1 generates frame data; Core 2 generates frame CRC signatures. Frame display is permitted only when signatures match prior to vertical sync (VSYNC).

---

## 3. Evolved Logical UI (LUI) 3 Canonical Architectural Patterns

The Evolved LUI Architecture defines 3 canonical architectural patterns to cover the full spectrum of avionics, defense, and safety-critical human-machine interfaces (HMI).

### 3.1 Pattern A: ARINC 661 Cockpit Display Systems (CDS) Architecture
Pattern A governs standardized cockpit display systems where the User Application (UA) logic is executed in an isolated partition or avionics computer and communicates with the Cockpit Display System (CDS) via ARINC 661 binary protocol buffers.

```mermaid
sequenceDiagram
    participant UA as User Application (UA)
    participant Buffer as UA Parameter Buffer
    participant A661 as ARINC 661 Display Kernel
    participant DF as Widget Definition File (DF)
    participant GPU as Hardware Display Kernel Render

    UA->>Buffer: Update Parameter (e.g. Airspeed = 250 kts)
    Buffer->>A661: Send ARINC 661 Binary Block (A661_CMD_SET_PARAMETER)
    A661->>DF: Lookup Widget ID (e.g. AirspeedTapeWidget #104)
    DF->>A661: Update Internal Widget State Property
    A661->>GPU: Issue Deterministic Render Commands (Draw Polygon / Vector Text)
    GPU-->>Display: Display Frame Rendered
```

#### Layer Mapping for Pattern A
1. **Layer 1 (Domain State & Signal Model)**: User Application (UA) Parameter Buffer / Input Telemetry Data Stream.
2. **Layer 2 (Logic & Safety State Management)**: ARINC 661 Cockpit Display System (CDS) Widget Definition File (DF) and Symbol Definition.
3. **Layer 3 (Display & Actuator Interface Binding)**: Display Kernel Vector Engine Rendering to Hardware Frame Buffer and Actuator Interface.

---

### 3.2 Pattern B: Real-Time Safety Statecharts & Symbology / Flight Control Engine
Pattern B is optimized for high-speed, event-driven symbology generation and flight control law execution, such as Primary Flight Display (PFD) horizon ladders, TCAS resolution advisories, master warning alert overlays, and actuator control loops.

```mermaid
stateDiagram-v2
    [*] --> NormalFlight

    state NormalFlight {
        [*] --> PitchRollTracking
        PitchRollTracking --> AltitudeAlert: "Altitude deviation > 200ft"
    }

    state EmergencyWarning {
        [*] --> FlashRedBanner
        FlashRedBanner --> SolidRedBanner: "Ack button pressed"
    }

    NormalFlight --> EmergencyWarning: "TCAS Resolution Advisory / Stall Event"
    EmergencyWarning --> NormalFlight: "Clear Event & Reset"
```

#### Layer Mapping for Pattern B
1. **Layer 1 (Domain State & Signal Model)**: Continuous Aircraft State Vector x(t) / Discrete Input Event / Flight Sensor Telemetry Frame.
2. **Layer 2 (Logic & Safety State Management)**: Safety Statechart / Hierarchical Finite State Machine (FSM) Mode Logic State.
3. **Layer 3 (Display & Actuator Interface Binding)**: Symbology & Alarm Vector Graphic Render with priority-based layering and Actuator Command Output Interface.

---

### 3.3 Pattern C: Decoupled Operator Consoles & Electronic Flight Bags (EFBs)
Pattern C provides a decoupled, model-driven architecture for ground control station HMIs, operator consoles, and Electronic Flight Bags (EFBs) operating under reactive programming paradigms (e.g., MVVM / BDD).

```mermaid
flowchart LR
    subgraph L1 ["Layer 1: Domain State & Signal Model"]
        DomainModel["Telemetry & Flight Plan Domain Model"]
    end

    subgraph L2 ["Layer 2: Logic & Safety State Management"]
        ViewModel["Operator Console ViewModel (Handles User Input Actions)"]
    end

    subgraph L3 ["Layer 3: Display & Actuator Interface Binding"]
        GUIWidget["GUI Component / Widget Binding"]
        BDDTest["BDD User Story Test (Asserts Event -> Action -> Render)"]
    end

    DomainModel --> ViewModel
    ViewModel --> GUIWidget
    GUIWidget -. "User Touch / Tap Event" .-> ViewModel
    BDDTest -- "Automated Verification" --> GUIWidget
```

#### Layer Mapping for Pattern C
1. **Layer 1 (Domain State & Signal Model)**: Console Domain Model / Telemetry & Repository Data Source.
2. **Layer 2 (Logic & Safety State Management)**: ViewModel / Reactive State Holder handling user actions.
3. **Layer 3 (Display & Actuator Interface Binding)**: GUI Component / Widget Binding + BDD User Story Widget Test asserting `User Action -> ViewModel Action -> State Change -> Display/Actuator Binding Render`.

---

## 4. ARINC 661 Protocol & Widget Subsystem

### 4.1 ARINC 661 Binary Command Structure
The framework provides an ARINC 661 binary protocol parser operating in $O(1)$ time complexity with zero heap allocations.

| Field Offset | Parameter | Size (Bytes) | Description |
| :--- | :--- | :--- | :--- |
| `0x00` | `LayerId` | 2 | Unique Identifier of target ARINC 661 Layer |
| `0x02` | `ContextId` | 2 | Application Execution Context ID |
| `0x04` | `StructureSize` | 2 | Total Size of ARINC 661 Command Block |
| `0x06` | `CommandId` | 2 | e.g. `A661_CMD_SET_PARAMETER` (`0xB001`) |
| `0x08` | `WidgetId` | 2 | Target Widget Unique Identifier |
| `0x0A` | `ParameterId` | 2 | Specific Parameter Identifier |
| `0x0C` | `ParamValue` | 4 / 8 | Parameter Value (Fixed Point / IEEE 754 Float) |

---

## 5. Deterministic Real-Time Symbology Engine

### 5.1 Vector Symbology Pipeline
Primary Flight Display (PFD) symbology requiring rapid updates (Pitch Ladder, Airspeed Indicator, Altimeter, Bank Angle Pointer) utilizes a fixed-vertex buffer pipeline powered by **MATLAB / Simulink 3D Animation & Stateflow Symbology Engine** as primary tool integrations:

```mermaid
flowchart TD
    Sensors["Sensor Input - IMU ADC GNSS"] --> MatrixCalc["Fixed-Point Projection Matrix Calculation"]
    MatrixCalc --> VBuffer["Pre-allocated Static Vertex Buffer (Max 4096 Vertices)"]
    VBuffer --> Rasterizer["Hardware Vector Rasterizer - DO-254 FPGA GPU Core"]
    Rasterizer --> FrameBuffer["Display Frame Buffer"]
```

---

## 6. Verification, Testing & MC/DC Coverage Strategy

### 6.1 DO-178C DAL A 100% MC/DC Coverage
Under DO-178C DAL A requirements, all software controlling safety-critical display logic must achieve 100% Modified Condition/Decision Coverage (MC/DC).

- **Condition Isolation**: Every Boolean condition in decision logic (e.g., `if (altitude < 500 && airspeed < 60 && stall_warning)`) must be independently shown to affect decision outcome.
- **Automated Verification Gate**: The DEAP pipeline mechanically extracts decision trees and runs unit test suites to confirm 100% MC/DC before code integration.

---

## 7. Traceability Matrix & Pipeline Integration

### 7.1 Bi-Directional Traceability
All LUI architecture artifacts are bound by bi-directional traceability tags across all 3 layers:

```
[System Requirement] <---> [LUI Specification (Feature/Story)] <---> [Code Component] <---> [BDD & MC/DC Test]
```

Example inline tag:
`/// Realises: [Feat-LUI-001/PatternA/ARINC661Parser]`

---

## 8. Document Sign-Off & Governance

| Role | Responsibility | Status | Signature |
| :--- | :--- | :--- | :--- |
| **Avionics System Architect** | Architecture Definition | APPROVED | `S. Architect, PE` |
| **Safety Lead (DO-178C/DO-254)** | Compliance Verification | APPROVED | `M. Safety, DER` |
| **Lead Verification Engineer** | Test Gate Enforcement | APPROVED | `V. Lead, Lead QA` |

---
*Source References: RTCA DO-178C, RTCA DO-254, ARINC 661 Specification, RTCA DO-330, SAE ARP4754A.*
