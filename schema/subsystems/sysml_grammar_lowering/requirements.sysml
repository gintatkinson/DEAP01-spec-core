doc /* Complete OMG SysML v2 / KerML Metamodel Lowering & Grammar */
package SysmlGrammarLowering {
    doc /* UUIDv5: bb1eed77-c074-5fcf-af35-578e302d7222 | Complexity: Class P (O(1)) | Standard: IEEE 29148-2018 / RFC 2119 / OMG SysML v2 | Diagnostics: E0199, E0200, E0203 */
    requirement def REQ_0054_KerML_Root_Element_Relationship {
        id = "REQ-0054";
        text = "The compiler metamodel lowering engine shall define and enforce the foundational KerML abstract syntax root hierarchy as a rooted directed multigraph of semantic elements E, directed relationships R \subseteq E, metadata annotations M \subseteq E, and containment ownership relations O \subset E";
        attribute uuidv5 : String = "bb1eed77-c074-5fcf-af35-578e302d7222";
        attribute complexity_class : String = "Class P (O(1))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119 / OMG SysML v2";
        attribute diagnostic_codes : String = "E0199, E0200, E0203";
        attribute ac_01_universal_root_element_allocation_and_deterministic_identity : String = "Given: Abstract model declaring Package_0 and Classifier_Alpha. When: Lowering allocates entities into the contiguous AST memory. Then: Every element occupies an AST handle (PointerIndirectionDepth = 0), assigns an interned scalar SymbolId, and receives an RFC 4122 UUIDv5 identifier from OID_{DEAP} and FQN(e). Diagnostic: E0200 emitted on handle exhaustion, alignment fault, or invalid topological identifier formatting.";
        attribute ac_02_directed_relationship_endpoints_and_non_reflexivity_validation : String = "Given: Model declaring relationship Rel_Alpha with source { Element_Alpha} } and target { Element_Beta} }. When: Relationship endpoint validation executes. Then: Endpoints are verified as non-empty, mutually disjoint (source target = ), and allocated in active arena storage without emitting diagnostics. Diagnostic: E0200 emitted if endpoints are empty, reflexive, or unallocated.";
        attribute ac_03_structural_containment_single_ownership_and_cycle_recovery : String = "Given: Containment hierarchy with single ownership and an injected cycle Package_0}, Element_Alpha}, Package_0} . When: Containment validation and cycle analysis execute. Then: Conflicting owners (|owner| 1) trigger E0200; cycles ((e, e) O^+) trigger E0203 with cycle path reconstruction, and synchronization token recovery severs the back-edge. Diagnostic: E0200 for multiple owners; E0203 for containment cycles; E0199 upon exhausting C_{max_errors}}.";
        attribute ac_04_non_intrusive_annotation_semantics_and_type_invariance : String = "Given: Semantic element Element_Alpha with denotation ⟦ Element_Alpha} ⟧. When: Metadata annotation Annotation_0 is attached to Element_Alpha. Then: Algebraic type denotation satisfies ⟦ Element_Alpha} Annotation_0} ⟧ ⟦ Element_Alpha} ⟧, preserving subtyping and operational semantics without alteration. Diagnostic: E0200 emitted if metadata alters type denotation or triggers semantic evaluation side-effects.";
        doc /* G_{\text{KerML}} = \langle \mathcal{E}, \mathcal{R}, \mathcal{M}, \mathcal{O} \rangle, \quad \mathcal{R} \subseteq \mathcal{E}, \quad \mathcal{M} \subseteq \mathcal{E}, \quad \mathcal{O} \subset \mathcal{E} \times \mathcal{E} */
        constraint def Invariant_REQ_0054_FormalInvariant;
        doc /* e = \langle \text{id}_e, \text{name}_e, \text{owner}_e, \mathcal{C}_e, \mathcal{A}_e, \Delta_e, \text{span}_e \rangle */
        constraint def Invariant_REQ_0054_Every_element_is_an_immutable_tuple;
        doc /* \begin{aligned} \text{directed}(r) &\implies (\text{source}(r) \neq \emptyset) \wedge (\text{target}(r) \neq \emptyset) \wedge (\text{source}(r) \cap \text{target}(r) = \emptyset) \\ \forall u &\in (\text{source}(r) \cup \text{target}(r)), \quad \text{IsAllocated}(\text{id}(u)) \equiv \text{true} \end{aligned} */
        constraint def Invariant_REQ_0054_Relationship_endpoint_invariants;
        doc /* \forall v \in \mathcal{E}, \quad |\{ u \in \mathcal{E} \mid (u, v) \in \mathcal{O} \}| \le 1 \quad \wedge \quad (v, v) \notin \mathcal{O}^+ */
        constraint def Invariant_REQ_0054_Ownership_forest_and_acyclicity_invariants;
        doc /* \forall u, v \in \mathcal{E}, \quad (u, v) \in \mathcal{O} \iff v \in \mathcal{C}_u \iff \text{owner}(v) = u */
        constraint def Invariant_REQ_0054_FormalInvariant_2;
        doc /* \forall e \in \mathcal{E}, \; \forall \alpha \in \mathcal{M}, \quad ⟦ e \oplus \alpha ⟧ \equiv ⟦ e ⟧ */
        constraint def Invariant_REQ_0054_Annotation_non_interference;
        doc /* \begin{aligned} \text{source}(r) = \emptyset \vee \text{target}(r) = \emptyset \vee (\text{source}(r) \cap \text{target}(r) \neq \emptyset) &\implies \text{Diagnostic}(r, \text{E0200}) \in \mathcal{D}_{\text{sink}} \\ \exists u \in (\text{source}(r) \cup \text{target}(r)), \, \neg\text{IsAllocated}(\text{id}(u)) &\implies \text{Diagnostic}(r, \text{E0200}) \in \mathcal{D}_{\text{sink}} \\ |\{ u \in \mathcal{E} \mid (u, v) \in \mathcal{O} \}| \gt 1 &\implies \text{Diagnostic}(v, \text{E0200}) \in \mathcal{D}_{\text{sink}} \\ (e, e) \in \mathcal{O}^+ &\implies \text{Diagnostic}(\mathcal{P}_{\text{cycle}}, \text{E0203}) \in \mathcal{D}_{\text{sink}} \\ |\mathcal{D}_{\text{sink}}| \ge C_{\text{max\_errors}} &\implies \text{DiagnosticCode::E0199} \in \mathcal{D}_{\text{sink}} \end{aligned} */
        constraint def Invariant_REQ_0054_Diagnostic_trigger_mappings_per_REQ_0191;
        require Invariant_REQ_0054_FormalInvariant;
        require Invariant_REQ_0054_Every_element_is_an_immutable_tuple;
        require Invariant_REQ_0054_Relationship_endpoint_invariants;
        require Invariant_REQ_0054_Ownership_forest_and_acyclicity_invariants;
        require Invariant_REQ_0054_FormalInvariant_2;
        require Invariant_REQ_0054_Annotation_non_interference;
        require Invariant_REQ_0054_Diagnostic_trigger_mappings_per_REQ_0191;
        derived from REQ_0191_Standardized_Compiler_Diagnostic_Taxonomy_Source_Mapping;
        derived from REQ_0042_Circular_Dependency;
        verify by AC_01_Universal_Root_Element_Allocation_and_Deterministic_Identity;
        verify by AC_02_Directed_Relationship_Endpoints_and_Non_Reflexivity_Validation;
        verify by AC_03_Structural_Containment_Single_Ownership_and_Cycle_Recovery;
        verify by AC_04_Non_Intrusive_Annotation_Semantics_and_Type_Invariance;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: 9e2f0360-44cc-5d8c-8b8b-6018610e2a3d | Complexity: Class P (O(1)) | Standard: IEEE 29148-2018 / RFC 2119 / OMG SysML v2 | Diagnostics: E0199, E0200, E0203 */
    requirement def REQ_0055_KerML_Namespace_Membership_OwningMembership {
        id = "REQ-0055";
        text = "The semantic model lowering engine shall represent lexical scopes, naming environments, and containment hierarchies as formal Namespace scopes composed of typed Membership relationships, mapping interned member symbols to semantic element node handles in the AST graph.";
        attribute uuidv5 : String = "9e2f0360-44cc-5d8c-8b8b-6018610e2a3d";
        attribute complexity_class : String = "Class P (O(1))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119 / OMG SysML v2";
        attribute diagnostic_codes : String = "E0199, E0200, E0203";
        attribute ac_01_membership_partitioning_and_o_local_scope_resolution : String = "Given: Model declaring Package_0 hosting OwningMembership for Classifier_Alpha and non-owning Membership for Classifier_Beta owned by Package_1. When: Symbol indexing resolves local members by interned SymbolId. Then: Both resolve in amortized O(1) time with owner(Classifier_Alpha}) = Package_0} and owner(Classifier_Beta}) = Package_1}. Diagnostic: E0200 emitted on local lookup failure.";
        attribute ac_02_single_ownership_bijection_invariant_enforcement : String = "Given: Element Classifier_Alpha owned by Package_0 via OwningMembership. When: Lowering processes a secondary OwningMembership for Classifier_Alpha in Package_1. Then: The compiler rejects duplicate ownership, retains singular ownership in Package_0, and binds sentinel _{node}. Diagnostic: E0200 emitted on duplicate owning membership violation.";
        attribute ac_03_lexical_uniqueness_and_colliding_name_rejection : String = "Given: Namespace Package_0 declaring distinct elements Classifier_Alpha and Classifier_Beta. When: Both declarations specify identical member name Item_X within Package_0. Then: The compiler rejects the second membership as a duplicate name collision, while permitting distinct memberships binding to identical elements. Diagnostic: E0200 emitted on duplicate member name collision.";
        attribute ac_04_ownership_forest_strict_acyclicity_verification : String = "Given: Circular ownership where Package_0 owns Package_1 and Package_1 attempts to own Package_0 via OwningMembership. When: Lowering executes ownership cycle validation. Then: The engine detects (e, e) O^+, severs back-edge, binds sentinel _{node}, and continues compilation. Diagnostic: E0203 emitted with cycle path trace Package_0}, Package_1}, Package_0} .";
        doc /* \mathcal{M} = \mathcal{M}_{\text{own}} \cup \mathcal{M}_{\text{ref}}, \quad \mathcal{M}_{\text{own}} \cap \mathcal{M}_{\text{ref}} = \emptyset */
        constraint def Invariant_REQ_0055_FormalInvariant;
        doc /* \mu_N(\sigma) = \begin{cases} e_m, & \text{if } \exists m \in \mathcal{M}_N : \sigma_m = \sigma \\ \bot, & \text{otherwise} \end{cases} */
        constraint def Invariant_REQ_0055_Local_symbol_resolution_map_N;
        doc /* \forall e \in \mathcal{E}_{\text{owned}}, \; |\{ m \in \mathcal{M}_{\text{own}} \mid \text{member}(m) = e \}| = 1, \quad \text{owner}(e) = \begin{cases} N_m, & \text{if } \exists ! m \in \mathcal{M}_{\text{own}} : \text{member}(m) = e \\ \bot, & \text{otherwise} \end{cases} */
        constraint def Invariant_REQ_0055_Single_ownership_bijection;
        doc /* \forall e \in \mathcal{E}, \quad (e, e) \notin \mathcal{O}^+ */
        constraint def Invariant_REQ_0055_Acyclic_ownership_relation_with_irreflexive_transitive_closure;
        doc /* \forall m_1, m_2 \in \mathcal{M}_N, \quad (\sigma_{m_1} = \sigma_{m_2} \ne \bot) \implies \text{member}(m_1) = \text{member}(m_2) */
        constraint def Invariant_REQ_0055_Lexical_uniqueness_within_namespace;
        doc /* \begin{aligned} \text{QN}(e) &= \begin{cases} [\text{name}(e)], & \text{if } \text{owner}(e) = \bot \\ \text{QN}(\text{owner}(e)) \mathbin{\Vert} [\text{name}(e)], & \text{otherwise} \end{cases} \\ \text{FQN}(e) &= \text{StringOf}(\sigma_1) \mathbin{\Vert} \text{"::"} \dots \text{"::"} \mathbin{\Vert} \text{StringOf}(\sigma_k), \quad \Delta_e = \text{UUIDv5}(\text{OID}_{\text{DEAP}}, \text{FQN}(e)) \end{aligned} */
        constraint def Invariant_REQ_0055_Qualified_name_recurrence_canonical_path_serialization_and_deterministic_UUIDv5_anchor;
        doc /* \begin{aligned} (\sigma_{m_1} = \sigma_{m_2} \ne \bot) \wedge \text{member}(m_1) \ne \text{member}(m_2) &\implies \text{Diagnostic}(m_2, \text{E0200}) \in \mathcal{D}_{\text{sink}} \\ |\{ m \in \mathcal{M}_{\text{own}} \mid \text{member}(m) = e \}| \gt 1 &\implies \text{Diagnostic}(e, \text{E0200}) \in \mathcal{D}_{\text{sink}} \\ (e, e) \in \mathcal{O}^+ &\implies \text{Diagnostic}(e, \text{E0203}) \in \mathcal{D}_{\text{sink}} \\ \text{ResolvePath}(\pi, N) = \bot &\implies \text{Diagnostic}(\pi, \text{E0200}) \in \mathcal{D}_{\text{sink}} \end{aligned} */
        constraint def Invariant_REQ_0055_Diagnostic_trigger_invariants;
        require Invariant_REQ_0055_FormalInvariant;
        require Invariant_REQ_0055_Local_symbol_resolution_map_N;
        require Invariant_REQ_0055_Single_ownership_bijection;
        require Invariant_REQ_0055_Acyclic_ownership_relation_with_irreflexive_transitive_closure;
        require Invariant_REQ_0055_Lexical_uniqueness_within_namespace;
        require Invariant_REQ_0055_Qualified_name_recurrence_canonical_path_serialization_and_deterministic_UUIDv5_anchor;
        require Invariant_REQ_0055_Diagnostic_trigger_invariants;
        verify by AC_01_Membership_Partitioning_and_O_Local_Scope_Resolution;
        verify by AC_02_Single_Ownership_Bijection_Invariant_Enforcement;
        verify by AC_03_Lexical_Uniqueness_and_Colliding_Name_Rejection;
        verify by AC_04_Ownership_Forest_Strict_Acyclicity_Verification;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: c1343fa7-4d7b-5a7c-8b6d-852eb849cd7c | Complexity: Class P (O(V + E)) | Standard: IEEE 29148-2018 / RFC 2119 / OMG SysML v2 | Diagnostics: E0200, E0203 */
    requirement def REQ_0056_KerML_Import_Declaration_Semantics {
        id = "REQ-0056";
        text = "The KerML scoping engine shall evaluate namespace import declarations as deterministic relational projections over exported membership sets across compilation units.";
        attribute uuidv5 : String = "c1343fa7-4d7b-5a7c-8b6d-852eb849cd7c";
        attribute complexity_class : String = "Class P (O(V + E))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119 / OMG SysML v2";
        attribute diagnostic_codes : String = "E0200, E0203";
        attribute ac_01_public_vs_private_import_visibility_re_export : String = "Given: Namespace Package_0 declaring public Classifier_Alpha, namespace Package_1 declaring public import Package_0::Classifier_Alpha;, and namespace Package_2 declaring import Package_1::*;. When: Metamodel lowering resolves symbol Classifier_Alpha within Package_2. Then: Classifier_Alpha is resolved in Package_2 via public re-export; replacing Package_1 with private import causes lookup to fail with diagnostic error code E0200. Diagnostic: E0200 emitted on foreign access to non-exported private import.";
        attribute ac_02_recursive_wildcard_import_and_sub_namespace_closure : String = "Given: Nested namespace hierarchy Package_0::SubPackage_1::Classifier_Alpha and target namespace Package_2 declaring import Package_0::**;. When: Scoping engine evaluates reflexive-transitive closure M_{rec}(Package_0}). Then: Symbol Classifier_Alpha resolves unqualified within Package_2, and visited-set cycle detection guarantees deterministic completion without infinite recursion. Diagnostic: E0203 emitted if cyclic namespace containment causes an illegal containment loop.";
        attribute ac_03_deterministic_shadow_precedence : String = "Given: Target namespace Package_1 declaring local member Classifier_Alpha and an import import Package_0::Classifier_Alpha;. When: Scoping engine resolves unqualified reference Classifier_Alpha within Package_1. Then: Unqualified lookup binds strictly to the local member (Find = M_{owned}), while the imported symbol remains accessible via fully qualified path ::Package_0::Classifier_Alpha. Diagnostic: E0200 emitted if shadow precedence fails to bind local declaration.";
        attribute ac_04_ambiguous_unqualified_import_collision_detection : String = "Given: Namespaces Package_0 and Package_1 both exporting public Classifier_Alpha, and target namespace Package_2 containing import Package_0::*; import Package_1::*; with no local Classifier_Alpha. When: Metamodel lowering attempts unqualified resolution of Classifier_Alpha within Package_2. Then: Scoping engine detects conflicting imported symbols (|{e}| 1), rejects the resolution, and emits diagnostic error code E0200. Diagnostic: E0200 emitted on ambiguous unqualified import collision.";
        doc /* \mathcal{V} = \{ \text{Private}, \text{Public} \}, \quad \text{Private} \prec \text{Public} */
        constraint def Invariant_REQ_0056_Visibility_lattice;
        doc /* \begin{aligned} \mathcal{M}_{\text{imp}}(I) &= \{ \langle \alpha(s), e \rangle \mid \langle s, e \rangle \in \text{ExportedMemberships}(N_{\text{src}}) \;\wedge\; \phi(\langle s, e \rangle) \equiv \text{true} \} \\ \text{ExportedMemberships}(N) &= \{ m \in \mathcal{M}_{\text{owned}}(N) \cup \mathcal{M}_{\text{imp}}(N) \mid \text{Vis}(m) = \text{Public} \} \end{aligned} */
        constraint def Invariant_REQ_0056_An_import_declaration_with_visibility_filter_predicate;
        doc /* \mathcal{M}_{\text{rec}}(N_{\text{src}}) = \bigcup_{N' \in \text{SubNamespaces}^*(N_{\text{src}})} \text{ExportedMemberships}(N') */
        constraint def Invariant_REQ_0056_Recursive_wildcard_imports;
        doc /* \begin{aligned} \text{Find}(s, N_{\text{tgt}}) &= \begin{cases} \{ e \mid \langle s, e \rangle \in \mathcal{M}_{\text{owned}}(N_{\text{tgt}}) \}, & \text{if } \exists \langle s, e \rangle \in \mathcal{M}_{\text{owned}}(N_{\text{tgt}}) \\ \{ e \mid \langle s, e \rangle \in \mathcal{M}_{\text{imp}}(N_{\text{tgt}}) \}, & \text{otherwise} \end{cases} \\ \text{Ambiguous}(s, N_{\text{tgt}}) &\iff (\forall e, \langle s, e \rangle \notin \mathcal{M}_{\text{owned}}(N_{\text{tgt}})) \;\wedge\; |\{ e \mid \langle s, e \rangle \in \mathcal{M}_{\text{imp}}(N_{\text{tgt}}) \}| \gt 1 \end{aligned} */
        constraint def Invariant_REQ_0056_Symbol_lookup_resolution_and_conflict_detection;
        doc /* \text{Ambiguous}(s, N_{\text{tgt}}) \;\vee\; \text{TargetNotFound}(I) \implies \text{Diagnostic}(s, \text{E0200}) \in \mathcal{D}_{\text{sink}} */
        constraint def Invariant_REQ_0056_Diagnostic_trigger;
        require Invariant_REQ_0056_Visibility_lattice;
        require Invariant_REQ_0056_An_import_declaration_with_visibility_filter_predicate;
        require Invariant_REQ_0056_Recursive_wildcard_imports;
        require Invariant_REQ_0056_Symbol_lookup_resolution_and_conflict_detection;
        require Invariant_REQ_0056_Diagnostic_trigger;
        verify by AC_01_Public_vs_Private_Import_Visibility_Re_export;
        verify by AC_02_Recursive_Wildcard_Import_and_Sub_Namespace_Closure;
        verify by AC_03_Deterministic_Shadow_Precedence;
        verify by AC_04_Ambiguous_Unqualified_Import_Collision_Detection;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: 8eddb83e-6d77-5e25-8cb7-99446770451d | Complexity: Class P (O(1)) | Standard: IEEE 29148-2018 / RFC 2119 / OMG SysML v2 | Diagnostics: E0100, E0199, E0200, E0203 */
    requirement def REQ_0057_KerML_Core_Classifiers {
        id = "REQ-0057";
        text = "The semantic metamodel lowering engine shall formalize the foundational KerML classifier hierarchy into a bounded subtyping poset (K, \le) rooted at Type. Classifiers shall be partitioned into structural value types (DataType) and extensional identity-bearing entities (Class, Structure) under";
        attribute uuidv5 : String = "8eddb83e-6d77-5e25-8cb7-99446770451d";
        attribute complexity_class : String = "Class P (O(1))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119 / OMG SysML v2";
        attribute diagnostic_codes : String = "E0100, E0199, E0200, E0203";
        attribute ac_01_foundational_taxonomic_subtyping_hierarchy : String = "Given: Metamodel definitions Structure_Omega :> Class_Zeta :> Classifier_Alpha :> Type_0 and DataType_Delta :> Classifier_Alpha. When: Subtyping poset (K, ) construction executes. Then: Structure_Omega} Type_0} and DataType_Delta} Type_0} evaluate to true; Structure_Omega} DataType_Delta} evaluates to false; zero diagnostics emitted. Diagnostic: E0200 emitted on invalid subtyping resolution.";
        attribute ac_02_disjoint_classifier_specialization_enforcement : String = "Given: Classifier Classifier_Invalid declaring multiple inheritance over both DataType_Delta and Class_Zeta. When: Metamodel lowering verifies classifier disjointness ⟦ DataType ⟧ ⟦ Class ⟧ = . Then: The compiler rejects the declaration, binds error sentinel _{classifier}, and emits diagnostic E0200. Diagnostic: E0200 emitted on disjoint classifier specialization violation.";
        attribute ac_03_identity_duality_value_congruence_vs_object_identity : String = "Given: Instances a, b ⟦ DataType_Delta} ⟧ with identical feature values (param_x = 42.0), and instances u, v ⟦ Class_Zeta} ⟧ with identical values (param_x = 42.0) but distinct handles (id(u) id(v)). When: Equivalence evaluation executes on pairs (a, b) and (u, v). Then: Value congruence yields a _{val} b = true; object identity yields u = v false. Diagnostic: E0200 emitted on identity evaluation violation.";
        attribute ac_04_datatype_feature_immutability_enforcement : String = "Given: Declaration DataType_Delta containing a mutable feature or behavioral action lifecycle hook. When: Metamodel semantic checking inspects feature constraints. Then: The compiler rejects mutable or behavioral feature declarations in DataType_Delta, emitting diagnostic E0200. Diagnostic: E0200 emitted on mutable feature declaration within a DataType.";
        doc /* \begin{aligned} \text{Structure} &\le \text{Class} \le \text{Classifier} \le \text{Type} \\ \text{DataType} &\le \text{Classifier} \le \text{Type} \end{aligned} */
        constraint def Invariant_REQ_0057_FormalInvariant;
        doc /* C_1 \le C_2 \iff ⟦ C_1 ⟧ \subseteq ⟦ C_2 ⟧ */
        constraint def Invariant_REQ_0057_Denotational_semantics_maps_each_classifier_to_an_instance_set;
        doc /* ⟦ \text{DataType} ⟧ \cap ⟦ \text{Class} ⟧ = \emptyset, \quad ⟦ \text{DataType} ⟧ \cap ⟦ \text{Structure} ⟧ = \emptyset */
        constraint def Invariant_REQ_0057_Disjointness_between_value_types_and_identity_bearing_entities_is_invariant;
        doc /* \begin{aligned} a, b \in ⟦ \text{DataType} ⟧ &\implies \left( a \equiv_{\text{val}} b \iff \forall f \in \mathcal{F}_{\tau(a)}, \, \pi_f(a) \equiv_{\text{val}} \pi_f(b) \right) \\ u, v \in ⟦ \text{Class} ⟧ \cup ⟦ \text{Structure} ⟧ &\implies \left( u = v \iff \text{id}(u) = \text{id}(v) \right) \end{aligned} */
        constraint def Invariant_REQ_0057_Identity_duality_is_governed_by_structural_congruence_and_token_identity;
        doc /* \forall C \in \mathcal{K}, \quad C \le \text{DataType} \implies \forall f \in \mathcal{F}_C, \, (\text{is\_read\_only}(f) \equiv \text{true} \;\wedge\; \text{is\_behavioral}(f) \equiv \text{false}) */
        constraint def Invariant_REQ_0057_DataType_feature_immutability_and_acyclicity_invariants;
        doc /* \forall C \in \mathcal{K}, \quad (C, C) \notin \mathcal{E}_{\text{gen}}^+ */
        constraint def Invariant_REQ_0057_FormalInvariant_2;
        doc /* \forall C \in \mathcal{K}, \quad \Delta_C = \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(C)) */
        constraint def Invariant_REQ_0057_Deterministic_RFC_4122_UUIDv5_topological_anchoring_satisfies;
        require Invariant_REQ_0057_FormalInvariant;
        require Invariant_REQ_0057_Denotational_semantics_maps_each_classifier_to_an_instance_set;
        require Invariant_REQ_0057_Disjointness_between_value_types_and_identity_bearing_entities_is_invariant;
        require Invariant_REQ_0057_Identity_duality_is_governed_by_structural_congruence_and_token_identity;
        require Invariant_REQ_0057_DataType_feature_immutability_and_acyclicity_invariants;
        require Invariant_REQ_0057_FormalInvariant_2;
        require Invariant_REQ_0057_Deterministic_RFC_4122_UUIDv5_topological_anchoring_satisfies;
        verify by AC_01_Foundational_Taxonomic_Subtyping_Hierarchy;
        verify by AC_02_Disjoint_Classifier_Specialization_Enforcement;
        verify by AC_03_Identity_Duality_Value_Congruence_vs_Object_Identity;
        verify by AC_04_DataType_Feature_Immutability_Enforcement;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: 4a99d369-c8da-5c7e-a490-ab0cb4977158 | Complexity: Class P (O( | Standard: IEEE 29148-2018 / RFC 2119 / OMG SysML v2 | Diagnostics: E0199, E0200 */
    requirement def REQ_0058_KerML_Association_AssociationStructure {
        id = "REQ-0058";
        text = "The semantic lowering engine shall formalize multi-ary relations as Association and AssociationStructure classifiers possessing k \ge 2 participant end features (AssociationEnd), enforcing relational constraints, directional navigability, and fiber cardinality bounds over extensional Link tuples.";
        attribute uuidv5 : String = "4a99d369-c8da-5c7e-a490-ab0cb4977158";
        attribute complexity_class : String = "Class P (O(";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119 / OMG SysML v2";
        attribute diagnostic_codes : String = "E0199, E0200";
        attribute ac_01_multi_ary_association_arity_validation : String = "Given: Association_Invalid with k = 1 end and Association_A with k = 2 ends (End_1, End_2). When: Metamodel lowering performs association definition validation. Then: Association_Invalid is rejected with diagnostic E0200, whereas Association_A lowers successfully. Diagnostic: E0200 emitted on arity k 2.";
        attribute ac_02_link_participant_type_conformance : String = "Given: Association_A over Classifier_Alpha and Classifier_Beta, and Link_1 whose second participant has runtime type Classifier_Gamma (Classifier_Gamma} Classifier_Beta}). When: Semantic type checking validates Link_1. Then: Compilation rejects Link_1 and records diagnostic E0200 with exact source byte spans. Diagnostic: E0200 emitted on participant type mismatch.";
        attribute ac_03_fiber_multiplicity_bound_enforcement : String = "Given: Association_A between Classifier_Alpha and Classifier_Beta with interval _2 = [1, 3] on Classifier_Beta. When: Fiber validation evaluates instances of Classifier_Alpha with 0, 2, and 4 associated links. Then: Instances with 0 and 4 links trigger diagnostic E0200, while the instance with 2 links validates. Diagnostic: E0200 emitted on fiber multiplicity bound violation.";
        attribute ac_04_non_navigable_end_traversal_rejection : String = "Given: Association_A with End_1 marked Navigable and End_2 marked NonNavigable. When: An expression attempts traversal from End_1 to End_2. Then: Semantic analysis halts traversal and emits diagnostic E0200. Diagnostic: E0200 emitted on non-navigable end access attempt.";
        doc /* A = \langle \text{id}_A, \text{name}_A, (e_1, \dots, e_k), \Phi_A, \Delta_A \rangle, \quad k \ge 2 */
        constraint def Invariant_REQ_0058_FormalInvariant;
        doc /* e_i = \langle C_i, [l_i, u_i], \text{nav}_i \rangle, \quad C_i \in \mathcal{C}, \; l_i \le u_i, \; u_i \ge 1 */
        constraint def Invariant_REQ_0058_FormalInvariant_2;
        doc /* L = (o_1, \dots, o_k) \in \prod_{i=1}^k ⟦ C_i ⟧, \quad \forall i \in \{1, \dots, k\}: \tau(\pi_i(L)) \le C_i */
        constraint def Invariant_REQ_0058_FormalInvariant_3;
        doc /* \forall j \in \{1, \dots, k\}, \; \forall x \in ⟦ C_j ⟧, \; \forall i \ne j: \quad l_i \le |\{ \pi_i(L) \mid L \in ⟦ A ⟧ \wedge \pi_j(L) = x \}| \le u_i */
        constraint def Invariant_REQ_0058_Fiber_multiplicity_bounds;
        doc /* \text{Nav}_{j \to i}(x) = \begin{cases} \{ \pi_i(L) \mid L \in ⟦ A ⟧ \wedge \pi_j(L) = x \}, & \text{if } \text{nav}_i = \text{Navigable} \\ \bot, & \text{if } \text{nav}_i = \text{NonNavigable} \end{cases} */
        constraint def Invariant_REQ_0058_Navigational_projection;
        doc /* \begin{aligned} |\mathcal{E}_{A_{\text{sub}}}| = |\mathcal{E}_{A_{\text{base}}}| &\;\wedge\; \forall i \in \{1, \dots, k\}, \; \tau(e_i^{\text{sub}}) \le \tau(e_i^{\text{base}}) \\ &\;\wedge\; \forall i \in \{1, \dots, k\}, \; [l_i^{\text{sub}}, u_i^{\text{sub}}] \subseteq [l_i^{\text{base}}, u_i^{\text{base}}] \\ &\;\wedge\; \forall i \in \{1, \dots, k\}, \; (\text{nav}_i^{\text{base}} = \text{Navigable} \implies \text{nav}_i^{\text{sub}} = \text{Navigable}) \end{aligned} */
        constraint def Invariant_REQ_0058_Specialization_requires;
        doc /* \begin{aligned} \Delta_A &= \text{UUIDv5}(\text{OID}_{\text{DEAP}}, \text{FQN}(A)) \\ (k \lt 2) \;\vee\; \exists i \, (\neg(\tau(o_i) \le C_i)) \;\vee\; \text{FiberViolation} &\implies \text{Diagnostic}(A, \text{E0200}) \in \mathcal{D}_{\text{sink}} \end{aligned} */
        constraint def Invariant_REQ_0058_Deterministic_UUIDv5_topological_anchor_and_diagnostic_triggers;
        require Invariant_REQ_0058_FormalInvariant;
        require Invariant_REQ_0058_FormalInvariant_2;
        require Invariant_REQ_0058_FormalInvariant_3;
        require Invariant_REQ_0058_Fiber_multiplicity_bounds;
        require Invariant_REQ_0058_Navigational_projection;
        require Invariant_REQ_0058_Specialization_requires;
        require Invariant_REQ_0058_Deterministic_UUIDv5_topological_anchor_and_diagnostic_triggers;
        verify by AC_01_Multi_Ary_Association_Arity_Validation;
        verify by AC_02_Link_Participant_Type_Conformance;
        verify by AC_03_Fiber_Multiplicity_Bound_Enforcement;
        verify by AC_04_Non_Navigable_End_Traversal_Rejection;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: f8a99dba-f533-51eb-a7a0-08837836699c | Complexity: Class P (O(K + V)) | Standard: IEEE 29148-2018 / RFC 2119 / OMG SysML v2 | Diagnostics: E0100, E0199, E0200, E0201, E0202, E0203 */
    requirement def REQ_0059_KerML_Interaction {
        id = "REQ-0059";
        text = "The semantic lowering engine shall formalize occurrence communications and flow exchanges as Interaction classifiers in accordance with KerML. An Interaction classifier shall multiply specialize both Behavior and Association, combining behavioral occurrence execution semantics with relational";
        attribute uuidv5 : String = "f8a99dba-f533-51eb-a7a0-08837836699c";
        attribute complexity_class : String = "Class P (O(K + V))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119 / OMG SysML v2";
        attribute diagnostic_codes : String = "E0100, E0199, E0200, E0201, E0202, E0203";
        attribute ac_01_dual_inheritance_metamodel_lowering : String = "Given: A KerML AST declaring interaction classifier Interaction_Alpha specializing both Behavior and Association. When: The compiler executes the semantic lowering pass. Then: The lowered AST node exhibits dual generalization edges to Behavior and Association, inheriting occurrence execution and association endpoint semantics. Diagnostic: E0200 emitted on invalid or ambiguous generalization hierarchy.";
        attribute ac_02_participant_cardinality_lower_bound_enforcement : String = "Given: An interaction classifier Interaction_Alpha declaring k 2 participant end features (e.g., 1 participant Participant_0). When: Metamodel validation evaluates structural association endpoints. Then: The compiler rejects the definition and emits diagnostic error code E0200. Diagnostic: E0200 emitted when |P_{I}| 2.";
        attribute ac_03_message_transfer_directional_causality_verification : String = "Given: A message transfer step s_{transfer} between Participant_0 and Participant_1 with transmission event e_{send} and reception event e_{recv}. When: Temporal causality is evaluated where (e_{recv}) (e_{send}). Then: The lowering engine detects the temporal inversion and emits diagnostic error code E0201. Diagnostic: E0201 emitted when (e_{send}) (e_{recv}).";
        attribute ac_04_message_payload_subtype_conformance : String = "Given: A message transfer step s_{transfer} declared with classifier type Payload_Type_A and payload entity _{payload} of type Payload_Type_B. When: Type checking validates payload conformance against the transfer type contract. Then: If Payload_Type_B does not specialize Payload_Type_A ((_{payload}) _{sub} _{msg}), the engine emits diagnostic error code E0202; otherwise, compilation succeeds. Diagnostic: E0202 emitted on payload subtyping mismatch.";
        doc /* \text{Interaction} \le_{\text{sub}} \text{Behavior} \;\wedge\; \text{Interaction} \le_{\text{sub}} \text{Association} */
        constraint def Invariant_REQ_0059_Dual_Specialization_and_Taxonomy;
        doc /* \begin{aligned} \text{Behavior} &\le_{\text{sub}} \text{Class} \le_{\text{sub}} \text{Classifier} \le_{\text{sub}} \text{Type} \\ \text{Association} &\le_{\text{sub}} \text{Relationship} \;\wedge\; \text{Association} \le_{\text{sub}} \text{Classifier} \end{aligned} */
        constraint def Invariant_REQ_0059_FormalInvariant;
        doc /* I = \langle \text{id}_{I}, \mathcal{P}_{I}, \mathcal{S}_{I}, \prec_{I}, \Delta_{I} \rangle, \quad |\mathcal{P}_{I}| \ge 2 */
        constraint def Invariant_REQ_0059_Interaction_Entity_Formalization;
        doc /* \forall s_{1}, s_{2} \in \mathcal{S}_{I}, \quad (s_{1} \prec_{I} s_{2} \implies \neg(s_{2} \prec_{I} s_{1})) \;\wedge\; (s_{1} \prec_{I} s_{2} \implies \tau_{\text{end}}(s_{1}) \le \tau_{\text{start}}(s_{2})) */
        constraint def Invariant_REQ_0059_Step_Precedence_Poset_and_Temporal_Realization;
        doc /* \forall s_{\text{transfer}} \in \mathcal{S}_{I}, \quad \tau(e_{\text{send}}) \lt \tau(e_{\text{recv}}) \;\wedge\; \tau(\delta_{\text{payload}}) \le_{\text{sub}} \tau_{\text{msg}} */
        constraint def Invariant_REQ_0059_Message_Transfer_Causality_and_Subtyping;
        doc /* \forall s \in \mathcal{S}_{I}, \quad \neg(s \prec_{I}^{+} s) \iff \exists \sigma : \mathcal{S}_{I} \to \{1, \dots, |\mathcal{S}_{I}|\}, \quad \forall (u, v) \in \prec_{I}, \; \sigma(u) \lt \sigma(v) */
        constraint def Invariant_REQ_0059_Acyclicity_and_Topological_Ranking;
        doc /* \begin{aligned} |\mathcal{P}_{I}| \lt 2 &\implies \text{Diagnostic}(I, \text{E0200}) \\ \tau(e_{\text{send}}) \ge \tau(e_{\text{recv}}) &\implies \text{Diagnostic}(s_{\text{transfer}}, \text{E0201}) \\ \neg(\tau(\delta_{\text{payload}}) \le_{\text{sub}} \tau_{\text{msg}}) &\implies \text{Diagnostic}(s_{\text{transfer}}, \text{E0202}) \\ \exists s \in \mathcal{S}_{I} : s \prec_{I}^{+} s &\implies \text{Diagnostic}(I, \text{E0203}) \end{aligned} */
        constraint def Invariant_REQ_0059_Diagnostic_Code_Binding;
        doc /* \Delta_{I} = \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(I)) */
        constraint def Invariant_REQ_0059_Deterministic_UUIDv5_Anchor;
        require Invariant_REQ_0059_Dual_Specialization_and_Taxonomy;
        require Invariant_REQ_0059_FormalInvariant;
        require Invariant_REQ_0059_Interaction_Entity_Formalization;
        require Invariant_REQ_0059_Step_Precedence_Poset_and_Temporal_Realization;
        require Invariant_REQ_0059_Message_Transfer_Causality_and_Subtyping;
        require Invariant_REQ_0059_Acyclicity_and_Topological_Ranking;
        require Invariant_REQ_0059_Diagnostic_Code_Binding;
        require Invariant_REQ_0059_Deterministic_UUIDv5_Anchor;
        verify by AC_01_Dual_Inheritance_Metamodel_Lowering;
        verify by AC_02_Participant_Cardinality_Lower_Bound_Enforcement;
        verify by AC_03_Message_Transfer_Directional_Causality_Verification;
        verify by AC_04_Message_Payload_Subtype_Conformance;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: 87385097-1e99-5971-bbc0-c2ee18aae170 | Complexity: Class P (O(1)) | Standard: IEEE 29148-2018 / RFC 2119 / OMG SysML v2 | Diagnostics: E0199, E0200, E0201 */
    requirement def REQ_0060_KerML_Core_Features {
        id = "REQ-0060";
        text = "The semantic lowering engine and AST graph engine shall formalize properties, behavioral invocations, structural slots, directional parameters, and execution steps as instances of the unified Feature abstract syntax metaclass. In the KerML metamodel lowering pipeline, Feature shall specialize both";
        attribute uuidv5 : String = "87385097-1e99-5971-bbc0-c2ee18aae170";
        attribute complexity_class : String = "Class P (O(1))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119 / OMG SysML v2";
        attribute diagnostic_codes : String = "E0199, E0200, E0201";
        attribute ac_01_featuretyping_binding_value_projection_resolution : String = "Given: Classifier_Alpha declaring Feature_A typed by Classifier_Beta via FeatureTyping. When: Lowering executes feature type resolution. Then: Feature_A binds to Classifier_Beta with node handle resolution, valid projection mapping _{Feature_A}}, zero diagnostics emitted, and total-function completion. Diagnostic: E0200 emitted on unresolved feature type.";
        attribute ac_02_multiplicity_interval_bounds_validation : String = "Given: AST feature declarations with valid intervals [0..1], [1..*] and malformed interval [5..2] (m_{ m_{). When: Multiplicity well-formedness _{mult} is evaluated. Then: Valid intervals are accepted; malformed interval [5..2] is rejected and sentinel _{type} is assigned. Diagnostic: E0200 emitted on m_{ m_{.";
        attribute ac_03_fiber_cardinality_enforcement : String = "Given: Classifier_Alpha instance possessing Feature_A with multiplicity [1, 3] and evaluated instance fiber cardinality 0 or 4. When: The type verification engine validates fiber cardinality against (Feature_A}). Then: Cardinality violations outside [m_{, m_{] are detected and rejected. Diagnostic: E0200 emitted on fiber cardinality violation.";
        attribute ac_04_parameter_directionality_invocation_binding : String = "Given: Behavior_Alpha defining parameters Param_In (dir = In) and Param_Out (dir = Out). When: An invocation passes an immutable constant expression to Param_Out or attempts to write to Param_In. Then: Parameter direction mismatch is detected and invocation binding is rejected. Diagnostic: E0201 emitted for invalid parameter flow or mutability mismatch.";
        doc /* f = \langle \text{id}_f, \text{name}_f, \text{owner}_f, \tau(f), \mu(f), \text{dir}(f), \kappa_f \rangle */
        constraint def Invariant_REQ_0060_FormalInvariant;
        doc /* \pi_f : ⟦ C ⟧ \longrightarrow \mathcal{P}(⟦ \tau(f) ⟧) */
        constraint def Invariant_REQ_0060_Feature_evaluation_projection;
        doc /* \begin{aligned} \Phi_{\text{mult}}(f) &\iff (m_{\min}(f) \le m_{\max}(f)) \wedge (m_{\max}(f) \ge 1 \vee m_{\max}(f) = 0) \\ \Phi_{\text{fiber}}(f) &\iff \forall x \in ⟦ C ⟧, \quad m_{\min}(f) \le |\pi_f(x)| \le m_{\max}(f) \end{aligned} */
        constraint def Invariant_REQ_0060_Multiplicity_interval_and_fiber_cardinality_invariants;
        doc /* \begin{aligned} ⟦ \tau(f) ⟧ &= \bigcap_{T \in \tau(f)} ⟦ T ⟧, \quad \forall x \in ⟦ C ⟧, \; \pi_f(x) \subseteq ⟦ \tau(f) ⟧ \\ \kappa_f = \text{Step} &\implies \exists T \in \tau(f) : T \le \text{Behavior} \wedge T \ne \bot_{\text{type}} \end{aligned} */
        constraint def Invariant_REQ_0060_Multi_type_intersection_and_step_behavioral_typing_invariants;
        doc /* \text{ValidBinding}(a, p) \iff \begin{cases} (\tau(a) \le \tau(p)) \wedge \text{IsReadable}(a), & \text{dir}(p) = \text{In} \\ (\tau(p) \le \tau(a)) \wedge \text{IsAssignable}(a), & \text{dir}(p) = \text{Out} \\ (\tau(a) \equiv \tau(p)) \wedge \text{IsReadable}(a) \wedge \text{IsAssignable}(a), & \text{dir}(p) = \text{InOut} \end{cases} */
        constraint def Invariant_REQ_0060_Parameter_invocation_binding_validity;
        doc /* \begin{aligned} (\tau(f) = \emptyset \vee \bot_{\text{type}} \in \tau(f) \vee \neg \Phi_{\text{mult}}(f)) &\implies \text{Diagnostic}(f, \text{E0200}) \\ (\kappa_f = \text{Step} \wedge \forall T \in \tau(f), \, T \not\le \text{Behavior}) &\implies \text{Diagnostic}(f, \text{E0200}) \\ (\kappa_f = \text{Parameter} \wedge \neg \text{ValidBinding}(a, f)) &\implies \text{Diagnostic}(f, \text{E0201}) \end{aligned} */
        constraint def Invariant_REQ_0060_Diagnostic_mapping_invariant;
        require Invariant_REQ_0060_FormalInvariant;
        require Invariant_REQ_0060_Feature_evaluation_projection;
        require Invariant_REQ_0060_Multiplicity_interval_and_fiber_cardinality_invariants;
        require Invariant_REQ_0060_Multi_type_intersection_and_step_behavioral_typing_invariants;
        require Invariant_REQ_0060_Parameter_invocation_binding_validity;
        require Invariant_REQ_0060_Diagnostic_mapping_invariant;
        verify by AC_01_FeatureTyping_Binding_Value_Projection_Resolution;
        verify by AC_02_Multiplicity_Interval_Bounds_Validation;
        verify by AC_03_Fiber_Cardinality_Enforcement;
        verify by AC_04_Parameter_Directionality_Invocation_Binding;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: c8188a43-ccd5-570d-91e6-b128419ad088 | Complexity: Class P (O(V + E)) | Standard: IEEE 29148-2018 / RFC 2119 / OMG SysML v2 | Diagnostics: E0200, E0203 */
    requirement def REQ_0061_KerML_Subsetting_Relationship {
        id = "REQ-0061";
        text = "The compiler metamodel lowering engine and semantic type checker shall formalize KerML feature subsetting (Subsetting, denoted by operator :>) as value-set inclusion within the subtyping meet-semilattice. When a feature subsetting relationship f_{sub} :> f_{sup} is declared between subsetting";
        attribute uuidv5 : String = "c8188a43-ccd5-570d-91e6-b128419ad088";
        attribute complexity_class : String = "Class P (O(V + E))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119 / OMG SysML v2";
        attribute diagnostic_codes : String = "E0200, E0203";
        attribute ac_01_canonical_feature_subsetting_lowering : String = "Given: Classifier Classifier_Alpha declaring feature Feature_Sup : Classifier_Alpha [0..10], and specializing classifier Classifier_Beta :> Classifier_Alpha declaring Feature_Sub :> Feature_Sup : Classifier_Beta [1..5]. When: The compiler executes the metamodel lowering pass. Then: The subsetting relationship is verified with co-variant typing (Feature_Sub}) (Feature_Sup}) and valid multiplicity narrowing, synthesizing an AST relationship edge with deterministic UUIDv5 anchor and zero diagnostics. Diagnostic: None.";
        attribute ac_02_co_variant_subtyping_incompatibility_rejection : String = "Given: Supertype feature Feature_Sup : Classifier_Alpha and subsetting declaration Feature_Sub :> Feature_Sup : Classifier_Gamma, where Classifier_Gamma does not specialize Classifier_Alpha. When: The compiler performs subtyping verification on the subsetting declaration. Then: The compiler rejects the incompatible type relationship, binds error sentinel _{type} to prevent downstream failure cascade, and emits diagnostic code E0200. Diagnostic: E0200.";
        attribute ac_03_multiplicity_bound_narrowing_and_feasibility_violation : String = "Given: Feature Feature_Sup with multiplicity [0..5] and subsetting declarations specifying invalid intervals Feature_Sub1 :> Feature_Sup [0..8] (widened upper bound) or Feature_Sub2 :> Feature_Sup [6..8] (infeasible lower bound). When: The compiler evaluates multiplicity interval constraints during lowering. Then: The compiler rejects each invalid interval declaration and emits diagnostic code E0200. Diagnostic: E0200.";
        attribute ac_04_domain_specialization_and_directional_polarity_enforcement : String = "Given: Feature Feature_Sup in Classifier_Alpha and directed port Port_1 : in Classifier_Alpha. When: The compiler validates cross-classifier subsetting Feature_Sub :> Feature_Sup from disjoint Classifier_Gamma, or inverted directionality Port_2 :> Port_1 : out Classifier_Alpha. Then: The compiler rejects the domain mismatch and conflicting flow directionality, emitting diagnostic code E0200. Diagnostic: E0200.";
        doc /* f = \langle \text{id}_f, \text{name}_f, \text{owner}_f, \tau(f), \mu(f), \delta(f) \rangle */
        constraint def Invariant_REQ_0061_FormalInvariant;
        doc /* \begin{aligned} \forall (f_{\text{sub}}, f_{\text{sup}}) \in \mathcal{R}_{\text{subset}}, \quad &\forall x \in ⟦ \text{owner}(f_{\text{sub}}) ⟧ : \pi_{f_{\text{sub}}}(x) \subseteq \pi_{f_{\text{sup}}}(x) \\ \forall (f_{\text{sub}}, f_{\text{sup}}) \in \mathcal{R}_{\text{subset}}, \quad &\tau(f_{\text{sub}}) \le \tau(f_{\text{sup}}) \wedge \text{owner}(f_{\text{sub}}) \le \text{owner}(f_{\text{sup}}) \\ \forall (f_{\text{sub}}, f_{\text{sup}}) \in \mathcal{R}_{\text{subset}}, \quad &\mu(f_{\text{sub}}).m_{\max} \le \mu(f_{\text{sup}}).m_{\max} \wedge \mu(f_{\text{sub}}).m_{\min} \le \mu(f_{\text{sup}}).m_{\max} \\ \forall (f_{\text{sub}}, f_{\text{sup}}) \in \mathcal{R}_{\text{subset}}, \quad &\delta(f_{\text{sup}}) \ne \text{None} \implies \delta(f_{\text{sub}}) = \delta(f_{\text{sup}}) \end{aligned} */
        constraint def Invariant_REQ_0061_Value_Set_Containment_and_Poset_Invariants;
        doc /* \forall f \in \mathcal{F}, \quad (f, f) \notin \mathcal{R}_{\text{subset}}^+ */
        constraint def Invariant_REQ_0061_Subsetting_Graph_Strict_Acyclicity;
        doc /* \forall r = (f_{\text{sub}}, f_{\text{sup}}) \in \mathcal{R}_{\text{subset}}, \quad \Delta_{\text{rel}}(r) = \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(f_{\text{sub}}) \mathbin{\Vert} \text{"::subset::"} \mathbin{\Vert} \text{FQN}(f_{\text{sup}})) */
        constraint def Invariant_REQ_0061_Deterministic_RFC_4122_UUIDv5_Relationship_Anchor;
        require Invariant_REQ_0061_FormalInvariant;
        require Invariant_REQ_0061_Value_Set_Containment_and_Poset_Invariants;
        require Invariant_REQ_0061_Subsetting_Graph_Strict_Acyclicity;
        require Invariant_REQ_0061_Deterministic_RFC_4122_UUIDv5_Relationship_Anchor;
        verify by AC_01_Canonical_Feature_Subsetting_Lowering;
        verify by AC_02_Co_variant_Subtyping_Incompatibility_Rejection;
        verify by AC_03_Multiplicity_Bound_Narrowing_and_Feasibility_Violation;
        verify by AC_04_Domain_Specialization_and_Directional_Polarity_Enforcement;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: d300863e-c4c6-55b1-94c7-30ee8390eb56 | Complexity: Class P (O(V + E)) | Standard: IEEE 29148-2018 / RFC 2119 / OMG SysML v2 | Diagnostics: E0200, E0203 */
    requirement def REQ_0062_KerML_Redefinition_Relationship {
        id = "REQ-0062";
        text = "The metamodel lowering engine shall formalize the KerML and SysML v2 feature redefinition relationship (:>>, redefines) as contextual slot replacement over multiple inheritance directed acyclic graphs (DAGs), enforcing monotonic C3 superclass linearization, subtyping conformance, and multiplicity";
        attribute uuidv5 : String = "d300863e-c4c6-55b1-94c7-30ee8390eb56";
        attribute complexity_class : String = "Class P (O(V + E))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119 / OMG SysML v2";
        attribute diagnostic_codes : String = "E0200, E0203";
        attribute ac_01_contextual_slot_replacement_and_shadowing : String = "Given: Classifier Classifier_Alpha defining Feature_A : Real [0..10], and Classifier_Beta :> Classifier_Alpha declaring redefinition Feature_A_sub :>> Feature_A : Real [1..5]. When: Lowering resolves slot bindings within Classifier_Beta. Then: Reference lookup for Feature_A in Classifier_Beta resolves to Feature_A_sub, contextually shadowing Feature_A while preserving it in Classifier_Alpha. Diagnostic: E0200 emitted on slot resolution failure.";
        attribute ac_02_covariant_subtyping_and_multiplicity_interval_tightening : String = "Given: Classifier Classifier_Alpha defining Feature_A : Classifier_Alpha [2..8], and Classifier_Beta :> Classifier_Alpha declaring Feature_A_redef :>> Feature_A : Classifier_Beta [2..5]. When: Type checking evaluates covariance (Feature_A_redef}) (Feature_A}) and interval containment (Feature_A_redef}) (Feature_A}). Then: Conforming covariance and sub-intervals are accepted; contravariant types or widened intervals ([1..8] or [2..10]) are rejected with error sentinel _{redef}. Diagnostic: E0200 emitted on type non-conformance or multiplicity interval widening.";
        attribute ac_03_port_flow_direction_and_structural_invariance : String = "Given: Classifier Classifier_Alpha defining directed port Port_1 : in Classifier_Omega, and Classifier_Beta :> Classifier_Alpha declaring redefinition Port_1_redef :>> Port_1. When: Lowering validates port directionality and structural flags (isOrdered, isUnique). Then: Port_1_redef must preserve in polarity ( = In) unless conjugated (~); unauthorized direction inversion or weakening flags is rejected. Diagnostic: E0200 emitted on polarity mismatch (RedefinitionDirectionMismatch).";
        attribute ac_04_c3_monotonicity_diamond_disambiguation_and_acyclicity : String = "Given: Diamond inheritance hierarchy (Classifier_Beta :> Classifier_Alpha, Classifier_Gamma :> Classifier_Alpha, Classifier_Delta :> Classifier_Beta, Classifier_Gamma). When: The compiler computes C3(Classifier_Delta}), verifies acyclicity via Tarjan's algorithm, and resolves redefinitions of Feature_A. Then: Precedence resolves in O(V + E) time; conflicting unranked redefinitions trigger ambiguity errors; circular redefinitions (f_1 :>> f_2 :>> f_1) are rejected. Diagnostic: E0200 emitted on C3 inconsistency or diamond ambiguity; E0203 emitted on circular redefinition cycle.";
        doc /* f = \langle \text{id}_f, \text{name}_f, \text{owner}_f, \tau(f), \mu(f), \delta(f), \text{ord}(f), \text{uniq}(f), \mathcal{R}_f \rangle */
        constraint def Invariant_REQ_0062_FormalInvariant;
        doc /* \text{C3}(C) = [C] \mathbin{\Vert} \text{merge}(\text{C3}(P_1), \dots, \text{C3}(P_n), [P_1, \dots, P_n]) */
        constraint def Invariant_REQ_0062_Monotonic_C3_Linearization_Invariant;
        doc /* \begin{aligned} \tau(f_{\text{redef}}) &\le \tau(f_{\text{orig}}) \iff \tau(f_{\text{redef}}) \wedge \tau(f_{\text{orig}}) = \tau(f_{\text{redef}}) \\ \mu(f_{\text{redef}}) &\subseteq \mu(f_{\text{orig}}) \iff (l_{\text{orig}} \le l_{\text{redef}} \;\wedge\; u_{\text{redef}} \le u_{\text{orig}}) \\ \delta(f_{\text{orig}}) &\ne \text{None} \implies \delta(f_{\text{redef}}) = \delta(f_{\text{orig}}) \\ \text{ord}(f_{\text{orig}}) &\implies \text{ord}(f_{\text{redef}}) \\ \text{uniq}(f_{\text{orig}}) &\implies \text{uniq}(f_{\text{redef}}) \end{aligned} */
        constraint def Invariant_REQ_0062_Feature_Redefinition_Conformance_Invariant;
        doc /* \text{ResolveSlot}(C, s) = \begin{cases} f_{\text{redef}} & \text{if } \exists f_{\text{redef}} \in \text{Features}(C) : (f_{\text{redef}} :>> f_{\text{orig}}) \wedge (\text{name}(f_{\text{orig}}) = s) \\ f & \text{if } \exists f \in \text{Features}(C) : \text{name}(f) = s \\ \text{ResolveSlot}(P, s) & \text{where } P = \text{head}(\text{tail}(\text{C3}(C))) \text{ if inherited} \\ \bot_{\text{slot}} & \text{otherwise} \end{cases} */
        constraint def Invariant_REQ_0062_Contextual_Slot_Replacement_Mapping;
        doc /* \begin{aligned} (f, f) \in \mathcal{R}_{\text{redef}}^+ &\implies \text{Diagnostic}(f, \text{E0203}) \\ \tau(f_{\text{redef}}) \not\le \tau(f_{\text{orig}}) &\implies \text{Diagnostic}(f_{\text{redef}}, \text{E0200}) \\ \mu(f_{\text{redef}}) \not\subseteq \mu(f_{\text{orig}}) &\implies \text{Diagnostic}(f_{\text{redef}}, \text{E0200}) \\ \delta(f_{\text{orig}}) \ne \delta(f_{\text{redef}}) &\implies \text{Diagnostic}(f_{\text{redef}}, \text{E0200}) \\ \text{C3}(C) = \bot_{\text{c3}} &\implies \text{Diagnostic}(C, \text{E0200}) \end{aligned} */
        constraint def Invariant_REQ_0062_Acyclicity_and_Diagnostic_Binding;
        doc /* \text{UUID}(r) = \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(f_{\text{redef}}) \mathbin{\Vert} \text{":>>"} \mathbin{\Vert} \text{FQN}(f_{\text{orig}})) */
        constraint def Invariant_REQ_0062_Deterministic_RFC_4122_UUIDv5_Relationship_Anchor;
        require Invariant_REQ_0062_FormalInvariant;
        require Invariant_REQ_0062_Monotonic_C3_Linearization_Invariant;
        require Invariant_REQ_0062_Feature_Redefinition_Conformance_Invariant;
        require Invariant_REQ_0062_Contextual_Slot_Replacement_Mapping;
        require Invariant_REQ_0062_Acyclicity_and_Diagnostic_Binding;
        require Invariant_REQ_0062_Deterministic_RFC_4122_UUIDv5_Relationship_Anchor;
        derived from REQ_0063_KerML_Conjugation_Relationship_Lowering;
        verify by AC_01_Contextual_Slot_Replacement_and_Shadowing;
        verify by AC_02_Covariant_Subtyping_and_Multiplicity_Interval_Tightening;
        verify by AC_03_Port_Flow_Direction_and_Structural_Invariance;
        verify by AC_04_C3_Monotonicity_Diamond_Disambiguation_and_Acyclicity;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: abaf4675-a49f-5c64-82e2-bc657b129bf2 | Complexity: Class P (O(K) = O( | Standard: IEEE 29148-2018 / RFC 2119 / OMG SysML v2 | Diagnostics: E0100, E0199, E0200, E0201, E0203 */
    requirement def REQ_0063_KerML_Conjugation_Relationship_Lowering {
        id = "REQ-0063";
        text = "The semantic lowering engine and AST graph engine shall formalize the KerML conjugation relationship (~) and port inversion transformation across all port definitions, interface definitions, and port usages. The engine shall evaluate conjugation as an algebraic involution over the directional";
        attribute uuidv5 : String = "abaf4675-a49f-5c64-82e2-bc657b129bf2";
        attribute complexity_class : String = "Class P (O(K) = O(";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119 / OMG SysML v2";
        attribute diagnostic_codes : String = "E0100, E0199, E0200, E0201, E0203";
        attribute ac_01_algebraic_direction_involution_port_inversion : String = "Given: PortDef_Alpha declaring features Flow_In (In), Flow_Out (Out), and Flow_Bi (InOut). When: Lowering processes conjugated classifier ~PortDef_Alpha. Then: Containment features invert orientations: Flow_In becomes Out, Flow_Out becomes In, and Flow_Bi remains InOut. Diagnostic: E0201 emitted on directional inversion failure.";
        attribute ac_02_structural_payload_invariant_contract_preservation : String = "Given: PortDef_Alpha declaring feature Flow_A with payload type param_x : Real, multiplicity [1..2], and invariant constraints. When: Lowering lowers ~PortDef_Alpha into the AST. Then: The conjugated feature preserves payload type param_x : Real, multiplicity [1..2], and constraints identically. Diagnostic: E0200 emitted on payload schema or multiplicity mutation during conjugation.";
        attribute ac_03_involutive_simplification_normalization_gate : String = "Given: AST declarations containing nested operators ~~PortDef_Alpha and ~~~PortDef_Alpha. When: Lowering evaluates operator parity normalization. Then: ~~PortDef_Alpha simplifies to PortDef_Alpha and ~~~PortDef_Alpha simplifies to canonical ~PortDef_Alpha. Diagnostic: E0200 emitted if normalization fails to resolve canonical parity forms.";
        attribute ac_04_non_conjugable_target_classifier_rejection : String = "Given: An AST model applying ~ to non-conjugable classifier ~Classifier_Alpha (Real, Integer, or a class without ports). When: Lowering validates conjugation target applicability. Then: The compiler rejects the expression, assigns sentinel node _{type}, and emits diagnostic E0200. Diagnostic: E0200 emitted when ~ is applied to non-conjugable classifiers.";
        doc /* \begin{aligned} {\sim}(\text{In}) &= \text{Out}, \quad {\sim}(\text{Out}) = \text{In} \\ {\sim}(\text{InOut}) &= \text{InOut}, \quad {\sim}(\text{None}) = \text{None} \end{aligned} */
        constraint def Invariant_REQ_0063_FormalInvariant;
        doc /* \forall d \in \mathcal{D}, \quad {\sim}({\sim}d) \equiv d \implies {\sim}({\sim}P) \equiv P */
        constraint def Invariant_REQ_0063_Involution_identity;
        doc /* \begin{aligned} \text{features}(\widetilde{P}) &= \{ f' \mid \exists f \in \text{features}(P), \, \text{name}(f') = \text{name}(f) \wedge \tau(f') = \tau(f) \\ &\quad \wedge \mu(f') = \mu(f) \wedge \text{dir}(f') = {\sim}\text{dir}(f) \} \end{aligned} */
        constraint def Invariant_REQ_0063_For_port_classifier_its_conjugated_counterpart_satisfies;
        doc /* {\sim}^k P = \begin{cases} P, & k \equiv 0 \pmod 2 \\ {\sim}P, & k \equiv 1 \pmod 2 \end{cases} */
        constraint def Invariant_REQ_0063_Consecutive_conjugation_simplification;
        doc /* P_1 \le P_2 \iff {\sim}P_1 \le {\sim}P_2, \quad {\sim}(P_1 \wedge P_2) \equiv {\sim}P_1 \wedge {\sim}P_2 */
        constraint def Invariant_REQ_0063_Lattice_order_preserving_automorphism;
        doc /* \begin{aligned} \text{IsConjugated}(C) \wedge \text{Kind}(C) \notin \{ \text{PortDef}, \text{InterfaceDef}, \text{PortUsage} \} &\implies \text{Diagnostic}(C, \text{E0200}) \\ \text{Connected}(p_1, p_2) \wedge \text{dir}(p_1) = \text{dir}(p_2) \wedge \text{dir}(p_1) \ne \text{InOut} &\implies \text{Diagnostic}(\langle p_1, p_2 \rangle, \text{E0201}) \\ \text{HasCycle}(\text{C3Linearize}(P)) &\implies \text{Diagnostic}(P, \text{E0203}) \end{aligned} */
        constraint def Invariant_REQ_0063_Diagnostic_mapping_rules;
        require Invariant_REQ_0063_FormalInvariant;
        require Invariant_REQ_0063_Involution_identity;
        require Invariant_REQ_0063_For_port_classifier_its_conjugated_counterpart_satisfies;
        require Invariant_REQ_0063_Consecutive_conjugation_simplification;
        require Invariant_REQ_0063_Lattice_order_preserving_automorphism;
        require Invariant_REQ_0063_Diagnostic_mapping_rules;
        verify by AC_01_Algebraic_Direction_Involution_Port_Inversion;
        verify by AC_02_Structural_Payload_Invariant_Contract_Preservation;
        verify by AC_03_Involutive_Simplification_Normalization_Gate;
        verify by AC_04_Non_Conjugable_Target_Classifier_Rejection;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: 279a50da-8ed5-54c9-b690-386f5f3e45f8 | Complexity: Class P (O(1)) | Standard: IEEE 29148-2018 / RFC 2119 / OMG SysML v2 | Diagnostics: E0199, E0200 */
    requirement def REQ_0064_KerML_Disjoining_Differencing_Intersecting {
        id = "REQ-0064";
        text = "The semantic type checker and metamodel lowering engine shall implement a formal Boolean algebra over classifier type denotations, providing set-theoretic type operators (unioning, intersecting, differencing, disjoining) to construct compound classifiers and enforce static disjointness invariants.";
        attribute uuidv5 : String = "279a50da-8ed5-54c9-b690-386f5f3e45f8";
        attribute complexity_class : String = "Class P (O(1))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119 / OMG SysML v2";
        attribute diagnostic_codes : String = "E0199, E0200";
        attribute ac_01_unioning_compound_type_subtyping_invariant : String = "Given: Classifiers Classifier_Alpha and Classifier_Beta in Package_0. When: Lowering processes Classifier_Alpha unioning Classifier_Beta. Then: Engine emits compound type T_{union} with ⟦ T_{union} ⟧ = ⟦ Classifier_Alpha} ⟧ ⟦ Classifier_Beta} ⟧, enforces covariance Classifier_Alpha} T_{union} and Classifier_Beta} T_{union}, and binds deterministic RFC 4122 UUIDv5. Diagnostic: E0200 emitted on invalid or unresolvable operand types.";
        attribute ac_02_intersecting_compound_type_conformance_invariant : String = "Given: Classifiers Classifier_Alpha and Classifier_Beta in Package_0. When: Lowering processes Classifier_Alpha intersecting Classifier_Beta. Then: Engine emits intersection type T_{inter} with ⟦ T_{inter} ⟧ = ⟦ Classifier_Alpha} ⟧ ⟦ Classifier_Beta} ⟧, enforces contravariance T_{inter} Classifier_Alpha} and T_{inter} Classifier_Beta}, preserving feature conjunction. Diagnostic: E0200 emitted on invalid operand types.";
        attribute ac_03_differencing_relative_complement_evaluation : String = "Given: Specializing classifiers where Classifier_Beta} Classifier_Alpha}. When: Lowering evaluates Classifier_Alpha differencing Classifier_Beta. Then: Denotation satisfies ⟦ Classifier_Alpha} Classifier_Beta} ⟧ = ⟦ Classifier_Alpha} ⟧ ⟦ Classifier_Beta} ⟧; declaring an instance on empty difference Classifier_Beta} Classifier_Alpha} binds _{type}. Diagnostic: E0200 emitted on uninhabitable difference type instantiation.";
        attribute ac_04_disjoining_mutual_exclusion_enforcement : String = "Given: Classifiers declared mutually disjoint via disjoining Classifier_Alpha, Classifier_Beta. When: Classifier Classifier_Gamma specializes both Classifier_Alpha and Classifier_Beta. Then: Engine detects Classifier_Alpha} Classifier_Beta}, identifies empty intersection ⟦ Classifier_Gamma} ⟧ , binds sentinel _{type}, and recovers at delimiter. Diagnostic: E0200 emitted with source span identifying the disjoint type conflict.";
        doc /* \begin{aligned} ⟦ T_1 \cup T_2 ⟧ &= ⟦ T_1 ⟧ \cup ⟦ T_2 ⟧, & T_1 \le (T_1 \cup T_2) \wedge T_2 \le (T_1 \cup T_2) \\ ⟦ T_1 \cap T_2 ⟧ &= ⟦ T_1 ⟧ \cap ⟦ T_2 ⟧, & (T_1 \cap T_2) \le T_1 \wedge (T_1 \cap T_2) \le T_2 \\ ⟦ T_1 \setminus T_2 ⟧ &= ⟦ T_1 ⟧ \setminus ⟦ T_2 ⟧, & (T_1 \setminus T_2) \le T_1 \wedge (T_1 \setminus T_2) \perp T_2 \\ T_1 \perp T_2 &\iff ⟦ T_1 ⟧ \cap ⟦ T_2 ⟧ = \emptyset, & (T_1 \perp T_2 \wedge S_1 \le T_1 \wedge S_2 \le T_2) \implies S_1 \perp S_2 \\ T_1 \le T_2 &\iff ⟦ T_1 ⟧ \subseteq ⟦ T_2 ⟧ \iff ⟦ T_1 \setminus T_2 ⟧ = \emptyset \end{aligned} */
        constraint def Invariant_REQ_0064_FormalInvariant;
        doc /* \Delta_{T_1 \odot T_2} = \text{UUIDv5}(\text{OID}_{\text{DEAP}}, \text{FQN}(T_1) \mathbin{\Vert} \text{"::op::"} \mathbin{\Vert} \text{FQN}(T_2)) */
        constraint def Invariant_REQ_0064_Deterministic_RFC_4122_UUIDv5_identification_and_diagnostic_trigger_invariants_per_REQ_0191;
        doc /* \begin{aligned} (T_1 \perp T_2 \wedge \exists C \in \mathcal{T} : C \le T_1 \wedge C \le T_2) &\implies \text{Diagnostic}(C, \text{E0200}) \in \mathcal{D}_{\text{sink}} \\ (T_1 \perp T_2 \wedge \exists x : x : T_1 \wedge x : T_2) &\implies \text{Diagnostic}(x, \text{E0200}) \in \mathcal{D}_{\text{sink}} \\ (T_1 \le T_2 \wedge \exists x : x : (T_1 \setminus T_2)) &\implies \text{Diagnostic}(x, \text{E0200}) \in \mathcal{D}_{\text{sink}} \\ |\mathcal{D}_{\text{sink}}| \ge C_{\text{max\_errors}} &\implies \text{DiagnosticCode::E0199} \in \mathcal{D}_{\text{sink}} \end{aligned} */
        constraint def Invariant_REQ_0064_FormalInvariant_2;
        require Invariant_REQ_0064_FormalInvariant;
        require Invariant_REQ_0064_Deterministic_RFC_4122_UUIDv5_identification_and_diagnostic_trigger_invariants_per_REQ_0191;
        require Invariant_REQ_0064_FormalInvariant_2;
        derived from REQ_0191_Standardized_Compiler_Diagnostic_Taxonomy_Source_Mapping;
        verify by AC_01_Unioning_Compound_Type_Subtyping_Invariant;
        verify by AC_02_Intersecting_Compound_Type_Conformance_Invariant;
        verify by AC_03_Differencing_Relative_Complement_Evaluation;
        verify by AC_04_Disjoining_Mutual_Exclusion_Enforcement;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: f0d53f25-68f4-50ab-ba79-c951ae49494f | Complexity: Class P (O(1)) | Standard: IEEE 29148-2018 / RFC 2119 / OMG SysML v2 | Diagnostics: E0199, E0200, E0201, E0203 */
    requirement def REQ_0065_KerML_Expressions {
        id = "REQ-0065";
        text = "The KerML expression lowering engine and semantic type checker shall transform KerML expressions into an immutable, strongly typed abstract syntax tree (AST) rooted in Expression. Expression shall specialize Step, Feature, Classifier, and Type, formalizing every expression as a computational step";
        attribute uuidv5 : String = "f0d53f25-68f4-50ab-ba79-c951ae49494f";
        attribute complexity_class : String = "Class P (O(1))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119 / OMG SysML v2";
        attribute diagnostic_codes : String = "E0199, E0200, E0201, E0203";
        attribute ac_01_literal_ingestion_feature_navigation_typing : String = "Given: Literals (true, 42, 3.14159, \"symbol_token\", *, null) and scoped feature reference Classifier_Alpha.param_x : Real. When: Lowering executes literal ingestion and scoped identifier resolution. Then: Literals receive standard library scalar types, strings are interned, valid references resolve to type Real, and ungrounded identifiers or missing members are rejected. Diagnostic: E0200 emitted on unresolved identifier, missing member, or literal overflow.";
        attribute ac_02_operator_conformance_division_by_zero_conditional_unification : String = "Given: Operations param_x + 5.0, static zero division param_x / 0.0, invalid addition \"token\" + 42, and conditional if flag then param_x else 0.0. When: The type checker validates operator typing and conditional branch unification. Then: Valid expressions type-check successfully; operand mismatches, static division by literal zero, and un-unifiable conditional branches are rejected at compile time. Diagnostic: E0200 emitted on operator signature mismatch, division by zero, or branch unification failure.";
        attribute ac_03_invocation_parameter_directionality_mutability_lvalue_gate : String = "Given: Function Function_Alpha(in p_in : Integer, out p_out : Integer, inout p_io : Integer) : Boolean. When: Invocations bind arguments to parameters, including binding constant literal 10 or expression param_x + 1 to p_out or p_io. Then: Valid bindings are accepted; arity or type mismatches emit E0200; passing immutable rvalues to Out or InOut parameters is rejected. Diagnostic: E0200 emitted on arity or type mismatch; E0201 emitted on parameter directionality or mutability violation.";
        attribute ac_04_referential_transparency_cycle_detection_error_recovery : String = "Given: Pure expression Expression_1, circular constant chain (c_1 = c_2 + 1, c_2 = c_1 * 2), and error limit C_{max_errors}} = 50. When: The compiler executes constant evaluation, cycle detection, and error recovery. Then: Pure expression evaluation preserves store state (_{post} _{pre}); cyclic constants emit E0203 with precise spans; sentinel _{expr} is substituted, and reaching C_{max_errors}} emits E0199 and halts. Diagnostic: E0203 emitted on circular constant dependency; E0199 emitted when diagnostic capacity reaches C_{max_errors}}.";
        doc /* \text{Expression} \le_{\text{sub}} \text{Step} \le_{\text{sub}} \text{Feature} \le_{\text{sub}} \text{Type} */
        constraint def Invariant_REQ_0065_Metamodel_Specialization_Hierarchy;
        doc /* E = \langle \text{id}_E, \kappa_E, \tau(E), \mu(E), \text{Children}_E, \Delta_E, \text{span}_E, \pi_{\text{pure}} \rangle */
        constraint def Invariant_REQ_0065_Expression_Node_Formalization;
        doc /* \begin{aligned} \Gamma \vdash E : T &\implies \forall \sigma \in \Sigma_{\text{store}}, \quad \text{eval}(E, \sigma) = (v, \sigma') \implies v \in ⟦ T ⟧ \wedge (\pi_{\text{pure}}(E) = \text{Pure} \implies \sigma' \equiv \sigma) \\ \Gamma \vdash E_1 \odot E_2 : T_{\text{res}} &\iff (\tau(E_1), \tau(E_2)) \in \text{dom}(\text{op}_{\odot}) \wedge T_{\text{res}} = \text{codom}(\text{op}_{\odot}) \\ \Gamma \vdash (\text{if } E_{\text{c}} \text{ then } E_1 \text{ else } E_2) : (T_1 \vee T_2) &\iff \Gamma \vdash E_{\text{c}} : \text{Boolean} \wedge \Gamma \vdash E_1 : T_1 \wedge \Gamma \vdash E_2 : T_2 \end{aligned} */
        constraint def Invariant_REQ_0065_Typing_Soundness_Referential_Transparency_and_Operator_Invariants;
        doc /* \begin{aligned} \text{ValidArg}(a_i, p_i) &\iff \begin{cases} (\tau(a_i) \le \tau(p_i)) \wedge \text{IsReadable}(a_i), & \text{dir}(p_i) = \text{In} \\ (\tau(p_i) \le \tau(a_i)) \wedge \text{IsAssignable}(a_i), & \text{dir}(p_i) = \text{Out} \\ (\tau(a_i) \equiv \tau(p_i)) \wedge \text{IsReadable}(a_i) \wedge \text{IsAssignable}(a_i), & \text{dir}(p_i) = \text{InOut} \end{cases} \\ \Delta_E &= \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(E) \mathbin{\Vert} \text{Discriminant}(\kappa_E) \mathbin{\Vert} \text{id}_E) \\ (\kappa_E = \text{BinaryOp}(/, E_1, E_2) \wedge \text{IsZero}(E_2)) &\implies \text{Diagnostic}(E, \text{E0200}) \in \mathcal{D}_{\text{sink}} \\ (\kappa_E = \text{Invocation}(F, A) \wedge \exists i : \neg \text{ValidArg}(a_i, p_i) \wedge \text{dir}(p_i) \ne \text{In}) &\implies \text{Diagnostic}(a_i, \text{E0201}) \in \mathcal{D}_{\text{sink}} \\ (E \to^+ E \text{ in constant dependency DAG}) &\implies \text{Diagnostic}(E, \text{E0203}) \in \mathcal{D}_{\text{sink}} \end{aligned} */
        constraint def Invariant_REQ_0065_Invocation_Directionality_and_Diagnostic_Mapping_Invariants;
        require Invariant_REQ_0065_Metamodel_Specialization_Hierarchy;
        require Invariant_REQ_0065_Expression_Node_Formalization;
        require Invariant_REQ_0065_Typing_Soundness_Referential_Transparency_and_Operator_Invariants;
        require Invariant_REQ_0065_Invocation_Directionality_and_Diagnostic_Mapping_Invariants;
        derived from REQ_0036_Concurrent_String_Interning;
        derived from REQ_0010_Deterministic_RFC_4122_UUIDv5_Topological_Namespace_Hashing;
        verify by AC_01_Literal_Ingestion_Feature_Navigation_Typing;
        verify by AC_02_Operator_Conformance_Division_by_Zero_Conditional_Unification;
        verify by AC_03_Invocation_Parameter_Directionality_Mutability_LValue_Gate;
        verify by AC_04_Referential_Transparency_Cycle_Detection_Error_Recovery;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: 1b6581e1-15ff-536e-b6b3-3e19ca1cce4e | Complexity: Class P (O(1)) | Standard: IEEE 29148-2018 / RFC 2119 / OMG SysML v2 | Diagnostics: E0100, E0199, E0200, E0201, E0203 */
    requirement def REQ_0066_KerML_Functions {
        id = "REQ-0066";
        text = "The functional lowering engine of Subsystem 4 shall formalize KerML Function classifiers as pure mathematical morphisms from input parameter domains to output result codomains. A Function specializes Behavior (and transitively Class, Classifier, Type) under referential transparency, total";
        attribute uuidv5 : String = "1b6581e1-15ff-536e-b6b3-3e19ca1cce4e";
        attribute complexity_class : String = "Class P (O(1))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119 / OMG SysML v2";
        attribute diagnostic_codes : String = "E0100, E0199, E0200, E0201, E0203";
        attribute ac_01_pure_mathematical_morphism_referential_transparency : String = "Given: Function Function_Alpha(Param_1 : Type_A, Param_2 : Type_B) : Type_A declaring pure expression body E_{body}. When: Functional lowering evaluates morphism ⟦ Function_Alpha} ⟧ under store . Then: Input parameters are immutable, evaluation produces identical outputs for identical inputs (a = b ⟦ F ⟧(a) = ⟦ F ⟧(b)), _{post} _{pre}, and zero diagnostics are emitted. Diagnostic: E0200 emitted on side-effect detection or state mutation within a function body.";
        attribute ac_02_parameter_directionality_prohibition_of_inout : String = "Given: Model declaring Function_Beta containing a formal parameter with direction inout. When: Semantic parameter validation inspects directionality tags dir(p) { In, Out, Return }. Then: The compiler rejects Function_Beta, binds error sentinel _{type}, and emits diagnostic E0201. Diagnostic: E0201 emitted on illegal inout parameter declaration on a Function.";
        attribute ac_03_return_type_specialization_body_expression_conformance : String = "Given: Function Function_Alpha with return type Type_A, whose body expression evaluates to Type_B. When: Semantic type checking evaluates subtyping conformance (E_{body}) T_{ret}. Then: If Type_B specializes Type_A, compilation succeeds; otherwise, the compiler rejects the function, binds _{type}, and emits diagnostic E0200. Diagnostic: E0200 emitted on return type incompatibility or unresolved return/parameter types.";
        attribute ac_04_call_site_arity_argument_subtyping_conformance : String = "Given: Function Function_Alpha(Param_1 : Type_A, Param_2 : Type_B) : Type_A and calls I_1 = Function_Alpha}(a_1), I_2 = Function_Alpha}(a_1, a_2, a_3), and I_3 = Function_Alpha}(a_1, a_{invalid}). When: Call-site validation checks arity and argument types against formal signatures. Then: Arity mismatches (I_1, I_2) and argument type mismatch (I_3) are rejected with diagnostic E0200. Diagnostic: E0200 emitted on invocation arity mismatch or argument type incompatibility.";
        doc /* \text{Function} \le \text{Behavior} \le \text{Class} \le \text{Classifier} \le \text{Type} */
        constraint def Invariant_REQ_0066_FormalInvariant;
        doc /* ⟦ F ⟧ : \prod_{i=1}^k ⟦ T_i ⟧ \longrightarrow ⟦ T_{\text{ret}} ⟧ */
        constraint def Invariant_REQ_0066_A_function_with_inputs_x_1;
        doc /* \begin{aligned} \forall \mathbf{a}, \mathbf{b} \in \prod_{i=1}^k ⟦ T_i ⟧, \quad \mathbf{a} = \mathbf{b} &\implies ⟦ F ⟧(\mathbf{a}) = ⟦ F ⟧(\mathbf{b}) \\ \text{Reads}(F) \subseteq \{ x_1, \dots, x_k \} \cup \mathcal{C}_{\text{const}} \;\wedge\; \text{Writes}(F) = \emptyset &\implies \sigma_{\text{post}} \equiv \sigma_{\text{pre}} \end{aligned} */
        constraint def Invariant_REQ_0066_Referential_transparency_and_store_invariance;
        doc /* \begin{aligned} \forall p \in \mathcal{P}_{\text{in}}(F), \, \text{dir}(p) = \text{In} &\;\wedge\; \text{dir}(p_{\text{ret}}(F)) \in \{ \text{Return}, \text{Out} \} \;\wedge\; \forall p \in \text{Params}(F), \, \text{dir}(p) \ne \text{InOut} \\ \Gamma \vdash E_{\text{body}}(F) : T_{\text{eval}} &\implies T_{\text{eval}} \le T_{\text{ret}} \;\wedge\; T_{\text{ret}} \ne \bot_{\text{type}} \end{aligned} */
        constraint def Invariant_REQ_0066_Parameter_directionality_return_typing_conformance_and_non_mutation_satisfy;
        doc /* \begin{aligned} \text{ValidInvocation}(I, F) &\iff (m = k) \;\wedge\; \left( \forall i \in \{ 1, \dots, k \}, \, \Gamma \vdash a_i : T_{a_i} \;\wedge\; T_{a_i} \le T_i \right) \\ \forall (F, F) \in \mathcal{E}_{\text{call}}^+, &\quad \exists \mu_{\text{term}}, \, \forall \mathbf{x}, \, \text{RecCond}(\mathbf{x}) \implies \mu_{\text{term}}(\text{RecArgs}(\mathbf{x})) \lt \mu_{\text{term}}(\mathbf{x}) \end{aligned} */
        constraint def Invariant_REQ_0066_Invocation_validity_and_recursion_well_foundedness_satisfy;
        doc /* \forall u \in \mathcal{V}_{\text{func}} \cup \mathcal{V}_{\text{param}}, \quad \Delta_u = \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(u)) */
        constraint def Invariant_REQ_0066_Deterministic_RFC_4122_UUIDv5_topological_anchoring_satisfies;
        require Invariant_REQ_0066_FormalInvariant;
        require Invariant_REQ_0066_A_function_with_inputs_x_1;
        require Invariant_REQ_0066_Referential_transparency_and_store_invariance;
        require Invariant_REQ_0066_Parameter_directionality_return_typing_conformance_and_non_mutation_satisfy;
        require Invariant_REQ_0066_Invocation_validity_and_recursion_well_foundedness_satisfy;
        require Invariant_REQ_0066_Deterministic_RFC_4122_UUIDv5_topological_anchoring_satisfies;
        verify by AC_01_Pure_Mathematical_Morphism_Referential_Transparency;
        verify by AC_02_Parameter_Directionality_Prohibition_of_inout;
        verify by AC_03_Return_Type_Specialization_Body_Expression_Conformance;
        verify by AC_04_Call_Site_Arity_Argument_Subtyping_Conformance;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: 467f497c-eb92-5193-bb98-1be296c8f403 | Complexity: Class P (O(V + E)) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG KerML §8.2.4 | Diagnostics: E0100, E0200, E0203 */
    requirement def REQ_0067_KerML_Occurrences {
        id = "REQ-0067";
        text = "The KerML lowering engine SHALL formalize spatio-temporal entities as OccurrenceDefinition (specializing Class) and OccurrenceUsage (specializing Feature and Occurrence). Every OccurrenceUsage MUST be typed by one or more OccurrenceDefinition classifiers via FeatureTyping; typing with";
        attribute uuidv5 : String = "467f497c-eb92-5193-bb98-1be296c8f403";
        attribute complexity_class : String = "Class P (O(V + E))";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG KerML §8.2.4";
        attribute diagnostic_codes : String = "E0100, E0200, E0203";
        attribute ac_01_canonical_occurrence_lowering_and_temporal_extent_binding : String = "Given: A model declaring OccurrenceDef_Alpha with extent [0.0, 100.0] and nested OccurrenceUsage_1 : OccurrenceDef_Alpha [1..1] with extent [10.0, 50.0]. When: The compiler executes occurrence lowering and type validation. Then: Both elements are lowered into contiguous arena storage with valid 32-bit node handles, correct feature typing, and zero diagnostic emissions. Diagnostic: E0200 emitted if type resolution or temporal extent binding fails.";
        attribute ac_02_non_occurrence_classifier_type_rejection : String = "Given: A structural primitive or identity-free data type DataType_Beta. When: An occurrence usage is declared typing a non-occurrence classifier (OccurrenceUsage_Err : DataType_Beta). Then: The compiler rejects the declaration at compile time, assigns an error sentinel, and records diagnostic error code E0200. Diagnostic: E0200 emitted for invalid occurrence classifier typing.";
        attribute ac_03_temporal_interval_bounds_and_containment_violations : String = "Given: An occurrence with inverted bounds [50.0, 10.0] (t_{start} t_{end}), or a sub-occurrence OccurrenceUsage_Child with extent [10.0, 60.0] exceeding parent OccurrenceDef_Alpha extent [0.0, 50.0]. When: The static semantic validation pass evaluates temporal well-formedness and containment. Then: The compiler rejects the invalid interval or containment violation with exact source byte spans and diagnostic error code E0200. Diagnostic: E0200 emitted for temporal interval boundary or containment violations.";
        attribute ac_04_sub_occurrence_containment_acyclicity_verification : String = "Given: A synthetic cyclic sub-occurrence containment chain (OccurrenceUsage_1 containing OccurrenceUsage_2 containing OccurrenceUsage_1). When: The topological cycle detection pass traverses the containment relation graph. Then: The compiler detects the circular dependency, halts containment processing for the cycle, and emits diagnostic error code E0203 detailing participating nodes. Diagnostic: E0203 emitted for circular occurrence containment dependency.";
        doc /* \begin{aligned} D &= \langle \text{id}_D, \text{name}_D, \mathcal{S}_D, \mathcal{F}_D, \mathcal{T}_D, \Delta_D \rangle \in \mathcal{O}_{\text{def}} \\ U &= \langle \text{id}_U, \text{name}_U, \text{owner}_U, \tau(U), \mu(U), \mathcal{T}_U, \Delta_U \rangle \in \mathcal{O}_{\text{use}} \end{aligned} */
        constraint def Invariant_REQ_0067_FormalInvariant;
        doc /* \pi_{\text{time}}(\mathcal{W}(o)) = [t_{\text{start}}(o), t_{\text{end}}(o)] \subset \mathbb{R}, \quad t_{\text{start}}(o) \le t_{\text{end}}(o) */
        constraint def Invariant_REQ_0067_FormalInvariant_2;
        doc /* \forall (o_1, o_2) \in \mathcal{R}_{\text{sub\_occ}}, \quad \mathcal{W}(o_1) \subseteq \mathcal{W}(o_2) \implies [t_{\text{start}}(o_1), t_{\text{end}}(o_1)] \subseteq [t_{\text{start}}(o_2), t_{\text{end}}(o_2)] */
        constraint def Invariant_REQ_0067_Sub_Occurrence_Manifold_and_Temporal_Interval_Containment_Invariant;
        doc /* \forall (o_1, o_2) \in \mathcal{R}_{\text{sub\_occ}}, \quad t_{\text{start}}(o_2) \le t_{\text{start}}(o_1) \le t_{\text{end}}(o_1) \le t_{\text{end}}(o_2) */
        constraint def Invariant_REQ_0067_which_decomposes_to;
        doc /* \begin{aligned} \forall o \in \mathcal{O}_{\text{all}}, \quad &(o, o) \notin \mathcal{R}_{\text{sub\_occ}}^+ \\ \forall U \in \mathcal{O}_{\text{use}}, \quad &\tau(U) \subseteq \mathcal{O}_{\text{def}} \quad \wedge \quad 0 \le m_{\min}(U) \le m_{\max}(U) \\ \forall n \in \mathcal{O}_{\text{all}}, \quad &\Delta_n = \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0067_Acyclicity_Multiplicity_and_RFC_4122_UUIDv5_Invariants;
        require Invariant_REQ_0067_FormalInvariant;
        require Invariant_REQ_0067_FormalInvariant_2;
        require Invariant_REQ_0067_Sub_Occurrence_Manifold_and_Temporal_Interval_Containment_Invariant;
        require Invariant_REQ_0067_which_decomposes_to;
        require Invariant_REQ_0067_Acyclicity_Multiplicity_and_RFC_4122_UUIDv5_Invariants;
        verify by AC_01_Canonical_Occurrence_Lowering_and_Temporal_Extent_Binding;
        verify by AC_02_Non_Occurrence_Classifier_Type_Rejection;
        verify by AC_03_Temporal_Interval_Bounds_and_Containment_Violations;
        verify by AC_04_Sub_Occurrence_Containment_Acyclicity_Verification;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: ed698792-bf46-5dad-a504-389464496131 | Complexity: Class P (O(1) temporal slice check, O(N) occurrence traversal) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG KerML §8.2.4 | Diagnostics: E0100, E0199, E0200 */
    requirement def REQ_0068_KerML_PortionKind_TimeSlice_Snapshot {
        id = "REQ-0068";
        text = "The KerML lowering engine SHALL formalize occurrence decomposition operators, lowering snapshot, time slice, and event occurrence constructs into rigorous geometric intersections over 4-dimensional occurrence world-lines and world-volumes (W \subseteq \mathbb{R}^3 \times \mathbb{R}). An occurrence";
        attribute uuidv5 : String = "ed698792-bf46-5dad-a504-389464496131";
        attribute complexity_class : String = "Class P (O(1) temporal slice check, O(N) occurrence traversal)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG KerML §8.2.4";
        attribute diagnostic_codes : String = "E0100, E0199, E0200";
        attribute ac_01_canonical_snapshot_timeslice_eventoccurrence_lowering : String = "Given: Occurrence Occurrence_Alpha with lifetime [0.0, 100.0], containing Snapshot_T0 at t0 = 25.0, TimeSlice_Phase1 over [10.0, 40.0], and EventOccurrence_Ev1 at te = 15.0 with boolean predicate param_x > 0.0. When: The compiler executes occurrence portion lowering and validation. Then: All three constructs lower into contiguous arena storage with valid 32-bit handles, verified temporal bounds, and zero diagnostic emissions. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_temporal_lifetime_breach_rejection : String = "Given: Occurrence Occurrence_Alpha with lifetime [0.0, 100.0]. When: A snapshot Snapshot_T1 is declared at t0 = 120.0 (t_0 t_{end}) or time slice TimeSlice_Phase2 spans [-10.0, 50.0] (t_1 t_{start}). Then: The compiler rejects the out-of-bounds declarations, binds error sentinels, and records diagnostic code E0200. Diagnostic: E0200 emitted for temporal lifetime boundary breach.";
        attribute ac_03_inverted_timeslice_interval_rejection : String = "Given: Occurrence Occurrence_Alpha with lifetime [0.0, 100.0]. When: A time slice TimeSlice_Inv declares inverted bounds [60.0, 20.0] (t_1 t_2). Then: The compiler rejects the inverted interval and emits diagnostic code E0200. Diagnostic: E0200 emitted for inverted time slice interval.";
        attribute ac_04_non_boolean_event_trigger_predicate_rejection : String = "Given: Occurrence Occurrence_Alpha with lifetime [0.0, 100.0]. When: An event occurrence EventOccurrence_Bad declares trigger expression param_x + 1.0 typed as Real instead of Boolean. Then: The compiler rejects the non-boolean trigger predicate and records diagnostic code E0200. Diagnostic: E0200 emitted for invalid trigger expression type.";
        doc /* \begin{aligned} \text{Snapshot}(O, t_0) &= \mathcal{W}(O) \cap \{ (\mathbf{x}, t) \in \mathbb{R}^3 \times \mathbb{R} \mid t = t_0 \} \\ \text{TimeSlice}(O, t_1, t_2) &= \mathcal{W}(O) \cap \{ (\mathbf{x}, t) \in \mathbb{R}^3 \times \mathbb{R} \mid t_1 \le t \le t_2 \} \\ \text{EventOccurrence}(O, t_e, \Phi) &= \{ (\mathbf{x}, t) \in \mathcal{W}(O) \mid t = t_e \wedge \Phi(\mathbf{x}, t_e) = \text{true} \} \end{aligned} */
        constraint def Invariant_REQ_0068_FormalInvariant;
        doc /* \begin{aligned} \lambda_t(\pi_t(\text{Snapshot}(O, t_0))) &= 0 \\ \lambda_t(\pi_t(\text{TimeSlice}(O, t_1, t_2))) &= t_2 - t_1 \ge 0 \\ \lambda_t(\pi_t(\text{EventOccurrence}(O, t_e, \Phi))) &= 0 \\ \forall t_a, t_b \in \mathcal{T}(O), \quad t_a \ne t_b &\implies \text{Snapshot}(O, t_a) \cap \text{Snapshot}(O, t_b) = \emptyset \end{aligned} */
        constraint def Invariant_REQ_0068_Temporal_Lebesgue_measure_and_pairwise_disjointness_invariants;
        doc /* \begin{aligned} P_{\text{sub}} \sqsubseteq_{\text{portion}} P_{\text{sup}} \sqsubseteq_{\text{portion}} O &\implies \mathcal{W}(P_{\text{sub}}) \subseteq \mathcal{W}(P_{\text{sup}}) \subseteq \mathcal{W}(O) \\ \forall n \in \mathcal{V}_{\text{portion}}, \quad \Delta_n &= \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0068_Portioning_transitivity_and_RFC_4122_UUIDv5_topological_anchoring_invariants;
        require Invariant_REQ_0068_FormalInvariant;
        require Invariant_REQ_0068_Temporal_Lebesgue_measure_and_pairwise_disjointness_invariants;
        require Invariant_REQ_0068_Portioning_transitivity_and_RFC_4122_UUIDv5_topological_anchoring_invariants;
        verify by AC_01_Canonical_Snapshot_TimeSlice_EventOccurrence_Lowering;
        verify by AC_02_Temporal_Lifetime_Breach_Rejection;
        verify by AC_03_Inverted_TimeSlice_Interval_Rejection;
        verify by AC_04_Non_Boolean_Event_Trigger_Predicate_Rejection;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: d81b9904-d2b0-5a93-b2b9-f6f7a9c809c4 | Complexity: Class P (O(1) step transition check, O(V + E) control flow traversal) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG KerML §8.2.5 | Diagnostics: E0100, E0199, E0200, E0201, E0202 */
    requirement def REQ_0069_KerML_Behaviors {
        id = "REQ-0069";
        text = "The behavioral lowering engine of Subsystem 4 SHALL formalize behaviors, actions, invocations, and temporal performance occurrences in strict accordance with the OMG Kernel Modeling Language (KerML) and Systems Modeling Language (SysML v2) specifications. Behavior SHALL specialize Class (and";
        attribute uuidv5 : String = "d81b9904-d2b0-5a93-b2b9-f6f7a9c809c4";
        attribute complexity_class : String = "Class P (O(1) step transition check, O(V + E) control flow traversal)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG KerML §8.2.5";
        attribute diagnostic_codes : String = "E0100, E0199, E0200, E0201, E0202";
        attribute ac_01_canonical_behavior_actionusage_performance_lowering : String = "Given: A model declaring Behavior_Alpha with parameter Param_1 : Real [In] and nested Step_Alpha : Behavior_Alpha. When: The compiler lowers the behavior definition and action step into the AST. Then: Both elements are lowered with valid 32-bit handles, verified parameter types, and zero diagnostic emissions. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_non_behavioral_step_type_rejection : String = "Given: A structural classifier Classifier_Alpha that does not specialize Behavior. When: A step is declared with invalid typing (Step_Bad : Classifier_Alpha). Then: The compiler rejects the step declaration, binds error sentinel _{type}, and emits diagnostic code E0200. Diagnostic: E0200 emitted for non-behavioral step typing.";
        attribute ac_03_parameter_subtyping_directionality_violation_rejection : String = "Given: Behavior_Alpha declaring parameter Param_In : Real [In]. When: An invocation passes an argument with incompatible type Classifier_Beta or attempts to mutate Param_In inside the body. Then: The compiler rejects the invocation with diagnostic code E0202 on subtyping failure or E0201 on illegal mutation. Diagnostic: E0201 emitted for directionality breach; E0202 emitted for subtyping mismatch.";
        attribute ac_04_sub_step_temporal_containment_boundary_breach : String = "Given: Performance Performance_1 with lifetime [10.0, 50.0]. When: A constituent sub-step Step_Child declares temporal execution bounds [5.0, 30.0] (t_{start} 10.0) or [20.0, 60.0] (t_{end} 50.0). Then: The compiler rejects the containment violation with exact source byte spans and emits diagnostic code E0200. Diagnostic: E0200 emitted for sub-step temporal containment breach.";
        doc /* \begin{aligned} \mathcal{Q}_{\text{perf}} &= \{ \text{Unstarted}, \text{Executing}, \text{Suspended}, \text{Completed}, \text{Terminated} \} \\ \delta_{\text{perf}} &: \mathcal{Q}_{\text{perf}} \times \Sigma_{\text{event}} \longrightarrow \mathcal{Q}_{\text{perf}} \\ \forall s \in \Sigma, \quad \text{Pre}(s(t_{\text{start}})) &\equiv \text{true} \implies \text{Post}(s(t_{\text{start}}), s(t_{\text{end}})) \equiv \text{true} \end{aligned} */
        constraint def Invariant_REQ_0069_FormalInvariant;
        doc /* \begin{aligned} \forall s_i \in \mathcal{S}_B, \quad [t_{\text{start}}(s_i), t_{\text{end}}(s_i)] &\subseteq [t_{\text{start}}(P), t_{\text{end}}(P)] \\ \forall a_k \in \text{Args}(s), \quad \tau(a_k) &\le \tau(p_k) \\ \forall n \in \mathcal{V}_{\text{perf}}, \quad \Delta_n &= \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0069_Sub_step_temporal_containment_parameter_subtyping_and_RFC_4122_UUIDv5_invariants;
        require Invariant_REQ_0069_FormalInvariant;
        require Invariant_REQ_0069_Sub_step_temporal_containment_parameter_subtyping_and_RFC_4122_UUIDv5_invariants;
        verify by AC_01_Canonical_Behavior_ActionUsage_Performance_Lowering;
        verify by AC_02_Non_Behavioral_Step_Type_Rejection;
        verify by AC_03_Parameter_Subtyping_Directionality_Violation_Rejection;
        verify by AC_04_Sub_Step_Temporal_Containment_Boundary_Breach;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: cb9f13bb-7ca2-5402-b696-cc49dff8fafd | Complexity: Class P (O(V + E) topological sort and visibility checks) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.3 | Diagnostics: E0100, E0199, E0200, E0203 */
    requirement def REQ_0070_SysML_v2_Package {
        id = "REQ-0070";
        text = "The SysML v2 model organization engine SHALL structure models into modular package and subsystem hierarchies, formalizing visibility boundaries, package imports, and encapsulation boundaries as an acyclic package dependency graph. A package declaration (package <name> {... }) defines a sovereign";
        attribute uuidv5 : String = "cb9f13bb-7ca2-5402-b696-cc49dff8fafd";
        attribute complexity_class : String = "Class P (O(V + E) topological sort and visibility checks)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.3";
        attribute diagnostic_codes : String = "E0100, E0199, E0200, E0203";
        attribute ac_01_canonical_package_hierarchy_public_export_lowering : String = "Given: A valid model declaring Package_0 containing public Classifier_Alpha and a distinct Package_1 with import Package_0::*;. When: The compiler executes package lowering and import symbol resolution. Then: Package_0 and Package_1 are lowered into the AST, and Classifier_Alpha is successfully resolved within Package_1 with zero diagnostics. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_private_symbol_encapsulation_breach_rejection : String = "Given: Package_0 declaring private member Classifier_Beta (private Classifier_Beta). When: An external Package_1 attempts to reference or import Package_0::Classifier_Beta. Then: The compiler rejects the foreign access, records the offending source byte span, and emits diagnostic code E0200. Diagnostic: E0200 emitted for private symbol encapsulation breach.";
        attribute ac_03_circular_package_import_dependency_detection : String = "Given: Two packages declaring mutual imports (Package_0 imports Package_1 and Package_1 imports Package_0). When: The compiler constructs and traverses the package dependency graph G_{pkg}. Then: The cycle is detected, topological sorting halts for the circular component, and diagnostic code E0203 is emitted. Diagnostic: E0203 emitted for circular package dependency.";
        attribute ac_04_bounded_package_nesting_depth_enforcement : String = "Given: A deeply nested package hierarchy declaring 17 nested packages (Package_0::Package_1::...::Package_16). When: The compiler validates package lexical nesting depth. Then: The compiler detects depth D = 17 16, halts nested package lowering, and emits diagnostic code E0200. Diagnostic: E0200 emitted for exceeding maximum package nesting depth.";
        doc /* \begin{aligned} G_{\text{pkg}} &= (\mathcal{P}, \mathcal{E}_{\text{dep}}), \quad \mathcal{E}_{\text{dep}} = \mathcal{D}_{\text{imp}} \cup \mathcal{D}_{\text{ref}} \\ \text{ExportedSymbols}(P) &= \{ s \in \text{Symbols}(P) \mid \text{visibility}(s) = \text{Public} \} \\ \text{PrivateSymbols}(P) &= \{ s \in \text{Symbols}(P) \mid \text{visibility}(s) = \text{Private} \} \end{aligned} */
        constraint def Invariant_REQ_0070_FormalInvariant;
        doc /* \begin{aligned} \forall P \in \mathcal{P}, \quad &(P, P) \notin \mathcal{E}_{\text{dep}}^+ \\ \forall P \in \mathcal{P}, \quad &\text{Depth}(P) \le 16 \\ \forall n \in \mathcal{P}, \quad &\Delta_n = \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0070_Acyclicity_nesting_depth_and_RFC_4122_UUIDv5_invariants;
        require Invariant_REQ_0070_FormalInvariant;
        require Invariant_REQ_0070_Acyclicity_nesting_depth_and_RFC_4122_UUIDv5_invariants;
        verify by AC_01_Canonical_Package_Hierarchy_Public_Export_Lowering;
        verify by AC_02_Private_Symbol_Encapsulation_Breach_Rejection;
        verify by AC_03_Circular_Package_Import_Dependency_Detection;
        verify by AC_04_Bounded_Package_Nesting_Depth_Enforcement;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: eacf8cba-181e-5c3c-9c99-0a174c5e85d9 | Complexity: Class P (O(V + E)) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.3 | Diagnostics: E0100, E0199, E0200, E0203 */
    requirement def REQ_0071_SysML_v2_PartDefinition {
        id = "REQ-0071";
        text = "The structural compiler SHALL formalize SysML v2 part definitions (part def, PartDefinition) and part usages (part, PartUsage) into strongly typed, compositely owned structural elements anchored within the core KerML classifier subtyping lattice (K, \le). PartDefinition SHALL specialize KerML";
        attribute uuidv5 : String = "eacf8cba-181e-5c3c-9c99-0a174c5e85d9";
        attribute complexity_class : String = "Class P (O(V + E))";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.3";
        attribute diagnostic_codes : String = "E0100, E0199, E0200, E0203";
        attribute ac_01_canonical_part_definition_and_composite_usage_lowering : String = "Given: A valid schema declaring PartDef_Alpha containing composite part Part_1 : PartDef_Beta [1..*]. When: The compiler executes metamodel lowering and structural validation. Then: Both elements are lowered into contiguous arena storage with is_composite = true, default multiplicity preserved, and zero diagnostic emissions. Diagnostic: E0200 emitted if typing or composite ownership fails.";
        attribute ac_02_rejection_of_datatype_inheritance : String = "Given: A part definition declared specializing a primitive value type (PartDef_Err : DataType_Beta). When: Semantic type checking is evaluated against the classifier lattice. Then: The compiler rejects the declaration at compile time and emits diagnostic error code E0200. Diagnostic: E0200 emitted for identity-free value type inheritance on part definitions.";
        attribute ac_03_malformed_multiplicity_range_rejection : String = "Given: A part usage declared with inverted multiplicity bounds part Part_Inv : PartDef_Alpha [5..2]. When: Semantic validation checks multiplicity interval bounds. Then: The compiler rejects the inverted interval and emits diagnostic error code E0200. Diagnostic: E0200 emitted for invalid multiplicity interval (m_{ m_{).";
        attribute ac_04_sub_assembly_containment_cycle_detection : String = "Given: A recursive physical containment cycle (PartDef_Alpha containing Part_0 : PartDef_Beta [1..1], and PartDef_Beta containing Part_1 : PartDef_Alpha [1..1]). When: The topological cycle detection pass traverses the containment graph. Then: The compiler detects the cyclic dependency with non-zero lower multiplicity and emits diagnostic error code E0203. Diagnostic: E0203 emitted for circular composite containment.";
        doc /* \begin{aligned} \text{PartDef} &\le \text{Structure} \le \text{Class} \le \text{Classifier} \\ ⟦ \text{PartDef} ⟧ &\cap ⟦ \text{DataType} ⟧ = \emptyset \end{aligned} */
        constraint def Invariant_REQ_0071_FormalInvariant;
        doc /* \begin{aligned} \forall x \in ⟦ D ⟧, \quad \text{Destroy}(x) &\implies \forall y \in \pi_u(x), \quad \text{Destroy}(y) \\ \forall y \in \mathcal{O}_{\text{part}}, \quad &|\text{owning\_composite}(y)| \le 1 \end{aligned} */
        constraint def Invariant_REQ_0071_Existential_lifetime_dependency_and_anti_aliasing_invariant;
        doc /* \begin{aligned} \forall D \in \mathcal{D}_{\text{part}}, \quad &(D, D) \notin \mathcal{E}_{\text{contain}}^+ \quad \text{where} \quad m_{\min} \ge 1 \\ \forall n \in \mathcal{D}_{\text{part}} \cup \mathcal{U}_{\text{part}}, \quad &\Delta_n = \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0071_Sub_assembly_containment_graph_acyclicity_and_RFC_4122_UUIDv5_invariants;
        require Invariant_REQ_0071_FormalInvariant;
        require Invariant_REQ_0071_Existential_lifetime_dependency_and_anti_aliasing_invariant;
        require Invariant_REQ_0071_Sub_assembly_containment_graph_acyclicity_and_RFC_4122_UUIDv5_invariants;
        verify by AC_01_Canonical_Part_Definition_and_Composite_Usage_Lowering;
        verify by AC_02_Rejection_of_DataType_Inheritance;
        verify by AC_03_Malformed_Multiplicity_Range_Rejection;
        verify by AC_04_Sub_Assembly_Containment_Cycle_Detection;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: b3469699-3d65-50e9-b35a-e85caa063106 | Complexity: Class P (O(1) port resolution, O(F) feature conjugation) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.4 | Diagnostics: E0100, E0199, E0200 */
    requirement def REQ_0072_SysML_v2_PortDefinition {
        id = "REQ-0072";
        text = "The structural compiler SHALL formalize SysML v2 port definitions (port def, PortDefinition) and port usages (port, PortUsage) as interaction point classifiers specializing KerML Structure. A port definition P defines an interface protocol specifying typed directed features. Every port usage p";
        attribute uuidv5 : String = "b3469699-3d65-50e9-b35a-e85caa063106";
        attribute complexity_class : String = "Class P (O(1) port resolution, O(F) feature conjugation)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.4";
        attribute diagnostic_codes : String = "E0100, E0199, E0200";
        attribute ac_01_canonical_port_definition_and_conjugate_usage_lowering : String = "Given: A port definition PortDef_Alpha declaring directed features in Flow_A : Real and out Flow_B : Real, and usage port Port_1 : ~PortDef_Alpha. When: The compiler executes port lowering and interface conjugation. Then: Port usage Port_1 is lowered into arena storage with inverted feature directions (out Flow_A and in Flow_B) and zero diagnostic emissions. Diagnostic: E0200 emitted if conjugate feature expansion fails.";
        attribute ac_02_non_port_classifier_conjugation_rejection : String = "Given: A part usage attempting to apply conjugation to a non-port classifier (port Port_Err : ~PartDef_Beta). When: Semantic type checking validates the conjugation target. Then: The compiler rejects the declaration at compile time and emits diagnostic error code E0200. Diagnostic: E0200 emitted for invalid conjugation on non-port classifier.";
        attribute ac_03_boundary_encapsulation_breach_rejection : String = "Given: An external connector attempting to bind directly to internal constituent Part_Alpha.InternalVar bypassing declared boundary ports. When: Structural connection routing validates endpoint visibility. Then: The compiler halts connection lowering and emits diagnostic error code E0200. Diagnostic: E0200 emitted for boundary encapsulation violation.";
        attribute ac_04_port_multiplicity_and_direction_verification : String = "Given: A port usage declared with invalid direction specifier or inverted multiplicity bounds port Port_Bad : PortDef_Alpha [3..1]. When: Semantic validation checks port multiplicity intervals. Then: The compiler rejects the malformed declaration with exact source spans and diagnostic error code E0200. Diagnostic: E0200 emitted for invalid port multiplicity interval.";
        doc /* \begin{aligned} \text{PortDefinition} &\le \text{Structure} \le \text{Class} \\ p \in \mathcal{U}_{\text{port}} &\implies \tau(p) \le \text{PortDefinition} \quad \wedge \quad \text{dir}(p) \in \{ \text{In}, \text{Out}, \text{InOut} \} \end{aligned} */
        constraint def Invariant_REQ_0072_FormalInvariant;
        doc /* \begin{aligned} p : {\sim}P \implies \forall f \in \text{Features}(P), \quad \text{dir}(\pi_f(p)) = {\sim}\text{dir}(f) \\ {\sim}({\sim}d) = d \quad \forall d \in \{ \text{In}, \text{Out}, \text{InOut} \} \end{aligned} */
        constraint def Invariant_REQ_0072_Conjugate_port_interface_direction_inversion_invariant;
        doc /* \begin{aligned} \forall (e_1, e_2) \in \mathcal{E}_{\text{conn}}, \quad e_1 \in \text{External}(C) &\implies e_2 \in \text{Ports}(C) \\ \forall n \in \mathcal{D}_{\text{port}} \cup \mathcal{U}_{\text{port}}, \quad \Delta_n &= \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0072_Boundary_encapsulation_and_RFC_4122_UUIDv5_invariants;
        require Invariant_REQ_0072_FormalInvariant;
        require Invariant_REQ_0072_Conjugate_port_interface_direction_inversion_invariant;
        require Invariant_REQ_0072_Boundary_encapsulation_and_RFC_4122_UUIDv5_invariants;
        verify by AC_01_Canonical_Port_Definition_and_Conjugate_Usage_Lowering;
        verify by AC_02_Non_Port_Classifier_Conjugation_Rejection;
        verify by AC_03_Boundary_Encapsulation_Breach_Rejection;
        verify by AC_04_Port_Multiplicity_and_Direction_Verification;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: 81ec112f-82ed-5723-96a1-8d571346d02c | Complexity: Class P (O(1) item typing, O(F) flow conservation) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.5 | Diagnostics: E0100, E0199, E0200 */
    requirement def REQ_0073_SysML_v2_ItemDefinition {
        id = "REQ-0073";
        text = "The data and flow modeling engine SHALL formalize SysML v2 item definitions (item def, ItemDefinition) and item usages (item, ItemUsage) as structural classifiers specializing KerML Structure. Item usages represent physical, energetic, or informational payloads flowing across interconnection";
        attribute uuidv5 : String = "81ec112f-82ed-5723-96a1-8d571346d02c";
        attribute complexity_class : String = "Class P (O(1) item typing, O(F) flow conservation)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.5";
        attribute diagnostic_codes : String = "E0100, E0199, E0200";
        attribute ac_01_canonical_value_item_and_discrete_token_lowering : String = "Given: An immutable value item ItemDef_Alpha : DataType and a discrete token item ItemDef_Beta : Class. When: The compiler executes item definition lowering and classification. Then: Both elements are lowered into arena storage with correct value/token duality tags and zero diagnostic emissions. Diagnostic: E0200 emitted if classifier duality resolution fails.";
        attribute ac_02_non_structural_item_classifier_rejection : String = "Given: An item definition attempting to specialize a behavioral step classifier (ItemDef_Err : Step). When: Semantic type checking validates metamodel classifier conformance. Then: The compiler rejects the declaration at compile time and emits diagnostic error code E0200. Diagnostic: E0200 emitted for invalid item metamodel specialization.";
        attribute ac_03_conservative_flow_balance_violation_detection : String = "Given: A junction node with incoming flow rate Flow_In = 10.0 and outgoing flow rate Flow_Out = 7.0 for a conservative item. When: Semantic flow validation evaluates conservation balance equations. Then: The compiler detects the unbuffered rate discrepancy and emits diagnostic error code E0200. Diagnostic: E0200 emitted for conservative flow conservation imbalance.";
        attribute ac_04_item_multiplicity_range_conformance : String = "Given: An item usage declared with inverted cardinality bounds item Item_0 : ItemDef_Alpha [4..1]. When: Multiplicity interval validation is executed. Then: The compiler rejects the inverted interval with exact source spans and diagnostic error code E0200. Diagnostic: E0200 emitted for invalid item multiplicity interval.";
        doc /* \begin{aligned} \text{ItemDefinition} &\le \text{Structure} \\ \mathcal{D}_{\text{item}} &= \mathcal{D}_{\text{item\_val}} \uplus \mathcal{D}_{\text{item\_tok}} \\ \mathcal{D}_{\text{item\_val}} &\subseteq \{ D \in \mathcal{D}_{\text{item}} \mid D \le \text{DataType} \} \\ \mathcal{D}_{\text{item\_tok}} &\subseteq \{ D \in \mathcal{D}_{\text{item}} \mid D \le \text{Class} \} \end{aligned} */
        constraint def Invariant_REQ_0073_FormalInvariant;
        doc /* \begin{aligned} \forall v \in \mathcal{V}_{\text{flow}}, \quad &\sum_{e \in \text{InEdges}(v)} \text{rate}(e) = \sum_{e \in \text{OutEdges}(v)} \text{rate}(e) \\ \forall n \in \mathcal{D}_{\text{item}} \cup \mathcal{U}_{\text{item}}, \quad &\Delta_n = \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0073_Conservative_flow_balance_and_token_conservation_invariant;
        require Invariant_REQ_0073_FormalInvariant;
        require Invariant_REQ_0073_Conservative_flow_balance_and_token_conservation_invariant;
        verify by AC_01_Canonical_Value_Item_and_Discrete_Token_Lowering;
        verify by AC_02_Non_Structural_Item_Classifier_Rejection;
        verify by AC_03_Conservative_Flow_Balance_Violation_Detection;
        verify by AC_04_Item_Multiplicity_Range_Conformance;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: fccf8a19-6097-5281-abc1-4509cf95499a | Complexity: Class P (O(1) endpoint lookup, O(E) topology validation) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.6 | Diagnostics: E0100, E0199, E0200 */
    requirement def REQ_0074_SysML_v2_ConnectionDefinition {
        id = "REQ-0074";
        text = "The interconnect engine SHALL formalize SysML v2 connection definitions (connection def, ConnectionDefinition) and connection usages (connection, connect, Connector) as relational association structures binding two or more port endpoints. ConnectionDefinition SHALL specialize KerML";
        attribute uuidv5 : String = "fccf8a19-6097-5281-abc1-4509cf95499a";
        attribute complexity_class : String = "Class P (O(1) endpoint lookup, O(E) topology validation)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.6";
        attribute diagnostic_codes : String = "E0100, E0199, E0200";
        attribute ac_01_canonical_connection_lowering_and_conjugate_compatibility : String = "Given: A connection connect Part_Alpha.Port_Out to Part_Beta.Port_In where Port_Out : PortDef_Alpha and Port_In : ~PortDef_Alpha. When: The compiler executes connection lowering and interface compatibility verification. Then: The connector is lowered into arena storage with bound endpoints and zero diagnostic emissions. Diagnostic: E0200 emitted if conjugate compatibility check fails.";
        attribute ac_02_conflicting_directional_flow_rejection : String = "Given: A connection binding two output ports directly together (connect Part_Alpha.Port_Out1 to Part_Beta.Port_Out2). When: Semantic directionality validation analyzes endpoint interaction modes. Then: The compiler rejects the conflicting output connection at compile time and emits diagnostic error code E0200. Diagnostic: E0200 emitted for conflicting port flow directions.";
        attribute ac_03_incompatible_port_type_rejection : String = "Given: A connection binding incompatible port definitions PortDef_Alpha and PortDef_Beta where PortDef_Alpha} {PortDef_Beta}. When: Semantic type checking validates endpoint conjugate conformance. Then: The compiler rejects the connector declaration and emits diagnostic error code E0200. Diagnostic: E0200 emitted for port type incompatibility.";
        attribute ac_04_out_of_scope_dangling_endpoint_detection : String = "Given: A connection referencing an inaccessible or non-existent port path Part_Alpha.NonExistentPort. When: Endpoint namespace resolution is evaluated. Then: The compiler halts connection lowering for the dangling endpoint and emits diagnostic error code E0200. Diagnostic: E0200 emitted for unresolved or out-of-scope connection endpoint.";
        doc /* \begin{aligned} \text{ConnectionDefinition} &\le \text{AssociationStructure} \le \text{Structure} \\ C = (p_1, p_2) \in \mathcal{E}_{\text{conn}} &\implies \tau(p_1) \le {\sim}\tau(p_2) \end{aligned} */
        constraint def Invariant_REQ_0074_FormalInvariant;
        doc /* \begin{aligned} \text{dir}(p_1) = \text{Out} &\implies \text{dir}(p_2) \in \{ \text{In}, \text{InOut} \} \\ \text{dir}(p_1) = \text{In} &\implies \text{dir}(p_2) \in \{ \text{Out}, \text{InOut} \} \\ \text{LCA}(p_1, p_2) &\in \text{Ancestors}(C) \cup \{ \text{Owner}(C) \} \end{aligned} */
        constraint def Invariant_REQ_0074_Directional_flow_legality_and_topological_endpoint_scoping_invariant;
        doc /* \begin{aligned} |\text{Endpoints}(C)| &\ge 2 \\ \forall n \in \mathcal{D}_{\text{conn}} \cup \mathcal{E}_{\text{conn}}, \quad \Delta_n &= \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0074_Multi_point_connector_and_RFC_4122_UUIDv5_invariants;
        require Invariant_REQ_0074_FormalInvariant;
        require Invariant_REQ_0074_Directional_flow_legality_and_topological_endpoint_scoping_invariant;
        require Invariant_REQ_0074_Multi_point_connector_and_RFC_4122_UUIDv5_invariants;
        verify by AC_01_Canonical_Connection_Lowering_and_Conjugate_Compatibility;
        verify by AC_02_Conflicting_Directional_Flow_Rejection;
        verify by AC_03_Incompatible_Port_Type_Rejection;
        verify by AC_04_Out_of_Scope_Dangling_Endpoint_Detection;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: 175a4875-f152-5c8c-90d6-403719d5a38a | Complexity: Class P (O(1) conjugate check, O(V + E) C3) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.6 | Diagnostics: E0100, E0199, E0200, E0201, E0202, E0220 */
    requirement def REQ_0075_SysML_v2_InterfaceDefinition {
        id = "REQ-0075";
        text = "The protocol engine SHALL formalize SysML v2 interface definitions (interface def) as contractual association structures specifying complementary end-roles, and interface usages (interface) as communication protocol bindings between component ports. InterfaceDefinition SHALL specialize KerML";
        attribute uuidv5 : String = "175a4875-f152-5c8c-90d6-403719d5a38a";
        attribute complexity_class : String = "Class P (O(1) conjugate check, O(V + E) C3)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.6";
        attribute diagnostic_codes : String = "E0100, E0199, E0200, E0201, E0202, E0220";
        attribute ac_01_canonical_lowering_and_binding : String = "Given: InterfaceDef_Alpha (ends e1 : PortDef_0, e2 : ~PortDef_0) and ports Port_1 : PortDef_0, Port_2 : ~PortDef_0. When: Lowering interface InterfaceUsage_1 : InterfaceDef_Alpha connect part_1.Port_1 to part_2.Port_2;. Then: Usage lowers into arena storage with bound endpoints and empty diagnostic sink. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_non_conjugate_end_role_rejection : String = "Given: Definition interface def InterfaceDef_Invalid { end e1 : PortDef_0; end e2 : PortDef_0; }. When: Validating end-role conjugate duality. Then: Lowering rejects definition and emits diagnostic E0201. Diagnostic: E0201 on conjugate duality mismatch.";
        attribute ac_03_incompatible_port_binding_rejection : String = "Given: InterfaceDef_Alpha and binding Port_1 : PortDef_0 to Port_3 : PortDef_Beta (PortDef_Beta} {PortDef_0}). When: Verifying port binding subtyping conformance. Then: Lowering rejects binding and emits diagnostic E0201. Diagnostic: E0201 on port binding subtyping mismatch.";
        attribute ac_04_dangling_end_role_and_arity_rejection : String = "Given: Ternary definition interface def InterfaceDef_Ternary, or usage binding only one port. When: Validating arity and endpoint completeness. Then: Lowering rejects ternary definition with E0200 and dangling usage with E0220. Diagnostic: E0200 on invalid arity; E0220 on dangling usage.";
        doc /* \begin{aligned} \text{InterfaceDefinition} &\le \text{ConnectionDefinition} \le \text{AssociationStructure} \\ \text{InterfaceUsage} &\le \text{ConnectionUsage} \le \text{Feature} \\ \text{ends}(I) = (e_1, e_2) &\implies \tau(e_2) \equiv {\sim}\tau(e_1) \\ u : I \wedge \mathcal{B}_u = (p_1, p_2) &\implies \text{type}(p_1) \le \tau(e_1) \wedge \text{type}(p_2) \le \tau(e_2) \\ |\text{ends}(I)| = 2 &\wedge |\mathcal{B}_u| = 2 \\ \forall w \in \mathcal{F}_I, \quad \tau(\text{payload}(w)) &\le \tau(\text{flow}(w)) \\ \forall n \in \mathcal{D}_{\text{intf}} \cup \mathcal{U}_{\text{intf}}, \quad \Delta_n &= \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0075_FormalInvariant;
        require Invariant_REQ_0075_FormalInvariant;
        verify by AC_01_Canonical_Lowering_and_Binding;
        verify by AC_02_Non_Conjugate_End_Role_Rejection;
        verify by AC_03_Incompatible_Port_Binding_Rejection;
        verify by AC_04_Dangling_End_Role_and_Arity_Rejection;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: 5f2c1a0d-6d71-5b4e-bb8d-6b418141e90e | Complexity: Class P (O(1) directionality, O(F) flow checking) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.6 | Diagnostics: E0100, E0199, E0200, E0201, E0202 */
    requirement def REQ_0076_SysML_v2_ItemFlow {
        id = "REQ-0076";
        text = "The metamodel lowering engine SHALL formalize SysML v2 flow usages (ItemFlow) as directed transfers traversing connectors between port endpoints, specializing KerML Step and OccurrenceUsage in arena storage.";
        attribute uuidv5 : String = "5f2c1a0d-6d71-5b4e-bb8d-6b418141e90e";
        attribute complexity_class : String = "Class P (O(1) directionality, O(F) flow checking)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.6";
        attribute diagnostic_codes : String = "E0100, E0199, E0200, E0201, E0202";
        attribute ac_01_canonical_itemflow_lowering : String = "Given: Component Part_0 with Port_Src : out Item_0 and Port_Tgt : in Item_0 connected by Conn_0. When: Lowering flow of Item_0 from Port_Src to Port_Tgt. Then: Flow lowers as KerML Step and OccurrenceUsage with zero diagnostics. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_conflicting_port_direction_rejection : String = "Given: Flow streaming from Port_Src : in Item_0 to Port_Tgt : in Item_0. When: Directionality validation evaluates endpoint modes. Then: Compiler rejects the declaration with diagnostic E0201. Diagnostic: E0201 (Flow direction mismatch).";
        attribute ac_03_incompatible_item_payload_subtyping : String = "Given: Ports Port_Src : out Item_Super, Port_Tgt : in Item_Super, and payload Item_Unrelated (Item_Unrelated} Item_Super}). When: Type checking validates payload covariance. Then: Compiler rejects the flow declaration with diagnostic E0202. Diagnostic: E0202 (Payload type incompatibility).";
        attribute ac_04_rate_interval_and_capacity_bounds : String = "Given: Flow on Conn_0 (Capacity = 1000 B/s) with inverted rate [100.0, 10.0] or throughput > 1000 B/s. When: Rate multiplicity and bandwidth checks execute. Then: Compiler isolates the invalid flow with diagnostic E0202. Diagnostic: E0202 (Malformed rate or capacity overflow).";
        doc /* \begin{aligned} \text{ItemFlow} &\le \text{Step} \le \text{Feature} \\ \text{ItemFlow} &\le \text{OccurrenceUsage} \le \text{OccurrenceDefinition} \\ \Phi_{\text{dir}}(F) &\iff \text{dir}(p_{\text{src}}) \in \{ \text{Out}, \text{InOut} \} \;\wedge\; \text{dir}(p_{\text{tgt}}) \in \{ \text{In}, \text{InOut} \} \\ \Phi_{\text{payload}}(F) &\iff I_{\text{payload}} \le \text{payload\_type}(p_{\text{src}}) \;\wedge\; I_{\text{payload}} \le \text{payload\_type}(p_{\text{tgt}}) \\ \Phi_{\text{rate}}(F) &\iff 0 \le r_{\min} \le r_{\max}, \quad R_{\text{flow}} = [r_{\min}, r_{\max}] \\ \Phi_{\text{cap}}(F, C) &\iff r_{\max} \cdot \text{size}(I_{\text{payload}}) \le \text{Capacity}(C) \\ \forall n \in \mathcal{E}_{\text{flow}}, \quad \Delta_n &= \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0076_KerML_metamodel_lowering_specialization_and_streaming_invariants;
        require Invariant_REQ_0076_KerML_metamodel_lowering_specialization_and_streaming_invariants;
        verify by AC_01_Canonical_ItemFlow_Lowering;
        verify by AC_02_Conflicting_Port_Direction_Rejection;
        verify by AC_03_Incompatible_Item_Payload_Subtyping;
        verify by AC_04_Rate_Interval_and_Capacity_Bounds;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: 95e986c4-135e-5e26-83fd-814f63b42e72 | Complexity: Class P (O(V + E) verification) | Standard: IEEE 29148-2018 / RFC 2119 / OMG SysML v2 | Diagnostics: E0100, E0199, E0200, E0203 */
    requirement def REQ_0077_SysML_v2_AllocationDefinition {
        id = "REQ-0077";
        text = "The SysML v2 lowering engine SHALL formalize allocation definitions (allocation def) and usages (allocate... to..., AllocationUsage) as a typed bipartite relation from behavioral source elements to structural target elements. AllocationDefinition SHALL specialize KerML ConnectionDefinition, and";
        attribute uuidv5 : String = "95e986c4-135e-5e26-83fd-814f63b42e72";
        attribute complexity_class : String = "Class P (O(V + E) verification)";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119 / OMG SysML v2";
        attribute diagnostic_codes : String = "E0100, E0199, E0200, E0203";
        attribute ac_01_canonical_allocation_lowering_and_port_realization : String = "Given: Action Action_A with Port_Out : out Classifier_Alpha allocated to Part_Alpha with identical Port_Out. When: Lowering allocate Action_A to Part_Alpha. Then: Lowered as valid bipartite edge (s, t) in arena storage with mapped ports; zero diagnostics. Diagnostic: E0200 on failure.";
        attribute ac_02_bipartite_inversion_and_reflexive_rejection : String = "Given: Inverted allocate Part_Alpha to Action_A or reflexive allocate Action_A to Action_A. When: Bipartite domain and reflexivity validation executes. Then: Compile-time rejection binding sentinel _{alloc} with diagnostic E0200. Diagnostic: E0200 on violation.";
        attribute ac_03_port_mismatch_and_polarity_conflict : String = "Given: Action_A with Port_In : in Classifier_Alpha allocated to Part_Alpha with inverted or missing Port_In. When: Port allocation consistency is validated. Then: Rejection of the allocation usage with diagnostic E0200. Diagnostic: E0200 on violation.";
        attribute ac_04_circular_allocation_dependency_detection : String = "Given: Cyclic allocation chain Element_1 Element_2 Element_3 Element_1. When: Topological acyclicity analysis executes over the allocation graph. Then: Cycle detected via Tarjan SCC, emitting diagnostic E0203. Diagnostic: E0203 on violation.";
        doc /* \begin{aligned} \text{AllocationDefinition} \le \text{ConnectionDefinition} &\le \text{Classifier}, \quad \text{AllocationUsage} \le \text{ConnectionUsage} \le \text{Feature} \\ V_S \cap V_T = \emptyset, \quad \forall (s,t) &\in E_{\text{alloc}} : s \in V_S \wedge t \in V_T \wedge s \ne t \\ \forall p_s \in \text{Ports}(s), \, \exists ! p_t &\in \text{Ports}(t) : \text{dir}(p_t) = \text{dir}(p_s) \wedge \tau(p_s) \le \tau(p_t) \\ \forall v \in V_S \cup V_T, \, (v,v) \notin E_{\text{alloc}}^+, &\quad \forall n \in \mathcal{D}_{\text{alloc}} \cup \mathcal{U}_{\text{alloc}} : \Delta_n = \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0077_Metamodel_subtyping_taxonomy_and_bipartite_allocation_mapping_invariants;
        require Invariant_REQ_0077_Metamodel_subtyping_taxonomy_and_bipartite_allocation_mapping_invariants;
        verify by AC_01_Canonical_Allocation_Lowering_and_Port_Realization;
        verify by AC_02_Bipartite_Inversion_and_Reflexive_Rejection;
        verify by AC_03_Port_Mismatch_and_Polarity_Conflict;
        verify by AC_04_Circular_Allocation_Dependency_Detection;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: 1b74dcfc-dc78-551c-a61e-55c313f4670a | Complexity: Class P (O(V + E) DAG topological acyclicity) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.7 | Diagnostics: E0100, E0199, E0200, E0201, E0202, E0203 */
    requirement def REQ_0078_SysML_v2_ActionDefinition {
        id = "REQ-0078";
        text = "The compiler SHALL lower SysML v2 action definitions (action def, ActionDefinition) as process classifiers specializing KerML Behavior and OccurrenceDefinition, and action usages (action, ActionUsage) as discrete steps specializing KerML Step and OccurrenceUsage.";
        attribute uuidv5 : String = "1b74dcfc-dc78-551c-a61e-55c313f4670a";
        attribute complexity_class : String = "Class P (O(V + E) DAG topological acyclicity)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.7";
        attribute diagnostic_codes : String = "E0100, E0199, E0200, E0201, E0202, E0203";
        attribute ac_01_canonical_action_definition_and_step_lowering : String = "Given: Valid action def ActionDef_Alpha with steps action Step_Alpha : ActionDef_Beta and action Step_Beta : ActionDef_Gamma. When: Lowering constructs the action workflow AST. Then: Nodes are lowered into arena storage with dual KerML specializations (Behavior, Step) and zero diagnostics. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_conflicting_parameter_directionality_rejection : String = "Given: Action workflow binding input pin in param_in : Real as data source or output pin out param_out : Real as target. When: Data flow parameter polarity validation executes. Then: Compiler rejects inverted bindings with sentinel _{action} and diagnostic E0201. Diagnostic: E0201 (Parameter direction mismatch).";
        attribute ac_03_data_flow_payload_subtyping_and_multiplicity_overflow : String = "Given: Data flow binding output of type Classifier_Alpha to incompatible input Classifier_Beta, or exceeding target multiplicity. When: Type and multiplicity conformance checking executes. Then: Compiler rejects incompatible edge and emits diagnostic E0202. Diagnostic: E0202 (Payload type mismatch or multiplicity overflow).";
        attribute ac_04_control_node_degree_topology_violation : String = "Given: Action definition with malformed control node (e.g., fork with in-degree 2 or join with out-degree 2). When: Control node degree validation executes. Then: Compiler rejects malformed topology with source spans and diagnostic E0200. Diagnostic: E0200 (Malformed control topology).";
        doc /* \begin{aligned} \text{ActionDef} &\le \text{Behavior} \le \text{OccurrenceDef}, \quad \text{ActionUsage} \le \text{Step} \le \text{OccurrenceUsage} \\ \forall (u, p_1, v, p_2) \in E_{\text{data}}, \quad &\delta(p_1) \in \{ \text{Out}, \text{InOut} \} \wedge \delta(p_2) \in \{ \text{In}, \text{InOut} \} \wedge \tau(p_1) \le \tau(p_2) \\ \text{Ready}(s) \iff &(\forall (u, s) \in E_{\text{ctrl}}, \, \text{Token}(u, s) = 1) \wedge (\forall (u, p_1, s, p_2) \in E_{\text{data}}, \, \text{Bound}(p_2) = 1) \\ \forall v \in V_{\text{steps}}, \quad &(v, v) \notin E_{\text{dep}}^+, \quad \forall n \in \mathcal{D}_{\text{action}} \cup \mathcal{U}_{\text{action}}, \quad \Delta_n = \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0078_Metamodel_specialization_data_flow_step_readiness_and_dependency_acyclicity_invariants;
        require Invariant_REQ_0078_Metamodel_specialization_data_flow_step_readiness_and_dependency_acyclicity_invariants;
        verify by AC_01_Canonical_Action_Definition_and_Step_Lowering;
        verify by AC_02_Conflicting_Parameter_Directionality_Rejection;
        verify by AC_03_Data_Flow_Payload_Subtyping_and_Multiplicity_Overflow;
        verify by AC_04_Control_Node_Degree_Topology_Violation;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: 0acb8a8b-8dc8-50a8-8857-b16e8246b4d7 | Complexity: Class P (O(1) lookup, O(V + E) lowering) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.7 | Diagnostics: E0100, E0199, E0200, E0203 */
    requirement def REQ_0079_SysML_v2_StateDefinition_StateUsage {
        id = "REQ-0079";
        text = "The statechart lowering engine SHALL formalize SysML v2 state machine definitions (state def, StateDefinition) and state usages (state, StateUsage) as behavioral classifiers specializing KerML ActionDefinition and ActionUsage.";
        attribute uuidv5 : String = "0acb8a8b-8dc8-50a8-8857-b16e8246b4d7";
        attribute complexity_class : String = "Class P (O(1) lookup, O(V + E) lowering)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.7";
        attribute diagnostic_codes : String = "E0100, E0199, E0200, E0203";
        attribute ac_01_canonical_hierarchical_state_lowering : String = "Given: Valid state def StateDef_Alpha declaring composite State_A with sub-states State_1 and State_2. When: State machine lowering and hierarchy construction execute. Then: States lower into arena storage with active exclusivity (|active(State_A})| 1) and zero diagnostics. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_orthogonal_parallel_state_and_cartesian_space_conformance : String = "Given: Parallel composite ParallelState_Omega declaring k = 2 orthogonal regions Region_1 and Region_2. When: Region partitioning is evaluated. Then: Cartesian space States(R_1) States(R_2) is established with |active(R_i)| = 1. Diagnostic: E0200 on region partitioning failure.";
        attribute ac_03_degenerate_parallel_state_and_cross_region_transition_rejection : String = "Given: A parallel state with k 2 regions, or direct transition crossing orthogonal regions without fork/join. When: Semantic validation checks region cardinality and transition boundaries. Then: The compiler rejects the model and emits diagnostic E0200. Diagnostic: E0200 on degenerate parallel regions or unsynchronized transitions.";
        attribute ac_04_circular_state_containment_cycle_detection : String = "Given: Cyclic containment chain State_1 State_2 State_3 State_1. When: Topological acyclicity analysis traverses the containment graph. Then: The compiler isolates the cycle, halts containment lowering, and emits diagnostic E0203. Diagnostic: E0203 on circular state containment cycle.";
        doc /* \begin{aligned} \text{StateDef} \le \text{ActionDefinition} &\le \text{Behavior}, \quad \text{StateUsage} \le \text{ActionUsage} \le \text{Feature} \\ s_1 \sqsubset^+ s_2 &\implies s_2 \not\sqsubset^+ s_1 \\ \text{is\_parallel}(s) = \text{false} &\implies |\text{active}(\mathcal{C}_s)| \le 1 \\ \text{is\_parallel}(s) = \text{true} &\implies |\mathcal{R}_s| = k \ge 2 \;\wedge\; \mathbf{s} \in \prod_{i=1}^k \text{States}(R_i) \\ (s_{\text{src}} \in R_i \wedge s_{\text{tgt}} \in R_j \wedge i \ne j) &\implies \text{false} \\ \forall n \in \mathcal{S}_{\text{all}}, \quad \Delta_n &= \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0079_Metamodel_specialization_hierarchy_containment_acyclicity_and_concurrency_invariants;
        require Invariant_REQ_0079_Metamodel_specialization_hierarchy_containment_acyclicity_and_concurrency_invariants;
        verify by AC_01_Canonical_Hierarchical_State_Lowering;
        verify by AC_02_Orthogonal_Parallel_State_and_Cartesian_Space_Conformance;
        verify by AC_03_Degenerate_Parallel_State_and_Cross_Region_Transition_Rejection;
        verify by AC_04_Circular_State_Containment_Cycle_Detection;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: efb9e7ec-bdd3-53ae-88b0-7d28be789ccb | Complexity: Class P (O(1) precedence, O(h) LCA ladder) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.7 | Diagnostics: E0199, E0200, E0201, E0202, E0203 */
    requirement def REQ_0080_SysML_v2_TransitionUsage_with_Triggers_Guards {
        id = "REQ-0080";
        text = "The compiler SHALL lower SysML v2 transition usages (TransitionUsage) as directed behavioral edges between state configurations, specializing KerML ActionUsage and Step.";
        attribute uuidv5 : String = "efb9e7ec-bdd3-53ae-88b0-7d28be789ccb";
        attribute complexity_class : String = "Class P (O(1) precedence, O(h) LCA ladder)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.7";
        attribute diagnostic_codes : String = "E0199, E0200, E0201, E0202, E0203";
        attribute ac_01_canonical_lowering_3_phase_execution : String = "Given: StateDef_Alpha declaring State_A, State_B, and transition Transition_1 first State_A accept Event_0 if Guard_0 do Action_Effect then State_B. When: Transition lowering executes. Then: Transition_1 lowers into arena storage with an ordered 3-phase ladder (exit State_A, run Action_Effect, enter State_B). Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_inner_first_conflict_resolution_precedence : String = "Given: Composite State_Composite with substate State_Sub_1, parent transition t_{parent} to State_B on Event_0, and child transition t_{child} from State_Sub_1 to State_A on Event_0. When: Event_0 is dispatched in active configuration { State_Composite}, State_Sub_1} }. Then: Engine selects t_{child} over t_{parent} because Depth(State_Sub_1}) > Depth(State_Composite}). Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_orthogonal_fork_join_sibling_guard_rejection : String = "Given: Transitions with non-orthogonal endpoints, or sibling transitions sharing Event_0 with overlapping guards. When: Orthogonality and guard disjointness validations execute. Then: Compiler rejects malformed definitions with sentinel _{transition} and emits E0200. Diagnostic: E0200 on non-orthogonal endpoints or overlapping sibling guards.";
        attribute ac_04_trigger_typing_guard_typing_instant_cycle_rejection : String = "Given: Transitions with non-event trigger accept Classifier_Alpha, non-boolean guard if param_x + 1.0, or instant cycle State_1 -> State_2 -> State_1. When: Semantic type checking and cycle detection execute. Then: Compiler emits E0201 for trigger mismatch, E0202 for non-boolean guard, and E0203 for instant cycle. Diagnostic: E0201, E0202, or E0203 on corresponding violation.";
        doc /* \begin{aligned} \text{TransitionUsage} \le \text{ActionUsage} &\le \text{Step} \le \text{OccurrenceUsage} \\ \text{Enabled}(t, C, e, \sigma) &\iff (S_{\text{src}}(t) \subseteq C) \wedge (e_{\text{trigger}}(t) \approx e) \wedge (g_{\text{guard}}(t)(\sigma) = \text{true}) \\ \text{Conflicting}(t_1, t_2) &\iff \text{ExitSet}(t_1) \cap \text{ExitSet}(t_2) \ne \emptyset \\ \text{Conflicting}(t_1, t_2) \wedge \text{Enabled}(t_1) \wedge \text{Enabled}(t_2) &\implies (\text{Priority}(t_1) > \text{Priority}(t_2) \iff \text{Depth}(t_1.s_{\text{src}}) > \text{Depth}(t_2.s_{\text{src}})) \\ \forall s \in \mathcal{S}_{\text{states}}, \quad &(s, s) \notin \mathcal{T}_{\epsilon}^+, \quad \forall t \in \mathcal{T}_{\text{transitions}}, \quad \Delta_t = \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(t)) \end{aligned} */
        constraint def Invariant_REQ_0080_Metamodel_specialization_enabledness_conflict_precedence_and_cycle_invariants;
        require Invariant_REQ_0080_Metamodel_specialization_enabledness_conflict_precedence_and_cycle_invariants;
        verify by AC_01_Canonical_Lowering_3_Phase_Execution;
        verify by AC_02_Inner_First_Conflict_Resolution_Precedence;
        verify by AC_03_Orthogonal_Fork_Join_Sibling_Guard_Rejection;
        verify by AC_04_Trigger_Typing_Guard_Typing_Instant_Cycle_Rejection;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: eb2be5ec-7529-5ce0-80a6-020baaaa2ba7 | Complexity: Class P (O(1) dispatch, O(V_S + V_A) lowering) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.7 / KerML §7.4 | Diagnostics: E0100, E0199, E0200, E0201 */
    requirement def REQ_0081_SysML_v2_EntryAction_DoAction {
        id = "REQ-0081";
        text = "The state machine engine SHALL formalize SysML v2 state lifecycle hooks (entry, do, exit) as temporal occurrences bound to state activation, residence, and deactivation intervals. StateDefinition and StateUsage specialize KerML Behavior; EntryActionUsage, DoActionUsage, and ExitActionUsage";
        attribute uuidv5 : String = "eb2be5ec-7529-5ce0-80a6-020baaaa2ba7";
        attribute complexity_class : String = "Class P (O(1) dispatch, O(V_S + V_A) lowering)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.7 / KerML §7.4";
        attribute diagnostic_codes : String = "E0100, E0199, E0200, E0201";
        attribute ac_01_canonical_state_lifecycle_lowering_multiplicity : String = "Given: Valid State_A declaring entry Action_Entry, do Action_Do, and exit Action_Exit. When: Lowering constructs lifecycle AST nodes. Then: All three hooks lower into arena storage with valid handles and zero diagnostics. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_duplicate_lifecycle_hook_rejection : String = "Given: State_A declaring duplicate un-sequenced entry action hooks. When: Multiplicity validation executes. Then: Lowering rejects duplicate hook with sentinel _{action} and emits E0200. Diagnostic: E0200 on duplicate lifecycle hook.";
        attribute ac_03_non_behavioral_classifier_typing_rejection : String = "Given: Data type DataType_Value and State_A declaring entry : DataType_Value. When: Classifier conformance validation evaluates hook typing. Then: Lowering rejects binding with sentinel _{action} and emits E0201. Diagnostic: E0201 on invalid action classifier typing.";
        attribute ac_04_atomic_run_to_completion_and_do_preemption : String = "Given: Active State_A executing Action_Do after atomic Action_Entry ( t = 0). When: Outgoing transition T : State_A} State_B} fires at t_{fire}. Then: Action_Do aborts immediately (t_{end} = t_{fire}), and Action_Exit executes atomically before transition effects. Diagnostic: Diagnostic sink remains empty.";
        doc /* \begin{aligned} \text{HookKind} &\in \{ \text{EntryHook}, \text{DoHook}, \text{ExitHook} \} \\ \forall S \in \mathcal{S}_{\text{state}}, \quad &|\text{entry}(S)| \le 1 \;\wedge\; |\text{do}(S)| \le 1 \;\wedge\; |\text{exit}(S)| \le 1 \\ \forall A \in \mathcal{A}_{\text{hook}}, \quad &\tau(A) \le \text{Behavior} \;\wedge\; t_{\text{start}}(A) \le t_{\text{end}}(A) \\ t_{\text{entry}}(S) \le t_{\text{start}}(A_{\text{entry}}) &\le t_{\text{end}}(A_{\text{entry}}) \le t_{\text{start}}(A_{\text{do}}) \le t_{\text{end}}(A_{\text{do}}) \le t_{\text{start}}(A_{\text{exit}}) \le t_{\text{end}}(A_{\text{exit}}) \le t_{\text{exit}}(S) \\ \text{Preemptible}(A) &= (\text{kind}_A = \text{DoHook}) \\ \text{fire}(T, t_{\text{fire}}) \wedge \sigma(A_{\text{do}}) = \text{Executing} &\implies \sigma(A_{\text{do}}) \gets \text{Aborted} \;\wedge\; t_{\text{end}}(A_{\text{do}}) = t_{\text{fire}} \;\wedge\; t_{\text{start}}(A_{\text{exit}}) \ge t_{\text{fire}} \\ \forall n \in \mathcal{S}_{\text{state}} \cup \mathcal{A}_{\text{hook}}, \quad &\Delta_n = \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0081_State_lifecycle_action_classification_multiplicity_bounding_and_temporal_ordering;
        require Invariant_REQ_0081_State_lifecycle_action_classification_multiplicity_bounding_and_temporal_ordering;
        verify by AC_01_Canonical_State_Lifecycle_Lowering_Multiplicity;
        verify by AC_02_Duplicate_Lifecycle_Hook_Rejection;
        verify by AC_03_Non_Behavioral_Classifier_Typing_Rejection;
        verify by AC_04_Atomic_Run_to_Completion_and_Do_Preemption;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: 463a9478-f46e-5be3-8fae-2c9c3a6e34bd | Complexity: Class P (O(V + E) DAG topological evaluation) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.12 | Diagnostics: E0100, E0199, E0200, E0201, E0203 */
    requirement def REQ_0082_SysML_v2_CalculationDefinition {
        id = "REQ-0082";
        text = "The lowering engine SHALL formalize SysML v2 calculation definitions (calc def, CalculationDefinition) as functional classifiers specializing KerML Function, and calculation usages (calc, CalculationUsage) as steps specializing KerML Step.";
        attribute uuidv5 : String = "463a9478-f46e-5be3-8fae-2c9c3a6e34bd";
        attribute complexity_class : String = "Class P (O(V + E) DAG topological evaluation)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.12";
        attribute diagnostic_codes : String = "E0100, E0199, E0200, E0201, E0203";
        attribute ac_01_canonical_calculation_lowering : String = "Given: Valid calc def CalcDef_Alpha(in Param_X : Real) : Real and usage calc Calc_1 : CalcDef_Alpha(Param_X = 1.0). When: Calculation lowering constructs the functional AST. Then: Nodes lower into arena storage specializing KerML Function and Step with zero diagnostics. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_mutating_parameter_directionality_rejection : String = "Given: calc def CalcDef_Impure(inout Param_M : Real) : Real. When: Parameter directionality validation evaluates declared parameters. Then: Compiler rejects the declaration with sentinel _{calc} and emits diagnostic E0201. Diagnostic: E0201 (Parameter directionality violation).";
        attribute ac_03_covariant_argument_subtyping_and_classifier_conformance : String = "Given: Usage calc Calc_Err : CalcDef_Alpha(Param_X = \"str\") or typed by structural PartDef_Alpha. When: Feature typing and argument subtyping execute. Then: Compiler rejects invalid typing with precise source spans and emits diagnostic E0200. Diagnostic: E0200 (Argument type mismatch or non-functional classifier).";
        attribute ac_04_static_circular_calculation_dependency_detection : String = "Given: Calculation usages inducing static cycle Calc_1 -> Calc_2 -> Calc_1 in dependency graph G_{calc}. When: Topological acyclicity analysis executes via Tarjan's SCC algorithm. Then: Compiler isolates the cycle, halts recursive evaluation, and emits diagnostic E0203. Diagnostic: E0203 (Circular calculation dependency).";
        doc /* \begin{aligned} \text{CalcDef} \le \text{Function} &\le \text{Behavior}, \quad \text{CalcUsage} \le \text{Step} \wedge \tau(u) \le \text{Function} \\ \forall \mathbf{x} \in \prod_{i=1}^m \mathcal{D}(\tau(p_i)), \quad &\Delta \text{State} \equiv \emptyset \;\wedge\; \text{eval}(D, \mathbf{x}) = f_D(\mathbf{x}) \\ \forall p \in \mathcal{P}_D^{\text{in}}, \quad \text{dir}(p) = \text{In} &\;\wedge\; \text{dir}(p_D^{\text{ret}}) \in \{ \text{Return}, \text{Out} \} \;\wedge\; \text{dir}(p) \ne \text{InOut} \\ \forall (p_i, a_i) \in \mathcal{B}_u, \quad &\tau(a_i) \le \tau(p_i) \;\wedge\; \forall v \in V_{\text{calc}}, \, (v, v) \notin E_{\text{calc}}^+ \\ \forall n \in \mathcal{C}_{\text{calc}} \cup \mathcal{U}_{\text{calc}}, \quad &\Delta_n = \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0082_Metamodel_specialization_referential_transparency_parameter_directionality_and_dependency_acyclicity;
        require Invariant_REQ_0082_Metamodel_specialization_referential_transparency_parameter_directionality_and_dependency_acyclicity;
        verify by AC_01_Canonical_Calculation_Lowering;
        verify by AC_02_Mutating_Parameter_Directionality_Rejection;
        verify by AC_03_Covariant_Argument_Subtyping_and_Classifier_Conformance;
        verify by AC_04_Static_Circular_Calculation_Dependency_Detection;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: 1ee09e4b-5e48-5858-8ef5-4d28e0e9e535 | Complexity: Class P (O(1) stack ops / evaluation, O(N + M + E) AST) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.7 | Diagnostics: E0100, E0199, E0200, E0201, E0203 */
    requirement def REQ_0083_SysML_v2_ConstraintDefinition {
        id = "REQ-0083";
        text = "The metamodel lowering engine SHALL formalize SysML v2 constraint def (ConstraintDefinition) and constraint (ConstraintUsage) as Boolean predicate functions specializing KerML Predicate and Function. Every ConstraintDefinition SHALL define a pure invariant proposition over formal parameters P_{CD}";
        attribute uuidv5 : String = "1ee09e4b-5e48-5858-8ef5-4d28e0e9e535";
        attribute complexity_class : String = "Class P (O(1) stack ops / evaluation, O(N + M + E) AST)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.7";
        attribute diagnostic_codes : String = "E0100, E0199, E0200, E0201, E0203";
        attribute ac_01_canonical_lowering_and_binding : String = "Given: ConstraintDef_Alpha (in param_x : Real; in param_y : Real; param_x <= param_y) and usage ConstraintUsage_1 in Classifier_Alpha. When: Lowering constraint definition, binding arguments, and verifying Boolean codomain. Then: Nodes lower into arena storage with bound arguments and empty diagnostic sink. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_non_boolean_predicate_rejection : String = "Given: Definition ConstraintDef_Alpha with non-Boolean body param_x + param_y : Real. When: Validating Boolean codomain invariant _{codomain}. Then: Lowering rejects definition, assigns sentinel _{type}, and emits diagnostic E0200. Diagnostic: E0200 on non-Boolean body.";
        attribute ac_03_parameter_directionality_and_arity_rejection : String = "Given: Definition declaring out param_z : Real, or usage with arity mismatch (m k). When: Validating parameter directionality and signature arity conformance. Then: Directionality violation emits E0201; arity mismatch emits E0200. Diagnostic: E0201 on invalid parameter direction; E0200 on arity mismatch.";
        attribute ac_04_circular_dependency_detection_and_recovery : String = "Given: Mutually recursive definitions ConstraintDef_Alpha and ConstraintDef_Beta in call graph G_{c_dep}}. When: Dependency cycle analysis and sync-token error recovery execute. Then: Cycle detection emits E0203; compiler emits terminal E0199 at limit C_{max_errors}}. Diagnostic: E0203 on cyclic dependency; E0199 on diagnostic saturation.";
        doc /* \begin{aligned} \text{ConstraintDefinition} &\le \text{Predicate} \le \text{Function} \longrightarrow \mathbb{B} \\ \Phi_{\text{codomain}}(CD) &\iff \tau(e_{CD}) \equiv \text{Boolean} \\ \forall p \in \mathbf{P}_{CD}, &\quad \text{dir}(p) = \text{In} \\ \Phi_{\text{binding}}(CU, CD) &\iff (|\mathbf{A}_{CU}| = |\mathbf{P}_{CD}|) \;\wedge\; \bigwedge_{i=1}^k (\tau(a_i) \le \tau(p_i)) \\ \forall \sigma \in \Sigma_{\text{valid}}, &\quad ⟦ CU ⟧(\sigma) \equiv ⟦ e_{\tau(CU)} ⟧\left( [p_i \mapsto \sigma(a_i)]_{i=1}^k \right) \equiv \text{true} \\ \forall CD \in \mathcal{CD}, &\quad (CD, CD) \notin \mathcal{R}_{\text{c\_dep}}^+ \\ \forall n \in \mathcal{CD} \cup \mathcal{CU}, &\quad \Delta_n = \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0083_FormalInvariant;
        require Invariant_REQ_0083_FormalInvariant;
        verify by AC_01_Canonical_Lowering_and_Binding;
        verify by AC_02_Non_Boolean_Predicate_Rejection;
        verify by AC_03_Parameter_Directionality_and_Arity_Rejection;
        verify by AC_04_Circular_Dependency_Detection_and_Recovery;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: 147a25c4-24cc-5d6e-aede-ee0a83287b0b | Complexity: Class P (O(1) monitor step, O(V + E) AST lowering) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.4 | Diagnostics: E0100, E0199, E0200, E0201, E0203 */
    requirement def REQ_0084_SysML_v2_AssertConstraintUsage_Lowering {
        id = "REQ-0084";
        text = "The KerML/SysML v2 lowering engine SHALL lower assert constraint declarations (assert constraint, AssertConstraintUsage) into invariant assertions specializing KerML ConstraintUsage and Invariant. Every AssertConstraintUsage binds Boolean predicate \Phi to a classifier context (part def, item def,";
        attribute uuidv5 : String = "147a25c4-24cc-5d6e-aede-ee0a83287b0b";
        attribute complexity_class : String = "Class P (O(1) monitor step, O(V + E) AST lowering)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.4";
        attribute diagnostic_codes : String = "E0100, E0199, E0200, E0201, E0203";
        attribute ac_01_canonical_assert_constraint_lowering : String = "Given: Classifier Classifier_Alpha with definition ConstraintDef_Alpha (in param_x : Real; in param_y : Real; param_x <= param_y;) and usage AssertConstraint_1. When: Lowering and parameter binding execute. Then: AssertConstraint_1 lowers into arena storage with is_asserted true and empty diagnostic sink. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_non_boolean_codomain_impure_expression_rejection : String = "Given: Assert constraint evaluating to Real (attr_a + attr_b) or mutating state (attr_a = param_x). When: Type checking and purity validation execute. Then: Lowering rejects declaration, assigns sentinel _{assert}, and emits diagnostic E0200. Diagnostic: E0200 (Non-Boolean codomain or state mutation).";
        attribute ac_03_parameter_directionality_arity_violation_rejection : String = "Given: Assert constraint referencing definition with out parameter, or binding 1 argument for a 2-parameter definition. When: Parameter directionality and binding verification execute. Then: Lowering rejects non-in parameters with E0201 and arity mismatches with E0200. Diagnostic: E0201 (Invalid parameter direction); E0200 (Parameter arity mismatch).";
        attribute ac_04_circular_constraint_dependency_detection : String = "Given: Mutually recursive definitions ConstraintDef_Alpha and ConstraintDef_Beta. When: Cycle detection evaluates call graph R_{c_dep}}. Then: Compiler detects dependency cycle, halts topological sorting, and emits diagnostic E0203. Diagnostic: E0203 (Circular constraint dependency).";
        doc /* \begin{aligned} \text{AssertConstraintUsage} &\le \text{ConstraintUsage} \le \text{Predicate} \le \text{Function} \longrightarrow \mathbb{B} \\ \text{is\_asserted}(ACU) &\equiv \text{true} \;\wedge\; \tau(\Phi) \equiv \text{Boolean} \\ \forall p \in \mathbf{P}_{\tau(ACU)}, \quad \text{dir}(p) &= \text{In} \;\wedge\; \Delta\text{State}(\Phi) \equiv \emptyset \\ |\mathbf{A}_{ACU}| = |\mathbf{P}_{\tau(ACU)}| &\wedge \bigwedge_{i=1}^k (\tau(a_i) \le \tau(p_i)) \\ \forall CD \in \mathcal{CD}, \quad (CD, CD) &\notin \mathcal{R}_{\text{c\_dep}}^+ \\ \square \Phi \iff \forall \sigma \in \text{Traces}(\mathcal{M}), \; \forall t \ge 0 &: ⟦ \Phi ⟧(\sigma_t) \equiv \text{true} \\ \Phi_{\text{proof}}(ACU) \iff \text{UNSAT}&\left( \text{Reachable}(\sigma) \wedge \neg ⟦ \Phi ⟧(\sigma) \right) \\ \text{Alloc}(\mathcal{M}_{\text{monitor}}) &\equiv 0 \\ \forall n \in \mathcal{ACU}, \quad \Delta_n &= \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0084_KerML_metamodel_lowering_specialization_and_formal_invariants;
        require Invariant_REQ_0084_KerML_metamodel_lowering_specialization_and_formal_invariants;
        verify by AC_01_Canonical_Assert_Constraint_Lowering;
        verify by AC_02_Non_Boolean_Codomain_Impure_Expression_Rejection;
        verify by AC_03_Parameter_Directionality_Arity_Violation_Rejection;
        verify by AC_04_Circular_Constraint_Dependency_Detection;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: e0a7f813-ecd8-5478-aa4d-32325fea1deb | Complexity: Class P (O(1) ID lookup, O(V + E) DAG sort) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.11 | Diagnostics: E0100, E0199, E0200, E0201, E0203 */
    requirement def REQ_0085_SysML_v2_RequirementDefinition {
        id = "REQ-0085";
        text = "The compiler SHALL lower SysML v2 requirement definitions (requirement def, RequirementDefinition) as contractual classifiers specializing KerML ConstraintDefinition, and requirement usages (requirement, RequirementUsage) as contextual features specializing KerML ConstraintUsage.";
        attribute uuidv5 : String = "e0a7f813-ecd8-5478-aa4d-32325fea1deb";
        attribute complexity_class : String = "Class P (O(1) ID lookup, O(V + E) DAG sort)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.11";
        attribute diagnostic_codes : String = "E0100, E0199, E0200, E0201, E0203";
        attribute ac_01_canonical_requirement_lowering_and_contract_resolution : String = "Given: ReqDef_Alpha with subject Subject_A : Classifier_Alpha, assume Subject_A.attr_a >= 0.0, require Subject_A.attr_a <= 100.0. When: Lowering Req_1 : ReqDef_Alpha binding Subject_A = Classifier_Alpha. Then: Nodes lower into arena storage with resolved subject binding and empty diagnostic sink. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_global_duplicate_requirement_identifier_rejection : String = "Given: Two declarations with duplicate identifier req_id = \"REQ-0085\". When: Global identifier uniqueness validation executes. Then: Lowering rejects duplicate declaration with source spans and emits E0200. Diagnostic: E0200 (Duplicate requirement identifier).";
        attribute ac_03_subject_parameter_binding_mismatch_and_missing_binding : String = "Given: Usage Req_1 : ReqDef_Alpha with missing subject or binding Classifier_Beta (Classifier_Beta} Classifier_Alpha}). When: Subject parameter subtyping validation executes. Then: Lowering rejects invalid binding with sentinel _{req} and emits E0200. Diagnostic: E0200 (Subject parameter binding mismatch).";
        attribute ac_04_non_input_parameter_directionality_rejection : String = "Given: Requirement definition declaring parameter with direction out (out param_x : Real). When: Parameter signature directionality analysis executes. Then: Lowering rejects non-In directionality and emits E0201. Diagnostic: E0201 (Invalid requirement parameter direction).";
        doc /* \begin{aligned} \text{RequirementDef} &\le \text{ConstraintDef} \le \text{Predicate} \le \text{Function} \longrightarrow \mathbb{B} \\ \text{RequirementUsage} &\le \text{ConstraintUsage} \le \text{Feature} \\ \forall R_1 \ne R_2, \quad &\text{req\_id}(R_1) \ne \text{req\_id}(R_2) \\ \text{dir}(p_{\text{subject}}) \equiv \text{In} &\;\wedge\; \forall p \in \mathbf{P}_{RD}, \quad \text{dir}(p) \equiv \text{In} \\ \tau(e_{\text{assume}}) \equiv \text{Boolean} &\;\wedge\; \tau(e_{\text{require}}) \equiv \text{Boolean} \\ \tau(a_{\text{subject}}) \le \tau(p_{\text{subject}}) &\;\wedge\; \bigwedge_{i=1}^k (\tau(a_i) \le \tau(p_i)) \\ \forall \sigma \in \Sigma_{\text{valid}}, \quad ⟦ e_{\text{assume}} ⟧(\sigma) &\implies ⟦ e_{\text{require}} ⟧(\sigma) \\ \forall R \in \mathcal{RD} \cup \mathcal{RU}, \quad &(R, R) \notin \mathcal{R}_{\text{req\_dep}}^+ \\ \forall n \in \mathcal{RD} \cup \mathcal{RU}, \quad \Delta_n &= \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0085_Metamodel_specialization_contract_tuples_identifier_uniqueness_parameter_typing_and_dependency_acyclicity_invariants;
        require Invariant_REQ_0085_Metamodel_specialization_contract_tuples_identifier_uniqueness_parameter_typing_and_dependency_acyclicity_invariants;
        verify by AC_01_Canonical_Requirement_Lowering_and_Contract_Resolution;
        verify by AC_02_Global_Duplicate_Requirement_Identifier_Rejection;
        verify by AC_03_Subject_Parameter_Binding_Mismatch_and_Missing_Binding;
        verify by AC_04_Non_Input_Parameter_Directionality_Rejection;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: 8d961e04-37e9-56d4-bb4f-fdaa3335ab0c | Complexity: Class P (O(1) lookup, O( | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.11 | Diagnostics: E0100, E0199, E0200, E0203 */
    requirement def REQ_0086_SysML_v2_SatisfyRequirementUsage_Traceability {
        id = "REQ-0086";
        text = "The compiler SHALL lower SysML v2 satisfy requirement usages (SatisfyRequirementUsage, declared via satisfy requirement or satisfy... by...) as directed realization dependencies linking structural parts or actions to requirements, specializing KerML Feature, Step, and Relationship.";
        attribute uuidv5 : String = "8d961e04-37e9-56d4-bb4f-fdaa3335ab0c";
        attribute complexity_class : String = "Class P (O(1) lookup, O(";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.11";
        attribute diagnostic_codes : String = "E0100, E0199, E0200, E0203";
        attribute ac_01_canonical_explicit_and_contextual_satisfy_lowering : String = "Given: Req_1 : ReqDef_Alpha and Part_Alpha : Classifier_Alpha in Package_0. When: Lowering satisfy requirement Req_1 by Part_Alpha; or contextual part Part_Alpha { satisfy requirement Req_1; }. Then: Edge (Part_Alpha, Req_1) enters G_{trace} with populated adjacency maps and empty diagnostic sink. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_subject_classifier_subtyping_conformance_violation : String = "Given: Req_1 with subject Classifier_Alpha and Part_Beta : Classifier_Beta (Classifier_Beta} Classifier_Alpha}). When: Lowering evaluates subject subtyping. Then: Lowering rejects the binding, binds sentinel _{sat}, and emits diagnostic E0200. Diagnostic: E0200 (Subject type mismatch).";
        attribute ac_03_invalid_realization_domain_and_reflexive_satisfaction_prohibition : String = "Given: Malformed satisfy declaring self-satisfaction (satisfy Req_1 by Req_1;) or non-requirement target (satisfy Part_Beta by Part_Alpha;). When: Bipartite domain disjointness and irreflexivity validation execute. Then: Lowering halts edge insertion, binds _{sat}, and emits diagnostic E0200. Diagnostic: E0200 (Invalid satisfaction endpoint).";
        attribute ac_04_traceability_coverage_and_orphan_requirement_detection : String = "Given: Requirements Req_1 and Req_2 with only Req_1 satisfied by Part_Alpha under completeness enforcement. When: Traceability coverage analysis and orphan detection execute. Then: Coverage evaluates to Coverage(V_R) = 0.5, orphan Req_2 is flagged, and diagnostic E0200 is emitted. Diagnostic: E0200 (Orphan requirement violation).";
        doc /* \begin{aligned} \text{SatisfyUsage} &\le_{\text{sub}} \text{RequirementUsage} \le_{\text{sub}} \text{Feature} \\ \text{SatisfyUsage} &\le_{\text{sub}} \text{AssertConstraintUsage} \le_{\text{sub}} \text{Relationship} \\ V_E \cap V_R &= \emptyset \;\wedge\; E_{\text{sat}} \subseteq V_E \times V_R \\ \forall (e, r) \in E_{\text{sat}}, \quad & e \ne r \;\wedge\; (\tau(p_{\text{subject}}(r)) \ne \top \implies \tau(e) \le_{\text{sub}} \tau(p_{\text{subject}}(r))) \\ \text{Coverage}(V_R) &= \frac{|\{ r \in V_R \mid \exists e \in V_E, (e, r) \in E_{\text{sat}} \}|}{|V_R|} \\ \text{Orphans}(V_R) &= \{ r \in V_R \mid \text{BackwardSat}(r) = \emptyset \} \\ \forall v \in V_{\text{all}}, \quad & (v, v) \notin E_{\text{sat\_all}}^+ \\ \forall (e, r) \in E_{\text{sat}}, \quad & \Delta_{\text{sat}} = \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(e) \mathbin{\Vert} \text{"\_satisfies\_"} \mathbin{\Vert} \text{FQN}(r)) \end{aligned} */
        constraint def Invariant_REQ_0086_Metamodel_specialization_bipartite_domain_partition_subject_subtyping_coverage_metric_and_acyclicity_invariants;
        require Invariant_REQ_0086_Metamodel_specialization_bipartite_domain_partition_subject_subtyping_coverage_metric_and_acyclicity_invariants;
        verify by AC_01_Canonical_Explicit_and_Contextual_Satisfy_Lowering;
        verify by AC_02_Subject_Classifier_Subtyping_Conformance_Violation;
        verify by AC_03_Invalid_Realization_Domain_and_Reflexive_Satisfaction_Prohibition;
        verify by AC_04_Traceability_Coverage_and_Orphan_Requirement_Detection;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: 0f2583a9-96b7-5158-9e35-405d1bce68bb | Complexity: Class P (O(1) dispatch step, O(V + E) AST lowering) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.13 | Diagnostics: E0100, E0199, E0200, E0201, E0202, E0203 */
    requirement def REQ_0087_SysML_v2_VerificationCaseDefinition {
        id = "REQ-0087";
        text = "The compiler SHALL lower SysML v2 verification case definitions (verification def, VerificationCaseDefinition) as behavioral procedures specializing KerML Behavior and SysML v2 CaseDefinition, and verification usages (verification, VerificationCaseUsage) as discrete occurrences specializing KerML";
        attribute uuidv5 : String = "0f2583a9-96b7-5158-9e35-405d1bce68bb";
        attribute complexity_class : String = "Class P (O(1) dispatch step, O(V + E) AST lowering)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.13";
        attribute diagnostic_codes : String = "E0100, E0199, E0200, E0201, E0202, E0203";
        attribute ac_01_canonical_verification_case_lowering_verdict_resolution : String = "Given: VerificationDef_Alpha in Package_0 with subject Subject_A : Classifier_Alpha and output verdict : VerdictKind, instantiated as VerificationUsage_1 binding Subject_A = Classifier_Alpha. When: Lowering and type checking execute. Then: Nodes lower into arena storage with valid subject binding, specializing KerML Behavior, with empty diagnostic sink. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_subject_parameter_directionality_violation_rejection : String = "Given: VerificationDef_Alpha declaring formal subject parameter with out directionality (out Subject_A : Classifier_Alpha). When: Parameter directionality analysis executes. Then: Lowering rejects non-In/InOut directionality with sentinel _{verif} and emits diagnostic E0201. Diagnostic: E0201 (Invalid subject parameter direction).";
        attribute ac_03_subject_argument_subtyping_mismatch_missing_binding : String = "Given: Usage VerificationUsage_1 : VerificationDef_Alpha omitting subject argument or binding argument of type Classifier_Beta (Classifier_Beta} Classifier_Alpha}). When: Subject parameter conformance checking executes. Then: Lowering rejects invalid bindings with sentinel _{verif}, emitting E0202 on type mismatch or E0200 on omission. Diagnostic: E0202 on subtyping mismatch, E0200 on missing subject argument.";
        attribute ac_04_mandatory_tri_state_verdict_codomain_violation : String = "Given: VerificationDef_Alpha omitting verdict return parameter or specifying invalid return type Real. When: Signature verification and verdict type unification execute. Then: Lowering rejects malformed signature with sentinel _{verif} and emits diagnostic E0200. Diagnostic: E0200 (Malformed verification signature).";
        doc /* \begin{aligned} \text{VerificationDef}\le\text{CaseDefinition}&\le\text{Behavior}\le\text{Class} \\ \text{VerificationUsage}\le\text{CaseUsage}&\le\text{Step}\le\text{OccurrenceUsage}\le\text{Feature} \\ \mathcal{V}&=\{\text{Pass},\text{Fail},\text{Inconclusive}\} \\ \forall s\in\mathbf{P}_{\text{subj}}(VD),\quad\text{dir}(s)&\in\{\text{In},\text{InOut}\}\;\wedge\;\mathbf{P}_{\text{subj}}(VD)\ne\emptyset \\ \tau(p_{\text{verdict}})\equiv\text{VerdictKind}&\;\wedge\;\text{dir}(p_{\text{verdict}})\equiv\text{Out} \\ \forall VU\in\mathcal{VU},\quad|\mathbf{A}_{\text{subj}}|&=|\mathbf{P}_{\text{subj}}|\;\wedge\;\bigwedge_{i=1}^j(\tau(a_{s_i})\le\tau(s_i)) \\ \text{eval}:\mathcal{VD}\times\mathcal{S}_{\text{subject}}&\longrightarrow\mathcal{V} \\ \text{Verified}(R,s)\iff\forall(VU,R)\in\mathcal{R}_{\text{verify}},&\quad\text{eval}(\tau(VU),s)\equiv\text{Pass} \\ \forall VD\in\mathcal{VD},\quad(VD,VD)&\notin\mathcal{E}_{\text{v\_dep}}^+ \\ \forall n\in\mathcal{VD}\cup\mathcal{VU},\quad\Delta_n&=\text{UUIDv5}(\text{NS}_{\text{DEAP}},\text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0087_Metamodel_specialization_subject_typing_verdict_codomain_and_invocation_acyclicity_invariants;
        require Invariant_REQ_0087_Metamodel_specialization_subject_typing_verdict_codomain_and_invocation_acyclicity_invariants;
        verify by AC_01_Canonical_Verification_Case_Lowering_Verdict_Resolution;
        verify by AC_02_Subject_Parameter_Directionality_Violation_Rejection;
        verify by AC_03_Subject_Argument_Subtyping_Mismatch_Missing_Binding;
        verify by AC_04_Mandatory_Tri_State_Verdict_Codomain_Violation;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: 1a967535-f7bf-5a70-9007-3f9cd6fabcf8 | Complexity: Class P (O(V + E)) | Standard: IEEE 29148-2018 / RFC 2119 / SysML v2 §7.13 | Diagnostics: E0100, E0199, E0200, E0203 */
    requirement def REQ_0088_SysML_v2_ViewDefinition_ViewUsage {
        id = "REQ-0088";
        text = "The compiler SHALL lower SysML v2 view definitions (view def) and usages (view) into structural projection elements, and viewpoint definitions (viewpoint def) and usages (viewpoint) into requirement elements.";
        attribute uuidv5 : String = "1a967535-f7bf-5a70-9007-3f9cd6fabcf8";
        attribute complexity_class : String = "Class P (O(V + E))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119 / SysML v2 §7.13";
        attribute diagnostic_codes : String = "E0100, E0199, E0200, E0203";
        attribute ac_01_canonical_viewdefinition_and_viewpoint_lowering : String = "Given: Package_0 with viewpoint def ViewpointDef_Alpha { concern Concern_Alpha; } and view def ViewDef_Alpha { satisfy viewpoint ViewpointDef_Alpha; }. When: Lowering and viewpoint conformance checking execute. Then: Nodes lower into arena storage with SatisfyRequirementUsage and empty diagnostic sink. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_view_subgraph_model_projection_completeness : String = "Given: Model M_{model} (V=1000, E=2500) matching filter predicate _{filter}. When: Projection engine executes _V for ViewDef_Alpha. Then: M_{view} contains 100% of matching vertices, 0% of non-matching vertices, and preserves induced edges. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_invalid_specialization_and_non_viewpoint_target_rejection : String = "Given: Declarations view def ViewDef_Alpha :> DataType_Alpha or satisfy viewpoint Classifier_Alpha. When: Semantic type checking executes. Then: Lowering rejects invalid specializations and non-viewpoint targets with sentinel _{view} and diagnostic E0200. Diagnostic: E0200 (Invalid classifier specialization or unresolved viewpoint target).";
        attribute ac_04_circular_viewpoint_dependency_and_delimiter_recovery : String = "Given: Cyclic definitions ViewpointDef_Alpha :> ViewpointDef_Beta :> ViewpointDef_Alpha followed by a malformed statement. When: Cycle detection and delimiter recovery execute. Then: Lowering detects dependency cycle E0203, resynchronizes on delimiter (;), and continues parsing. Diagnostic: E0203 (Circular viewpoint dependency); E0199 on terminal error cutoff.";
        doc /* \begin{aligned} \text{ViewDef} \le \text{PartDef} \le \text{Structure} &\le \text{Class} \le \text{Classifier} \le \text{Type} \\ \text{ViewUsage} \le \text{PartUsage} &\le \text{ItemUsage} \le \text{Feature} \\ \text{ViewpointDef} \le \text{RequirementDef} &\le \text{ConstraintDef} \le \text{Predicate} \\ \text{ViewpointUsage} \le \text{RequirementUsage} &\le \text{ConstraintUsage} \le \text{Feature} \\ ⟦ \text{ViewDef} ⟧ \subseteq ⟦ \text{PartDef} ⟧ &\wedge ⟦ \text{ViewDef} ⟧ \cap (⟦ \text{DataType} ⟧ \cup ⟦ \text{ActionDef} ⟧) = \emptyset \\ \pi_V : (\mathcal{V}_{\text{model}}, \mathcal{E}_{\text{model}}) &\longrightarrow (\mathcal{V}_{\text{view}}, \mathcal{E}_{\text{view}}) \\ \forall n \in \mathcal{V}_{\text{model}}, \quad n \in \mathcal{V}_{\text{view}} &\iff \Phi_{\text{filter}}(n) \equiv \text{true} \\ \mathcal{E}_{\text{view}} = \{ (u, v) \in \mathcal{E}_{\text{model}} &\mid u, v \in \mathcal{V}_{\text{view}} \} \\ V \models VP \iff (\forall c \in \mathbf{C}_{VP}, \text{Satisfies}(V, c)) &\wedge (\mathcal{V}_{\text{view}} = \{ n \mid \Phi_{\text{filter}}(n) \equiv \text{true} \}) \\ \forall VPD \in \mathcal{VPD}, \quad (VPD, VPD) &\notin \mathcal{R}_{\text{vp\_dep}}^+ \\ \forall n \in \mathcal{VD} \cup \mathcal{VU} \cup \mathcal{VPD} \cup \mathcal{VPU}, \quad \Delta_n &= \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0088_Metamodel_specialization_induced_subgraph_projection_conformance_acyclicity_and_deterministic_UUIDv5_derivation_invariants;
        require Invariant_REQ_0088_Metamodel_specialization_induced_subgraph_projection_conformance_acyclicity_and_deterministic_UUIDv5_derivation_invariants;
        verify by AC_01_Canonical_ViewDefinition_and_Viewpoint_Lowering;
        verify by AC_02_View_Subgraph_Model_Projection_Completeness;
        verify by AC_03_Invalid_Specialization_and_Non_Viewpoint_Target_Rejection;
        verify by AC_04_Circular_Viewpoint_Dependency_and_Delimiter_Recovery;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: a1f41091-b6c1-5fe5-86e4-b2d2ea224c96 | Complexity: Class P (O(1) lookup, O(V + E) graph projection) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.16, §8.4 | Diagnostics: E0199, E0200, E0202, E0203 */
    requirement def REQ_0089_SysML_v2_Expose_Filtering {
        id = "REQ-0089";
        text = "The compiler SHALL lower SysML v2 expose statements in ViewDefinition and ViewUsage as declarative selection expressions, projecting conformant induced subgraphs and validating viewpoint obligations.";
        attribute uuidv5 : String = "a1f41091-b6c1-5fe5-86e4-b2d2ea224c96";
        attribute complexity_class : String = "Class P (O(1) lookup, O(V + E) graph projection)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.16, §8.4";
        attribute diagnostic_codes : String = "E0199, E0200, E0202, E0203";
        attribute ac_01_canonical_expose_path_resolution_and_induced_subgraph_projection : String = "Given: Package_0 with classifiers Classifier_Alpha and Classifier_Beta connected by relation Rel_A. When: Lowering view def ViewDef_Alpha { expose Package_0::Classifier_Alpha, Package_0::Classifier_Beta; }. Then: Classifiers resolve into V_{exp} and induced edge Rel_A is included in E_{exp}. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_wildcard_expansion_and_predicate_filtering_boundary : String = "Given: Package_0 with 10 parts typed by PartDef_Alpha and 5 ports, tagged @Tag_Alpha. When: Lowering view def ViewDef_Alpha { expose Package_0::*; expose filter (isKindOf(PartDef_Alpha) and @Tag_Alpha); }. Then: Shallow wildcard and filter admit exactly the tagged parts into V_{exp} without descending into sub-packages. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_unresolved_path_target_and_filter_codomain_type_mismatch : String = "Given: View declaring expose Package_0::UndefinedClassifier; and expose filter (param_x + 5.0); with param_x : Real. When: Scoped symbol resolution and predicate type analysis execute. Then: Lowering rejects undefined target with E0200, rejects non-boolean predicate with E0202, and assigns sentinel _{exp}. Diagnostic: E0200 on unresolved path; E0202 on filter type mismatch.";
        attribute ac_04_viewpoint_conformance_violation_and_circular_expose_recovery : String = "Given: View declaring satisfy viewpoint ViewpointDef_Alpha; omitting mandatory Classifier_Alpha, alongside cyclic views ViewDef_Alpha and ViewDef_Beta. When: Conformance checking and cycle detection execute. Then: Validator emits E0200 for omission, E0203 for cycle, recovers on ; and }, and emits terminal E0199 at C_{max_errors}}. Diagnostic: E0200 on viewpoint violation; E0203 on cycle; E0199 on error ceiling.";
        doc /* \begin{aligned} \text{Expand}(p,\Omega)&=\begin{cases} \{e\in\mathcal{V}_{\text{model}}\mid\text{FQN}(e)=p\}&\Omega=\text{Exact} \\ \{e\in\mathcal{V}_{\text{model}}\mid\text{Parent}(e)=\text{ResolvePath}(p)\}&\Omega=\text{ShallowWildcard} \\ \{e\in\mathcal{V}_{\text{model}}\mid\text{FQN}(e)=p\vee\text{DescendantOf}(e,p)\}&\Omega=\text{RecursiveWildcard} \end{cases} \\ V_{\text{exp}}(V)&=\bigcup_{E\in\mathbf{E}_V}\left(\bigcup_{p\in\mathbf{P}_E}\text{Expand}(p,\Omega_E)\cap\{e\in\mathcal{V}_{\text{model}}\mid\Phi_E(e)=\text{true}\}\right) \\ E_{\text{exp}}(V)&=\{(u,v)\in\mathcal{E}_{\text{model}}\mid u\in V_{\text{exp}}(V)\wedge v\in V_{\text{exp}}(V)\} \\ V\models VP&\iff(\mathbf{M}_{\text{vp}}\subseteq V_{\text{exp}}(V))\wedge(\forall u\in V_{\text{exp}}(V),\Phi_{\text{vp}}(u)=\text{true})\wedge(V_{\text{exp}}(V)\cap\mathbf{X}_{\text{vp}}=\emptyset) \\ \forall V\in\mathcal{VD},&(V,V)\notin\mathcal{R}_{\text{exp\_dep}}^+\;\wedge\;\forall n\in\mathcal{E}_{\text{exp}}\cup\{G_{\text{view}}(V)\},\Delta_n=\text{UUIDv5}(\text{NS}_{\text{DEAP}},\text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0089_Expose_path_expansion_aggregation_induced_edge_preservation_viewpoint_conformance_and_dependency_acyclicity;
        require Invariant_REQ_0089_Expose_path_expansion_aggregation_induced_edge_preservation_viewpoint_conformance_and_dependency_acyclicity;
        verify by AC_01_Canonical_Expose_Path_Resolution_and_Induced_Subgraph_Projection;
        verify by AC_02_Wildcard_Expansion_and_Predicate_Filtering_Boundary;
        verify by AC_03_Unresolved_Path_Target_and_Filter_Codomain_Type_Mismatch;
        verify by AC_04_Viewpoint_Conformance_Violation_and_Circular_Expose_Recovery;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: b6a46e5b-de7c-5040-94b7-590d8b517bda | Complexity: Class P (O(1) dictionary lookup, O(V) indexing) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.13 / KerML §7.4 | Diagnostics: E0199, E0200, E0203 */
    requirement def REQ_0090_SysML_v2_MetadataDefinition {
        id = "REQ-0090";
        text = "The compiler SHALL lower SysML v2 metadata definitions (metadata def, MetadataDefinition) as structural classifiers specializing KerML Structure and Class, and semantic annotation usages (@, MetadataUsage) as extrinsic metadata attachments linking attribute dictionaries to AST nodes without";
        attribute uuidv5 : String = "b6a46e5b-de7c-5040-94b7-590d8b517bda";
        attribute complexity_class : String = "Class P (O(1) dictionary lookup, O(V) indexing)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.13 / KerML §7.4";
        attribute diagnostic_codes : String = "E0199, E0200, E0203";
        attribute ac_01_canonical_metadata_lowering_and_annotation : String = "Given: Package Package_0 declaring metadata def MetadataDef_Alpha { attr attr_1 : Integer; } and @MetadataDef_Alpha(attr_1 = 42) part def Classifier_Alpha;. When: Lowering and symbol resolution execute. Then: MetadataDef_Alpha lowers as KerML Structure, attribute bindings populate dictionary , and diagnostic sink remains empty. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_target_metaclass_restriction_enforcement : String = "Given: Definition MetadataDef_Alpha restricting target to Feature (about Feature), and part def Classifier_Alpha; annotated with @MetadataDef_Alpha. When: Metaclass target conformance validation executes. Then: Lowering detects Classifier_Alpha is not a feature, rejects attachment, and emits E0200. Diagnostic: E0200 (Invalid annotated element target).";
        attribute ac_03_attribute_type_incompatibility_rejection : String = "Given: Usage @MetadataDef_Alpha(attr_1 = \"invalid\") part def Classifier_Alpha; where attr_1 expects Integer. When: Attribute argument type-checking executes. Then: Lowering detects type mismatch (String Integer), substitutes sentinel _{meta}, and emits E0200. Diagnostic: E0200 (Metadata attribute type mismatch).";
        attribute ac_04_delimiter_synchronization_and_error_recovery : String = "Given: Source buffer with malformed @UnresolvedDef(err;;) part def Classifier_Beta; preceding valid part def Classifier_Alpha;. When: Parsing and synchronization error recovery execute. Then: Parser resynchronizes on delimiters, records E0200, lowers Classifier_Alpha, and emits E0199 upon reaching C_{max_errors}}. Diagnostic: E0200 on syntax/resolution fault; E0199 on error threshold overflow.";
        doc /* \begin{aligned} \text{MetadataDef} &\le \text{Structure} \le \text{Class} \le \text{Classifier} \le \text{Type} \\ \text{MetadataUsage} &\le \text{ItemUsage} \le \text{Feature} \\ ⟦ e ⟧ &\equiv ⟦ e \mathbin{\Vert} @M ⟧ \\ \alpha : \mathcal{V}_{\text{AST}} \times \mathcal{S}_{\text{intern}} &\longrightarrow \mathcal{V}_{\text{val}} \cup \{ \bot_{\text{none}} \} \\ \forall MU \in \mathcal{MU}, \forall (a \mapsto v) \in \mathbf{V}_{MU}, \quad &\alpha(h_{\text{target}}(MU), a) = v \;\wedge\; \tau(v) \le \tau(a) \\ \tau_{\text{target}}(\tau(MU)) \ne \top_{\text{any}} &\implies \text{kind}(h_{\text{target}}(MU)) \le \tau_{\text{target}}(\tau(MU)) \\ (h_{\text{target}}(MU_1) = h_{\text{target}}(MU_2) \;\wedge\; \tau(MU_1) &= \tau(MU_2) \;\wedge\; \kappa_{\text{rep}} = \text{Unique}) \implies MU_1 = MU_2 \\ \forall n \in \mathcal{MD} \cup \mathcal{MU}, \quad \Delta_n &= \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0090_Metamodel_specialization_semantic_neutrality_dictionary_mapping_typing_and_UUIDv5_invariants;
        require Invariant_REQ_0090_Metamodel_specialization_semantic_neutrality_dictionary_mapping_typing_and_UUIDv5_invariants;
        verify by AC_01_Canonical_Metadata_Lowering_and_Annotation;
        verify by AC_02_Target_Metaclass_Restriction_Enforcement;
        verify by AC_03_Attribute_Type_Incompatibility_Rejection;
        verify by AC_04_Delimiter_Synchronization_and_Error_Recovery;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: a3a2c4d9-3a7c-5e7e-805c-6b6114f0d491 | Complexity: Class P (O(1) lookup / equality) | Standard: IEEE 29148-2018 / RFC 2119 / OMG KerML §7.2 | Diagnostics: E0199, E0200, E0203 (per REQ-0191) */
    requirement def REQ_0091_KerML_Standard_Library {
        id = "REQ-0091";
        text = "The compiler SHALL bootstrap KerML Base (Package_Base) into root scope at initialization, providing Anything, classifiers Element and Relationship, Nothing, and operators Op_Equal and Op_NotEqual.";
        attribute uuidv5 : String = "a3a2c4d9-3a7c-5e7e-805c-6b6114f0d491";
        attribute complexity_class : String = "Class P (O(1) lookup / equality)";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119 / OMG KerML §7.2";
        attribute diagnostic_codes : String = "E0199, E0200, E0203 (per REQ-0191)";
        attribute ac_01_canonical_base_bootstrapping : String = "Given: Initialized compiler environment. When: Resolving root library symbols. Then: Anything, Element, Relationship, Nothing, ==, and != resolve in O(1). Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_top_type_maximality_and_specialization : String = "Given: Metamodel with classifier Classifier_Alpha. When: Evaluating subtyping queries. Then: Classifier_Alpha <= Anything, Element <= Anything, and Relationship <= Element evaluate true; Anything <= Classifier_Alpha is false. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_primitive_equality_semantics : String = "Given: Values a, b V_{val} with congruent fields, and distinct entities u, v O_{obj} (id(u) id(v)). When: Evaluating a == b, u == v, and cross-domain a == u. Then: a == b is true; u == v and a == u are false. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_04_reserved_symbol_and_cycle_rejection : String = "Given: Model redeclaring Anything or declaring cycle Anything :> Classifier_Alpha. When: Namespace validation and cycle detection execute. Then: Rejection emits E0200 or E0203; recovery halts at C_{max_errors}} with E0199. Diagnostic: E0200, E0203, or E0199.";
        doc /* \begin{aligned} \forall T \in \mathcal{T},\; &\bot \le T \le \top,\; \top = \text{Anything},\; \bot = \text{Nothing} \\ \text{Relationship} &\le \text{Element} \le \text{Anything} \le C \implies C = \text{Anything} \\ \mathcal{U}_{\text{univ}} &= \mathcal{V}_{\text{val}} \cup \mathcal{O}_{\text{obj}},\; \mathcal{V}_{\text{val}} \cap \mathcal{O}_{\text{obj}} = \emptyset \\ (a == b) &\iff \begin{cases} a \equiv_{\text{val}} b, & a, b \in \mathcal{V}_{\text{val}} \\ \text{id}(a) = \text{id}(b), & a, b \in \mathcal{O}_{\text{obj}} \\ \text{false}, & \text{otherwise} \end{cases} \\ (a != b) &\iff \neg(a == b),\; \Delta_n = \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0091_Subtyping_meet_semilattice_equivalence_semantics_and_identity_invariants;
        require Invariant_REQ_0091_Subtyping_meet_semilattice_equivalence_semantics_and_identity_invariants;
        verify by AC_01_Canonical_Base_Bootstrapping;
        verify by AC_02_Top_Type_Maximality_and_Specialization;
        verify by AC_03_Primitive_Equality_Semantics;
        verify by AC_04_Reserved_Symbol_and_Cycle_Rejection;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: 2b01c22c-5f72-5c7c-9289-643bd51ff6c5 | Complexity: Class P (O(1) scalar lowering, O(L) string interning) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG KerML §7.3 | Diagnostics: E0199, E0200 (per REQ-0191) */
    requirement def REQ_0092_Canonical_Scalar_Type_Lowering {
        id = "REQ-0092";
        text = "The compiler SHALL lower KerML ScalarValues types (Boolean, Integer, Real, String) and numeric subtypes (Natural, Positive) into canonical machine representations, preserving semantic value precision, exact integer boundaries, and IEEE 754-2019 binary64 semantics.";
        attribute uuidv5 : String = "2b01c22c-5f72-5c7c-9289-643bd51ff6c5";
        attribute complexity_class : String = "Class P (O(1) scalar lowering, O(L) string interning)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG KerML §7.3";
        attribute diagnostic_codes : String = "E0199, E0200 (per REQ-0191)";
        attribute ac_01_canonical_scalar_lowering : String = "Given: Package Package_0 declaring scalar attributes val_b : Boolean = true, val_i : Integer = 42, val_r : Real = 3.14159, and val_s : String = \"alpha\". When: Lowering scalar expressions into arena storage. Then: Values map to B, Z_{64}, R_{64}, and interned SymbolId without loss. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_monotonic_widening_precision : String = "Given: Literals n_1 = 2^{53} and n_2 = 2^{53} + 1 promoted to Real. When: Applying widening injection . Then: n_1 promotes losslessly (float64(n_1) = n_1); n_2 rounds within 1{2}ulp. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_integer_overflow_malformed_literal_rejection : String = "Given: Integer literal exceeding 2^{63}-1 or malformed float literal. When: Parsing and evaluating literal expressions. Then: Rejection assigns sentinel _{scalar} and emits E0200. Diagnostic: E0200 (Scalar literal bounds or syntax violation).";
        attribute ac_04_narrowing_prohibition_delimiter_recovery : String = "Given: Narrowing assignment param_y : Integer = 3.14159 followed by ; and val_pos : Positive = 1. When: Type checking and recovery execute. Then: Rejection emits E0200, resynchronizes at ;, lowers subsequent declarations, and halts at C_{max_errors}} with E0199. Diagnostic: E0200; E0199 upon error threshold saturation.";
        doc /* \begin{aligned} \text{Positive} &\le \text{Natural} \le \text{Integer} \le \text{Real},\quad \mathcal{T}_{\text{disjoint}} = \{ \mathbb{B}, \text{SymbolId}, \mathbb{R}_{64} \} \\ ⟦ \text{Boolean} ⟧ &\longleftrightarrow \mathbb{B},\quad ⟦ \text{String} ⟧ \longleftrightarrow \text{SymbolId},\quad ⟦ \text{Integer} ⟧ \longleftrightarrow \mathbb{Z}_{64},\quad ⟦ \text{Real} ⟧ \longleftrightarrow \mathbb{R}_{64} \\ \forall n \in \mathbb{Z}_{64}, &\; (|n| \le 2^{53} \implies \text{float64}(n) = n) \;\wedge\; (|n| \gt 2^{53} \implies |\text{float64}(n) - n| \le \tfrac{1}{2} \text{ulp}(n)) \\ \forall \tau_1, \tau_2 \in \mathcal{T}_{\text{scalar}}, &\; \tau_1 \not\le \tau_2 \implies \text{Coerce}(\tau_1, \tau_2) = \bot_{\text{scalar}},\quad \Delta_v = \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(v)) \end{aligned} */
        constraint def Invariant_REQ_0092_Subtyping_lattice_canonical_mappings_precision_preservation_and_disjointness;
        require Invariant_REQ_0092_Subtyping_lattice_canonical_mappings_precision_preservation_and_disjointness;
        verify by AC_01_Canonical_Scalar_Lowering;
        verify by AC_02_Monotonic_Widening_Precision;
        verify by AC_03_Integer_Overflow_Malformed_Literal_Rejection;
        verify by AC_04_Narrowing_Prohibition_Delimiter_Recovery;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: 0b2b3058-0632-56d4-94bd-cb187b99762b | Complexity: Class P (O(1) subtyping, O( | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG KerML §7.3 | Diagnostics: E0199, E0200, E0201, E0203 (per REQ-0191) */
    requirement def REQ_0093_KerML_Standard_Library {
        id = "REQ-0093";
        text = "The compiler SHALL bootstrap KerML Collections (Package_Collections) into root scope at initialization, providing Collection<T>, Set<T>, Sequence<T>, OrderedSet<T>, and Bag<T>.";
        attribute uuidv5 : String = "0b2b3058-0632-56d4-94bd-cb187b99762b";
        attribute complexity_class : String = "Class P (O(1) subtyping, O(";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG KerML §7.3";
        attribute diagnostic_codes : String = "E0199, E0200, E0201, E0203 (per REQ-0191)";
        attribute ac_01_canonical_collections_bootstrapping : String = "Given: Initialized compiler environment. When: Bootstrapping standard library packages. Then: Collection, Set, Sequence, OrderedSet, and Bag resolve in O(1) with exact matrix flags. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_covariant_subtyping_multiple_inheritance : String = "Given: Classifier_Alpha <= Classifier_Beta in Package_0. When: Evaluating subtyping queries. Then: OrderedSet<Classifier_Alpha> <= Set<Classifier_Alpha>, OrderedSet<Classifier_Alpha> <= Sequence<Classifier_Alpha>, and Set<Classifier_Alpha> <= Set<Classifier_Beta> evaluate true. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_subtyping_and_element_type_incompatibility : String = "Given: Set<Classifier_Alpha> assigned Set<Classifier_Beta> with Classifier_Beta not specializing Classifier_Alpha. When: Validating subtyping and elements. Then: Lowering rejects assignment, binds sentinel _{type}, and emits E0200. Diagnostic: E0200 (Subtyping or element mismatch).";
        attribute ac_04_parameter_directionality_and_cycle_rejection : String = "Given: Model with InOut parameter or alias cycle Type_A = Set<Type_B> and Type_B = Sequence<Type_A>. When: Signature validation and cycle analysis execute. Then: Rejection emits E0201 or E0203; recovery halts at C_{max_errors}} with E0199. Diagnostic: E0201, E0203, or E0199.";
        doc /* \begin{aligned} \mathcal{T}_{\text{coll}} &= \{\text{Collection}, \text{Set}, \text{Sequence}, \text{OrderedSet}, \text{Bag}\} \\ \forall T \in \mathcal{T},\; &\text{OrderedSet}\langle T \rangle = \text{Set}\langle T \rangle \wedge \text{Sequence}\langle T \rangle \le \text{Bag}\langle T \rangle \le \text{Collection}\langle T \rangle \\ \forall \mathcal{C} \in \mathcal{T}_{\text{coll}},\; &T_1 \le T_2 \implies \mathcal{C}\langle T_1 \rangle \le \mathcal{C}\langle T_2 \rangle \\ \forall p \in \mathcal{P}_{\text{op}},\; &\text{dir}(p) = \text{In},\quad \Delta_n = \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0093_Metamodel_subtyping_covariance_and_UUIDv5_invariants;
        require Invariant_REQ_0093_Metamodel_subtyping_covariance_and_UUIDv5_invariants;
        verify by AC_01_Canonical_Collections_Bootstrapping;
        verify by AC_02_Covariant_Subtyping_Multiple_Inheritance;
        verify by AC_03_Subtyping_and_Element_Type_Incompatibility;
        verify by AC_04_Parameter_Directionality_and_Cycle_Rejection;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: 59baacb8-6746-534a-b047-2e0eb46e4d6e | Complexity: Class P (O(1) token dispatch, O(V + E) topology verification) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG KerML §7.4, §8.3 | Diagnostics: E0199, E0200, E0202, E0203 (per REQ-0191) */
    requirement def REQ_0094_KerML_Standard_Library {
        id = "REQ-0094";
        text = "The compiler SHALL bootstrap KerML ControlFunctions (Package_ControlFunctions) into root scope at initialization, providing ControlNode (specializing Feature and Step) and primitive functions Control_Fork, Control_Join, Control_Decision, and Control_Merge (specializing ControlNode and Function).";
        attribute uuidv5 : String = "59baacb8-6746-534a-b047-2e0eb46e4d6e";
        attribute complexity_class : String = "Class P (O(1) token dispatch, O(V + E) topology verification)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG KerML §7.4, §8.3";
        attribute diagnostic_codes : String = "E0199, E0200, E0202, E0203 (per REQ-0191)";
        attribute ac_01_canonical_library_bootstrap : String = "Given: Initialized compiler root environment. When: Standard library Package_ControlFunctions is bootstrapped. Then: Control_Fork, Control_Join, Control_Decision, and Control_Merge resolve as specializations of ControlNode and Function with stable 32-bit arena handles. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_fork_join_token_synchronization : String = "Given: Model declaring Control_Fork distributing to Step_Alpha and Step_Beta, converging into Control_Join. When: Discrete execution simulation evaluates token flow. Then: Control_Fork replicates 1 token into 2 concurrent tokens; Control_Join awaits both tokens and fires 1 synchronized token upon complete arrival. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_malformed_control_node_degree_rejection : String = "Given: Model declaring Control_Fork with in-degree 2 or out-degree 1, or Control_Join with in-degree 1. When: Static topology degree verification executes. Then: Lowering rejects malformed nodes and emits diagnostic E0200. Diagnostic: E0200 (Malformed control topology).";
        attribute ac_04_guard_typing_and_deadlock_detection : String = "Given: Model declaring Control_Decision with non-Boolean guard param_x + 1.0 or overlapping guards, or Control_Join with un-forked decision inputs. When: Semantic type analysis and deadlock freedom validation execute. Then: Rejection emits E0202 for non-Boolean guards and E0200 for ambiguous guards or join deadlock, binding sentinel _{ctrl}, and emitting E0199 at C_{max_errors}}. Diagnostic: E0200, E0202, or E0199.";
        doc /* \begin{aligned} \mathcal{C}_{\text{ctrl}} &= \{\text{Control\_Fork}, \text{Control\_Join}, \text{Control\_Decision}, \text{Control\_Merge}\} \subset \text{ControlNode} \cap \text{Function} \\ \forall u \in V_{\text{ctrl}}, &\quad |\text{InEdges}(u)|=d_{\text{in}}(u), \quad |\text{OutEdges}(u)|=d_{\text{out}}(u) \\ \text{Fork}(u) &: M(e_{\text{in}})\ge 1 \implies M'(e_{\text{in}})=M(e_{\text{in}})-1 \wedge (\forall e\in\text{OutEdges},\; M'(e)=M(e)+1) \\ \text{Join}(u) &: (\forall e\in\text{InEdges},\; M(e)\ge 1) \implies (\forall e,\; M'(e)=M(e)-1) \wedge M'(e_{\text{out}})=M(e_{\text{out}})+1 \\ \text{Decision}(u) &: (M(e_{\text{in}})\ge 1 \wedge g_i(\sigma)) \implies M'(e_{\text{in}})=M(e_{\text{in}})-1 \wedge M'(e_i)=M(e_i)+1 \\ \text{Merge}(u) &: (\exists e_j\in\text{InEdges} : M(e_j)\ge 1) \implies M'(e_j)=M(e_j)-1 \wedge M'(e_{\text{out}})=M(e_{\text{out}})+1 \\ \forall u\in V_{\text{dec}}, &\quad \forall i\ne j,\; \neg(g_i(\sigma)\wedge g_j(\sigma)) \wedge \tau(g_i)\equiv\text{Boolean} \\ \forall u\in V_{\text{join}}, &\quad \exists w\in V_{\text{fork}} : \text{Reachable}(w, u) \wedge \text{SAT}\left(\bigwedge_e \text{Guards}(w \rightsquigarrow e)\right) \\ \forall u\in V_{\text{ctrl}}, &\quad (u, u)\notin E_{\text{ctrl,node}}^+ \wedge \Delta_u=\text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(u)) \end{aligned} */
        constraint def Invariant_REQ_0094_Control_classifier_specialization_degree_constraints_token_transition_semantics_guard_properties_and_cycle_freedom_invariants;
        require Invariant_REQ_0094_Control_classifier_specialization_degree_constraints_token_transition_semantics_guard_properties_and_cycle_freedom_invariants;
        verify by AC_01_Canonical_Library_Bootstrap;
        verify by AC_02_Fork_Join_Token_Synchronization;
        verify by AC_03_Malformed_Control_Node_Degree_Rejection;
        verify by AC_04_Guard_Typing_and_Deadlock_Detection;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: 7f0247fc-685c-500d-b767-3dc05b1cbb74 | Complexity: Class P (O(1) scale check, O(N) lowering) | Standard: IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.15 | Diagnostics: E0199, E0200, E0300 (per REQ-0191) */
    requirement def REQ_0095_SysML_v2_Quantities_Library {
        id = "REQ-0095";
        text = "The compiler SHALL bootstrap SysML v2 Quantities (Package_Quantities) into root scope, formalizing quantities as magnitude-unit pairs and classifying ratio and interval scales.";
        attribute uuidv5 : String = "7f0247fc-685c-500d-b767-3dc05b1cbb74";
        attribute complexity_class : String = "Class P (O(1) scale check, O(N) lowering)";
        attribute governing_standard : String = "IEEE 29148-2018 §8.4 / RFC 2119 / OMG SysML v2 §7.15";
        attribute diagnostic_codes : String = "E0199, E0200, E0300 (per REQ-0191)";
        attribute ac_01_canonical_quantity_lowering : String = "Given: Package_Quantities with ratio scale Scale_0 and unit Unit_Alpha. When: Lowering Quantity_Alpha = 10.0 [Unit_Alpha] and evaluating 2.5 * Quantity_Alpha. Then: Node binds magnitude 25.0, unit Unit_Alpha, scale Scale_0. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_interval_scale_operations : String = "Given: Interval scale Scale_1 with units Unit_Alpha and Unit_Beta (m = 1.8, c = 32.0). When: Lowering difference Quantity_Alpha - Quantity_Beta and sum Quantity_Alpha + Quantity_Delta. Then: Difference lowers to Q_{ Q_{ratio}; sum lowers to interval position in Scale_1. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_invalid_interval_scale_rejection : String = "Given: Model attempting addition (Quantity_Alpha + Quantity_Beta) or scaling (2.0 * Quantity_Alpha) in Scale_1. When: Validating scale invariants. Then: Rejection emits E0200 and binds sentinel _{quantity}. Diagnostic: E0200 (Invalid interval scale operation).";
        attribute ac_04_dimensional_homogeneity_rejection : String = "Given: Model adding heterogeneous quantities (d_1 d_2). When: Validating dimensional consistency. Then: Rejection halts lowering and emits E0300. Diagnostic: E0300 (Dimensional homogeneity violation).";
        doc /* \begin{aligned} \text{RatioScale} &\le \text{MeasurementScale} \le \text{DataType},\; \text{IntervalScale} \le \text{MeasurementScale} \\ ⟦ \text{RatioScale} ⟧ &\cap ⟦ \text{IntervalScale} ⟧ = \emptyset,\; \text{Unit} \le \text{DataType} \\ Q &= \langle v, u, S, \mathbf{d} \rangle \in \mathbb{R} \times \mathcal{U} \times \mathcal{MS} \times \mathbb{Q}^7 \\ Q_1, Q_2 \in \mathcal{Q}_{\text{interval}} &\implies Q_1 - Q_2 \in \mathcal{Q}_{\text{ratio}},\; Q_{\text{pos}} + Q_{\Delta} \in \mathcal{Q}_{\text{interval}} \\ (Q_1 \pm Q_2) &\implies \mathbf{d}(Q_1) \equiv \mathbf{d}(Q_2),\; \Delta_n = \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0095_Measurement_scale_semantics_transformation_groups_and_dimensional_invariants;
        require Invariant_REQ_0095_Measurement_scale_semantics_transformation_groups_and_dimensional_invariants;
        verify by AC_01_Canonical_Quantity_Lowering;
        verify by AC_02_Interval_Scale_Operations;
        verify by AC_03_Invalid_Interval_Scale_Rejection;
        verify by AC_04_Dimensional_Homogeneity_Rejection;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: 9ebc08b9-22aa-5436-ae7a-e400a883c776 | Complexity: Class P (O(1) vector ops, O(N) lowering) | Standard: IEEE 29148-2018 / RFC 2119 / ISO 80000 | Diagnostics: E0199, E0200, E0203, E0300 */
    requirement def REQ_0096_SysML_v2_Quantities_Library {
        id = "REQ-0096";
        text = "The compiler SHALL bootstrap Package_ISQBase and Package_ISQ into root scope at initialization.";
        attribute uuidv5 : String = "9ebc08b9-22aa-5436-ae7a-e400a883c776";
        attribute complexity_class : String = "Class P (O(1) vector ops, O(N) lowering)";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119 / ISO 80000";
        attribute diagnostic_codes : String = "E0199, E0200, E0203, E0300";
        attribute ac_01_canonical_library_bootstrapping : String = "Given: Initialized compiler runtime. When: Bootstrapping Package_ISQBase and Package_ISQ. Then: 7 base dimensions map to e_1 e_7, derived dimensions register coordinates, and nodes bind deterministic UUIDv5 IDs. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_dimensional_vector_arithmetic_and_inference : String = "Given: Quantities Quantity_A (Force) and Quantity_B (Length). When: Lowering Quantity_A * Quantity_B, Quantity_A / Quantity_B, Quantity_B^2. Then: Product infers Energy in O(1), quotient infers (1,0,-2,0,0,0,0), and power infers Area. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_static_dimensional_homogeneity_rejection : String = "Given: Expression Quantity_A + Quantity_B with mismatched dimensions (d(Q_A) d(Q_B)). When: Dimensional type checking executes. Then: Rejection emits E0300 and binds sentinel _{dim} to continue processing. Diagnostic: E0300 (DimensionalMismatch).";
        attribute ac_04_cyclic_derivation_immutability_and_exponent_range_rejection : String = "Given: Cyclic derivation Dimension_Alpha :> Dimension_Beta or mutation in Package_ISQBase. When: Semantic validation and cycle detection run. Then: Cycle emits E0203, mutation emits E0200, and recovery halts at C_{max_errors}} with E0199. Diagnostic: E0203, E0200, or E0199.";
        doc /* \begin{aligned} \mathcal{B}_{\text{ISQ}} &= \{\mathbf{e}_M, \dots, \mathbf{e}_J\} \subset \mathbb{Z}^7,\; \mathbf{d}(D) \in [-128, 127]^7 \\ \mathbf{d}(Q_1 \cdot Q_2) &= \mathbf{d}(Q_1) + \mathbf{d}(Q_2),\; \mathbf{d}(Q_1 / Q_2) = \mathbf{d}(Q_1) - \mathbf{d}(Q_2),\; \mathbf{d}(Q^k) = k \cdot \mathbf{d}(Q) \\ \Phi_{\text{homogenous}} &\iff \mathbf{d}(A) = \mathbf{d}(B),\; \neg \Phi \wedge (A \pm B \vee A == B) \implies \text{Diagnostic}(n, \text{E0300}) \\ (D, D) \in \mathcal{R}_{\text{dim\_dep}}^+ &\implies \text{Diagnostic}(D, \text{E0203}),\; d_i(D) \notin [-128, 127] \implies \text{Diagnostic}(D, \text{E0200}) \\ \Delta_n &= \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0096_Canonical_basis_lattice_group_laws_homogeneity_predicates_and_diagnostic_emission_rules;
        require Invariant_REQ_0096_Canonical_basis_lattice_group_laws_homogeneity_predicates_and_diagnostic_emission_rules;
        verify by AC_01_Canonical_Library_Bootstrapping;
        verify by AC_02_Dimensional_Vector_Arithmetic_and_Inference;
        verify by AC_03_Static_Dimensional_Homogeneity_Rejection;
        verify by AC_04_Cyclic_Derivation_Immutability_and_Exponent_Range_Rejection;
        satisfy by GrammarLoweringEngine;
    }
    doc /* UUIDv5: a59cc0bd-9e2b-5c09-8eb4-30712de43c71 | Complexity: Class P (O(1) conversion) | Standard: IEEE 29148-2018 / RFC 2119 / OMG SysML v2 §7.15 | Diagnostics: E0199, E0200, E0201, E0203 (per REQ-0191) */
    requirement def REQ_0097_SysML_v2_Quantities_Library {
        id = "REQ-0097";
        text = "The compiler SHALL bootstrap Package_SI and Package_SIPrefixes into root scope, lowering UnitDefinition and PrefixDefinition into immutable AST nodes.";
        attribute uuidv5 : String = "a59cc0bd-9e2b-5c09-8eb4-30712de43c71";
        attribute complexity_class : String = "Class P (O(1) conversion)";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119 / OMG SysML v2 §7.15";
        attribute diagnostic_codes : String = "E0199, E0200, E0201, E0203 (per REQ-0191)";
        attribute ac_01_canonical_si_base_derived_unit_bootstrapping : String = "Given: Compiler loading Package_SI. When: Resolving base and derived units. Then: Base units lower with c = 1, o = 0 and basis e_i Z^7; derived units preserve power products. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_02_metric_prefix_scaling_exact_rational_conversion : String = "Given: Unit Unit_Alpha (c Q^+) and prefix Prefix_Alpha (10^k). When: Lowering Prefix_Alpha Unit_Alpha and evaluating conversion. Then: Factor evaluates to 10^k c in exact arithmetic Q without drift. Diagnostic: Diagnostic sink remains empty.";
        attribute ac_03_incommensurable_unit_conversion_rejection : String = "Given: Incommensurable Unit_Alpha and Unit_Beta (d(Unit_Alpha}) d(Unit_Beta})). When: Dimensional validation executes. Then: Lowering halts conversion, assigns sentinel _{unit}, and records E0201. Diagnostic: E0201.";
        attribute ac_04_malformed_unit_specialization_delimiter_recovery : String = "Given: Malformed unit def Unit_Alpha :> Classifier_Alpha; preceding valid unit def Unit_Beta;. When: Delimiter recovery executes. Then: Lowering rejects Unit_Alpha with E0200, recovers on delimiter, and halts at C_{max_errors}} with E0199. Diagnostic: E0200, E0199.";
        doc /* \begin{aligned} \forall U \in \mathcal{U},\; &\mathbf{d}(U) \in \mathbb{Z}^7,\; c_U \in \mathbb{Q}^+,\; o_U \in \mathbb{Q} \\ \mathcal{B}=\{\text{kg},\text{m},\text{s},\text{A},\text{K},\text{mol},\text{cd}\},\; &\mathbf{d}(B_i)=\mathbf{e}_i,\; c_{B_i}=1,\; o_{B_i}=0 \\ \text{Prefix}(k, U) \implies &\mathbf{d}(P \cdot U)=\mathbf{d}(U),\; c_{P \cdot U}=10^k \cdot c_U \\ U_1 \sim U_2 \iff \mathbf{d}(U_1)=\mathbf{d}(U_2),\; &\text{convert}(v, U_1, U_2)=\frac{c_{U_1} \cdot v + o_{U_1} - o_{U_2}}{c_{U_2}} \\ \forall U \in \mathcal{U},\; (U, U) \notin \mathcal{R}_{\text{unit\_dep}}^+,\; &\Delta_n = \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(n)) \end{aligned} */
        constraint def Invariant_REQ_0097_Dimensional_lattice_and_conversion_invariants;
        require Invariant_REQ_0097_Dimensional_lattice_and_conversion_invariants;
        verify by AC_01_Canonical_SI_Base_Derived_Unit_Bootstrapping;
        verify by AC_02_Metric_Prefix_Scaling_Exact_Rational_Conversion;
        verify by AC_03_Incommensurable_Unit_Conversion_Rejection;
        verify by AC_04_Malformed_Unit_Specialization_Delimiter_Recovery;
        satisfy by GrammarLoweringEngine;
    }
}
