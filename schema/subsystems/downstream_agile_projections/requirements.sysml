doc /* Downstream Specification Projections */
package DownstreamAgileProjections {
    doc /* UUIDv5: 7098189e-c5b3-5f42-91ba-3f9071e53e88 | Complexity: Class P (O( | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / RFC 4122 | Diagnostics: E0199, E0300, E0301 */
    requirement def REQ_0159_Deterministic_RFC_4122_UUIDv5_Document_Anchors_for_Specification_Artifacts {
        id = "REQ-0159";
        text = "The projection engine UUIDv5_Engine_0 of Subsystem 9 SHALL inject deterministic RFC 4122 UUIDv5 anchors into frontmatter headers of emitted specification artifacts (Artifact_Alpha).";
        attribute uuidv5 : String = "7098189e-c5b3-5f42-91ba-3f9071e53e88";
        attribute complexity_class : String = "Class P (O(";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / RFC 4122";
        attribute diagnostic_codes : String = "E0199, E0300, E0301";
        attribute ac_01_canonical_frontmatter_uuidv5_injection : String = "Given: AST with package Package_0 and entity Classifier_Alpha (Node_0). When: Lowering Artifact_Alpha via UUIDv5_Engine_0. Then: Emitted frontmatter id matches RFC 4122 UUIDv5 with Version = 5 and Variant = 2. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_boundary_path_length_conformance : String = "Given: AST node with canonical path length L = 4096 bytes. When: Computing SHA-1 compression and bitwise masks. Then: Processes 65 blocks, emitting canonical 36-character hyphenated UUIDv5. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_rejection_of_malformed_paths_and_banned_uuids : String = "Given: Node with host-absolute path or pseudo-random UUIDv4. When: Validating frontmatter anchors. Then: Fails closed, emits E0300 or E0301, and binds _{anchor}. Diagnostic: E0300, E0301.";
        attribute ac_04_delimiter_synchronization_token_error_recovery : String = "Given: Malformed model with invalid statements separated by ; and |. When: Recovering across delimiter boundaries. Then: Emits E0300 and E0301, resynchronizing until C_{max_errors}} emits E0199. Diagnostic: E0300, E0301, E0199.";
        attribute ac_05_bitwise_determinism_release_performance : String = "Given: Model with 10^4 artifacts across N_{max_workers}} threads. When: Compiling repeatedly under Release mode (--release). Then: Emits 0-byte diff churn; latency < 25 ms, RSS < 100 MB. Diagnostic: Diagnostic sink remains empty.";
        doc /* \begin{aligned} \mathcal{N}_{\text{topological}}(v)&=p_0\mathbin{::}p_1\mathbin{::}\dots\mathbin{::}\text{Name}(v) \\ H(v)&=\text{SHA1}(\text{NS}_{\text{DEAP}}\mathbin{\Vert}\text{Bytes}(\mathcal{N}_{\text{topological}}(v)))=(h_0,\dots,h_{19}) \\ u_i&=h_i\;(\forall i\notin\{6,8\}),\quad u_6=(h_6\operatorname{AND}0\text{x0F})\mathbin{|}0\text{x50},\quad u_8=(h_8\operatorname{AND}0\text{x3F})\mathbin{|}0\text{x80} \\ U(v)&=(u_0,\dots,u_{15})\in(\mathbb{Z}_{256})^{16},\quad\text{Version}(U(v))=5,\quad\text{Variant}(U(v))=2 \\ \mathcal{N}_{\text{topological}}(v_1)=\mathcal{N}_{\text{topological}}(v_2)&\iff U(v_1)=U(v_2) \\ \neg\text{IsValidPath}(\mathcal{N}_{\text{topological}}(v))&\implies\text{Diagnostic}(v,\text{E0300})\in\mathcal{D}_{\text{sink}}\land\text{Anchor}(v)\gets\bot_{\text{anchor}} \\ (\text{Version}(U(v))\ne 5\lor\text{IsNonDeterministic}(U(v)))&\implies\text{Diagnostic}(v,\text{E0301})\in\mathcal{D}_{\text{sink}}\land\text{Anchor}(v)\gets\bot_{\text{anchor}} \\ |\mathcal{D}_{\text{sink}}| \ge C_{\text{max\_errors}}&\implies\text{Diagnostic}(\text{E0199})\in\mathcal{D}_{\text{sink}} \end{aligned} */
        constraint def Invariant_REQ_0159_The_canonical_path_structure_SHA_1_digest_masking_and_diagnostic_triggers_satisfy;
        require Invariant_REQ_0159_The_canonical_path_structure_SHA_1_digest_masking_and_diagnostic_triggers_satisfy;
        verify by AC_01_Canonical_Frontmatter_UUIDv5_Injection;
        verify by AC_02_Boundary_Path_Length_Conformance;
        verify by AC_03_Rejection_of_Malformed_Paths_and_Banned_UUIDs;
        verify by AC_04_Delimiter_Synchronization_Token_Error_Recovery;
        verify by AC_05_Bitwise_Determinism_Release_Performance;
        satisfy by AgileProjectionEngine;
    }
    doc /* UUIDv5: ee29aa4c-1bd3-5422-a59c-c7c02690d164 | Complexity: Class P (O( | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / DAG Scheduling Specification | Diagnostics: E0199, E0300, E0302 */
    requirement def REQ_0160_Bottom_Up_Feature_First_Dependency_Ordering_and_DAG_Scheduling {
        id = "REQ-0160";
        text = "The downstream projection engine of Subsystem 9 SHALL schedule features in strict bottom-up topological order via Scheduler_1 over dependency DAG DependencyDAG_0 (G_{dep} = (F, E_{dep})), synthesizing leaf features before dependent composite features to eliminate forward reference gaps.";
        attribute uuidv5 : String = "ee29aa4c-1bd3-5422-a59c-c7c02690d164";
        attribute complexity_class : String = "Class P (O(";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / DAG Scheduling Specification";
        attribute diagnostic_codes : String = "E0199, E0300, E0302";
        attribute ac_01_canonical_bottom_up_feature_ordering : String = "Given: DAG with leaves Feature_Leaf_A, Feature_Leaf_B and composite Feature_Composite_C. When: Scheduling via Scheduler_1. Then: Confirms Order(Feature_Leaf_A}) < Order(Feature_Composite_C}) and Order(Feature_Leaf_B}) < Order(Feature_Composite_C}). Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_deterministic_lexicographical_tie_breaking : String = "Given: Ready queue with leaves Feature_Beta, Feature_Alpha, Feature_Gamma (deg^{-} = 0). When: Scheduling across repeated runs. Then: Extracts candidates in ascending lexicographical order (Feature_Alpha, Feature_Beta, Feature_Gamma) with 0-byte diff churn. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_cyclic_dependency_detection_and_rejection : String = "Given: Dependency cycle between Feature_Alpha and Feature_Beta. When: Evaluating cycle detection. Then: Rejects cyclic nodes, emits E0302 with cycle path, and binds sentinel _{sched}. Diagnostic: E0302 (CyclicFeatureDependency).";
        attribute ac_04_missing_prerequisite_and_delimiter_recovery : String = "Given: Reference to missing Feature_Missing followed by ; and Feature_Alpha. When: Delimiter error recovery processes tokens. Then: Emits E0300, recovers on ;, parses Feature_Alpha, halting at C_{max_errors}} with E0199. Diagnostic: E0300, E0199.";
        doc /* \begin{aligned} \text{deg}^{-}(f)&=|\{f_p\in\mathcal{F}\mid(f_p,f)\in\mathcal{E}_{\text{dep}}\}|,\quad\text{IsLeaf}(f)\iff\text{deg}^{-}(f)=0 \\ \forall(f_a,f_b)\in\mathcal{E}_{\text{dep}},&\quad\text{Order}(f_a)\lt\text{Order}(f_b) \\ Q_t&=\{f\in\mathcal{F}\setminus\mathcal{L}_{t-1}\mid\text{deg}^{-}_t(f)=0\},\quad f_t=\arg\min_{f\in Q_t}(\text{FQN}(f)) \\ \mathcal{L}_t&=\mathcal{L}_{t-1}\mathbin{\Vert}\langle f_t\rangle \\ (\text{HasCycle}(\mathcal{G}_{\text{dep}})\lor(f,f)\in\mathcal{E}_{\text{dep}})&\implies\text{Diagnostic}(f,\text{E0302})\in\mathcal{D}_{\text{sink}}\land\mathcal{G}_{\text{dep}}\gets\bot_{\text{sched}} \\ f_{\text{prereq}}\notin\mathcal{F}&\implies\text{Diagnostic}(f,\text{E0300})\in\mathcal{D}_{\text{sink}} \\ |\mathcal{D}_{\text{sink}}|\ge C_{\text{max\_errors}}&\implies\text{Diagnostic}(\text{E0199})\in\mathcal{D}_{\text{sink}},\quad\Delta_f=\text{UUIDv5}(\text{OID}_{\text{DEAP}},\text{FQN}(f)) \end{aligned} */
        constraint def Invariant_REQ_0160_FormalInvariant;
        require Invariant_REQ_0160_FormalInvariant;
        verify by AC_01_Canonical_Bottom_Up_Feature_Ordering;
        verify by AC_02_Deterministic_Lexicographical_Tie_Breaking;
        verify by AC_03_Cyclic_Dependency_Detection_and_Rejection;
        verify by AC_04_Missing_Prerequisite_and_Delimiter_Recovery;
        satisfy by AgileProjectionEngine;
    }
    doc /* UUIDv5: 62138c24-7491-5944-9c0b-c5775216cd2d | Complexity: Class P (O(|C| + |S|)) | Standard: OMG SysML v2 / INCOSE SE Handbook v5.0 / SAFe 6.0 | Diagnostics: E0199, E0300, E0303 */
    requirement def REQ_0161_Epics_Projection {
        id = "REQ-0161";
        text = "The projection engine of Subsystem 9 SHALL synthesize Epic specifications by projecting capability allocations across architectural subsystems under an M:N allocation model pursuant to OMG SysML v2 and INCOSE SE Handbook v5.0.";
        attribute uuidv5 : String = "62138c24-7491-5944-9c0b-c5775216cd2d";
        attribute complexity_class : String = "Class P (O(|C| + |S|))";
        attribute governing_standard : String = "OMG SysML v2 / INCOSE SE Handbook v5.0 / SAFe 6.0";
        attribute diagnostic_codes : String = "E0199, E0300, E0303";
        attribute ac_01_structural_decomposition_and_capability_projection : String = "Given: Subsystem Subsystem_0 with Part_0, Port_0 (D = 3), and Capability_0. When: Lowering Epic Epic_Alpha. Then: Emits Epic_Alpha with structural tree, capability table, and UUIDv5 anchors. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_multi_subsystem_capability_allocation : String = "Given: Capabilities mapped across one or more subsystems under an M:N allocation model. When: Projecting capability allocations across subsystems into Epics. Then: Confirms capabilities allocate across multiple subsystems (|AllocatedSubsystems(c)| >= 1) without disjoint partition rejection, synthesizing valid cross-cutting Epics. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_dangling_capability_allocation_rejection : String = "Given: Capability Capability_Alpha unallocated (|AllocatedSubsystems(c)| = 0) or allocated to non-existent subsystem S not in S. When: Validating capability allocation integrity. Then: Rejects model, emits E0303, and binds sentinel _{epic}. Diagnostic: E0303 (CapabilityAllocationViolation).";
        attribute ac_04_decomposition_cycle_and_delimiter_recovery : String = "Given: Subsystem Subsystem_0 with cyclic nesting (> D) and undeclared targets, separated by ;. When: Delimiter recovery processes faults. Then: Emits E0300 and E0303, recovers on ;, and halts at C_{max_errors}} with E0199. Diagnostic: E0300, E0303, E0199.";
        doc /* \begin{aligned} \mathcal{S}&=\{S_1,\dots,S_m\},\quad\mathcal{M}_{\text{capabilities}}=\{c_1,\dots,c_n\} \\ \mathcal{A}&\subseteq\mathcal{M}_{\text{capabilities}}\times\mathcal{S} \\ \text{AllocatedSubsystems}(c)&=\{S\in\mathcal{S}\mid(c,S)\in\mathcal{A}\} \\ \text{SubsystemCapabilities}(S_k)&=\{c\in\mathcal{M}_{\text{capabilities}}\mid(c,S_k)\in\mathcal{A}\} \\ \text{Epic}(c)&=\langle\text{name}(c),\text{FQN}(c),\text{AllocatedSubsystems}(c)\rangle,\quad\forall c\in\mathcal{M}_{\text{capabilities}} \\ \forall c\in\mathcal{M}_{\text{capabilities}},&\quad|\text{AllocatedSubsystems}(c)|\ge 1 \\ (\text{AllocatedSubsystems}(c)=\emptyset&\lor\exists S\in\text{AllocatedSubsystems}(c):S\notin\mathcal{S})\implies\text{Diagnostic}(c,\text{E0303})\in\mathcal{D}_{\text{sink}}\land c\gets\bot_{\text{epic}} \\ (\text{HasCycle}(\text{Hierarchy}(S_k))&\lor\text{depth}(S_k)\gt D)\implies\text{Diagnostic}(S_k,\text{E0300})\in\mathcal{D}_{\text{sink}}\land S_k\gets\bot_{\text{epic}} \\ |\mathcal{D}_{\text{sink}}|\ge C_{\text{max\_errors}}&\implies\text{Diagnostic}(\text{E0199})\in\mathcal{D}_{\text{sink}},\quad\Delta_x=\text{UUIDv5}(\text{OID}_{\text{DEAP}},\text{FQN}(x)) \end{aligned} */
        constraint def Invariant_REQ_0161_FormalInvariant;
        require Invariant_REQ_0161_FormalInvariant;
        verify by AC_01_Structural_Decomposition_and_Capability_Projection;
        verify by AC_02_Multi_Subsystem_Capability_Allocation;
        verify by AC_03_Dangling_Capability_Allocation_Rejection;
        verify by AC_04_Decomposition_Cycle_and_Delimiter_Recovery;
        satisfy by AgileProjectionEngine;
    }
    doc /* UUIDv5: 8fcd0f74-3160-5527-a581-e38bbfcdc2b9 | Complexity: Class P (O( | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / Downstream Projection Contract | Diagnostics: E0199, E0300, E0304 */
    requirement def REQ_0162_Epics_Projection {
        id = "REQ-0162";
        text = "The downstream engine of Subsystem 9 SHALL synthesize UML class and component architecture diagrams within Epic specifications using synthesizer Diagram_Alpha, streaming artifacts directly through ArtifactSink_0.";
        attribute uuidv5 : String = "8fcd0f74-3160-5527-a581-e38bbfcdc2b9";
        attribute complexity_class : String = "Class P (O(";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / Downstream Projection Contract";
        attribute diagnostic_codes : String = "E0199, E0300, E0304";
        attribute ac_01_canonical_architecture_diagram_synthesis : String = "Given: Model Package_0 declaring subsystem Classifier_Alpha with parts Node_0 and Node_1. When: Synthesizing architecture diagrams via Diagram_Alpha. Then: Emits valid UML class and component Mermaid diagrams. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_streaming_artifact_sink_memory_invariant : String = "Given: Epic specification with artifact size |A| 10 MB. When: Streaming chunks through ArtifactSink_0 with threshold B_{max_chunk_size}} = 64 KB. Then: Confirms resident memory satisfies M_{sink} 64 KB O(1). Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_phantom_diagram_edge_and_dangling_endpoint_rejection : String = "Given: Diagram referencing an edge to dangling endpoint Node_1 lacking AST backing. When: Enforcing diagram soundness. Then: Fails closed, emits E0300, and binds sentinel _{diag}. Diagnostic: E0300 (PhantomDiagramEdgeViolation).";
        attribute ac_04_streaming_sink_write_failure_and_error_recovery : String = "Given: Write fault on ArtifactSink_0 and syntax faults separated by ;. When: Delimiter error recovery processes faults. Then: Emits E0304, binds _{sink}, halts at C_{max_errors}} with E0199. Diagnostic: E0304, E0199.";
        doc /* \begin{aligned} \forall (u,v)\in E_{\text{diag}},&\quad\exists c\in\text{Connections}(\text{AST})\cup\text{Containments}(\text{AST})\land\text{endpoints}(c)=\{u,v\} \\ \forall u\in V_{\text{diag}},&\quad\exists n\in\mathcal{V}_{\text{AST}}\cap\text{Scope}(\text{Epic})\land\text{handle}(u)=n \\ M_{\text{sink}}\le&\;B_{\text{max\_chunk\_size}}\in O(1),\quad\lim_{|A|\to\infty}\frac{M_{\text{sink}}(|A|)}{|A|}=0 \\ u_i\prec_{\text{canon}} u_j&\iff\text{FQN}(u_i)\prec_{\text{lex}}\text{FQN}(u_j) \\ \neg\exists c:\text{endpoints}(c)=\{u,v\}&\implies\text{Diagnostic}(e,\text{E0300})\in\mathcal{D}_{\text{sink}}\land e\gets\bot_{\text{diag}} \\ \text{WriteFailure}(\text{ArtifactSink\_0})&\implies\text{Diagnostic}(\Sigma_{\text{sink}},\text{E0304})\in\mathcal{D}_{\text{sink}}\land\Sigma_{\text{sink}}\gets\bot_{\text{sink}} \\ |\mathcal{D}_{\text{sink}}|\ge C_{\text{max\_errors}}&\implies\text{Diagnostic}(\text{E0199})\in\mathcal{D}_{\text{sink}},\quad\Delta_x=\text{UUIDv5}(\text{OID}_{\text{DEAP}},\text{FQN}(x)) \end{aligned} */
        constraint def Invariant_REQ_0162_FormalInvariant;
        require Invariant_REQ_0162_FormalInvariant;
        verify by AC_01_Canonical_Architecture_Diagram_Synthesis;
        verify by AC_02_Streaming_Artifact_Sink_Memory_Invariant;
        verify by AC_03_Phantom_Diagram_Edge_and_Dangling_Endpoint_Rejection;
        verify by AC_04_Streaming_Sink_Write_Failure_and_Error_Recovery;
        satisfy by AgileProjectionEngine;
    }
    doc /* UUIDv5: e4967cb4-af4b-58bb-bad9-ba3824ba3ee7 | Complexity: Class P (O( | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 / KerML | Diagnostics: E0199, E0300, E0305 */
    requirement def REQ_0163_Epics_Projection {
        id = "REQ-0163";
        text = "Subsystem 9 projection engine SHALL synthesize macro statecharts and operational scenarios within Epic specifications.";
        attribute uuidv5 : String = "e4967cb4-af4b-58bb-bad9-ba3824ba3ee7";
        attribute complexity_class : String = "Class P (O(";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 / KerML";
        attribute diagnostic_codes : String = "E0199, E0300, E0305";
        attribute ac_01_canonical_macro_statechart_projection : String = "Given: State machine in Package_0 with states State_0, State_1. When: Synthesizing macro statechart MacroStatechart_Alpha. Then: Collapses micro-steps, projects transitions, and emits Mermaid stateDiagram-v2. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_operational_scenario_step_verification : String = "Given: Path from State_0 to State_1 on Event_0 with guard Guard_0. When: Synthesizing operational scenario Scenario_0. Then: Validates step sequence and verifies disjoint guards. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_reachability_and_bisimulation_violation_rejection : String = "Given: Macro transition asserted without a realizing AST path. When: Verifying reachability and bisimulation. Then: Fails closed, emits diagnostic E0300, and binds sentinel _{macro}. Diagnostic: E0300 (MacroReachabilityBisimulationViolation).";
        attribute ac_04_inconsistent_scenario_trace_and_delimiter_recovery : String = "Given: Scenario referencing ungrounded State_Undefined, delimited by ;. When: Executing delimiter recovery. Then: Emits E0305, resynchronizes, and halts at C_{max_errors}} with E0199. Diagnostic: E0305, E0199.";
        doc /* \begin{aligned} \alpha&:\mathcal{S}_{\text{AST}}\twoheadrightarrow S_{\text{macro}},\quad\bigcup_{m\in S_{\text{macro}}}\alpha^{-1}(m)=\mathcal{S}_{\text{AST}} \\ \forall m,m'\in S_{\text{macro}},&\quad m\xrightarrow{\sigma}m'\iff\exists s\in\alpha^{-1}(m),s'\in\alpha^{-1}(m'):s\xRightarrow{\sigma}s' \\ M_{\text{AST}}\sim_{\text{bisim}} M_{\text{epic}}&\iff\forall\langle s,m\rangle\in\mathcal{R}_{\text{bisim}},(\forall\sigma\in\Sigma_{\text{obs}},s\xrightarrow{\sigma}s'\implies\exists m':m\xrightarrow{\sigma}m') \\ \forall t_1,t_2\in\delta_{\text{macro}},&\quad(m_{\text{src}}(t_1)=m_{\text{src}}(t_2)\land e_{t_1}=e_{t_2}\land t_1\ne t_2)\implies(g_{t_1}\land g_{t_2}\equiv\text{false}) \\ \forall i\in\{1,\dots,p-1\},&\quad s_{\text{tgt}}(\text{step}_i)=s_{\text{src}}(\text{step}_{i+1})\land\exists t\in\delta_{\text{macro}}:\text{Matches}(t,\text{step}_i) \\ (\exists t\in\delta_{\text{macro}}:\neg\text{Realized}(t))&\lor\neg(M_{\text{AST}}\sim_{\text{bisim}} M_{\text{epic}})\implies\text{Diagnostic}(t,\text{E0300})\in\mathcal{D}_{\text{sink}}\land t\gets\bot_{\text{macro}} \\ (\exists\mathcal{O}:\neg\text{ValidTrace}(\mathcal{O}))&\lor\text{Conflict}(t_1,t_2)\implies\text{Diagnostic}(\mathcal{O},\text{E0305})\in\mathcal{D}_{\text{sink}}\land\mathcal{O}\gets\bot_{\text{scenario}} \\ |\mathcal{D}_{\text{sink}}|\ge C_{\text{max\_errors}}&\implies\text{Diagnostic}(\text{E0199})\in\mathcal{D}_{\text{sink}},\quad\Delta_x=\text{UUIDv5}(\text{OID}_{\text{DEAP}},\text{FQN}(x)) \end{aligned} */
        constraint def Invariant_REQ_0163_The_state_abstraction_quotient_map_behavioral_reachability_weak_bisimulation_scenario_step_sequence_and_diagnostic_rules_satisfy;
        require Invariant_REQ_0163_The_state_abstraction_quotient_map_behavioral_reachability_weak_bisimulation_scenario_step_sequence_and_diagnostic_rules_satisfy;
        verify by AC_01_Canonical_Macro_Statechart_Projection;
        verify by AC_02_Operational_Scenario_Step_Verification;
        verify by AC_03_Reachability_and_Bisimulation_Violation_Rejection;
        verify by AC_04_Inconsistent_Scenario_Trace_and_Delimiter_Recovery;
        satisfy by AgileProjectionEngine;
    }
    doc /* UUIDv5: 30be0357-f32d-5b49-a99b-ca931dc76215 | Complexity: Class P (O( | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / Downstream Projection Mandate | Diagnostics: E0199, E0300, E0306 */
    requirement def REQ_0164_Feature_Projection {
        id = "REQ-0164";
        text = "The projection engine of Subsystem 9 SHALL synthesize Feature specifications from the AST and enforce the 3-Layer Definition of Done (3-Layer DoD) via DoD_Validator_0.";
        attribute uuidv5 : String = "30be0357-f32d-5b49-a99b-ca931dc76215";
        attribute complexity_class : String = "Class P (O(";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / Downstream Projection Mandate";
        attribute diagnostic_codes : String = "E0199, E0300, E0306";
        attribute ac_01_full_3_layer_feature_conformance : String = "Given: Feature Feature_Alpha in Package_0 declaring valid Layer_1, Layer_2, and Layer_3. When: Lowering feature through DoD_Validator_0. Then: Confirms DoD(Feature_Alpha}) true and emits artifact. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_boundary_layer_completeness : String = "Given: Minimal feature Feature_Beta with single attribute, transition, and interface binding. When: Evaluating layer non-emptiness. Then: Confirms IsEmpty(Layer_) false for {1, 2, 3} and passes. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_empty_layer_rejection : String = "Given: Feature Feature_Gamma lacking Layer_3. When: Evaluating structural completeness. Then: Fails closed, emits E0300, and binds sentinel _{layer}. Diagnostic: E0300 (Feature Definition of Done violation).";
        attribute ac_04_unbound_placeholder_rejection_and_error_recovery : String = "Given: Feature Feature_Delta with placeholder TODO in Layer_2, delimiter ;, and valid Feature_Epsilon. When: Delimiter recovery processes Feature_Delta. Then: Emits E0306, recovers on ;, validates Feature_Epsilon, and halts at C_{max_errors}} with E0199. Diagnostic: E0306, E0199.";
        doc /* \begin{aligned} F&=\langle\text{Layer}_1(F),\text{Layer}_2(F),\text{Layer}_3(F)\rangle \\ \text{DoD}(F)&\iff\bigwedge_{\ell=1}^3(\neg\text{IsEmpty}(\text{Layer}_\ell(F))\land\neg\text{ContainsPlaceholder}(\text{Layer}_\ell(F))) \\ \exists\ell\in\{1,2,3\}\text{ s.t. }\text{IsEmpty}(\text{Layer}_\ell(F))&\implies\text{Diagnostic}(F,\text{E0300})\in\mathcal{D}_{\text{sink}}\land F\gets\bot_{\text{layer}}\land\text{Reject}(F) \\ \exists\ell\in\{1,2,3\}\text{ s.t. }\text{ContainsPlaceholder}(\text{Layer}_\ell(F))&\implies\text{Diagnostic}(F,\text{E0306})\in\mathcal{D}_{\text{sink}}\land F\gets\bot_{\text{layer}}\land\text{Reject}(F) \\ |\mathcal{D}_{\text{sink}}|\ge C_{\text{max\_errors}}&\implies\text{Diagnostic}(\text{E0199})\in\mathcal{D}_{\text{sink}}\land\text{Halt}() \\ \forall F\in\mathcal{F},&\quad\text{UUID}(F)=\text{UUIDv5}(\text{OID}_{\text{DEAP}},\text{FQN}(F)) \end{aligned} */
        constraint def Invariant_REQ_0164_FormalInvariant;
        require Invariant_REQ_0164_FormalInvariant;
        verify by AC_01_Full_3_Layer_Feature_Conformance;
        verify by AC_02_Boundary_Layer_Completeness;
        verify by AC_03_Empty_Layer_Rejection;
        verify by AC_04_Unbound_Placeholder_Rejection_and_Error_Recovery;
        satisfy by AgileProjectionEngine;
    }
    doc /* UUIDv5: de528a7b-8df8-5c7a-98c9-b6b4b21a5c7e | Complexity: Class P (O( | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 / KerML | Diagnostics: E0199, E0300, E0307 */
    requirement def REQ_0165_Feature_Layer_1 {
        id = "REQ-0165";
        text = "The projection engine of Subsystem 9 SHALL project Feature Layer 1 domain state and primitive structural typing specifications.";
        attribute uuidv5 : String = "de528a7b-8df8-5c7a-98c9-b6b4b21a5c7e";
        attribute complexity_class : String = "Class P (O(";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 / KerML";
        attribute diagnostic_codes : String = "E0199, E0300, E0307";
        attribute ac_01_canonical_domain_state_and_structural_typing : String = "Given: Package_0 with Property_1 : PrimitiveType_0 ([0, 100], default 25, unit 0, Configurable) and Property_2 : Real ([0.0, 50.0], default 10.0, unit m/s}, ReadOnly). When: Lowering Feature Layer 1. Then: Projects attributes into Layer 1 with valid types and envelopes. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_operational_range_boundary_conformance : String = "Given: Feature Feature_Alpha attribute Property_Alpha with envelope [0.0, 100.0] and default v_{default} = 100.0. When: Validating range containment. Then: Confirms v_{ v_{default} v_{ holds at boundary. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_range_envelope_and_default_value_violation_rejection : String = "Given: Feature Feature_0 attribute Property_1 with inverted range [100, 50] or default 150 [0, 100]. When: Validating interval containment. Then: Rejects attribute, binds sentinel _{attr}, emitting E0307. Diagnostic: E0307 (RangeConstraintViolation).";
        attribute ac_04_physical_dimension_mismatch_and_delimiter_recovery : String = "Given: Feature declaring Property_1 with unit vector u Q^7, delimiter ;, and valid Property_2. When: Encountering dimensional incompatibility. Then: Emits E0300, binds _{dim}, recovers across ;, parses Property_2, halting at C_{max_errors}} with E0199. Diagnostic: E0300, E0199.";
        doc /* \begin{aligned} \forall F\in\mathcal{F}_{\text{proj}},\forall a\in\mathbf{A}_F,&\quad a=\langle\text{name}_a,\tau_a,\mathbf{u}_a,[v_{\min}(a),v_{\max}(a)],v_{\text{default}}(a),\text{access}_a,\Delta_a\rangle \\ \forall a\in\mathbf{A}_F,&\quad\tau_a\in\mathcal{T}_{\text{types}}\setminus\{\bot\}\land\text{access}_a\in\{\text{ReadOnly},\text{Configurable}\}\land\mathbf{u}_a\in\mathbb{Q}^7 \\ &\quad\land v_{\min}(a)\le v_{\text{default}}(a)\le v_{\max}(a)\land v_{\text{default}}(a)\in\text{Domain}(\tau_a)\setminus\{\bot,\text{"TBD"}\} \\ (\tau_a=\bot\lor v_{\min}\gt v_{\max}&\lor v_{\text{default}}\notin[v_{\min},v_{\max}]\lor v_{\text{default}}\notin\text{Domain}(\tau_a))\implies\text{Diagnostic}(a,\text{E0307})\in\mathcal{D}_{\text{sink}}\land a\gets\bot_{\text{attr}} \\ (\mathbf{u}_a\notin\mathbb{Q}^7&\lor\text{DimMismatch}(a))\implies\text{Diagnostic}(a,\text{E0300})\in\mathcal{D}_{\text{sink}}\land\mathbf{u}_a\gets\bot_{\text{dim}} \\ |\mathcal{D}_{\text{sink}}|\ge C_{\text{max\_errors}}&\implies\text{Diagnostic}(\text{E0199})\in\mathcal{D}_{\text{sink}},\quad\Delta_a=\text{UUIDv5}(\text{OID}_{\text{DEAP}},\text{FQN}(a)) \end{aligned} */
        constraint def Invariant_REQ_0165_The_attribute_schema_typing_invariants_metric_constraints_and_diagnostic_triggers_satisfy;
        require Invariant_REQ_0165_The_attribute_schema_typing_invariants_metric_constraints_and_diagnostic_triggers_satisfy;
        verify by AC_01_Canonical_Domain_State_and_Structural_Typing;
        verify by AC_02_Operational_Range_Boundary_Conformance;
        verify by AC_03_Range_Envelope_and_Default_Value_Violation_Rejection;
        verify by AC_04_Physical_Dimension_Mismatch_and_Delimiter_Recovery;
        satisfy by AgileProjectionEngine;
    }
    doc /* UUIDv5: 868d7f06-19bf-52f1-91d4-2a1955f441f6 | Complexity: Class P (O( | Standard: IEEE 29148-2018 / RFC 2119 / SysML v2 | Diagnostics: E0199, E0300, E0308 */
    requirement def REQ_0166_Feature_Layer_2 {
        id = "REQ-0166";
        text = "Subsystem 9 SHALL project Feature Layer 2 specifications for every classifier Feature_0.";
        attribute uuidv5 : String = "868d7f06-19bf-52f1-91d4-2a1955f441f6";
        attribute complexity_class : String = "Class P (O(";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119 / SysML v2";
        attribute diagnostic_codes : String = "E0199, E0300, E0308";
        attribute ac_01_operation_signature_and_statechart_lowering : String = "Given: Feature Feature_0 with operation Operation_Alpha and machine StateMachine_0. When: Lowering Feature Layer 2 specifications. Then: Formalizes signature op and transition t. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_pseudostate_and_spontaneous_transition_boundary : String = "Given: Machine StateMachine_0 with initial state State_Init and transition t_{init} (). When: Validating initial state topology. Then: Confirms single outgoing transition from State_Init and lowers t_{init}. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_type_mismatch_and_non_boolean_guard_rejection : String = "Given: Transition t_1 with guard typing to Real, or parameter type error. When: Type checking transition guards and signatures. Then: Fails closed, emits E0308, and binds sentinel _{trans} or _{op}. Diagnostic: E0308 (TypeMismatchViolation).";
        attribute ac_04_unmapped_state_and_delimiter_recovery : String = "Given: Transition t_2 referencing State_Missing, then ; and Operation_Alpha. When: Delimiter recovery resynchronizes across statement tokens. Then: Emits E0300, binds _{trans}, lowers Operation_Alpha, and halts at C_{max_errors}} with E0199. Diagnostic: E0300, E0199.";
        attribute ac_05_uuidv5_determinism_and_release_performance_bounds : String = "Given: 10^4 elements across N_{max_workers}} threads. When: Lowering Layer 2 projections in Release mode (--release). Then: Generates identical UUIDv5 anchors; latency < 25 ms, RSS < 100 MB. Diagnostic: Diagnostic sink remains empty.";
        doc /* \begin{aligned} \forall op\in\mathcal{O}(F),&\quad\mathbf{P}_{op}=\langle(p_i,\text{dir}_i,\tau_i)\rangle_{i=1}^k\land\forall i,(\text{dir}_i\in\{\text{In},\text{Out},\text{InOut}\}\land\tau_i\in\mathcal{V}_{\text{AST}}) \\ \forall t\in\mathcal{T}(F),&\quad t=\langle s_{\text{src}}(t),e_{\text{trig}}(t),g_{\text{guard}}(t),s_{\text{tgt}}(t)\rangle\land s_{\text{src}}(t),s_{\text{tgt}}(t)\in\mathcal{S}(F) \\ \forall t\in\mathcal{T}(F),&\quad\text{Type}(g_{\text{guard}}(t))=\tau_{\text{bool}}\land\text{IsSideEffectFree}(g_{\text{guard}}(t))\equiv\text{true} \\ (s_{\text{src}}(t_a)=s_{\text{src}}(t_b)\land e_{\text{trig}}(t_a)=e_{\text{trig}}(t_b))&\implies(g_{\text{guard}}(t_a)\land g_{\text{guard}}(t_b)\equiv\text{false}) \\ (\tau_i=\bot\lor\text{Type}(g_{\text{guard}})\ne\tau_{\text{bool}})&\implies\text{Diagnostic}(\text{E0308})\in\mathcal{D}_{\text{sink}}\land op\gets\bot_{\text{op}} \\ (s_{\text{src}}\notin\mathcal{S}(F)\lor s_{\text{tgt}}\notin\mathcal{S}(F)\lor p_i=p_j)&\implies\text{Diagnostic}(\text{E0300})\in\mathcal{D}_{\text{sink}}\land t\gets\bot_{\text{trans}} \\ |\mathcal{D}_{\text{sink}}|\ge C_{\text{max\_errors}}&\implies\text{Diagnostic}(\text{E0199})\in\mathcal{D}_{\text{sink}},\quad\Delta_x=\text{UUIDv5}(\text{OID}_{\text{DEAP}},\text{FQN}(x)) \end{aligned} */
        constraint def Invariant_REQ_0166_Feature_Layer_2_projections_satisfy;
        require Invariant_REQ_0166_Feature_Layer_2_projections_satisfy;
        verify by AC_01_Operation_Signature_and_Statechart_Lowering;
        verify by AC_02_Pseudostate_and_Spontaneous_Transition_Boundary;
        verify by AC_03_Type_Mismatch_and_Non_Boolean_Guard_Rejection;
        verify by AC_04_Unmapped_State_and_Delimiter_Recovery;
        verify by AC_05_UUIDv5_Determinism_and_Release_Performance_Bounds;
        satisfy by AgileProjectionEngine;
    }
    doc /* UUIDv5: 8dbd183f-84d4-51e9-913a-96ce1cba10ad | Complexity: Class P (O( | Standard: IEEE 29148-2018 / RFC 2119 / SysML v2 | Diagnostics: E0199, E0300, E0309 */
    requirement def REQ_0167_Feature_Layer_3 {
        id = "REQ-0167";
        text = "Subsystem 9 SHALL project Feature Layer 3 physical dynamics, Kirchhoff network laws, and conservative equations for every classifier Feature_0.";
        attribute uuidv5 : String = "8dbd183f-84d4-51e9-913a-96ce1cba10ad";
        attribute complexity_class : String = "Class P (O(";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119 / SysML v2";
        attribute diagnostic_codes : String = "E0199, E0300, E0309";
        attribute ac_01_canonical_kirchhoff_network_lowering : String = "Given: Feature Feature_0 with network KirchhoffNetwork_Alpha, junctions Junction_0, Junction_1, and balanced flows Flow_1, Flow_2. When: Lowering Feature Layer 3 projections. Then: Lowers network graph into arena storage with balanced equations. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_fundamental_cycle_potential_balance_boundary : String = "Given: Loop C in KirchhoffNetwork_Alpha with three flows whose potential drops sum to zero. When: Validating cycle potential balance. Then: Confirms potential conservation across cycle C. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_non_zero_junction_divergence_rejection : String = "Given: Network KirchhoffNetwork_Alpha with junction Junction_0 having flow imbalance (f_{in} f_{out}). When: Validating Kirchhoff flow conservation. Then: Rejects junction, emits E0309, and binds sentinel _{junction}. Diagnostic: E0309 (KirchhoffFlowConservationViolation).";
        attribute ac_04_unmapped_junction_and_delimiter_recovery : String = "Given: Flow Flow_1 referencing undefined Junction_Missing, delimiter ;, and valid Flow_2. When: Recovering across statement delimiters. Then: Emits E0300, binds _{flow}, lowers Flow_2, and halts at C_{max_errors}} with E0199. Diagnostic: E0300, E0199.";
        doc /* \begin{aligned} \forall G\in\mathcal{G}(F),&\quad G=(J,E)\land\forall e\in E,e=\langle j_{\text{src}}(e),j_{\text{tgt}}(e),\mathbf{u}_e(e),\mathbf{u}_f(e),\Delta e(e),f(e)\rangle \\ \forall j\in J,&\quad\sum_{e\in\text{In}(j)}f(e)-\sum_{e\in\text{Out}(j)}f(e)=0 \\ \forall C\in\mathcal{C}(G),&\quad\sum_{e\in C}\text{sgn}(e,C)\cdot\Delta e(e)=0 \\ \forall j\in J,\forall e_1,e_2\in\text{Incident}(j),&\quad\mathbf{u}_f(e_1)=\mathbf{u}_f(e_2)\in\mathbb{Q}^7\land\mathbf{u}_e(e_1)=\mathbf{u}_e(e_2)\in\mathbb{Q}^7 \\ (j_{\text{src}}\notin J\lor j_{\text{tgt}}\notin J\lor\mathbf{u}_f(e_1)\ne\mathbf{u}_f(e_2))&\implies\text{Diagnostic}(e,\text{E0300})\in\mathcal{D}_{\text{sink}}\land e\gets\bot_{\text{flow}} \\ \left(\left|\sum_{\text{In}}f-\sum_{\text{Out}}f\right|\gt 0\lor\left|\sum_{C}\Delta e\right|\gt 0\right)&\implies\text{Diagnostic}(j,\text{E0309})\in\mathcal{D}_{\text{sink}}\land j\gets\bot_{\text{junction}} \\ |\mathcal{D}_{\text{sink}}| \ge C_{\text{max\_errors}}&\implies\text{Diagnostic}(\text{E0199})\in\mathcal{D}_{\text{sink}},\quad\Delta_x=\text{UUIDv5}(\text{OID}_{\text{DEAP}},\text{FQN}(x)) \end{aligned} */
        constraint def Invariant_REQ_0167_Feature_Layer_3_projections_satisfy;
        require Invariant_REQ_0167_Feature_Layer_3_projections_satisfy;
        verify by AC_01_Canonical_Kirchhoff_Network_Lowering;
        verify by AC_02_Fundamental_Cycle_Potential_Balance_Boundary;
        verify by AC_03_Non_Zero_Junction_Divergence_Rejection;
        verify by AC_04_Unmapped_Junction_and_Delimiter_Recovery;
        satisfy by AgileProjectionEngine;
    }
    doc /* UUIDv5: fc4a4f89-c48e-5b16-be29-9e6e0da32924 | Complexity: Class P (O( | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / Agile MBSE Projection | Diagnostics: E0199, E0300, E0310 */
    requirement def REQ_0168_User_Story_Projection {
        id = "REQ-0168";
        text = "Subsystem 9 SHALL project User Story specifications for feature Feature_0.";
        attribute uuidv5 : String = "fc4a4f89-c48e-5b16-be29-9e6e0da32924";
        attribute complexity_class : String = "Class P (O(";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / Agile MBSE Projection";
        attribute diagnostic_codes : String = "E0199, E0300, E0310";
        attribute ac_01_canonical_user_story_lowering : String = "Given: Feature Feature_0 with UserStory_Alpha (Persona_0, Action_0, value 100.0, MustHave). When: Lowering User Story specifications. Then: Formalizes narrative, assigns MoSCoW rank 1, and passes. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_deterministic_moscow_ordering : String = "Given: Stories across MustHave, ShouldHave, CouldHave, WontHave in arbitrary order. When: Sorting catalog sequence _{proj}. Then: Orders stories by MoSCoW rank with value tie-breaking. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_invalid_priority_rejection : String = "Given: Story UserStory_Beta with invalid priority MaybeHave or negative metric -5.0. When: Validating priority and business value. Then: Fails closed, binds sentinel _{story}, emitting E0310. Diagnostic: E0310 (PriorityClassificationViolation).";
        attribute ac_04_missing_persona_and_delimiter_recovery : String = "Given: Story missing persona, delimiter ;, and valid UserStory_Beta. When: Delimiter recovery resynchronizes across tokens. Then: Emits E0300, binds _{story}, parses UserStory_Beta, halting at C_{max_errors}} with E0199. Diagnostic: E0300, E0199.";
        doc /* \begin{aligned} \forall u\in\mathcal{U}(F),& u=\langle\text{id}_u,p_u,i_u,b_u,\rho_u,\Delta_u\rangle\land p_u\in\mathcal{A}_{\text{actors}}\land i_u\in\mathcal{C}_{\text{actions}}(F) \\ \forall u\in\mathcal{U}(F),& \text{metric}(b_u)\ge 0\land\text{rationale}(b_u)\ne\epsilon\land\rho_u\in\mathcal{P}_{\text{MoSCoW}} \\ \mathcal{P}_{\text{MoSCoW}}&=\{\text{MustHave},\text{ShouldHave},\text{CouldHave},\text{WontHave}\} \\ (p_u\notin\mathcal{A}_{\text{actors}}\lor i_u\notin\mathcal{C}_{\text{actions}}(F))&\implies\text{Diagnostic}(u,\text{E0300})\in\mathcal{D}_{\text{sink}}\land u\gets\bot_{\text{story}} \\ (\rho_u\notin\mathcal{P}_{\text{MoSCoW}}\lor\text{metric}(b_u)\lt 0)&\implies\text{Diagnostic}(u,\text{E0310})\in\mathcal{D}_{\text{sink}}\land u\gets\bot_{\text{story}} \\ |\mathcal{D}_{\text{sink}}|\ge C_{\text{max\_errors}}&\implies\text{Diagnostic}(\text{E0199})\in\mathcal{D}_{\text{sink}}\land\text{HaltPipeline}() \\ \forall u\in\mathcal{U}(F),& \Delta_u=\text{UUIDv5}(\text{OID}_{\text{DEAP}},\text{FQN}(F)\mathbin{\Vert}\text{"::UserStory::"}\mathbin{\Vert}\text{name}_u) \end{aligned} */
        constraint def Invariant_REQ_0168_FormalInvariant;
        require Invariant_REQ_0168_FormalInvariant;
        verify by AC_01_Canonical_User_Story_Lowering;
        verify by AC_02_Deterministic_MoSCoW_Ordering;
        verify by AC_03_Invalid_Priority_Rejection;
        verify by AC_04_Missing_Persona_and_Delimiter_Recovery;
        satisfy by AgileProjectionEngine;
    }
    doc /* UUIDv5: d4062e22-8bf8-5089-a292-0b299e469502 | Complexity: Class P (O( | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 / KerML | Diagnostics: E0199, E0300, E0311 */
    requirement def REQ_0169_Use_Case_Projection {
        id = "REQ-0169";
        text = "The downstream projection engine of Subsystem 9 SHALL project formal Use Case specifications from the compilation unit abstract syntax tree.";
        attribute uuidv5 : String = "d4062e22-8bf8-5089-a292-0b299e469502";
        attribute complexity_class : String = "Class P (O(";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 / KerML";
        attribute diagnostic_codes : String = "E0199, E0300, E0311";
        attribute ac_01_canonical_use_case_and_primary_success_path_projection : String = "Given: Package_0 declaring UseCase_Alpha with actor Actor_0, precondition Precondition_1, primary scenario Scenario_0 with steps Step_0, Step_1 , and postcondition Postcondition_1. When: Projecting use case specifications. Then: Projects UseCase_Alpha with verified step order and actor associations. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_alternate_extension_branching_and_resumption_boundary : String = "Given: Use case UseCase_Alpha declaring alternate extension Extension_0 branching from Step_0 under guard g_{alt} and resuming at Step_1. When: Validating scenario branching topology and step anchors. Then: Confirms anchor Step_0 and resumption step Step_1 reside in Scenario_0. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_dangling_extension_anchor_and_flow_rejection : String = "Given: UseCase_Alpha where Extension_0 specifies unresolvable anchor Step_Missing or null precondition P_{pre} = . When: Validating step connectivity and branch invariants. Then: Rejects flow, binds sentinel _{uc}, and emits diagnostic E0311. Diagnostic: E0311 (AlternateExtensionViolation).";
        attribute ac_04_unresolved_actor_reference_and_delimiter_recovery : String = "Given: Use case referencing undeclared actor Actor_Missing, followed by delimiter ; and valid UseCase_Beta. When: Lowering use case actors with delimiter recovery enabled. Then: Emits E0300, binds _{uc}, recovers across ;, projects UseCase_Beta, and halts at C_{max_errors}} with E0199. Diagnostic: E0300, E0199.";
        doc /* \begin{aligned} \forall UC\in\mathcal{U}_{\text{proj}},&\quad UC=\langle\text{name}_{UC},\mathbf{A}_{UC},P_{\text{pre}},S_{\text{primary}},\mathcal{X}_{\text{alt}},Q_{\text{post}},\Delta_{UC}\rangle \\ \forall s\in S_{\text{primary}},&\quad\text{actor}(s)\in\mathbf{A}_{UC}\land\text{action}(s)\in\mathcal{V}_{\text{AST}}\land\text{post}(s)\in\text{Expr}_{\text{bool}} \\ \forall x\in\mathcal{X}_{\text{alt}},&\quad x=\langle s_{\text{anchor}}(x),g_{\text{alt}}(x),S_{\text{alt}}(x),s_{\text{resume}}(x),Q_{\text{alt}}(x)\rangle \\ &\quad\land s_{\text{anchor}}(x)\in S_{\text{primary}}\land(s_{\text{resume}}(x)\in S_{\text{primary}}\lor s_{\text{resume}}(x)=\bot_{\text{term}}) \\ (P_{\text{pre}}=\bot\lor\exists x,s_{\text{anchor}}(x)\notin S_{\text{primary}})&\implies\text{Diagnostic}(UC,\text{E0311})\in\mathcal{D}_{\text{sink}}\land UC\gets\bot_{\text{uc}} \\ \exists a\in\mathbf{A}_{UC}\text{ s.t. }a\notin\mathcal{V}_{\text{AST}}&\implies\text{Diagnostic}(UC,\text{E0300})\in\mathcal{D}_{\text{sink}}\land UC\gets\bot_{\text{uc}} \\ |\mathcal{D}_{\text{sink}}|\ge C_{\text{max\_errors}}&\implies\text{Diagnostic}(\text{E0199})\in\mathcal{D}_{\text{sink}},\quad\Delta_{UC}=\text{UUIDv5}(\text{OID}_{\text{DEAP}},\text{FQN}(UC)) \end{aligned} */
        constraint def Invariant_REQ_0169_Projected_use_case_specifications_scenario_step_sequences_and_alternate_branches_satisfy;
        require Invariant_REQ_0169_Projected_use_case_specifications_scenario_step_sequences_and_alternate_branches_satisfy;
        verify by AC_01_Canonical_Use_Case_and_Primary_Success_Path_Projection;
        verify by AC_02_Alternate_Extension_Branching_and_Resumption_Boundary;
        verify by AC_03_Dangling_Extension_Anchor_and_Flow_Rejection;
        verify by AC_04_Unresolved_Actor_Reference_and_Delimiter_Recovery;
        satisfy by AgileProjectionEngine;
    }
    doc /* UUIDv5: 53086ebc-88e9-53e3-8f0a-7cb244837862 | Complexity: Class P (O( | Standard: IEEE 29148-2018 / RFC 2119 / MIL-STD-881E | Diagnostics: E0199, E0300, E0312 */
    requirement def REQ_0170_Engineering_Tasks_Projection {
        id = "REQ-0170";
        text = "Subsystem 9 SHALL project Engineering Tasks from AST definitions into a Work Breakdown Structure (WBS) rooted at WBS_0.";
        attribute uuidv5 : String = "53086ebc-88e9-53e3-8f0a-7cb244837862";
        attribute complexity_class : String = "Class P (O(";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119 / MIL-STD-881E";
        attribute diagnostic_codes : String = "E0199, E0300, E0312";
        attribute ac_01_canonical_wbs_task_hierarchy_and_effort_projection : String = "Given: Feature Feature_0 task Task_0 containing leaves Task_Alpha (h = 8.0) and Task_Beta (h = 16.0) with DoD_Criteria_0. When: Lowering tasks and synthesizing WBS hierarchy. Then: Projects WBS_0 with effort h_{Task_0}} = 24.0. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_boundary_leaf_task_allocation_and_minimum_effort : String = "Given: Leaf Task_Alpha with minimum effort h = 0.5 and criterion DoD_Criteria_0. When: Validating effort boundaries and DoD completeness. Then: Confirms h 0, validates DoD satisfaction, and lowers task. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_definition_of_done_violation_and_placeholder_rejection : String = "Given: Task Task_Alpha with empty DoD or placeholder \"TODO\". When: Validating Definition of Done completeness. Then: Fails closed, emits E0312, and binds sentinel _{task}. Diagnostic: E0312 (TaskDefinitionOfDoneViolation).";
        attribute ac_04_hierarchy_disconnect_and_delimiter_recovery : String = "Given: Composite Task_0 with effort mismatch (h = 30.0 24.0), child referencing Task_Missing, delimiter ;, and valid Task_Beta. When: Recovering across delimiters on invalid tasks. Then: Emits E0300 and E0312, binds sentinels _{task}, recovers across ;, parses Task_Beta, halting at C_{max_errors}} with E0199. Diagnostic: E0300, E0312, E0199.";
        doc /* \begin{aligned} \forall t\in\mathcal{T}_{\text{tasks}},&\quad t=\langle\text{id}_t,\text{name}_t,\text{parent}_t,\text{wbs}_t,\mathbf{C}_{\text{DoD}}(t),h_t,\Delta_t\rangle \\ \forall t\in\mathcal{T}_{\text{tasks}},&\quad\text{parent}_t\in(\mathcal{T}_{\text{tasks}}\cup\mathcal{F}_{\text{proj}})\land\mathbf{C}_{\text{DoD}}(t)\ne\emptyset\land h_t\gt 0 \\ \forall t\in\mathcal{T}_{\text{tasks}},&\quad\text{Children}(t)\ne\emptyset\implies h_t=\sum_{c\in\text{Children}(t)} h_c \\ \forall c\in\mathbf{C}_{\text{DoD}}(t),&\quad\neg\text{ContainsPlaceholder}(c) \\ (\text{parent}_t\notin(\mathcal{T}_{\text{tasks}}\cup\mathcal{F}_{\text{proj}})\lor\text{IsCycle}(t))&\implies\text{Diagnostic}(t,\text{E0300})\in\mathcal{D}_{\text{sink}}\land t\gets\bot_{\text{task}} \\ (\mathbf{C}_{\text{DoD}}(t)=\emptyset\lor\text{ContainsPlaceholder}(t))&\implies\text{Diagnostic}(t,\text{E0312})\in\mathcal{D}_{\text{sink}}\land t\gets\bot_{\text{task}} \\ (h_t\le 0\lor(\text{Children}(t)\ne\emptyset\land h_t\ne\sum_c h_c))&\implies\text{Diagnostic}(t,\text{E0312})\in\mathcal{D}_{\text{sink}}\land t\gets\bot_{\text{task}} \\ |\mathcal{D}_{\text{sink}}| \ge C_{\text{max\_errors}}&\implies\text{Diagnostic}(\text{E0199})\in\mathcal{D}_{\text{sink}},\quad\Delta_t=\text{UUIDv5}(\text{OID}_{\text{DEAP}},\text{FQN}(t)) \end{aligned} */
        constraint def Invariant_REQ_0170_Engineering_task_projections_satisfy;
        require Invariant_REQ_0170_Engineering_task_projections_satisfy;
        verify by AC_01_Canonical_WBS_Task_Hierarchy_and_Effort_Projection;
        verify by AC_02_Boundary_Leaf_Task_Allocation_and_Minimum_Effort;
        verify by AC_03_Definition_of_Done_Violation_and_Placeholder_Rejection;
        verify by AC_04_Hierarchy_Disconnect_and_Delimiter_Recovery;
        satisfy by AgileProjectionEngine;
    }
    doc /* UUIDv5: 2e7a3f47-ec5a-52f1-aa95-fc71295b9d79 | Complexity: Class P (O( | Standard: IEEE 29148-2018 / RFC 2119 / SysML v2 | Diagnostics: E0199, E0300, E0313 */
    requirement def REQ_0171_Downstream_Synchronization_Engine {
        id = "REQ-0171";
        text = "Subsystem 9 SHALL perform bidirectional incremental AST patching and synchronization via SyncEngine_Alpha across upstream AST models and downstream projections against base AST T_{base}.";
        attribute uuidv5 : String = "2e7a3f47-ec5a-52f1-aa95-fc71295b9d79";
        attribute complexity_class : String = "Class P (O(";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119 / SysML v2";
        attribute diagnostic_codes : String = "E0199, E0300, E0313";
        attribute ac_01__3_way_ast_diff_and_non_conflicting_patch_application : String = "Given: Base T_{base} with upstream addition of Classifier_Alpha in Package_0 and downstream update of param_x : Real on Port_1. When: SyncEngine_Alpha computes diff ASTDiff_0 and applies patch Patch_1. Then: Reconciles edits into T_{merged} preserving all modifications. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_idempotent_patch_re_application_boundary : String = "Given: Synchronized tree T_{merged} with base T_{base} = T_{merged}. When: SyncEngine_Alpha computes diff across identical trees. Then: Generates empty diff = with 0-byte state churn. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_concurrent_conflicting_modification_rejection : String = "Given: Base node Classifier_Alpha where upstream sets param_x = 10.0 and downstream sets param_x = 20.0. When: SyncEngine_Alpha evaluates 3-way merge rules. Then: Fails closed, emits E0313, and binds sentinel _{patch}. Diagnostic: E0313 (SynchronizationConflictViolation).";
        attribute ac_04_structural_cycle_violation_and_delimiter_recovery : String = "Given: Patch with cycle in Package_0, delimiter ;, and valid Port_1. When: Structural validation detects cycle and recovers across ;. Then: Emits E0300, binds sentinel _{patch}, resumes on Port_1, halting at C_{max_errors}} with E0199. Diagnostic: E0300, E0199.";
        doc /* \begin{aligned} \Delta_{\text{up}}=\text{Diff}(T_{\text{base}},T_{\text{upstream}}),&\quad\Delta_{\text{down}}=\text{Diff}(T_{\text{base}},T_{\text{downstream}}) \\ \text{Conflict}(v)&\iff(v\in\text{Mod}(\Delta_{\text{up}})\cap\text{Mod}(\Delta_{\text{down}}))\land(\text{Val}_{\text{up}}(v)\ne\text{Val}_{\text{down}}(v)) \\ \text{Conflict}(v)&\implies\text{Diagnostic}(v,\text{E0313})\in\mathcal{D}_{\text{sink}}\land\text{Patch}_1(v)\gets\bot_{\text{patch}} \\ (v\in\text{Mod}(\Delta_{\text{up}})\land\neg\text{Conflict}(v))&\implies\text{Val}_{\text{merged}}(v)=\text{Val}_{\text{up}}(v) \\ (v\in\text{Mod}(\Delta_{\text{down}})\land\neg\text{Conflict}(v))&\implies\text{Val}_{\text{merged}}(v)=\text{Val}_{\text{down}}(v) \\ (\text{IsCycle}(T_{\text{merged}})\lor\text{Orphan}(v))&\implies\text{Diagnostic}(v,\text{E0300})\in\mathcal{D}_{\text{sink}}\land T_{\text{merged}}\gets\bot_{\text{patch}} \\ |\mathcal{D}_{\text{sink}}|\ge C_{\text{max\_errors}}&\implies\text{Diagnostic}(\text{E0199})\in\mathcal{D}_{\text{sink}},\quad\Delta_v=\text{UUIDv5}(\text{OID}_{\text{DEAP}},\text{FQN}(v)) \end{aligned} */
        constraint def Invariant_REQ_0171_The_3_way_synchronization_diffing_operations_conflict_predicates_and_diagnostic_mappings_satisfy;
        require Invariant_REQ_0171_The_3_way_synchronization_diffing_operations_conflict_predicates_and_diagnostic_mappings_satisfy;
        verify by AC_01__3_Way_AST_Diff_and_Non_Conflicting_Patch_Application;
        verify by AC_02_Idempotent_Patch_Re_Application_Boundary;
        verify by AC_03_Concurrent_Conflicting_Modification_Rejection;
        verify by AC_04_Structural_Cycle_Violation_and_Delimiter_Recovery;
        satisfy by AgileProjectionEngine;
    }
    doc /* UUIDv5: bf107d67-ba84-5f40-8b1b-fb07469a4c51 | Complexity: Class P (O( | Standard: IEEE 29148-2018 / RFC 2119 / SysML v2 | Diagnostics: E0199, E0300, E0314 */
    requirement def REQ_0172_Bidirectional_Traceability_Link_Invalidation {
        id = "REQ-0172";
        text = "Subsystem 9 SHALL maintain bidirectional traceability across AST entities and projected artifacts, executing cascade Cascade_1 over TraceGraph_Alpha (G_{trace} = (V, E_{trace})) upon AST mutation.";
        attribute uuidv5 : String = "bf107d67-ba84-5f40-8b1b-fb07469a4c51";
        attribute complexity_class : String = "Class P (O(";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119 / SysML v2";
        attribute diagnostic_codes : String = "E0199, E0300, E0314";
        attribute ac_01_canonical_forward_and_backward_dirty_cascade : String = "Given: Graph TraceGraph_Alpha with Classifier_Alpha (DirtyNode_0) linked to Artifact_Alpha. When: Mutating DirtyNode_0 and triggering Cascade_1. Then: Traverses edges via BFS, marking Classifier_Alpha and Artifact_Alpha dirty. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_multi_parent_convergence_and_idempotent_traversal : String = "Given: Artifact Artifact_Gamma with edges from mutated DirtyNode_0 and clean Classifier_Beta. When: Executing cascade invalidation across multiple passes. Then: Marks Artifact_Gamma dirty on initial visit, skipping redundant passes with zero diff churn. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_asymmetric_traceability_link_rejection : String = "Given: Forward edge (u, v) E_{fwd} lacking reciprocal edge (v, u) E_{bwd}. When: Verifying link symmetry during cascade initialization. Then: Rejects asymmetric link, binds sentinel _{link}, and emits E0314. Diagnostic: E0314 (TraceabilityLinkInvalidationViolation).";
        attribute ac_04_dangling_endpoint_and_delimiter_recovery : String = "Given: Malformed link referencing Node_Missing, delimiter ;, and valid Link_0. When: Delimiter error recovery processes tokens. Then: Emits E0300, recovers across ;, parses Link_0, halting at C_{max_errors}} with E0199. Diagnostic: E0300, E0199.";
        doc /* \begin{aligned} \mathcal{G}_{\text{trace}}&=(V,E_{\text{trace}}),\quad V=V_{\text{AST}}\cup V_{\text{proj}},\quad E_{\text{trace}}=E_{\text{fwd}}\cup E_{\text{bwd}} \\ \forall(u,v)&\in E_{\text{fwd}},\quad(v,u)\in E_{\text{bwd}} \\ \forall v&\in V,\quad(\exists u\in V_{\text{dirty}}\land(u,v)\in E_{\text{trace}}^*)\implies\text{Dirty}(v)=1 \\ (u\notin V\lor v\notin V)&\implies\text{Diagnostic}((u,v),\text{E0300})\in\mathcal{D}_{\text{sink}}\land(u,v)\gets\bot_{\text{trace}} \\ ((u,v)\in E_{\text{fwd}}\land(v,u)\notin E_{\text{bwd}})&\implies\text{Diagnostic}((u,v),\text{E0314})\in\mathcal{D}_{\text{sink}}\land v\gets\bot_{\text{link}} \\ |\mathcal{D}_{\text{sink}}|\ge C_{\text{max\_errors}}&\implies\text{Diagnostic}(\text{E0199})\in\mathcal{D}_{\text{sink}},\quad\Delta_v=\text{UUIDv5}(\text{OID}_{\text{DEAP}},\text{FQN}(v)) \end{aligned} */
        constraint def Invariant_REQ_0172_Traceability_topology_reachability_cascades_and_diagnostic_bindings_satisfy;
        require Invariant_REQ_0172_Traceability_topology_reachability_cascades_and_diagnostic_bindings_satisfy;
        verify by AC_01_Canonical_Forward_and_Backward_Dirty_Cascade;
        verify by AC_02_Multi_Parent_Convergence_and_Idempotent_Traversal;
        verify by AC_03_Asymmetric_Traceability_Link_Rejection;
        verify by AC_04_Dangling_Endpoint_and_Delimiter_Recovery;
        satisfy by AgileProjectionEngine;
    }
    doc /* UUIDv5: 5610bc93-aa9a-5813-88b1-38379aee5188 | Complexity: Class P (O( | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / Downstream Projection Mandate | Diagnostics: E0199, E0300, E0315 */
    requirement def REQ_0173_Automated_Conflict_Resolution_Policies_for_Upstream_Downstream_Divergence {
        id = "REQ-0173";
        text = "The projection engine of Subsystem 9 SHALL reconcile divergence between upstream AST models and downstream projections via ConflictResolver_Alpha.";
        attribute uuidv5 : String = "5610bc93-aa9a-5813-88b1-38379aee5188";
        attribute complexity_class : String = "Class P (O(";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / Downstream Projection Mandate";
        attribute diagnostic_codes : String = "E0199, E0300, E0315";
        attribute ac_01_disjoint_non_conflicting_3_way_divergence_resolution : String = "Given: Base B in Package_0, edit U on Element_0, and edit D on disjoint Element_1 under Policy_1. When: Lowering through ConflictResolver_Alpha. Then: Reconciles mutations into M_{resolved}, preserving both edits. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_convergent_concurrent_mutation_boundary : String = "Given: Base B attribute param_x : Real on Element_0, with identical edits in U and D. When: Evaluating convergent divergence under Policy_1. Then: Confirms U(e) = D(e) and merges mutation into M_{resolved}. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_contradictory_divergence_rejection_and_sentinel_binding : String = "Given: Conflicting edits to param_x on Element_0 across U and D under Policy_Alpha. When: Resolving conflicting divergence. Then: Fails closed, emits diagnostic E0315, and binds sentinel _{resolve}. Diagnostic: E0315 (DivergenceConflictViolation).";
        attribute ac_04_malformed_schema_record_and_delimiter_recovery : String = "Given: Record Divergence_0 violating typing, delimiter ;, and valid Divergence_1. When: Executing delimiter recovery across ;. Then: Emits E0300, binds _{resolve}, resynchronizes across ;, parses Divergence_1, halting at C_{max_errors}} with E0199. Diagnostic: E0300, E0199.";
        attribute ac_05_deterministic_uuidv5_anchors_and_release_performance_bounds : String = "Given: Model with |C_{diff}| = 10^4 diff elements evaluated across N_{max_workers}} threads. When: Benchmarking in Release mode (--release). Then: Emits identical UUIDv5 anchors; latency < 25 ms, peak RSS < 100 MB. Diagnostic: Diagnostic sink remains empty.";
        doc /* \begin{aligned} C_{\text{diff}}&=\{e\in\text{Dom}(B)\cup\text{Dom}(U)\cup\text{Dom}(D)\mid U(e)\ne B(e)\lor D(e)\ne B(e)\} \\ \forall e\in C_{\text{diff}},&\quad\Delta_U(e)=(U(e)\ne B(e)),\quad\Delta_D(e)=(D(e)\ne B(e)) \\ M_{\text{resolved}}(e)&=\begin{cases} U(e) & \text{if }\Delta_U(e)\land\neg\Delta_D(e) \\ D(e) & \text{if }\neg\Delta_U(e)\land\Delta_D(e) \\ U(e) & \text{if }\Delta_U(e)\land\Delta_D(e)\land U(e)=D(e) \\ \bot_{\text{resolve}} & \text{if }\Delta_U(e)\land\Delta_D(e)\land U(e)\ne D(e) \end{cases} \\ (\Delta_U(e)\land\Delta_D(e)\land U(e)\ne D(e))&\implies\text{Diagnostic}(e,\text{E0315})\in\mathcal{D}_{\text{sink}}\land M_{\text{resolved}}(e)\gets\bot_{\text{resolve}} \\ \text{IsMalformed}(e)&\implies\text{Diagnostic}(e,\text{E0300})\in\mathcal{D}_{\text{sink}}\land M_{\text{resolved}}(e)\gets\bot_{\text{resolve}} \\ |\mathcal{D}_{\text{sink}}|\ge C_{\text{max\_errors}}&\implies\text{Diagnostic}(\text{E0199})\in\mathcal{D}_{\text{sink}},\quad\Delta_e=\text{UUIDv5}(\text{OID}_{\text{DEAP}},\text{FQN}(e)) \end{aligned} */
        constraint def Invariant_REQ_0173_Candidate_diff_set_resolved_model_and_diagnostic_triggers_satisfy;
        require Invariant_REQ_0173_Candidate_diff_set_resolved_model_and_diagnostic_triggers_satisfy;
        verify by AC_01_Disjoint_Non_Conflicting_3_Way_Divergence_Resolution;
        verify by AC_02_Convergent_Concurrent_Mutation_Boundary;
        verify by AC_03_Contradictory_Divergence_Rejection_and_Sentinel_Binding;
        verify by AC_04_Malformed_Schema_Record_and_Delimiter_Recovery;
        verify by AC_05_Deterministic_UUIDv5_Anchors_and_Release_Performance_Bounds;
        satisfy by AgileProjectionEngine;
    }
    doc /* UUIDv5: a375490a-a035-519d-a417-212d22faeb53 | Complexity: Class P (O( | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / Cryptographic Manifest Ledger | Diagnostics: E0199, E0300, E0316 */
    requirement def REQ_0174_Specification_Manifest_Generation {
        id = "REQ-0174";
        text = "The downstream projection engine of Subsystem 9 SHALL synthesize a specification manifest (Manifest_Alpha) and verification ledger (HashLedger_0) covering all emitted artifacts (A).";
        attribute uuidv5 : String = "a375490a-a035-519d-a417-212d22faeb53";
        attribute complexity_class : String = "Class P (O(";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / Cryptographic Manifest Ledger";
        attribute diagnostic_codes : String = "E0199, E0300, E0316";
        attribute ac_01_canonical_manifest_and_cryptographic_ledger_generation : String = "Given: Model with specification artifacts (Artifact_Alpha) in Package_0. When: Lowering Manifest_Alpha and HashLedger_0. Then: Emits relative paths, sizes, digests (Digest_1), root hash H_{root}, and UUIDv5 anchors. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_boundary_artifact_size_and_large_manifest_scaling : String = "Given: Artifact Artifact_0 of 16 MB within 10^4 artifacts. When: Computing digests and aggregating ledger entries. Then: Processes artifacts within O(|A| size) bounds without memory exhaustion. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_tamper_detection_and_hash_mismatch_rejection : String = "Given: Ledger HashLedger_0 where Artifact_Alpha is modified by 1 byte. When: Validating manifest integrity. Then: Fails closed, emits E0316 (HashVerificationMismatch), and binds sentinel _{ledger}. Diagnostic: E0316.";
        attribute ac_04_malformed_relative_path_and_delimiter_recovery : String = "Given: Artifact with absolute path \"/opt/spec.md\", delimiter |, and Artifact_Beta. When: Recovering across delimiter boundaries. Then: Emits E0300, recovers on |, parses Artifact_Beta, halting at C_{max_errors}} with E0199. Diagnostic: E0300, E0199.";
        doc /* \begin{aligned} \forall a\in\mathcal{A},\quad\text{Entry}(a)&=\langle\mathcal{P}_{\text{rel}}(a),\tau_a,S_a,h_a,\Delta_a\rangle \\ h_a=\text{Digest}(\text{Bytes}(a))\in\{0,1\}^{256},&\quad S_a=|\text{Bytes}(a)|\in\mathbb{N}^+ \\ \mathcal{P}_{\text{rel}}(a_1)\prec_{\text{lex}}\dots\prec_{\text{lex}}\mathcal{P}_{\text{rel}}(a_N),&\quad H_{\text{root}}=\text{Digest}\left(\bigoplus_{i=1}^N(\mathcal{P}_{\text{rel}}(a_i)\mathbin{\Vert}\text{Hex}(h_{a_i}))\right) \\ \Delta_a&=\text{UUIDv5}(\text{OID}_{\text{DEAP}},\mathcal{P}_{\text{rel}}(a)\mathbin{\Vert}\text{"::Manifest"}) \\ \neg\text{IsValidRelPath}(\mathcal{P}_{\text{rel}}(a))&\implies\text{Diagnostic}(a,\text{E0300})\in\mathcal{D}_{\text{sink}}\land\text{Entry}(a)\gets\bot_{\text{path}} \\ h_a\ne h_a^{\text{actual}}&\implies\text{Diagnostic}(a,\text{E0316})\in\mathcal{D}_{\text{sink}}\land\text{Ledger}\gets\bot_{\text{ledger}} \\ |\mathcal{D}_{\text{sink}}|\ge C_{\text{max\_errors}}&\implies\text{Diagnostic}(\text{E0199})\in\mathcal{D}_{\text{sink}} \end{aligned} */
        constraint def Invariant_REQ_0174_FormalInvariant;
        require Invariant_REQ_0174_FormalInvariant;
        verify by AC_01_Canonical_Manifest_and_Cryptographic_Ledger_Generation;
        verify by AC_02_Boundary_Artifact_Size_and_Large_Manifest_Scaling;
        verify by AC_03_Tamper_Detection_and_Hash_Mismatch_Rejection;
        verify by AC_04_Malformed_Relative_Path_and_Delimiter_Recovery;
        satisfy by AgileProjectionEngine;
    }
}
