doc /* Core Metamodel, Node Arena & AST Graph Engine */
package CoreMetamodelNodeArena {
    doc /* UUIDv5: daf29dfd-e618-5203-9859-b49422a84842 | Complexity: Class P (O(1)) | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0200 */
    requirement def REQ_0034_Contiguous_Memory_Lowering {
        id = "REQ-0034";
        text = "The compiler core metamodel lowering engine shall lower all ingested schema abstract syntax tree (AST) entities (packages, part definitions, port definitions, action definitions, constraint definitions) into contiguous virtual memory allocations within a typed node arena. AST entities must be";
        attribute uuidv5 : String = "daf29dfd-e618-5203-9859-b49422a84842";
        attribute complexity_class : String = "Class P (O(1))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0200";
        attribute ac_01_contiguous_virtual_memory_layout_alignment : String = "Given: AST entities (Package_0, PartDef_Alpha) lowered into arena A_{node} with N = 10^4 nodes. When: Lowering commits entities into sequential arena slots. Then: For all consecutive pairs (i, i+1), Address(Node_{i+1}) - Address(Node_i) = S_{slot} and Address(Node_i) 0 8. Diagnostic: E0200 emitted on non-contiguous layout or alignment violation.";
        attribute ac_02_constant_time_random_access_zero_pointer_indirection : String = "Given: Node arena with N = 10^5 AST nodes addressed via handles (NodeIndex). When: Executing 10^6 randomized index lookups across the arena buffer. Then: Every slot dereference computes in O(1) constant time via base-plus-offset arithmetic with PointerIndirectionDepth(A_{node}) = 0. Diagnostic: E0200 emitted on invalid handle dereference or non-zero pointer indirection.";
        attribute ac_03_linear_lowering_throughput_scaling : String = "Given: Synthetic AST models scaling from N = 10^3 to N = 10^6 entities. When: Lowering transforms the model into the contiguous arena in a single pass. Then: Lowering latency satisfies T_{lower}(N) c_1 N with regression fit R^2 0.99. Diagnostic: E0200 emitted if lowering throughput exceeds linear algorithmic bounds.";
        attribute ac_04_arena_boundary_enforcement_non_panicking_diagnostic_recovery : String = "Given: Arena buffer with fixed capacity C_{slots} and invalid index request (i C_{slots}). When: An operation attempts slot allocation or lookup exceeding capacity limits. Then: Execution halts gracefully without panicking, recording structured diagnostic E0200. Diagnostic: E0200 recorded upon arena capacity exhaustion or out-of-bounds slot access.";
        doc /* \begin{aligned} \forall i \in \{0, \dots, N-1\}, \quad \text{Address}(\text{Node}_i) &= \text{Base}(\mathcal{A}_{\text{node}}) + i \cdot S_{\text{slot}} \\ \forall i \in \{0, \dots, N-2\}, \quad \text{Address}(\text{Node}_{i+1}) - \text{Address}(\text{Node}_i) &= S_{\text{slot}} \\ \forall i \in \{0, \dots, N-1\}, \quad \text{Address}(\text{Node}_i) &\equiv 0 \pmod{A_{\text{align}}} \end{aligned} */
        constraint def Invariant_REQ_0034_Contiguous_Address_Arithmetic_and_Alignment_Invariant;
        doc /* \begin{aligned} \text{PointerIndirectionDepth}(\mathcal{A}_{\text{node}}) &= 0 \\ \forall (u, v) \in \mathcal{E}_{\text{AST}}, \quad \text{TargetHandle}(u, v) &\in \mathbb{N}_0 \end{aligned} */
        constraint def Invariant_REQ_0034_Zero_Pointer_Indirection_Depth_Invariant;
        doc /* \begin{aligned} T_{\text{lower}}(N) &\le c_1 \cdot N + c_0, \quad \forall N \ge 1 \\ T_{\text{access}}(i) &\le c_2 \cdot 1, \quad \forall i \in \{0, \dots, N-1\} \end{aligned} */
        constraint def Invariant_REQ_0034_Asymptotic_Lowering_and_Constant_Time_Access_Invariants;
        doc /* \mathcal{M}_{\text{cache}}(k) \le \left\lceil \frac{k \cdot S_{\text{slot}}}{L_{\text{cache}}} \right\rceil + 1 */
        constraint def Invariant_REQ_0034_Spatial_Cache_Line_Locality_Bound;
        doc /* i \ge C_{\text{slots}} \implies \text{Diagnostic}(\text{span}(\text{Node}_i), \text{E0200}) */
        constraint def Invariant_REQ_0034_Capacity_and_Diagnostic_Invariant;
        require Invariant_REQ_0034_Contiguous_Address_Arithmetic_and_Alignment_Invariant;
        require Invariant_REQ_0034_Zero_Pointer_Indirection_Depth_Invariant;
        require Invariant_REQ_0034_Asymptotic_Lowering_and_Constant_Time_Access_Invariants;
        require Invariant_REQ_0034_Spatial_Cache_Line_Locality_Bound;
        require Invariant_REQ_0034_Capacity_and_Diagnostic_Invariant;
        verify by AC_01_Contiguous_Virtual_Memory_Layout_Alignment;
        verify by AC_02_Constant_Time_Random_Access_Zero_Pointer_Indirection;
        verify by AC_03_Linear_Lowering_Throughput_Scaling;
        verify by AC_04_Arena_Boundary_Enforcement_Non_Panicking_Diagnostic_Recovery;
        satisfy by NodeArenaASTGraphEngine;
    }
    doc /* UUIDv5: 2027c355-5976-5f18-85f6-1c40e2e06bba | Complexity: Class P (O(1)) | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0200 */
    requirement def REQ_0035_Deterministic_Node_Handle_Addressing {
        id = "REQ-0035";
        text = "The compiler metamodel, node arena storage, and abstract syntax tree (AST) graph engine shall address, reference, and interconnect all AST nodes throughout the compilation pipeline exclusively via deterministic typed integer handles (NodeHandle<T>). Raw virtual memory pointers (*const T, *mut T),";
        attribute uuidv5 : String = "2027c355-5976-5f18-85f6-1c40e2e06bba";
        attribute complexity_class : String = "Class P (O(1))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0200";
        attribute ac_01_generational_invalidation_on_slot_recycling : String = "Given: Arena A with slot k containing Classifier_Alpha referenced by handle H_1 = k, g, Tag_{Alpha} . When: Slot k is recycled to store Classifier_Beta, advancing its generation counter to (g + 1) 2^{16}}. Then: Evaluating Deref(H_1, A) deterministically returns , while newly allocated handle H_2 = k, g + 1, Tag_{Beta} dereferences in O(1) time. Diagnostic: E0200 emitted on attempted access via stale handle H_1.";
        attribute ac_02_type_tag_discriminator_validation : String = "Given: Arena slot k storing Package_0 with tag Tag_{Package}, and handle H_{mismatch} = k, g, Tag_{Port} . When: The compiler evaluates Deref(H_{mismatch}, A). Then: Validation fails, dereferencing evaluates to , and execution halts without panics. Diagnostic: E0200 emitted on type tag mismatch.";
        attribute ac_03_arena_capacity_bounds_and_sentinel_safety : String = "Given: Arena A of capacity C_{arena}, out-of-bounds handle H_{oob} = C_{arena} + 10, 0, 0 , and null sentinel H_{null} = _{pack}(2^{32} - 1, 0, 0). When: The compiler evaluates Deref(H_{oob}, A) or Deref(H_{null}, A). Then: Both evaluate to , preserving Valid(H, A) = false and zero dangling pointers. Diagnostic: E0200 emitted when dereferencing out-of-bounds or null handles.";
        attribute ac_04_cross_platform_serialization_parity : String = "Given: An AST graph with entities Package_0, Classifier_Alpha, and Port_1 interconnected via typed node handles. When: The graph is serialized to CBOR/JSON (REQ-0044) and deserialized in independent processes. Then: Deserialized handle integers, slot offsets, and edge topologies exhibit 100% bitwise parity without pointer swizzling. Diagnostic: E0200 emitted on deserialization handle corruption.";
        attribute ac_05_panic_free_release_mode_latency_and_memory_bounds : String = "Given: An AST model with 10^6 nodes allocated in contiguous node arena chunks. When: Dereferencing throughput and arena memory are benchmarked in Release mode (--release) under an isolated harness (Directive 5, REQ-0196). Then: Dereferencing throughput exceeds 10^8 ops/s (latency 10 ns) with peak RSS below 100 MB and zero panics. Diagnostic: E0200 emitted if memory corruption or runtime failure occurs.";
        doc /* H = \langle \text{Index}, \text{Generation}, \text{TypeTag} \rangle \in \mathbb{U}_{32} \times \mathbb{U}_{16} \times \mathbb{U}_{16} */
        constraint def Invariant_REQ_0035_FormalInvariant;
        doc /* \begin{aligned} \pi_{\text{pack}}(\text{Index}, \text{Generation}, \text{TypeTag}) &= (\text{Index} \ll 32) \lor (\text{Generation} \ll 16) \lor \text{TypeTag} \\ \text{Index}(H) &= (H \gg 32) \land (2^{32} - 1) \\ \text{Generation}(H) &= (H \gg 16) \land (2^{16} - 1) \\ \text{TypeTag}(H) &= H \land (2^{16} - 1) \end{aligned} */
        constraint def Invariant_REQ_0035_Bit_packing_projection_and_decoding_projections_satisfy;
        doc /* \begin{aligned} \text{Valid}(H, \mathcal{A}) \iff &\; \text{Index}(H) \lt C_{\text{arena}} \wedge \mathcal{A}.\text{slots}[\text{Index}(H)].\text{allocated} \wedge \\ &\; \mathcal{A}.\text{slots}[\text{Index}(H)].\text{gen} = \text{Generation}(H) \wedge \mathcal{A}.\text{slots}[\text{Index}(H)].\text{tag} = \text{TypeTag}(H) \end{aligned} */
        constraint def Invariant_REQ_0035_FormalInvariant_2;
        doc /* \text{Deref}(H, \mathcal{A}) = \begin{cases} \text{ref}(\mathcal{A}.\text{slots}[\text{Index}(H)].\text{node}), & \text{if } \text{Valid}(H, \mathcal{A}) = \text{true} \\ \bot, & \text{otherwise} \end{cases} */
        constraint def Invariant_REQ_0035_FormalInvariant_3;
        doc /* \mathcal{A}.\text{slots}[i].\text{gen}_{t+1} = (\mathcal{A}.\text{slots}[i].\text{gen}_t + 1) \pmod{2^{16}} \implies \text{Deref}(H_t, \mathcal{A}_{t+1}) = \bot */
        constraint def Invariant_REQ_0035_Slot_Reallocation_Invalidation;
        doc /* \text{DanglingPointers}(\text{NodeArena}) = 0 \quad \wedge \quad \text{Deref}(H_{\text{null}}, \mathcal{A}) = \bot */
        constraint def Invariant_REQ_0035_Zero_Dangling_Pointers_Null_Sentinel;
        require Invariant_REQ_0035_FormalInvariant;
        require Invariant_REQ_0035_Bit_packing_projection_and_decoding_projections_satisfy;
        require Invariant_REQ_0035_FormalInvariant_2;
        require Invariant_REQ_0035_FormalInvariant_3;
        require Invariant_REQ_0035_Slot_Reallocation_Invalidation;
        require Invariant_REQ_0035_Zero_Dangling_Pointers_Null_Sentinel;
        derived from REQ_0191_Standardized_Compiler_Diagnostic_Taxonomy_Source_Mapping;
        derived from REQ_0011_Deterministic_Non_Panicking_Execution_Contract;
        derived from REQ_0003_Bitwise_Determinism;
        derived from REQ_0010_Deterministic_RFC_4122_UUIDv5_Topological_Namespace_Hashing;
        derived from REQ_0044_Lossless_AST_Serialization;
        derived from REQ_0050_AST_Slice_Serialization_Contract_with_Reified_Dependency_Envelopes;
        derived from REQ_0012_Bounded_Work_Stealing_Parallelism;
        derived from REQ_0036_Concurrent_String_Interning;
        verify by AC_01_Generational_Invalidation_on_Slot_Recycling;
        verify by AC_02_Type_Tag_Discriminator_Validation;
        verify by AC_03_Arena_Capacity_Bounds_and_Sentinel_Safety;
        verify by AC_04_Cross_Platform_Serialization_Parity;
        verify by AC_05_Panic_Free_Release_Mode_Latency_and_Memory_Bounds;
        satisfy by NodeArenaASTGraphEngine;
    }
    doc /* UUIDv5: b0dfa9b7-cb12-5c1d-9fed-da91f097631c | Complexity: Class P (O( | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0200 */
    requirement def REQ_0036_Concurrent_String_Interning {
        id = "REQ-0036";
        text = "The compiler core metamodel, lexical analyzer, and abstract syntax tree (AST) graph engine shall intern all identifier strings, type names, docstrings, and qualified paths across compilation units into a thread-safe concurrent string interner pool, mapping each distinct string to a 32-bit scalar";
        attribute uuidv5 : String = "b0dfa9b7-cb12-5c1d-9fed-da91f097631c";
        attribute complexity_class : String = "Class P (O(";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0200";
        attribute ac_01_deduplicated_scalar_handle_assignment_o_equality : String = "Given: Abstract MBSE tokens with repeated identifier strings (Package_0, Classifier_Alpha, Port_1, Flow_A, param_x : Real). When: The compiler lexical scanner interns each token into the global symbol interner pool. Then: Identical strings map to identical 32-bit scalar handles (_a = _b s_a = s_b), handle memory footprint satisfies sizeof() 4 bytes, and identifier comparisons during type checking execute in O(1) constant time via scalar integer equality with zero character iteration. Diagnostic: E0200 emitted if duplicate string values receive distinct handles or scalar equality evaluation.";
        attribute ac_02_zero_copy_allocation_boundary_exception : String = "Given: A memory-mapped source file buffer B_{file} ingested during ParsePass. When: The compiler profiles dynamic heap allocations during lexical analysis and AST construction. Then: Dynamic heap allocations during parsing are strictly confined to the symbol interner pool (HeapAllocs(ParsePass(B_{file})) Storage(StringInternerPool)), while all AST nodes reside in bump arenas or borrow slices from B_{file}. Diagnostic: E0200 emitted if unauthorized heap string allocations occur outside the interner pool during parsing.";
        attribute ac_03_bijective_inversion_thread_safe_bounded_concurrency : String = "Given: Worker threads (T N_{max_workers}}) scheduled on the bounded work-stealing pool processing disjoint AST units with overlapping synthetic identifiers. When: Threads concurrently intern strings and perform bidirectional resolution. Then: For all interned strings, resolve(intern(s)) = s and intern(resolve()) = without data races, deadlocks, or handle divergence across threads (intern_{t_a}(s) = intern_{t_b}(s)). Diagnostic: E0200 emitted if round-trip resolution diverges or concurrent race conditions produce inconsistent handles.";
        attribute ac_04_total_function_sentinel_diagnostics_release_performance : String = "Given: Pathological inputs (invalid UTF-8 bytes, strings exceeding L_{max_str}} = 65536 bytes, or pool capacity exhaustion), and Release mode (--release) benchmark harnesses. When: The compiler interns malformed inputs or executes 10^6 token benchmarks. Then: Pathological inputs yield sentinel _{sym} = 2^{32} - 1 and emit diagnostic E0200 without panicking; release benchmarking achieves throughput exceeding 5 10^6 tokens/s}, reverse resolution latency under 20 ns, execution latency under 25 ms, and greater than 60% AST memory reduction. Diagnostic: E0200 emitted on invalid encoding, length.";
        doc /* \forall \sigma \in \text{SymbolId} \cup \{ \bot_{\text{sym}} \}, \quad \text{sizeof}(\sigma) \le S_{\max} = 4\text{ bytes} */
        constraint def Invariant_REQ_0036_FormalInvariant;
        doc /* \text{intern}(\mathcal{I}, s) = \begin{cases} \mathcal{M}_{\text{fwd}}(s), & \text{if } s \in \text{dom}(\mathcal{M}_{\text{fwd}}) \\ \sigma_{\text{new}}, & \text{if } s \in \mathcal{S} \setminus \text{dom}(\mathcal{M}_{\text{fwd}}) \;\wedge\; |\text{dom}(\mathcal{M}_{\text{fwd}})| \lt S_{\text{max\_count}} \\ \bot_{\text{sym}}, & \text{otherwise} \end{cases} */
        constraint def Invariant_REQ_0036_The_interner_pool_defines_forward_map;
        doc /* \begin{aligned} \forall s \in \text{dom}(\mathcal{M}_{\text{fwd}}), \quad &\text{resolve}(\mathcal{I}, \text{intern}(\mathcal{I}, s)) = s \\ \forall s_1, s_2 \in \text{dom}(\mathcal{M}_{\text{fwd}}), \quad &s_1 = s_2 \iff \text{intern}(\mathcal{I}, s_1) = \text{intern}(\mathcal{I}, s_2) \end{aligned} */
        constraint def Invariant_REQ_0036_Bijective_Round_Trip_and_Scalar_Equality_Invariants;
        doc /* \text{HeapAllocs}(\text{ParsePass}(\mathcal{B}_{\text{file}})) \subseteq \text{Storage}(\text{StringInternerPool}) */
        constraint def Invariant_REQ_0036_Buffer_Allocation_Boundary_Exception_Invariant;
        doc /* \forall t_a, t_b \in \mathcal{T}_{\text{workers}}, \quad \text{intern}_{t_a}(\mathcal{I}, s) = \text{intern}_{t_b}(\mathcal{I}, s) */
        constraint def Invariant_REQ_0036_Concurrent_Determinism_Invariant;
        doc /* (\neg\text{ValidUTF8}(s) \;\vee\; |s| \gt L_{\text{max\_str}} \;\vee\; |\text{dom}(\mathcal{M}_{\text{fwd}})| \ge S_{\text{max\_count}}) \implies \text{intern}(\mathcal{I}, s) = \bot_{\text{sym}} \;\wedge\; \text{Diagnostic}(s, \text{DiagnosticCode::E0200}) \in \mathcal{D} */
        constraint def Invariant_REQ_0036_Diagnostic_Code_Binding_Invariant;
        require Invariant_REQ_0036_FormalInvariant;
        require Invariant_REQ_0036_The_interner_pool_defines_forward_map;
        require Invariant_REQ_0036_Bijective_Round_Trip_and_Scalar_Equality_Invariants;
        require Invariant_REQ_0036_Buffer_Allocation_Boundary_Exception_Invariant;
        require Invariant_REQ_0036_Concurrent_Determinism_Invariant;
        require Invariant_REQ_0036_Diagnostic_Code_Binding_Invariant;
        derived from REQ_0009_Zero_Copy_Bump_Arena_Memory_Management;
        derived from REQ_0012_Bounded_Work_Stealing_Parallelism;
        derived from REQ_0191_Standardized_Compiler_Diagnostic_Taxonomy_Source_Mapping;
        verify by AC_01_Deduplicated_Scalar_Handle_Assignment_O_Equality;
        verify by AC_02_Zero_Copy_Allocation_Boundary_Exception;
        verify by AC_03_Bijective_Inversion_Thread_Safe_Bounded_Concurrency;
        verify by AC_04_Total_Function_Sentinel_Diagnostics_Release_Performance;
        satisfy by NodeArenaASTGraphEngine;
    }
    doc /* UUIDv5: 9df7852d-6325-5ec6-9c14-cec7bc0c51aa | Complexity: Class P (O(1)) | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0200 */
    requirement def REQ_0037_Classifier_Definition {
        id = "REQ-0037";
        text = "The core metamodel shall enforce a strict formal duality and architectural partition between classifier definitions (ClassifierDef) representing reusable types and feature usages (FeatureUsage) representing contextual occurrences within enclosing namespaces. Structural elements in the abstract";
        attribute uuidv5 : String = "9df7852d-6325-5ec6-9c14-cec7bc0c51aa";
        attribute complexity_class : String = "Class P (O(1))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0200";
        attribute ac_01_disjoint_duality_partition_enforcement : String = "Given: An input AST containing structural definitions and usages within Package_0. When: The compiler lowers nodes into core AST arena structures. Then: Every structural node is partitioned into C or U such that C U = ; instantiating a definition directly as a bare usage fails compilation. Diagnostic: E0200 emitted on structural definition-usage partition violation.";
        attribute ac_02_authoritative_classifier_typing_resolution : String = "Given: A classifier definition Classifier_Alpha and feature usages Usage_1 : Classifier_Alpha and Usage_2 : Classifier_Alpha. When: The type checker executes symbol typing pass : U C. Then: 100% of feature usages resolve to the arena handle of Classifier_Alpha with zero diagnostics emitted. Diagnostic: E0200 emitted if a feature usage contains an unresolved classifier reference or lacks a type binding.";
        attribute ac_03_subtyping_conformance_verification : String = "Given: Classifier_Beta specializing Classifier_Alpha (Classifier_Beta :> Classifier_Alpha) and feature usage Usage_Sub : Classifier_Beta. When: Usage_Sub is assigned to an interface slot, parameter, or port expecting Classifier_Alpha. Then: The type checker evaluates ConformsTo(Usage_Sub}, Classifier_Alpha}) = true because (Usage_Sub}) Classifier_Alpha}, accepting the binding. Diagnostic: E0200 emitted if an incompatible classifier type Classifier_Gamma (Classifier_Gamma} Classifier_Alpha}) is bound to the target slot.";
        attribute ac_04_monotonic_covariance_in_multiplicity_narrowing : String = "Given: Classifier_Alpha declaring feature Feature_A with multiplicity interval [0, 10]. When: A specializing classifier or usage redefines Feature_A with narrowed interval [1, 5]. Then: The compiler verifies [1, 5] [0, 10] and accepts the redefinition; redefining with widened interval [0, 20] fails compilation. Diagnostic: E0200 emitted on non-monotonic multiplicity interval widening.";
        doc /* \mathcal{V}_{\text{structural}} = \mathcal{C} \cup \mathcal{U}, \quad \mathcal{C} \cap \mathcal{U} = \emptyset */
        constraint def Invariant_REQ_0037_FormalInvariant;
        doc /* \tau : \mathcal{U} \longrightarrow \mathcal{C} */
        constraint def Invariant_REQ_0037_The_typing_relation_is_a_total_function_mapping_every_valid_feature_usage_to_an_authoritative_classifier_definition;
        doc /* \begin{aligned} \forall C \in \mathcal{C}, &\quad C \le C \\ \forall C_1, C_2, C_3 \in \mathcal{C}, &\quad (C_1 \le C_2 \wedge C_2 \le C_3) \implies C_1 \le C_3 \\ \forall C_1, C_2 \in \mathcal{C}, &\quad (C_1 \le C_2 \wedge C_2 \le C_1) \implies C_1 = C_2 \\ \text{ConformsTo}(u, C_{\text{target}}) &\iff \tau(u) \le C_{\text{target}} \end{aligned} */
        constraint def Invariant_REQ_0037_Subtyping_Partial_Order_and_Conformance;
        doc /* \begin{aligned} \text{ValidRedef}(u, u_{\text{base}}) &\iff \Phi_{\text{type}} \wedge \Phi_{\text{mult}} \wedge \Phi_{\text{env}} \wedge \Phi_{\text{dir}} \\ \Phi_{\text{type}} &\iff \tau(u) \le \tau(u_{\text{base}}) \\ \Phi_{\text{mult}} &\iff (l_{\text{base}} \le l_u \wedge u_u \le u_{\text{base}}) \\ \Phi_{\text{env}} &\iff (v_{\text{default}}(u) \ne \bot \implies v_{\text{default}}(u) \in \text{Domain}(\tau(u_{\text{base}}))) \\ \Phi_{\text{dir}} &\iff (\delta(u_{\text{base}}) \ne \text{None} \implies \delta(u) = \delta(u_{\text{base}})) \end{aligned} */
        constraint def Invariant_REQ_0037_Monotonic_Covariance_Redefinition_Invariant;
        doc /* \begin{aligned} (\tau(u) = \bot) \vee \neg \text{ConformsTo}(u, C_{\text{target}}) \vee \neg \text{ValidRedef}(u, u_{\text{base}}) \implies \text{Diagnostic}(u, \text{DiagnosticCode::E0200}) \in \mathcal{D}_{\text{sink}} \end{aligned} */
        constraint def Invariant_REQ_0037_Diagnostic_Code_Binding_Invariant;
        require Invariant_REQ_0037_FormalInvariant;
        require Invariant_REQ_0037_The_typing_relation_is_a_total_function_mapping_every_valid_feature_usage_to_an_authoritative_classifier_definition;
        require Invariant_REQ_0037_Subtyping_Partial_Order_and_Conformance;
        require Invariant_REQ_0037_Monotonic_Covariance_Redefinition_Invariant;
        require Invariant_REQ_0037_Diagnostic_Code_Binding_Invariant;
        derived from REQ_0191_Standardized_Compiler_Diagnostic_Taxonomy_Source_Mapping;
        verify by AC_01_Disjoint_Duality_Partition_Enforcement;
        verify by AC_02_Authoritative_Classifier_Typing_Resolution;
        verify by AC_03_Subtyping_Conformance_Verification;
        verify by AC_04_Monotonic_Covariance_in_Multiplicity_Narrowing;
        satisfy by NodeArenaASTGraphEngine;
    }
    doc /* UUIDv5: 2128f8b9-0c5f-56ea-99be-e1b350e936dc | Complexity: Class P (O(1)) | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0199, E0201 */
    requirement def REQ_0038_Strongly_Typed_Port_Directions {
        id = "REQ-0038";
        text = "The Core Metamodel, Node Arena & AST Graph Engine shall enforce strongly typed port directionality (in, out, inout) on all port definitions and usages, verifying directional compatibility across topological connection bindings. Port definitions shall declare an immutable base direction d_{decl} \in";
        attribute uuidv5 : String = "2128f8b9-0c5f-56ea-99be-e1b350e936dc";
        attribute complexity_class : String = "Class P (O(1))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0199, E0201";
        attribute ac_01_peer_to_peer_anti_parallel_direction_verification : String = "Given: Sibling instances Part_0 and Part_1 declaring peer connections: (Out, In), (In, Out), and (InOut, InOut). When: The AST type checking pass validates peer connection directionality. Then: All three connection edges evaluate to valid, node handles are marked validated, and zero diagnostics are emitted. Diagnostic: E0201 on unexpected validation rejection.";
        attribute ac_02_hierarchical_delegation_flow_preservation : String = "Given: Component Component_Alpha with boundary ports connected to child Part_0 ports: outer In to inner In, inner Out to outer Out, and outer InOut to inner InOut. When: The type checking pass evaluates delegation connections across containment boundaries. Then: All delegation connections satisfy P_{outer}.d_{eff} = P_{inner}.d_{eff} and are accepted with zero diagnostics. Diagnostic: E0201 on valid delegation rejection.";
        attribute ac_03_port_conjugation_involution_and_effective_direction_resolution : String = "Given: Port usage Port_2 with base direction In and conjugation flag c = true, connected to peer Port_1 with base direction In and c = false. When: Resolving effective directions and checking peer connection validity. Then: Resolves d_{eff}(Port_2}) = _{conj}(In) = Out, satisfies In _{peer} Out, and accepts the connection with zero diagnostics. Diagnostic: E0201 on conjugation involution evaluation failure.";
        attribute ac_04_peer_and_delegation_direction_mismatch_rejection : String = "Given: Invalid peer pairings ((In, In), (Out, Out), (In, InOut)) and inverted delegation pairings (outer In to inner Out, outer Out to inner In). When: Type checking processes connection topology. Then: Rejects each invalid edge, logs structured E0201 diagnostics with byte spans and port handles, and continues verification via synchronization recovery. Diagnostic: E0201 recorded in diagnostic sink D_{sink}.";
        doc /* \tau_{\text{conj}}(d) = \begin{cases} \text{Out}, & \text{if } d = \text{In} \\ \text{In}, & \text{if } d = \text{Out} \\ \text{InOut}, & \text{if } d = \text{InOut} \end{cases}, \qquad \forall d \in \mathcal{D}, \quad \tau_{\text{conj}}(\tau_{\text{conj}}(d)) \equiv d */
        constraint def Invariant_REQ_0038_FormalInvariant;
        doc /* d_{\text{eff}}(P) = \begin{cases} \tau_{\text{conj}}(P.d_{\text{decl}}), & \text{if } P.c = \text{true} \\ P.d_{\text{decl}}, & \text{if } P.c = \text{false} \end{cases} */
        constraint def Invariant_REQ_0038_For_port_usage;
        doc /* \begin{aligned} P_{\text{src}} \sim_{\text{peer}} P_{\text{tgt}} &\iff (P_{\text{src}}.d_{\text{eff}}, P_{\text{tgt}}.d_{\text{eff}}) \in \{ (\text{Out}, \text{In}), (\text{In}, \text{Out}), (\text{InOut}, \text{InOut}) \} \\ P_{\text{outer}} \sim_{\text{delegation}} P_{\text{inner}} &\iff P_{\text{outer}}.d_{\text{eff}} = P_{\text{inner}}.d_{\text{eff}} \end{aligned} */
        constraint def Invariant_REQ_0038_FormalInvariant_2;
        doc /* \text{ValidDirection}(C) \iff \begin{cases} P_{\text{src}} \sim_{\text{peer}} P_{\text{tgt}}, & \text{if } \kappa_{\text{conn}}(C) = \text{Assembly} \\ P_{\text{outer}} \sim_{\text{delegation}} P_{\text{inner}}, & \text{if } \kappa_{\text{conn}}(C) = \text{Delegation} \end{cases} */
        constraint def Invariant_REQ_0038_Directional_validity_and_diagnostic_bindings_per_REQ_0191;
        doc /* \begin{aligned} \neg \text{ValidDirection}(C) &\implies \exists D \in \mathcal{D}_{\text{sink}} : \text{code}(D) = \text{DiagnosticCode::E0201} \\ |\mathcal{D}_{\text{sink}}| \ge C_{\text{max\_errors}} &\implies \exists D_{\text{term}} \in \mathcal{D}_{\text{sink}} : \text{code}(D_{\text{term}}) = \text{DiagnosticCode::E0199} \end{aligned} */
        constraint def Invariant_REQ_0038_FormalInvariant_3;
        doc /* \forall u \in \mathcal{V}_{\text{port}} \cup \mathcal{E}_{\text{conn}}, \quad \text{UUID}(u) = \text{UUIDv5}(\text{OID}_{\text{DEAP}}, \text{FQN}(u)), \quad \text{Handle}(u) = \langle \text{ArenaId}, \text{Offset}_u \rangle */
        constraint def Invariant_REQ_0038_Deterministic_UUIDv5_and_Arena_Node_Addressing;
        require Invariant_REQ_0038_FormalInvariant;
        require Invariant_REQ_0038_For_port_usage;
        require Invariant_REQ_0038_FormalInvariant_2;
        require Invariant_REQ_0038_Directional_validity_and_diagnostic_bindings_per_REQ_0191;
        require Invariant_REQ_0038_FormalInvariant_3;
        require Invariant_REQ_0038_Deterministic_UUIDv5_and_Arena_Node_Addressing;
        derived from REQ_0191_Standardized_Compiler_Diagnostic_Taxonomy_Source_Mapping;
        verify by AC_01_Peer_to_Peer_Anti_Parallel_Direction_Verification;
        verify by AC_02_Hierarchical_Delegation_Flow_Preservation;
        verify by AC_03_Port_Conjugation_Involution_and_Effective_Direction_Resolution;
        verify by AC_04_Peer_and_Delegation_Direction_Mismatch_Rejection;
        satisfy by NodeArenaASTGraphEngine;
    }
    doc /* UUIDv5: 202d2e0f-6949-5794-95cf-e3a582f03c7d | Complexity: Class P (O(H)) | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0199, E0202 */
    requirement def REQ_0039_Flow_Payload_Binding {
        id = "REQ-0039";
        text = "The Core Metamodel, Node Arena & AST Graph Engine shall bind item flow payloads to authoritative classifier definitions and validate stream transmission rate multiplicities across connectors and channels. Every item flow (ItemFlow or FlowUsage) connecting source port P_{src} and target port P_{tgt}";
        attribute uuidv5 : String = "202d2e0f-6949-5794-95cf-e3a582f03c7d";
        attribute complexity_class : String = "Class P (O(H))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0199, E0202";
        attribute ac_01_valid_payload_type_binding_and_covariance : String = "Given: Ports Port_1 : Classifier_Beta and Port_2 : Classifier_Alpha with Classifier_Beta :> Classifier_Alpha, connected by flow Flow_A with payload Classifier_Beta and rate [10.0, 50.0] Hz. When: Flow type checking executes. Then: The engine validates _F :> (P_{src}) and _F :> (P_{tgt}), passing with zero diagnostics. Diagnostic: E0202 on covariance failure.";
        attribute ac_02_incompatible_payload_type_rejection_without_adapter : String = "Given: Disjoint ports Port_1 : Classifier_Alpha and Port_2 : Classifier_Gamma connected by flow Flow_Err with payload Classifier_Gamma without an adapter (A_F = ). When: Flow type checking executes. Then: Rejects flow, binds sentinel _{flow}, and emits structured diagnostic E0202 with source byte span coordinates. Diagnostic: Emits E0202 (PayloadTypeIncompatibility).";
        attribute ac_03_adapter_mediated_payload_conformance : String = "Given: Ports Port_1 : Classifier_Alpha and Port_2 : Classifier_Gamma connected by flow Flow_Adapt with adapter Adapter_AlphaToGamma (InType = Classifier_Alpha}, OutType = Classifier_Gamma}). When: Evaluating adapter payload compatibility. Then: Confirms (P_{src}) InType(A_F) and OutType(A_F) (P_{tgt}), passing with zero diagnostics. Diagnostic: E0202 on adapter mismatch.";
        attribute ac_04_rate_multiplicity_well_formedness_and_channel_capacity_bounds : String = "Given: Channel Channel_0 (C_{channel} = 1000 bytes/sec), flow Flow_Err1 with rate [100.0, 10.0] Hz, and flow Flow_Err2 with peak throughput 1280 bytes/sec. When: Validating rate multiplicity and channel capacity constraints. Then: Rejects Flow_Err1 for inverted rate bounds and Flow_Err2 for capacity overflow, emitting structured diagnostics. Diagnostic: Emits E0202 (MalformedRateMultiplicityInterval or ChannelCapacityOverflow).";
        doc /* F = \langle \text{id}_F, P_{\text{src}}, P_{\text{tgt}}, \tau_F, R_{\text{flow}}, C_{\text{channel}}, \mathcal{A}_F, \text{span}_F \rangle */
        constraint def Invariant_REQ_0039_FormalInvariant;
        doc /* \Phi_{\text{type}}(F) \iff \begin{cases} (\tau_F :> \tau(P_{\text{src}})) \;\wedge\; (\tau_F :> \tau(P_{\text{tgt}})), & \text{if } \mathcal{A}_F = \bot \\ (\tau(P_{\text{src}}) \le \text{InType}(\mathcal{A}_F)) \;\wedge\; (\text{OutType}(\mathcal{A}_F) \le \tau(P_{\text{tgt}})), & \text{if } \mathcal{A}_F \ne \bot \end{cases} */
        constraint def Invariant_REQ_0039_FormalInvariant_2;
        doc /* \begin{aligned} \Phi_{\text{rate}}(F) &\iff (0 \le r_{\min}) \;\wedge\; (r_{\min} \le r_{\max}) \\ \Phi_{\text{cap}}(F) &\iff r_{\max} \cdot \text{sizeof}(\tau_F) \le C_{\text{channel}} \\ \Phi_{\text{mux}}(\kappa) &\iff \sum_{F \in \mathcal{E}_{\text{flow}}(\kappa)} r_{\max}(F) \cdot \text{sizeof}(\tau_F) \le C_{\text{channel}}(\kappa) \end{aligned} */
        constraint def Invariant_REQ_0039_Rate_multiplicity_interval_and_channel_throughput_predicates_satisfy;
        doc /* \begin{aligned} \neg \Phi_{\text{type}}(F) \;\vee\; \neg \Phi_{\text{rate}}(F) \;\vee\; \neg \Phi_{\text{cap}}(F) &\implies \text{DiagnosticCode::E0202} \in \mathcal{D} \\ \neg \Phi_{\text{mux}}(\kappa) &\implies \text{DiagnosticCode::E0202} \in \mathcal{D} \\ |\mathcal{D}| \ge C_{\text{max\_errors}} &\implies \text{DiagnosticCode::E0199} \in \mathcal{D} \\ \forall F \in \mathcal{E}_{\text{flow}}, \quad \text{UUID}(F) &= \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(F)) \end{aligned} */
        constraint def Invariant_REQ_0039_Diagnostic_code_binding_per_REQ_0191_and_deterministic_UUIDv5_anchoring_satisfy;
        require Invariant_REQ_0039_FormalInvariant;
        require Invariant_REQ_0039_FormalInvariant_2;
        require Invariant_REQ_0039_Rate_multiplicity_interval_and_channel_throughput_predicates_satisfy;
        require Invariant_REQ_0039_Diagnostic_code_binding_per_REQ_0191_and_deterministic_UUIDv5_anchoring_satisfy;
        derived from REQ_0191_Standardized_Compiler_Diagnostic_Taxonomy_Source_Mapping;
        verify by AC_01_Valid_Payload_Type_Binding_and_Covariance;
        verify by AC_02_Incompatible_Payload_Type_Rejection_Without_Adapter;
        verify by AC_03_Adapter_Mediated_Payload_Conformance;
        verify by AC_04_Rate_Multiplicity_Well_Formedness_and_Channel_Capacity_Bounds;
        satisfy by NodeArenaASTGraphEngine;
    }
    doc /* UUIDv5: ae00cb7d-b97e-5771-a84b-7d60fce442fa | Complexity: Class P (O(1)) | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0200 */
    requirement def REQ_0040_Two_Pass_Symbol_Hoisting {
        id = "REQ-0040";
        text = "The core abstract syntax tree (AST) engine shall execute a deterministic two-pass symbol resolution pipeline across workspace compilation units, decoupling declaration order from reference order.";
        attribute uuidv5 : String = "ae00cb7d-b97e-5771-a84b-7d60fce442fa";
        attribute complexity_class : String = "Class P (O(1))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0200";
        attribute ac_01_mutual_recursion_forward_reference_resolution : String = "Given: Sibling packages Package_0 and Package_1 where Classifier_Alpha in Package_0 references Package_1::Classifier_Beta, and Classifier_Beta in Package_1 references Package_0::Classifier_Alpha. When: Pass 1 hoists symbols and Pass 2 resolves references. Then: Both cross-references resolve to exact 32-bit arena node handles in with zero diagnostics emitted. Diagnostic: E0200 on resolution failure.";
        attribute ac_02_declaration_order_permutation_invariance : String = "Given: An AST model with 500 definitions and 1,000 cross-references permuted into 10 distinct declaration sequences _1, , _{10}. When: The two-pass symbol pipeline executes on each permutation. Then: All 10 runs produce bitwise identical symbol tables and resolved AST graphs with 0-byte diff churn. Diagnostic: E0200 if resolved handles depend on declaration order.";
        attribute ac_03_duplicate_symbol_declaration_rejection : String = "Given: An AST declaring two classifier definitions with identical name Classifier_Alpha in scope Package_0. When: Pass 1 hoists definitions in Package_0. Then: Pass 1 rejects the duplicate declaration, retains the primary definition handle in , records exact source coordinates, and continues AST traversal. Diagnostic: E0200 on duplicate symbol declaration in the same scope.";
        attribute ac_04_unresolved_and_ambiguous_symbol_diagnostics : String = "Given: An AST containing reference r_1 targeting missing symbol Classifier_Missing and reference r_2 matching multiple candidate definitions across imported scopes. When: Pass 2 resolves references against symbol table . Then: Pass 2 binds sentinel _{sym} to each faulted reference, accumulates structured diagnostic records, and completes traversal. Diagnostic: E0200 on unresolvable or ambiguous symbol references.";
        doc /* \Gamma = \text{Pass}_1(\mathcal{A}) = \bigcup_{n \in \mathcal{V}_{\text{defs}}} \{ \text{FQN}(n) \mapsto \text{Handle}(n) \} */
        constraint def Invariant_REQ_0040_Pass_1_constructs_hierarchical_symbol_environment;
        doc /* \forall n_1, n_2 \in \mathcal{V}_{\text{defs}}, \quad (\text{Scope}(n_1) = \text{Scope}(n_2) \wedge \text{Name}(n_1) = \text{Name}(n_2)) \implies n_1 = n_2 */
        constraint def Invariant_REQ_0040_Scope_Uniqueness_Invariant;
        doc /* \mathcal{A}_{\text{resolved}} = \text{Pass}_2(\mathcal{A}, \Gamma) \iff \forall r \in \mathcal{V}_{\text{refs}}, \quad \text{BindTarget}(r, \rho(r, \text{Scope}(r), \Gamma)) */
        constraint def Invariant_REQ_0040_Pass_2_binds_references_via_lookup_function;
        doc /* \rho(r, s, \Gamma) = \begin{cases} \mu_s(\text{Name}(r)), & \text{if } \mu_s(\text{Name}(r)) \ne \bot \\ \rho(r, p, \Gamma), & \text{if } \mu_s(\text{Name}(r)) = \bot \wedge (s \prec p) \\ \mu_{\text{global}}(\text{FQN}(r)), & \text{if } s = \sigma_0 \wedge \text{IsQualified}(r) \wedge \mu_{\text{global}}(\text{FQN}(r)) \ne \bot \\ \bot_{\text{sym}}, & \text{otherwise} \end{cases} */
        constraint def Invariant_REQ_0040_FormalInvariant;
        doc /* \begin{aligned} \forall r \in \mathcal{V}_{\text{refs}}, \quad \text{Pos}(r) \lt \text{Pos}(d) &\implies \rho(r, \text{Scope}(r), \Gamma) = d \\ \text{Pass}_1(\pi(\mathcal{A})) = \Gamma, &\quad \text{Pass}_2(\pi(\mathcal{A}), \Gamma) = \mathcal{A}_{\text{resolved}} \\ \forall n \in \mathcal{V}_{\text{defs}}, \quad \text{UUID}(n) &= \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(n)) \\ \exists r \in \mathcal{V}_{\text{refs}} : \rho(r, \text{Scope}(r), \Gamma) = \bot_{\text{sym}} &\implies \text{Diagnostic}(r, \text{E0200}) \in \mathcal{D}_{\text{sink}} \end{aligned} */
        constraint def Invariant_REQ_0040_Mutual_Recursion_Permutation_and_Binding_Invariants;
        require Invariant_REQ_0040_Pass_1_constructs_hierarchical_symbol_environment;
        require Invariant_REQ_0040_Scope_Uniqueness_Invariant;
        require Invariant_REQ_0040_Pass_2_binds_references_via_lookup_function;
        require Invariant_REQ_0040_FormalInvariant;
        require Invariant_REQ_0040_Mutual_Recursion_Permutation_and_Binding_Invariants;
        verify by AC_01_Mutual_Recursion_Forward_Reference_Resolution;
        verify by AC_02_Declaration_Order_Permutation_Invariance;
        verify by AC_03_Duplicate_Symbol_Declaration_Rejection;
        verify by AC_04_Unresolved_and_Ambiguous_Symbol_Diagnostics;
        satisfy by NodeArenaASTGraphEngine;
    }
    doc /* UUIDv5: a5e64058-2009-5e05-bfa7-a33611ac63fe | Complexity: Class P (O(1)) | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100, E0199, E0200 */
    requirement def REQ_0041_Lexical_Scope_Lookup {
        id = "REQ-0041";
        text = "The Core Metamodel, Node Arena & AST Graph Engine shall implement hierarchical lexical scope lookup and path evaluation using double-colon scoping (::) to resolve identifier references across nested lexical scopes, package boundaries, and imported namespaces.";
        attribute uuidv5 : String = "a5e64058-2009-5e05-bfa7-a33611ac63fe";
        attribute complexity_class : String = "Class P (O(1))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100, E0199, E0200";
        attribute ac_01_lexical_scope_shadowing : String = "Given: AST hierarchy Package_Root::Package_0::Classifier_Alpha with param_x : Real in Package_0 and local param_x : Real in Classifier_Alpha. When: Resolving unqualified param_x within Classifier_Alpha and qualified ::Package_Root::Package_0::param_x. Then: Unqualified param_x resolves to Classifier_Alpha local handle; qualified path resolves to Package_0 handle with zero diagnostics. Diagnostic: E0200 on shadow resolution failure or qualified path fault.";
        attribute ac_02_absolute_vs_relative_qualified_path_resolution : String = "Given: Sibling scopes Package_Root::Package_0::Classifier_Alpha and Package_Root::Package_1::Classifier_Beta. When: Classifier_Beta resolves absolute path ::Package_Root::Package_0::Classifier_Alpha and Package_Root resolves relative path Package_0::Classifier_Alpha. Then: Both paths resolve to Classifier_Alpha 32-bit arena handle with 0-byte diff churn. Diagnostic: E0200 on qualified path traversal fault.";
        attribute ac_03_package_visibility_boundary_enforcement : String = "Given: Package Package_0 declaring private member Feature_A and public member Feature_B. When: Scope Package_1 resolves Package_0::Feature_A and Package_0::Feature_B. Then: Resolution of Feature_B succeeds; resolution of Feature_A is rejected with diagnostic E0200 with source spans. Diagnostic: E0200 on inaccessible private member access across packages.";
        attribute ac_04_ambiguous_import_conflict_detection : String = "Given: A scope importing Package_0 and Package_1, both exporting identifier Port_1 without local shadowing. When: An unqualified reference to Port_1 is evaluated. Then: The resolver rejects the reference as ambiguous, emits diagnostic E0200 citing candidate locations, and binds sentinel _{node}. Diagnostic: E0200 on colliding imported identifiers.";
        doc /* \text{IsVisible}(e, S) \iff (e.\text{vis}_e = \text{Public}) \vee (e.\text{vis}_e = \text{Private} \wedge \text{Pkg}(e) = \text{Pkg}(S)) \vee (e.\text{vis}_e = \text{Protected} \wedge S \le \text{Scope}(e)) */
        constraint def Invariant_REQ_0041_Visibility_predicate;
        doc /* \text{Find}(\sigma, S, S_{\text{caller}}) = \begin{cases} e.h_e, & \text{if } \exists e \in \mathcal{M}_S : e.\sigma_e = \sigma \wedge \text{IsVisible}(e, S_{\text{caller}}) \\ \text{Find}(\sigma, S.\text{parent}, S_{\text{caller}}), & \text{if } (\forall e \in \mathcal{M}_S, e.\sigma_e \ne \sigma) \wedge S.\text{parent} \ne \bot \\ \bot_{\text{node}}, & \text{otherwise} \end{cases} */
        constraint def Invariant_REQ_0041_Unqualified_lookup;
        doc /* \text{FindQualified}(\pi, \alpha, S) = \text{Traverse}(\text{BaseScope}(p_1, \alpha, S), 2, k, \pi, S) */
        constraint def Invariant_REQ_0041_Qualified_path_resolution;
        doc /* \begin{aligned} \text{BaseScope}(p_1, \alpha, S) &= \begin{cases} \text{Member}(S_{\text{root}}, p_1), & \text{if } \alpha = \text{Absolute} \\ \text{ScopeOf}(\text{Find}(p_1, S, S)), & \text{if } \alpha = \text{Relative} \end{cases} \\ \text{Traverse}(S, j, k, \pi, S_{\text{caller}}) &= \begin{cases} S, & \text{if } j \gt k \wedge S \ne \bot_{\text{node}} \\ \bot_{\text{node}}, & \text{if } S = \bot_{\text{node}} \\ \text{Traverse}(\text{Member}(S, p_j), j + 1, k, \pi, S_{\text{caller}}), & \text{otherwise} \end{cases} \end{aligned} */
        constraint def Invariant_REQ_0041_where;
        doc /* \begin{aligned} (e_1 \in \mathcal{M}_{S_{\text{inner}}} \wedge e_2 \in \mathcal{M}_{S_{\text{outer}}} \wedge S_{\text{inner}} \prec^+ S_{\text{outer}} \wedge e_1.\sigma_e = e_2.\sigma_e) &\implies \text{Find}(e_1.\sigma_e, S_{\text{inner}}, S_{\text{inner}}) = e_1.h_e \\ \forall S \in \mathcal{S}, \quad \text{Depth}(S) \le D_{\max} = 16, &\quad \Delta_S = \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(S)) \\ \text{Find}(\sigma, S, S) = \bot_{\text{node}} \vee \text{FindQualified}(\pi, \alpha, S) = \bot_{\text{node}} &\implies \text{Diagnostic}(\text{E0200}) \in \mathcal{D}_{\text{sink}} \\ |\mathcal{D}_{\text{sink}}| \ge C_{\text{max\_errors}} &\implies \text{Diagnostic}(\text{E0199}) \in \mathcal{D}_{\text{sink}} \end{aligned} */
        constraint def Invariant_REQ_0041_Shadowing_bounds_and_diagnostic_invariants;
        require Invariant_REQ_0041_Visibility_predicate;
        require Invariant_REQ_0041_Unqualified_lookup;
        require Invariant_REQ_0041_Qualified_path_resolution;
        require Invariant_REQ_0041_where;
        require Invariant_REQ_0041_Shadowing_bounds_and_diagnostic_invariants;
        derived from REQ_0191_Standardized_Compiler_Diagnostic_Taxonomy_Source_Mapping;
        verify by AC_01_Lexical_Scope_Shadowing;
        verify by AC_02_Absolute_vs_Relative_Qualified_Path_Resolution;
        verify by AC_03_Package_Visibility_Boundary_Enforcement;
        verify by AC_04_Ambiguous_Import_Conflict_Detection;
        satisfy by NodeArenaASTGraphEngine;
    }
    doc /* UUIDv5: b2716f9c-c48b-5897-8021-7dab9787528c | Complexity: Class P (O(1)) | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0199, E0203 */
    requirement def REQ_0042_Circular_Dependency {
        id = "REQ-0042";
        text = "The core abstract syntax tree (AST) engine shall implement a deterministic cycle detection engine utilizing Tarjan's strongly connected components (SCC) algorithm to identify, isolate, and reject circular dependencies across metamodel relationship graphs during compile-time semantic analysis.";
        attribute uuidv5 : String = "b2716f9c-c48b-5897-8021-7dab9787528c";
        attribute complexity_class : String = "Class P (O(1))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0199, E0203";
        attribute ac_01_direct_self_specialization_cycle_detection : String = "Given: Classifier Classifier_Alpha declared with self-specialization Classifier_Alpha :> Classifier_Alpha. When: Cycle detection executes on specialization graph G_{inherit}. Then: Tarjan SCC analysis identifies a self-loop of size 1, extracts traceback Classifier_Alpha}, Classifier_Alpha} , and emits diagnostic E0203. Diagnostic: E0203 emitted on direct self-specialization cycle.";
        attribute ac_02_mutual_and_multi_node_inheritance_cycle_rejection : String = "Given: An AST model declaring mutual specialization (Classifier_Alpha :> Classifier_Beta, Classifier_Beta :> Classifier_Alpha) or a 4-node circular chain (A :> B :> C :> D :> A). When: Cycle detection evaluates specialization graph G_{inherit}. Then: The engine detects the multi-node SCC, derives deterministic cycle path P_{cycle} ordered by minimum handle, and rejects compilation. Diagnostic: E0203 emitted on cyclic inheritance chains.";
        attribute ac_03_structural_composition_and_feature_redefinition_cycle_rejection : String = "Given: An AST model where Part_0 contains Part_1 and Part_1 contains Part_0, or where feature Usage_1 redefines Usage_2 (:>>) and Usage_2 redefines Usage_1. When: Cycle detection evaluates composition graph G_{comp} and redefinition graph G_{redef}. Then: The engine detects the cyclic dependency in each respective graph, rejects recursive instantiation, and emits diagnostic E0203. Diagnostic: E0203 emitted on structural containment or feature redefinition cycles.";
        attribute ac_04_synchronization_token_error_recovery_and_overlay_pruning : String = "Given: An AST model containing two disjoint inheritance cycles alongside valid acyclic subgraphs. When: Cycle detection executes with synchronization token error recovery enabled. Then: The engine records distinct E0203 diagnostics for both cycles in D_{sink}, prunes closing back-edges in E_{overlay} substituting sentinel _{cycle}, and completes semantic validation on remaining valid subgraphs. Diagnostic: E0203 recorded per cycle; E0199 emitted if total diagnostics reach C_{max_errors}}.";
        doc /* \begin{aligned} G_{\text{inherit}} &= (\mathcal{V}_{\text{class}}, \mathcal{E}_{\text{inherit}}) \quad \text{where } (u, v) \in \mathcal{E}_{\text{inherit}} \iff \text{Specializes}(u, v) \\ G_{\text{comp}} &= (\mathcal{V}_{\text{elem}}, \mathcal{E}_{\text{comp}}) \quad \text{where } (u, v) \in \mathcal{E}_{\text{comp}} \iff \text{Contains}(u, v) \\ G_{\text{redef}} &= (\mathcal{V}_{\text{feat}}, \mathcal{E}_{\text{redef}}) \quad \text{where } (u, v) \in \mathcal{E}_{\text{redef}} \iff \text{Redefines}(u, v) \end{aligned} */
        constraint def Invariant_REQ_0042_FormalInvariant;
        doc /* \text{IsAcyclic}(G) \iff \forall v \in \mathcal{V}, \quad v \notin \text{Descendants}(v) \iff \forall C \in \text{TarjanSCC}(G), \quad (|C| = 1 \wedge \forall v \in C, \, (v, v) \notin \mathcal{E}) */
        constraint def Invariant_REQ_0042_Universal_Graph_Acyclicity_and_Tarjan_Singleton_Invariants;
        doc /* \text{lowlink}(u) = \min \left( \{ \text{dfn}(u) \} \cup \{ \text{lowlink}(w) \mid (u, w) \in \mathcal{E}_{\text{tree}} \} \cup \{ \text{dfn}(w) \mid (u, w) \in \mathcal{E} \setminus \mathcal{E}_{\text{tree}}, \, w \in \mathcal{S}_{\text{stack}} \} \right) */
        constraint def Invariant_REQ_0042_Tarjan_Lowlink_Recurrence_and_Root_Condition;
        doc /* \text{IsSCCRoot}(u) \iff \text{lowlink}(u) = \text{dfn}(u) */
        constraint def Invariant_REQ_0042_FormalInvariant_2;
        doc /* \begin{aligned} \exists C \in \text{TarjanSCC}(G) : (|C| \gt 1 \vee \exists v \in C, \, (v, v) \in \mathcal{E}) &\implies \text{Diagnostic}(\mathcal{P}_{\text{cycle}}, \text{E0203}) \in \mathcal{D}_{\text{sink}} \\ \mathcal{E}_{\text{overlay}} = \mathcal{E} \setminus \{ (v_k, v_1) \} &\implies \text{IsAcyclic}((\mathcal{V}, \mathcal{E}_{\text{overlay}})) \equiv \text{true} \\ \forall u \in \mathcal{V}, \quad \text{UUID}(u) &= \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(u)) \end{aligned} */
        constraint def Invariant_REQ_0042_Deterministic_Cycle_Traceback_and_Diagnostic_Invariants;
        require Invariant_REQ_0042_FormalInvariant;
        require Invariant_REQ_0042_Universal_Graph_Acyclicity_and_Tarjan_Singleton_Invariants;
        require Invariant_REQ_0042_Tarjan_Lowlink_Recurrence_and_Root_Condition;
        require Invariant_REQ_0042_FormalInvariant_2;
        require Invariant_REQ_0042_Deterministic_Cycle_Traceback_and_Diagnostic_Invariants;
        derived from REQ_0191_Standardized_Compiler_Diagnostic_Taxonomy_Source_Mapping;
        verify by AC_01_Direct_Self_Specialization_Cycle_Detection;
        verify by AC_02_Mutual_and_Multi_Node_Inheritance_Cycle_Rejection;
        verify by AC_03_Structural_Composition_and_Feature_Redefinition_Cycle_Rejection;
        verify by AC_04_Synchronization_Token_Error_Recovery_and_Overlay_Pruning;
        satisfy by NodeArenaASTGraphEngine;
    }
    doc /* UUIDv5: a1b617cc-9c61-5bdd-a43a-3d53ae820fc2 | Complexity: Class P (O(L)) | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0199, E0200 */
    requirement def REQ_0043_Multi_File_Module_Ingestion {
        id = "REQ-0043";
        text = "The compiler shall support multi-file models partitioned across hierarchical directories, deterministically mapping relative filesystem paths to nested package namespaces within the abstract syntax tree (AST). For every source file ingested beneath the workspace schema root directory (R_{schema}),";
        attribute uuidv5 : String = "a1b617cc-9c61-5bdd-a43a-3d53ae820fc2";
        attribute complexity_class : String = "Class P (O(L))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0199, E0200";
        attribute ac_01_canonical_directory_to_namespace_hierarchy_resolution : String = "Given: A compilation unit at relative path schema/package_0/module_alpha/unit_beta/model_alpha.sysml. When: The compiler executes path normalization and directory-to-namespace mapping. Then: The file binds deterministically to leaf package Package0::ModuleAlpha::UnitBeta, and all top-level AST declarations are registered as owned members of that package node. Diagnostic: E0200 emitted on namespace resolution failure or invalid identifier mapping.";
        attribute ac_02_idempotent_intermediate_package_synthesis : String = "Given: Sibling source files sharing common directory prefixes (schema/package_0/module_alpha/unit_0/part_a.sysml and schema/package_0/module_alpha/unit_1/part_b.sysml). When: The compiler constructs package hierarchy nodes in the AST. Then: Intermediate package container nodes for Package0 and Package0::ModuleAlpha are synthesized exactly once, reused across sibling files, and assigned deterministic RFC 4122 UUIDv5 identifiers. Diagnostic: E0200 emitted if duplicate package handles are allocated or parent-child linkages diverge.";
        attribute ac_03_explicit_package_declaration_reconciliation_and_conflict_rejection : String = "Given: A source file at schema/package_0/module_alpha/unit_0/part_a.sysml containing explicit declaration package ConflictingRoot::ModuleBeta;. When: Semantic analysis compares the declared package with directory-derived namespace Package0::ModuleAlpha::Unit0. Then: The compiler rejects the conflicting declaration, emits diagnostic E0200, and executes synchronization token error recovery to resume parsing at the next file boundary. Diagnostic: E0200 emitted on namespace path discrepancy.";
        attribute ac_04_segment_normalization_collision_detection_and_depth_bounds : String = "Given: Sibling directories normalizing to identical PascalCase identifiers (e.g., module_alpha and module-alpha), or a path with nesting depth k = 17 exceeding D_{ = 16. When: Directory segment normalization and depth validation execute. Then: The compiler rejects the offending units, emits diagnostic E0200 with the source coordinate, and maintains stable compiler arena state. Diagnostic: E0200 emitted on segment collision or nesting depth overflow (k 16).";
        doc /* P_{\text{rel}}(f) = p_1 / p_2 / \dots / p_k / \text{name}.\text{ext}, \quad 0 \le k \le D_{\max} = 16 */
        constraint def Invariant_REQ_0043_FormalInvariant;
        doc /* \Omega_{\text{ns}}(P_{\text{rel}}(f)) = \begin{cases} \text{SymbolId}(\text{PascalCase}(p_1) \mathbin{\Vert} \text{"::"} \mathbin{\Vert} \dots \mathbin{\Vert} \text{PascalCase}(p_k)), & \text{if } k \ge 1 \\ \text{SymbolId}(\text{"RootPackage"}), & \text{if } k = 0 \end{cases} */
        constraint def Invariant_REQ_0043_The_directory_to_namespace_mapping_function_evaluates_as;
        doc /* \forall i \in \{1, \dots, k\}, \quad \exists! P_i \in \text{Packages}(\mathcal{A}) \quad \text{such that} \quad \text{FQN}(P_i) = \bigodot_{j=1}^i \text{PascalCase}(p_j) */
        constraint def Invariant_REQ_0043_For_every_file_the_AST_package_set_contains_all_prefix_packages;
        doc /* \begin{aligned} \Delta_{\text{uuid}}(P) &= \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(P)) \\ \forall e \in \text{TopLevelDeclarations}(f), &\quad \text{Owner}(e) = \Omega_{\text{ns}}(P_{\text{rel}}(f)) \end{aligned} */
        constraint def Invariant_REQ_0043_Every_package_node_and_top_level_AST_declaration_satisfy;
        doc /* \begin{aligned} k \gt D_{\max} &\implies \text{Diagnostic}(f, \text{E0200}, \text{NestingDepthExceeded}) \in \mathcal{D}_{\text{sink}} \\ \text{Conflict}(\text{DeclaredPkg}(f), \Omega_{\text{ns}}(P_{\text{rel}}(f))) &\implies \text{Diagnostic}(f, \text{E0200}, \text{NamespaceMismatch}) \in \mathcal{D}_{\text{sink}} \\ (p_a \ne p_b \wedge \text{PascalCase}(p_a) = \text{PascalCase}(p_b)) &\implies \text{Diagnostic}(p_b, \text{E0200}, \text{NamespaceCollision}) \in \mathcal{D}_{\text{sink}} \\ |\mathcal{D}_{\text{sink}}| \ge C_{\text{max\_errors}} &\implies \exists D_{\text{term}} \in \mathcal{D}_{\text{sink}} : \text{Code}(D_{\text{term}}) = \text{E0199} \end{aligned} */
        constraint def Invariant_REQ_0043_Structural_and_semantic_violations_trigger_diagnostics_per_REQ_0191;
        require Invariant_REQ_0043_FormalInvariant;
        require Invariant_REQ_0043_The_directory_to_namespace_mapping_function_evaluates_as;
        require Invariant_REQ_0043_For_every_file_the_AST_package_set_contains_all_prefix_packages;
        require Invariant_REQ_0043_Every_package_node_and_top_level_AST_declaration_satisfy;
        require Invariant_REQ_0043_Structural_and_semantic_violations_trigger_diagnostics_per_REQ_0191;
        derived from REQ_0191_Standardized_Compiler_Diagnostic_Taxonomy_Source_Mapping;
        verify by AC_01_Canonical_Directory_to_Namespace_Hierarchy_Resolution;
        verify by AC_02_Idempotent_Intermediate_Package_Synthesis;
        verify by AC_03_Explicit_Package_Declaration_Reconciliation_and_Conflict_Rejection;
        verify by AC_04_Segment_Normalization_Collision_Detection_and_Depth_Bounds;
        satisfy by NodeArenaASTGraphEngine;
    }
    doc /* UUIDv5: 59348ea5-f96f-5c9e-8879-11a3c2be41ad | Complexity: Class P (O(N + E + M) = O(N)) | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0199, E0200 */
    requirement def REQ_0044_Lossless_AST_Serialization {
        id = "REQ-0044";
        text = "The Core Metamodel, Node Arena & AST Graph Engine shall provide deterministic, bidirectional, lossless serialization and deserialization of compiled Abstract Syntax Trees (AST) to and from minified JSON (RFC 8259) and binary CBOR (RFC 8949). Codecs must preserve all AST structural entities:";
        attribute uuidv5 : String = "59348ea5-f96f-5c9e-8879-11a3c2be41ad";
        attribute complexity_class : String = "Class P (O(N + E + M) = O(N))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0199, E0200";
        attribute ac_01_bidirectional_structural_round_trip_isomorphism_dereference_equality : String = "Given: A compiled model AST A containing abstract MBSE entities (Package_0, Classifier_Alpha, Port_1, Flow_A, param_x : Real). When: The compiler serializes A to minified JSON or binary CBOR and deserializes back into AST A' = D_{F}(S_{F}(A)). Then: Reconstructed AST satisfies A' A and n V, Deref(D_{F}(S_{F}(n))) n, preserving node count, hierarchy, edge targets, span coordinates, node flags, and RFC 4122 UUIDv5 hashes. Diagnostic: E0200 emitted on structural discrepancy, attribute divergence, or span mismatch between A and A'.";
        attribute ac_02_cbor_binary_compaction_bound : String = "Given: A compiled AST model A comprising |V| 100 nodes. When: The AST is serialized to canonical minified JSON and canonical binary CBOR. Then: Serialized byte sizes satisfy |S_{CBOR}(A)| 0.40 |S_{JSON}(A)|, achieving at least a 60% compaction gain. Diagnostic: E0200 emitted if the emitted CBOR artifact exceeds 40% of the minified JSON byte size.";
        attribute ac_03_string_interner_symbol_deduplication_idempotency : String = "Given: An AST containing K node instances referencing M unique identifier strings (M K). When: The model undergoes successive serialize-deserialize-serialize cycles. Then: Serialized payloads store a centralized table of exactly M deduplicated strings, deserialization interns symbols without duplication per Directive 3, and successive serialization streams B_1 = B_2 exhibit zero diff churn per REQ-0003. Diagnostic: E0200 emitted on symbol duplication or bitwise divergence across serialize-deserialize cycles.";
        attribute ac_04_error_recovery_diagnostic_binding_performance_gates : String = "Given: Serialized payload inputs containing malformed byte tokens at offset p_{byte}, or a 100,000-node AST evaluated in compiled Release mode (--release). When: Deserialization evaluates malformed streams or executes performance benchmark harnesses. Then: Per Directive 4, the deserializer resynchronizes on record boundaries, emitting structured diagnostic E0200 (or E0199 on sink exhaustion); per Directive 5 and REQ-0196, 100,000-node CBOR deserialization latency is 15 ms, serialization is 25 ms, and RSS memory is 100 MB with zero panics. Diagnostic: E0200 emitted for malformed stream tokens,.";
        doc /* \forall \mathcal{F} \in \{ \text{JSON}, \text{CBOR} \}, \quad \mathcal{D}_{\mathcal{F}}(\mathcal{S}_{\mathcal{F}}(\mathcal{A})) \cong \mathcal{A} */
        constraint def Invariant_REQ_0044_Round_Trip_Structural_Isomorphism;
        doc /* \forall n \in \mathcal{V}, \quad \text{Deref}(\mathcal{D}_{\mathcal{F}}(\mathcal{S}_{\mathcal{F}}(n))) \equiv n */
        constraint def Invariant_REQ_0044_Element_Wise_Preservation_Invariant;
        doc /* \begin{aligned} \text{UUID}(n) &= \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(n)) \\ \text{Span}(n) &= \langle \text{file\_id}, p_{\text{start}}, p_{\text{end}}, \text{line}, \text{col} \rangle \end{aligned} */
        constraint def Invariant_REQ_0044_FormalInvariant;
        doc /* \forall \mathcal{F} \in \{ \text{JSON}, \text{CBOR} \}, \quad \mathcal{S}_{\mathcal{F}}(\mathcal{D}_{\mathcal{F}}(\mathcal{S}_{\mathcal{F}}(\mathcal{A}))) = \mathcal{S}_{\mathcal{F}}(\mathcal{A}) */
        constraint def Invariant_REQ_0044_Bitwise_Serialization_Idempotency;
        doc /* |\mathcal{V}| \ge 100 \implies |\mathcal{S}_{\text{CBOR}}(\mathcal{A})| \le 0.40 \cdot |\mathcal{S}_{\text{JSON}}(\mathcal{A})| */
        constraint def Invariant_REQ_0044_CBOR_Binary_Compaction_Bound;
        doc /* \begin{aligned} \text{MalformedPayload}(\mathcal{B}) &\implies \text{Diagnostic}(p_{\text{byte}}, \text{E0200}) \in \mathcal{D}_{\text{sink}} \\ \neg(\mathcal{D}_{\mathcal{F}}(\mathcal{S}_{\mathcal{F}}(\mathcal{A})) \cong \mathcal{A}) &\implies \text{Diagnostic}(p_{\text{byte}}, \text{E0200}) \in \mathcal{D}_{\text{sink}} \\ |\mathcal{D}_{\text{sink}}| \ge C_{\text{max\_errors}} &\implies \text{Diagnostic}(p_{\text{byte}}, \text{E0199}) \in \mathcal{D}_{\text{sink}} \end{aligned} */
        constraint def Invariant_REQ_0044_Diagnostic_Code_Binding_Invariant;
        require Invariant_REQ_0044_Round_Trip_Structural_Isomorphism;
        require Invariant_REQ_0044_Element_Wise_Preservation_Invariant;
        require Invariant_REQ_0044_FormalInvariant;
        require Invariant_REQ_0044_Bitwise_Serialization_Idempotency;
        require Invariant_REQ_0044_CBOR_Binary_Compaction_Bound;
        require Invariant_REQ_0044_Diagnostic_Code_Binding_Invariant;
        derived from REQ_0002_Surjective_Lexical_Provenance_Gate;
        derived from REQ_0010_Deterministic_RFC_4122_UUIDv5_Topological_Namespace_Hashing;
        derived from REQ_0009_Zero_Copy_Bump_Arena_Memory_Management;
        derived from REQ_0034_Contiguous_Memory_Lowering;
        derived from REQ_0035_Deterministic_Node_Handle_Addressing;
        derived from REQ_0036_Concurrent_String_Interning;
        derived from REQ_0003_Bitwise_Determinism;
        derived from REQ_0013_Synchronization_Token_Error_Recovery_Architecture;
        derived from REQ_0016_Synchronization_Token_Recovery_on_Table_Delimiter_Boundaries;
        derived from REQ_0191_Standardized_Compiler_Diagnostic_Taxonomy_Source_Mapping;
        derived from REQ_0011_Deterministic_Non_Panicking_Execution_Contract;
        derived from REQ_0196_Performance_Assurance_Gates;
        verify by AC_01_Bidirectional_Structural_Round_Trip_Isomorphism_Dereference_Equality;
        verify by AC_02_CBOR_Binary_Compaction_Bound;
        verify by AC_03_String_Interner_Symbol_Deduplication_Idempotency;
        verify by AC_04_Error_Recovery_Diagnostic_Binding_Performance_Gates;
        satisfy by NodeArenaASTGraphEngine;
    }
    doc /* UUIDv5: 7814e7b1-e163-5828-90f0-5146ed48d963 | Complexity: Class P (O( | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0199, E0200 */
    requirement def REQ_0045_Immutable_AST_Graph_Traversal_Engine_with_Bounded_Linear_Complexity {
        id = "REQ-0045";
        text = "The Core Metamodel, Node Arena & AST Graph Engine shall provide an immutable graph traversal engine that visits vertices and directed edges of the abstract syntax tree (AST) multigraph G = (V_{AST}, E_{AST}) in topological, depth-first (pre-order and post-order), or breadth-first (level-order)";
        attribute uuidv5 : String = "7814e7b1-e163-5828-90f0-5146ed48d963";
        attribute complexity_class : String = "Class P (O(";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0199, E0200";
        attribute ac_01_linear_traversal_scaling_across_scale_topologies : String = "Given: Synthetic AST multigraphs rooted at Package_0 scaling from 10^3 to 10^5 vertices and edges (Classifier_Alpha, Port_1, Flow_A). When: The traversal engine executes DFS pre-order, DFS post-order, and BFS passes. Then: Runtime scales linearly satisfying T(V, E) c_1 (V + E) + c_0 (R^2 0.99), visiting all reachable vertices exactly once (VisitedCount(v) = 1) and unreachable vertices zero times. Diagnostic: E0200 emitted on traversal failure or non-linear scaling termination.";
        attribute ac_02_immutability_and_zero_heap_allocation_verification : String = "Given: A populated node arena A_{node} containing synthetic elements Package_0, Classifier_Alpha, and Port_1, instrumented with SHA-256 digests and allocator hooks. When: The traversal engine executes DFS and BFS passes across the full model graph. Then: Post-traversal SHA-256 arena digests are bitwise identical to pre-traversal digests (Mutations = 0), and allocator hooks record zero heap allocations (HeapAllocs = 0). Diagnostic: E0200 emitted on arena byte alteration or unexpected heap allocation.";
        attribute ac_03_cyclic_graph_termination_and_single_visit_invariant : String = "Given: A synthetic AST multigraph with circular references (Classifier_Alpha -> Classifier_Beta -> Classifier_Alpha and self-loop Port_1 -> Port_1). When: The engine executes traversal starting from root vertex Package_0. Then: The visited bitset detects back-edges, skips redundant exploration, terminates deterministically without infinite loops, and records VisitedCount(v) = 1 for all reachable cycle nodes in O(V + E) steps. Diagnostic: E0200 emitted on cycle-induced divergence or duplicate node visitation.";
        attribute ac_04_traversal_ordering_sequences_and_visitor_control_directives : String = "Given: A synthetic DAG containing hierarchy Package_0::Classifier_Alpha::Port_1 and visitors configured with Continue, Prune, and Halt directives. When: Executing pre-order, post-order, topological, and directive-controlled traversals. Then: Pre-order visits parents before children (pos(u) pos(v)); post-order visits children before parents (pos(u) pos(v)); Prune skips child branches; Halt terminates traversal immediately in O(1) steps. Diagnostic: E0200 emitted on traversal order sequence violation.";
        doc /* \begin{aligned} T_{\text{traversal}}(G, v_0) &\le c_1 \cdot (|V_{\text{reach}}(v_0)| + |E_{\text{reach}}(v_0)|) + c_0 \\ \forall v \in \mathcal{V}_{\text{AST}}, \quad \sigma(v, t_{\text{post}}) &\equiv \sigma(v, t_{\text{pre}}), \quad \text{Mutations} = 0, \quad \text{HeapAllocs} = 0 \end{aligned} */
        constraint def Invariant_REQ_0045_Bounded_Traversal_Time_and_Zero_Mutation_Invariants;
        doc /* \begin{aligned} \forall v \in V_{\text{reach}}(v_0), \quad \text{VisitedCount}(v) &= 1, \qquad \forall v \notin V_{\text{reach}}(v_0), \quad \text{VisitedCount}(v) = 0 \\ B[\text{Index}(v_1)] = 1 &\implies \text{TraverseEdge}(v_m, v_1) = \text{Skip} \end{aligned} */
        constraint def Invariant_REQ_0045_Bitset_Completeness_and_Cycle_Termination_Invariants;
        doc /* \begin{aligned} \forall (u, v, \kappa) \in \mathcal{E}_{\text{tree}}, \quad \text{pos}(\sigma_{\text{pre}}, u) &\lt \text{pos}(\sigma_{\text{pre}}, v), \quad \text{pos}(\sigma_{\text{post}}, u) \gt \text{pos}(\sigma_{\text{post}}, v) \\ \forall (u, v, \kappa) \in \mathcal{E}_{\text{DAG}}, \quad \text{pos}(\sigma_{\text{topo}}, u) &\lt \text{pos}(\sigma_{\text{topo}}, v) \end{aligned} */
        constraint def Invariant_REQ_0045_Traversal_Ordering_Sequence_Invariants;
        doc /* \begin{aligned} |S_{\text{stack}}(t)| &\le D_{\text{max\_depth}} \\ \neg\text{ValidHandle}(H, \mathcal{A}_{\text{node}}) \vee (|S_{\text{stack}}(t)| \gt D_{\text{max\_depth}}) &\implies \exists D \in \mathcal{D}_{\text{sink}} : \text{code}(D) = \text{DiagnosticCode::E0200} \\ |\mathcal{D}_{\text{sink}}| \ge C_{\text{max\_errors}} &\implies \exists D_{\text{term}} \in \mathcal{D}_{\text{sink}} : \text{code}(D_{\text{term}}) = \text{DiagnosticCode::E0199} \end{aligned} */
        constraint def Invariant_REQ_0045_Auxiliary_Depth_Bound_and_Diagnostic_Bindings_per_REQ_0191;
        require Invariant_REQ_0045_Bounded_Traversal_Time_and_Zero_Mutation_Invariants;
        require Invariant_REQ_0045_Bitset_Completeness_and_Cycle_Termination_Invariants;
        require Invariant_REQ_0045_Traversal_Ordering_Sequence_Invariants;
        require Invariant_REQ_0045_Auxiliary_Depth_Bound_and_Diagnostic_Bindings_per_REQ_0191;
        derived from REQ_0191_Standardized_Compiler_Diagnostic_Taxonomy_Source_Mapping;
        verify by AC_01_Linear_Traversal_Scaling_Across_Scale_Topologies;
        verify by AC_02_Immutability_and_Zero_Heap_Allocation_Verification;
        verify by AC_03_Cyclic_Graph_Termination_and_Single_Visit_Invariant;
        verify by AC_04_Traversal_Ordering_Sequences_and_Visitor_Control_Directives;
        satisfy by NodeArenaASTGraphEngine;
    }
    doc /* UUIDv5: 670b4084-8483-5958-b95a-7f84e98185fb | Complexity: Class P (O(1)) | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0200 */
    requirement def REQ_0046_Mutable_AST_Transformation {
        id = "REQ-0046";
        text = "The abstract syntax tree (AST) engine shall provide a deterministic, mutable transformation pass engine for lowering, syntactic desugaring, optimization rewrites, constant propagation, and semantic enrichments while strictly preserving graph topological integrity and typed reference safety. All AST";
        attribute uuidv5 : String = "670b4084-8483-5958-b95a-7f84e98185fb";
        attribute complexity_class : String = "Class P (O(1))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0200";
        attribute ac_01_in_place_slot_index_and_physical_address_invariance : String = "Given: Node Classifier_Alpha in arena slot k with address Base(A) + k S_{slot}. When: The engine applies an attribute modification or desugaring rewrite pass. Then: Slot index satisfies Index(Classifier_Alpha}, A_{t+1}) = k, virtual memory address remains invariant, and zero reallocations occur. Diagnostic: E0200 emitted on slot relocation or index divergence.";
        attribute ac_02_generational_counter_monotonicity_stale_dereference_invalidation : String = "Given: Node handle H_1 = k, g, referencing slot k with generation g. When: Slot k undergoes in-place mutation or replacement at step t t+1. Then: Generation increments to (g + 1) 2^{16}}, dereferencing H_1 returns , and new handle H_2 = k, (g + 1) 2^{16}}, dereferences in O(1) time. Diagnostic: E0200 emitted on stale handle access.";
        attribute ac_03_desugaring_structural_integrity_zero_dangling_edges : String = "Given: An AST containing composite interconnect declarations between Port_1 and Port_2. When: A desugaring pass lowers composite connectors into directed primitive flows. Then: 100% of emitted directed edges connect active, valid node slots in A_{t+1}, satisfying DanglingEdges(A_{t+1}) = . Diagnostic: E0200 emitted on dangling edge or invalid endpoint handle.";
        attribute ac_04_deterministic_rfc_4122_uuidv5_anchor_preservation : String = "Given: Structural node Package_0::Classifier_Alpha with canonical topological path p. When: The transformation engine executes an in-place rewrite pass. Then: The node's RFC 4122 UUIDv5 matches UUIDv5(NS_{DEAP}, p) with version bits 0x50 and variant bits 0x80, ensuring 0-byte diff churn. Diagnostic: E0200 emitted on UUIDv5 computation failure or variant mismatch.";
        doc /* \mathcal{S}_t[i] = \langle \mathcal{N}_t(i), g_t(i), \tau_t(i), b_t(i) \rangle */
        constraint def Invariant_REQ_0046_FormalInvariant;
        doc /* \mathcal{T}_{\text{rewrite}} : \mathcal{A}_t \times \mathcal{P}_{\text{rules}} \longrightarrow \mathcal{A}_{t+1} \times \mathcal{D} */
        constraint def Invariant_REQ_0046_An_AST_rewrite_pass_is_a_deterministic_transformation;
        doc /* \forall n \in \text{ModifiedNodes}(\mathcal{T}_{\text{rewrite}}), \quad \text{Index}(n, \mathcal{A}_{t+1}) = \text{Index}(n, \mathcal{A}_t) \;\wedge\; \text{Address}(n, \mathcal{A}_{t+1}) = \text{Address}(n, \mathcal{A}_t) */
        constraint def Invariant_REQ_0046_In_Place_Slot_Index_and_Physical_Address_Invariance;
        doc /* \begin{aligned} g_{t+1}(i) &= (g_t(i) + 1) \pmod{2^{16}} \\ \forall H = \langle i, g_H, \tau \rangle, \quad g_H \ne g_{t+1}(i) &\implies \text{Valid}(H, \mathcal{A}_{t+1}) = \text{false} \;\wedge\; \text{Deref}(H, \mathcal{A}_{t+1}) = \bot \end{aligned} */
        constraint def Invariant_REQ_0046_Generational_Mutation_and_Stale_Reference_Invalidation_Invariant;
        doc /* \forall e = (u, v) \in \text{Edges}(\mathcal{A}_{t+1}), \quad u \in \text{Nodes}(\mathcal{A}_{t+1}) \;\wedge\; v \in \text{Nodes}(\mathcal{A}_{t+1}) \;\wedge\; \text{DanglingEdges}(\mathcal{A}_{t+1}) = \emptyset */
        constraint def Invariant_REQ_0046_Topological_Graph_Closure_and_Zero_Dangling_Edges_Invariant;
        doc /* \begin{aligned} \mathcal{P}_{\text{path}}(n, \mathcal{A}_{t+1}) = \mathcal{P}_{\text{path}}(n, \mathcal{A}_t) &\implies \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \mathcal{P}_{\text{path}}(n, \mathcal{A}_{t+1})) = \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \mathcal{P}_{\text{path}}(n, \mathcal{A}_t)) \\ \neg\text{Valid}(H, \mathcal{A}_{t+1}) &\implies \text{Diagnostic}(\text{span}(H), \text{E0200}) \in \mathcal{D} \end{aligned} */
        constraint def Invariant_REQ_0046_Deterministic_UUIDv5_and_Diagnostic_Code_Invariants;
        require Invariant_REQ_0046_FormalInvariant;
        require Invariant_REQ_0046_An_AST_rewrite_pass_is_a_deterministic_transformation;
        require Invariant_REQ_0046_In_Place_Slot_Index_and_Physical_Address_Invariance;
        require Invariant_REQ_0046_Generational_Mutation_and_Stale_Reference_Invalidation_Invariant;
        require Invariant_REQ_0046_Topological_Graph_Closure_and_Zero_Dangling_Edges_Invariant;
        require Invariant_REQ_0046_Deterministic_UUIDv5_and_Diagnostic_Code_Invariants;
        verify by AC_01_In_Place_Slot_Index_and_Physical_Address_Invariance;
        verify by AC_02_Generational_Counter_Monotonicity_Stale_Dereference_Invalidation;
        verify by AC_03_Desugaring_Structural_Integrity_Zero_Dangling_Edges;
        verify by AC_04_Deterministic_RFC_4122_UUIDv5_Anchor_Preservation;
        satisfy by NodeArenaASTGraphEngine;
    }
    doc /* UUIDv5: a4c7e39d-7336-50db-b9cf-082884b38897 | Complexity: Class P (O(1)) | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0200, E0203 */
    requirement def REQ_0047_Graph_Theoretic_AST_Representation {
        id = "REQ-0047";
        text = "The compiler core shall construct, maintain, and query an in-memory bidirectional directed property multigraph G = (V, E) over abstract syntax tree (AST) nodes stored in the AST. The multigraph implementation must remain strictly technology-neutral, clean-room engineered, and decoupled from";
        attribute uuidv5 : String = "a4c7e39d-7336-50db-b9cf-082884b38897";
        attribute complexity_class : String = "Class P (O(1))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0200, E0203";
        attribute ac_01_bidirectional_adjacency_registration_degree_conservation : String = "Given: AST nodes Package_0, Classifier_Alpha, Classifier_Beta, and Port_1. When: The multigraph builder inserts directed edges across kinds Contains, Specializes, and ConnectsTo. Then: Every edge e is queryable in both OutEdges(src(e)) and InEdges(dst(e)), satisfying _{v V |OutEdges(v)| = |E| = _{v V |InEdges(v)|. Diagnostic: E0200 emitted on adjacency registration discrepancy.";
        attribute ac_02_parallel_multigraph_edge_admissibility : String = "Given: AST nodes Port_1 and Port_2. When: The compiler records connection edge e_1 (ConnectsTo) and two flow edges e_2, e_3 (FlowsTo for Flow_A and Flow_B) between (Port_1}, Port_2}). Then: All three edges are assigned distinct 32-bit EdgeId keys without collision or property loss. Diagnostic: E0200 emitted if parallel edge insertion collapses distinct edge identities.";
        attribute ac_03_structural_containment_forest_acyclicity : String = "Given: Containment path where Package_0 contains Classifier_Alpha, which contains Port_1. When: A compilation pass attempts to insert an inverted containment edge (Contains) from child Port_1 to ancestor Package_0. Then: The engine rejects the cycle, preserves valid forest structure, and emits diagnostic E0203 without panicking. Diagnostic: E0203 emitted on detected cycle in containment hierarchy.";
        attribute ac_04_invalid_and_stale_node_handle_rejection : String = "Given: An edge insertion request referencing an unallocated 32-bit arena index or an expired node handle. When: Endpoint handles are validated during edge insertion. Then: Insertion is rejected under a total-function contract, graph state remains unchanged, and diagnostic E0200 is recorded. Diagnostic: E0200 emitted on invalid or out-of-bounds node handle.";
        doc /* G = \langle V, E, \text{src}, \text{dst}, \text{kind}, \text{prop} \rangle */
        constraint def Invariant_REQ_0047_FormalInvariant;
        doc /* \begin{aligned} V &= \text{Nodes}(\mathcal{A}) \subset \mathbb{U}_{32}, \quad E \subset \mathbb{U}_{32} \\ \text{src}, \text{dst} &: E \longrightarrow V, \quad \text{kind} : E \longrightarrow \mathcal{K}_{\text{edge}}, \quad \text{prop} : E \longrightarrow \mathcal{P} \end{aligned} */
        constraint def Invariant_REQ_0047_where;
        doc /* \begin{aligned} \mathcal{K}_{\text{edge}} &= \{ \text{Contains}, \text{Specializes}, \text{Types}, \text{ConnectsTo}, \text{FlowsTo}, \text{AllocatesTo}, \text{Constrains}, \text{DependsOn} \} \\ \exists e_1, e_2 \in E &: (e_1 \ne e_2 \;\wedge\; \text{src}(e_1) = \text{src}(e_2) \;\wedge\; \text{dst}(e_1) = \text{dst}(e_2)) \end{aligned} */
        constraint def Invariant_REQ_0047_Canonical_edge_kinds_and_multigraph_parallel_edge_admissibility;
        doc /* \begin{aligned} \text{OutEdges}(u) = \{ e \in E \mid \text{src}(e) = u \}, &\quad \text{InEdges}(v) = \{ e \in E \mid \text{dst}(e) = v \} \\ \forall e \in E, \quad e \in \text{OutEdges}(\text{src}(e)) &\;\wedge\; e \in \text{InEdges}(\text{dst}(e)) \\ \sum_{v \in V} |\text{OutEdges}(v)| &= |E| = \sum_{v \in V} |\text{InEdges}(v)| \end{aligned} */
        constraint def Invariant_REQ_0047_Bidirectional_Adjacency_and_Degree_Conservation;
        doc /* \begin{aligned} e_1 \prec_E e_2 \iff & (\text{kind}(e_1) \lt \text{kind}(e_2)) \lor (\text{kind}(e_1) = \text{kind}(e_2) \wedge \text{dst}(e_1) \lt \text{dst}(e_2)) \\ & \lor (\text{kind}(e_1) = \text{kind}(e_2) \wedge \text{dst}(e_1) = \text{dst}(e_2) \wedge e_1 \lt e_2) \end{aligned} */
        constraint def Invariant_REQ_0047_Deterministic_Canonical_Edge_Ordering;
        doc /* \begin{aligned} T_{\text{neighbors}}(v) &\le C_{\text{adj}} \cdot (|\text{OutEdges}(v)| + |\text{InEdges}(v)| + 1) \\ G_{\text{cont}} = (V, E_{\text{Contains}}), &\quad \text{Cycles}(G_{\text{cont}}) = \emptyset \\ \exists e \in E : (\text{src}(e) \notin V \lor \text{dst}(e) \notin V) &\implies \text{Diagnostic}(e, \text{E0200}) \in \mathcal{D} \\ \text{Cycles}(G_{\text{cont}}) \ne \emptyset &\implies \text{Diagnostic}(e, \text{E0203}) \in \mathcal{D} \end{aligned} */
        constraint def Invariant_REQ_0047_Output_Sensitive_Query_Bound_Containment_Acyclicity_and_Diagnostics;
        require Invariant_REQ_0047_FormalInvariant;
        require Invariant_REQ_0047_where;
        require Invariant_REQ_0047_Canonical_edge_kinds_and_multigraph_parallel_edge_admissibility;
        require Invariant_REQ_0047_Bidirectional_Adjacency_and_Degree_Conservation;
        require Invariant_REQ_0047_Deterministic_Canonical_Edge_Ordering;
        require Invariant_REQ_0047_Output_Sensitive_Query_Bound_Containment_Acyclicity_and_Diagnostics;
        derived from REQ_0034_Contiguous_Memory_Lowering;
        derived from REQ_0035_Deterministic_Node_Handle_Addressing;
        derived from REQ_0003_Bitwise_Determinism;
        derived from REQ_0010_Deterministic_RFC_4122_UUIDv5_Topological_Namespace_Hashing;
        derived from REQ_0011_Deterministic_Non_Panicking_Execution_Contract;
        verify by AC_01_Bidirectional_Adjacency_Registration_Degree_Conservation;
        verify by AC_02_Parallel_Multigraph_Edge_Admissibility;
        verify by AC_03_Structural_Containment_Forest_Acyclicity;
        verify by AC_04_Invalid_and_Stale_Node_Handle_Rejection;
        satisfy by NodeArenaASTGraphEngine;
    }
    doc /* UUIDv5: 0dba3619-fdda-50fb-81b5-e61f6ea66fb5 | Complexity: Class P (O( | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0199, E0200, E0203 */
    requirement def REQ_0048_Topological_Sort_Dependency_Scheduling_Tarjan_SCC {
        id = "REQ-0048";
        text = "The core abstract syntax tree (AST) graph engine shall provide deterministic topological dependency scheduling, cycle detection, and shortest-path routing across model elements. Vertices represent lowered AST entities (packages, classifier definitions, feature usages, and ports) in the AST";
        attribute uuidv5 : String = "0dba3619-fdda-50fb-81b5-e61f6ea66fb5";
        attribute complexity_class : String = "Class P (O(";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0199, E0200, E0203";
        attribute ac_01_diamond_dag_topological_scheduling_dependency_invariant : String = "Given: A DAG over elements Package_0, Classifier_Alpha, Classifier_Beta, Classifier_Gamma, and Classifier_Delta, where Classifier_Alpha precedes Classifier_Beta and Classifier_Gamma, and both precede Classifier_Delta. When: Kahn topological sorting executes to establish compilation schedule L. Then: Sequence L orders Classifier_Alpha before Classifier_Beta and Classifier_Gamma, and both before Classifier_Delta, satisfying Index(u, L) Index(v, L) for all (u, v) E. Diagnostic: E0200 on ordering violation.";
        attribute ac_02_deterministic_lexicographical_tie_breaking : String = "Given: Sibling nodes Classifier_Beta and Classifier_Gamma in Package_0 attaining in-degree zero simultaneously in ready queue Q. When: Kahn algorithm dequeues the next schedulable node handle. Then: Node Package_0::Classifier_Beta is selected before Package_0::Classifier_Gamma by lexicographical order of interned SymbolId tokens, producing identical schedule L across edge insertion permutations. Diagnostic: E0200 on non-deterministic tie-breaking.";
        attribute ac_03_tarjan_scc_condensation_trivial_component_isolation : String = "Given: An acyclic model multigraph G containing 1,000 nodes and 2,500 edges. When: Tarjan SCC decomposition executes. Then: Exactly 1,000 singleton components (|C_i| = 1) are identified, producing condensation DAG G^{scc} without self-loops and zero diagnostics emitted. Diagnostic: E0203 if false cycles are reported.";
        attribute ac_04_circular_dependency_detection_traceback_synchronization_recovery : String = "Given: An AST model containing cyclic loop Classifier_Alpha Classifier_Beta Classifier_Gamma Classifier_Alpha and independent acyclic element Classifier_Delta. When: Tarjan SCC evaluates the model graph. Then: The engine detects component |C| = 3, extracts traceback path Classifier_Alpha}, Classifier_Beta}, Classifier_Gamma}, Classifier_Alpha} , records diagnostic E0203, marks cyclic handles invalid, and recovers to schedule Classifier_Delta. Diagnostic: E0203 on cycle detection; E0199 on diagnostic sink overflow (C_{max_errors}}).";
        doc /* G = (V, E), \quad V \subset \mathbb{U}_{32}, \quad E \subseteq V \times V, \quad \text{deg}^{-}(v) = |\{ u \in V \mid (u, v) \in E \}| */
        constraint def Invariant_REQ_0048_FormalInvariant;
        doc /* \begin{aligned} \forall (u, v) \in E, \quad &\text{Index}(u, \mathcal{L}) \lt \text{Index}(v, \mathcal{L}) \\ Q_t &= \{ v \in V \setminus \{ v_1, \dots, v_{t-1} \} \mid \text{deg}^{-}_t(v) = 0 \} \\ v_t &= \arg\min_{v \in Q_t} (\text{QualifiedName}(v)), \quad \mathcal{L}_t = \mathcal{L}_{t-1} \mathbin{\Vert} \langle v_t \rangle \end{aligned} */
        constraint def Invariant_REQ_0048_Kahn_Topological_Linear_Ordering_Deterministic_Tie_Breaking_Invariant;
        doc /* \begin{aligned} \text{low\_link}(v) &= \min \left( \{ \text{df\_num}(v) \} \cup \{ \text{low\_link}(w) \mid (v, w) \in E_{\text{tree}} \} \cup \{ \text{df\_num}(w) \mid (v, w) \in (E_{\text{back}} \cup E_{\text{cross}}) \wedge w \in S \} \right) \\ G^{\text{scc}} &= (\mathcal{C}, E^{\text{scc}}), \quad E^{\text{scc}} = \{ (C_i, C_j) \in \mathcal{C} \times \mathcal{C} \mid i \ne j \wedge \exists u \in C_i, v \in C_j : (u, v) \in E \} \end{aligned} */
        constraint def Invariant_REQ_0048_Tarjan_Strongly_Connected_Components_and_Condensation_DAG_Invariant;
        doc /* \begin{aligned} (|C| \gt 1 \lor \exists v \in C : (v, v) \in E) &\implies \text{Diagnostic}(C, \text{E0203}, \mathcal{P}_{\text{cycle}}) \in \mathcal{D}_{\text{sink}} \\ \text{ShortestPath}(u, v) &= \arg\min_{\mathcal{P} \in \text{Paths}(u, v)} |\mathcal{P}| \end{aligned} */
        constraint def Invariant_REQ_0048_Cycle_Detection_Diagnostics_and_Shortest_Path_Query_Invariants;
        require Invariant_REQ_0048_FormalInvariant;
        require Invariant_REQ_0048_Kahn_Topological_Linear_Ordering_Deterministic_Tie_Breaking_Invariant;
        require Invariant_REQ_0048_Tarjan_Strongly_Connected_Components_and_Condensation_DAG_Invariant;
        require Invariant_REQ_0048_Cycle_Detection_Diagnostics_and_Shortest_Path_Query_Invariants;
        derived from REQ_0003_Bitwise_Determinism;
        verify by AC_01_Diamond_DAG_Topological_Scheduling_Dependency_Invariant;
        verify by AC_02_Deterministic_Lexicographical_Tie_Breaking;
        verify by AC_03_Tarjan_SCC_Condensation_Trivial_Component_Isolation;
        verify by AC_04_Circular_Dependency_Detection_Traceback_Synchronization_Recovery;
        satisfy by NodeArenaASTGraphEngine;
    }
    doc /* UUIDv5: 99ff3654-b23b-5c45-aebe-5158bd651f34 | Complexity: Class P (O(1)) | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0199, E0200, E0201 */
    requirement def REQ_0049_Context_Bounded_AST_Slicing_for_Token_Isolated_Subagents {
        id = "REQ-0049";
        text = "The core metamodel, node arena, and AST graph engine shall provide a deterministic, depth-bounded semantic AST slicing facility (query_slice) to extract self-contained subgraphs surrounding a target element for focused reasoning, code generation, and micro-verification within context-isolated";
        attribute uuidv5 : String = "99ff3654-b23b-5c45-aebe-5158bd651f34";
        attribute complexity_class : String = "Class P (O(1))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0199, E0200, E0201";
        attribute ac_01_depth_bounded_metric_neighborhood_extraction : String = "Given: A 6-level containment hierarchy (Package_0 down to Part_5) and depth bound k = 2. When: query_slice targets mid-level entity Part_2. Then: Subgraph contains strictly nodes at distance dist_{G}(n_{target}, v) 2; nodes at distance d 2 without dependencies are excluded. Diagnostic: E0200 emitted on traversal violation.";
        attribute ac_02_transitive_semantic_closure_and_independent_compilability : String = "Given: Classifier_Alpha referencing type DataType_Sigma and constraint Constraint_Alpha outside metric ball (d k). When: query_slice is executed on Classifier_Alpha and slice is compiled in an isolated subagent context. Then: DataType_Sigma and Constraint_Alpha are included in V_{slice}, satisfying UnresolvedSymbols(G_{slice}) = . Diagnostic: E0200 emitted on unresolvable symbol.";
        attribute ac_03_boundary_port_reification_and_stub_synthesis : String = "Given: Port_1 on Part_0 connected across slice boundary to external Port_2. When: query_slice extracts Part_0 while excluding Port_2's parent classifier. Then: Boundary stub Port_Stub is synthesized preserving direction (in, out, inout) and payload type without including external hierarchies. Diagnostic: E0201 emitted on interface signature mismatch.";
        attribute ac_04_token_volume_reduction_and_context_isolation : String = "Given: Model A with 10^4 nodes and target Classifier_Alpha with depth k 3. When: Slice G_{slice} is serialized to canonical representation. Then: Serialized token volume satisfies |Tokens(G_{slice})| 0.15 |Tokens(A)|. Diagnostic: E0200 emitted on token volume violation.";
        doc /* E_{\text{sym}} = \{ \{u, v\} \mid \exists \kappa : (u, v, \kappa) \in E \;\lor\; (v, u, \kappa) \in E \} */
        constraint def Invariant_REQ_0049_FormalInvariant;
        doc /* \begin{aligned} \mathcal{B}_k(n_{\text{target}}) &= \{ v \in V \mid \text{dist}_{G}(n_{\text{target}}, v) \le k \} \\ \text{Ref}(v) &= \{ w \in V \mid \exists (v, w, \kappa) \in E : \kappa \in \{ \text{Typing}, \text{Subtyping}, \text{Constraint} \} \} \end{aligned} */
        constraint def Invariant_REQ_0049_The_metric_neighborhood_and_transitive_dependency_set_evaluate;
        doc /* \begin{aligned} V_{\text{slice}} &= \mathcal{B}_k(n_{\text{target}}) \cup \bigcup_{v \in V_{\text{slice}}} \text{Ref}(v) \\ E_{\text{slice}} &= \{ (u, v, \kappa) \in E \mid u \in V_{\text{slice}} \;\wedge\; v \in V_{\text{slice}} \} \end{aligned} */
        constraint def Invariant_REQ_0049_The_sliced_subgraph_is_the_least_fixed_point;
        doc /* \begin{aligned} \forall v \in V_{\text{slice}}, \; \text{Ref}(v) \subseteq V_{\text{slice}} &\implies \text{UnresolvedSymbols}(G_{\text{slice}}) = \emptyset \\ (u \in V_{\text{slice}} \;\wedge\; v \notin V_{\text{slice}} \;\wedge\; (u, v, \text{Conn}) \in E) &\implies \exists v_{\text{stub}} \in V_{\text{slice}} : \tau_V(v_{\text{stub}}) = \tau_V(v) \\ |\text{Tokens}(\text{Slice}(n_{\text{target}}, k))| &\le 0.15 \cdot |\text{Tokens}(\mathcal{A})| \quad (k \le 3) \end{aligned} */
        constraint def Invariant_REQ_0049_Semantic_Closure_Boundary_Stub_Port_and_Token_Invariants;
        doc /* \begin{aligned} \forall v \in V_{\text{slice}}, \quad \text{UUID}(v) &= \text{UUIDv5}(\text{OID}_{\text{DEAP}}, \text{FQN}(v)) \\ (\neg\text{ValidHandle}(n_{\text{target}}) \;\lor\; k \le 0 \;\lor\; k \gt k_{\text{max}}) &\implies \text{Diagnostic}(n_{\text{target}}, \text{E0200}) \in \mathcal{D}_{\text{sink}} \\ |\mathcal{D}_{\text{sink}}| \ge C_{\text{max\_errors}} &\implies \exists D \in \mathcal{D}_{\text{sink}} : \text{Code}(D) = \text{E0199} \end{aligned} */
        constraint def Invariant_REQ_0049_Deterministic_Identity_and_Diagnostic_Invariants;
        require Invariant_REQ_0049_FormalInvariant;
        require Invariant_REQ_0049_The_metric_neighborhood_and_transitive_dependency_set_evaluate;
        require Invariant_REQ_0049_The_sliced_subgraph_is_the_least_fixed_point;
        require Invariant_REQ_0049_Semantic_Closure_Boundary_Stub_Port_and_Token_Invariants;
        require Invariant_REQ_0049_Deterministic_Identity_and_Diagnostic_Invariants;
        verify by AC_01_Depth_Bounded_Metric_Neighborhood_Extraction;
        verify by AC_02_Transitive_Semantic_Closure_and_Independent_Compilability;
        verify by AC_03_Boundary_Port_Reification_and_Stub_Synthesis;
        verify by AC_04_Token_Volume_Reduction_and_Context_Isolation;
        satisfy by NodeArenaASTGraphEngine;
    }
    doc /* UUIDv5: c52b4749-c122-58ca-ae07-5ee9651f2767 | Complexity: Class P (O( | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0199, E0200 */
    requirement def REQ_0050_AST_Slice_Serialization_Contract_with_Reified_Dependency_Envelopes {
        id = "REQ-0050";
        text = "The core metamodel and AST graph engine shall serialize extracted abstract syntax tree (AST) slices into standalone packages with reified dependency envelopes. Each serialized package must encapsulate the sliced subgraph alongside all referenced external classifier definitions, port contracts,";
        attribute uuidv5 : String = "c52b4749-c122-58ca-ae07-5ee9651f2767";
        attribute complexity_class : String = "Class P (O(";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0199, E0200";
        attribute ac_01_standalone_dependency_closure_and_symbol_resolution : String = "Given: Extracted slice G_{slice} rooted at Classifier_Alpha within Package_0 referencing external type Type_0 and unit Unit_0. When: The serialization engine computes envelope Deps(G_{slice}) and serializes package P_{slice}. Then: Deserialization into an isolated arena yields UnresolvedSymbols(G_{standalone}) = and ValidateSlice(G_{standalone}) Valid without parent AST access. Diagnostic: E0200 emitted on any missing external symbol declaration.";
        attribute ac_02_minimal_interface_stub_synthesis_and_structural_pruning : String = "Given: External classifier Classifier_Beta referenced by G_{slice}, defining public port Port_1 : Type_0, private subcomponents, and behavioral actions. When: The engine reifies Classifier_Beta into dependency envelope node set V_{deps}. Then: Synthesized stubs retain public signatures (Port_1, Type_0) with behavioral bodies stripped (Body(v) = ). Diagnostic: E0200 emitted if unexported private members or internal bodies leak into the envelope.";
        attribute ac_03_relocatable_handle_remapping_and_deterministic_uuidv5_anchors : String = "Given: Sliced nodes and dependency stubs with arbitrary parent arena handles. When: The serialization pass encodes node records into the output buffer. Then: Handles are bijectively mapped to dense slice-relative indices [0, |V_{slice}| + |V_{deps}| - 1], each node preserving its immutable UUIDv5 identifier UUIDv5(OID_{DEAP}, FQN(u)). Diagnostic: E0200 emitted on handle remapping collisions or non-contiguous index gaps.";
        attribute ac_04_dual_format_serialization_and_bitwise_parity : String = "Given: Validated standalone graph G_{standalone} = G_{slice} Deps(G_{slice}). When: The engine serializes the package into canonical binary format and deterministic JSON. Then: Deserializing either format reconstructs identical AST topologies, UUIDv5 identifiers, and attribute values with bitwise parity. Diagnostic: E0200 emitted on encoding failure or deserialization structural divergence.";
        doc /* \text{ReferencedSymbols}(G_{\text{slice}}) \subseteq \text{DefinedSymbols}(G_{\text{slice}}) \cup \text{DefinedSymbols}(\text{Deps}(G_{\text{slice}})) */
        constraint def Invariant_REQ_0050_Transitive_Symbol_Closure_Invariant;
        doc /* \text{UnresolvedSymbols}(G_{\text{slice}} \cup \text{Deps}(G_{\text{slice}})) = \emptyset */
        constraint def Invariant_REQ_0050_Zero_Dangling_References_Invariant;
        doc /* \text{UnresolvedSymbols}(G) = \text{ReferencedSymbols}(G) \setminus \text{DefinedSymbols}(G) */
        constraint def Invariant_REQ_0050_FormalInvariant;
        doc /* \text{ValidateSlice}(G_{\text{slice}} \cup \text{Deps}(G_{\text{slice}})) \equiv \text{Valid} */
        constraint def Invariant_REQ_0050_Standalone_Semantic_Validity_Invariant;
        doc /* \mathcal{P}_{\text{slice}} = \langle \mathcal{M}_{\text{manifest}}, G_{\text{slice}}, \text{Deps}(G_{\text{slice}}) \rangle */
        constraint def Invariant_REQ_0050_Serialized_Package_Tuple_Contract;
        doc /* \mathcal{M}_{\text{manifest}} = \langle \text{UUID}_{\text{root}}, h_{\text{target}}, k, \nu_{\text{schema}}, \delta_{\text{payload}}, |V_{\text{slice}}|, |V_{\text{deps}}| \rangle */
        constraint def Invariant_REQ_0050_FormalInvariant_2;
        doc /* \forall v \in V_{\text{deps}}, \quad \text{Body}(v) = \emptyset \quad \wedge \quad \text{IsPublicSignatureOnly}(v) \equiv \text{true} */
        constraint def Invariant_REQ_0050_Minimal_Interface_Stub_Invariant;
        doc /* \forall u \in V_{\text{slice}} \cup V_{\text{deps}}, \quad \text{UUID}(u) = \text{UUIDv5}(\text{OID}_{\text{DEAP}}, \text{FQN}(u)) */
        constraint def Invariant_REQ_0050_Deterministic_RFC_4122_UUIDv5_Topological_Invariant;
        doc /* \begin{aligned} \sigma_{\text{remap}} &: (V_{\text{slice}} \cup V_{\text{deps}}) \longrightarrow \{ 0, 1, \dots, |V_{\text{slice}}| + |V_{\text{deps}}| - 1 \} \\ \forall u, v &\in V_{\text{slice}} \cup V_{\text{deps}}, \quad u \ne v \implies \sigma_{\text{remap}}(u) \ne \sigma_{\text{remap}}(v) \end{aligned} */
        constraint def Invariant_REQ_0050_Relocatable_Slice_Handle_Remapping_Invariant;
        doc /* \begin{aligned} \mathcal{S}_{\text{ext}}(G_{\text{slice}}) \not\subseteq \text{DefinedSymbols}(\text{Deps}(G_{\text{slice}})) &\implies \text{Diagnostic}(G_{\text{slice}}, \text{DiagnosticCode::E0200}) \in \mathcal{D}_{\text{sink}} \\ |\mathcal{D}_{\text{sink}}| \ge C_{\text{max\_errors}} &\implies \text{DiagnosticCode::E0199} \in \mathcal{D}_{\text{sink}} \end{aligned} */
        constraint def Invariant_REQ_0050_Diagnostic_Code_Binding_and_Error_Recovery_Invariant;
        require Invariant_REQ_0050_Transitive_Symbol_Closure_Invariant;
        require Invariant_REQ_0050_Zero_Dangling_References_Invariant;
        require Invariant_REQ_0050_FormalInvariant;
        require Invariant_REQ_0050_Standalone_Semantic_Validity_Invariant;
        require Invariant_REQ_0050_Serialized_Package_Tuple_Contract;
        require Invariant_REQ_0050_FormalInvariant_2;
        require Invariant_REQ_0050_Minimal_Interface_Stub_Invariant;
        require Invariant_REQ_0050_Deterministic_RFC_4122_UUIDv5_Topological_Invariant;
        require Invariant_REQ_0050_Relocatable_Slice_Handle_Remapping_Invariant;
        require Invariant_REQ_0050_Diagnostic_Code_Binding_and_Error_Recovery_Invariant;
        derived from REQ_0049_Context_Bounded_AST_Slicing_for_Token_Isolated_Subagents;
        derived from REQ_0098_Formal_mathbb_7_Rational_Vector_Space_over_SI_Base_Dimensions;
        derived from REQ_0112_Parametric_Operational_Envelopes;
        derived from REQ_0038_Strongly_Typed_Port_Directions;
        verify by AC_01_Standalone_Dependency_Closure_and_Symbol_Resolution;
        verify by AC_02_Minimal_Interface_Stub_Synthesis_and_Structural_Pruning;
        verify by AC_03_Relocatable_Handle_Remapping_and_Deterministic_UUIDv5_Anchors;
        verify by AC_04_Dual_Format_Serialization_and_Bitwise_Parity;
        satisfy by NodeArenaASTGraphEngine;
    }
    doc /* UUIDv5: b559cfe1-5118-57c7-9ca6-acb5a95dbce9 | Complexity: Class P (O( | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0200, E0203 */
    requirement def REQ_0051_Incremental_Compilation_Cache_Invalidation_via_Module_Dependency_DAG {
        id = "REQ-0051";
        text = "The compiler core metamodel and AST graph engine shall implement a deterministic incremental compilation caching subsystem structured upon an explicit module dependency directed acyclic graph (DAG) G = (V, E). For each module M \in V, the compiler shall compute a 64-bit cryptographic content hash";
        attribute uuidv5 : String = "b559cfe1-5118-57c7-9ca6-acb5a95dbce9";
        attribute complexity_class : String = "Class P (O(";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0200, E0203";
        attribute ac_01_leaf_module_mutation_and_isolated_recompilation : String = "Given: A dependency DAG in Package_0 structured as Module_Alpha -> Module_Beta -> Module_Gamma, with state hashes matching cached records. When: Leaf module Module_Gamma source is mutated (H_{source}(Module_Gamma}) H_{cached_source}}(Module_Gamma})) and incremental compilation executes. Then: Strictly Module_Gamma is marked for recomputation (R = { Module_Gamma} }); upstream Module_Alpha and Module_Beta are reused directly from cache (T_{recompile} = 0). Diagnostic: E0200 emitted on redundant re-analysis of unaffected modules.";
        attribute ac_02_root_module_mutation_and_transitive_downstream_invalidation : String = "Given: A dependency chain Module_Alpha -> Module_Beta -> Module_Gamma and disjoint Module_Delta in Package_0. When: Root module Module_Alpha source is mutated. Then: The compiler identifies TransitiveDependents(Module_Alpha}) = { Module_Beta}, Module_Gamma} }, recomputes R = { Module_Alpha}, Module_Beta}, Module_Gamma} }, and reuses Module_Delta from cache. Diagnostic: E0200 emitted if any transitive dependent is omitted from invalidation or if Module_Delta is re-analyzed.";
        attribute ac_03_clean_no_op_incremental_cache_hit_bypass : String = "Given: An unmodified workspace where all module source hashes match cached records (H_{source}(M) = H_{cached_source}}(M) for all M V). When: Incremental compilation executes. Then: All module state hashes match cached state hashes (H_{state}(M) = H_{cached}(M)), all AST subgraphs are reused from cache (T_{recompile}(M) = 0), and Merkle root parity is preserved. Diagnostic: E0200 emitted if any module undergoes re-parsing during an unmodified build.";
        attribute ac_04_circular_module_dependency_rejection : String = "Given: An incremental module import introducing a cyclic dependency (Module_Alpha -> Module_Beta -> Module_Alpha). When: The compiler executes DAG construction and cycle detection. Then: The compiler detects non-acyclicity, halts lowering, rejects the invalid graph, and emits diagnostic error code E0203 detailing cycle path coordinates without panicking. Diagnostic: E0203 (Circular module dependency detected in module DAG).";
        doc /* \text{TransitiveDependents}(M) = \{ D \in V \mid M \to^+ D \} */
        constraint def Invariant_REQ_0051_The_transitive_downstream_dependent_set_is_defined_by_forward_path_reachability;
        doc /* H_{\text{state}}(M) = \text{Hash64}\left( H_{\text{source}}(M) \mathbin{\Vert} \bigoplus_{D \in \text{Deps}(M)} H_{\text{state}}(D) \right) */
        constraint def Invariant_REQ_0051_The_composite_state_hash_satisfies;
        doc /* \begin{aligned} \forall (U, V) \in E &\implies \pi(U) \lt \pi(V) \\ H_{\text{state}}(M) = H_{\text{cached}}(M) &\implies \text{ReuseCachedArtifacts}(M) \quad \land \quad T_{\text{recompile}}(M) = 0 \end{aligned} */
        constraint def Invariant_REQ_0051_Topological_Order_and_Cache_Bypass_Invariants;
        doc /* H_{\text{source}}(M) \neq H_{\text{cached\_source}}(M) \implies \forall D \in \text{TransitiveDependents}(M), \quad \text{InvalidateCache}(D) */
        constraint def Invariant_REQ_0051_Downstream_Transitive_Invalidation_Invariant;
        doc /* \mathcal{R} = \bigcup_{M \in \mathcal{M}_{\text{mut}}} \left( \{ M \} \cup \text{TransitiveDependents}(M) \right) = \{ M \in V \mid H_{\text{state}}(M) \neq H_{\text{cached}}(M) \} */
        constraint def Invariant_REQ_0051_Recomputation_Set_Equivalence;
        doc /* \begin{aligned} \mathcal{A}_{\text{inc}} \equiv \mathcal{A}_{\text{clean}} &\quad \land \quad \text{MerkleRoot}(\mathcal{A}_{\text{inc}}) = \text{MerkleRoot}(\mathcal{A}_{\text{clean}}) \\ \forall v \in V, \quad \neg (v &\to^+ v) \\ \text{Commit}(\mathcal{M}_{\text{staged}}) &\implies (\mathcal{M}_{\text{manifest}} \leftarrow \mathcal{M}_{\text{staged}}) \end{aligned} */
        constraint def Invariant_REQ_0051_Deterministic_Merkle_Parity_and_Atomic_Commit_Invariants;
        require Invariant_REQ_0051_The_transitive_downstream_dependent_set_is_defined_by_forward_path_reachability;
        require Invariant_REQ_0051_The_composite_state_hash_satisfies;
        require Invariant_REQ_0051_Topological_Order_and_Cache_Bypass_Invariants;
        require Invariant_REQ_0051_Downstream_Transitive_Invalidation_Invariant;
        require Invariant_REQ_0051_Recomputation_Set_Equivalence;
        require Invariant_REQ_0051_Deterministic_Merkle_Parity_and_Atomic_Commit_Invariants;
        verify by AC_01_Leaf_Module_Mutation_and_Isolated_Recompilation;
        verify by AC_02_Root_Module_Mutation_and_Transitive_Downstream_Invalidation;
        verify by AC_03_Clean_No_Op_Incremental_Cache_Hit_Bypass;
        verify by AC_04_Circular_Module_Dependency_Rejection;
        satisfy by NodeArenaASTGraphEngine;
    }
    doc /* UUIDv5: 7341d8f8-89e8-5c16-9b39-fa7fe1e24050 | Complexity: Class P (O(1)) | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0200 */
    requirement def REQ_0052_Memory_Mapped_Diff_Engine {
        id = "REQ-0052";
        text = "The persistence engine shall perform read-only memory-mapped diffing of candidate artifact byte buffers against on-disk files prior to writing. It shall bypass file emission when generated content matches on-disk bytes identically, preserving modification timestamps (mtime), inodes, and metadata";
        attribute uuidv5 : String = "7341d8f8-89e8-5c16-9b39-fa7fe1e24050";
        attribute complexity_class : String = "Class P (O(1))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0200";
        attribute ac_01_timestamp_invariance_on_identical_re_emission : String = "Given: Target output path P containing an existing on-disk artifact with modification timestamp t_0, and freshly generated byte buffer B_{new} with identical content. When: The persistence engine processes artifact P during compilation pass t_1 t_0. Then: ShouldWrite(P, B_{new}) evaluates to false, zero write system calls execute, and mtime(P, t_1) = t_0. Diagnostic: E0200 emitted on unexpected filesystem error.";
        attribute ac_02_size_discrepancy_fast_short_circuit : String = "Given: Target path P where FileSize(P) |B_{new}|. When: The persistence engine evaluates the emission decision predicate. Then: ShouldWrite(P, B_{new}) evaluates to true in O(1) time, memory mapping is strictly bypassed (MmapCreated(P) = false), and the file is atomically rewritten. Diagnostic: E0200 emitted on atomic commit failure.";
        attribute ac_03_single_byte_divergence_detection_and_atomic_commit : String = "Given: On-disk file P and candidate buffer B_{new} of identical length L, differing by 1 byte at index k {0, L/2 , L-1}. When: Vectorized memory-mapped comparison evaluates B_{mmap}(P) against B_{new}. Then: Divergence is detected at index k, ShouldWrite(P, B_{new}) evaluates to true, and B_{new} is persisted atomically via a temporary sibling file. Diagnostic: E0200 emitted if atomic rename fails.";
        attribute ac_04_non_existent_target_atomic_creation : String = "Given: Target path P in namespace Package_0 where FileExists(P) = false. When: The persistence engine processes output artifact P. Then: ShouldWrite(P, B_{new}) evaluates to true without memory mapping, creating the destination artifact atomically. Diagnostic: E0200 emitted on directory creation or permission error.";
        doc /* \text{ShouldWrite}(P, B_{\text{new}}) = \begin{cases} \text{true}, & \text{if } \neg \text{FileExists}(P) \\ \text{true}, & \text{if } \text{FileExists}(P) \wedge \text{FileSize}(P) \ne |B_{\text{new}}| \\ \text{false}, & \text{if } \text{FileExists}(P) \wedge \text{FileSize}(P) = |B_{\text{new}}| \wedge \mathcal{B}_{\text{mmap}}(P) = B_{\text{new}} \\ \text{true}, & \text{otherwise} \end{cases} */
        constraint def Invariant_REQ_0052_The_emission_decision_function;
        doc /* \begin{aligned} \text{ShouldWrite}(P, B_{\text{new}}) = \text{false} \implies &\; \text{mtime}(P, t+1) = \text{mtime}(P, t) \;\wedge \\ &\; \text{inode}(P, t+1) = \text{inode}(P, t) \;\wedge \\ &\; \forall T \in \mathcal{T}_{\text{build}}, \; \text{Rebuild}(T) = \text{false} \end{aligned} */
        constraint def Invariant_REQ_0052_When_emission_is_bypassed_filesystem_attributes_remain_invariant_and_downstream_build_tasks_require_zero_rebuilding;
        doc /* \begin{aligned} \text{FileSize}(P) \ne |B_{\text{new}}| &\implies \text{MmapCreated}(P) = \text{false} \\ \text{HeapAllocations}(\text{DiffEngine}(P, B_{\text{new}})) &= 0 \end{aligned} */
        constraint def Invariant_REQ_0052_Size_divergence_bypasses_virtual_memory_mapping_and_diffing_executes_with_zero_dynamic_heap_allocations;
        doc /* \text{Fault}(P) \in \{ \text{PermissionDenied}, \text{MappingFailed}, \text{IOError} \} \implies \text{Diagnostic}(P, \text{E0200}) \in \mathcal{D} */
        constraint def Invariant_REQ_0052_Filesystem_permission_faults_or_mapping_failures_emit_diagnostic_E0200;
        require Invariant_REQ_0052_The_emission_decision_function;
        require Invariant_REQ_0052_When_emission_is_bypassed_filesystem_attributes_remain_invariant_and_downstream_build_tasks_require_zero_rebuilding;
        require Invariant_REQ_0052_Size_divergence_bypasses_virtual_memory_mapping_and_diffing_executes_with_zero_dynamic_heap_allocations;
        require Invariant_REQ_0052_Filesystem_permission_faults_or_mapping_failures_emit_diagnostic_E0200;
        verify by AC_01_Timestamp_Invariance_on_Identical_Re_Emission;
        verify by AC_02_Size_Discrepancy_Fast_Short_Circuit;
        verify by AC_03_Single_Byte_Divergence_Detection_and_Atomic_Commit;
        verify by AC_04_Non_Existent_Target_Atomic_Creation;
        satisfy by NodeArenaASTGraphEngine;
    }
    doc /* UUIDv5: 9ab78e33-0ab5-5c52-9948-b11a4487080f | Complexity: Class P (O(N_{prev} + N_{act})) | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0200 */
    requirement def REQ_0053_Tombstone_Pruning_Engine_for_Renamed {
        id = "REQ-0053";
        text = "The compilation persistence engine shall maintain a centralized artifact manifest (CodegenManifest) and execute automated tombstone pruning (prune_tombstones) during each compilation pass. When any AST element (such as Package_0, Classifier_Alpha, or Port_1) is deleted, renamed, or relocated, the";
        attribute uuidv5 : String = "9ab78e33-0ab5-5c52-9948-b11a4487080f";
        attribute complexity_class : String = "Class P (O(N_{prev} + N_{act}))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0200";
        attribute ac_01_classifier_renaming_and_tombstone_pruning : String = "Given: Model containing Package_0::Classifier_Alpha with manifested artifacts A_{manifest}, where Classifier_Alpha is renamed to Package_0::Classifier_Beta. When: Code generation invokes prune_tombstones. Then: Obsolete files for Classifier_Alpha are added to T_{prune} and unlinked (FileExists false), new files exist, and A_{manifest}^{updated} records strictly active files. Diagnostic: E0200 emitted on unlinking failure or residual obsolete files.";
        attribute ac_02_element_deletion_and_recursive_directory_pruning : String = "Given: Model containing Package_0::SubPackage_1::Classifier_Alpha whose output directory generated/Package_0/SubPackage_1/ contains strictly files from Classifier_Alpha. When: Classifier_Alpha is deleted and compilation executes. Then: All generated files are unlinked, empty directory generated/Package_0/SubPackage_1/ is deleted, and non-empty ancestor generated/Package_0/ is preserved. Diagnostic: E0200 emitted on directory deletion failure.";
        attribute ac_03_workspace_output_confinement_and_traversal_rejection : String = "Given: A manifest entry containing traversal tokens (../Package_0/Escaped.sysml). When: prune_tombstones validates paths prior to unlinking. Then: Resolution rejects unconfined paths (IsDescendant false), zero unlinking syscalls are issued, and compilation proceeds. Diagnostic: E0200 emitted on traversal violation.";
        attribute ac_04_idempotent_unlinking_and_non_panicking_fault_tolerance : String = "Given: An orphaned file in T_{prune} that is missing or has read-only permissions (EACCES). When: prune_tombstones attempts unlinking. Then: Missing files succeed as idempotent no-ops; permission faults record E0200 and pruning continues without panicking. Diagnostic: E0200 emitted on filesystem permission or I/O failure.";
        doc /* \mathcal{P}_{\text{rel}} = \{ p \in \Sigma^* \mid \neg\text{HasTraversalPrefix}(p) \;\wedge\; \text{NormalizePath}(p) = p \} */
        constraint def Invariant_REQ_0053_FormalInvariant;
        doc /* \forall p \in \mathcal{P}_{\text{rel}}, \quad \text{IsDescendant}(\text{Resolve}(\mathcal{W}_{\text{out}}, p), \mathcal{W}_{\text{out}}) \equiv \text{true} */
        constraint def Invariant_REQ_0053_Output_confinement_requires;
        doc /* \mathcal{T}_{\text{prune}} = \mathcal{A}_{\text{manifest}} \setminus \mathcal{A}_{\text{active}} */
        constraint def Invariant_REQ_0053_FormalInvariant_2;
        doc /* \text{UnlinkFile}(p) = \begin{cases} \text{true}, & \text{if } \neg\text{FileExists}(\text{Resolve}(\mathcal{W}_{\text{out}}, p)) \;\vee\; \text{Unlink}(\text{Resolve}(\mathcal{W}_{\text{out}}, p)) = \text{OK} \\ \text{false}, & \text{if } \text{Unlink}(\text{Resolve}(\mathcal{W}_{\text{out}}, p)) = \text{Err}(e) \end{cases} */
        constraint def Invariant_REQ_0053_Safe_idempotent_unlinking;
        doc /* \text{CleanEmptyParentDirs}(d) = \begin{cases} 0, & \text{if } d = \mathcal{W}_{\text{out}} \;\vee\; \neg\text{DirExists}(d) \;\vee\; |\text{Entries}(d)| \gt 0 \\ 1 + \text{CleanEmptyParentDirs}(\text{Parent}(d)), & \text{if } |\text{Entries}(d)| = 0 \;\wedge\; \text{Rmdir}(d) = \text{OK} \end{cases} */
        constraint def Invariant_REQ_0053_Recursive_directory_pruning;
        doc /* \begin{aligned} \forall p \in \mathcal{T}_{\text{prune}}, &\quad \text{FileExists}(\text{Resolve}(\mathcal{W}_{\text{out}}, p)) \equiv \text{false} \\ \pi_{\text{path}}(\mathcal{M}_{\text{updated}}) &\equiv \mathcal{A}_{\text{active}} \;\wedge\; \pi_{\text{path}}(\mathcal{M}_{\text{updated}}) \cap \mathcal{T}_{\text{prune}} = \emptyset \end{aligned} */
        constraint def Invariant_REQ_0053_Manifest_synchronization_and_tree_cleanliness_invariant;
        doc /* \neg\text{IsDescendant}(\text{Resolve}(\mathcal{W}_{\text{out}}, p), \mathcal{W}_{\text{out}}) \;\vee\; \text{UnlinkFile}(p) = \text{false} \implies \text{Diagnostic}(p, \text{E0200}) \in \mathcal{D}_{\text{sink}} */
        constraint def Invariant_REQ_0053_Diagnostic_error_binding;
        require Invariant_REQ_0053_FormalInvariant;
        require Invariant_REQ_0053_Output_confinement_requires;
        require Invariant_REQ_0053_FormalInvariant_2;
        require Invariant_REQ_0053_Safe_idempotent_unlinking;
        require Invariant_REQ_0053_Recursive_directory_pruning;
        require Invariant_REQ_0053_Manifest_synchronization_and_tree_cleanliness_invariant;
        require Invariant_REQ_0053_Diagnostic_error_binding;
        verify by AC_01_Classifier_Renaming_and_Tombstone_Pruning;
        verify by AC_02_Element_Deletion_and_Recursive_Directory_Pruning;
        verify by AC_03_Workspace_Output_Confinement_and_Traversal_Rejection;
        verify by AC_04_Idempotent_Unlinking_and_Non_Panicking_Fault_Tolerance;
        satisfy by NodeArenaASTGraphEngine;
    }
}
