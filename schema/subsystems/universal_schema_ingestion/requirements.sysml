doc /* Universal Schema Ingestion Engine */
package UniversalSchemaIngestion {
    doc /* UUIDv5: a5840107-5c1d-59b3-bdf6-acd7ea1d1e16 | Complexity: Class P (O(K) = O(1)) | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100 */
    requirement def REQ_0014_File_Format_Detection_Engine_by_Extension {
        id = "REQ-0014";
        text = "The compiler ingestion engine shall classify input schema files into target schema formats using file extensions and deterministic content prefix signatures across 5 supported formats: CommonMark Markdown tables (.md, .markdown), Protocol Buffers (.proto), OMG IDL (.idl), OpenAPI schemas (.json,";
        attribute uuidv5 : String = "a5840107-5c1d-59b3-bdf6-acd7ea1d1e16";
        attribute complexity_class : String = "Class P (O(K) = O(1))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100";
        attribute ac_01_multi_format_classification_precision : String = "Given: Canonical schema files covering all 5 supported formats containing synthetic MBSE declarations (Package_0, Classifier_Alpha, Port_1, Flow_A, param_x : Real). When: The ingestion engine executes format classification (F). Then: Each file is classified into its matching format (MarkdownTable, Protobuf, OmgIdl, OpenApi, XmlSchema) with 100% precision and zero false positives. Diagnostic: None emitted on valid format classification.";
        attribute ac_02_bounded_window_inspection_regex_prohibition : String = "Given: Schema input file F of size |F| 10,MB. When: The format detection engine inspects file F. Then: Exactly (|F|, K) 512 bytes are read from disk or buffer, static assertions confirm RegexEvaluations() = 0 and BacktrackCount() = 0, and execution terminates in O(1) time. Diagnostic: E0100 emitted if reading fails or content is unclassifiable.";
        attribute ac_03_unknown_corrupted_payload_diagnostic_binding : String = "Given: Input files with unrecognized extensions, empty files (|F| = 0), truncated fragments (|F| K), arbitrary binary blobs, or contradictory signatures. When: The classifier evaluates input file F. Then: (F) evaluates to Unknown, constructing diagnostic E0100 with byte span [0, (|F|, K)], and recovers to process subsequent compilation units without aborting. Diagnostic: E0100 emitted into diagnostic sink D.";
        doc /* \text{head}(F, K) = \langle b_{0}, \dots, b_{m-1} \rangle, \quad m = \min(|F|, K) */
        constraint def Invariant_REQ_0014_FormalInvariant;
        doc /* \mathcal{M} = \{ \text{MarkdownTable}, \text{Protobuf}, \text{OmgIdl}, \text{OpenApi}, \text{XmlSchema}, \text{Unknown} \} */
        constraint def Invariant_REQ_0014_The_target_format_domain_is_defined_as;
        doc /* \Delta(F) = \begin{cases} \text{MarkdownTable}, & \text{if } \text{ext}(F) \in \{\text{".md"}, \text{".markdown"}\} \wedge \text{HasMarkdownTable}(\text{head}(F, K)) \\ \text{Protobuf}, & \text{if } \text{ext}(F) = \text{".proto"} \vee \text{HasPrefix}(\text{head}(F, K), \text{"syntax = proto"}) \\ \text{OmgIdl}, & \text{if } \text{ext}(F) = \text{".idl"} \vee \text{HasKeyword}(\text{head}(F, K), \text{"module"}) \\ \text{OpenApi}, & \text{if } \text{ext}(F) \in \{\text{".json"}, \text{".yaml"}, \text{".yml"}\} \wedge \text{HasKeys}(\text{head}(F, K), \{\text{"openapi"}, \text{"swagger"}\}) \\ \text{XmlSchema}, & \text{if } \text{ext}(F) \in \{\text{".xml"}, \text{".xsd"}\} \wedge \text{HasXmlRoot}(\text{head}(F, K)) \\ \text{Unknown}, & \text{otherwise} \end{cases} */
        constraint def Invariant_REQ_0014_The_deterministic_format_classifier_function;
        doc /* \forall F \in \mathcal{F}, \quad \text{BytesInspected}(F) = \min(|F|, K) \le K = 512 */
        constraint def Invariant_REQ_0014_Bounded_inspection_zero_regex_and_zero_backtracking;
        doc /* \text{RegexEvaluations}(\Delta) = 0 \quad \wedge \quad \text{BacktrackCount}(\Delta) = 0 */
        constraint def Invariant_REQ_0014_FormalInvariant_2;
        doc /* \forall F \in \mathcal{F}, \quad \text{Panic}(\Delta(F)) \equiv \text{false} \quad \wedge \quad \Delta(F) \in \mathcal{M} */
        constraint def Invariant_REQ_0014_Total_function_determinism_and_diagnostic_emission;
        doc /* \Delta(F) = \text{Unknown} \implies \mathcal{D}_{\text{emit}}(F) = \langle \text{E0100}, \text{path}(F), \text{span}(0, \min(|F|, K)) \rangle */
        constraint def Invariant_REQ_0014_FormalInvariant_3;
        doc /* \forall D \in \mathcal{D}_{\text{format}}, \quad \text{code}(D) \in \mathcal{E}_{\text{ingest}} = \{ e \in \mathbb{N} \mid 100 \le e \le 199 \} */
        constraint def Invariant_REQ_0014_FormalInvariant_4;
        require Invariant_REQ_0014_FormalInvariant;
        require Invariant_REQ_0014_The_target_format_domain_is_defined_as;
        require Invariant_REQ_0014_The_deterministic_format_classifier_function;
        require Invariant_REQ_0014_Bounded_inspection_zero_regex_and_zero_backtracking;
        require Invariant_REQ_0014_FormalInvariant_2;
        require Invariant_REQ_0014_Total_function_determinism_and_diagnostic_emission;
        require Invariant_REQ_0014_FormalInvariant_3;
        require Invariant_REQ_0014_FormalInvariant_4;
        verify by AC_01_Multi_Format_Classification_Precision;
        verify by AC_02_Bounded_Window_Inspection_Regex_Prohibition;
        verify by AC_03_Unknown_Corrupted_Payload_Diagnostic_Binding;
        satisfy by UniversalIngestionEngine;
    }
    doc /* UUIDv5: d51a9578-d15d-5631-893d-786553664acf | Complexity: Class P (O(1)) | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100, E0102 */
    requirement def REQ_0015_Event_Driven_CommonMark_Table_Lexer {
        id = "REQ-0015";
        text = "The ingestion engine shall parse CommonMark Markdown tables containing component declarations, interfaces, attributes, and constraints using an explicit deterministic pushdown automaton (DPDA) that emits an event stream of structural tokens. Lexing and tokenization must strictly prohibit regular";
        attribute uuidv5 : String = "d51a9578-d15d-5631-893d-786553664acf";
        attribute complexity_class : String = "Class P (O(1))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100, E0102";
        attribute ac_01_deterministic_event_driven_streaming_tokenization : String = "Given: A valid CommonMark table declaring MBSE structural entities (Package_0, Classifier_Alpha, Port_1, Flow_A, param_x : Real) encoded as a contiguous UTF-8 byte stream. When: The table lexer M_{table} executes the lexical analysis pass. Then: The lexer emits a balanced stream of structural tokens S_{events} on-the-fly via the streaming sink, satisfying count(StartCell, S_{events}) = count(EndCell, S_{events}) and count(StartRow, S_{events}) = count(EndRow, S_{events}) without intermediate full-tree buffering. Diagnostic: E0100 emitted on token stream generation failure or unbalanced.";
        attribute ac_02_zero_backtracking_and_zero_regular_expression_execution : String = "Given: An input schema file containing CommonMark Markdown tables. When: Lexical analysis and token classification are executed across the byte stream. Then: Byte scanning advances monotonically (pos_{t+1} pos_t) with zero backtracking (BacktrackCount = 0) and zero regular expression evaluations (RegexEvaluations = 0), executing purely via scalar byte-matching transitions. Diagnostic: E0100 emitted on non-monotonic stream cursor rollback or regular expression library invocation.";
        attribute ac_03_bounded_pushdown_stack_depth_invariant : String = "Given: A deeply nested or complex tabular markdown sequence. When: The pushdown automaton stack transitions are evaluated during table parsing. Then: The pushdown stack depth remains strictly bounded by |stack_t| D 4 across all evaluation steps t, rejecting unclosed frames or exceeding depths deterministically. Diagnostic: E0102 emitted if table nesting depth exceeds 4 or stack underflow occurs.";
        attribute ac_04_total_function_non_panicking_contract_and_delimiter_synchronization : String = "Given: A malformed table input containing unclosed cells, illegal byte sequences, or truncated row delimiter lines. When: The lexer encounters a lexical anomaly or delimiter mismatch. Then: The engine terminates without panicking, emits a structured diagnostic with exact byte span and line/column coordinates, and resynchronizes to the next delimiter boundary. Diagnostic: E0102 emitted for table delimiter or cell framing anomalies.";
        doc /* M_{\text{table}} = (Q, \Sigma, \Gamma, \delta, q_0, Z_0, F) */
        constraint def Invariant_REQ_0015_The_CommonMark_table_lexer_is_formalized_as_a_7_tuple_deterministic_pushdown_automaton;
        doc /* \begin{aligned} Q &= \{ q_{\text{scan}}, q_{\text{header}}, q_{\text{delimiter}}, q_{\text{row}}, q_{\text{cell}}, q_{\text{escape}}, q_{\text{halt}} \} \\ \Sigma &= \{ b \in \mathbb{N} \mid 0 \le b \le 255 \} \\ \Gamma &= \{ Z_0, \text{TableFrame}, \text{HeaderFrame}, \text{RowFrame}, \text{CellFrame} \} \\ q_0 &= q_{\text{scan}}, \quad Z_0 \in \Gamma, \quad F = \{ q_{\text{halt}} \} \end{aligned} */
        constraint def Invariant_REQ_0015_where;
        doc /* \delta : Q \times (\Sigma \cup \{ \epsilon \}) \times \Gamma \longrightarrow Q \times \Gamma^* */
        constraint def Invariant_REQ_0015_The_deterministic_transition_function_satisfies;
        doc /* \forall q \in Q, \; \forall \gamma \in \Gamma, \; (\delta(q, \epsilon, \gamma) \neq \emptyset \implies \forall a \in \Sigma, \; \delta(q, a, \gamma) = \emptyset) \quad \wedge \quad |\delta(q, a, \gamma)| \le 1 */
        constraint def Invariant_REQ_0015_Determinism_and_transition_cardinality;
        doc /* \text{tag} \in \{ \text{StartTable}, \text{StartHeader}, \text{EndHeader}, \text{StartRow}, \text{EndRow}, \text{StartCell}, \text{CellText}, \text{EndCell}, \text{EndTable} \} */
        constraint def Invariant_REQ_0015_The_token_stream_emits_tuples;
        doc /* \text{BacktrackCount}(M_{\text{table}}) = 0, \quad \text{RegexEvaluations}(M_{\text{table}}) = 0 */
        constraint def Invariant_REQ_0015_Zero_Backtracking_and_Zero_Regex_Invariant;
        doc /* \forall t \in \mathbb{N}, \quad \text{pos}_{t+1} \ge \text{pos}_t \quad \wedge \quad |\text{stack}_t| \le D \le 4 */
        constraint def Invariant_REQ_0015_Monotonic_Forward_Progress_and_Bounded_Stack_Invariant;
        doc /* \begin{aligned} \text{count}(\text{StartTable}, \mathcal{S}_{\text{events}}) &= \text{count}(\text{EndTable}, \mathcal{S}_{\text{events}}) \\ \text{count}(\text{StartHeader}, \mathcal{S}_{\text{events}}) &= \text{count}(\text{EndHeader}, \mathcal{S}_{\text{events}}) \\ \text{count}(\text{StartRow}, \mathcal{S}_{\text{events}}) &= \text{count}(\text{EndRow}, \mathcal{S}_{\text{events}}) \\ \text{count}(\text{StartCell}, \mathcal{S}_{\text{events}}) &= \text{count}(\text{EndCell}, \mathcal{S}_{\text{events}}) \end{aligned} */
        constraint def Invariant_REQ_0015_Token_Stream_Structural_Balance_Invariant;
        doc /* \forall D_k \in \mathcal{D}_{\text{lexer}}, \quad \text{code}(D_k) \in \{ \text{E0100}, \text{E0102} \} */
        constraint def Invariant_REQ_0015_Diagnostic_Code_Containment_Invariant;
        require Invariant_REQ_0015_The_CommonMark_table_lexer_is_formalized_as_a_7_tuple_deterministic_pushdown_automaton;
        require Invariant_REQ_0015_where;
        require Invariant_REQ_0015_The_deterministic_transition_function_satisfies;
        require Invariant_REQ_0015_Determinism_and_transition_cardinality;
        require Invariant_REQ_0015_The_token_stream_emits_tuples;
        require Invariant_REQ_0015_Zero_Backtracking_and_Zero_Regex_Invariant;
        require Invariant_REQ_0015_Monotonic_Forward_Progress_and_Bounded_Stack_Invariant;
        require Invariant_REQ_0015_Token_Stream_Structural_Balance_Invariant;
        require Invariant_REQ_0015_Diagnostic_Code_Containment_Invariant;
        verify by AC_01_Deterministic_Event_Driven_Streaming_Tokenization;
        verify by AC_02_Zero_Backtracking_and_Zero_Regular_Expression_Execution;
        verify by AC_03_Bounded_Pushdown_Stack_Depth_Invariant;
        verify by AC_04_Total_Function_Non_Panicking_Contract_and_Delimiter_Synchronization;
        satisfy by UniversalIngestionEngine;
    }
    doc /* UUIDv5: 9669c5f2-4c8c-5b0e-b489-c8e5fb38258d | Complexity: Class P (O(1)) | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0102, E0199 */
    requirement def REQ_0016_Synchronization_Token_Recovery_on_Table_Delimiter_Boundaries {
        id = "REQ-0016";
        text = "The CommonMark table parser within the Universal Schema Ingestion Engine shall implement Synchronization Token Error Recovery on table delimiter boundaries per Architectural Directive 4. When encountering unexpected formatting, unescaped pipe characters, malformed scalar literals, or delimiter";
        attribute uuidv5 : String = "9669c5f2-4c8c-5b0e-b489-c8e5fb38258d";
        attribute complexity_class : String = "Class P (O(1))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0102, E0199";
        attribute ac_01_multi_error_resynchronization_across_row_boundaries : String = "Given: 5-row CommonMark table with entities (Package_0, Classifier_Alpha, Port_1, Flow_A, param_x : Real) and syntax faults in rows 2 and 4. When: The parser ingests the table in a single compilation pass. Then: Rows 1, 3, and 5 lower into valid AST nodes, rows 2 and 4 retain valid cells with _{cell} at faulted positions, and exactly 2 diagnostics are emitted. Diagnostic: Emits localized E0102 diagnostics with 1-indexed coordinates and byte spans.";
        attribute ac_02_strictly_monotonic_stream_progress_loop_prevention : String = "Given: Corrupt table input containing consecutive invalid delimiter sequences and unmatched pipes. When: The parser triggers error recovery upon syntax faults. Then: Stream position satisfies ByteOffset_{t+1} ByteOffset_t, terminating in at most N + 1 steps without livelocks. Diagnostic: E0102 emitted on recovery transitions; fail-closed termination if the pointer fails to advance.";
        attribute ac_03_cascading_error_suppression_proximity_window : String = "Given: Primary delimiter fault at byte _{last} followed by secondary faults at distance q - _{last} ( = 3). When: The parser evaluates tokens following a synchronization recovery event. Then: Secondary faults within window are suppressed from diagnostic sink D. Diagnostic: Primary fault records E0102; secondary faults within 3 emit zero duplicate diagnostics.";
        attribute ac_04_bounded_diagnostic_sink_capacity_graceful_termination : String = "Given: Degenerate table schema containing M C_{max_errors}} syntax faults. When: Diagnostic accumulation reaches capacity limit C_{max_errors}}. Then: The parser halts parsing, appends a final sentinel, and concludes ingestion gracefully. Diagnostic: Diagnostic E0199 emitted indicating error ceiling reached.";
        attribute ac_05_total_determinism_panic_free_non_zero_exit : String = "Given: Invalid CommonMark tables compiled in Release mode across varying platforms and worker thread allocations. When: The compiler executes delimiter synchronization recovery. Then: Execution completes under a total-function contract with zero unhandled panics and bitwise identical diagnostic output. Diagnostic: E0102 through E0199 emitted in structured format with zero panic unwinding.";
        doc /* \mathcal{T}_{\text{sync}} = \{ \text{SYM}_{\text{PIPE}}, \text{SYM}_{\text{LF}}, t_{\text{EOF}} \} */
        constraint def Invariant_REQ_0016_FormalInvariant;
        doc /* m = \text{ScanUntil}(\mathcal{T}_{\text{sync}}, p) = \min \{ k \in \{p, \dots, N\} \mid b_k \in \mathcal{T}_{\text{sync}} \} */
        constraint def Invariant_REQ_0016_For_a_syntax_fault_at_byte_offset_in_cell_scanning_identifies_delimiter_index;
        doc /* \text{State}_{t+1} = \begin{cases} \text{ParseCell}(R_i, j+1), & \text{if } b_m = \text{SYM}_{\text{PIPE}} \\ \text{ParseRow}(R_{i+1}, 1), & \text{if } b_m = \text{SYM}_{\text{LF}} \\ \text{Halt}, & \text{if } b_m = t_{\text{EOF}} \end{cases} */
        constraint def Invariant_REQ_0016_Parser_resynchronization_transition_satisfies;
        doc /* \text{ByteOffset}_{t+1} = \begin{cases} m + 1, & \text{if } b_m \in \{ \text{SYM}_{\text{PIPE}}, \text{SYM}_{\text{LF}} \} \\ N, & \text{if } b_m = t_{\text{EOF}} \end{cases} \quad \implies \quad \text{ByteOffset}_{t+1} \gt \text{ByteOffset}_t */
        constraint def Invariant_REQ_0016_Monotonic_stream_progress_mandates;
        doc /* \forall \sigma \in \text{ParseExecutions}(B), \quad \text{Steps}(\sigma) \le N + 1 */
        constraint def Invariant_REQ_0016_Strict_monotonicity_guarantees_bounded_termination;
        doc /* R_i = \langle C_{i, 1}, \dots, C_{i, j-1}, \bot_{\text{cell}}, C_{i, j+1}, \dots, C_{i, K_i} \rangle */
        constraint def Invariant_REQ_0016_Row_synthesis_with_sentinel_substitution;
        doc /* \forall i, k \in \{1, \dots, M\}, \quad (i \neq k \;\wedge\; \neg \text{Fault}(R_k)) \implies \text{Extracted}(R_k) */
        constraint def Invariant_REQ_0016_Row_extraction_independence;
        doc /* \text{EmitDecision}(q, \tau_{\text{last}}) = \begin{cases} \text{Suppress}, & \text{if } q - \tau_{\text{last}} \le \delta \\ \text{Record}, & \text{if } q - \tau_{\text{last}} \gt \delta \end{cases} \quad (\delta \le 3) */
        constraint def Invariant_REQ_0016_Cascading_error_suppression;
        doc /* |\mathcal{D}| \le C_{\text{max\_errors}}, \quad \forall D \in \mathcal{D}, \; \text{Code}(D) \in \{ \text{E0102}, \text{E0199} \} */
        constraint def Invariant_REQ_0016_Diagnostic_sink_bounds;
        require Invariant_REQ_0016_FormalInvariant;
        require Invariant_REQ_0016_For_a_syntax_fault_at_byte_offset_in_cell_scanning_identifies_delimiter_index;
        require Invariant_REQ_0016_Parser_resynchronization_transition_satisfies;
        require Invariant_REQ_0016_Monotonic_stream_progress_mandates;
        require Invariant_REQ_0016_Strict_monotonicity_guarantees_bounded_termination;
        require Invariant_REQ_0016_Row_synthesis_with_sentinel_substitution;
        require Invariant_REQ_0016_Row_extraction_independence;
        require Invariant_REQ_0016_Cascading_error_suppression;
        require Invariant_REQ_0016_Diagnostic_sink_bounds;
        derived from REQ_0191_Standardized_Compiler_Diagnostic_Taxonomy_Source_Mapping;
        verify by AC_01_Multi_Error_Resynchronization_Across_Row_Boundaries;
        verify by AC_02_Strictly_Monotonic_Stream_Progress_Loop_Prevention;
        verify by AC_03_Cascading_Error_Suppression_Proximity_Window;
        verify by AC_04_Bounded_Diagnostic_Sink_Capacity_Graceful_Termination;
        verify by AC_05_Total_Determinism_Panic_Free_Non_Zero_Exit;
        satisfy by UniversalIngestionEngine;
    }
    doc /* UUIDv5: 5ea507dc-aeda-56ff-8536-2f8238aa0ab7 | Complexity: Class P (O( | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0102 */
    requirement def REQ_0017_Dynamic_Semantic_Header_Key_Normalization {
        id = "REQ-0017";
        text = "The CommonMark tabular schema parser within the Universal Schema Ingestion Engine shall implement dynamic semantic header key normalization and order-independent column binding for all tabular model specifications. The parser shall accept column headers in arbitrary horizontal order and normalize";
        attribute uuidv5 : String = "5ea507dc-aeda-56ff-8536-2f8238aa0ab7";
        attribute complexity_class : String = "Class P (O(";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0102";
        attribute ac_01_order_independent_column_permutation_invariance : String = "Given: A CommonMark table defining component Classifier_Alpha with canonical columns Name, Type, Multiplicity, Description . When: The compiler ingests tables across all 4! = 24 column permutations (e.g., Description, Name, Type, Multiplicity ). Then: All 24 permuted tables synthesize bitwise identical AST entities with zero diff churn. Diagnostic: Deterministic AST generation; zero diagnostics emitted.";
        attribute ac_02_multi_stage_semantic_header_normalization_equivalence : String = "Given: Header strings with varied case (\"NAME\", \"name\"), whitespace (\" Name \"), punctuation (\"part-name\", \"part_name\", \"[Part Name]\"), and aliases (\"id\", \"identifier\", \"name\"). When: The parser executes normalization (H) and alias lookup. Then: All header variations resolve deterministically to identical canonical key k_{name}. Diagnostic: Successful resolution to k_{name}; zero diagnostics emitted.";
        attribute ac_03_injective_column_assignment_duplicate_header_collision_detection : String = "Given: A table containing duplicate column headers or distinct headers resolving to identical key k_{name} (e.g., columns \"Name\" and \"Identifier\"). When: The parser constructs binding vector . Then: Injective assignment fails, entity synthesis halts, synchronization token recovery engages, and a structured diagnostic is emitted. Diagnostic: Emits E0102 with colliding column indices, raw header tokens, and source span coordinates.";
        attribute ac_04_mandatory_coverage_bounded_column_arity_enforcement : String = "Given: A table omitting mandatory attribute k_{type} M(T_{entity}), or an unmapped header in strict mode, or a table exceeding C = 33 C_{max_cols}} columns. When: The parser inspects the table header row and validates binding vector . Then: The parser halts schema ingestion prior to row processing and emits a structured diagnostic without allocating unbounded buffers. Diagnostic: Emits E0102 identifying missing mandatory attribute, unmapped header, or column limit exceeded.";
        doc /* \text{CaseFold}(b) = \begin{cases} b + 0\text{x}20, & \text{if } 0\text{x}41 \le b \le 0\text{x}5\text{A} \\ b, & \text{otherwise} \end{cases} */
        constraint def Invariant_REQ_0017_ASCII_case_folding;
        doc /* \eta(H) = \text{CaseFold}(\text{StripPunct}(\text{Trim}(H))) */
        constraint def Invariant_REQ_0017_The_punctuation_stripping_operator;
        doc /* \forall k_a \ne k_b \in \mathcal{K}_{\text{canonical}}, \quad \text{Aliases}(k_a) \cap \text{Aliases}(k_b) = \emptyset */
        constraint def Invariant_REQ_0017_For_canonical_keys_with_pairwise_disjoint_aliases;
        doc /* \beta(H) = \begin{cases} k, & \text{if } \exists ! \, k \in \mathcal{K}_{\text{canonical}} \text{ s.t. } \eta(H) \in \text{Aliases}(k) \\ \bot, & \text{otherwise} \end{cases} */
        constraint def Invariant_REQ_0017_Binding_function;
        doc /* \begin{aligned} \forall H \in \mathbf{H}, \quad \beta(H) \ne \bot &\implies |\{ k \in \mathcal{K}_{\text{canonical}} \mid \eta(H) \in \text{Aliases}(k) \}| = 1 \\ \forall i \ne j \in \{0, \dots, C-1\}, \quad (\beta(H_i) \ne \bot \;\wedge\; \beta(H_j) \ne \bot) &\implies \beta(H_i) \ne \beta(H_j) \\ \mathcal{M}(T_{\text{entity}}) &\subseteq \{ \beta(H_j) \mid 0 \le j \lt C \} \end{aligned} */
        constraint def Invariant_REQ_0017_Deterministic_Resolution_Injective_Assignment_and_Mandatory_Coverage;
        doc /* \mathcal{E}(\mathbf{H}, \mathbf{r}) = \{ (\beta(H_j), c_j) \mid 0 \le j \lt C \;\wedge\; \beta(H_j) \ne \bot \} */
        constraint def Invariant_REQ_0017_For_data_row_synthesized_attributes_and_permutation_invariance_satisfy;
        doc /* \forall \pi \in \mathfrak{S}_C, \quad \mathcal{E}(\pi(\mathbf{H}), \pi(\mathbf{r})) \equiv \mathcal{E}(\mathbf{H}, \mathbf{r}) \implies \text{SynthesizeAST}(\pi(\mathcal{T})) \equiv \text{SynthesizeAST}(\mathcal{T}) */
        constraint def Invariant_REQ_0017_FormalInvariant;
        doc /* \begin{aligned} (\exists i \ne j \text{ s.t. } \beta(H_i) = \beta(H_j) \ne \bot) &\implies \text{Diagnostic}(R_0, C_j, \text{E0102}) \in \mathcal{D} \\ (\mathcal{M}(T_{\text{entity}}) \setminus \{ \beta(H_j) \}_{j=0}^{C-1} \ne \emptyset) &\implies \text{Diagnostic}(R_0, 0, \text{E0102}) \in \mathcal{D} \\ (\exists j \text{ s.t. } \beta(H_j) = \bot \;\wedge\; \text{Strict}(\mathcal{S})) &\implies \text{Diagnostic}(R_0, C_j, \text{E0102}) \in \mathcal{D} \\ (C \gt C_{\text{max\_cols}}) &\implies \text{Diagnostic}(R_0, C, \text{E0102}) \in \mathcal{D} \end{aligned} */
        constraint def Invariant_REQ_0017_Diagnostic_Error_Code_Binding_Invariant;
        require Invariant_REQ_0017_ASCII_case_folding;
        require Invariant_REQ_0017_The_punctuation_stripping_operator;
        require Invariant_REQ_0017_For_canonical_keys_with_pairwise_disjoint_aliases;
        require Invariant_REQ_0017_Binding_function;
        require Invariant_REQ_0017_Deterministic_Resolution_Injective_Assignment_and_Mandatory_Coverage;
        require Invariant_REQ_0017_For_data_row_synthesized_attributes_and_permutation_invariance_satisfy;
        require Invariant_REQ_0017_FormalInvariant;
        require Invariant_REQ_0017_Diagnostic_Error_Code_Binding_Invariant;
        derived from REQ_0016_Synchronization_Token_Recovery_on_Table_Delimiter_Boundaries;
        verify by AC_01_Order_Independent_Column_Permutation_Invariance;
        verify by AC_02_Multi_Stage_Semantic_Header_Normalization_Equivalence;
        verify by AC_03_Injective_Column_Assignment_Duplicate_Header_Collision_Detection;
        verify by AC_04_Mandatory_Coverage_Bounded_Column_Arity_Enforcement;
        satisfy by UniversalIngestionEngine;
    }
    doc /* UUIDv5: ccce39a3-18dd-539a-8528-1eddf58848cf | Complexity: Class P (O(1)) | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100, E0102 */
    requirement def REQ_0018_Multi_Line_Table_Cell_Block_Token_Preservation {
        id = "REQ-0018";
        text = "The CommonMark table parser within the Universal Schema Ingestion Engine shall preserve multi-line cell content, nested bullet points, list items, and inline code spans within table cells without dropping formatting, truncating text, or losing structural hierarchy. The engine must normalize";
        attribute uuidv5 : String = "ccce39a3-18dd-539a-8528-1eddf58848cf";
        attribute complexity_class : String = "Class P (O(1))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100, E0102";
        attribute ac_01_html_line_break_variant_and_escaped_linefeed_normalization : String = "Given: CommonMark table cell C_{raw} containing break variants (<br>, <br/>, <br />, <BR>, <BR/>, <BR />) and escaped linefeeds (, ) around text tokens (param_x : Real). When: Block normalizer pass _{block} executes during table ingestion. Then: Every break sequence normalizes to LineBreak(SYM_{LF}), preserving surrounding text without loss. Diagnostic: E0100 emitted on invalid escape sequences or corrupted break delimiters.";
        attribute ac_02_verbatim_code_span_preservation : String = "Given: Table cell containing an inline code span enclosed in backticks ( Port_1 <br> Port_2 | Flow_A ). When: Block normalizer encounters code span delimiters during cell lexing. Then: Content is emitted verbatim as a single CodeSpan token without normalizing embedded HTML breaks or splitting on pipes. Diagnostic: E0102 emitted if an unclosed code span reaches column delimiter pipe (|).";
        attribute ac_03_nested_list_item_hierarchy_and_bullet_depth_extraction : String = "Given: Table cell containing multi-line bullet items with space indentation (- Item_Alpha<br> - Subitem_Beta<br> - Subitem_Gamma). When: Normalizer evaluates leading whitespace following a normalized line break. Then: Items lower into ordered ListItem tokens with indentation mapped to bullet depth d {1, 2, 3} and marker kinds preserved. Diagnostic: E0100 emitted if bullet nesting depth exceeds bound D_{max_bullet}} = 8.";
        attribute ac_04_bijective_round_trip_serialization : String = "Given: Normalized token sequence T = _{block}(C_{raw}) from a schema cell defining synthetic ports (Port_1, Port_2). When: Serialization pass emits T via _{emit} and re-ingests the resulting string via _{block}. Then: Round-trip identity _{block}(_{emit}(T)) T holds with 0-byte diff churn. Diagnostic: E0100 emitted on round-trip token divergence or information loss.";
        doc /* \Phi_{\text{block}} : \Sigma^* \longrightarrow \mathcal{T}_{\text{block}}^*, \quad \mathcal{T}_{\text{block}} = \{ \text{TextRun}(s), \text{LineBreak}(\text{SYM}_{\text{LF}}), \text{ListItem}(d, m, s), \text{CodeSpan}(s), \bot_{\text{block}} \} */
        constraint def Invariant_REQ_0018_FormalInvariant;
        doc /* \mathcal{P}_{\text{break}} = \{ \langle \text{0x3C}, \text{0x62}, \text{0x72}, \text{0x3E} \rangle, \langle \text{0x3C}, \text{0x62}, \text{0x72}, \text{0x2F}, \text{0x3E} \rangle, \langle \text{0x3C}, \text{0x62}, \text{0x72}, \text{0x20}, \text{0x2F}, \text{0x3E} \rangle, \langle \text{0x5C}, \text{0x6E} \rangle, \langle \text{0x5C}, \text{0x72}, \text{0x5C}, \text{0x6E} \rangle \} */
        constraint def Invariant_REQ_0018_FormalInvariant_2;
        doc /* \text{MatchBreak}(C_{\text{raw}}, p) = \begin{cases} \pi, & \text{if } \exists \pi \in \mathcal{P}_{\text{break}} \text{ such that } \forall k \in \{0, \dots, |\pi|-1\}, \, C_{\text{raw}}[p+k] = \pi[k] \\ \emptyset, & \text{otherwise} \end{cases} */
        constraint def Invariant_REQ_0018_The_deterministic_break_detection_operator_evaluates_at_byte_offset;
        doc /* \forall p \in [p_{\text{open}}, p_{\text{close}}], \quad \text{MatchBreak}(C_{\text{raw}}, p) \implies \text{Suppressed} */
        constraint def Invariant_REQ_0018_Within_an_active_backtick_code_span_interval_break_matching_is_strictly_suppressed;
        doc /* \text{Slice}(\text{CodeSpan}(s)) = \langle C_{\text{raw}}[p_{\text{open}} + 1], \dots, C_{\text{raw}}[p_{\text{close}} - 1] \rangle */
        constraint def Invariant_REQ_0018_FormalInvariant_3;
        doc /* d = \min \left( \left\lfloor \frac{S}{2} \right\rfloor + 1, \; D_{\text{max\_bullet}} \right) */
        constraint def Invariant_REQ_0018_In_line_leading_state_leading_whitespace_byte_count_determines_bullet_nesting_depth;
        doc /* \begin{aligned} \text{RegexEvaluations}(\Phi_{\text{block}}) &= 0 \\ \text{BacktrackCount}(\Phi_{\text{block}}) &= 0 \\ \forall t \in \mathbb{N}_0, \quad \text{pos}_{t+1} &\gt \text{pos}_t \end{aligned} */
        constraint def Invariant_REQ_0018_The_normalizer_enforces_the_Zero_Regex_Zero_Backtracking_and_Monotonic_Progress_Invariants;
        doc /* \Phi_{\text{block}}(\Psi_{\text{emit}}(\mathcal{T})) \equiv \mathcal{T} \quad \wedge \quad \forall D \in \mathcal{D}_{\text{cell}}, \; \text{Code}(D) \in \{ \text{E0100}, \text{E0102} \} */
        constraint def Invariant_REQ_0018_Bijective_Round_Trip_Equivalence_and_Diagnostic_Containment_Invariants;
        require Invariant_REQ_0018_FormalInvariant;
        require Invariant_REQ_0018_FormalInvariant_2;
        require Invariant_REQ_0018_The_deterministic_break_detection_operator_evaluates_at_byte_offset;
        require Invariant_REQ_0018_Within_an_active_backtick_code_span_interval_break_matching_is_strictly_suppressed;
        require Invariant_REQ_0018_FormalInvariant_3;
        require Invariant_REQ_0018_In_line_leading_state_leading_whitespace_byte_count_determines_bullet_nesting_depth;
        require Invariant_REQ_0018_The_normalizer_enforces_the_Zero_Regex_Zero_Backtracking_and_Monotonic_Progress_Invariants;
        require Invariant_REQ_0018_Bijective_Round_Trip_Equivalence_and_Diagnostic_Containment_Invariants;
        derived from REQ_0191_Standardized_Compiler_Diagnostic_Taxonomy_Source_Mapping;
        verify by AC_01_HTML_Line_Break_Variant_and_Escaped_Linefeed_Normalization;
        verify by AC_02_Verbatim_Code_Span_Preservation;
        verify by AC_03_Nested_List_Item_Hierarchy_and_Bullet_Depth_Extraction;
        verify by AC_04_Bijective_Round_Trip_Serialization;
        satisfy by UniversalIngestionEngine;
    }
    doc /* UUIDv5: 1519acae-b878-52ea-b0cf-f40957205d2e | Complexity: Class P (O( | Standard: IEEE 29148-2018 / RFC 2119 / OMG SysML v2 | Diagnostics: E0102 */
    requirement def REQ_0019_Table_Row_to_Entity_Structural_Mapping {
        id = "REQ-0019";
        text = "The CommonMark table parser within the Universal Schema Ingestion Engine shall map each valid table row to a corresponding intermediate representation (IR) entity in contiguous memory storage while sanitizing raw entity names into valid OMG SysML v2 and KerML basic identifiers using deterministic";
        attribute uuidv5 : String = "1519acae-b878-52ea-b0cf-f40957205d2e";
        attribute complexity_class : String = "Class P (O(";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119 / OMG SysML v2";
        attribute diagnostic_codes : String = "E0102";
        attribute ac_01_valid_basic_identifier_passthrough : String = "Given: A table row declaring an entity with raw name cell S_{raw} = \"Classifier_Alpha\"} conforming to BasicIdentifier. When: The ingestion engine executes identifier sanitization _{id}(S_{raw}). Then: The identifier passes through unaltered with _{id}(S_{raw}) = \"Classifier_Alpha\"}. Diagnostic: Diagnostic E0102 emitted if a valid basic identifier is mutated or rejected.";
        attribute ac_02_non_alphanumeric_character_sanitization_leading_digit_prefixing : String = "Given: A table row with raw name cell S_{raw} = \"123-Alpha (Beta)\"}. When: The ingestion engine executes the sanitization transformation pipeline. Then: The sanitized identifier equals \"id_123_Alpha_Beta_\"}, confirming underscore mapping, delimiter collapse, id_ prefixing, and trailing underscore retention. Diagnostic: Diagnostic E0102 emitted if non-alphanumeric characters or leading digits fail canonical sanitization.";
        attribute ac_03_scope_sibling_collision_disambiguation : String = "Given: An input table containing three distinct rows declaring entities with identical raw name S_{raw} = \"Port_1\"} in parent scope Package_0. When: The engine processes rows sequentially and executes collision resolution Disambiguate. Then: The engine assigns distinct identifiers \"Port_1\"}, \"Port_1_2\"}, and \"Port_1_3\"}, ensuring scope injectivity. Diagnostic: Diagnostic E0102 emitted on unhandled identifier collision or non-deterministic disambiguation index assignment.";
        attribute ac_04_reserved_keyword_disambiguation : String = "Given: A table row declaring an entity whose raw name matches reserved keyword S_{raw} = \"part\"}. When: The ingestion engine evaluates S_{raw} against K_{reserved} and executes sanitization. Then: The engine prepends id_, producing sanitized identifier \"id_part\"}. Diagnostic: Diagnostic E0102 emitted if a reserved keyword is lowered without id_ prefix disambiguation.";
        doc /* \text{BasicIdentifier} = \{ s \in \Sigma^+ \mid s[0] \in (\Sigma_{\text{alpha}} \cup \{ 0\text{x}5\text{F} \}) \;\wedge\; (\forall j \in \{1, \dots, |s|-1\}, \, s[j] \in \Sigma_{\text{ident}}) \;\wedge\; s \notin \mathcal{K}_{\text{reserved}} \} */
        constraint def Invariant_REQ_0019_FormalInvariant;
        doc /* \sigma_{\text{id}}(S_{\text{raw}}) = \begin{cases} S_{\text{raw}}, & \text{if } S_{\text{raw}} \in \text{BasicIdentifier} \\ \pi_{\text{id}}(\text{Collapse}(\rho^*(\text{Trim}(S_{\text{raw}})))), & \text{otherwise} \end{cases} */
        constraint def Invariant_REQ_0019_The_sanitization_function;
        doc /* \pi_{\text{id}}(s) = \begin{cases} \text{"id\_"} \mathbin{\Vert} s, & \text{if } s = \varepsilon \;\lor\; s[0] \in \Sigma_{\text{digit}} \;\lor\; s \in \mathcal{K}_{\text{reserved}} \\ s, & \text{otherwise} \end{cases} */
        constraint def Invariant_REQ_0019_FormalInvariant_2;
        doc /* \sigma_{\text{id}}(\text{"123-Alpha (Beta)"}) = \text{"id\_123\_Alpha\_Beta\_"} */
        constraint def Invariant_REQ_0019_Canonical_Example_Sanitization_Invariant;
        doc /* \begin{aligned} \text{Count}(\sigma, \text{Scope}) &= |\{ s \in \text{Scope} \mid s = \sigma \;\lor\; \exists k \in \mathbb{N}_{\ge 2} : s = \sigma \mathbin{\Vert} \text{"\_"} \mathbin{\Vert} \text{Str}(k) \}| \\ \text{Disambiguate}(\sigma, \text{Scope}) &= \begin{cases} \sigma, & \text{if } \sigma \notin \text{Scope} \\ \sigma \mathbin{\Vert} \text{"\_"} \mathbin{\Vert} \text{Str}(\text{Count}(\sigma, \text{Scope}) + 1), & \text{if } \sigma \in \text{Scope} \end{cases} \end{aligned} */
        constraint def Invariant_REQ_0019_FormalInvariant_3;
        doc /* \begin{aligned} \text{ResolveIdent}(\mathbf{r}_i, \text{Scope}_{i-1}) &= \begin{cases} \text{Disambiguate}(\sigma_{\text{id}}(c_{i, j_{\text{name}}}), \text{Scope}_{i-1}), & \text{if } j_{\text{name}} \ne \bot \;\wedge\; \text{Trim}(c_{i, j_{\text{name}}}) \ne \varepsilon \\ \text{Disambiguate}(\text{"id\_anonymous\_"} \mathbin{\Vert} \text{Str}(i), \text{Scope}_{i-1}), & \text{otherwise} \end{cases} \\ \text{Scope}_i &= \text{Scope}_{i-1} \cup \{ \text{ResolveIdent}(\mathbf{r}_i, \text{Scope}_{i-1}) \} \end{aligned} */
        constraint def Invariant_REQ_0019_Row_Identifier_Resolution_and_Scope_Evolution;
        doc /* \begin{aligned} (j_{\text{name}} = \bot \;\lor\; \text{Trim}(c_{i, j_{\text{name}}}) = \varepsilon) \;\wedge\; \text{Strict}(\mathcal{T}) &\implies \text{Diagnostic}(i, j_{\text{name}}, \text{E0102}, \text{MissingIdentifier}) \in \mathcal{D} \\ |c_{i, j_{\text{name}}}| \gt L_{\text{max}} &\implies \text{Diagnostic}(i, j_{\text{name}}, \text{E0102}, \text{IdentifierTooLong}) \in \mathcal{D} \end{aligned} */
        constraint def Invariant_REQ_0019_Diagnostic_Error_Code_Binding_Invariants;
        require Invariant_REQ_0019_FormalInvariant;
        require Invariant_REQ_0019_The_sanitization_function;
        require Invariant_REQ_0019_FormalInvariant_2;
        require Invariant_REQ_0019_Canonical_Example_Sanitization_Invariant;
        require Invariant_REQ_0019_FormalInvariant_3;
        require Invariant_REQ_0019_Row_Identifier_Resolution_and_Scope_Evolution;
        require Invariant_REQ_0019_Diagnostic_Error_Code_Binding_Invariants;
        derived from REQ_0002_Surjective_Lexical_Provenance_Gate;
        verify by AC_01_Valid_Basic_Identifier_Passthrough;
        verify by AC_02_Non_Alphanumeric_Character_Sanitization_Leading_Digit_Prefixing;
        verify by AC_03_Scope_Sibling_Collision_Disambiguation;
        verify by AC_04_Reserved_Keyword_Disambiguation;
        satisfy by UniversalIngestionEngine;
    }
    doc /* UUIDv5: dc278a56-3a19-5113-a2ee-85edd2f01cf3 | Complexity: Class P (O(1)) | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0102 */
    requirement def REQ_0020_Disambiguation_of_Array_Indexing_Syntax_in_Tabular_Schemas {
        id = "REQ-0020";
        text = "The CommonMark table parser within the Universal Schema Ingestion Engine shall implement deterministic lexical disambiguation of array indexing syntax across tabular MBSE schemas. When ingesting cell tokens for structural identifiers, component references, port bindings, or interconnect endpoints,";
        attribute uuidv5 : String = "dc278a56-3a19-5113-a2ee-85edd2f01cf3";
        attribute complexity_class : String = "Class P (O(1))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0102";
        attribute ac_01_positive_single_dimensional_array_indexing : String = "Given: Tabular cells with bracketed identifiers (Part_Alpha[0], Port_1[42], Flow_A[65535]). When: Lowering cells via disambiguation pass _{array}. Then: Decomposes into interned base symbol SymbolId and 32-bit unsigned integers (0, 42, 65535) in IndexedReferenceNode. Diagnostic: E0102 on decomposition failure.";
        attribute ac_02_scalar_identifier_passthrough : String = "Given: Tabular cells with unindexed scalar identifiers (Classifier_Alpha, Port_1, Flow_A). When: Lowering cells via _{array}. Then: Evaluates to scalar references base, without triggering array logic. Diagnostic: E0102 on false-positive index detection.";
        attribute ac_03_multi_dimensional_array_indexing_bounds : String = "Given: Tabular cells with multi-dimensional indexing (Matrix_Node[2][5], Node_Alpha[1][0][7]) with rank K 8. When: Lowering cells via _{multi}. Then: Decomposes into base symbol and index tuples ( 2, 5 , 1, 0, 7 ); rejects K 8. Diagnostic: E0102 on rank overflow (K 8).";
        attribute ac_04_strict_lexical_separation_of_type_multiplicity_ranges : String = "Given: Schema cells with type multiplicity bounds (Classifier_Alpha[1..*], DataType_Beta[0..1]). When: Processing type columns versus instance columns. Then: Decomposes type cells to multiplicity L, U ; rejects multiplicity syntax in instance columns. Diagnostic: E0102 on multiplicity syntax in instance columns.";
        doc /* \text{Val}(d_0 \dots d_{m-1}) = \sum_{j=0}^{m-1} (d_j - 0\text{x}30) \cdot 10^{m-1-j} */
        constraint def Invariant_REQ_0020_FormalInvariant;
        doc /* \theta_{\text{array}}(S) = \begin{cases} \langle u, \text{Val}(v) \rangle, & \text{if } S = u \mathbin{\Vert} \text{"["} \mathbin{\Vert} v \mathbin{\Vert} \text{"]"} \;\wedge\; u \in \text{Ident} \;\wedge\; v \in \Sigma_{\text{digit}}^+ \;\wedge\; \text{Val}(v) \lt 2^{32} \\ \langle S, \bot \rangle, & \text{if } S \in \text{Ident} \\ \bot_{\text{err}}, & \text{otherwise} \end{cases} */
        constraint def Invariant_REQ_0020_Array_decomposition_operator;
        doc /* \theta_{\text{multi}}(S) = \begin{cases} \langle u, \langle \text{Val}(v_1), \dots, \text{Val}(v_K) \rangle \rangle, & \text{if } S = u \mathbin{\Vert} \prod_{j=1}^K (\text{"["} \mathbin{\Vert} v_j \mathbin{\Vert} \text{"]"}) \;\wedge\; u \in \text{Ident} \;\wedge\; 1 \le K \le 8 \\ \bot_{\text{err}}, & \text{otherwise} \end{cases} */
        constraint def Invariant_REQ_0020_For_multi_dimensional_indexing_with_rank;
        doc /* \begin{aligned} \theta_{\text{multiplicity}}(S) &= \begin{cases} \langle \text{Type}(S), \langle L(S), U(S) \rangle \rangle, & \text{if } S = \text{Ident} \mathbin{\Vert} \text{"["} \mathbin{\Vert} \text{RangeExpr} \mathbin{\Vert} \text{"]"} \\ \langle S, \langle 1, 1 \rangle \rangle, & \text{if } S \in \text{Ident} \\ \bot_{\text{err}}, & \text{otherwise} \end{cases} \\ \mathcal{L}(\text{IndexedIdentifier}) &\cap \mathcal{L}(\text{MultiplicityRangeType}) = \emptyset \\ (\kappa = \text{ColKind}_{\text{Instance}} \;\wedge\; \theta_{\text{array}}(S) = \bot_{\text{err}}) &\implies \exists D \in \mathcal{D} : \text{Code}(D) = \text{DiagnosticCode::E0102} \end{aligned} */
        constraint def Invariant_REQ_0020_FormalInvariant_2;
        require Invariant_REQ_0020_FormalInvariant;
        require Invariant_REQ_0020_Array_decomposition_operator;
        require Invariant_REQ_0020_For_multi_dimensional_indexing_with_rank;
        require Invariant_REQ_0020_FormalInvariant_2;
        derived from REQ_0191_Standardized_Compiler_Diagnostic_Taxonomy_Source_Mapping;
        verify by AC_01_Positive_Single_Dimensional_Array_Indexing;
        verify by AC_02_Scalar_Identifier_Passthrough;
        verify by AC_03_Multi_Dimensional_Array_Indexing_Bounds;
        verify by AC_04_Strict_Lexical_Separation_of_Type_Multiplicity_Ranges;
        satisfy by UniversalIngestionEngine;
    }
    doc /* UUIDv5: 6e0f876b-a57c-55e4-bee6-606761e29ebf | Complexity: Class P (O(R \cdot C) = O(R)) | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0102, E0103 */
    requirement def REQ_0021_Tabular_Schema_Ingestion_for_Structural_Parts {
        id = "REQ-0021";
        text = "The Universal Schema Ingestion Engine shall parse CommonMark tabular schemas specifying structural parts and component assemblies into IngestedPart intermediate entities. For each data row, the parser shall extract the part definition or usage name, the classifier type reference, the multiplicity";
        attribute uuidv5 : String = "6e0f876b-a57c-55e4-bee6-606761e29ebf";
        attribute complexity_class : String = "Class P (O(R \cdot C) = O(R))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0102, E0103";
        attribute ac_01_canonical_component_definition_lowering : String = "Given: A CommonMark table declaring structural parts with names, classifier types, and multiplicity strings ([1..*], [0..1], [2..4]). When: Lowering table rows into IngestedPart entities. Then: 100% of rows lower into valid typed arena nodes with interned SymbolId handles, accurate bounds l, u , and stable node offsets. Diagnostic: E0102 on AST generation failure.";
        attribute ac_02_default_multiplicity_assignment : String = "Given: A structural part table row with an empty multiplicity cell (M = ). When: Applying multiplicity evaluation ParseMultiplicity. Then: Assigns the default interval 1, 1 to the entity without emitting diagnostics. Diagnostic: E0103 on default multiplicity derivation failure.";
        attribute ac_03_multiplicity_syntax_and_inverted_range_rejection : String = "Given: A component definition table with malformed multiplicity ([5..2], [-1..3], [alpha], [1..). When: Parsing multiplicity cells during schema ingestion. Then: Rejects invalid tokens, substitutes sentinel _{mult}, emits diagnostic E0103 with byte coordinates, and recovers at the next row delimiter. Diagnostic: E0103 recorded in diagnostic sink D.";
        attribute ac_04_structural_containment_forest_and_hierarchy : String = "Given: A multi-tier assembly table specifying hierarchical nesting (Assembly_0 containing Subsystem_1, containing Part_1). When: Constructing the structural containment graph G_{contain}. Then: Constructs an acyclic forest where parent handles match owning assemblies and root assemblies have parent = . Diagnostic: E0102 on parent handle resolution failure.";
        doc /* P_i = \langle \text{id}_i, \text{name}_i, \text{type}_i, \text{mult}_i, \text{parent}_i, \mathcal{A}_i \rangle */
        constraint def Invariant_REQ_0021_FormalInvariant;
        doc /* \text{ParseMultiplicity}(M) = \begin{cases} \langle 1, 1 \rangle, & \text{if } M \in \{ \bot, \text{""} \} \\ \langle k, k \rangle, & \text{if } M = \text{Str}(k) \;\wedge\; k \in \mathbb{N}_0 \\ \langle 0, \infty \rangle, & \text{if } M = \text{"*"} \\ \langle l, u \rangle, & \text{if } M = \text{"[}l\text{..}u\text{]"} \;\wedge\; l \le u \\ \langle l, \infty \rangle, & \text{if } M = \text{"[}l\text{..*]"} \\ \bot_{\text{mult}}, & \text{otherwise} \end{cases} */
        constraint def Invariant_REQ_0021_Multiplicity_evaluation;
        doc /* \begin{aligned} \forall P_i \in \mathcal{P}, \quad \text{mult}_i = \langle l_i, u_i \rangle &\implies 0 \le l_i \le u_i \;\wedge\; u_i \ge 1 \\ M = \bot &\implies \text{ParseMultiplicity}(M) = \langle 1, 1 \rangle \end{aligned} */
        constraint def Invariant_REQ_0021_Bounded_Multiplicity_and_Default_Invariants;
        doc /* \begin{aligned} \forall P \in \mathcal{P}, \quad &(P, P) \notin \mathcal{E}_{\text{contain}}^+ \\ \forall P_c \in \mathcal{P}, \quad &|\{ P_p \in \mathcal{P} \mid (P_p, P_c) \in \mathcal{E}_{\text{contain}} \}| \le 1 \end{aligned} */
        constraint def Invariant_REQ_0021_FormalInvariant_2;
        doc /* \begin{aligned} (\text{ParseMultiplicity}(M_i) = \bot_{\text{mult}} \;\vee\; l_i \gt u_i) &\implies \exists D \in \mathcal{D} : \text{code}(D) = \text{DiagnosticCode::E0103} \\ ((P_i, P_i) \in \mathcal{E}_{\text{contain}}^+ \;\vee\; \text{name}_i = \bot \;\vee\; \text{type}_i = \bot) &\implies \exists D \in \mathcal{D} : \text{code}(D) = \text{DiagnosticCode::E0102} \end{aligned} */
        constraint def Invariant_REQ_0021_Diagnostic_Code_Bindings_per_REQ_0191;
        require Invariant_REQ_0021_FormalInvariant;
        require Invariant_REQ_0021_Multiplicity_evaluation;
        require Invariant_REQ_0021_Bounded_Multiplicity_and_Default_Invariants;
        require Invariant_REQ_0021_FormalInvariant_2;
        require Invariant_REQ_0021_Diagnostic_Code_Bindings_per_REQ_0191;
        derived from REQ_0019_Table_Row_to_Entity_Structural_Mapping;
        derived from REQ_0191_Standardized_Compiler_Diagnostic_Taxonomy_Source_Mapping;
        derived from REQ_0016_Synchronization_Token_Recovery_on_Table_Delimiter_Boundaries;
        verify by AC_01_Canonical_Component_Definition_Lowering;
        verify by AC_02_Default_Multiplicity_Assignment;
        verify by AC_03_Multiplicity_Syntax_and_Inverted_Range_Rejection;
        verify by AC_04_Structural_Containment_Forest_and_Hierarchy;
        satisfy by UniversalIngestionEngine;
    }
    doc /* UUIDv5: 630738dc-98d6-5c15-b60e-be7d63747ff4 | Complexity: Class P (O(N)) | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0102, E0199 */
    requirement def REQ_0022_Tabular_Schema_Ingestion_for_Directed_Ports {
        id = "REQ-0022";
        text = "The CommonMark table parser within the Universal Schema Ingestion Engine shall parse tabular schemas specifying directed ports into intermediate IngestedPort AST entities. For each data row, the parser shall extract the port identifier, owning component handle, declared direction (in, out, inout),";
        attribute uuidv5 : String = "630738dc-98d6-5c15-b60e-be7d63747ff4";
        attribute complexity_class : String = "Class P (O(N))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0102, E0199";
        attribute ac_01_positive_ingestion_of_directed_ports : String = "Given: Valid CommonMark table declaring directed ports (Port_0, Port_1, Port_2) on component Classifier_Alpha with directions in, out, inout and payload Flow_A. When: Lowering rows into AST entities during the port ingestion pass. Then: 100% of rows synthesize into IngestedPort AST nodes with declared directions and valid arena handles. Diagnostic: E0102 on row parsing or lowering failure.";
        attribute ac_02_conjugated_port_direction_involution : String = "Given: Schema rows specifying conjugated ports (~Port_1 or conjugation flag true) for directions in, out, and inout. When: Evaluating effective directions via involution operator _{conj}. Then: Directions invert polarity (In Out, Out In, InOut InOut), satisfying identity _{conj}(_{conj}(d)) d. Diagnostic: E0102 on direction polarity resolution failure.";
        attribute ac_03_payload_typing_and_contract_preservation : String = "Given: Declared ports with payload type Flow_A, multiplicity bounds [1..1], and invariant constraints. When: Applying involution _{conj} during port lowering. Then: Inverted port entities retain bitwise identical payload type handles, multiplicity bounds, and invariant constraints. Diagnostic: E0102 on payload mutation or type mismatch under conjugation.";
        attribute ac_04_scoped_port_identifier_collision_rejection : String = "Given: Table rows declaring duplicate port identifier Port_1 under the identical owning component Classifier_Alpha. When: Ingesting the colliding row during AST construction. Then: Emits structured diagnostic E0102 with 1-indexed coordinates and byte span, substitutes sentinel _{port}, and preserves the original port entity. Diagnostic: E0102 emitted upon scoped identifier collision.";
        doc /* \text{Dir} = \{ \text{In}, \text{Out}, \text{InOut} \}, \quad \text{ParseDir}(s) = \begin{cases} \text{In}, & \text{if } s \in \{ \text{"in"}, \text{"input"} \} \\ \text{Out}, & \text{if } s \in \{ \text{"out"}, \text{"output"} \} \\ \text{InOut}, & \text{if } s \in \{ \text{"inout"}, \text{"in\_out"} \} \\ \bot, & \text{otherwise} \end{cases} */
        constraint def Invariant_REQ_0022_FormalInvariant;
        doc /* \begin{aligned} \tau_{\text{conj}}(\text{In}) = \text{Out}, \quad \tau_{\text{conj}}(\text{Out}) &= \text{In}, \quad \tau_{\text{conj}}(\text{InOut}) = \text{InOut} \\ d_{\text{eff}}(d, c) &= \begin{cases} \tau_{\text{conj}}(d), & \text{if } c = \text{true} \\ d, & \text{if } c = \text{false} \end{cases} \\ \forall d \in \text{Dir}, \; \forall c \in \mathbb{B}, \quad d_{\text{eff}}(d_{\text{eff}}(d, c), c) &\equiv d \end{aligned} */
        constraint def Invariant_REQ_0022_The_port_conjugation_involution_operator;
        doc /* \text{PortDef} = \langle \text{id} \in \mathcal{S}_{\text{sym}}, \text{owner} \in \mathcal{H}_{\text{node}}, d_{\text{decl}} \in \text{Dir}, d_{\text{eff}} \in \text{Dir}, \tau_{\text{payload}} \in \mathcal{H}_{\text{type}}, c \in \mathbb{B}, \mu \in \mathbb{N}_0 \times (\mathbb{N}_0 \cup \{ \infty \}) \rangle */
        constraint def Invariant_REQ_0022_FormalInvariant_2;
        doc /* \begin{aligned} \text{PayloadType}(\tau_{\text{conj}}(P)) &\equiv \text{PayloadType}(P) \\ \text{Multiplicity}(\tau_{\text{conj}}(P)) &\equiv \text{Multiplicity}(P) \\ \text{ContractInvariants}(\tau_{\text{conj}}(P)) &\equiv \text{ContractInvariants}(P) \end{aligned} */
        constraint def Invariant_REQ_0022_Conjugation_preserves_payload_typing_multiplicity_bounds_and_contract_invariants;
        doc /* \begin{aligned} \forall P_a, P_b \in \mathcal{P}, \quad (P_a.\text{owner} = P_b.\text{owner} \;\wedge\; P_a.\text{id} = P_b.\text{id}) &\implies P_a = P_b \\ \text{ParseDir}(\text{Cell}(R_i, C_{\text{dir}})) = \bot &\implies \exists D \in \mathcal{D} : \text{Code}(D) = \text{DiagnosticCode::E0102} \\ (\exists k \ne i : P_k.\text{owner} = P_i.\text{owner} \;\wedge\; P_k.\text{id} = P_i.\text{id}) &\implies \exists D \in \mathcal{D} : \text{Code}(D) = \text{DiagnosticCode::E0102} \\ \text{Cell}(R_i, C_{\text{id}}) = \varepsilon &\implies \exists D \in \mathcal{D} : \text{Code}(D) = \text{DiagnosticCode::E0102} \\ |\mathcal{D}| \ge C_{\text{max\_errors}} &\implies \exists D_{\text{term}} \in \mathcal{D} : \text{Code}(D_{\text{term}}) = \text{DiagnosticCode::E0199} \end{aligned} */
        constraint def Invariant_REQ_0022_Scoped_identifier_uniqueness_and_diagnostic_code_bindings_per_REQ_0191_satisfy;
        require Invariant_REQ_0022_FormalInvariant;
        require Invariant_REQ_0022_The_port_conjugation_involution_operator;
        require Invariant_REQ_0022_FormalInvariant_2;
        require Invariant_REQ_0022_Conjugation_preserves_payload_typing_multiplicity_bounds_and_contract_invariants;
        require Invariant_REQ_0022_Scoped_identifier_uniqueness_and_diagnostic_code_bindings_per_REQ_0191_satisfy;
        derived from REQ_0191_Standardized_Compiler_Diagnostic_Taxonomy_Source_Mapping;
        verify by AC_01_Positive_Ingestion_of_Directed_Ports;
        verify by AC_02_Conjugated_Port_Direction_Involution;
        verify by AC_03_Payload_Typing_and_Contract_Preservation;
        verify by AC_04_Scoped_Port_Identifier_Collision_Rejection;
        satisfy by UniversalIngestionEngine;
    }
    doc /* UUIDv5: b6878261-7ad9-5e86-8860-11f38dabeb81 | Complexity: Class P (O(1)) | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0103, E0199 */
    requirement def REQ_0023_Tabular_Schema_Ingestion_for_Attributes_Data_Types {
        id = "REQ-0023";
        text = "The Universal Schema Ingestion Engine shall parse CommonMark tabular schemas specifying classifier attributes, scalar types, default values, engineering units, and numerical bounds into IngestedAttr AST entities. The parser shall map columns via an order-independent binding vector into canonical";
        attribute uuidv5 : String = "b6878261-7ad9-5e86-8860-11f38dabeb81";
        attribute complexity_class : String = "Class P (O(1))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0103, E0199";
        attribute ac_01_valid_attribute_numerical_envelope_extraction : String = "Given: A tabular schema with attributes across Real, Integer, Boolean, and String, valid intervals ([0.0, 100.0], (0, 50], [-10, 10], [0.0, +inf)), defaults, and SI units (\"m/s\", \"kg\"). When: The parser processes table rows. Then: Each row synthesizes an IngestedAttr AST node with exact SI exponent vector u Q^7, exact boundary limits, and zero diagnostics. Diagnostic: None emitted.";
        attribute ac_02_degenerate_singleton_interval_validation : String = "Given: Table rows with singleton intervals (L = U = 10.0) and default v_{default} = 10.0. When: Parser evaluates closed [10.0, 10.0] versus open (10.0, 10.0). Then: Closed [10.0, 10.0] succeeds; open (10.0, 10.0) is rejected as empty, substituting sentinel _{attr}. Diagnostic: E0103 on open singleton interval.";
        attribute ac_03_inverted_interval_bounds_default_boundary_violation : String = "Given: Table rows with inverted bounds (L U, e.g. [100.0, 50.0]) or defaults violating bounds (v_{default} = 150.0 for [0.0, 100.0]). When: Interval validity and default containment predicates are evaluated. Then: Validation fails closed, substituting sentinel _{attr} into the AST with exact source span mapping. Diagnostic: E0103.";
        attribute ac_04_non_numeric_type_envelope_incompatibility : String = "Given: A row with non-numeric attribute type (Boolean, String) declaring a restrictive numerical envelope or non-zero unit vector (param_flag : Boolean with [0, 1]). When: Type-envelope compatibility rules are verified. Then: Incompatible envelopes are rejected and sentinel _{attr} is substituted. Diagnostic: E0103.";
        doc /* A = \langle \text{id}, \text{owner}, \tau, v_{\text{default}}, \mathcal{I}, \mathbf{u}, \text{span} \rangle */
        constraint def Invariant_REQ_0023_FormalInvariant;
        doc /* \begin{aligned} \tau &\in \mathcal{T}_{\text{type}} = \{ \text{Real}, \text{Integer}, \text{Boolean}, \text{String}, \text{ClassifierRef} \} \\ v_{\text{default}} &\in \text{Domain}(\tau) \cup \{ \bot \}, \quad \mathbf{u} \in \mathbb{Q}^7 \\ \mathcal{I} &= \langle L, U, o_L, o_U \rangle \in (\mathbb{R} \cup \{ -\infty \}) \times (\mathbb{R} \cup \{ +\infty \}) \times \mathbb{B}^2 \\ \text{span} &= \langle \text{file\_id}, p_{\text{start}}, p_{\text{end}}, \text{line}, \text{col} \rangle \end{aligned} */
        constraint def Invariant_REQ_0023_FormalInvariant_2;
        doc /* \begin{aligned} \text{ValidInterval}(\mathcal{I}) &\iff (L \le U) \wedge (L = U \implies (\neg o_L \wedge \neg o_U \wedge L \in \mathbb{R})) \\ v \in \mathcal{I} &\iff (o_L \mathrel{?} L \lt v \mathrel{:} L \le v) \wedge (o_U \mathrel{?} v \lt U \mathrel{:} v \le U) \\ v_{\text{default}} \ne \bot &\implies (v_{\text{default}} \in \text{Domain}(\tau) \wedge (\tau \in \{ \text{Real}, \text{Integer} \} \implies v_{\text{default}} \in \mathcal{I})) \\ \tau \in \{ \text{Boolean}, \text{String} \} &\implies (\mathcal{I} = \langle -\infty, +\infty, \text{true}, \text{true} \rangle \wedge \mathbf{u} = \mathbf{0}_{\mathbb{Q}^7}) \end{aligned} */
        constraint def Invariant_REQ_0023_Well_formedness_containment_and_compatibility_predicates;
        doc /* \begin{aligned} (\neg \text{ValidInterval}(\mathcal{I}) \vee (v_{\text{default}} \ne \bot \wedge v_{\text{default}} \notin \mathcal{I}) \vee \mu(\text{unit\_str}) = \bot) &\implies \text{Diagnostic}(A, \text{E0103}) \in \mathcal{D} \\ |\mathcal{D}| \ge C_{\text{max\_errors}} &\implies \exists D_{\text{term}} \in \mathcal{D} : \text{code}(D_{\text{term}}) = \text{E0199} \end{aligned} */
        constraint def Invariant_REQ_0023_Diagnostic_trigger_invariants;
        require Invariant_REQ_0023_FormalInvariant;
        require Invariant_REQ_0023_FormalInvariant_2;
        require Invariant_REQ_0023_Well_formedness_containment_and_compatibility_predicates;
        require Invariant_REQ_0023_Diagnostic_trigger_invariants;
        verify by AC_01_Valid_Attribute_Numerical_Envelope_Extraction;
        verify by AC_02_Degenerate_Singleton_Interval_Validation;
        verify by AC_03_Inverted_Interval_Bounds_Default_Boundary_Violation;
        verify by AC_04_Non_Numeric_Type_Envelope_Incompatibility;
        satisfy by UniversalIngestionEngine;
    }
    doc /* UUIDv5: e4fb977a-d151-565b-8da7-606b66917657 | Complexity: Class P (O(R) = O(N)) | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0102, E0103, E0199 */
    requirement def REQ_0024_Tabular_Schema_Ingestion_for_Formal_Constraints {
        id = "REQ-0024";
        text = "The Universal Schema Ingestion Engine shall parse CommonMark tabular schemas defining formal constraints, safety conditions, and operational envelopes into strongly typed IngestedConstraint AST entities. For each row, the parser shall extract the constraint identifier (id), owning classifier";
        attribute uuidv5 : String = "e4fb977a-d151-565b-8da7-606b66917657";
        attribute complexity_class : String = "Class P (O(R) = O(N))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0102, E0103, E0199";
        attribute ac_01_verbatim_constraint_expression_ingestion : String = "Given: Tabular rows containing constraint expressions (param_x >= 0.0 && param_x <= 100.0, param_a + param_b == param_c, !(flag_1 && flag_2) || state == State_A). When: Ingestion pass executes. Then: 100% of raw expression strings in synthesized IngestedConstraint AST entities match source cell byte slices with 0-byte divergence. Diagnostic: E0102 on cell extraction failure or byte mutation.";
        attribute ac_02_constraint_kind_and_severity_defaulting : String = "Given: Tabular rows omitting kind and severity. When: Parser synthesizes IngestedConstraint entities. Then: Omitted kind fields evaluate to Assert and omitted severity fields evaluate to Critical. Diagnostic: E0102 on failure to apply normative defaults.";
        attribute ac_03_scoped_constraint_identifier_uniqueness : String = "Given: Schema declaring two constraints with duplicate identifier Constraint_1 under owner Classifier_Alpha. When: Parser validates scoped uniqueness. Then: Detects identifier collision, emits E0102, substitutes sentinel _{constraint}, and continues row processing. Diagnostic: E0102 with duplicate identifier byte span.";
        attribute ac_04_operational_envelope_interval_consistency : String = "Given: Rows specifying valid envelopes ([0.0, 100.0]) alongside an invalid row with inverted bounds ([100.0, 10.0]). When: Parser evaluates interval consistency. Then: Valid intervals yield I_i E; inverted row emits E0103 and substitutes sentinel _{constraint}. Diagnostic: E0103 on inverted or degenerate open bounds.";
        doc /* C_i = \langle \text{id}_i, \text{owner}_i, \text{expr}_i, \text{kind}_i, \text{severity}_i, \mathcal{I}_i, \text{span}_i \rangle */
        constraint def Invariant_REQ_0024_FormalInvariant;
        doc /* \begin{aligned} \text{id}_i, \text{owner}_i &\in \mathcal{S}_{\text{sym}}, \quad \text{expr}_i \in \Sigma^* \\ \text{kind}_i &\in \{ \text{Assert}, \text{Assume} \}, \quad \text{severity}_i \in \{ \text{Critical}, \text{Degraded}, \text{Advisory} \} \\ \mathcal{I}_i &\in (\mathbb{R} \cup \{ -\infty \}) \times (\mathbb{R} \cup \{ +\infty \}) \times \mathbb{B}^2 \cup \{ \bot_{\text{env}} \} \\ \text{span}_i &= \langle \text{file\_id}, p_{\text{start}}, p_{\text{end}}, \text{line}, \text{col} \rangle \end{aligned} */
        constraint def Invariant_REQ_0024_with_constituent_domains;
        doc /* \text{RawBytes}(\text{expr}_i) \equiv \text{SourceCellSlice}(R_i, \beta^{-1}(\text{expr})) */
        constraint def Invariant_REQ_0024_Verbatim_Expression_Preservation_Invariant;
        doc /* \begin{aligned} \text{ParseKind}(s) &= \begin{cases} \text{Assert}, & \text{if } s \in \{ \bot, \text{""}, \text{"assert"}, \text{"invariant"}, \text{"guarantee"} \} \\ \text{Assume}, & \text{if } s \in \{ \text{"assume"}, \text{"precondition"}, \text{"require"} \} \\ \bot_{\text{kind}}, & \text{otherwise} \end{cases} \\ \text{ParseSeverity}(s) &= \begin{cases} \text{Critical}, & \text{if } s \in \{ \bot, \text{""}, \text{"critical"}, \text{"fatal"}, \text{"high"} \} \\ \text{Degraded}, & \text{if } s \in \{ \text{"degraded"}, \text{"warning"}, \text{"medium"} \} \\ \text{Advisory}, & \text{if } s \in \{ \text{"advisory"}, \text{"info"}, \text{"low"} \} \\ \bot_{\text{sev}}, & \text{otherwise} \end{cases} \\ \text{ValidEnvelope}(\langle L, U, o_L, o_U \rangle) &\iff (L \le U) \wedge (L = U \implies \neg o_L \wedge \neg o_U \wedge L \in \mathbb{R}) \\ \forall C_a, C_b \in \mathcal{C}, \quad (C_a.\text{owner} = C_b.\text{owner} &\wedge C_a.\text{id} = C_b.\text{id}) \implies C_a = C_b \end{aligned} */
        constraint def Invariant_REQ_0024_Classification_defaulting_envelope_consistency_and_scoped_uniqueness_satisfy;
        doc /* \begin{aligned} (\text{MissingCol}(R_i) \vee \text{ParseKind} = \bot_{\text{kind}} \vee \text{ParseSeverity} = \bot_{\text{sev}} \vee \text{DuplicateId}) &\implies \text{DiagnosticCode::E0102} \\ (\mathcal{I}_i \ne \bot_{\text{env}} \wedge \neg \text{ValidEnvelope}(\mathcal{I}_i)) &\implies \text{DiagnosticCode::E0103} \\ |\mathcal{D}| \ge C_{\text{max\_errors}} &\implies \text{DiagnosticCode::E0199} \end{aligned} */
        constraint def Invariant_REQ_0024_Diagnostic_Code_Binding_Invariant;
        require Invariant_REQ_0024_FormalInvariant;
        require Invariant_REQ_0024_with_constituent_domains;
        require Invariant_REQ_0024_Verbatim_Expression_Preservation_Invariant;
        require Invariant_REQ_0024_Classification_defaulting_envelope_consistency_and_scoped_uniqueness_satisfy;
        require Invariant_REQ_0024_Diagnostic_Code_Binding_Invariant;
        derived from REQ_0138_Structured_ConstraintExpr_AST_Invariant_Generation_with_De_Morgan_Normalization;
        derived from REQ_0188_SMT_LIB2_Z3_First_Order_Constraint_Encoding;
        verify by AC_01_Verbatim_Constraint_Expression_Ingestion;
        verify by AC_02_Constraint_Kind_and_Severity_Defaulting;
        verify by AC_03_Scoped_Constraint_Identifier_Uniqueness;
        verify by AC_04_Operational_Envelope_Interval_Consistency;
        satisfy by UniversalIngestionEngine;
    }
    doc /* UUIDv5: ae9aa9c2-1f29-5263-a58e-3cba1992ff78 | Complexity: Class P (O(N)) | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0102, E0104, E0199 */
    requirement def REQ_0025_Tabular_Schema_Ingestion_for_Topological_Connections {
        id = "REQ-0025";
        text = "The CommonMark table parser within the Universal Schema Ingestion Engine shall parse tabular schemas specifying topological connections into IngestedConnection intermediate AST entities. For each data row, the parser must extract the connection identifier, source port endpoint path (P_{src}),";
        attribute uuidv5 : String = "ae9aa9c2-1f29-5263-a58e-3cba1992ff78";
        attribute complexity_class : String = "Class P (O(N))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0102, E0104, E0199";
        attribute ac_01_canonical_connection_ingestion_and_endpoint_decomposition : String = "Given: Valid table declaring connection Conn_0 between Part_0::Port_1 and Part_1::Port_2 with protocol Protocol_A and latency bound 5.0. When: Lowering the connection row. Then: Emits IngestedConnection with decomposed OwnerPath and PortIdentifier in AST storage without diagnostics. Diagnostic: E0104 on failure.";
        attribute ac_02_endpoint_path_well_formedness_enforcement : String = "Given: Connection row with malformed endpoint path having fewer than two segments (Port_1) or empty segments (Part_0::::Port_1). When: Performing endpoint path structural validation. Then: Rejects path, substitutes sentinel _{conn}, and records localized diagnostic with byte span [p_{start}, p_{end}). Diagnostic: Emits E0104 with source span coordinates.";
        attribute ac_03_self_loop_detection_and_explicit_delay_enforcement : String = "Given: Connection row specifying identical endpoints (Part_0::Port_1 to Part_0::Port_1). When: Evaluated with unbuffered delay flag ( = false) versus explicit delay flag ( = true). Then: Unbuffered self-loop is rejected with sentinel substitution; explicit delay self-loop lowers successfully into AST entity. Diagnostic: Emits E0104 when = false; zero diagnostics when = true.";
        attribute ac_04_latency_bounds_and_scoped_identifier_uniqueness : String = "Given: Connection rows specifying negative latency (_{lat} = -5.0) or duplicate identifier Conn_0 in scope Package_0::Classifier_Alpha. When: Validating temporal bounds and scope uniqueness. Then: Rejects invalid rows with diagnostic emission and sentinel substitution _{conn}. Diagnostic: Emits E0104 on envelope or uniqueness violation.";
        doc /* C_i = \langle \text{id}_i, P_{\text{src}, i}, P_{\text{tgt}, i}, \pi_i, \tau_{\text{lat}, i}, \delta_i, \text{span}_i \rangle */
        constraint def Invariant_REQ_0025_FormalInvariant;
        doc /* \begin{aligned} P &= \text{OwnerPath}(P) \mathbin{\Vert} \text{"::"} \mathbin{\Vert} \text{PortIdentifier}(P) \\ \text{Segments}(P) &= k \ge 2 \quad \wedge \quad \forall j \in \{1, \dots, k\}, \; s_j \in \text{Ident} \end{aligned} */
        constraint def Invariant_REQ_0025_Endpoint_path_decomposition_and_segment_bounds_satisfy;
        doc /* (P_{\text{src}} = P_{\text{tgt}}) \implies (\delta = \text{true}) \quad \wedge \quad (P_{\text{src}} = P_{\text{tgt}} \;\wedge\; \delta = \text{false}) \implies \bot_{\text{conn}} */
        constraint def Invariant_REQ_0025_In_topological_multigraph_edge_Self_loops_without_delay_models_are_strictly_prohibited;
        doc /* \begin{aligned} \tau_{\text{lat}} \ne \bot &\implies \tau_{\text{lat}} \ge 0 \\ \forall C_a, C_b \in \mathcal{C}, \quad (\text{Scope}(C_a) = \text{Scope}(C_b) \;\wedge\; C_a.\text{id} = C_b.\text{id}) &\implies C_a = C_b \end{aligned} */
        constraint def Invariant_REQ_0025_Latency_bounds_and_scoped_identifier_uniqueness_satisfy;
        doc /* \begin{aligned} (\text{Segments}(P_{\text{src}}) \lt 2 \;\vee\; \text{Segments}(P_{\text{tgt}}) \lt 2 \;\vee\; (P_{\text{src}} = P_{\text{tgt}} \wedge \neg\delta) \;\vee\; \tau_{\text{lat}} \lt 0) &\implies \text{DiagnosticCode::E0104} \in \mathcal{D} \\ |\mathcal{D}| \ge C_{\text{max\_errors}} &\implies \text{DiagnosticCode::E0199} \in \mathcal{D} \end{aligned} */
        constraint def Invariant_REQ_0025_Diagnostic_code_binding_per_REQ_0191_and_sink_saturation_satisfy;
        require Invariant_REQ_0025_FormalInvariant;
        require Invariant_REQ_0025_Endpoint_path_decomposition_and_segment_bounds_satisfy;
        require Invariant_REQ_0025_In_topological_multigraph_edge_Self_loops_without_delay_models_are_strictly_prohibited;
        require Invariant_REQ_0025_Latency_bounds_and_scoped_identifier_uniqueness_satisfy;
        require Invariant_REQ_0025_Diagnostic_code_binding_per_REQ_0191_and_sink_saturation_satisfy;
        derived from REQ_0191_Standardized_Compiler_Diagnostic_Taxonomy_Source_Mapping;
        verify by AC_01_Canonical_Connection_Ingestion_and_Endpoint_Decomposition;
        verify by AC_02_Endpoint_Path_Well_Formedness_Enforcement;
        verify by AC_03_Self_Loop_Detection_and_Explicit_Delay_Enforcement;
        verify by AC_04_Latency_Bounds_and_Scoped_Identifier_Uniqueness;
        satisfy by UniversalIngestionEngine;
    }
    doc /* UUIDv5: e6cb1f58-e661-5f2d-9527-46446efe9c27 | Complexity: Class P (O(N)) | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0102, E0199 */
    requirement def REQ_0026_Tabular_Schema_Ingestion_for_Behaviors_Actions {
        id = "REQ-0026";
        text = "The Universal Schema Ingestion Engine shall parse CommonMark tables specifying behavioral actions, activities, and discrete state transitions into intermediate IngestedAction and IngestedTransition AST entities. For action rows, the parser must extract action identifier id, owning classifier";
        attribute uuidv5 : String = "e6cb1f58-e661-5f2d-9527-46446efe9c27";
        attribute complexity_class : String = "Class P (O(N))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0102, E0199";
        attribute ac_01_canonical_behavioral_action_ingestion : String = "Given: CommonMark table declaring actions Action_0 and Activity_Alpha with inputs (param_in : Real), outputs (param_out : Integer), trigger Event_0, guard guard_cond_0, and constraints pre_0, post_0. When: The parser ingests and lowers table rows. Then: 100% of rows lower into IngestedAction AST nodes in contiguous AST storage with valid symbol handles, disjoint parameter sets, and zero diagnostics. Diagnostic: E0102 emitted on extraction or parameter syntax failure.";
        attribute ac_02_state_transition_table_ingestion_endpoint_scoping : String = "Given: CommonMark table declaring transitions Transition_0 and Transition_1 in StateMachine_0 between states State_Idle and State_Active with trigger Event_Start and effect Action_Notify. When: The parser resolves transition rows against state scope States(StateMachine_0}). Then: Rows lower into IngestedTransition AST entities with verified endpoint handles s_{source}, s_{target} States(StateMachine_0}). Diagnostic: E0102 emitted if s_{source} or s_{target} fails scope resolution.";
        attribute ac_03_parameter_signature_well_formedness_disjointness_gate : String = "Given: Behavioral table row with duplicate parameter names (param_x : Real, param_x : Integer) or non-disjoint parameter sets (I O ). When: The parameter parser tokenizes and validates declared parameter sets. Then: The row is rejected, sentinel _{action} is substituted, and parsing resumes. Diagnostic: E0102 emitted with exact byte span and parameter error classification.";
        attribute ac_04_delimiter_synchronization_recovery_on_row_boundaries : String = "Given: 10-row behavioral table with syntax faults on rows 3 and 7 (unclosed delimiter and malformed parameter syntax). When: The parser ingests the table under Synchronization Token Error Recovery. Then: Rows 1, 2, 4, 5, 6, 8, 9, 10 lower into valid AST entities, rows 3 and 7 record _{action}, exactly 2 diagnostics are logged, and parsing resumes on SYM_{LF} boundaries. Diagnostic: E0102 emitted for rows 3 and 7; E0199 emitted if fault count reaches C_{max_errors}}.";
        doc /* \mathcal{K}_{\text{behavior}} = \{ \text{id}, \text{owner}, \text{kind}, \text{inputs}, \text{outputs}, \text{trigger}, \text{guard}, \text{pre}, \text{post}, \text{source}, \text{target}, \text{effect} \} */
        constraint def Invariant_REQ_0026_FormalInvariant;
        doc /* A = \langle \text{id}, \text{owner}, \mathcal{I}, \mathcal{O}, e_{\text{trigger}}, g_{\text{guard}}, \mathcal{P}_{\text{pre}}, \mathcal{P}_{\text{post}} \rangle */
        constraint def Invariant_REQ_0026_FormalInvariant_2;
        doc /* \begin{aligned} \forall a \ne b &\implies p_a \ne p_b \\ \forall c \ne d &\implies q_c \ne q_d \\ \{ p_1, \dots, p_m \} &\cap \{ q_1, \dots, q_k \} = \emptyset \end{aligned} */
        constraint def Invariant_REQ_0026_FormalInvariant_3;
        doc /* T = \langle s_{\text{source}}, s_{\text{target}}, e_{\text{trigger}}, g_{\text{guard}}, A_{\text{effect}} \rangle */
        constraint def Invariant_REQ_0026_FormalInvariant_4;
        doc /* s_{\text{source}}, s_{\text{target}} \in \text{States}(\text{owner}) = \{ s \in \mathcal{S}_{\text{sym}} \mid \text{DeclaredState}(s, \text{owner}) \} */
        constraint def Invariant_REQ_0026_satisfying_endpoint_state_machine_containment_and_guard_determinism;
        doc /* (s_{\text{source}, a} = s_{\text{source}, b} \;\wedge\; e_{\text{trigger}, a} = e_{\text{trigger}, b} \;\wedge\; a \ne b) \implies (g_{\text{guard}, a} \wedge g_{\text{guard}, b} \equiv \text{false}) */
        constraint def Invariant_REQ_0026_FormalInvariant_5;
        doc /* \Phi_{\text{behavior}}(R_i) = \begin{cases} A_i \text{ or } T_i, & \text{if } \text{ValidRow}(R_i) \\ \bot_{\text{action}} \text{ or } \bot_{\text{trans}}, & \text{if } \neg\text{ValidRow}(R_i) \end{cases} */
        constraint def Invariant_REQ_0026_Row_evaluation_and_diagnostic_code_bindings_satisfy;
        doc /* \begin{aligned} \neg\text{ValidRow}(R_i) &\implies \text{Diagnostic}(R_i, \text{DiagnosticCode::E0102}) \in \mathcal{D} \\ |\mathcal{D}| \ge C_{\text{max\_errors}} &\implies \exists D_{\text{term}} \in \mathcal{D} : \text{Code}(D_{\text{term}}) = \text{DiagnosticCode::E0199} \end{aligned} */
        constraint def Invariant_REQ_0026_FormalInvariant_6;
        require Invariant_REQ_0026_FormalInvariant;
        require Invariant_REQ_0026_FormalInvariant_2;
        require Invariant_REQ_0026_FormalInvariant_3;
        require Invariant_REQ_0026_FormalInvariant_4;
        require Invariant_REQ_0026_satisfying_endpoint_state_machine_containment_and_guard_determinism;
        require Invariant_REQ_0026_FormalInvariant_5;
        require Invariant_REQ_0026_Row_evaluation_and_diagnostic_code_bindings_satisfy;
        require Invariant_REQ_0026_FormalInvariant_6;
        derived from REQ_0191_Standardized_Compiler_Diagnostic_Taxonomy_Source_Mapping;
        derived from REQ_0013_Synchronization_Token_Error_Recovery_Architecture;
        derived from REQ_0016_Synchronization_Token_Recovery_on_Table_Delimiter_Boundaries;
        verify by AC_01_Canonical_Behavioral_Action_Ingestion;
        verify by AC_02_State_Transition_Table_Ingestion_Endpoint_Scoping;
        verify by AC_03_Parameter_Signature_Well_Formedness_Disjointness_Gate;
        verify by AC_04_Delimiter_Synchronization_Recovery_on_Row_Boundaries;
        satisfy by UniversalIngestionEngine;
    }
    doc /* UUIDv5: 5b117440-5bfe-5366-b221-545d6925be8e | Complexity: Class P (O(N)) | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100, E0102, E0103 */
    requirement def REQ_0027_Protocol_Buffer_AST_Ingestion_for_Messages_Fields {
        id = "REQ-0027";
        text = "The compiler ingestion engine shall provide a native Protocol Buffer version 3 (Proto3) parser that ingests .proto schemas into intermediate schema representation (IR) without external toolchain dependencies. The engine shall lower Protobuf messages to IngestedPart definitions, fields to typed";
        attribute uuidv5 : String = "5b117440-5bfe-5366-b221-545d6925be8e";
        attribute complexity_class : String = "Class P (O(N))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100, E0102, E0103";
        attribute ac_01_canonical_message_and_field_lowering : String = "Given: Valid Proto3 schema with message Message_Alpha, fields (Field_1 : int32 = 1, Field_2 : string = 2, Field_3 : Message_Beta = 3), and multiplicity qualifiers (repeated, optional, singular). When: Lowering the AST into intermediate schema IR. Then: Emits IngestedPart and IngestedAttr with exact types, tags, and multiplicities ( 0, , 0, 1 , 1, 1 ) without diagnostics. Diagnostic: E0100 on parsing failure.";
        attribute ac_02_field_tag_range_and_reserved_boundary_enforcement : String = "Given: Proto3 messages with field tags at boundaries (i = 1, i = 536{,}870{,}911), out-of-range (i = 0, i = 536{,}870{,}912), or in reserved range (i [19000, 19999]). When: Evaluating field tag validity during ingestion. Then: Boundary tags lower cleanly; invalid tags (i 0, i 2^{29}-1, or i [19000, 19999]) emit structured diagnostics. Diagnostic: E0103 on invalid field tags.";
        attribute ac_03_duplicate_field_tag_collision_detection : String = "Given: Message Message_Alpha declaring fields Field_1 and Field_2 both with tag i = 5. When: Ingesting field declarations within Message_Alpha. Then: Detects tag collision, logs structured diagnostic, and substitutes sentinel _{field}. Diagnostic: E0102 on duplicate field tag collision.";
        attribute ac_04_enum_tag_zero_default_and_nested_scope_resolution : String = "Given: Enum Enum_0 with first constant at tag 1, and parent message Message_Alpha declaring inner message Message_Beta. When: Ingesting enum definitions and resolving nested scopes. Then: Rejects Enum_0 for missing tag-0 default, and resolves Message_Beta to path Package_0::Message_Alpha::Message_Beta. Diagnostic: E0103 on missing tag-0 default; E0100 on scope failure.";
        doc /* \mathcal{P}_{\text{proto}} = \langle \mathcal{H}_{\text{syntax}}, \mathcal{N}_{\text{pkg}}, \mathcal{I}_{\text{imp}}, \mathcal{M}, \mathcal{E}, \mathcal{S} \rangle */
        constraint def Invariant_REQ_0027_FormalInvariant;
        doc /* \begin{aligned} \text{message } M \{ \dots \} &\longmapsto \text{IngestedPart}(M) \\ \text{field } T \, f = i &\longmapsto \text{IngestedAttr}(f, \text{type}: T, \text{tag}: i, \text{mult}: \mu(f)) \\ \text{rpc } R(A) \text{ returns } (B) &\longmapsto \text{IngestedAction}(R, \text{in}: [\langle \text{param}_{\text{in}}, A, \mu_{\text{req}} \rangle], \text{out}: [\langle \text{param}_{\text{out}}, B, \mu_{\text{resp}} \rangle]) \\ \text{enum } E \{ \dots \} &\longmapsto \text{IngestedClassifierDef}(E, \text{kind}: \text{Enum}) \end{aligned} */
        constraint def Invariant_REQ_0027_The_lowering_transformation_maps_AST_declarations_into_schema_IR;
        doc /* \mu(f) = \begin{cases} \langle 0, \infty \rangle, & \text{if } f \text{ is \texttt{repeated}} \\ \langle 0, 1 \rangle, & \text{if } f \text{ is \texttt{optional}} \\ \langle 1, 1 \rangle, & \text{otherwise} \end{cases} */
        constraint def Invariant_REQ_0027_Multiplicity_mapping_satisfies;
        doc /* \begin{aligned} \forall M \in \mathcal{M}, \forall f \in \text{Fields}(M): &\quad \text{Tag}(f) \in [1, 2^{29}-1] \setminus [19000, 19999] \\ \forall M \in \mathcal{M}, \forall f_1 \ne f_2 \in \text{Fields}(M): &\quad \text{Tag}(f_1) \ne \text{Tag}(f_2) \\ \forall E \in \mathcal{E}: &\quad \exists v_0 \in \text{Values}(E) : (\text{Index}(v_0) = 0 \wedge \text{Tag}(v_0) = 0) \\ \forall M_{\text{child}} \in \text{NestedMessages}(M_{\text{parent}}): &\quad \text{FQP}(M_{\text{child}}) = \text{FQP}(M_{\text{parent}}) \mathbin{::} \text{Name}(M_{\text{child}}) \end{aligned} */
        constraint def Invariant_REQ_0027_Field_tag_validity_uniqueness_enum_defaults_and_nested_message_scoping_satisfy;
        doc /* \begin{aligned} \text{Tag}(f) \notin [1, 2^{29}-1] \setminus [19000, 19999] &\implies \text{Diagnostic}(f, \text{DiagnosticCode::E0103}) \in \mathcal{D} \\ (\exists f_1 \ne f_2 \in \text{Fields}(M) : \text{Tag}(f_1) = \text{Tag}(f_2)) &\implies \text{Diagnostic}(f_2, \text{DiagnosticCode::E0102}) \in \mathcal{D} \\ (\text{Index}(v_0) = 0 \wedge \text{Tag}(v_0) \ne 0) &\implies \text{Diagnostic}(E, \text{DiagnosticCode::E0103}) \in \mathcal{D} \\ (\mathcal{H}_{\text{syntax}} \ne \text{"proto3"} \vee \text{SyntaxFault}(\text{span})) &\implies \text{Diagnostic}(\text{span}, \text{DiagnosticCode::E0100}) \in \mathcal{D} \end{aligned} */
        constraint def Invariant_REQ_0027_Diagnostic_bindings_per_REQ_0191_satisfy;
        require Invariant_REQ_0027_FormalInvariant;
        require Invariant_REQ_0027_The_lowering_transformation_maps_AST_declarations_into_schema_IR;
        require Invariant_REQ_0027_Multiplicity_mapping_satisfies;
        require Invariant_REQ_0027_Field_tag_validity_uniqueness_enum_defaults_and_nested_message_scoping_satisfy;
        require Invariant_REQ_0027_Diagnostic_bindings_per_REQ_0191_satisfy;
        derived from REQ_0191_Standardized_Compiler_Diagnostic_Taxonomy_Source_Mapping;
        verify by AC_01_Canonical_Message_and_Field_Lowering;
        verify by AC_02_Field_Tag_Range_and_Reserved_Boundary_Enforcement;
        verify by AC_03_Duplicate_Field_Tag_Collision_Detection;
        verify by AC_04_Enum_Tag_Zero_Default_and_Nested_Scope_Resolution;
        satisfy by UniversalIngestionEngine;
    }
    doc /* UUIDv5: a9a1f791-92a5-55df-acd8-b9cab8846282 | Complexity: Class P (O(N)) | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0102, E0103 */
    requirement def REQ_0028_OMG_IDL_AST_Ingestion_for_Structs_Interfaces {
        id = "REQ-0028";
        text = "The compiler ingestion engine shall parse OMG IDL (.idl) specifications into intermediate schema representation (IR) without external preprocessors. The engine shall lower IDL module namespaces to nested IngestedPackage scopes, struct definitions to IngestedClassifierDef data types with sequential";
        attribute uuidv5 : String = "a9a1f791-92a5-55df-acd8-b9cab8846282";
        attribute complexity_class : String = "Class P (O(N))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0102, E0103";
        attribute ac_01_canonical_struct_union_and_typedef_lowering : String = "Given: Valid IDL declaring struct Struct_Alpha { long field_1; boolean flag_2; };, union Union_Beta switch (short) { case 1: long v1; default: boolean v2; };, and typedef double Real_Alias;. When: The parser ingests and lowers declarations into intermediate schema IR. Then: Emits IngestedClassifierDef (kind: Struct, Union) with typed members and discriminant cases, emits IngestedTypeDef, and allocates nodes in contiguous AST storage with stable offsets. Diagnostic: E0102 on syntax fault; E0103 on discriminant type mismatch.";
        attribute ac_02_interface_operation_and_directional_parameter_partitioning : String = "Given: Interface Interface_Gamma declaring long op_step(in Real p_in, out Real p_out, inout Real p_state);. When: Lowering the operation declaration into intermediate schema IR. Then: Emits IngestedAction where p_in is partitioned into inputs, p_out into outputs, p_state into both inputs and outputs, and non-void return long synthesizes output parameter result : long. Diagnostic: E0102 on parameter signature failure.";
        attribute ac_03_missing_or_invalid_parameter_direction_diagnostic_binding : String = "Given: IDL operation omitting an explicit direction token (void op_bad(Real p_missing);) or specifying an illegal keyword (void op_bad2(input Real p_bad);). When: Validating parameter direction tokens during parsing. Then: Emits diagnostic E0102 with precise byte span, substitutes sentinel _{action}, and continues parsing subsequent declarations. Diagnostic: E0102 emitted on missing or invalid parameter direction.";
        attribute ac_04_sequence_and_fixed_array_multiplicity_lowering : String = "Given: IDL declarations containing bounded sequence sequence<Real, 16>, unbounded sequence sequence<Real>, and fixed array Real arr[8]. When: Lowering type expressions into schema IR attributes. Then: Assigns exact multiplicity intervals 0, 16 , 0, , and 8, 8 respectively. Diagnostic: E0103 on malformed sequence bound.";
        doc /* \begin{aligned} \text{module } M \{ \mathcal{D}^* \} &\longmapsto \text{IngestedPackage}(M, \text{members}: \mathcal{T}_{\text{idl}}(\mathcal{D}^*)) \\ \text{interface } I \{ \mathcal{F}^* \} &\longmapsto \text{IngestedPart}(I, \text{is\_interface}: \text{true}, \text{members}: \mathcal{T}_{\text{idl}}(\mathcal{F}^*)) \\ \text{struct } S \{ \mathcal{M}^* \} &\longmapsto \text{IngestedClassifierDef}(S, \text{kind}: \text{Struct}, \text{fields}: \mathcal{T}_{\text{idl}}(\mathcal{M}^*)) \\ \text{union } U \text{ switch}(D) \{ \mathcal{B}^* \} &\longmapsto \text{IngestedClassifierDef}(U, \text{kind}: \text{Union}, \text{disc}: D, \text{cases}: \mathcal{T}_{\text{idl}}(\mathcal{B}^*)) \\ \text{enum } E \{ e_0, \dots, e_{k-1} \} &\longmapsto \text{IngestedClassifierDef}(E, \text{kind}: \text{Enum}, \text{variants}: \langle e_0, \dots, e_{k-1} \rangle) \\ \text{typedef } T \; A &\longmapsto \text{IngestedTypeDef}(A, \text{target}: T) \end{aligned} */
        constraint def Invariant_REQ_0028_FormalInvariant;
        doc /* \begin{aligned} \mathcal{P}_{\text{in}}(\text{op}) &= \{ \langle \text{name}(p), \text{type}(p) \rangle \mid p \in \text{Params}(\text{op}) \wedge \text{dir}(p) \in \{ \text{In}, \text{InOut} \} \} \\ \mathcal{P}_{\text{out}}(\text{op}) &= \{ \langle \text{name}(p), \text{type}(p) \rangle \mid p \in \text{Params}(\text{op}) \wedge \text{dir}(p) \in \{ \text{Out}, \text{InOut} \} \} \cup \text{ReturnParam}(T_{\text{ret}}) \\ \text{ReturnParam}(T_{\text{ret}}) &= \begin{cases} \{ \langle \text{result}, T_{\text{ret}} \rangle \}, & T_{\text{ret}} \ne \text{void} \\ \emptyset, & T_{\text{ret}} = \text{void} \end{cases} \\ \forall p &\in \text{Params}(\text{op}), \quad \text{dir}(p) \in \{ \text{In}, \text{Out}, \text{InOut} \} \end{aligned} */
        constraint def Invariant_REQ_0028_For_an_operation_lowering;
        doc /* \begin{aligned} \mu(T) &= \begin{cases} \langle 0, K \rangle, & T = \text{sequence}\langle T', K \rangle \\ \langle 0, \infty \rangle, & T = \text{sequence}\langle T' \rangle \\ \langle K, K \rangle, & T = T'[K] \\ \langle 1, 1 \rangle, & \text{otherwise} \end{cases} \\ \forall M &\in \mathcal{M}_{\text{pkg}}, \quad (M, M) \notin \mathcal{E}_{\text{mod}}^+ \end{aligned} */
        constraint def Invariant_REQ_0028_Multiplicity_mapping_and_module_DAG_acyclicity_satisfy;
        doc /* \begin{aligned} \text{dir}(p) \notin \{ \text{In}, \text{Out}, \text{InOut} \} \vee \text{SyntaxFault}(\text{span}) &\implies \text{DiagnosticCode::E0102} \in \mathcal{D} \\ \text{UnsupportedType}(T) \vee \text{DiscriminantMismatch}(U) &\implies \text{DiagnosticCode::E0103} \in \mathcal{D} \end{aligned} */
        constraint def Invariant_REQ_0028_Diagnostic_bindings_per_REQ_0191_satisfy;
        require Invariant_REQ_0028_FormalInvariant;
        require Invariant_REQ_0028_For_an_operation_lowering;
        require Invariant_REQ_0028_Multiplicity_mapping_and_module_DAG_acyclicity_satisfy;
        require Invariant_REQ_0028_Diagnostic_bindings_per_REQ_0191_satisfy;
        derived from REQ_0191_Standardized_Compiler_Diagnostic_Taxonomy_Source_Mapping;
        verify by AC_01_Canonical_Struct_Union_and_Typedef_Lowering;
        verify by AC_02_Interface_Operation_and_Directional_Parameter_Partitioning;
        verify by AC_03_Missing_or_Invalid_Parameter_Direction_Diagnostic_Binding;
        verify by AC_04_Sequence_and_Fixed_Array_Multiplicity_Lowering;
        satisfy by UniversalIngestionEngine;
    }
    doc /* UUIDv5: 06c88f7d-e7cc-5a68-b123-8eee898e6222 | Complexity: Class P (O(N)) | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100, E0102, E0103, E0199 */
    requirement def REQ_0029_OpenAPI_Schema_Ingestion_for_Paths_Operations {
        id = "REQ-0029";
        text = "The Universal Schema Ingestion Engine shall parse OpenAPI 3.0 and 3.1 specifications in JSON and YAML formats into intermediate schema IR without external dependencies. The root document lowers to an IngestedPackage (e.g., Package_0::Spec_Alpha). Schema objects under components/schemas lower to";
        attribute uuidv5 : String = "06c88f7d-e7cc-5a68-b123-8eee898e6222";
        attribute complexity_class : String = "Class P (O(N))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100, E0102, E0103, E0199";
        attribute ac_01_canonical_schema_and_path_lowering : String = "Given: Valid OpenAPI 3.0/3.1 document declaring schema Schema_Alpha and path /endpoint_alpha (get, post). When: Ingesting and lowering into schema IR. Then: Synthesizes IngestedPackage, IngestedClassifierDef, and two IngestedAction entities in AST storage without diagnostics. Diagnostic: E0100 or E0102 on lowering failure.";
        attribute ac_02_path_route_template_and_parameter_consistency : String = "Given: Path /endpoint_alpha/{param_x} with operation parameter param_x (in: path, required: true). When: Validating route template consistency. Then: Maps param_x to required input parameter with multiplicity 1, 1 ; template mismatch triggers diagnostic. Diagnostic: Emits E0102 on route mismatch.";
        attribute ac_03_request_body_and_response_payload_binding : String = "Given: Operation declaring requestBody schema Payload_0 (required: true) and response status 200 schema Response_Alpha. When: Lowering payloads. Then: Synthesizes input parameter payload : Payload_0 ( 1, 1 ) and output parameter result : Response_Alpha ( 1, 1 ). Diagnostic: Emits E0102 or E0103 on binding failure.";
        attribute ac_04_json_pointer_dereferencing_and_cycle_safety : String = "Given: Schemas with local pointer references (#/components/schemas/Schema_Alpha) including recursive reference (Schema_Beta to Schema_Beta). When: Resolving references during type binding. Then: Resolves targets via handle indirection without recursion overflow; unresolvable reference emits diagnostic. Diagnostic: Emits E0102 on unresolvable target.";
        doc /* \begin{aligned} \mathcal{O} &= \langle V_{\text{version}}, \mathcal{I}_{\text{info}}, \mathcal{P}_{\text{paths}}, \mathcal{C}_{\text{components}} \rangle \\ \text{components/schemas/}S &\longmapsto \text{IngestedClassifierDef}(S, \text{kind}: \text{ClassifierKind::Struct}, \text{fields}: \mathcal{T}_{\text{schema}}(S)) \\ \text{paths/}P/m &\longmapsto \text{IngestedAction}(\text{ActionId}(P, m), \text{endpoint}: u(P), \text{inputs}: \mathcal{P}_{\text{in}}(P, m), \text{outputs}: \mathcal{P}_{\text{out}}(P, m)) \end{aligned} */
        constraint def Invariant_REQ_0029_FormalInvariant;
        doc /* \begin{aligned} \text{ActionId}(P, m) &= \begin{cases} \text{id}_{\text{op}}, & \text{if } \text{operationId}(P, m) = \text{id}_{\text{op}} \ne \bot \\ m \mathbin{\Vert} \text{"\_"} \mathbin{\Vert} \text{Sanitize}(u(P)), & \text{otherwise} \end{cases} \\ (P_1, m_1) \ne (P_2, m_2) &\implies \text{ActionId}(P_1, m_1) \ne \text{ActionId}(P_2, m_2) \\ \forall P \in \mathcal{P}_{\text{paths}}, \, \forall m \in \text{Ops}(P), &\quad \{ \text{name}(p) \mid p \in \text{Params}(P, m) \;\wedge\; \text{in}(p) = \text{Path} \} = \text{Vars}(u(P)) \end{aligned} */
        constraint def Invariant_REQ_0029_Operation_action_identifier_synthesis_uniqueness_and_path_parameter_bijection_satisfy;
        doc /* \begin{aligned} \text{requestBody}(P, m) &\longmapsto \text{IngestedAttr}(\text{"payload"}, \text{type}: \tau_{\text{req}}, \text{mult}: \mu_{\text{req}}) \\ \text{responses}(P, m, c) &\longmapsto \text{IngestedAttr}(\text{"result"}, \text{type}: \tau_{\text{resp}}, \text{mult}: \langle 1, 1 \rangle), \quad \forall c \in [200, 299] \\ \text{ResolveRef}(R, \mathcal{O}) &= \begin{cases} \mathcal{C}_{\text{components}}.\mathcal{S}_{\text{schemas}}[S], & \text{if } R = \text{"\#/components/schemas/"} \mathbin{\Vert} S \;\wedge\; S \in \text{dom}(\mathcal{C}_{\text{components}}.\mathcal{S}_{\text{schemas}}) \\ \bot, & \text{otherwise} \end{cases} \end{aligned} */
        constraint def Invariant_REQ_0029_Payload_schema_lowering_and_reference_resolution_satisfy;
        doc /* \begin{aligned} \text{UUID}(u) &= \text{UUIDv5}(\text{OID}_{\text{DEAP}}, \text{FQN}(u)) \\ (V_{\text{version}} \notin \{ \text{"3.0.0"}, \dots, \text{"3.1.0"} \} \;\vee\; \text{HasSyntaxError}(t)) &\implies \text{DiagnosticCode::E0100} \in \mathcal{D} \\ (\text{ActionId}(P_1, m_1) = \text{ActionId}(P_2, m_2) \;\vee\; \text{PathMismatch} \;\vee\; \text{ResolveRef} = \bot) &\implies \text{DiagnosticCode::E0102} \in \mathcal{D} \\ \text{IsUnsupportedType}(\tau) &\implies \text{DiagnosticCode::E0103} \in \mathcal{D} \\ |\mathcal{D}| \ge C_{\text{max\_errors}} &\implies \text{DiagnosticCode::E0199} \in \mathcal{D} \end{aligned} */
        constraint def Invariant_REQ_0029_Deterministic_UUIDv5_entity_generation_and_diagnostic_code_binding_per_REQ_0191_satisfy;
        require Invariant_REQ_0029_FormalInvariant;
        require Invariant_REQ_0029_Operation_action_identifier_synthesis_uniqueness_and_path_parameter_bijection_satisfy;
        require Invariant_REQ_0029_Payload_schema_lowering_and_reference_resolution_satisfy;
        require Invariant_REQ_0029_Deterministic_UUIDv5_entity_generation_and_diagnostic_code_binding_per_REQ_0191_satisfy;
        derived from REQ_0191_Standardized_Compiler_Diagnostic_Taxonomy_Source_Mapping;
        verify by AC_01_Canonical_Schema_and_Path_Lowering;
        verify by AC_02_Path_Route_Template_and_Parameter_Consistency;
        verify by AC_03_Request_Body_and_Response_Payload_Binding;
        verify by AC_04_JSON_Pointer_Dereferencing_and_Cycle_Safety;
        satisfy by UniversalIngestionEngine;
    }
    doc /* UUIDv5: bceb80d9-77ea-515d-ae33-a6759f04e5a4 | Complexity: Class P (O(N)) | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100, E0102, E0103, E0199 */
    requirement def REQ_0030_XML_Schema_Definition {
        id = "REQ-0030";
        text = "The Universal Schema Ingestion Engine shall provide a native XML Schema Definition (W3C XSD) and Architecture Description Language (ADL) parser that ingests structural schemas and component topologies into intermediate schema IR without external toolchains. The parser must execute via a";
        attribute uuidv5 : String = "bceb80d9-77ea-515d-ae33-a6759f04e5a4";
        attribute complexity_class : String = "Class P (O(N))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100, E0102, E0103, E0199";
        attribute ac_01_canonical_component_and_complex_type_lowering : String = "Given: Valid XSD/ADL document declaring complex type Type_Alpha and component Component_Alpha with attributes param_x : Real and param_y : Int32. When: Lowering the XML AST into intermediate schema IR. Then: Emits IngestedPart entities containing IngestedAttr entries with exact scalar types in AST storage without diagnostics. Diagnostic: E0100 on XML grammar failure.";
        attribute ac_02_port_directionality_mapping_for_provided_required_and_bidirectional_ports : String = "Given: Component Component_Alpha declaring provided port Port_1 (P-Port), required port Port_2 (R-Port), and bidirectional port Port_3. When: Lowering port definitions into intermediate schema IR. Then: Maps Port_1 to Out, Port_2 to In, and Port_3 to InOut within generated IngestedPort entities. Diagnostic: E0102 on missing or invalid port direction specification.";
        attribute ac_03_connector_topology_lowering_and_directional_compatibility_gate : String = "Given: ADL connector Conn_0 connecting Component_Alpha::Port_1 (Out) to Component_Beta::Port_2 (In), and invalid connector Conn_1 connecting Component_Beta::Port_2 (In) to Component_Alpha::Port_1 (Out). When: Validating topological connections and port endpoint directions. Then: Lowers Conn_0 into an IngestedConnection link; rejects Conn_1 due to reversed flow, substitutes sentinel _{conn}, and emits a structured diagnostic. Diagnostic: E0102 on directional mismatch or unresolvable endpoint path.";
        attribute ac_04_occurrence_bound_multiplicity_and_validation : String = "Given: XSD elements with valid bounds (minOccurs=\"0\", maxOccurs=\"1\", and minOccurs=\"1\", maxOccurs=\"unbounded\"), and an invalid element with minOccurs=\"5\" and maxOccurs=\"2\". When: Parsing occurrence attributes and computing multiplicity interval (e). Then: Lowers valid elements with multiplicity 0, 1 and 1, ; rejects invalid declaration with minOccurs maxOccurs, logs a structured diagnostic, and substitutes sentinel _{attr}. Diagnostic: E0103 on inverted occurrence bounds (minOccurs maxOccurs).";
        doc /* \mathcal{X}_{\text{xml}} = \langle \mathcal{N}_{\text{ns}}, \mathcal{T}_{\text{types}}, \mathcal{C}_{\text{comps}}, \mathcal{K}_{\text{conns}} \rangle */
        constraint def Invariant_REQ_0030_FormalInvariant;
        doc /* \begin{aligned} \text{xs:complexType } T &\longmapsto \text{IngestedPart}(T, \text{attrs}: \mathcal{F}_{\text{attrs}}(T)) \\ \text{xs:element } e &\longmapsto \text{IngestedAttr}(e, \text{type}: \tau(e), \text{mult}: \mu(e)) \\ \text{adl:component } C &\longmapsto \text{IngestedPart}(C, \text{ports}: \mathcal{F}_{\text{ports}}(C)) \\ \text{adl:port } p &\longmapsto \text{IngestedPort}(p, \text{dir}: \text{Direction}(p)) \\ \text{adl:connector } c &\longmapsto \text{IngestedConnection}(c.\text{src}, c.\text{tgt}) \end{aligned} */
        constraint def Invariant_REQ_0030_The_schema_lowering_transformation;
        doc /* \text{Direction}(p) = \begin{cases} \text{Out}, & \text{if } p \in \text{ProvidedPorts} \\ \text{In}, & \text{if } p \in \text{RequiredPorts} \\ \text{InOut}, & \text{if } p \in \text{BidirectionalPorts} \end{cases} */
        constraint def Invariant_REQ_0030_Port_directionality_mapping_satisfies;
        doc /* \begin{aligned} \mu(e) = \langle \text{minOccurs}(e), \text{maxOccurs}(e) \rangle &\quad \wedge \quad \text{minOccurs}(e) \le \text{maxOccurs}(e) \\ \text{Compatible}(P_{\text{src}}, P_{\text{tgt}}) &\iff \text{Direction}(P_{\text{src}}) \in \{ \text{Out}, \text{InOut} \} \;\wedge\; \text{Direction}(P_{\text{tgt}}) \in \{ \text{In}, \text{InOut} \} \end{aligned} */
        constraint def Invariant_REQ_0030_Occurrence_multiplicity_and_connector_compatibility_satisfy;
        doc /* \delta_{\text{pda}} : Q \times (\Sigma \cup \{ \epsilon \}) \times \Gamma \longrightarrow \mathcal{P}(Q \times \Gamma^*) */
        constraint def Invariant_REQ_0030_The_Zero_Regex_XML_Processing_Invariant_enforces_deterministic_pushdown_transition;
        doc /* \begin{aligned} \text{XMLMalformed}(\text{span}) &\implies \text{Diagnostic}(\text{span}, \text{DiagnosticCode::E0100}) \in \mathcal{D} \\ \text{minOccurs}(e) \gt \text{maxOccurs}(e) &\implies \text{Diagnostic}(\text{span}(e), \text{DiagnosticCode::E0103}) \in \mathcal{D} \\ \text{IsUnsupportedType}(T) &\implies \text{Diagnostic}(\text{span}(T), \text{DiagnosticCode::E0103}) \in \mathcal{D} \\ (\neg\text{Compatible}(P_{\text{src}}, P_{\text{tgt}}) \;\vee\; \text{UnresolvedEndpoint}(P)) &\implies \text{Diagnostic}(\text{span}(c), \text{DiagnosticCode::E0102}) \in \mathcal{D} \\ |\mathcal{D}| \ge C_{\text{max\_errors}} &\implies \text{Diagnostic}(\text{span}(\text{EOF}), \text{DiagnosticCode::E0199}) \in \mathcal{D} \end{aligned} */
        constraint def Invariant_REQ_0030_Diagnostic_code_bindings_per_REQ_0191_satisfy;
        require Invariant_REQ_0030_FormalInvariant;
        require Invariant_REQ_0030_The_schema_lowering_transformation;
        require Invariant_REQ_0030_Port_directionality_mapping_satisfies;
        require Invariant_REQ_0030_Occurrence_multiplicity_and_connector_compatibility_satisfy;
        require Invariant_REQ_0030_The_Zero_Regex_XML_Processing_Invariant_enforces_deterministic_pushdown_transition;
        require Invariant_REQ_0030_Diagnostic_code_bindings_per_REQ_0191_satisfy;
        derived from REQ_0191_Standardized_Compiler_Diagnostic_Taxonomy_Source_Mapping;
        verify by AC_01_Canonical_Component_and_Complex_Type_Lowering;
        verify by AC_02_Port_Directionality_Mapping_for_Provided_Required_and_Bidirectional_Ports;
        verify by AC_03_Connector_Topology_Lowering_and_Directional_Compatibility_Gate;
        verify by AC_04_Occurrence_Bound_Multiplicity_and_Validation;
        satisfy by UniversalIngestionEngine;
    }
    doc /* UUIDv5: a6c05a65-2060-5b96-ac10-393764de929f | Complexity: Class P (O( | Standard: IEEE 29148-2018 / RFC 2119 / OMG SysML v2 | Diagnostics: E0102, E0103, E0199 */
    requirement def REQ_0031_Canonical_SysML_v2_Textual_Model_Emission {
        id = "REQ-0031";
        text = "The compiler ingestion engine shall provide a canonical SysML v2 textual concrete syntax emitter (E_{sysml}) that serializes parsed intermediate schema IR entities into standard-compliant OMG SysML v2 textual notation, writing the unified model atomically to schema/model.sysml.";
        attribute uuidv5 : String = "a6c05a65-2060-5b96-ac10-393764de929f";
        attribute complexity_class : String = "Class P (O(";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119 / OMG SysML v2";
        attribute diagnostic_codes : String = "E0102, E0103, E0199";
        attribute ac_01_canonical_sysml_v2_syntax_emission : String = "Given: Schema IR with Package_0, Classifier_Alpha, ports Port_1, Port_2, attributes, connections, constraints, and actions. When: Synthesizing the canonical SysML v2 model via E_{sysml}. Then: Emits valid concrete syntax to schema/model.sysml parsing cleanly with zero diagnostics. Diagnostic: E0102 on syntax mapping failure.";
        attribute ac_02_deterministic_qualified_name_symbol_ordering : String = "Given: Schema IR declaring child entities in reverse order within Package_0. When: Lowering child entities to text. Then: Emits child elements in ascending order _{sym} by fully qualified name per REQ-0032. Diagnostic: E0102 on ordering violation.";
        attribute ac_03_identifier_sanitization_and_keyword_escaping : String = "Given: Schema IR entities with identifiers colliding with reserved keywords (package, part, def, attribute, connect, flow, action). When: Serializing identifier tokens. Then: Encloses colliding identifiers in single backticks (e.g., package ) while emitting valid identifiers unescaped. Diagnostic: E0102 on escaping failure.";
        attribute ac_04_atomic_file_persistence_and_integrity : String = "Given: Schema IR targeting schema/model.sysml with simulated failure prior to AtomicRename. When: Executing file persistence to disk. Then: Destination schema/model.sysml retains its prior valid state or transitions completely, leaving zero corrupted files. Diagnostic: E0199 on I/O write or rename failure.";
        doc /* \begin{aligned} \mathcal{E}_{\text{sysml}}(P) &\longmapsto \text{"package "} \mathbin{\Vert} \text{Sanitize}(P) \mathbin{\Vert} \text{" \{ ... \}"} \\ \mathcal{E}_{\text{sysml}}(C) &\longmapsto \text{"part def "} \mathbin{\Vert} \text{Sanitize}(C) \mathbin{\Vert} \text{" \{ ... \}"} \\ \mathcal{E}_{\text{sysml}}(p) &\longmapsto \text{DirPrefix}(p) \mathbin{\Vert} \text{"port "} \mathbin{\Vert} \text{Sanitize}(p) \mathbin{\Vert} \text{" : "} \mathbin{\Vert} \text{type}(p) \mathbin{\Vert} \text{";"} \\ \mathcal{E}_{\text{sysml}}(a) &\longmapsto \text{"attribute "} \mathbin{\Vert} \text{Sanitize}(a) \mathbin{\Vert} \text{" : "} \mathbin{\Vert} \text{type}(a) \mathbin{\Vert} \text{MultSuffix}(a) \mathbin{\Vert} \text{";"} \\ \mathcal{E}_{\text{sysml}}(k) &\longmapsto \text{"connect "} \mathbin{\Vert} \text{path}(k_{\text{src}}) \mathbin{\Vert} \text{" to "} \mathbin{\Vert} \text{path}(k_{\text{tgt}}) \mathbin{\Vert} \text{";"} \\ \mathcal{E}_{\text{sysml}}(c) &\longmapsto \text{"assert constraint "} \mathbin{\Vert} \text{Sanitize}(c) \mathbin{\Vert} \text{" \{ "} \mathbin{\Vert} \text{expr}(c) \mathbin{\Vert} \text{" \};"} \\ \mathcal{E}_{\text{sysml}}(\alpha) &\longmapsto \text{"action def "} \mathbin{\Vert} \text{Sanitize}(\alpha) \mathbin{\Vert} \text{" \{ ... \}"} \end{aligned} */
        constraint def Invariant_REQ_0031_FormalInvariant;
        doc /* \text{DirPrefix}(p) = \begin{cases} \text{"in "}, & \text{dir}(p) = \text{In} \\ \text{"out "}, & \text{dir}(p) = \text{Out} \\ \text{"inout "}, & \text{dir}(p) = \text{InOut} \\ \varepsilon, & \text{dir}(p) = \bot \end{cases} \qquad \text{Sanitize}(s) = \begin{cases} \text{"\`"} \mathbin{\Vert} s \mathbin{\Vert} \text{"\`"}, & s \in \mathcal{K}_{\text{sysml}} \lor \neg\text{IsIdent}(s) \\ s, & \text{otherwise} \end{cases} */
        constraint def Invariant_REQ_0031_Directional_port_prefixes_and_identifier_sanitization_satisfy;
        doc /* \begin{aligned} \forall u \in \mathcal{V}_{\text{IR}}, \; \forall c_i, c_j \in \text{Children}(u): &\quad i \lt j \implies \text{FQN}(c_i) \prec_{\text{sym}} \text{FQN}(c_j) \\ \mathcal{S}_{\text{model}} = \mathcal{E}_{\text{sysml}}(\rho) &\implies \mathcal{S}_{\text{model}} \in \mathcal{L}(\mathcal{G}_{\text{sysml}}) \;\wedge\; \text{Diagnostics}(\text{Parse}_{\text{sysml}}(\mathcal{S}_{\text{model}})) = \emptyset \\ \text{WriteModel}(\mathcal{S}_{\text{model}}) &\iff \text{AtomicRename}(\text{TempFile}, \text{"schema/model.sysml"}) \\ \text{IOError}(\text{TempFile}) \lor \text{IOError}(\text{AtomicRename}) &\implies \text{DiagnosticCode::E0199} \in \mathcal{D} \\ \text{SyntaxFault}(u) &\implies \text{DiagnosticCode::E0102} \in \mathcal{D} \\ \text{UnresolvedType}(T) &\implies \text{DiagnosticCode::E0103} \in \mathcal{D} \end{aligned} */
        constraint def Invariant_REQ_0031_Deterministic_qualified_name_ordering_syntax_grammar_membership_atomic_persistence_and_diagnostic_bindings_satisfy;
        require Invariant_REQ_0031_FormalInvariant;
        require Invariant_REQ_0031_Directional_port_prefixes_and_identifier_sanitization_satisfy;
        require Invariant_REQ_0031_Deterministic_qualified_name_ordering_syntax_grammar_membership_atomic_persistence_and_diagnostic_bindings_satisfy;
        derived from REQ_0003_Bitwise_Determinism;
        derived from REQ_0032_Deterministic_Qualified_Name_Symbol_Sorting_for_Canonical_Model_Emission;
        verify by AC_01_Canonical_SysML_v2_Syntax_Emission;
        verify by AC_02_Deterministic_Qualified_Name_Symbol_Ordering;
        verify by AC_03_Identifier_Sanitization_and_Keyword_Escaping;
        verify by AC_04_Atomic_File_Persistence_and_Integrity;
        satisfy by UniversalIngestionEngine;
    }
    doc /* UUIDv5: 2f859e57-faf5-5088-8de9-da3a720d19be | Complexity: Class P (O(L)) | Standard: IEEE 29148-2018 / RFC 2119 / OMG SysML v2 | Diagnostics: E0100, E0102, E0199 */
    requirement def REQ_0032_Deterministic_Qualified_Name_Symbol_Sorting_for_Canonical_Model_Emission {
        id = "REQ-0032";
        text = "The textual emission and model serialization engine shall sort all model declarations--packages, classifiers, ports, attributes, connections, constraints, and behaviors--strictly by canonical fully qualified name prior to canonical model emission (schema/model.sysml, REQ-0031) and serialized";
        attribute uuidv5 : String = "2f859e57-faf5-5088-8de9-da3a720d19be";
        attribute complexity_class : String = "Class P (O(L))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119 / OMG SysML v2";
        attribute diagnostic_codes : String = "E0100, E0102, E0199";
        attribute ac_01_permutation_invariance_and_zero_diff_churn : String = "Given: AST with 500 declarations across packages, classifiers, ports, and attributes in arbitrary order. When: Sorting symbols by qualified name and emitting canonical SysML v2 text across 20 randomized orderings. Then: All 20 emitted output byte streams produce identical SHA-256 digests, confirming zero diff churn (Diff = ). Diagnostic: E0100 on bitwise determinism mismatch.";
        attribute ac_02_hierarchical_scope_qualified_name_ordering : String = "Given: Nested AST elements (Package_B::Classifier_1, Package_A::Subsystem_1::Classifier_Alpha, Package_A::Classifier_Beta). When: Sorting declarations prior to canonical emission. Then: Emits top-level packages alphabetically (Package_A before Package_B), and inside Package_A, orders members alphabetically (Package_A::Classifier_Beta before Package_A::Subsystem_1). Diagnostic: E0100 on scope ordering inversion.";
        attribute ac_03_lexicographical_ascii_byte_order_and_case_precedence : String = "Given: Sibling declarations named Classifier_1, Classifier_10, Classifier_2, Classifier_Alpha, Classifier_alpha. When: Applying qualified-name symbol sorting. Then: Sorts strictly by ASCII byte value (Classifier_1 _{lex} Classifier_10 _{lex} Classifier_2 _{lex} Classifier_Alpha _{lex} Classifier_alpha). Diagnostic: E0100 on collation order mismatch.";
        attribute ac_04_duplicate_qualified_name_collision_recovery : String = "Given: AST containing two distinct classifier declarations with identical qualified name Package_0::Classifier_Alpha and identical metaclass in the same scope. When: Executing symbol sorting and verification. Then: Detects symbol collision, records diagnostic E0102, substitutes sentinel _{sym}, and resumes sorting remaining declarations. Diagnostic: E0102 on duplicate symbol declaration collision.";
        doc /* d = \langle \text{id}, \text{parent}, \tau \rangle \in \mathcal{S}_{\text{sym}} \times (\mathcal{D} \cup \{ \bot \}) \times \mathcal{T}_{\text{kind}} */
        constraint def Invariant_REQ_0032_FormalInvariant;
        doc /* \text{QName}(d) = \begin{cases} \text{id}(d), & \text{if } \text{parent}(d) = \bot \\ \text{QName}(\text{parent}(d)) \mathbin{\Vert} \text{"::"} \mathbin{\Vert} \text{id}(d), & \text{otherwise} \end{cases} */
        constraint def Invariant_REQ_0032_The_canonical_fully_qualified_name_function;
        doc /* s_1 \prec_{\text{lex}} s_2 \iff \exists i \ge 0 : (i \le \min(|s_1|, |s_2|)) \wedge (\forall j \lt i, s_1[j] = s_2[j]) \wedge \big((i \lt \min(|s_1|, |s_2|) \implies s_1[i] \lt s_2[i]) \vee (i = |s_1| \lt |s_2|)\big) */
        constraint def Invariant_REQ_0032_FormalInvariant_2;
        doc /* d_1 \prec_{\text{sym}} d_2 \iff (\text{QName}(d_1) \prec_{\text{lex}} \text{QName}(d_2)) \vee \big(\text{QName}(d_1) = \text{QName}(d_2) \wedge \text{KindOrdinal}(\tau(d_1)) \lt \text{KindOrdinal}(\tau(d_2))\big) */
        constraint def Invariant_REQ_0032_The_canonical_declaration_ordering_relation;
        doc /* \forall d_1 \ne d_2 \in \mathcal{D}_{\text{valid}}, \quad (d_1 \prec_{\text{sym}} d_2) \oplus (d_2 \prec_{\text{sym}} d_1) */
        constraint def Invariant_REQ_0032_Strict_Total_Ordering_Invariant;
        doc /* \forall \mathcal{A}_1, \mathcal{A}_2 \in \Pi(\mathcal{A}), \forall \sigma_1, \sigma_2: \quad \mathcal{E}(\mathcal{A}_1; \sigma_1) = \mathcal{E}(\mathcal{A}_2; \sigma_2) \wedge \text{SHA256}(\mathcal{E}(\mathcal{A}_1; \sigma_1)) = \text{SHA256}(\mathcal{E}(\mathcal{A}_2; \sigma_2)) */
        constraint def Invariant_REQ_0032_Zero_Diff_Churn_Determinism;
        doc /* \begin{aligned} (\exists d_1 \ne d_2 : \text{QName}(d_1) = \text{QName}(d_2) \wedge \tau(d_1) = \tau(d_2)) &\implies \text{Diagnostic}(d_2, \text{DiagnosticCode::E0102}) \in \mathcal{D}_{\text{diag}} \\ \text{SHA256}(\mathcal{E}(\mathcal{A}_1)) \ne \text{SHA256}(\mathcal{E}(\mathcal{A}_2)) &\implies \text{Diagnostic}(\text{span}_{\text{root}}, \text{DiagnosticCode::E0100}) \in \mathcal{D}_{\text{diag}} \\ |\mathcal{D}_{\text{diag}}| \ge C_{\text{max\_errors}} &\implies \exists D_{\text{term}} \in \mathcal{D}_{\text{diag}} : \text{code}(D_{\text{term}}) = \text{DiagnosticCode::E0199} \end{aligned} */
        constraint def Invariant_REQ_0032_Diagnostic_Code_Binding_Invariants;
        require Invariant_REQ_0032_FormalInvariant;
        require Invariant_REQ_0032_The_canonical_fully_qualified_name_function;
        require Invariant_REQ_0032_FormalInvariant_2;
        require Invariant_REQ_0032_The_canonical_declaration_ordering_relation;
        require Invariant_REQ_0032_Strict_Total_Ordering_Invariant;
        require Invariant_REQ_0032_Zero_Diff_Churn_Determinism;
        require Invariant_REQ_0032_Diagnostic_Code_Binding_Invariants;
        derived from REQ_0031_Canonical_SysML_v2_Textual_Model_Emission;
        derived from REQ_0044_Lossless_AST_Serialization;
        derived from REQ_0145_Automated_N_2_System_Interface_Matrix_Construction;
        derived from REQ_0178_Manifest_Driven_File_Persistence_with_XXH3_Content_Fingerprinting;
        derived from REQ_0019_Table_Row_to_Entity_Structural_Mapping;
        verify by AC_01_Permutation_Invariance_and_Zero_Diff_Churn;
        verify by AC_02_Hierarchical_Scope_Qualified_Name_Ordering;
        verify by AC_03_Lexicographical_ASCII_Byte_Order_and_Case_Precedence;
        verify by AC_04_Duplicate_Qualified_Name_Collision_Recovery;
        satisfy by UniversalIngestionEngine;
    }
    doc /* UUIDv5: 2d5fcd67-d3b5-5eb5-bfe7-275743473fc2 | Complexity: Class P (O(B)) | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100, E0102 */
    requirement def REQ_0033_Multi_File_Cryptographic_Digest_Computation {
        id = "REQ-0033";
        text = "The compiler ingestion engine shall compute, record, verify, and persist a multi-file cryptographic manifest recording SHA-256 content hashes, byte sizes, line counts, and AST entity counts for all workspace schema input files and the emitted canonical model (schema/model.sysml). The manifest shall";
        attribute uuidv5 : String = "2d5fcd67-d3b5-5eb5-bfe7-275743473fc2";
        attribute complexity_class : String = "Class P (O(B))";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100, E0102";
        attribute ac_01_multi_file_digest_manifest_generation_content_invariants : String = "Given: A workspace with input schemas (schema/Table_0.md, schema/Schema_1.proto, schema/Interface_2.idl) defining generic elements (Package_0, Classifier_Alpha, Port_1, Flow_A, param_x : Real) and emitted schema/model.sysml. When: The compiler executes schema ingestion and manifest generation. Then: .pipeline/schema-digest.json is generated containing a 5-tuple record for every file with matching SHA-256 hex string, exact byte count, line count, and AST entity count. Diagnostic: E0100 emitted if any file cannot be read or hashing fails.";
        attribute ac_02_deterministic_lexicographical_sorting_diff_invariance : String = "Given: Input schema files discovered across varying filesystem traversal orders. When: Manifest generation evaluates entry order and composite hash H_{root}. Then: Entries are sorted strictly by ascending ASCII path (P_{i} _{lex} P_{i+1}), producing identical composite hashes and 0-byte diff churn across repeated runs. Diagnostic: E0100 emitted on determinism failure or sorting inconsistency.";
        attribute ac_03_single_byte_invalidation_cache_model_reuse : String = "Given: A cached .pipeline/schema-digest.json file and subsequent compiler invocation. When: Live workspace digests are compared against the cached manifest (_{cache}). Then: Identical inputs reuse the pre-compiled AST and bypass re-emission of schema/model.sysml; modifying a single byte invalidates the cache and triggers full recompilation. Diagnostic: None emitted on valid cache hit or normal recompilation.";
        attribute ac_04_atomic_staging_crash_consistent_persistence : String = "Given: An active manifest write targeting .pipeline/schema-digest.json. When: The engine serializes the manifest document to disk. Then: Content is written to .pipeline/schema-digest.json.tmp.<pid>, flushed with fsync, and atomically renamed to .pipeline/schema-digest.json, preventing corrupted or partial file states. Diagnostic: E0100 emitted on filesystem write permission failure or disk saturation.";
        doc /* \text{Digest}(F_{i}) = \langle P_{i}, H_{i}, S_{i}, L_{i}, N_{i} \rangle */
        constraint def Invariant_REQ_0033_FormalInvariant;
        doc /* \begin{aligned} P_{i} &= \text{RelPath}(F_{i}, \text{Root}_{\text{ws}}) \in \mathcal{P}_{\text{rel}} \\ H_{i} &= \text{Hex}(\text{SHA256}(F_{i})) \in \Sigma_{\text{hex}}^{64} \\ S_{i} &= |F_{i}| \in \mathbb{N}_{0} \\ L_{i} &= \text{Lines}(F_{i}) \in \mathbb{N}_{0} \\ N_{i} &= |\text{Entities}(F_{i})| \in \mathbb{N}_{0} \end{aligned} */
        constraint def Invariant_REQ_0033_where;
        doc /* \forall i \in \{1, \dots, |\mathcal{F}_{\text{manifest}}| - 1\}, \quad P_{i} \prec_{\text{lex}} P_{i+1} */
        constraint def Invariant_REQ_0033_Deterministic_Lexicographical_Order_Invariant;
        doc /* \mathcal{H}_{\text{root}} = \text{Hex}\left( \text{SHA256}\left( \bigoplus_{i=1}^{|\mathcal{F}_{\text{manifest}}|} \left( P_{i} \mathbin{\Vert} \text{SYM}_{\text{DELIM}} \mathbin{\Vert} H_{i} \mathbin{\Vert} \text{SYM}_{\text{DELIM}} \mathbin{\Vert} \text{Dec}(S_{i}) \mathbin{\Vert} \text{SYM}_{\text{DELIM}} \mathbin{\Vert} \text{Dec}(L_{i}) \mathbin{\Vert} \text{SYM}_{\text{DELIM}} \mathbin{\Vert} \text{Dec}(N_{i}) \mathbin{\Vert} \text{SYM}_{\text{LF}} \right) \right) \right) */
        constraint def Invariant_REQ_0033_Composite_Root_Hash_Invariant;
        doc /* \text{UUID}_{\text{manifest}} = \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \mathcal{H}_{\text{root}}) */
        constraint def Invariant_REQ_0033_Deterministic_Manifest_UUIDv5_Invariant;
        doc /* \text{PersistManifest}(\mathcal{J}) \implies \text{Write}(\text{TmpPath}, \mathcal{J}) \;\wedge\; \text{FSync}(\text{TmpPath}) \;\wedge\; \text{AtomicRename}(\text{TmpPath}, \text{".pipeline/schema-digest.json"}) */
        constraint def Invariant_REQ_0033_Atomic_Persistence_Invariant;
        doc /* \Psi_{\text{cache}}(\mathcal{M}_{\text{cached}}, \mathcal{M}_{\text{active}}) \iff \left( \mathcal{M}_{\text{cached}}.\text{composite\_hash} = \mathcal{M}_{\text{active}}.\text{composite\_hash} \;\wedge\; |\mathcal{M}_{\text{cached}}.\text{entries}| = |\mathcal{M}_{\text{active}}.\text{entries}| \right) */
        constraint def Invariant_REQ_0033_Manifest_Cache_Invalidation_and_Reuse_Invariant;
        doc /* \Psi_{\text{cache}}(\mathcal{M}_{\text{cached}}, \mathcal{M}_{\text{active}}) \implies \text{ReuseCachedModel}() \;\wedge\; \text{BypassModelEmission}() */
        constraint def Invariant_REQ_0033_FormalInvariant_2;
        doc /* \begin{aligned} \text{IOError}(F_{i}) &\implies \text{Diagnostic}(P_{i}, \text{E0100}) \in \mathcal{D} \\ \text{MalformedJSON}(\mathcal{J}_{\text{disk}}) &\implies \text{Diagnostic}(\text{".pipeline/schema-digest.json"}, \text{E0102}) \in \mathcal{D} \end{aligned} */
        constraint def Invariant_REQ_0033_Diagnostic_Code_Binding_Invariant;
        require Invariant_REQ_0033_FormalInvariant;
        require Invariant_REQ_0033_where;
        require Invariant_REQ_0033_Deterministic_Lexicographical_Order_Invariant;
        require Invariant_REQ_0033_Composite_Root_Hash_Invariant;
        require Invariant_REQ_0033_Deterministic_Manifest_UUIDv5_Invariant;
        require Invariant_REQ_0033_Atomic_Persistence_Invariant;
        require Invariant_REQ_0033_Manifest_Cache_Invalidation_and_Reuse_Invariant;
        require Invariant_REQ_0033_FormalInvariant_2;
        require Invariant_REQ_0033_Diagnostic_Code_Binding_Invariant;
        verify by AC_01_Multi_File_Digest_Manifest_Generation_Content_Invariants;
        verify by AC_02_Deterministic_Lexicographical_Sorting_Diff_Invariance;
        verify by AC_03_Single_Byte_Invalidation_Cache_Model_Reuse;
        verify by AC_04_Atomic_Staging_Crash_Consistent_Persistence;
        satisfy by UniversalIngestionEngine;
    }
}
