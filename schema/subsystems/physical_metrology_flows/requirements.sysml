doc /* 7D Physical Metrology & Abstract Flow Conservation Networks */
package PhysicalMetrologyFlows {
    doc /* UUIDv5: a9dfc9c8-4af1-54e7-bdb7-b57797292b0f | Complexity: Class P (O(1) vector ops, O(N) lowering) | Standard: IEEE 29148-2018 / RFC 2119 / ISO 80000 | Diagnostics: E0199, E0300 */
    requirement def REQ_0098_Formal_mathbb_7_Rational_Vector_Space_over_SI_Base_Dimensions {
        id = "REQ-0098";
        text = "The metrology engine SHALL model physical dimensions as vector space (\mathbb{Q}^7,+,\cdot) over \mathbb{Q}. Canonical basis B_{SI} = \{e_M, \dots, e_J\} maps bijectively to the 7 SI base quantities (M, L, T, I, \Theta, N, J).";
        attribute uuidv5 : String = "a9dfc9c8-4af1-54e7-bdb7-b57797292b0f";
        attribute complexity_class : String = "Class P (O(1) vector ops, O(N) lowering)";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119 / ISO 80000";
        attribute diagnostic_codes : String = "E0199, E0300";
        attribute ac_01_canonical_basis_initialization : String = "Given: Compiler loading Package_0. When: Bootstrapping basis B_{SI} and dimensionless constants. Then: 7 base dimensions map to e_1 e_7 Q^7 and scalars map to 0. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_exact_vector_arithmetic_and_exponentiation : String = "Given: Quantities Quantity_1 ((1,1,-2,0,0,0,0)) and Quantity_2 ((0,1,0,0,0,0,0)) with k = 1/2. When: Lowering Quantity_1 * Quantity_2, Quantity_1 / Quantity_2, and Quantity_2^(1/2). Then: Dimensions evaluate to d_1 + d_2, d_1 - d_2, and k d_2. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_static_incommensurability_rejection : String = "Given: Expression param_x : Real [DimVector_Alpha] + param_y : Real [DimVector_Beta] with d_{ d_{. When: Dimensional type checking executes. Then: Lowering halts evaluation, assigns sentinel _{dim}, and emits E0300. Diagnostic: E0300 (DimensionalMismatch).";
        attribute ac_04_invalid_denominator_rejection_and_delimiter_recovery : String = "Given: Malformed DimVector_Alpha (q_i = 0) preceding valid DimVector_Beta. When: Delimiter recovery executes. Then: Ingestion emits E0300, recovers on delimiter, and halts at C_{max_errors}} with E0199. Diagnostic: E0300, E0199.";
        doc /* \begin{aligned} \mathcal{B}_{\text{SI}} &= \{\mathbf{e}_M, \dots, \mathbf{e}_J\} \subset \mathbb{Q}^7,\; \mathbf{d} = \sum_{i=1}^7 d_i \mathbf{e}_i,\; d_i = \frac{p_i}{q_i},\; \gcd(|p_i|, q_i) = 1 \\ \mathbf{d}(Q_1 \cdot Q_2) &= \mathbf{d}(Q_1) + \mathbf{d}(Q_2),\; \mathbf{d}(Q_1 / Q_2) = \mathbf{d}(Q_1) - \mathbf{d}(Q_2),\; \mathbf{d}(Q^k) = k \cdot \mathbf{d}(Q) \\ \Phi_{\text{homogenous}} &\iff \mathbf{d}(A) = \mathbf{d}(B),\; \neg \Phi \wedge (A \pm B \vee A \bowtie B) \implies \text{Diagnostic}(n, \text{E0300}) \\ q_i \le 0 &\implies \text{Diagnostic}(D, \text{E0300}),\; |\mathcal{D}_{\text{sink}}| \ge C_{\text{max\_errors}} \implies \text{Diagnostic}(D_{\text{term}}, \text{E0199}) \\ \Delta_D &= \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(D)) \end{aligned} */
        constraint def Invariant_REQ_0098_Canonical_basis_vector_homomorphisms_homogeneity_predicates_and_diagnostic_emission_rules;
        require Invariant_REQ_0098_Canonical_basis_vector_homomorphisms_homogeneity_predicates_and_diagnostic_emission_rules;
        verify by AC_01_Canonical_Basis_Initialization;
        verify by AC_02_Exact_Vector_Arithmetic_and_Exponentiation;
        verify by AC_03_Static_Incommensurability_Rejection;
        verify by AC_04_Invalid_Denominator_Rejection_and_Delimiter_Recovery;
        satisfy by MetrologyFlowEngine;
    }
    doc /* UUIDv5: 83cab42d-eb92-56bf-91cd-0f4bc1c7a675 | Complexity: Class P (O(1) coordinate check, O(N) lowering) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / ISO 80000-1 | Diagnostics: E0199, E0300, E0301 (per REQ-0191) */
    requirement def REQ_0099_Base_Dimension_Tuple_Representation {
        id = "REQ-0099";
        text = "The engine SHALL represent dimensions as irreducible rational 7-tuples over \mathbb{Q}^7 spanning ISQ base dimensions (M, L, T, I, \Theta, N, J).";
        attribute uuidv5 : String = "83cab42d-eb92-56bf-91cd-0f4bc1c7a675";
        attribute complexity_class : String = "Class P (O(1) coordinate check, O(N) lowering)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / ISO 80000-1";
        attribute diagnostic_codes : String = "E0199, E0300, E0301 (per REQ-0191)";
        attribute ac_01_canonical_rational_reduction_zero_identity : String = "Given: Rational coordinates (2/4,6/2,-9/3,0/1,0/1,0/1,0/1) and dimensionless scalar. When: Lowering dimension AST nodes. Then: Engine reduces coordinates to ((1,2),(3,1),(-3,1),(0,1),(0,1),(0,1),(0,1)) and maps scalar to 0. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_conservative_conjugate_power_pair_conformance : String = "Given: Interface Interface_Gamma with Across Across_Var and Through Through_Var. When: Validating conjugate pairs. Then: Dimensional sum evaluates to d_{Power} = (1,2,-3,0,0,0,0) with valid flag set true. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_conjugate_power_product_violation_rejection : String = "Given: Interface Interface_Gamma with Across (1,1,-2,0,0,0,0) and Through (0,0,1,0,0,0,0). When: Evaluating power invariant (d_e + d_f d_{Power}). Then: Lowering rejects definition with E0301 and assigns sentinel _{pair}. Diagnostic: E0301.";
        attribute ac_04_static_homogeneity_delimiter_recovery : String = "Given: Denominator q = 0 or mismatched addition param_x + param_y. When: Parsing and validation execute. Then: Compiler emits E0300, assigns sentinel _{dim}, recovers on delimiter (;), and halts at C_{max_errors}} with E0199. Diagnostic: E0300, E0199.";
        doc /* \begin{aligned} \mathbf{d} &\in \mathbb{Q}^7,\; d_i = p_i/q_i,\; q_i \in \mathbb{Z}^+,\; \gcd(|p_i|, q_i) = 1,\; \mathbf{0} = (0,\dots,0) \\ \mathbf{d}_{\text{Power}} &= (1,2,-3,0,0,0,0),\; \forall C = \langle e, f \rangle \in \mathcal{C}_{\text{pair}} \implies \mathbf{d}(e) + \mathbf{d}(f) \equiv \mathbf{d}_{\text{Power}} \\ \mathbf{d}(e) + \mathbf{d}(f) &\ne \mathbf{d}_{\text{Power}} \implies \text{Diagnostic}(C, \text{E0301}) \\ q_i \le 0 &\vee (\mathbf{d}(Q_1) \ne \mathbf{d}(Q_2) \wedge Q_1 \pm Q_2) \implies \text{Diagnostic}(n, \text{E0300}) \\ \Delta_n &= \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0099_Lattice_invariants_power_duality_and_diagnostics;
        require Invariant_REQ_0099_Lattice_invariants_power_duality_and_diagnostics;
        derived from REQ_0106_Conservative_vs_Non_Conservative_Flow_Network_Classification;
        verify by AC_01_Canonical_Rational_Reduction_Zero_Identity;
        verify by AC_02_Conservative_Conjugate_Power_Pair_Conformance;
        verify by AC_03_Conjugate_Power_Product_Violation_Rejection;
        verify by AC_04_Static_Homogeneity_Delimiter_Recovery;
        satisfy by MetrologyFlowEngine;
    }
    doc /* UUIDv5: 607d243a-c844-554f-ab38-c90e43262629 | Complexity: Class P (O(K + V + E)) | Standard: IEEE 29148-2018 / RFC 2119 / Tellegen | Diagnostics: E0199, E0300, E0301 (per REQ-0191) */
    requirement def REQ_0100_Exact_Rational_Exponent_Arithmetic {
        id = "REQ-0100";
        text = "The compiler SHALL execute dimensional algebra over \mathbb{Q} without floating-point arithmetic, synthesizing flow continuity (\sum f = 0) and loop conservation (\sum \Delta e = 0).";
        attribute uuidv5 : String = "607d243a-c844-554f-ab38-c90e43262629";
        attribute complexity_class : String = "Class P (O(K + V + E))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119 / Tellegen";
        attribute diagnostic_codes : String = "E0199, E0300, E0301 (per REQ-0191)";
        attribute ac_01_exact_rational_exponent_arithmetic_scaling : String = "Given: Expressions Q_1 : Mass^{1/2} and Q_2 : Mass^{1/3} in Package_0. When: Lowering computes Q_1 Q_2 and (Q_1)^{2/3}. Then: Exponents in Q^7 evaluate to 5/6 and 1/3 without drift. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_zero_denominator_incommensurable_dimensions : String = "Given: Exponent 3/0 or junction with mismatched flow dimensions. When: Dimensional checking executes. Then: Engine halts, assigns sentinel _{dim}, emitting E0300. Diagnostic: E0300.";
        attribute ac_03_sparse_incidence_matrix_continuity_synthesis : String = "Given: Network Network_Alpha with 4 junctions and 5 branches in Package_0. When: Constraint synthesis lowers topology. Then: Matrix A lowers and A f = 0 synthesizes. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_04_static_conservation_violation_recovery : String = "Given: Unbalanced static flow preceding delimiter (;) and valid flow. When: Semantic evaluation executes with error recovery. Then: Engine emits E0301, resynchronizes on delimiter, emitting E0199 at C_{max_errors}}. Diagnostic: E0301, E0199.";
        doc /* \begin{aligned} \forall \mathbf{d} \in \mathbb{Q}^7,\; &\mathbf{d} = (q_1, \dots, q_7),\; q_i \in \mathbb{Q},\; \mathbf{d}(Q_1 Q_2) = \mathbf{d}_1 + \mathbf{d}_2,\; \mathbf{d}(Q^k) = k \cdot \mathbf{d} \\ \mathbf{A} \mathbf{f} &= \mathbf{0} \iff \forall J_i \in \mathcal{V}_{\text{junc}},\; \sum_{e \in \text{Incident}(J_i)} A_{i, e} f_e = 0 \\ \mathbf{B} \mathbf{\Delta e} &= \mathbf{0} \iff \forall \mathcal{L}_k \in \mathcal{C},\; \sum_{e \in \text{Edges}(\mathcal{L}_k)} B_{k, e} \Delta e_e = 0 \\ \mathbf{A} \mathbf{B}^T &= \mathbf{0},\; \mathbf{f}^T \mathbf{\Delta e} = 0,\; \Delta_n = \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0100_Rational_arithmetic_and_topological_conservation_invariants;
        require Invariant_REQ_0100_Rational_arithmetic_and_topological_conservation_invariants;
        verify by AC_01_Exact_Rational_Exponent_Arithmetic_Scaling;
        verify by AC_02_Zero_Denominator_Incommensurable_Dimensions;
        verify by AC_03_Sparse_Incidence_Matrix_Continuity_Synthesis;
        verify by AC_04_Static_Conservation_Violation_Recovery;
        satisfy by MetrologyFlowEngine;
    }
    doc /* UUIDv5: 28881e44-527b-53f2-a928-fa9ff7b76349 | Complexity: Class P (O(1) vector checks, O(N) lowering) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / ISO 80000-1 | Diagnostics: E0199, E0300 */
    requirement def REQ_0101_Static_Dimensional_Homogeneity_Validation_for_Additive_Expressions {
        id = "REQ-0101";
        text = "The compiler SHALL validate dimensional homogeneity on additive (A\pm B), relational (A\sim B), and extremum (\min,\max) expressions in \mathbb{Q}^7.";
        attribute uuidv5 : String = "28881e44-527b-53f2-a928-fa9ff7b76349";
        attribute complexity_class : String = "Class P (O(1) vector checks, O(N) lowering)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / ISO 80000-1";
        attribute diagnostic_codes : String = "E0199, E0300";
        attribute ac_01_additive_homogeneity_and_dimension_inheritance : String = "Given: Operands Term_1 and Term_2 of dimension (1,1,-2,0,0,0,0)Q^7. When: Lowering Term_1 + Term_2 or Term_1 - Term_2. Then: Validation inherits operand dimension (1,1,-2,0,0,0,0). Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_commensurable_unit_conversion_synthesis : String = "Given: Operands Term_1 and Term_2 of dimension (0,1,0,0,0,0,0) in Unit_Alpha (c=1) and Unit_Beta (c=1/1000). When: Lowering Term_1 + Term_2. Then: Compiler synthesizes node scaling Term_2 by 1/1000Q^+ in unit Unit_Alpha. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_incommensurable_dimensional_mismatch_rejection : String = "Given: Operands Term_1 (Force) and Term_2 (Energy) with mismatched dimensions. When: Lowering Term_1 + Term_2 or Term_1 < Term_2. Then: Rejection emits E0300, isolating mismatched length coordinate, and assigns sentinel _{dim}. Diagnostic: E0300.";
        attribute ac_04_affine_interval_addition_rejection_and_delimiter_recovery : String = "Given: Sum Term_1 + Term_2 of interval quantities (o_1=o_2=5463/20) before valid statement. When: Semantic validation and delimiter recovery execute. Then: Rejection emits E0300, recovers on delimiter, and halts at C_{max_errors}} with E0199. Diagnostic: E0300, E0199.";
        doc /* \begin{aligned} \mathbf{d}_A\equiv\mathbf{d}_B &\iff \forall i \in \{1,\dots,7\},\; \mathbf{d}_A[i]=\mathbf{d}_B[i] \\ E=A\pm B &\implies (\mathbf{d}_A\equiv\mathbf{d}_B \implies \mathbf{d}_E=\mathbf{d}_A) \wedge (\mathbf{d}_A\not\equiv\mathbf{d}_B \implies \text{Diagnostic}(E,\text{E0300})) \\ E=A\sim B &\implies (\mathbf{d}_A\equiv\mathbf{d}_B \implies \mathbf{d}_E=\mathbf{0}) \wedge (\mathbf{d}_A\not\equiv\mathbf{d}_B \implies \text{Diagnostic}(E,\text{E0300})) \\ u_A\ne u_B &\implies \text{convert}(B,u_A)=(c_B/c_A)\cdot B \\ \Delta_E=\text{UUIDv5}(\text{NS}_{\text{DEAP}},&\text{FQN}(E)),\; \mathbf{d}_A=\bot_{\text{dim}} \implies \mathbf{d}_E=\bot_{\text{dim}} \end{aligned} */
        constraint def Invariant_REQ_0101_Dimensional_equivalence_additive_preservation_conversion_synthesis_and_diagnostic_rules;
        require Invariant_REQ_0101_Dimensional_equivalence_additive_preservation_conversion_synthesis_and_diagnostic_rules;
        verify by AC_01_Additive_Homogeneity_and_Dimension_Inheritance;
        verify by AC_02_Commensurable_Unit_Conversion_Synthesis;
        verify by AC_03_Incommensurable_Dimensional_Mismatch_Rejection;
        verify by AC_04_Affine_Interval_Addition_Rejection_and_Delimiter_Recovery;
        satisfy by MetrologyFlowEngine;
    }
    doc /* UUIDv5: 87075115-ccc2-5dc4-a270-f69dd24927aa | Complexity: Class P (O(1) arithmetic, O(N) AST lowering) | Standard: IEEE 29148-2018 / RFC 2119 / ISO 80000-1 | Diagnostics: E0199, E0300 (per REQ-0191) */
    requirement def REQ_0102_Multiplicative_Dimensional_Exponent_Addition {
        id = "REQ-0102";
        text = "The metrology engine SHALL compute and validate dimension vectors for multiplication (A \cdot B), division (A / B), and reciprocal (1 / B) via vector addition, subtraction, and negation in \mathbb{Q}^7.";
        attribute uuidv5 : String = "87075115-ccc2-5dc4-a270-f69dd24927aa";
        attribute complexity_class : String = "Class P (O(1) arithmetic, O(N) AST lowering)";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119 / ISO 80000-1";
        attribute diagnostic_codes : String = "E0199, E0300 (per REQ-0191)";
        attribute ac_01_multiplicative_exponent_addition_scale_composition : String = "Given: Quantities Quantity_Alpha (dimension e_M + e_L - 2e_T, scale c_1) and Quantity_Beta (dimension e_L, scale c_2). When: Lowering multiplication Expr_Alpha = Quantity_Alpha * Quantity_Beta. Then: Dimension evaluates to e_M + 2e_L - 2e_T with scale c_1 c_2, offset o = 0, and empty diagnostic sink. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_division_exponent_subtraction_dimensional_cancellation : String = "Given: Quantities Factor_1 (dimension e_L, scale 1000) and Factor_2 (dimension e_L, scale 1). When: Lowering division Expr_Alpha = Factor_1 / Factor_2. Then: Dimension evaluates to identity 0 with scale 1000 and empty diagnostic sink. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_affine_unit_rejection : String = "Given: Expression Expr_Alpha = param_x * param_y where param_x has affine unit Unit_Alpha (o = 273.15). When: Lowering and metrology type checking execute. Then: Expression is rejected with diagnostic E0300 and assigns sentinel _{dim}. Diagnostic: E0300 (Affine unit in multiplicative expression).";
        attribute ac_04_compile_time_division_by_zero_delimiter_recovery : String = "Given: Invalid division Expr_Fault = Factor_1 / 0.0 followed by ; and valid Expr_Beta = Factor_1 * Factor_2. When: Delimiter error recovery executes. Then: Lowering emits E0300, binds sentinel _{dim}, recovers on ;, parses Expr_Beta, and halts at C_{max_errors}} with E0199. Diagnostic: E0300, E0199.";
        doc /* \begin{aligned} \mathcal{D}_{\mathbb{Q}} = (\mathbb{Q}^7,+,-,\mathbf{0}),\quad \mathcal{B} &= \{[M],[L],[T],[I],[\Theta],[N],[J]\} \\ \forall e \in \mathcal{E}_{\text{expr}},\quad \tau(e) &= \langle \mathbf{d}(e),c(e),o(e) \rangle \in \mathbb{Q}^7 \times \mathbb{Q}^+ \times \mathbb{Q} \\ \mathbf{d}(A \cdot B) = \mathbf{d}(A)+\mathbf{d}(B),\quad c(A \cdot B) &= c(A) \cdot c(B),\quad o(A \cdot B) = 0 \\ \mathbf{d}(A / B) = \mathbf{d}(A)-\mathbf{d}(B),\quad c(A / B) &= \frac{c(A)}{c(B)},\quad o(A / B) = 0 \\ \mathbf{d}(1 / B) = -\mathbf{d}(B),\quad c(1 / B) &= \frac{1}{c(B)},\quad o(1 / B) = 0 \\ \mathbf{d}(A) \equiv \mathbf{d}(B) \implies \mathbf{d}(A / B) = \mathbf{0},\quad &(o(A) \ne 0 \vee o(B) \ne 0) \implies \text{Diagnostic}(e,\text{E0300}) \\ \text{is\_zero}(B) \implies \text{Diagnostic}(e,\text{E0300}) &\wedge \mathbf{d}(e) = \bot_{\text{dim}},\quad \Delta_n = \text{UUIDv5}(\text{NS}_{\text{DEAP}},\text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0102_Metrological_signature_spaces_vector_arithmetic_scale_factors_and_error_predicates;
        require Invariant_REQ_0102_Metrological_signature_spaces_vector_arithmetic_scale_factors_and_error_predicates;
        verify by AC_01_Multiplicative_Exponent_Addition_Scale_Composition;
        verify by AC_02_Division_Exponent_Subtraction_Dimensional_Cancellation;
        verify by AC_03_Affine_Unit_Rejection;
        verify by AC_04_Compile_Time_Division_by_Zero_Delimiter_Recovery;
        satisfy by MetrologyFlowEngine;
    }
    doc /* UUIDv5: 997a9b42-a403-53ea-b33b-a688089e8f42 | Complexity: Class P (O(1) coordinate check, O(V) AST traversal) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / ISO 80000-1 §3.8 | Diagnostics: E0199, E0300 */
    requirement def REQ_0103_Dimensionless_Constraint_Enforcement_for_Transcendental_Arguments {
        id = "REQ-0103";
        text = "The metrology engine and type checker of Subsystem 5 SHALL enforce that all arguments to transcendental and trigonometric functions evaluate strictly to the dimensionless null vector 0 \in \mathbb{Q}^7 over the seven ISQ base dimensions. Evaluated results MUST likewise yield d_{result} = 0 \in";
        attribute uuidv5 : String = "997a9b42-a403-53ea-b33b-a688089e8f42";
        attribute complexity_class : String = "Class P (O(1) coordinate check, O(V) AST traversal)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / ISO 80000-1 §3.8";
        attribute diagnostic_codes : String = "E0199, E0300";
        attribute ac_01_dimensionless_transcendental_evaluation : String = "Given: Expressions invoking , , , with dimensionless arguments (d = 0). When: Metrology type-checking executes. Then: Invocations pass verification with d = 0 and empty diagnostic sink. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_commensurable_multi_argument_resolution : String = "Given: Invocation atan2(E_y, E_x) with length arguments (d(E_y) = d(E_x) = [L]) and (param_theta}) with unit rad. When: Commensurability checking and angle conversion execute. Then: Commensurability holds, angle converts to radians, result yields d = 0, and sink remains empty. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_non_dimensionless_argument_rejection : String = "Given: Expression sin(Quantity_Alpha) where d(Quantity_Alpha}) = [L]. When: Metrology dimensional type checker evaluates argument dimension. Then: Compiler detects d 0, halts lowering, binds sentinel _{dim}, and emits E0300. Diagnostic: E0300.";
        attribute ac_04_incommensurable_operands_delimiter_recovery : String = "Given: Malformed atan2(E_y, E_x) (d(E_y) = [L], d(E_x) = [T]) preceding valid param_x : Real = cos(0.0);. When: Semantic validation and delimiter recovery execute. Then: Compiler emits E0300, recovers on delimiter (;), verifies valid cos expression, and emits E0199 at C_{max_errors}}. Diagnostic: E0300, E0199.";
        doc /* \begin{aligned} \mathbf{d} &: \mathcal{E} \to \mathbb{Q}^7 \cup \{ \bot_{\text{dim}} \}, \quad \mathbf{0} = (0, 0, 0, 0, 0, 0, 0) \in \mathbb{Q}^7 \\ \forall f \in \mathcal{F}_{\text{trans}}, \; n = \langle f, E \rangle \in \mathcal{E} &\implies (\mathbf{d}(E) = \mathbf{0} \land \mathbf{d}(n) = \mathbf{0}) \\ n = \langle \text{atan2}, E_y, E_x \rangle \in \mathcal{E} &\implies (\mathbf{d}(E_y) = \mathbf{d}(E_x) \land \mathbf{d}(n) = \mathbf{0}) \\ \forall k \ge 1, \; \mathbf{d}(c_k x^k) = \mathbf{0} &\iff k \cdot \mathbf{d}(x) = \mathbf{0} \iff \mathbf{d}(x) = \mathbf{0} \\ \mathbf{d}(E) \ne \mathbf{0} \lor \mathbf{d}(E_y) \ne \mathbf{d}(E_x) &\implies \text{Diagnostic}(n, \text{E0300}) \in \mathcal{D}_{\text{sink}} \\ \forall n \in \mathcal{V}_{\text{AST}}, \quad \Delta_n &= \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0103_Transcendental_argument_dimensionless_nullity_multi_argument_commensurability_and_UUIDv5_invariants;
        require Invariant_REQ_0103_Transcendental_argument_dimensionless_nullity_multi_argument_commensurability_and_UUIDv5_invariants;
        verify by AC_01_Dimensionless_Transcendental_Evaluation;
        verify by AC_02_Commensurable_Multi_Argument_Resolution;
        verify by AC_03_Non_Dimensionless_Argument_Rejection;
        verify by AC_04_Incommensurable_Operands_Delimiter_Recovery;
        satisfy by MetrologyFlowEngine;
    }
    doc /* UUIDv5: 6b419bc3-0e70-538d-a5b7-afddf4582acd | Complexity: Class P (O(1) rational scalar multiplication, O(V) AST lowering) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / ISO 80000-1 §3.8 | Diagnostics: E0199, E0300 (per REQ-0191) */
    requirement def REQ_0104_Rational_Power_Evaluation_for_Fractional_Exponents {
        id = "REQ-0104";
        text = "The engine SHALL compute and validate 7D dimension vectors and scale factors for rational power expressions (A^k, \operatorname{pow}(A, k), \operatorname{sqrt}(A), \operatorname{cbrt}(A), \operatorname{root}(A, n)) with base dimension d_A \in \mathbb{Q}^7 and rational exponent k = p/q \in";
        attribute uuidv5 : String = "6b419bc3-0e70-538d-a5b7-afddf4582acd";
        attribute complexity_class : String = "Class P (O(1) rational scalar multiplication, O(V) AST lowering)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / ISO 80000-1 §3.8";
        attribute diagnostic_codes : String = "E0199, E0300 (per REQ-0191)";
        attribute ac_01_rational_exponent_multiplication_radical_lowering : String = "Given: Base Base_Alpha ((0, 2, 0, 0, 0, 0, 0)) and scale c = 4. When: Lowering Base_Alpha ^ (3/2) and sqrt(Base_Alpha). Then: Dimensions evaluate to (0, 3, 0, 0, 0, 0, 0) with scale 8 and (0, 1, 0, 0, 0, 0, 0) with scale 2. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_zero_exponent_identity_dimensionless_invariance : String = "Given: Quantity Quantity_Alpha ((1, 1, -2, 0, 0, 0, 0)) and parameter param_x : Real (0). When: Lowering Quantity_Alpha ^ 0 and param_x ^ (5/3). Then: Both evaluate to null vector 0 Q^7 with scale 1. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_affine_unit_non_rational_exponent_rejection : String = "Given: Expression Base_Alpha ^ 2 with affine unit (o = 273.15) and Quantity_Alpha ^ param_y. When: Metrology type checking executes. Then: Rejection emits E0300 and assigns sentinel _{dim}. Diagnostic: E0300.";
        attribute ac_04_domain_singularity_delimiter_recovery : String = "Given: Expression Base_Alpha ^ (1 / 0) before valid Quantity_Beta = Base_Alpha ^ 2;. When: Validation and delimiter recovery execute. Then: Rejection emits E0300, assigns sentinel _{dim}, recovers on ;, and halts at C_{max_errors}} with E0199. Diagnostic: E0300, E0199.";
        doc /* \begin{aligned} \mathbf{d} &\in \mathbb{Q}^7,\; \mathbf{0} = (0,\dots,0),\; n = \langle \text{Pow}, A, p/q \rangle \implies \mathbf{d}(n) = (p/q) \cdot \mathbf{d}(A) \\ \mathbf{d}(\operatorname{sqrt}(A)) &= \frac{1}{2}\mathbf{d}(A),\; \mathbf{d}(\operatorname{cbrt}(A)) = \frac{1}{3}\mathbf{d}(A),\; \mathbf{d}(\operatorname{root}(A, m)) = \frac{1}{m}\mathbf{d}(A) \\ \mathbf{d}(A^0) &= \mathbf{0},\; \mathbf{d}((A^k)^m) = (km)\mathbf{d}(A),\; c(A^k) = c_A^k \; (k \in \mathbb{Z}) \\ (o(A) \ne 0 \wedge k \ne 1) &\vee (q = 0) \vee (\mathbf{d}_A \ne \mathbf{0} \wedge E_{\text{exp}} \notin \mathbb{Q}) \implies \text{Diagnostic}(n, \text{E0300}) \wedge \mathbf{d}(n) = \bot_{\text{dim}} \\ \Delta_n &= \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0104_Rational_power_evaluation_radical_lowering_and_diagnostic_rules;
        require Invariant_REQ_0104_Rational_power_evaluation_radical_lowering_and_diagnostic_rules;
        verify by AC_01_Rational_Exponent_Multiplication_Radical_Lowering;
        verify by AC_02_Zero_Exponent_Identity_Dimensionless_Invariance;
        verify by AC_03_Affine_Unit_Non_Rational_Exponent_Rejection;
        verify by AC_04_Domain_Singularity_Delimiter_Recovery;
        satisfy by MetrologyFlowEngine;
    }
    doc /* UUIDv5: 99116ca2-c69c-5b91-a748-9bdab8866e75 | Complexity: Class NP-complete} (LCP) / Class P (O(V^3+\kappa_{pivot} m^2)) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 | Diagnostics: E0199, E0300, E0301, E0399 */
    requirement def REQ_0105_Switched {
        id = "REQ-0105";
        text = "The compiler SHALL partition flow networks into continuous linear subgraphs solved via sparse LU and hybrid switched junctions formulated as a Linear Complementarity Problem (LCP), solved via Lemke pivoting bounded by iteration cutoff \kappa_{pivot}.";
        attribute uuidv5 : String = "99116ca2-c69c-5b91-a748-9bdab8866e75";
        attribute complexity_class : String = "Class NP-complete} (LCP) / Class P (O(V^3+\kappa_{pivot} m^2))";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119";
        attribute diagnostic_codes : String = "E0199, E0300, E0301, E0399";
        attribute ac_01_continuous_linear_subgraph_sparse_lu_factorization : String = "Given: Network FlowNetwork_Alpha in Package_0 with 12 junctions, 18 branches, and grounded Junction_1. When: Sparse LU solves continuous branches. Then: System K_{red}e=f_{ext} solves with residual |K_{red}e-f_{ext}|_2 10^{-12}. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_hybrid_switched_junction_complementarity_lemke_pivot_convergence : String = "Given: Network FlowNetwork_Alpha with 4 switched branches, matrix M 0, and cutoff _{pivot}=64. When: Lemke algorithm solves canonical LCP(M,q). Then: Solver terminates with z_0=0 in N_{pivot}_{pivot}, with w0,z0 and w^Tz=0. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_singular_conductance_matrix_rejection : String = "Given: Network FlowNetwork_Alpha with ungrounded junctions yielding rank(K_{red}) n-1. When: Conductance matrix factorization executes. Then: Lowering halts, assigns sentinel _{flow}, and emits E0301. Diagnostic: E0301.";
        attribute ac_04_pivot_iteration_limit_exceeded_delimiter_recovery : String = "Given: Cyclic network FlowNetwork_Alpha with cutoff _{pivot}=2 requiring 6 pivots, preceding ; and valid Classifier_Beta. When: Lemke solving exceeds cutoff (N_{pivot}_{pivot}). Then: Compiler records E0399, assigns _{lcp}, recovers on ;, parses Classifier_Beta, emitting E0199 at C_{max_errors}}. Diagnostic: E0399, E0199.";
        doc /* \begin{aligned} \mathcal{E}_{\text{flow}}&=\mathcal{E}_{\text{lin}}\cup\mathcal{E}_{\text{switch}},\;\mathcal{E}_{\text{lin}}\cap\mathcal{E}_{\text{switch}}=\emptyset,\;\mathbf{K}_{\text{red}}\mathbf{e}=\mathbf{f}_{\text{ext}},\;\mathbf{P}\mathbf{K}_{\text{red}}\mathbf{Q}=\mathbf{L}\mathbf{U} \\ \mathbf{w}-\mathbf{M}\mathbf{z}&=\mathbf{q},\;\mathbf{w}\ge\mathbf{0},\;\mathbf{z}\ge\mathbf{0},\;\mathbf{w}^T\mathbf{z}=0,\;\mathbf{M}=\mathbf{A}_{\text{switch}}^T\mathbf{K}_{\text{red}}^{-1}\mathbf{A}_{\text{switch}} \\ \mathbf{w}-\mathbf{M}\mathbf{z}-\mathbf{d}_{\text{aux}}z_0&=\mathbf{q},\;N_{\text{pivot}}\le\kappa_{\text{pivot}},\;z_0=0\implies(\mathbf{w}^*,\mathbf{z}^*)\in\operatorname{SOL}(\mathbf{M},\mathbf{q}) \\ \text{rank}(\mathbf{K}_{\text{red}})\lt n-1&\implies\text{Diagnostic}(J,\text{E0301}),\;N_{\text{pivot}}\gt\kappa_{\text{pivot}}\implies\text{Diagnostic}(S,\text{E0399}) \\ \forall n\in\mathcal{V}_{\text{AST}},\;\Delta_n&=\text{UUIDv5}(\text{NS}_{\text{DEAP}},\text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0105_Topological_partitioning_continuous_conductance_canonical_LCP_and_dimensional_invariants;
        require Invariant_REQ_0105_Topological_partitioning_continuous_conductance_canonical_LCP_and_dimensional_invariants;
        verify by AC_01_Continuous_Linear_Subgraph_Sparse_LU_Factorization;
        verify by AC_02_Hybrid_Switched_Junction_Complementarity_Lemke_Pivot_Convergence;
        verify by AC_03_Singular_Conductance_Matrix_Rejection;
        verify by AC_04_Pivot_Iteration_Limit_Exceeded_Delimiter_Recovery;
        satisfy by MetrologyFlowEngine;
    }
    doc /* UUIDv5: 7de336d6-2dbc-5cb3-9032-ddbc1e39e163 | Complexity: Class P (O(N_F) classification, O(N_V + N_F) routing) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.15 | Diagnostics: E0199, E0201, E0300 */
    requirement def REQ_0106_Conservative_vs_Non_Conservative_Flow_Network_Classification {
        id = "REQ-0106";
        text = "The flow solver SHALL classify all connections f \in F_{flow} into Conservative (F_{cons}) or Non-Conservative (F_{noncons}) flows.";
        attribute uuidv5 : String = "7de336d6-2dbc-5cb3-9032-ddbc1e39e163";
        attribute complexity_class : String = "Class P (O(N_F) classification, O(N_V + N_F) routing)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.15";
        attribute diagnostic_codes : String = "E0199, E0201, E0300";
        attribute ac_01_conservative_flow_classification : String = "Given: Acausal ports Port_1 and Port_2 linked by Flow_A with d_e = (1,-1,-2,0,0,0,0) and d_f = (0,3,-1,0,0,0,0). When: Lowering validates conjugate dimensions. Then: d_e + d_f d_{power} holds and Flow_A enters F_{cons}. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_non_conservative_signal_flow : String = "Given: Output Port_1 driving inputs Port_2, Port_3, Port_4 with scalar param_x : Real. When: Lowering builds signal routing. Then: Connections enter F_{noncons} with unattenuated broadcast. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_incompatible_hybrid_connection_rejection : String = "Given: Direct flow between conservative and non-conservative ports, and flow with d_e+d_f = (1,1,-2,0,0,0,0) d_{power}. When: Lowering checks flow compatibility. Then: Rejection emits E0300 and binds sentinel _{class}. Diagnostic: E0300.";
        attribute ac_04_multiple_driver_conflict_rejection : String = "Given: Two non-conservative outputs directly driving one input without arbitration. When: Lowering audits causal dependencies in G_{causal}. Then: Rejection emits E0201 for multiple drivers on the receiver port. Diagnostic: E0201.";
        doc /* \begin{aligned} \mathcal{F}_{\text{flow}} &= \mathcal{F}_{\text{cons}} \cup \mathcal{F}_{\text{noncons}}, \quad \mathcal{F}_{\text{cons}} \cap \mathcal{F}_{\text{noncons}} = \emptyset \\ \mathbf{d}_e+\mathbf{d}_f &\equiv \mathbf{d}_{\text{power}} = (1,2,-3,0,0,0,0) \in \mathbb{Q}^7 \\ \kappa(f) = \text{FlowClass}_{\text{cons}} &\iff \text{kind}(p_{\text{src}}) = \text{kind}(p_{\text{tgt}}) = \text{PortKind}_{\text{cons}} \wedge (\mathbf{d}_e+\mathbf{d}_f = \mathbf{d}_{\text{power}}) \\ \kappa(f) = \text{FlowClass}_{\text{noncons}} &\iff \text{kind}(p_{\text{src}}) = \text{PortKind}_{\text{out}} \wedge \text{kind}(p_{\text{tgt}}) = \text{PortKind}_{\text{in}} \\ \mathbf{A}_{\text{cons}}\mathbf{f}_{\text{through}} &= \mathbf{0}, \quad \mathbf{B}_{\text{cons}}\mathbf{\Delta e} = \mathbf{0}, \quad \mathbf{A}_{\text{cons}}\mathbf{B}_{\text{cons}}^T = \mathbf{0} \\ \forall f \in \mathcal{F}_{\text{flow}}, \quad &\Delta_f = \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(f)) \end{aligned} */
        constraint def Invariant_REQ_0106_Disjoint_flow_partition_conjugate_power_and_network_conservation_laws;
        require Invariant_REQ_0106_Disjoint_flow_partition_conjugate_power_and_network_conservation_laws;
        verify by AC_01_Conservative_Flow_Classification;
        verify by AC_02_Non_Conservative_Signal_Flow;
        verify by AC_03_Incompatible_Hybrid_Connection_Rejection;
        verify by AC_04_Multiple_Driver_Conflict_Rejection;
        satisfy by MetrologyFlowEngine;
    }
    doc /* UUIDv5: 9f4cf573-ed75-56fd-8516-8c0c1af2168d | Complexity: Class P (O(1) vector checks, O(N) lowering) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / ISO 80000-1 | Diagnostics: E0199, E0300, E0301 */
    requirement def REQ_0107_Generalized_Conjugate_Variable_Pair_Modeling {
        id = "REQ-0107";
        text = "The metrology engine SHALL validate conservative interfaces using dual conjugate variable pairs (e, f) whose dimensional product evaluates to universal Power d_{Power} = (1, 2, -3, 0, 0, 0, 0) \in \mathbb{Q}^7.";
        attribute uuidv5 : String = "9f4cf573-ed75-56fd-8516-8c0c1af2168d";
        attribute complexity_class : String = "Class P (O(1) vector checks, O(N) lowering)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / ISO 80000-1";
        attribute diagnostic_codes : String = "E0199, E0300, E0301";
        attribute ac_01_canonical_power_invariant_conjugate_lowering : String = "Given: Port Port_Alpha with Effort_Var ((1,2,-3,-1,0,0,0)) and Flow_Var ((0,0,0,1,0,0,0)). When: Lowering checks dimensional product. Then: Sum evaluates to d_{Power} with empty diagnostic sink. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_conservative_interconnect_compatibility : String = "Given: Ports Port_Alpha and Port_Beta sharing identical effort and flow dimensions. When: Interconnect validation and continuity synthesis execute. Then: Dimensions match, f_{Port_Alpha}} + f_{Port_Beta}} = 0 holds, and sink remains empty. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_conjugate_power_product_violation_rejection : String = "Given: Conservative port Port_Alpha declaring dimensions summing to (1,1,-1,0,0,0,0) d_{Power}. When: Conjugate invariant validation executes. Then: Rejection emits E0301, assigning sentinel _{pair}. Diagnostic: E0301.";
        attribute ac_04_port_dimensional_mismatch_delimiter_recovery : String = "Given: Connection with mismatched effort dimensions before delimiter (;) and valid Port_Gamma. When: Semantic validation and delimiter recovery execute. Then: Rejection emits E0300, recovers on delimiter, parses Port_Gamma, and halts at C_{max_errors}} with E0199. Diagnostic: E0300, E0199.";
        doc /* \begin{aligned} \mathbf{d}_{\text{Power}}=(1,2,-3,0,0,0,0)\in\mathbb{Q}^7, &\quad \mathbf{e}_T=(0,0,1,0,0,0,0)\in\mathbb{Q}^7 \\ \forall C=\langle e,f\rangle\in\mathcal{C}_{\text{pair}}, &\quad \mathbf{d}(e)+\mathbf{d}(f)\equiv\mathbf{d}_{\text{Power}} \\ (P_1,P_2)\in\mathcal{E}_{\text{conn}}\implies(\mathbf{d}(e_{P_1})\equiv\mathbf{d}(e_{P_2}) &\land \mathbf{d}(f_{P_1})\equiv\mathbf{d}(f_{P_2}) \land f_{P_1}+f_{P_2}=0) \\ \mathbf{d}(R)=\mathbf{d}(e)-\mathbf{d}(f), \quad \mathbf{d}(C)=\mathbf{d}(f)+\mathbf{e}_T-\mathbf{d}(e), &\quad \mathbf{d}(I)=\mathbf{d}(e)+\mathbf{e}_T-\mathbf{d}(f) \\ \mathbf{d}(e)+\mathbf{d}(f)\not\equiv\mathbf{d}_{\text{Power}} &\implies \text{Diagnostic}(C,\text{E0301}) \land C\gets\bot_{\text{pair}} \\ \mathbf{d}(e_{P_1})\not\equiv\mathbf{d}(e_{P_2})\lor\mathbf{d}(f_{P_1})\not\equiv\mathbf{d}(f_{P_2}) &\implies \text{Diagnostic}((P_1,P_2),\text{E0300}) \\ \Delta_n &= \text{UUIDv5}(\text{NS}_{\text{DEAP}},\text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0107_Canonical_Power_dimension_conjugate_pair_closure_port_compatibility_and_error_predicates;
        require Invariant_REQ_0107_Canonical_Power_dimension_conjugate_pair_closure_port_compatibility_and_error_predicates;
        verify by AC_01_Canonical_Power_Invariant_Conjugate_Lowering;
        verify by AC_02_Conservative_Interconnect_Compatibility;
        verify by AC_03_Conjugate_Power_Product_Violation_Rejection;
        verify by AC_04_Port_Dimensional_Mismatch_Delimiter_Recovery;
        satisfy by MetrologyFlowEngine;
    }
    doc /* UUIDv5: b7988cc3-7042-57c9-a715-c5ff85506ec7 | Complexity: Class P (O(V + E) lowering, O(E) balance) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / Tellegen | Diagnostics: E0199, E0220, E0300, E0301 */
    requirement def REQ_0108_Generalized_Kirchhoff_Flow_Law_0_at_Conservative_Junctions {
        id = "REQ-0108";
        text = "The network solver SHALL synthesize generalized Kirchhoff Flow Law (KFL) continuity equations at conservative junctions.";
        attribute uuidv5 : String = "b7988cc3-7042-57c9-a715-c5ff85506ec7";
        attribute complexity_class : String = "Class P (O(V + E) lowering, O(E) balance)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / Tellegen";
        attribute diagnostic_codes : String = "E0199, E0220, E0300, E0301";
        attribute ac_01_multi_branch_ideal_junction_flow_balance : String = "Given: Junction Junction_Alpha with incoming Flow_Edge_1, Flow_Edge_2, Flow_Edge_3 and outgoing Flow_Edge_4, Flow_Edge_5. When: Lowering Kirchhoff flow continuity. Then: Emits f_1 + f_2 + f_3 - f_4 - f_5 = 0. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_commensurable_unit_conversion_and_dynamic_accumulation : String = "Given: Dynamic junction Junction_Beta (S_i:[M]^1), incoming Flow_Edge_1 (g/s, c_1=1/1000), outgoing Flow_Edge_2 (kg/s, c_2=1). When: Lowering dynamic flow continuity. Then: Synthesizes (1/1000) f_1 - f_2 = d(param_s})}{dt} in [M]^1 [T]^{-1}. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_incommensurable_dimensional_mismatch_rejection : String = "Given: Junction Junction_Alpha connecting Flow_Edge_1 ([I]^1) and Flow_Edge_2 ([L]^3 [T]^{-1}). When: Dimensional checking executes. Then: Rejects junction, binds sentinel _{junc}, emitting E0300. Diagnostic: E0300 (DimensionalMismatch).";
        attribute ac_04_static_flow_balance_violation_and_delimiter_recovery : String = "Given: Junction Junction_Alpha with flow imbalance (in: 10, out: 8) before delimiter ; and valid Junction_Beta. When: Balance checking and delimiter recovery execute. Then: Emits E0301, resynchronizes on ;, parsing Junction_Beta, emitting E0199 at C_{max_errors}}. Diagnostic: E0301, E0199.";
        doc /* \begin{aligned} \mathcal{G}_{\text{flow}}=(\mathcal{V}_{\text{junc}},\mathcal{E}_{\text{flow}}),\quad A_{i,e}\in\{+1,-1,0\},\quad \mathbf{A}_{\text{ideal}}\mathbf{C}_{\text{unit}}\mathbf{f}(t)=\mathbf{0} \\ \forall J_i\in\mathcal{V}_{\text{accum}},\;\sum_{e\in\text{Incident}(J_i)}A_{i,e}c_e f_e(t)=\frac{d S_i}{dt}(t),\quad \mathbf{d}\left(\frac{d S_i}{dt}\right)\equiv\mathbf{d}_{\text{through}}(J_i) \\ \mathbf{d}(e_a)\not\equiv\mathbf{d}(e_b)\implies\text{Diagnostic}(J_i,\text{E0300})\wedge\mathbf{d}_{\text{through}}(J_i)=\bot_{\text{dim}} \\ \forall\mathcal{K}_c\subseteq\mathcal{G}_{\text{flow}},\;\text{rank}(\mathbf{A}_c)=|\mathcal{V}_c|-1\implies\text{rank}(\mathbf{A}_{\text{reduced}})=|\mathcal{V}_{\text{junc}}|-|\mathcal{K}| \\ (J_i\in\mathcal{V}_{\text{ideal}}\wedge|\sum A_{i,e}c_e f_e|\gt 0)\implies\text{Diagnostic}(J_i,\text{E0301}) \\ (|\text{Incident}(J_i)|=1\wedge f_e\ne 0)\implies\text{Diagnostic}(J_i,\text{E0220}),\quad \Delta_J=\text{UUIDv5}(\text{NS}_{\text{DEAP}},\text{FQN}(J)) \end{aligned} */
        constraint def Invariant_REQ_0108_Topological_incidence_flow_continuity_dynamic_accumulation_dimensional_consistency_and_diagnostic_predicates;
        require Invariant_REQ_0108_Topological_incidence_flow_continuity_dynamic_accumulation_dimensional_consistency_and_diagnostic_predicates;
        verify by AC_01_Multi_Branch_Ideal_Junction_Flow_Balance;
        verify by AC_02_Commensurable_Unit_Conversion_and_Dynamic_Accumulation;
        verify by AC_03_Incommensurable_Dimensional_Mismatch_Rejection;
        verify by AC_04_Static_Flow_Balance_Violation_and_Delimiter_Recovery;
        satisfy by MetrologyFlowEngine;
    }
    doc /* UUIDv5: c52acd62-a862-5c28-b9f9-638216863745 | Complexity: Class P (O(V + E) forest, O(E) balance) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / Tellegen | Diagnostics: E0199, E0300, E0301 */
    requirement def REQ_0109_Generalized_Kirchhoff_Potential_Law_0_across_Closed_Loops {
        id = "REQ-0109";
        text = "The network solver SHALL synthesize generalized Kirchhoff Potential Law (KPL) equations across conservative loops.";
        attribute uuidv5 : String = "c52acd62-a862-5c28-b9f9-638216863745";
        attribute complexity_class : String = "Class P (O(V + E) forest, O(E) balance)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / Tellegen";
        attribute diagnostic_codes : String = "E0199, E0300, E0301";
        attribute ac_01_cycle_basis_extraction_kpl_synthesis : String = "Given: Network Network_Alpha with 5 junctions (Junction_0--Junction_4) and 7 branches (FlowBranch_0--FlowBranch_6). When: Extracting spanning forest and cycle constraints. Then: Partitions 3 chords (N_C = 3), builds B, emitting B_{k, e} e_e = 0. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_potential_field_arbitrary_loop_closure : String = "Given: Network Network_Alpha (e(Junction_0}) = 0), e = A_{red}^T e_{node}, and cycle L_{comb} = L_1 L_2. When: Evaluating potential closure on L_{comb}. Then: Yields B e = 0 and e = 0. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_incommensurable_dimensional_mismatch_rejection : String = "Given: Cycle Loop_Alpha with FlowBranch_1 ([M]^1 [L]^2 [T]^{-3} [I]^{-1}) and FlowBranch_2 ([M]^1 [L]^{-1} [T]^{-2}). When: Dimensional checking executes on loop branches. Then: Rejection emits E0300, binding sentinel _{dim}. Diagnostic: E0300 (DimensionalMismatch).";
        attribute ac_04_static_imbalance_delimiter_recovery : String = "Given: Loop Loop_Beta with static imbalance e = 1.0 0 before ; and valid Loop_Gamma. When: Balance checking and delimiter recovery execute. Then: Emits E0301, resynchronizes on ;, parses Loop_Gamma, emitting E0199 at C_{max_errors}}. Diagnostic: E0301, E0199.";
        doc /* \begin{aligned} \mathcal{G}_{\text{flow}}=(\mathcal{V}_{\text{junc}},\mathcal{E}_{\text{flow}}),\quad |\mathcal{E}_{\text{tree}}|=|\mathcal{V}_{\text{junc}}|-|\mathcal{K}|,\quad N_C=|\mathcal{E}_{\text{flow}}|-|\mathcal{V}_{\text{junc}}|+|\mathcal{K}| \\ \mathbf{B}=[\mathbf{B}_T \mid \mathbf{I}_{N_C}]\in\{-1,0,+1\}^{N_C\times |\mathcal{E}_{\text{flow}}|},\quad \mathbf{B}\mathbf{\Delta e}(t)=\mathbf{0},\quad \mathbf{B}\mathbf{A}^T=\mathbf{0} \\ \forall\mathcal{L}_k\in\mathcal{C},\;\sum_{e\in\text{Edges}(\mathcal{L}_k)}B_{k,e}\Delta e_e(t)=0,\quad \mathbf{\Delta e}=\mathbf{A}_{\text{red}}^T\mathbf{e}_{\text{node}}\implies\mathbf{B}\mathbf{\Delta e}=\mathbf{0} \\ \mathbf{d}_{\text{across}}(e_a)\not\equiv\mathbf{d}_{\text{across}}(e_b)\implies\text{Diagnostic}(\mathcal{L}_k,\text{E0300})\wedge\mathbf{d}_{\text{across}}(\mathcal{L}_k)=\bot_{\text{dim}} \\ \left|\sum_{e\in\text{Edges}(\mathcal{L}_k)}B_{k,e}\Delta e_e\right|\gt 0\implies\text{Diagnostic}(\mathcal{L}_k,\text{E0301})\wedge\mathcal{L}_k\gets\bot_{\text{loop}},\quad \Delta_{\mathcal{L}_k}=\text{UUIDv5}(\text{NS}_{\text{DEAP}},\text{FQN}(\mathcal{L}_k)) \end{aligned} */
        constraint def Invariant_REQ_0109_Topological_cycle_basis_fundamental_cycle_matrix_loop_potential_conservation_and_diagnostic_predicates;
        require Invariant_REQ_0109_Topological_cycle_basis_fundamental_cycle_matrix_loop_potential_conservation_and_diagnostic_predicates;
        verify by AC_01_Cycle_Basis_Extraction_KPL_Synthesis;
        verify by AC_02_Potential_Field_Arbitrary_Loop_Closure;
        verify by AC_03_Incommensurable_Dimensional_Mismatch_Rejection;
        verify by AC_04_Static_Imbalance_Delimiter_Recovery;
        satisfy by MetrologyFlowEngine;
    }
    doc /* UUIDv5: 87e23bc6-38c1-58df-b69a-59c2f68b8116 | Complexity: Class P (O(V+E) incidence, O(V\cdot E) cut-set/cycle matrices) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / Tellegen | Diagnostics: E0199, E0203, E0220, E0300, E0301 */
    requirement def REQ_0110_Flow_Conservation_Network_Incidence_Matrix_Construction {
        id = "REQ-0110";
        text = "The network solver SHALL construct incidence matrix A\in\{-1,0,1\}^{n\times b} for network G_{flow}=(V_{junc},E_{flow}), verify topological rank, and synthesize cut-set and cycle matrices.";
        attribute uuidv5 : String = "87e23bc6-38c1-58df-b69a-59c2f68b8116";
        attribute complexity_class : String = "Class P (O(V+E) incidence, O(V\cdot E) cut-set/cycle matrices)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / Tellegen";
        attribute diagnostic_codes : String = "E0199, E0203, E0220, E0300, E0301";
        attribute ac_01_two_junction_branch_incidence_and_column_sum : String = "Given: Network Network_Alpha with Junction_Alpha, Junction_Beta, and FlowBranch_1. When: Lowering constructs matrix A. Then: Emits A=[+1;-1] with column sum 0 and rank(A)=1. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_connected_triangular_loop_spanning_tree_and_duality : String = "Given: Triangle network with 3 junctions, 3 branches, and admittances y_j 0. When: Lowering spanning tree, cut-set Q, and cycle B. Then: Partitions 2 tree branches, 1 chord, yielding B_T=-Q_C^T, AB^T=0, and Y_{node} 0. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_self_loop_and_floating_interface_rejection : String = "Given: Branch with u_1=v_1 and isolated junction Junction_Alpha (deg=0). When: Auditing incidence and degree invariants. Then: Rejection emits E0203 for self-loop and E0220 for isolated junction. Diagnostic: E0203, E0220.";
        attribute ac_04_rank_deficiency_and_delimiter_recovery : String = "Given: Defective network (rank(A) n-p) before delimiter (;) and valid Network_Beta. When: Verifying solvability and executing recovery. Then: Emits E0301, resynchronizes on ;, parses Network_Beta, halting at C_{max_errors}} with E0199. Diagnostic: E0301, E0199.";
        doc /* \begin{aligned} \mathcal{G}_{\text{flow}}=(\mathcal{V}_{\text{junc}},\mathcal{E}_{\text{flow}}),\quad A_{i,j}\in\{-1,0,+1\},\quad \sum_{i=1}^n A_{i,j}=0 \\ u_j=v_j\implies\text{Diagnostic}(e_j,\text{E0203})\wedge e_j\gets\bot_{\text{branch}} \\ \text{rank}(\mathbf{A})=n-p,\quad \text{rank}(\mathbf{A})\lt n-p\implies\text{Diagnostic}(\text{Network},\text{E0301}) \\ \mathbf{A}_{\text{red}}=[\mathbf{A}_T\mid\mathbf{A}_C],\quad \text{rank}(\mathbf{A}_{\text{red}})=n-p,\quad \det(\mathbf{A}_T)\in\{-1,+1\} \\ \mathbf{Q}=[\mathbf{I}_{n-p}\mid\mathbf{Q}_C],\quad \mathbf{B}=[-\mathbf{Q}_C^T\mid\mathbf{I}_{b-n+p}],\quad \mathbf{Q}\mathbf{f}=\mathbf{0},\quad \mathbf{B}\mathbf{\Delta e}=\mathbf{0} \\ \mathbf{A}\mathbf{B}^T=\mathbf{0},\quad \mathbf{Q}\mathbf{B}^T=\mathbf{0},\quad \mathbf{f}^T\mathbf{\Delta e}=0,\quad \mathbf{Y}_{\text{node}}=\mathbf{A}_{\text{red}}\mathbf{Y}\mathbf{A}_{\text{red}}^T\succ 0 \\ \mathbf{d}(e_j)\not\equiv\mathbf{d}(e_k)\implies\text{Diagnostic}(J_i,\text{E0300}),\quad \text{deg}(J_i)\le 1\implies\text{Diagnostic}(J_i,\text{E0220}) \\ \forall n\in\mathcal{V}_{\text{AST}},\quad \Delta_n=\text{UUIDv5}(\text{NS}_{\text{DEAP}},\text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0110_Topological_incidence_rank_solvability_cut_set_and_cycle_duality_and_diagnostic_predicates;
        require Invariant_REQ_0110_Topological_incidence_rank_solvability_cut_set_and_cycle_duality_and_diagnostic_predicates;
        verify by AC_01_Two_Junction_Branch_Incidence_and_Column_Sum;
        verify by AC_02_Connected_Triangular_Loop_Spanning_Tree_and_Duality;
        verify by AC_03_Self_Loop_and_Floating_Interface_Rejection;
        verify by AC_04_Rank_Deficiency_and_Delimiter_Recovery;
        satisfy by MetrologyFlowEngine;
    }
    doc /* UUIDv5: 6f09c51c-508b-5e90-8b5a-d142d4fe60eb | Complexity: Class P (O(m + S + R) balance, O(N) lowering) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / Tellegen | Diagnostics: E0199, E0300, E0301 */
    requirement def REQ_0111_Abstract_Power_Product_Balance_sum_P_frac {
        id = "REQ-0111";
        text = "The network solver SHALL verify power product balance across closed conservative boundaries \Omega.";
        attribute uuidv5 : String = "6f09c51c-508b-5e90-8b5a-d142d4fe60eb";
        attribute complexity_class : String = "Class P (O(m + S + R) balance, O(N) lowering)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / Tellegen";
        attribute diagnostic_codes : String = "E0199, E0300, E0301";
        attribute ac_01_multi_port_power_balance : String = "Given: Assembly Assembly_Alpha with inputs Port_1 (60 W), Port_2 (40 W) and output Port_3 (-100 W). When: Lowering power balance equations. Then: Evaluates P_{in} = P_{out} = 100 W. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_dynamic_storage_and_scaling : String = "Given: Assembly Assembly_Beta with input Port_1 (2 kW, c_1=1000), output Port_2 (1500 W), dissipation 200 W, storage rate 300 W. When: Lowering dynamic balance constraints. Then: Evaluates (1000 2) - 1500 = 300 + 200. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_negative_dissipation_rejection : String = "Given: Assembly Assembly_Gamma with passive dissipation P_{dissipated} = -50 W. When: Static dissipation validation executes. Then: Rejects assembly, binds sentinel _{power}, emitting E0301. Diagnostic: E0301 (KirchhoffConservationViolation).";
        attribute ac_04_dimensional_mismatch_and_recovery : String = "Given: Assembly Assembly_Delta with mismatched port dimensions before delimiter ; and valid Assembly_Epsilon. When: Dimension checking and delimiter recovery execute. Then: Emits E0300, binds _{dim}, recovers on ;, parsing Assembly_Epsilon, emitting E0199 at C_{max_errors}}. Diagnostic: E0300, E0199.";
        doc /* \begin{aligned} \mathbf{d}_{\text{Power}}=(1,2,-3,0,0,0,0)\in\mathbb{Q}^7,\quad \mathbf{d}_{\text{Energy}}=(1,2,-2,0,0,0,0)\in\mathbb{Q}^7 \\ P_k(t)=e_k(t)f_k(t),\quad P_{\text{ext}}(t)=\sum_{k=1}^m \sigma_k c_k P_k(t)=P_{\text{in}}(t)-P_{\text{out}}(t) \\ P_{\text{in}}(t)=P_{\text{out}}(t)+\frac{d E_{\text{internal}}}{dt}(t)+P_{\text{dissipated}}(t),\quad P_{\text{dissipated}}(t)\ge 0 \\ \text{is\_passive}(\Omega)\implies\left(\forall T\ge 0,\;\int_0^T P_{\text{ext}}(t)\,dt\ge -E_{\text{initial}}\right)\land\left(\forall t\ge 0,\;E_{\text{internal}}(t)\ge 0\right) \\ \mathbf{d}(e_k)+\mathbf{d}(f_k)\not\equiv\mathbf{d}_{\text{Power}}\lor\mathbf{d}(E_s)\not\equiv\mathbf{d}_{\text{Energy}}\implies\text{Diagnostic}(n,\text{E0300})\land n\gets\bot_{\text{dim}} \\ (P_{\text{dissipated}}\lt 0)\lor\left|P_{\text{in}}-P_{\text{out}}-\frac{dE}{dt}-P_{\text{dissipated}}\right|\gt 0\implies\text{Diagnostic}(\Omega,\text{E0301}) \\ \forall n\in\mathcal{V}_{\text{AST}},\quad \Delta_n=\text{UUIDv5}(\text{NS}_{\text{DEAP}},\text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0111_Conjugate_port_power_internal_storage_dissipation_non_negativity_power_balance_passivity_and_diagnostic_predicates;
        require Invariant_REQ_0111_Conjugate_port_power_internal_storage_dissipation_non_negativity_power_balance_passivity_and_diagnostic_predicates;
        verify by AC_01_Multi_Port_Power_Balance;
        verify by AC_02_Dynamic_Storage_and_Scaling;
        verify by AC_03_Negative_Dissipation_Rejection;
        verify by AC_04_Dimensional_Mismatch_and_Recovery;
        satisfy by MetrologyFlowEngine;
    }
    doc /* UUIDv5: 71ffa782-aca0-5839-aa8d-259e9c4ee546 | Complexity: Class P (O(1) interval check, O(N) lowering) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / ISO 80000-1 | Diagnostics: E0199, E0300, E0301 */
    requirement def REQ_0112_Parametric_Operational_Envelopes {
        id = "REQ-0112";
        text = "The metrology engine SHALL validate parameters and signals against declared operational envelopes [v_{\min}, v_{\max}] \subset \overline{\mathbb{R}}.";
        attribute uuidv5 : String = "71ffa782-aca0-5839-aa8d-259e9c4ee546";
        attribute complexity_class : String = "Class P (O(1) interval check, O(N) lowering)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / ISO 80000-1";
        attribute diagnostic_codes : String = "E0199, E0300, E0301";
        attribute ac_01_envelope_lowering_nominal_containment : String = "Given: Parameter Param_X : Real with envelope Envelope_Alpha ([10.0, 50.0]) and v_{nom} = 25.0. When: Lowering validates bounds and containment. Then: Passes with 10.0 25.0 50.0 and empty sink. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_unit_scaling_monotonic_inclusion : String = "Given: Parameter Param_Y with tiers E_{normal} = [20.0 mm, 80.0 mm], E_{safety} = [0.01 m, 0.10 m]. When: Lowering normalizes units (c = 1/1000) and checks monotonicity. Then: Validates [0.02, 0.08] [0.01, 0.10]. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_inverted_bounds_dimensional_rejection : String = "Given: Envelope Envelope_Beta ([100.0, 50.0]) and parameter Param_Z with bounds [L]^1 vs [T]^1. When: Lowering checks bound ordering and dimensions. Then: Rejects declarations, emits E0300, binds _{env}. Diagnostic: E0300 (InvalidOperationalEnvelopeBounds).";
        attribute ac_04_containment_violation_delimiter_recovery : String = "Given: Constant Param_Const = 75.0 with envelope [10.0, 50.0] before ;, and disjoint ports [10.0, 30.0], [50.0, 80.0]. When: Containment and interconnect intersection validate. Then: Emits E0301, recovers on ;, emits E0199 at C_{max_errors}}. Diagnostic: E0301, E0199.";
        doc /* \begin{aligned} \mathcal{E}_x=[v_{\min},v_{\max}]\subset\overline{\mathbb{R}},\quad v_{\min}\le v_{\max},\quad \mathbf{d}(v_{\min})\equiv\mathbf{d}(v_{\max})\equiv\mathbf{d}_{\mathcal{E}}\in\mathbb{Q}^7 \\ c_{\min}v_{\min}\le c_{\text{nom}}v_{\text{nom}}\le c_{\max}v_{\max}\iff\tilde{v}_{\text{nom}}\in\tilde{\mathcal{E}}_x,\quad c_i\in\mathbb{Q}^+ \\ \forall v\in\{v_{\text{const}},v_{\text{init}},v_{\text{nom}},v_{\text{default}}\},\;(v\lt v_{\min}\lor v\gt v_{\max})\implies\text{Diagnostic}(x,\text{E0301}) \\ v_{\min}^{\text{safe}}\le v_{\min}^{\text{norm}}\le v_{\max}^{\text{norm}}\le v_{\max}^{\text{safe}}\land(\mathcal{E}_{\text{normal}}\not\subseteq\mathcal{E}_{\text{safety}}\implies\text{Diagnostic}(\mathcal{E},\text{E0300})) \\ (P_1,P_2)\in\mathcal{E}_{\text{conn}}\implies(\mathcal{E}_1\cap\mathcal{E}_2=\emptyset\implies\text{Diagnostic}((P_1,P_2),\text{E0301})) \\ (v_{\min}\gt v_{\max}\lor\mathbf{d}(v_{\min})\not\equiv\mathbf{d}(v_{\max}))\implies\text{Diagnostic}(\mathcal{E},\text{E0300})\land\mathcal{E}\gets\bot_{\text{env}} \\ \forall n\in\mathcal{V}_{\text{AST}},\;\Delta_n=\text{UUIDv5}(\text{NS}_{\text{DEAP}},\text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0112_Operational_envelope_closed_interval_semantics_dimensional_homogeneity_point_containment_and_error_predicates;
        require Invariant_REQ_0112_Operational_envelope_closed_interval_semantics_dimensional_homogeneity_point_containment_and_error_predicates;
        verify by AC_01_Envelope_Lowering_Nominal_Containment;
        verify by AC_02_Unit_Scaling_Monotonic_Inclusion;
        verify by AC_03_Inverted_Bounds_Dimensional_Rejection;
        verify by AC_04_Containment_Violation_Delimiter_Recovery;
        satisfy by MetrologyFlowEngine;
    }
    doc /* UUIDv5: 0aca7409-8a38-5c36-9245-98f5fdceef5c | Complexity: Class P (O(k) linear bounding, O(N \cdot K) AST evaluation) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 | Diagnostics: E0199, E0300, E0301 */
    requirement def REQ_0113_Static_Boundary_Value_Analysis {
        id = "REQ-0113";
        text = "The metrology engine SHALL perform static boundary value analysis (BVA), verifying tolerance envelopes satisfy safety bounds.";
        attribute uuidv5 : String = "0aca7409-8a38-5c36-9245-98f5fdceef5c";
        attribute complexity_class : String = "Class P (O(k) linear bounding, O(N \cdot K) AST evaluation)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119";
        attribute diagnostic_codes : String = "E0199, E0300, E0301";
        attribute ac_01_linear_affine_tolerance_containment : String = "Given: Linear y = 2 x_1 + 3 x_2 with x_1 [1.0, 3.0], x_2 [2.0, 4.0], target [5.0, 20.0]. When: Lowering evaluates extrema. Then: Computes [y_{, y_{] = [8.0, 18.0] [5.0, 20.0] with _{lower} = 3.0, _{upper} = 2.0. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_unit_conversion_and_boundary_clearance : String = "Given: Nominal 1.0 m with tolerance 50.0 mm and bound [0.95, 1.05] m. When: Normalizing units and boundaries. Then: Scales via c_u = 1/1000 Q^+ to [0.95, 1.05] m with zero margin violation. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_safety_envelope_breach_rejection : String = "Given: Response [2.0, 12.0] against envelope [0.0, 10.0]. When: Containment checking executes. Then: Upper breach (12.0 10.0) emits E0301. Diagnostic: E0301 (ToleranceEnvelopeViolation).";
        attribute ac_04_dimension_mismatch_and_delimiter_recovery : String = "Given: Mismatched [L]^1 vs [T]^1 before ;, followed by valid Tolerance_Beta. When: Type checking and recovery execute. Then: Emits E0300, binds _{dim}, recovers on ;, parses Tolerance_Beta, halting at C_{max_errors}} with E0199. Diagnostic: E0300, E0199.";
        doc /* \begin{aligned} \mathcal{I}(x_i)=[x_{i,0}-\delta_i^-,x_{i,0}+\delta_i^+],\quad \mathcal{B}=\prod_{i=1}^k\mathcal{I}(x_i)\subset\mathbb{R}^k,\quad \mathcal{R}_y=[y_{\min},y_{\max}]\subseteq [y_{\min}^{\text{IA}},y_{\max}^{\text{IA}}] \\ [y_{\min},y_{\max}]\subseteq [Y_{\text{lower\_limit}},Y_{\text{upper\_limit}}]\iff (y_{\min}\ge Y_{\text{lower\_limit}}\land y_{\max}\le Y_{\text{upper\_limit}}) \\ \mu_{\text{lower}}=y_{\min}-Y_{\text{lower\_limit}}\ge 0,\quad \mu_{\text{upper}}=Y_{\text{upper\_limit}}-y_{\max}\ge 0 \\ \mathbf{d}(x_i)\equiv\mathbf{d}(\delta_i^-)\equiv\mathbf{d}(\delta_i^+),\quad \mathbf{d}(y)\equiv\mathbf{d}(f(\mathbf{x}))\equiv\mathbf{d}(Y_{\text{lower\_limit}})\equiv\mathbf{d}(Y_{\text{upper\_limit}}) \\ (x_{i,\min}\gt x_{i,\max}\lor\delta_i\lt 0)\implies\text{Diagnostic}(T,\text{E0301})\land T\gets\bot_{\text{tol}} \\ (\mathbf{d}(x_i)\not\equiv\mathbf{d}(\delta_i)\lor\mathbf{d}(f)\not\equiv\mathbf{d}(Y_{\text{limit}}))\implies\text{Diagnostic}(T,\text{E0300})\land\mathbf{d}(T)\gets\bot_{\text{dim}} \\ (y_{\min}\lt Y_{\text{lower\_limit}}\lor y_{\max}\gt Y_{\text{upper\_limit}})\implies\text{Diagnostic}(T,\text{E0301}),\quad \Delta_T=\text{UUIDv5}(\text{NS}_{\text{DEAP}},\text{FQN}(T)) \end{aligned} */
        constraint def Invariant_REQ_0113_Parametric_tolerance_hyperbox_response_extrema_safety_containment_and_diagnostic_trigger_predicates;
        require Invariant_REQ_0113_Parametric_tolerance_hyperbox_response_extrema_safety_containment_and_diagnostic_trigger_predicates;
        verify by AC_01_Linear_Affine_Tolerance_Containment;
        verify by AC_02_Unit_Conversion_and_Boundary_Clearance;
        verify by AC_03_Safety_Envelope_Breach_Rejection;
        verify by AC_04_Dimension_Mismatch_and_Delimiter_Recovery;
        satisfy by MetrologyFlowEngine;
    }
}
