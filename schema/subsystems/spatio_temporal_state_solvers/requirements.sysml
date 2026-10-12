doc /* Spatio-Temporal Dynamics & Discrete State Machine Solvers */
package SpatioTemporalStateSolvers {
    doc /* UUIDv5: 3543c7aa-6407-54f2-a3e2-c415220715bb | Complexity: Class P (O(1) check, O(N) lowering) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 | Diagnostics: E0199, E0400, E0401 */
    requirement def REQ_0114_Spatio_Temporal_Occurrence_Lifecycles {
        id = "REQ-0114";
        text = "The dynamics solver SHALL assign interval bounds [t_{start}, t_{end}] \subset \mathbb{R} \cup \{-\infty, +\infty\} to lifecycles, actions, and states referenced to epoch t_{epoch} = 0.";
        attribute uuidv5 : String = "3543c7aa-6407-54f2-a3e2-c415220715bb";
        attribute complexity_class : String = "Class P (O(1) check, O(N) lowering)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119";
        attribute diagnostic_codes : String = "E0199, E0400, E0401";
        attribute ac_01_valid_occurrence_lifecycle_and_interval_lowering : String = "Given: Occurrence_Alpha ([0.0, 10.0]) and event Event_0 ([5.0, 5.0]) in Package_0. When: Lowering assigns interval bounds. Then: Assigns durations 10.0 0 and 0.0; diagnostic sink remains empty. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_inverted_bounds_and_negative_epoch_rejection : String = "Given: Occurrence_1 ([15.0, 10.0]) and Occurrence_2 ([-5.0, 10.0]). When: Lowering checks interval bounds. Then: Rejects coordinates, binding _{interval} and emitting E0400. Diagnostic: E0400 (TemporalIntervalInverted / NegativeEpochTime).";
        attribute ac_03_duration_parameter_consistency_and_envelope_breach : String = "Given: Action_Alpha ([0.0, 10.0], _{nom} = 12.0) and Action_Beta ([0.0, 8.0], envelope [1.0, 5.0]). When: Verifying duration consistency. Then: Nominal error and envelope breach emit E0400. Diagnostic: E0400 (DurationBoundsInconsistency / DurationEnvelopeBreach).";
        attribute ac_04_hierarchical_containment_breach_and_state_overlap : String = "Given: Parent [10.0, 50.0] with child [5.0, 60.0], and exclusive states overlapping on [10.0, 12.0]. When: Containment and state continuity checks execute. Then: Breaches emit E0401, binding sentinel _{occ}. Diagnostic: E0401 (TemporalContainmentBreach / StateOccupancyOverlap).";
        doc /* \begin{aligned} I(o)=[t_{\text{start}}(o),t_{\text{end}}(o)],\quad 0\le t_{\text{start}}(o)\le t_{\text{end}}(o),\quad \text{duration}(I(o))=t_{\text{end}}(o)-t_{\text{start}}(o)\ge 0 \\ (o\in\mathcal{O}_{\text{event}}\implies t_{\text{start}}=t_{\text{end}})\land (o\in\mathcal{O}_{\text{durative}}\implies t_{\text{start}}\lt t_{\text{end}}) \\ \delta_{\min}\le\text{duration}(I(o))\le\delta_{\max},\quad |\text{duration}(I(o))-\delta_{\text{nom}}|\le\epsilon_{\text{tol}} \\ o_{\text{sub}}\subseteq_{\text{occ}} o_{\text{super}}\implies (t_{\text{start}}(o_{\text{super}})\le t_{\text{start}}(o_{\text{sub}})\le t_{\text{end}}(o_{\text{sub}})\le t_{\text{end}}(o_{\text{super}})) \\ S_i\xrightarrow{\tau} S_{i+1}\implies t_{\text{end}}(S_i)=t(\tau)=t_{\text{start}}(S_{i+1}),\quad I(S_i)\cap I(S_j)\subseteq\{t_{\text{boundary}}\} \\ (t_{\text{start}}\gt t_{\text{end}}\lor t_{\text{start}}\lt 0\lor \Delta t\lt 0)\implies\text{Diagnostic}(o,\text{E0400})\land I(o)\gets\bot_{\text{interval}} \\ (o_{\text{sub}}\not\subseteq_{\text{time}} o_{\text{super}}\lor I(S_i)\cap I(S_j)\not\subseteq\{t_{\text{boundary}}\})\implies\text{Diagnostic}(o,\text{E0401})\land o_{\text{sub}}\gets\bot_{\text{occ}} \\ \forall o\in\mathcal{O}_{\text{life}},\quad \Delta_o=\text{UUIDv5}(\text{NS}_{\text{DEAP}},\text{FQN}(o)) \end{aligned} */
        constraint def Invariant_REQ_0114_Occurrence_lifecycle_intervals_duration_consistency_hierarchical_containment_and_error_mappings_satisfy;
        require Invariant_REQ_0114_Occurrence_lifecycle_intervals_duration_consistency_hierarchical_containment_and_error_mappings_satisfy;
        verify by AC_01_Valid_Occurrence_Lifecycle_and_Interval_Lowering;
        verify by AC_02_Inverted_Bounds_and_Negative_Epoch_Rejection;
        verify by AC_03_Duration_Parameter_Consistency_and_Envelope_Breach;
        verify by AC_04_Hierarchical_Containment_Breach_and_State_Overlap;
        satisfy by StateMachineSolverEngine;
    }
    doc /* UUIDv5: 641d5da6-0911-5d4f-a90f-dd1f1f93d23e | Complexity: Class P (O(1) per relation check, O(N^2) pairwise) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 | Diagnostics: E0199, E0400, E0401 */
    requirement def REQ_0115_Allen_s_13_Qualitative_Interval_Relations_Axiomatic_Algebraic_Solver {
        id = "REQ-0115";
        text = "The dynamics solver SHALL implement an axiomatic solver for Allen's 13 qualitative interval relations over occurrences, actions, and states.";
        attribute uuidv5 : String = "641d5da6-0911-5d4f-a90f-dd1f1f93d23e";
        attribute complexity_class : String = "Class P (O(1) per relation check, O(N^2) pairwise)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119";
        attribute diagnostic_codes : String = "E0199, E0400, E0401";
        attribute ac_01_atomic_relation_classification : String = "Given: Interval_Alpha = [10.0, 20.0], Interval_Beta = [20.0, 30.0]. When: Evaluating endpoint ordering (x_e = y_s). Then: Classifies atomic meets (m), lowering difference inequalities. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_bitmask_algebraic_inversion_and_conjunction : String = "Given: Relations R_1 = { b, m }, R_2 = { m, o } on (Interval_X, Interval_Y). When: Evaluating bitmask operations. Then: Computes R_1 R_2 = { m } (0x0004), (R_1)^ = { bi, mi } (0x000A). Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_inverted_interval_and_relation_contradiction : String = "Given: Interval_X with x_s = 25.0 x_e = 15.0, and assertion equals on [0, 5] and [10, 15]. When: Validating interval constraints. Then: Inverted endpoints and relation conflict ([0,5] b [10,15] eq) emit E0400. Diagnostic: E0400.";
        attribute ac_04_disjoint_relation_conflict_and_delimiter_recovery : String = "Given: Conflicting assertions Interval_A meets Interval_B and Interval_B before Interval_A. When: Checking intersection and error recovery. Then: Detects R (R')^ = , emits E0401, binds _{rel}, recovers on ;, halting at C_{max_errors}} with E0199. Diagnostic: E0401, E0199.";
        doc /* \begin{aligned} \mathcal{B}_{13}=\{\mathbf{b},\mathbf{bi},\mathbf{m},\mathbf{mi},\mathbf{o},\mathbf{oi},\mathbf{s},\mathbf{si},\mathbf{d},\mathbf{di},\mathbf{f},\mathbf{fi},\mathbf{eq}\},\quad \mathbf{m}_R\in [0,8191] \\ (X\mathbf{b}Y\iff x_e\lt y_s)\land (X\mathbf{m}Y\iff x_e=y_s)\land (X\mathbf{o}Y\iff x_s\lt y_s\lt x_e\lt y_e) \\ (X\mathbf{s}Y\iff x_s=y_s\land x_e\lt y_e)\land (X\mathbf{d}Y\iff y_s\lt x_s\land x_e\lt y_e) \\ (X\mathbf{f}Y\iff y_s\lt x_s\land x_e=y_e)\land (X\mathbf{eq}Y\iff x_s=y_s\land x_e=y_e) \\ \forall r\in\mathcal{B}_{13},\quad (X\,r\,Y\iff Y\,r^\smile X),\quad (R^\smile)^\smile=R,\quad \mathbf{m}_{R_1\cap R_2}=\mathbf{m}_{R_1}\operatorname{AND}\mathbf{m}_{R_2} \\ x_s\ge x_e\implies\text{Diagnostic}(X,\text{E0400})\land X\gets\bot_{\text{interval}} \\ (\mathbf{eq}\notin R(X,X)\lor\text{Eval}(X,Y)\notin R(X,Y))\implies\text{Diagnostic}(C,\text{E0400})\land C\gets\bot_{\text{rel}} \\ (R(X,Y)\cap (R(Y,X))^\smile=\emptyset)\implies\text{Diagnostic}(C,\text{E0401})\land C\gets\bot_{\text{rel}} \\ |\mathcal{D}_{\text{sink}}|\ge C_{\text{max\_errors}}\implies\text{Diagnostic}(\text{Compiler},\text{E0199}),\quad \Delta_n=\text{UUIDv5}(\text{OID}_{\text{DEAP}},\text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0115_Axiomatic_interval_relations_bitmask_algebra_difference_lowering_and_diagnostic_rules;
        require Invariant_REQ_0115_Axiomatic_interval_relations_bitmask_algebra_difference_lowering_and_diagnostic_rules;
        verify by AC_01_Atomic_Relation_Classification;
        verify by AC_02_Bitmask_Algebraic_Inversion_and_Conjunction;
        verify by AC_03_Inverted_Interval_and_Relation_Contradiction;
        verify by AC_04_Disjoint_Relation_Conflict_and_Delimiter_Recovery;
        satisfy by StateMachineSolverEngine;
    }
    doc /* UUIDv5: 00edf301-60ee-5d3d-85d3-6b21e495fc65 | Complexity: Class P (O(1) table lookup, O(N^3) path consistency) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / Allen Interval Algebra | Diagnostics: E0199, E0400, E0401 */
    requirement def REQ_0116_Temporal_Relation_Composition_Table {
        id = "REQ-0116";
        text = "The dynamics solver SHALL perform path consistency constraint propagation using Allen's 13 \times 13 composition table, verifying qualitative temporal constraints.";
        attribute uuidv5 : String = "00edf301-60ee-5d3d-85d3-6b21e495fc65";
        attribute complexity_class : String = "Class P (O(1) table lookup, O(N^3) path consistency)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / Allen Interval Algebra";
        attribute diagnostic_codes : String = "E0199, E0400, E0401";
        attribute ac_01_direct_transitive_deduction : String = "Given: Asserted X_1 b X_2 (before) and X_2 b X_3 in TemporalNetwork_Alpha. When: Path consistency executes. Then: Refines M[X_1, X_3] from U = 0x1FFF to { b } = 0x0001. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_converse_symmetry_and_fixed_point_idempotency : String = "Given: Consistent network of N = 10 intervals converged to M^*. When: Verifying symmetry and re-executing propagation on M^*. Then: All M^*[j, i] (M^*[i, j])^, queue |Q| = 0, with zero mutations. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_latent_conflict_rejection : String = "Given: Circular triad X_1 b X_2, X_2 b X_3, and X_3 b X_1. When: Propagation intersects deduced X_1 b X_3 with (X_3 b X_1)^ = bi. Then: Intersection yields , rejecting with E0400. Diagnostic: E0400 (PathConsistencyViolation).";
        attribute ac_04_reflexive_contradiction_and_delimiter_recovery : String = "Given: Invalid X_1 b X_1 before ;, then valid TemporalNetwork_Beta. When: Lowering and constraint ingestion execute. Then: Emits E0401, binds _{tcn}, recovers at ;, parsing TemporalNetwork_Beta, halting at C_{max_errors}} with E0199. Diagnostic: E0401, E0199.";
        doc /* \begin{aligned} \mathbf{m}_R=\sum_{k=0}^{12}b_k\cdot 2^k\in [0,8191],\quad R_1\circ R_2=\bigcup_{r_1\in R_1}\bigcup_{r_2\in R_2}\mathbf{T}_{\text{comp}}[r_1][r_2] \\ \mathbf{M}^{(t+1)}[i,j]=\mathbf{M}^{(t)}[i,j]\cap (\mathbf{M}^{(t)}[i,k]\circ\mathbf{M}^{(t)}[k,j]),\quad \mathbf{M}[j,i]\equiv (\mathbf{M}[i,j])^\smile \\ \forall i,j,k\in \{1,\dots,N\},\quad \mathbf{M}^*[i,j]\subseteq \mathbf{M}^*[i,k]\circ\mathbf{M}^*[k,j] \\ (\exists i,j:\mathbf{M}[i,j]=\emptyset)\implies\text{Diagnostic}(\langle X_i,X_j\rangle,\text{E0400})\land\mathbf{M}\gets\bot_{\text{tcn}} \\ (\exists i:\mathbf{eq}\notin\mathbf{M}[i,i])\implies\text{Diagnostic}(X_i,\text{E0401})\land\mathbf{M}[i,i]\gets\bot_{\text{rel}} \\ |\mathcal{D}_{\text{sink}}|\ge C_{\text{max\_errors}}\implies\exists D_{\text{term}}\in\mathcal{D}_{\text{sink}}:\text{code}(D_{\text{term}})=\text{E0199} \end{aligned} */
        constraint def Invariant_REQ_0116_Qualitative_relation_bitmask_encoding_composition_evaluation_path_consistency_fixed_point_and_diagnostic_triggers;
        require Invariant_REQ_0116_Qualitative_relation_bitmask_encoding_composition_evaluation_path_consistency_fixed_point_and_diagnostic_triggers;
        verify by AC_01_Direct_Transitive_Deduction;
        verify by AC_02_Converse_Symmetry_and_Fixed_Point_Idempotency;
        verify by AC_03_Latent_Conflict_Rejection;
        verify by AC_04_Reflexive_Contradiction_and_Delimiter_Recovery;
        satisfy by StateMachineSolverEngine;
    }
    doc /* UUIDv5: a5f8e352-0833-5200-a575-76a65a0c02d1 | Complexity: Class NP-complete} / Class P (O(N^3) for H_{ORD}) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / Nebel & Bürckert (1995) | Diagnostics: E0199, E0400, E0406 */
    requirement def REQ_0117_Allen_s_Interval_Algebra_Temporal_Consistency {
        id = "REQ-0117";
        text = "The temporal solver SHALL enforce consistency over lifecycles and interval constraints.";
        attribute uuidv5 : String = "a5f8e352-0833-5200-a575-76a65a0c02d1";
        attribute complexity_class : String = "Class NP-complete} / Class P (O(N^3) for H_{ORD})";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / Nebel & Bürckert (1995)";
        attribute diagnostic_codes : String = "E0199, E0400, E0406";
        attribute ac_01_pure_ord_horn_consistent_network_propagation : String = "Given: Model Package_0 with 15 occurrences constrained by H_{ORD} relations. When: Path consistency propagation executes. Then: Computes fixed point in O(N^3) operations. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_pure_ord_horn_contradiction_rejection : String = "Given: Occurrences Occurrence_Alpha, Occurrence_Beta, and Occurrence_Gamma with cyclic before relations. When: Path consistency evaluates triangular compositions. Then: Derives empty relation R = , binds _{net}, emitting E0400. Diagnostic: E0400.";
        attribute ac_03_general_temporal_network_smt_satisfiability : String = "Given: Network Network_Beta with non-ORD-Horn constraints (R H_{ORD}). When: Compiling to QF_RDL and solving via SMT. Then: Proves satisfiability within _{temporal} steps. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_04_smt_step_budget_exceeded_delimiter_recovery : String = "Given: Network exceeding budget _{temporal} = 50 before ;, followed by Occurrence_Delta. When: SMT solver hits step limit and recovers. Then: Emits E0406, recovers on ;, parses Occurrence_Delta, emitting E0199 at C_{max_errors}}. Diagnostic: E0406, E0199.";
        doc /* \begin{aligned} \mathcal{B}_{13}=\{\mathbf{b},\mathbf{bi},\mathbf{m},\mathbf{mi},\mathbf{o},\mathbf{oi},\mathbf{s},\mathbf{si},\mathbf{d},\mathbf{di},\mathbf{f},\mathbf{fi},\mathbf{eq}\},\quad |\mathcal{H}_{\text{ORD}}|=868 \\ \mathbf{m}_{R(i,j)}=\sum_{k=0}^{12}b_k 2^k\in[0,8191],\quad \text{IsOrdHorn}(R(o_i,o_j))\iff\mathbf{m}_{R(i,j)}\in\mathcal{T}_{\text{ORD\_Horn}} \\ \forall i,j,k\in\{1,\dots,|\mathcal{O}|\},\quad R(o_i,o_k)\leftarrow R(o_i,o_k)\cap(R(o_i,o_j)\circ R(o_j,o_k)) \\ \Phi_{\text{temporal}}=\bigwedge_{o_i\in\mathcal{O}}(t_{i,s}-t_{i,e}\le -\epsilon_{\text{dur}})\land\bigwedge_{C_{ij}\notin\mathcal{H}_{\text{ORD}}}\left(\bigvee_{r\in R_{ij}}b_{ijr}\land\bigwedge_{r\in R_{ij}}(b_{ijr}\implies\text{Diff}(r))\right) \\ (R(o_i,o_j)=\emptyset\lor\Phi_{\text{temporal}}\equiv\text{UNSAT})\implies\text{Diagnostic}(\langle o_i,o_j\rangle,\text{E0400})\land\mathcal{G}_{\text{temp}}\gets\bot_{\text{net}} \\ \text{Steps}(\text{Solve}(\Phi_{\text{temporal}}))\gt\kappa_{\text{temporal}}\implies\text{Diagnostic}(\mathcal{G}_{\text{temp}}, \text{E0406})\land\mathcal{G}_{\text{temp}}\gets\bot_{\text{net}} \\ \forall n\in\mathcal{V}_{\text{AST}},\quad \Delta_n=\text{UUIDv5}(\text{NS}_{\text{DEAP}},\text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0117_Temporal_invariants_ORD_Horn_classification_path_consistency_QF_RDL_lowering_and_diagnostics;
        require Invariant_REQ_0117_Temporal_invariants_ORD_Horn_classification_path_consistency_QF_RDL_lowering_and_diagnostics;
        verify by AC_01_Pure_ORD_Horn_Consistent_Network_Propagation;
        verify by AC_02_Pure_ORD_Horn_Contradiction_Rejection;
        verify by AC_03_General_Temporal_Network_SMT_Satisfiability;
        verify by AC_04_SMT_Step_Budget_Exceeded_Delimiter_Recovery;
        satisfy by StateMachineSolverEngine;
    }
    doc /* UUIDv5: 0cf37885-69d7-51ea-9ff9-3ab9b0c9ea3d | Complexity: Class P (O(V + E) topological sort / Tarjan SCC) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 / KerML | Diagnostics: E0199, E0400, E0403 */
    requirement def REQ_0118_Directed_Causal_Precedence_DAG_Construction {
        id = "REQ-0118";
        text = "The dynamics solver SHALL construct directed causal precedence graph G_{prec} = (V, E) across action executions, behavioral steps, and state transitions, asserting acyclicity to guarantee deadlock-free execution.";
        attribute uuidv5 : String = "0cf37885-69d7-51ea-9ff9-3ab9b0c9ea3d";
        attribute complexity_class : String = "Class P (O(V + E) topological sort / Tarjan SCC)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 / KerML";
        attribute diagnostic_codes : String = "E0199, E0400, E0403";
        attribute ac_01_valid_acyclic_pipeline_precedence_and_linearization : String = "Given: Valid Package_0 declaring sequential actions Action_1, Action_2, Action_3 with intervals [0.0, 10.0], [10.0, 20.0], [20.0, 30.0]. When: Lowering constructs G_{prec}, verifies acyclicity via Tarjan SCC, and synthesizes topological order L. Then: Verifies strict acyclicity, emitting schedule Action_1}, Action_2}, Action_3} . Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_deterministic_lexicographical_ready_queue_tie_breaking : String = "Given: Diamond DAG PrecedenceGraph_Alpha where Action_1 precedes Action_2 and Action_3, both preceding Action_4. When: Kahn topological linearization dequeues candidate ready vertices. Then: Dequeues Action_2 before Action_3 via FQN byte order, yielding reproducible schedules across permuted runs. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_directed_causal_cycle_rejection_and_traceback : String = "Given: Malformed model declaring mutual cycle Action_1 Action_2 and Action_2 Action_1. When: Tarjan SCC decomposition audits G_{prec}. Then: Isolates component |C| = 2, reconstructs traceback Action_1}, Action_2}, Action_1} , and emits E0400. Diagnostic: E0400 (CausalPrecedenceCycleDetected).";
        attribute ac_04_temporal_ordering_inversion_and_delimiter_recovery : String = "Given: Invalid dependency Action_1 _{seq} Action_2 with intervals [10.0, 20.0] and [5.0, 15.0] before delimiter (;), followed by valid PrecedenceGraph_Beta. When: Lowering checks interval ordering and triggers delimiter recovery. Then: Emits E0403, binds _{prec}, recovers on ;, parses PrecedenceGraph_Beta, halting at C_{max_errors}} with E0199. Diagnostic: E0403, E0199.";
        doc /* \begin{aligned} G_{\text{prec}}=(V,E),\quad E=E_{\text{seq}}\cup E_{\text{flow}}\cup E_{\text{state}}\cup E_{\text{trig}}\cup E_{\text{temp}}\subseteq V\times V \\ u\prec v\iff (u,v)\in E,\quad \text{Reach}(v,G_{\text{prec}})=\{w\in V\mid v\prec^+ w\} \\ \text{Acyclic}(G_{\text{prec}})\iff \forall v\in V,\; v\notin\text{Reach}(v,G_{\text{prec}})\iff\forall C\in\text{TarjanSCC}(G_{\text{prec}}),\; (|C|=1\land (v,v)\notin E) \\ \forall (u,v)\in E,\; t_{\text{start}}(u)\le t_{\text{start}}(v)\quad\land\quad\forall (u,v)\in E_{\text{seq}},\; t_{\text{end}}(u)\le t_{\text{start}}(v) \\ \mathcal{L}=\langle v_1,\dots,v_n\rangle\implies \forall (u,v)\in E,\; \text{Index}(u,\mathcal{L})\lt\text{Index}(v,\mathcal{L}) \\ v_t=\arg\min_{v\in Q_t}(\text{FQN}(v)),\quad Q_t=\{v\in V\setminus\{v_1,\dots,v_{t-1}\}\mid\text{deg}^{-}(v)=0\} \\ \exists C:(|C|\gt 1\lor (v,v)\in E)\implies\text{Diagnostic}(\mathcal{P}_{\text{cycle}},\text{E0400})\land E_{\text{overlay}}=E\setminus\{(v_k,v_1)\} \\ (t_{\text{start}}(v)\lt t_{\text{start}}(u)\lor ((u,v)\in E_{\text{seq}}\land t_{\text{start}}(v)\lt t_{\text{end}}(u)))\implies\text{Diagnostic}((u,v),\text{E0403}) \\ |\mathcal{D}_{\text{sink}}|\ge C_{\text{max\_errors}}\implies\text{Diagnostic}(\text{Compiler},\text{E0199}),\quad \Delta_v=\text{UUIDv5}(\text{NS}_{\text{DEAP}},\text{FQN}(v)) \end{aligned} */
        constraint def Invariant_REQ_0118_Causal_precedence_DAG_construction_Tarjan_SCC_decomposition_temporal_consistency_and_topological_scheduling;
        require Invariant_REQ_0118_Causal_precedence_DAG_construction_Tarjan_SCC_decomposition_temporal_consistency_and_topological_scheduling;
        verify by AC_01_Valid_Acyclic_Pipeline_Precedence_and_Linearization;
        verify by AC_02_Deterministic_Lexicographical_Ready_Queue_Tie_Breaking;
        verify by AC_03_Directed_Causal_Cycle_Rejection_and_Traceback;
        verify by AC_04_Temporal_Ordering_Inversion_and_Delimiter_Recovery;
        satisfy by StateMachineSolverEngine;
    }
    doc /* UUIDv5: 9967ca70-d75e-53ca-aae1-e07ec6023403 | Complexity: Class P (O(V + E) DAG execution lowering) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 | Diagnostics: E0199, E0400, E0404 */
    requirement def REQ_0119_Action_Step_Sequence_Execution_Semantics {
        id = "REQ-0119";
        text = "The dynamics engine SHALL lower action sequences, fork nodes, and join nodes into execution DAGs, verifying precedence, barrier synchronization, and data-race freedom.";
        attribute uuidv5 : String = "9967ca70-d75e-53ca-aae1-e07ec6023403";
        attribute complexity_class : String = "Class P (O(V + E) DAG execution lowering)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119";
        attribute diagnostic_codes : String = "E0199, E0400, E0404";
        attribute ac_01_sequential_precedence_lowering : String = "Given: Steps Step_Alpha, Step_Beta with t_{end} = t_{start} = 10.0. When: Lowering constructs DAG G_{exec} and verifies precedence. Then: Validates t_{end} t_{start} without diagnostic errors. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_fork_join_concurrency_and_barrier_timing : String = "Given: Fork ForkNode_1 (k = 3) with paths Path_Alpha, Path_Beta, Path_Gamma (durations 10.0, 25.0, 15.0) meeting at JoinNode_1. When: Evaluating barrier synchronization and token conservation. Then: Computes t_{start}(JoinNode_1}) = 25.0 and Tokens_{net} = 0. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_degree_malformation_and_cycle_rejection : String = "Given: ForkNode_1 with out-degree 1, or cycle Step_Alpha} Step_Beta} Step_Alpha}. When: Validating degrees and acyclicity via Kahn's sort. Then: Rejects invalid degrees and cycles, emitting E0400. Diagnostic: E0400 (MalformedControlNode).";
        attribute ac_04_bernstein_race_hazard_and_delimiter_recovery : String = "Given: Branches Path_Alpha and Path_Beta mutating param_x : Real before ;, followed by Step_Gamma. When: Checking Bernstein conditions and recovery. Then: Emits E0404, recovers on ;, parses Step_Gamma, halting at C_{max_errors}} with E0199. Diagnostic: E0404, E0199.";
        doc /* \begin{aligned} G_{\text{exec}}=(V,E_{\text{ctrl}}\cup E_{\text{data}}),\quad V=V_{\text{step}}\cup V_{\text{ctrl}},\quad V_{\text{ctrl}}=V_{\text{fork}}\cup V_{\text{join}} \\ \forall (s_i,s_{i+1})\in E_{\text{ctrl}},\; t_{\text{end}}(s_i)\le t_{\text{start}}(s_{i+1}) \\ \text{in-deg}(N_{\text{fork}})=1\land\text{out-deg}(N_{\text{fork}})=k\ge 2,\; t_{\text{start}}(P_i)=t_{\text{end}}(N_{\text{fork}}) \\ \text{in-deg}(N_{\text{join}})=k\ge 2\land\text{out-deg}(N_{\text{join}})=1,\; t_{\text{start}}(N_{\text{join}})=\max_{i=1}^k t_{\text{end}}(P_i) \\ \text{Tokens}_{\text{consumed}}(N_{\text{join}})=\text{Tokens}_{\text{produced}}(N_{\text{fork}})=k\implies\Delta\text{Tokens}_{\text{net}}=0 \\ \forall i\ne j,\; (\mathcal{W}_i\cap\mathcal{W}_j=\emptyset)\land(\mathcal{W}_i\cap\mathcal{R}_j=\emptyset)\land(\mathcal{R}_i\cap\mathcal{W}_j=\emptyset) \\ \forall v\in V,\; v\notin\text{Reach}^+(v,G_{\text{exec}}),\quad \Delta_n=\text{UUIDv5}(\text{NS}_{\text{DEAP}},\text{FQN}(n)) \\ (\text{deg}(N)\text{ invalid}\lor\text{CycleDetected})\implies\text{Diagnostic}(n,\text{E0400}) \\ (t_{\text{start}}(N_{\text{join}})\lt\max_i t_{\text{end}}(P_i)\lor\text{DataRace})\implies\text{Diagnostic}(n,\text{E0404}) \end{aligned} */
        constraint def Invariant_REQ_0119_Execution_DAG_topology_fork_join_synchronization_and_diagnostic_trigger_predicates;
        require Invariant_REQ_0119_Execution_DAG_topology_fork_join_synchronization_and_diagnostic_trigger_predicates;
        verify by AC_01_Sequential_Precedence_Lowering;
        verify by AC_02_Fork_Join_Concurrency_and_Barrier_Timing;
        verify by AC_03_Degree_Malformation_and_Cycle_Rejection;
        verify by AC_04_Bernstein_Race_Hazard_and_Delimiter_Recovery;
        satisfy by StateMachineSolverEngine;
    }
    doc /* UUIDv5: cb960479-dda3-5a5b-9b29-485b9097cc10 | Complexity: Class P (O(D) state hierarchy traversal, O(M) orthogonal region check) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 / KerML | Diagnostics: E0199, E0402, E0405 */
    requirement def REQ_0120_Hierarchical_State_Machine_Containment {
        id = "REQ-0120";
        text = "The state machine solver SHALL validate and lower hierarchical statecharts and orthogonal regions into active configurations.";
        attribute uuidv5 : String = "cb960479-dda3-5a5b-9b29-485b9097cc10";
        attribute complexity_class : String = "Class P (O(D) state hierarchy traversal, O(M) orthogonal region check)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 / KerML";
        attribute diagnostic_codes : String = "E0199, E0402, E0405";
        attribute ac_01_hierarchical_tree_active_configuration : String = "Given: CompositeState_1 with children State_Alpha, State_Beta, activating State_Alpha. When: Lowering validates tree hierarchy and active states. Then: Verifies ancestor closure and XOR singularity (|C C| = 1). Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_orthogonal_concurrency_region_disjointness : String = "Given: ParallelState_Omega with regions Region_1, Region_2, activating leaves in both. When: Validating region multiplicity, disjointness, and concurrency. Then: Confirms m = 2 2, disjoint sets, and |C Leaves(R_i)| = 1. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_containment_acyclicity_ancestor_closure_rejection : String = "Given: Hierarchy cycle State_Alpha State_Beta State_Alpha, or active leaf without parent. When: Checking containment acyclicity and ancestor closure. Then: Rejects cycle or missing ancestor, emits E0402, and binds _{state}. Diagnostic: E0402 (StateHierarchyContainmentViolation).";
        attribute ac_04_degenerate_region_cross_region_recovery : String = "Given: Degenerate parallel state (m = 1) before ;, followed by an unsynchronized cross-region transition. When: Semantic validation and delimiter recovery execute. Then: Emits E0405, binds _{region}, recovers on ;, halting at C_{max_errors}} with E0199. Diagnostic: E0405, E0199.";
        doc /* \begin{aligned} \mathcal{S}_{\text{all}}=\mathcal{S}_{\text{atomic}}\cup\mathcal{S}_{\text{xor}}\cup\mathcal{S}_{\text{orth}}\cup\mathcal{R}\cup\mathcal{P},\quad \forall s\in\mathcal{S}_{\text{all}}\setminus\{s_{\text{root}}\},\;\exists ! p\in\mathcal{S}_{\text{all}}:\text{parent}(s)=p \\ \forall s\in\mathcal{S}_{\text{all}},\;(s,s)\notin\sqsubset^+,\quad \forall S_{\text{orth}}\in\mathcal{S}_{\text{orth}},\;|\mathcal{R}(S_{\text{orth}})|=m\ge 2 \\ \forall R_i,R_j\in\mathcal{R}(S_{\text{orth}}),\;i\ne j\implies\text{States}(R_i)\cap\text{States}(R_j)=\emptyset \\ s_{\text{root}}\in C\land(\forall s\in C,\;\text{ancestors}(s)\subseteq C)\land(\forall S_{\text{xor}}\in C\cap\mathcal{S}_{\text{xor}},\;|C\cap\mathcal{C}_{S_{\text{xor}}}|=1) \\ \forall S_{\text{orth}}\in C\cap\mathcal{S}_{\text{orth}},\;\forall R_i\in\mathcal{R}(S_{\text{orth}}),\;|C\cap\text{Leaves}(R_i)|=1 \\ \forall l_1,l_2\in\text{Leaves}(C),\;l_1\ne l_2\implies\text{LCA}(l_1,l_2)\in\mathcal{S}_{\text{orth}} \\ ((s,s)\in\sqsubset^+\lor\text{ancestors}(s)\not\subseteq C\lor |C\cap\mathcal{C}_{S_{\text{xor}}}|\ne 1)\implies\text{Diagnostic}(s,\text{E0402})\land s\gets\bot_{\text{state}} \\ (|\mathcal{R}(S_{\text{orth}})|\lt 2\lor |C\cap\text{Leaves}(R_i)|\ne 1\lor\text{UnsyncCrossRegion}(\tau))\implies\text{Diagnostic}(S_{\text{orth}},\text{E0405})\land S_{\text{orth}}\gets\bot_{\text{region}} \\ \forall s\in\mathcal{S}_{\text{all}},\;\Delta_s=\text{UUIDv5}(\text{NS}_{\text{DEAP}},\text{FQN}(s)) \end{aligned} */
        constraint def Invariant_REQ_0120_Statechart_tree_acyclicity_region_disjointness_configuration_validity_and_diagnostic_triggers;
        require Invariant_REQ_0120_Statechart_tree_acyclicity_region_disjointness_configuration_validity_and_diagnostic_triggers;
        verify by AC_01_Hierarchical_Tree_Active_Configuration;
        verify by AC_02_Orthogonal_Concurrency_Region_Disjointness;
        verify by AC_03_Containment_Acyclicity_Ancestor_Closure_Rejection;
        verify by AC_04_Degenerate_Region_Cross_Region_Recovery;
        satisfy by StateMachineSolverEngine;
    }
    doc /* UUIDv5: 04751a64-acaa-5a57-b311-f1ceb3c9881d | Complexity: Class P (O(1) event dequeue, O(T) RTC step lowering) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 / KerML | Diagnostics: E0199, E0400, E0402 */
    requirement def REQ_0121_State_Transition_Trigger_Event_Dispatching {
        id = "REQ-0121";
        text = "The state machine solver SHALL validate and lower discrete event dispatching under Run-to-Completion (RTC) semantics with a bounded FIFO event queue.";
        attribute uuidv5 : String = "04751a64-acaa-5a57-b311-f1ceb3c9881d";
        attribute complexity_class : String = "Class P (O(1) event dequeue, O(T) RTC step lowering)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 / KerML";
        attribute diagnostic_codes : String = "E0199, E0400, E0402";
        attribute ac_01_fifo_dequeue_rtc_transition_firing : String = "Given: EventQueue_1 with Event_Signal1},Event_Call1} in State_Alpha. When: Dequeuing Event_Signal1 fires Transition_1 to State_Beta. Then: Actions terminate; C'={State_Beta}} settles before next dequeue. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_rtc_non_interruptibility_queue_overflow : String = "Given: Active RTC step in StateMachine_Alpha with full queue (|Q|=65536). When: Event arrives during step, followed by overflow enqueue. Then: Buffers event at tail without preemption; overflow emits E0400. Diagnostic: E0400 (EventQueueOverflow).";
        attribute ac_03_trigger_payload_mismatch_ambiguous_guards : String = "Given: Transition expecting param_x : Real with payload mismatch, or overlapping sibling guards. When: Type checking and conflict analysis execute. Then: Rejects payload mismatch or ambiguous guard overlap with E0400. Diagnostic: E0400 (MalformedTriggerOrConflict).";
        attribute ac_04_hierarchy_containment_fault_recovery : String = "Given: Transition violating containment before ;, followed by a valid transition. When: Containment checking and delimiter recovery execute. Then: Emits E0402, binds _{trans}, recovers on ;, halting at C_{max_errors}} with E0199. Diagnostic: E0402, E0199.";
        doc /* \begin{aligned} e=\langle\text{id}_e,\text{name}_e,\kappa_e,\pi_e,t_{\text{enq}},\Delta_e\rangle\in\mathcal{E},\quad Q=\langle e_1,\dots,e_p\rangle\in\mathcal{E}^*,\;|Q|\le K_{\text{max\_queue}} \\ \text{MatchTrigger}(\tau,e)\iff(\text{trigger}(\tau)=\text{name}_e)\land(\text{kind}(\tau)=\kappa_e)\land(\text{type}(\pi_e)\sqsubseteq\text{type}(\text{param}(\tau))) \\ \text{Enabled}(\tau,C,e,\sigma)\iff\text{source}(\tau)\in C\land\text{MatchTrigger}(\tau,e)\land(\text{guard}(\tau)=\emptyset\lor\text{Eval}(\text{guard}(\tau),\sigma)) \\ \forall\tau_1,\tau_2\in\text{Enabled}(C,e,\sigma),\;\tau_1\succ\tau_2\iff\text{depth}(\text{source}(\tau_1))\gt\text{depth}(\text{source}(\tau_2)) \\ \langle C,Q,\sigma,t\rangle\xrightarrow{\text{RTCStep}(e)}\langle C',Q',\sigma',t'\rangle,\quad\forall a\in\text{Actions}(\mathcal{T}_{\text{fire}}),\;\text{status}(a,t')=\text{Terminated} \\ (|Q|\ge K_{\text{max\_queue}}\lor\text{type}(\pi_e)\not\sqsubseteq\text{type}(\text{param}(\tau))\lor\text{AmbiguousGuards}\lor\delta_{\text{timer}}\le 0)\implies\text{Diagnostic}(n,\text{E0400})\land n\gets\bot_{\text{event}} \\ (\text{source}(\tau)\notin C\lor\text{target}(\tau)\notin\mathcal{S}_{\text{all}})\implies\text{Diagnostic}(\tau,\text{E0402})\land\tau\gets\bot_{\text{trans}} \\ \forall n\in\mathcal{V}_{\text{event}}\cup\mathcal{V}_{\text{queue}}\cup\mathcal{V}_{\text{trigger}},\;\Delta_n=\text{UUIDv5}(\text{NS}_{\text{DEAP}},\text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0121_Event_queue_semantics_trigger_matching_and_diagnostic_triggers;
        require Invariant_REQ_0121_Event_queue_semantics_trigger_matching_and_diagnostic_triggers;
        verify by AC_01_FIFO_Dequeue_RTC_Transition_Firing;
        verify by AC_02_RTC_Non_Interruptibility_Queue_Overflow;
        verify by AC_03_Trigger_Payload_Mismatch_Ambiguous_Guards;
        verify by AC_04_Hierarchy_Containment_Fault_Recovery;
        satisfy by StateMachineSolverEngine;
    }
    doc /* UUIDv5: 4aac8fa3-b145-5d59-a81f-55b7b01eb9ea | Complexity: Class NP-complete} (SMT) / Tractable Fragment Class P (Syntactic) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 / KerML | Diagnostics: E0199, E0400, E0402 */
    requirement def REQ_0122_Deterministic_Transition_Guard_Disjointness_Verification {
        id = "REQ-0122";
        text = "The compiler SHALL verify that outgoing transitions from any state on identical events have mutually disjoint guards (G_1 \wedge G_2 \equiv false), ensuring determinism.";
        attribute uuidv5 : String = "4aac8fa3-b145-5d59-a81f-55b7b01eb9ea";
        attribute complexity_class : String = "Class NP-complete} (SMT) / Tractable Fragment Class P (Syntactic)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 / KerML";
        attribute diagnostic_codes : String = "E0199, E0400, E0402";
        attribute ac_01_disjoint_guard_verification_happy_path : String = "Given: State_Alpha with Transition_Alpha (param_x < 0.0) and Transition_Beta (param_x >= 0.0) on Event_0. When: Lowering verifies guard disjointness. Then: Disjointness is certified via interval checks. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_non_disjoint_transition_guard_rejection : String = "Given: Sibling Transition_Alpha (param_x >= 10.0) and Transition_Beta (param_x <= 20.0) on Event_0. When: SMT solver evaluates G_1 G_2. Then: Conjunction is satisfiable (param_x = 15.0), emitting E0400 with counterexample. Diagnostic: E0400 (NonDisjointTransitionGuards).";
        attribute ac_03_solver_step_limit_bound_enforcement : String = "Given: Guard conjunction requiring steps exceeding budget _{smt} = 100{,}000. When: SMT solver executes. Then: Halts deterministically without panicking, emitting E0400. Diagnostic: E0400 (GuardSolverStepLimitExceeded).";
        attribute ac_04_non_admissible_theory_delimiter_recovery : String = "Given: Guard param_x * param_y > 1.0 before ;, then valid transition. When: Admissibility checking and recovery execute. Then: Emits E0402, binds _{guard}, recovers on ;, halting at C_{max_errors}} with E0199. Diagnostic: E0402, E0199.";
        doc /* \begin{aligned} \mathcal{T}_{s,e}=\{t\in\mathcal{T}_{\text{out}}(s)\mid e_{\text{trigger}}(t)=e\},\quad \overline{G}(t)=(g_{\text{guard}}(t)\ne\bot_{\text{none}}\;?\;g_{\text{guard}}(t):\text{true}) \\ \forall s\in\mathcal{S},\;\forall e\in\mathcal{E}(s),\;\forall t_1,t_2\in\mathcal{T}_{s,e},\;t_1\ne t_2\implies\text{UNSAT}(\overline{G}(t_1)\land\overline{G}(t_2)) \\ \text{UNSAT}(G_1\land G_2)\iff\neg\exists\mathbf{x}\in\mathcal{X}:(G_1(\mathbf{x})=\text{true}\land G_2(\mathbf{x})=\text{true}) \\ \text{SAT}(\overline{G}(t_1)\land\overline{G}(t_2))\implies\text{Diagnostic}(\langle t_1,t_2\rangle,\text{E0400})\land t_1,t_2\gets\bot_{\text{trans}} \\ (\tau(g_{\text{guard}}(t))\not\equiv\text{Boolean}\lor\neg\text{Admissible}(g_{\text{guard}}(t)))\implies\text{Diagnostic}(t,\text{E0402})\land g_{\text{guard}}(t)\gets\bot_{\text{guard}} \\ N_{\text{step}}\gt\kappa_{\text{smt}}\implies\text{Diagnostic}(\langle t_1,t_2\rangle,\text{E0400}) \\ |\mathcal{D}_{\text{sink}}|\ge C_{\text{max\_errors}}\implies\exists D_{\text{term}}\in\mathcal{D}_{\text{sink}}:\text{code}(D_{\text{term}})=\text{E0199} \\ \forall t\in\mathcal{T},\;\Delta_t=\text{UUIDv5}(\text{NS}_{\text{DEAP}},\text{FQN}(t)) \end{aligned} */
        constraint def Invariant_REQ_0122_Partitioning_guard_normalization_mutual_exclusion_and_solver_bounds;
        require Invariant_REQ_0122_Partitioning_guard_normalization_mutual_exclusion_and_solver_bounds;
        verify by AC_01_Disjoint_Guard_Verification_Happy_Path;
        verify by AC_02_Non_Disjoint_Transition_Guard_Rejection;
        verify by AC_03_Solver_Step_Limit_Bound_Enforcement;
        verify by AC_04_Non_Admissible_Theory_Delimiter_Recovery;
        satisfy by StateMachineSolverEngine;
    }
    doc /* UUIDv5: 4754f095-d084-5a08-bf16-5232a31b1f90 | Complexity: Class P (O( | Standard: IEEE 29148-2018 §8.4 / RFC 2119 | Diagnostics: E0199, E0400, E0403 */
    requirement def REQ_0123_Complete_State_Machine_Reachability_Analysis {
        id = "REQ-0123";
        text = "The solver SHALL analyze state machine reachability, diagnosing dead states, unreachable transitions, and trap states.";
        attribute uuidv5 : String = "4754f095-d084-5a08-bf16-5232a31b1f90";
        attribute complexity_class : String = "Class P (O(";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119";
        attribute diagnostic_codes : String = "E0199, E0400, E0403";
        attribute ac_01_canonical_reachability_convergence : String = "Given: StateMachine_Alpha with initial State_0 reaching State_1, State_2 cyclically. When: Reachability executes fixed-point BFS traversal. Then: Computes R = {State_0}, State_1}, State_2}}, with D_{dead} = . Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_bitset_marking_hierarchical_propagation : String = "Given: State_Composite_1 with initial State_Sub_0 chained through 1,024 states. When: Reachability marks the bitset and traverses composite containment. Then: All 1,024 states are marked reachable; zero dead states reported. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_dead_state_island_rejection : String = "Given: StateMachine_Alpha with reachable State_0, isolated State_Dead_1, and island State_Dead_2 State_Dead_3. When: Reachability evaluates D_{dead} = S R. Then: Rejects compilation, diagnosing State_Dead_1, State_Dead_2, State_Dead_3 as dead. Diagnostic: E0400 (DeadStateDetected).";
        attribute ac_04_dead_transition_trap_delimiter_recovery : String = "Given: StateMachine_Alpha with false guard (g_ false) and trap State_Trap_1 before ;, followed by StateMachine_Beta. When: Reachability and delimiter recovery execute. Then: Emits E0403, binds _{trans}, recovers on ;, validating StateMachine_Beta, halting at C_{max_errors}} with E0199. Diagnostic: E0403, E0199.";
        doc /* \begin{aligned} \mathcal{G}_{\text{sm}}=\langle\mathcal{S},\mathcal{T},s_0,\mathcal{S}_{\text{final}}\rangle,\; s_0\in\mathcal{S},\;\mathcal{S}_{\text{final}}\subseteq\mathcal{S}\\ |\{s\in\mathcal{S}\mid\text{is\_initial}(s)=\text{true}\}|=1\\ \text{Post}(X)=\{s'\in\mathcal{S}\mid\exists s\in X,\;\exists\tau=(s,s',g_\tau)\in\mathcal{T}:\text{Sat}(g_\tau)\}\\ \mathcal{R}=\mu X.(\{s_0\}\cup\text{Post}(X))=\bigcup_{k=0}^{|\mathcal{S}|-1}R_k,\; R_0=\{s_0\},\; R_{k+1}=R_k\cup\text{Post}(R_k)\\ \mathcal{D}_{\text{dead}}=\mathcal{S}\setminus\mathcal{R},\;\mathcal{T}_{\text{dead}}=\{(s,s',g_\tau)\in\mathcal{T}\mid s\in\mathcal{D}_{\text{dead}}\lor\neg\text{Sat}(g_\tau)\}\\ \mathcal{D}_{\text{trap}}=\{s\in\mathcal{R}\setminus\mathcal{S}_{\text{final}}\mid\forall(s,s',g_\tau)\in\mathcal{T},\;\neg\text{Sat}(g_\tau)\}\\ (|\{s\in\mathcal{S}\mid\text{is\_initial}(s)\}|\ne 1\lor s\in\mathcal{D}_{\text{dead}})\implies\text{Diagnostic}(s,\text{E0400})\land s\gets\bot_{\text{state}}\\ (\tau\in\mathcal{T}_{\text{dead}}\lor s\in\mathcal{D}_{\text{trap}})\implies\text{Diagnostic}(\tau\lor s,\text{E0403})\land\tau\gets\bot_{\text{trans}}\\ \forall n\in\mathcal{S}\cup\mathcal{T},\;\Delta_n=\text{UUIDv5}(\text{NS}_{\text{DEAP}},\text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0123_State_machine_reachability_closure_dead_state_partitions_and_diagnostic_triggers;
        require Invariant_REQ_0123_State_machine_reachability_closure_dead_state_partitions_and_diagnostic_triggers;
        verify by AC_01_Canonical_Reachability_Convergence;
        verify by AC_02_Bitset_Marking_Hierarchical_Propagation;
        verify by AC_03_Dead_State_Island_Rejection;
        verify by AC_04_Dead_Transition_Trap_Delimiter_Recovery;
        satisfy by StateMachineSolverEngine;
    }
    doc /* UUIDv5: d67ee888-8533-5926-877a-16ee4a93743c | Complexity: Class PSPACE-complete} / NP-complete} (BMC depth k) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 | Diagnostics: E0199, E0400, E0406 */
    requirement def REQ_0124_Discrete_State_Machine_Reachability_Deadlock {
        id = "REQ-0124";
        text = "The solver SHALL verify reachability, deadlock freedom, and livelock absence across K \ge 2 orthogonal regions via Bounded Model Checking (BMC).";
        attribute uuidv5 : String = "d67ee888-8533-5926-877a-16ee4a93743c";
        attribute complexity_class : String = "Class PSPACE-complete} / NP-complete} (BMC depth k)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119";
        attribute diagnostic_codes : String = "E0199, E0400, E0406";
        attribute ac_01_bmc_unrolling_reachability_witness : String = "Given: CompositeState_Alpha with 2 regions (3 states each), target reachable at k = 3 (k_{max} = 10). When: Unrolling transitions into _k and solving SAT. Then: Returns SAT at k = 3, extracting witness s_0, s_1, s_2, s_3 . Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_bounded_horizon_cutoff_stuttering : String = "Given: CompositeState_Alpha requiring 60 steps (k_{max} = 50), with Region_1 active and Region_2 stuttering. When: Unrolling reaches horizon k_{max}. Then: Halts at depth 50, asserting x_2^{(t+1)} = x_2^{(t)}. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_deadlock_and_livelock_rejection : String = "Given: Model where Region_1 and Region_2 enter deadlocks or non-progress cycles. When: Evaluating _k with safety violation Safe(s_k). Then: Detects SAT at k k_{max}, emits E0400, binding trace to _{trace}. Diagnostic: E0400 (OrthogonalStateDeadlockDetected).";
        attribute ac_04_step_budget_cutoff_recovery : String = "Given: Model exceeding budget _{steps} = 100 before delimiter (;), followed by CompositeState_Beta. When: Step count exceeds budget (N_{step} _{steps}). Then: Emits E0406, binds _{state}, recovers on ;, halting at C_{max_errors}} with E0199. Diagnostic: E0406, E0199.";
        doc /* \begin{aligned} \mathcal{C}_{\text{global}}=\prod_{i=1}^K S_i,\quad \text{Occupancy}(\mathbf{x}^{(t)})\equiv\bigwedge_{i=1}^K\left(\bigvee_{q\in S_i}x_{i,q}^{(t)}\land\bigwedge_{q\ne q^\prime\in S_i}(\neg x_{i,q}^{(t)}\lor\neg x_{i,q^\prime}^{(t)})\right) \\ I(\mathbf{x}^{(0)})\equiv\text{Occupancy}(\mathbf{x}^{(0)})\land\bigwedge_{i=1}^K\left(x_{i,q_{\text{init},i}}^{(0)}\land\bigwedge_{q\in S_i\setminus\{q_{\text{init},i}\}}\neg x_{i,q}^{(0)}\right) \\ T(\mathbf{x}^{(t)},\mathbf{x}^{(t+1)})\equiv\text{Occupancy}(\mathbf{x}^{(t)})\land\text{Occupancy}(\mathbf{x}^{(t+1)})\land(T_{\text{interleave}}\lor T_{\text{sync}}\lor T_{\text{stutter}}) \\ \text{Safe}(\mathbf{x}^{(t)})\equiv(\text{IsTerminal}(\mathbf{x}^{(t)})\lor\exists\mathbf{x}^{(t+1)},\,T(\mathbf{x}^{(t)},\mathbf{x}^{(t+1)}))\land\neg\text{LivelockCycle}(\mathbf{x}^{(0)},\dots,\mathbf{x}^{(t)}) \\ \Phi_k=I(\mathbf{x}^{(0)})\land\left(\bigwedge_{t=0}^{k-1}T(\mathbf{x}^{(t)},\mathbf{x}^{(t+1)})\right)\land\neg\text{Safe}(\mathbf{x}^{(k)}) \\ \Phi_k\models\text{SAT}\implies\text{Diagnostic}(\text{Trace}(\mathbf{w}),\text{E0400})\land\mathbf{w}\gets\bot_{\text{trace}} \\ N_{\text{step}}\gt\kappa_{\text{steps}}\implies\text{Diagnostic}(\mathcal{S}_{\text{orth}},\text{E0406})\land\mathcal{S}_{\text{orth}}\gets\bot_{\text{state}} \\ |\mathcal{D}_{\text{sink}}|\ge C_{\text{max\_errors}}\implies\text{Diagnostic}(\text{Compiler},\text{E0199}),\quad \Delta_n=\text{UUIDv5}(\text{NS}_{\text{DEAP}},\text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0124_Orthogonal_state_space_occupancy_composite_transition_relations_safety_predicates_and_diagnostic_bounds;
        require Invariant_REQ_0124_Orthogonal_state_space_occupancy_composite_transition_relations_safety_predicates_and_diagnostic_bounds;
        verify by AC_01_BMC_Unrolling_Reachability_Witness;
        verify by AC_02_Bounded_Horizon_Cutoff_Stuttering;
        verify by AC_03_Deadlock_and_Livelock_Rejection;
        verify by AC_04_Step_Budget_Cutoff_Recovery;
        satisfy by StateMachineSolverEngine;
    }
    doc /* UUIDv5: 3a899b03-86b6-5a93-a5fd-061331fe2a91 | Complexity: Class P (O( | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 / KerML | Diagnostics: E0199, E0400, E0410 */
    requirement def REQ_0125_Failsafe_Fallback_Transitions {
        id = "REQ-0125";
        text = "The state machine solver SHALL validate failsafe fallback transitions, priority preemptive evacuation, and absorbing safe states across discrete statecharts.";
        attribute uuidv5 : String = "3a899b03-86b6-5a93-a5fd-061331fe2a91";
        attribute complexity_class : String = "Class P (O(";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 / KerML";
        attribute diagnostic_codes : String = "E0199, E0400, E0410";
        attribute ac_01_direct_and_hierarchical_fallback_evacuation : String = "Given: State_Nominal_Alpha in State_Super_0, State_Safe_Omega, and fallback Transition_Failsafe_1 from State_Super_0 ( = 100, g true). When: Validating fallback coverage and transition priorities. Then: Certifies inherited coverage, preempting nominal transitions ( = 10 100). Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_safe_state_reachability_and_absorbing_boundary : String = "Given: Operational states State_Operational_0, State_Operational_1 targeting State_Safe_Omega, with exit on Event_Reset. When: Validating reachability and absorbing boundaries. Then: Reverse BFS confirms reachability in O(V + E) time; verifies no spontaneous () or nominal exits. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_missing_fallback_and_isolation_rejection : String = "Given: State_Operational_0 lacking fallback for Event_Fault_Critical, or State_Safe_Omega with spontaneous exit. When: Auditing fallback completeness and absorbing boundaries. Then: Rejects model, emits E0400, and binds sentinel _{failsafe}. Diagnostic: E0400 (MissingFailsafeTransition or SpontaneousSafeStateExit).";
        attribute ac_04_priority_inversion_and_delimiter_recovery : String = "Given: Failsafe transition with guard param_x > 0.0 or priority = 5 _{nominal} = 10 before ;, followed by a valid state. When: Evaluating semantic rules and delimiter recovery. Then: Emits E0410, binds sentinel _{trans}, recovers on ;, halting at C_{max_errors}} with E0199. Diagnostic: E0410, E0199.";
        doc /* \begin{aligned} \mathcal{S}=\mathcal{S}_{\text{op}}\cup\mathcal{S}_{\text{safe}}\cup\mathcal{S}_{\text{init}},\quad \mathcal{E}=\mathcal{E}_{\text{nominal}}\cup\mathcal{E}_{\text{fault}}\cup\mathcal{E}_{\text{reset}} \\ \mathcal{T}_{\text{failsafe}}=\{T\in\mathcal{T}\mid e_{\text{trig}}(T)\in\mathcal{E}_{\text{fault}}\land s_{\text{tgt}}(T)\in\mathcal{S}_{\text{safe}}\} \\ \forall s\in\mathcal{S}_{\text{op}},\;\forall e\in\mathcal{E}_{\text{fault}},\;\exists T\in\mathcal{T}_{\text{failsafe}}:s_{\text{src}}(T)\in\text{scope}(s)\land e_{\text{trig}}(T)=e \\ \forall T_{\text{fail}}\in\mathcal{T}_{\text{failsafe}},\;\forall T_{\text{nom}}\in\mathcal{T}_{\text{nominal}}\text{ in scope},\;\pi(T_{\text{fail}})\gt\pi(T_{\text{nom}}) \\ \forall T\in\mathcal{T}_{\text{failsafe}},\;g(T)\equiv\text{true},\quad \forall s\in\mathcal{S}_{\text{op}},\;s_{\text{safe}}\in\text{Reach}(s,G_{\text{failsafe}}) \\ \forall T\in\mathcal{T},\;s_{\text{src}}(T)\in\mathcal{S}_{\text{safe}}\implies(e_{\text{trig}}(T)\in\mathcal{E}_{\text{reset}}\land e_{\text{trig}}(T)\ne\epsilon) \\ (\text{MissingFallback}(s,e)\lor s_{\text{safe}}\notin\text{Reach}(s,G_{\text{failsafe}})\lor\text{SpontaneousSafeExit}(T))\implies\text{Diagnostic}(u,\text{E0400})\land u\gets\bot_{\text{failsafe}} \\ (\pi(T_{\text{nom}})\ge\pi(T_{\text{fail}})\lor\text{EqualPrioConflict}(T_1,T_2)\lor g(T_{\text{fail}})\not\equiv\text{true})\implies\text{Diagnostic}(T,\text{E0410})\land T\gets\bot_{\text{trans}} \\ \forall u\in\mathcal{S}\cup\mathcal{T}\cup\mathcal{E},\;\Delta_u=\text{UUIDv5}(\text{NS}_{\text{DEAP}},\text{FQN}(u)) \end{aligned} */
        constraint def Invariant_REQ_0125_Failsafe_coverage_completeness_preemptive_priority_ordering_tautological_guard_satisfaction_safe_state_absorption_and_diagnostic_triggers;
        require Invariant_REQ_0125_Failsafe_coverage_completeness_preemptive_priority_ordering_tautological_guard_satisfaction_safe_state_absorption_and_diagnostic_triggers;
        verify by AC_01_Direct_and_Hierarchical_Fallback_Evacuation;
        verify by AC_02_Safe_State_Reachability_and_Absorbing_Boundary;
        verify by AC_03_Missing_Fallback_and_Isolation_Rejection;
        verify by AC_04_Priority_Inversion_and_Delimiter_Recovery;
        satisfy by StateMachineSolverEngine;
    }
    doc /* UUIDv5: 4816be6a-f42d-5f14-8b49-6a7ed290c5a6 | Complexity: Class P (O(D) tree configuration snapshot and restoration) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 / KerML | Diagnostics: E0199, E0400, E0411 */
    requirement def REQ_0126_State_History_Pseudostates_Shallow {
        id = "REQ-0126";
        text = "The state machine solver SHALL validate and execute Shallow History ([H]) and Deep History ([H*]) restoration semantics.";
        attribute uuidv5 : String = "4816be6a-f42d-5f14-8b49-6a7ed290c5a6";
        attribute complexity_class : String = "Class P (O(D) tree configuration snapshot and restoration)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 / KerML";
        attribute diagnostic_codes : String = "E0199, E0400, E0411";
        attribute ac_01_shallow_history_restoration : String = "Given: CompositeState_Alpha with State_Sub_0, State_Sub_1, and History_Shallow defaulting to State_Sub_0. When: State_Sub_1 active, CompositeState_Alpha exits and re-enters on History_Shallow. Then: Restores State_Sub_1 without evaluating State_Sub_0. Diagnostic: Diagnostic sink empty.";
        attribute ac_02_deep_history_subtree_restoration : String = "Given: CompositeState_Alpha with child CompositeState_Beta, active leaf State_Leaf_A2, and History_Deep. When: CompositeState_Alpha exits and re-enters on History_Deep. Then: Restores CompositeState_Beta and State_Leaf_A2 in O(D) time without initial transitions. Diagnostic: Diagnostic sink empty.";
        attribute ac_03_default_transition_execution : String = "Given: CompositeState_Alpha with History_Shallow and uninitialized history (H = ). When: Transition targets History_Shallow. Then: Fires T_{default}, activating State_Sub_0 in zero time ( t = 0). Diagnostic: Diagnostic sink empty.";
        attribute ac_04_structural_malformation_rejection : String = "Given: History at root s_{root}, duplicate history, and external target, delimited by ;. When: Validation passes execute. Then: Rejects root/duplicate with E0400, rejects external target with E0411, recovering on ; with E0199. Diagnostic: E0400, E0411, E0199.";
        doc /* \begin{aligned} \mathcal{P}_{\text{history}}=\mathcal{P}_{\text{shallow}}\cup\mathcal{P}_{\text{deep}},\quad \forall h\in\mathcal{P}_{\text{history}},\;\text{parent}(h)\in(\mathcal{S}_{\text{xor}}\cup\mathcal{S}_{\text{orth}}\cup\mathcal{R})\setminus\{s_{\text{root}}\} \\ \forall p,\;|\{h\in\mathcal{P}_{\text{shallow}}\mid\text{parent}(h)=p\}|\le 1\land |\{h\in\mathcal{P}_{\text{deep}}\mid\text{parent}(h)=p\}|\le 1 \\ \forall h\in\mathcal{P}_{\text{history}},\;|\{\tau\mid s_{\text{src}}(\tau)=h\}|=1\land s_{\text{tgt}}(T_{\text{default}}(h))\in\mathcal{C}_{\text{parent}(h)}\land C\cap\mathcal{P}_{\text{history}}=\emptyset \\ \mathcal{H}(S_{\text{comp}})\leftarrow C\cap\text{descendants}(S_{\text{comp}}),\quad (C\cap\mathcal{S}_{\text{final}}\ne\emptyset\implies\mathcal{H}(S_{\text{comp}})\leftarrow\emptyset) \\ C_{\text{restore}}(h_{\text{shallow}})=(\mathcal{H}=\emptyset\mathrel{?}\{s_{\text{def}}\}\cup\text{ResolveInitial}(s_{\text{def}}):\{s^*\}\cup\text{ResolveInitial}(s^*)) \\ C_{\text{restore}}(h_{\text{deep}})=(\mathcal{H}=\emptyset\mathrel{?}\{s_{\text{def}}\}\cup\text{ResolveInitial}(s_{\text{def}}):\mathcal{H}(S_{\text{comp}})),\quad \Delta_n=\text{UUIDv5}(\text{NS}_{\text{DEAP}},\text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0126_State_history_containment_default_transitions_and_configuration_restoration;
        require Invariant_REQ_0126_State_history_containment_default_transitions_and_configuration_restoration;
        verify by AC_01_Shallow_History_Restoration;
        verify by AC_02_Deep_History_Subtree_Restoration;
        verify by AC_03_Default_Transition_Execution;
        verify by AC_04_Structural_Malformation_Rejection;
        satisfy by StateMachineSolverEngine;
    }
    doc /* UUIDv5: 86365281-8ea9-5024-994b-72134612c093 | Complexity: Class P (O(m \cdot D) trace step generation) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 | Diagnostics: E0199, E0400, E0402 */
    requirement def REQ_0127_State_Machine_Simulation_Stepper {
        id = "REQ-0127";
        text = "The dynamics engine SHALL provide a deterministic stepper (SimulationStepper_Alpha) and trace logger (TraceLog_1) stepping statecharts against event traces.";
        attribute uuidv5 : String = "86365281-8ea9-5024-994b-72134612c093";
        attribute complexity_class : String = "Class P (O(m \cdot D) trace step generation)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119";
        attribute diagnostic_codes : String = "E0199, E0400, E0402";
        attribute ac_01_canonical_stepping_trace_generation : String = "Given: Package_0 declaring SimulationStepper_Alpha stepping State_Alpha State_Beta State_Gamma. When: Stepper processes event sequence e = Event_0}, Event_1} . Then: Completes 2 macro-steps, emitting TraceLog_1 with exact configurations and Status::Success. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_hierarchical_lca_exit_entry_action_ordering : String = "Given: Composite State_Comp_0 with State_Child_0, stepping from State_Alpha to State_Gamma. When: Stepping transitions across composite boundaries. Then: TraceLog_1 logs top-down entry and bottom-up exit, maintaining ancestor closure. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_guard_evaluation_fault_micro_step_limit_rejection : String = "Given: Invalid guard (param_x / 0.0 > 1.0) or cyclic -transitions. When: Evaluating candidate guards or completion micro-steps. Then: Traps fault or limit _{micro} > _{micro_max}}, emitting E0400 and binding _{step}. Diagnostic: E0400 (SimulationGuardEvaluationFault).";
        attribute ac_04_hierarchy_containment_violation_delimiter_recovery : String = "Given: Active configuration violating XOR singularity before ;, followed by a valid step. When: Validating configurations and performing delimiter recovery. Then: Emits E0402, binds _{state}, recovers on ;, halting at C_{max_errors}} with E0199. Diagnostic: E0402, E0199.";
        doc /* \begin{aligned} \mathcal{M}=\langle\mathcal{S},\mathcal{T},\mathcal{E},\mathcal{A},s_0,\mathcal{S}_{\text{final}},\sqsubset,\pi\rangle,\quad C=\langle\mathcal{S}_{\text{active}},\mathcal{V}_{\text{env}},\mathcal{Q}_{\text{events}},t_{\text{sim}},\kappa_{\text{step}}\rangle \\ C_0=\langle\text{InitActive}(s_0),\mathcal{V}_0,\mathcal{Q}_0,0.0,0\rangle,\quad s_0\in\mathcal{S}_{\text{active}}\land\text{ancestors}(s_0)\subseteq\mathcal{S}_{\text{active}} \\ \text{Enabled}(C,e)=\{T\in\mathcal{T}\mid s_{\text{src}}(T)\in\mathcal{S}_{\text{active}}\land e_{\text{trig}}(T)=e\land\text{Eval}(g_T,\mathcal{V}_{\text{env}})\equiv\text{true}\} \\ T^*=\arg\max_{T\in\text{Enabled}(C,e)}\langle\text{Depth}(s_{\text{src}}(T)),\pi(T),-\text{FQN}(T)\rangle \\ \mathcal{S}'_{\text{active}}=(\mathcal{S}_{\text{active}}\setminus\text{ExitPath}(s_{\text{src}}(T^*),s_{\text{tgt}}(T^*)))\cup\text{EntryPath}(s_{\text{src}}(T^*),s_{\text{tgt}}(T^*)) \\ \text{Step}(C,e)=\langle C',\sigma\rangle,\quad \sigma=\langle\kappa_{\text{step}},t_{\text{sim}},\mathcal{S}_{\text{active}},e,T^*,A_{\text{exec}},\mathcal{S}'_{\text{active}},\text{status}\rangle \\ \text{Trace}(\vec{e})=\langle\sigma_1,\dots,\sigma_m\rangle,\quad \vec{e}_1=\vec{e}_2\implies\mathcal{J}(\text{Trace}(\vec{e}_1))=\mathcal{J}(\text{Trace}(\vec{e}_2)) \\ (\text{Eval}(g_T,\mathcal{V}_{\text{env}})=\bot_{\text{eval}}\lor\kappa_{\text{micro}}\gt\kappa_{\text{micro\_max}})\implies\text{Diagnostic}(T,\text{E0400})\land\sigma\gets\bot_{\text{step}} \\ (\text{ancestors}(s)\not\subseteq\mathcal{S}_{\text{active}}\lor\text{LCA}(s_{\text{src}},s_{\text{tgt}})=\bot)\implies\text{Diagnostic}(s,\text{E0402})\land s\gets\bot_{\text{state}} \\ |\mathcal{D}_{\text{sink}}|\ge C_{\text{max\_errors}}\implies\text{Diagnostic}(\text{E0199}),\quad \Delta_\sigma=\text{UUIDv5}(\text{NS}_{\text{DEAP}},\text{FQN}(\mathcal{M})\mathbin{\Vert}\text{String}(\kappa)) \end{aligned} */
        constraint def Invariant_REQ_0127_Statechart_simulation_stepping_configuration_invariants_and_diagnostic_mappings_satisfy;
        require Invariant_REQ_0127_Statechart_simulation_stepping_configuration_invariants_and_diagnostic_mappings_satisfy;
        verify by AC_01_Canonical_Stepping_Trace_Generation;
        verify by AC_02_Hierarchical_LCA_Exit_Entry_Action_Ordering;
        verify by AC_03_Guard_Evaluation_Fault_Micro_Step_Limit_Rejection;
        verify by AC_04_Hierarchy_Containment_Violation_Delimiter_Recovery;
        satisfy by StateMachineSolverEngine;
    }
}
