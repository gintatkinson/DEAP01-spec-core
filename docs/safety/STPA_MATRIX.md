# STPA Safety & Failure Mode Tracking Matrix

## 1. Formal Safety Constraints
| SC ID | Constraint Statement / Description | Controller / Subsystem | Traceability / UCA |
| :--- | :--- | :--- | :--- |
| **SC-01** | Formal invariant for 7-tier bipartite DAG consistency | - | - |
| **SC-001** | System shall prevent Not providing during OA_01_Ingest_OEM_Artifacts | DEAPCompilerSystem | UCA-001 |
| **SC-002** | System shall prevent Providing during OA_01_Ingest_OEM_Artifacts | DEAPCompilerSystem | UCA-002 |
| **SC-003** | System shall prevent Too early / Too late / Out of order during OA_01_Ingest_OEM_Artifacts | DEAPCompilerSystem | UCA-003 |
| **SC-004** | System shall prevent Stopped too soon / Applied too long during OA_01_Ingest_OEM_Artifacts | DEAPCompilerSystem | UCA-004 |
| **SC-005** | System shall prevent Not providing during OA_01_01_Ingest_System_Vision | DEAPCompilerSystem | UCA-005 |
| **SC-006** | System shall prevent Providing during OA_01_01_Ingest_System_Vision | DEAPCompilerSystem | UCA-006 |
| **SC-007** | System shall prevent Too early / Too late / Out of order during OA_01_01_Ingest_System_Vision | DEAPCompilerSystem | UCA-007 |
| **SC-008** | System shall prevent Stopped too soon / Applied too long during OA_01_01_Ingest_System_Vision | DEAPCompilerSystem | UCA-008 |
| **SC-009** | System shall prevent Not providing during OA_01_02_Ingest_Universal_Schema | DEAPCompilerSystem | UCA-009 |
| **SC-010** | System shall prevent Providing during OA_01_02_Ingest_Universal_Schema | DEAPCompilerSystem | UCA-010 |
| **SC-011** | System shall prevent Too early / Too late / Out of order during OA_01_02_Ingest_Universal_Schema | DEAPCompilerSystem | UCA-011 |
| **SC-012** | System shall prevent Stopped too soon / Applied too long during OA_01_02_Ingest_Universal_Schema | DEAPCompilerSystem | UCA-012 |
| **SC-013** | System shall prevent Not providing during OA_01_1_Format_Detection | DEAPCompilerSystem | UCA-013 |
| **SC-014** | System shall prevent Providing during OA_01_1_Format_Detection | DEAPCompilerSystem | UCA-014 |
| **SC-015** | System shall prevent Too early / Too late / Out of order during OA_01_1_Format_Detection | DEAPCompilerSystem | UCA-015 |
| **SC-016** | System shall prevent Stopped too soon / Applied too long during OA_01_1_Format_Detection | DEAPCompilerSystem | UCA-016 |
| **SC-017** | System shall prevent Not providing during OA_01_2_Token_Lexing | DEAPCompilerSystem | UCA-017 |
| **SC-018** | System shall prevent Providing during OA_01_2_Token_Lexing | DEAPCompilerSystem | UCA-018 |
| **SC-019** | System shall prevent Too early / Too late / Out of order during OA_01_2_Token_Lexing | DEAPCompilerSystem | UCA-019 |
| **SC-020** | System shall prevent Stopped too soon / Applied too long during OA_01_2_Token_Lexing | DEAPCompilerSystem | UCA-020 |
| **SC-021** | System shall prevent Not providing during OA_01_3_Syntax_Normalization | DEAPCompilerSystem | UCA-021 |
| **SC-022** | System shall prevent Providing during OA_01_3_Syntax_Normalization | DEAPCompilerSystem | UCA-022 |
| **SC-023** | System shall prevent Too early / Too late / Out of order during OA_01_3_Syntax_Normalization | DEAPCompilerSystem | UCA-023 |
| **SC-024** | System shall prevent Stopped too soon / Applied too long during OA_01_3_Syntax_Normalization | DEAPCompilerSystem | UCA-024 |
| **SC-025** | System shall prevent Not providing during OA_02_Parse_SysML_AST | DEAPCompilerSystem | UCA-025 |
| **SC-026** | System shall prevent Providing during OA_02_Parse_SysML_AST | DEAPCompilerSystem | UCA-026 |
| **SC-027** | System shall prevent Too early / Too late / Out of order during OA_02_Parse_SysML_AST | DEAPCompilerSystem | UCA-027 |
| **SC-028** | System shall prevent Stopped too soon / Applied too long during OA_02_Parse_SysML_AST | DEAPCompilerSystem | UCA-028 |
| **SC-029** | System shall prevent Not providing during OA_02_01_Construct_Node_Arena | DEAPCompilerSystem | UCA-029 |
| **SC-030** | System shall prevent Providing during OA_02_01_Construct_Node_Arena | DEAPCompilerSystem | UCA-030 |
| **SC-031** | System shall prevent Too early / Too late / Out of order during OA_02_01_Construct_Node_Arena | DEAPCompilerSystem | UCA-031 |
| **SC-032** | System shall prevent Stopped too soon / Applied too long during OA_02_01_Construct_Node_Arena | DEAPCompilerSystem | UCA-032 |
| **SC-033** | System shall prevent Not providing during OA_02_02_Lower_Grammar_Constructs | DEAPCompilerSystem | UCA-033 |
| **SC-034** | System shall prevent Providing during OA_02_02_Lower_Grammar_Constructs | DEAPCompilerSystem | UCA-034 |
| **SC-035** | System shall prevent Too early / Too late / Out of order during OA_02_02_Lower_Grammar_Constructs | DEAPCompilerSystem | UCA-035 |
| **SC-036** | System shall prevent Stopped too soon / Applied too long during OA_02_02_Lower_Grammar_Constructs | DEAPCompilerSystem | UCA-036 |
| **SC-037** | System shall prevent Not providing during OA_02_1_KerML_AST_Build | DEAPCompilerSystem | UCA-037 |
| **SC-038** | System shall prevent Providing during OA_02_1_KerML_AST_Build | DEAPCompilerSystem | UCA-038 |
| **SC-039** | System shall prevent Too early / Too late / Out of order during OA_02_1_KerML_AST_Build | DEAPCompilerSystem | UCA-039 |
| **SC-040** | System shall prevent Stopped too soon / Applied too long during OA_02_1_KerML_AST_Build | DEAPCompilerSystem | UCA-040 |
| **SC-041** | System shall prevent Not providing during OA_02_2_Node_Allocation | DEAPCompilerSystem | UCA-041 |
| **SC-042** | System shall prevent Providing during OA_02_2_Node_Allocation | DEAPCompilerSystem | UCA-042 |
| **SC-043** | System shall prevent Too early / Too late / Out of order during OA_02_2_Node_Allocation | DEAPCompilerSystem | UCA-043 |
| **SC-044** | System shall prevent Stopped too soon / Applied too long during OA_02_2_Node_Allocation | DEAPCompilerSystem | UCA-044 |
| **SC-045** | System shall prevent Not providing during OA_02_3_Topological_Index | DEAPCompilerSystem | UCA-045 |
| **SC-046** | System shall prevent Providing during OA_02_3_Topological_Index | DEAPCompilerSystem | UCA-046 |
| **SC-047** | System shall prevent Too early / Too late / Out of order during OA_02_3_Topological_Index | DEAPCompilerSystem | UCA-047 |
| **SC-048** | System shall prevent Stopped too soon / Applied too long during OA_02_3_Topological_Index | DEAPCompilerSystem | UCA-048 |
| **SC-049** | System shall prevent Not providing during OA_03_Validate_Model_Semantics | DEAPCompilerSystem | UCA-049 |
| **SC-050** | System shall prevent Providing during OA_03_Validate_Model_Semantics | DEAPCompilerSystem | UCA-050 |
| **SC-051** | System shall prevent Too early / Too late / Out of order during OA_03_Validate_Model_Semantics | DEAPCompilerSystem | UCA-051 |
| **SC-052** | System shall prevent Stopped too soon / Applied too long during OA_03_Validate_Model_Semantics | DEAPCompilerSystem | UCA-052 |
| **SC-053** | System shall prevent Not providing during OA_03_01_Verify_Metrology_Flows | DEAPCompilerSystem | UCA-053 |
| **SC-054** | System shall prevent Providing during OA_03_01_Verify_Metrology_Flows | DEAPCompilerSystem | UCA-054 |
| **SC-055** | System shall prevent Too early / Too late / Out of order during OA_03_01_Verify_Metrology_Flows | DEAPCompilerSystem | UCA-055 |
| **SC-056** | System shall prevent Stopped too soon / Applied too long during OA_03_01_Verify_Metrology_Flows | DEAPCompilerSystem | UCA-056 |
| **SC-057** | System shall prevent Not providing during OA_03_02_Solve_State_Transitions | DEAPCompilerSystem | UCA-057 |
| **SC-058** | System shall prevent Providing during OA_03_02_Solve_State_Transitions | DEAPCompilerSystem | UCA-058 |
| **SC-059** | System shall prevent Too early / Too late / Out of order during OA_03_02_Solve_State_Transitions | DEAPCompilerSystem | UCA-059 |
| **SC-060** | System shall prevent Stopped too soon / Applied too long during OA_03_02_Solve_State_Transitions | DEAPCompilerSystem | UCA-060 |
| **SC-061** | System shall prevent Not providing during OA_03_1_Type_Checking | DEAPCompilerSystem | UCA-061 |
| **SC-062** | System shall prevent Providing during OA_03_1_Type_Checking | DEAPCompilerSystem | UCA-062 |
| **SC-063** | System shall prevent Too early / Too late / Out of order during OA_03_1_Type_Checking | DEAPCompilerSystem | UCA-063 |
| **SC-064** | System shall prevent Stopped too soon / Applied too long during OA_03_1_Type_Checking | DEAPCompilerSystem | UCA-064 |
| **SC-065** | System shall prevent Not providing during OA_03_2_Metrology_Verification | DEAPCompilerSystem | UCA-065 |
| **SC-066** | System shall prevent Providing during OA_03_2_Metrology_Verification | DEAPCompilerSystem | UCA-066 |
| **SC-067** | System shall prevent Too early / Too late / Out of order during OA_03_2_Metrology_Verification | DEAPCompilerSystem | UCA-067 |
| **SC-068** | System shall prevent Stopped too soon / Applied too long during OA_03_2_Metrology_Verification | DEAPCompilerSystem | UCA-068 |
| **SC-069** | System shall prevent Not providing during OA_03_3_State_Reachability | DEAPCompilerSystem | UCA-069 |
| **SC-070** | System shall prevent Providing during OA_03_3_State_Reachability | DEAPCompilerSystem | UCA-070 |
| **SC-071** | System shall prevent Too early / Too late / Out of order during OA_03_3_State_Reachability | DEAPCompilerSystem | UCA-071 |
| **SC-072** | System shall prevent Stopped too soon / Applied too long during OA_03_3_State_Reachability | DEAPCompilerSystem | UCA-072 |
| **SC-073** | System shall prevent Not providing during OA_04_Transpile_Safety_Artifacts | DEAPCompilerSystem | UCA-073 |
| **SC-074** | System shall prevent Providing during OA_04_Transpile_Safety_Artifacts | DEAPCompilerSystem | UCA-074 |
| **SC-075** | System shall prevent Too early / Too late / Out of order during OA_04_Transpile_Safety_Artifacts | DEAPCompilerSystem | UCA-075 |
| **SC-076** | System shall prevent Stopped too soon / Applied too long during OA_04_Transpile_Safety_Artifacts | DEAPCompilerSystem | UCA-076 |
| **SC-077** | System shall prevent Not providing during OA_04_01_Verify_Safety_Traceability | DEAPCompilerSystem | UCA-077 |
| **SC-078** | System shall prevent Providing during OA_04_01_Verify_Safety_Traceability | DEAPCompilerSystem | UCA-078 |
| **SC-079** | System shall prevent Too early / Too late / Out of order during OA_04_01_Verify_Safety_Traceability | DEAPCompilerSystem | UCA-079 |
| **SC-080** | System shall prevent Stopped too soon / Applied too long during OA_04_01_Verify_Safety_Traceability | DEAPCompilerSystem | UCA-080 |
| **SC-081** | System shall prevent Not providing during OA_04_02_Verify_ICD_Interconnect_Contracts | DEAPCompilerSystem | UCA-081 |
| **SC-082** | System shall prevent Providing during OA_04_02_Verify_ICD_Interconnect_Contracts | DEAPCompilerSystem | UCA-082 |
| **SC-083** | System shall prevent Too early / Too late / Out of order during OA_04_02_Verify_ICD_Interconnect_Contracts | DEAPCompilerSystem | UCA-083 |
| **SC-084** | System shall prevent Stopped too soon / Applied too long during OA_04_02_Verify_ICD_Interconnect_Contracts | DEAPCompilerSystem | UCA-084 |
| **SC-085** | System shall prevent Not providing during OA_04_1_STPA_Matrix_Synthesis | DEAPCompilerSystem | UCA-085 |
| **SC-086** | System shall prevent Providing during OA_04_1_STPA_Matrix_Synthesis | DEAPCompilerSystem | UCA-086 |
| **SC-087** | System shall prevent Too early / Too late / Out of order during OA_04_1_STPA_Matrix_Synthesis | DEAPCompilerSystem | UCA-087 |
| **SC-088** | System shall prevent Stopped too soon / Applied too long during OA_04_1_STPA_Matrix_Synthesis | DEAPCompilerSystem | UCA-088 |
| **SC-089** | System shall prevent Not providing during OA_04_2_UCA_Derivation | DEAPCompilerSystem | UCA-089 |
| **SC-090** | System shall prevent Providing during OA_04_2_UCA_Derivation | DEAPCompilerSystem | UCA-090 |
| **SC-091** | System shall prevent Too early / Too late / Out of order during OA_04_2_UCA_Derivation | DEAPCompilerSystem | UCA-091 |
| **SC-092** | System shall prevent Stopped too soon / Applied too long during OA_04_2_UCA_Derivation | DEAPCompilerSystem | UCA-092 |
| **SC-093** | System shall prevent Not providing during OA_04_3_FMECA_Generation | DEAPCompilerSystem | UCA-093 |
| **SC-094** | System shall prevent Providing during OA_04_3_FMECA_Generation | DEAPCompilerSystem | UCA-094 |
| **SC-095** | System shall prevent Too early / Too late / Out of order during OA_04_3_FMECA_Generation | DEAPCompilerSystem | UCA-095 |
| **SC-096** | System shall prevent Stopped too soon / Applied too long during OA_04_3_FMECA_Generation | DEAPCompilerSystem | UCA-096 |
| **SC-097** | System shall prevent Not providing during OA_05_Synthesize_Downstream_Projections | DEAPCompilerSystem | UCA-097 |
| **SC-098** | System shall prevent Providing during OA_05_Synthesize_Downstream_Projections | DEAPCompilerSystem | UCA-098 |
| **SC-099** | System shall prevent Too early / Too late / Out of order during OA_05_Synthesize_Downstream_Projections | DEAPCompilerSystem | UCA-099 |
| **SC-100** | System shall prevent Stopped too soon / Applied too long during OA_05_Synthesize_Downstream_Projections | DEAPCompilerSystem | UCA-100 |
| **SC-101** | System shall prevent Not providing during OA_05_01_Project_Agile_Backlog | DEAPCompilerSystem | UCA-101 |
| **SC-102** | System shall prevent Providing during OA_05_01_Project_Agile_Backlog | DEAPCompilerSystem | UCA-102 |
| **SC-103** | System shall prevent Too early / Too late / Out of order during OA_05_01_Project_Agile_Backlog | DEAPCompilerSystem | UCA-103 |
| **SC-104** | System shall prevent Stopped too soon / Applied too long during OA_05_01_Project_Agile_Backlog | DEAPCompilerSystem | UCA-104 |
| **SC-105** | System shall prevent Not providing during OA_05_02_Synthesize_Multi_Target_CodeGen | DEAPCompilerSystem | UCA-105 |
| **SC-106** | System shall prevent Providing during OA_05_02_Synthesize_Multi_Target_CodeGen | DEAPCompilerSystem | UCA-106 |
| **SC-107** | System shall prevent Too early / Too late / Out of order during OA_05_02_Synthesize_Multi_Target_CodeGen | DEAPCompilerSystem | UCA-107 |
| **SC-108** | System shall prevent Stopped too soon / Applied too long during OA_05_02_Synthesize_Multi_Target_CodeGen | DEAPCompilerSystem | UCA-108 |
| **SC-109** | System shall prevent Not providing during OA_05_1_Agile_Backlog_Projection | DEAPCompilerSystem | UCA-109 |
| **SC-110** | System shall prevent Providing during OA_05_1_Agile_Backlog_Projection | DEAPCompilerSystem | UCA-110 |
| **SC-111** | System shall prevent Too early / Too late / Out of order during OA_05_1_Agile_Backlog_Projection | DEAPCompilerSystem | UCA-111 |
| **SC-112** | System shall prevent Stopped too soon / Applied too long during OA_05_1_Agile_Backlog_Projection | DEAPCompilerSystem | UCA-112 |
| **SC-113** | System shall prevent Not providing during OA_05_2_Multi_Target_CodeGen | DEAPCompilerSystem | UCA-113 |
| **SC-114** | System shall prevent Providing during OA_05_2_Multi_Target_CodeGen | DEAPCompilerSystem | UCA-114 |
| **SC-115** | System shall prevent Too early / Too late / Out of order during OA_05_2_Multi_Target_CodeGen | DEAPCompilerSystem | UCA-115 |
| **SC-116** | System shall prevent Stopped too soon / Applied too long during OA_05_2_Multi_Target_CodeGen | DEAPCompilerSystem | UCA-116 |
| **SC-117** | System shall prevent Not providing during OA_05_3_Interface_Binding | DEAPCompilerSystem | UCA-117 |
| **SC-118** | System shall prevent Providing during OA_05_3_Interface_Binding | DEAPCompilerSystem | UCA-118 |
| **SC-119** | System shall prevent Too early / Too late / Out of order during OA_05_3_Interface_Binding | DEAPCompilerSystem | UCA-119 |
| **SC-120** | System shall prevent Stopped too soon / Applied too long during OA_05_3_Interface_Binding | DEAPCompilerSystem | UCA-120 |
| **SC-121** | System shall prevent Not providing during OA_06_Verify_Baseline_Parity | DEAPCompilerSystem | UCA-121 |
| **SC-122** | System shall prevent Providing during OA_06_Verify_Baseline_Parity | DEAPCompilerSystem | UCA-122 |
| **SC-123** | System shall prevent Too early / Too late / Out of order during OA_06_Verify_Baseline_Parity | DEAPCompilerSystem | UCA-123 |
| **SC-124** | System shall prevent Stopped too soon / Applied too long during OA_06_Verify_Baseline_Parity | DEAPCompilerSystem | UCA-124 |
| **SC-125** | System shall prevent Not providing during OA_06_01_Triage_Diagnostic_Catalog | DEAPCompilerSystem | UCA-125 |
| **SC-126** | System shall prevent Providing during OA_06_01_Triage_Diagnostic_Catalog | DEAPCompilerSystem | UCA-126 |
| **SC-127** | System shall prevent Too early / Too late / Out of order during OA_06_01_Triage_Diagnostic_Catalog | DEAPCompilerSystem | UCA-127 |
| **SC-128** | System shall prevent Stopped too soon / Applied too long during OA_06_01_Triage_Diagnostic_Catalog | DEAPCompilerSystem | UCA-128 |
| **SC-129** | System shall prevent Not providing during OA_06_02_Assure_Compiler_Performance_Gates | DEAPCompilerSystem | UCA-129 |
| **SC-130** | System shall prevent Providing during OA_06_02_Assure_Compiler_Performance_Gates | DEAPCompilerSystem | UCA-130 |
| **SC-131** | System shall prevent Too early / Too late / Out of order during OA_06_02_Assure_Compiler_Performance_Gates | DEAPCompilerSystem | UCA-131 |
| **SC-132** | System shall prevent Stopped too soon / Applied too long during OA_06_02_Assure_Compiler_Performance_Gates | DEAPCompilerSystem | UCA-132 |
| **SC-133** | System shall prevent Not providing during OA_06_1_Diagnostic_Catalog_Scan | DEAPCompilerSystem | UCA-133 |
| **SC-134** | System shall prevent Providing during OA_06_1_Diagnostic_Catalog_Scan | DEAPCompilerSystem | UCA-134 |
| **SC-135** | System shall prevent Too early / Too late / Out of order during OA_06_1_Diagnostic_Catalog_Scan | DEAPCompilerSystem | UCA-135 |
| **SC-136** | System shall prevent Stopped too soon / Applied too long during OA_06_1_Diagnostic_Catalog_Scan | DEAPCompilerSystem | UCA-136 |
| **SC-137** | System shall prevent Not providing during OA_06_2_Baseline_Parity_Audit | DEAPCompilerSystem | UCA-137 |
| **SC-138** | System shall prevent Providing during OA_06_2_Baseline_Parity_Audit | DEAPCompilerSystem | UCA-138 |
| **SC-139** | System shall prevent Too early / Too late / Out of order during OA_06_2_Baseline_Parity_Audit | DEAPCompilerSystem | UCA-139 |
| **SC-140** | System shall prevent Stopped too soon / Applied too long during OA_06_2_Baseline_Parity_Audit | DEAPCompilerSystem | UCA-140 |
| **SC-141** | System shall prevent Not providing during OA_06_3_Performance_Benchmarking | DEAPCompilerSystem | UCA-141 |
| **SC-142** | System shall prevent Providing during OA_06_3_Performance_Benchmarking | DEAPCompilerSystem | UCA-142 |
| **SC-143** | System shall prevent Too early / Too late / Out of order during OA_06_3_Performance_Benchmarking | DEAPCompilerSystem | UCA-143 |
| **SC-144** | System shall prevent Stopped too soon / Applied too long during OA_06_3_Performance_Benchmarking | DEAPCompilerSystem | UCA-144 |
| **SC-145** | System shall prevent Not providing during VerifyCompilerAssuranceAction | DEAPCompilerSystem | UCA-145 |
| **SC-146** | System shall prevent Providing during VerifyCompilerAssuranceAction | DEAPCompilerSystem | UCA-146 |
| **SC-147** | System shall prevent Too early / Too late / Out of order during VerifyCompilerAssuranceAction | DEAPCompilerSystem | UCA-147 |
| **SC-148** | System shall prevent Stopped too soon / Applied too long during VerifyCompilerAssuranceAction | DEAPCompilerSystem | UCA-148 |
| **SC-149** | System shall prevent Not providing during AllocateNodeAction | DEAPCompilerSystem | UCA-149 |
| **SC-150** | System shall prevent Providing during AllocateNodeAction | DEAPCompilerSystem | UCA-150 |
| **SC-151** | System shall prevent Too early / Too late / Out of order during AllocateNodeAction | DEAPCompilerSystem | UCA-151 |
| **SC-152** | System shall prevent Stopped too soon / Applied too long during AllocateNodeAction | DEAPCompilerSystem | UCA-152 |
| **SC-153** | System shall prevent Not providing during TriageDiagnosticAction | DEAPCompilerSystem | UCA-153 |
| **SC-154** | System shall prevent Providing during TriageDiagnosticAction | DEAPCompilerSystem | UCA-154 |
| **SC-155** | System shall prevent Too early / Too late / Out of order during TriageDiagnosticAction | DEAPCompilerSystem | UCA-155 |
| **SC-156** | System shall prevent Stopped too soon / Applied too long during TriageDiagnosticAction | DEAPCompilerSystem | UCA-156 |
| **SC-157** | System shall prevent Not providing during ProjectAgileSpecificationsAction | DEAPCompilerSystem | UCA-157 |
| **SC-158** | System shall prevent Providing during ProjectAgileSpecificationsAction | DEAPCompilerSystem | UCA-158 |
| **SC-159** | System shall prevent Too early / Too late / Out of order during ProjectAgileSpecificationsAction | DEAPCompilerSystem | UCA-159 |
| **SC-160** | System shall prevent Stopped too soon / Applied too long during ProjectAgileSpecificationsAction | DEAPCompilerSystem | UCA-160 |
| **SC-161** | System shall prevent Not providing during VerifySafetyTraceabilityAction | DEAPCompilerSystem | UCA-161 |
| **SC-162** | System shall prevent Providing during VerifySafetyTraceabilityAction | DEAPCompilerSystem | UCA-162 |
| **SC-163** | System shall prevent Too early / Too late / Out of order during VerifySafetyTraceabilityAction | DEAPCompilerSystem | UCA-163 |
| **SC-164** | System shall prevent Stopped too soon / Applied too long during VerifySafetyTraceabilityAction | DEAPCompilerSystem | UCA-164 |
| **SC-165** | System shall prevent Not providing during SynthesizeICDContractsAction | DEAPCompilerSystem | UCA-165 |
| **SC-166** | System shall prevent Providing during SynthesizeICDContractsAction | DEAPCompilerSystem | UCA-166 |
| **SC-167** | System shall prevent Too early / Too late / Out of order during SynthesizeICDContractsAction | DEAPCompilerSystem | UCA-167 |
| **SC-168** | System shall prevent Stopped too soon / Applied too long during SynthesizeICDContractsAction | DEAPCompilerSystem | UCA-168 |
| **SC-169** | System shall prevent Not providing during SynthesizeCodeAction | DEAPCompilerSystem | UCA-169 |
| **SC-170** | System shall prevent Providing during SynthesizeCodeAction | DEAPCompilerSystem | UCA-170 |
| **SC-171** | System shall prevent Too early / Too late / Out of order during SynthesizeCodeAction | DEAPCompilerSystem | UCA-171 |
| **SC-172** | System shall prevent Stopped too soon / Applied too long during SynthesizeCodeAction | DEAPCompilerSystem | UCA-172 |
| **SC-173** | System shall prevent Not providing during SolveFlowsAction | DEAPCompilerSystem | UCA-173 |
| **SC-174** | System shall prevent Providing during SolveFlowsAction | DEAPCompilerSystem | UCA-174 |
| **SC-175** | System shall prevent Too early / Too late / Out of order during SolveFlowsAction | DEAPCompilerSystem | UCA-175 |
| **SC-176** | System shall prevent Stopped too soon / Applied too long during SolveFlowsAction | DEAPCompilerSystem | UCA-176 |
| **SC-177** | System shall prevent Not providing during SolveStateDynamicsAction | DEAPCompilerSystem | UCA-177 |
| **SC-178** | System shall prevent Providing during SolveStateDynamicsAction | DEAPCompilerSystem | UCA-178 |
| **SC-179** | System shall prevent Too early / Too late / Out of order during SolveStateDynamicsAction | DEAPCompilerSystem | UCA-179 |
| **SC-180** | System shall prevent Stopped too soon / Applied too long during SolveStateDynamicsAction | DEAPCompilerSystem | UCA-180 |
| **SC-181** | System shall prevent Not providing during LowerSysMLAction | DEAPCompilerSystem | UCA-181 |
| **SC-182** | System shall prevent Providing during LowerSysMLAction | DEAPCompilerSystem | UCA-182 |
| **SC-183** | System shall prevent Too early / Too late / Out of order during LowerSysMLAction | DEAPCompilerSystem | UCA-183 |
| **SC-184** | System shall prevent Stopped too soon / Applied too long during LowerSysMLAction | DEAPCompilerSystem | UCA-184 |
| **SC-185** | System shall prevent Not providing during BootstrapSystemVisionAction | DEAPCompilerSystem | UCA-185 |
| **SC-186** | System shall prevent Providing during BootstrapSystemVisionAction | DEAPCompilerSystem | UCA-186 |
| **SC-187** | System shall prevent Too early / Too late / Out of order during BootstrapSystemVisionAction | DEAPCompilerSystem | UCA-187 |
| **SC-188** | System shall prevent Stopped too soon / Applied too long during BootstrapSystemVisionAction | DEAPCompilerSystem | UCA-188 |
| **SC-189** | System shall prevent Not providing during ClassifyFormatAction | DEAPCompilerSystem | UCA-189 |
| **SC-190** | System shall prevent Providing during ClassifyFormatAction | DEAPCompilerSystem | UCA-190 |
| **SC-191** | System shall prevent Too early / Too late / Out of order during ClassifyFormatAction | DEAPCompilerSystem | UCA-191 |
| **SC-192** | System shall prevent Stopped too soon / Applied too long during ClassifyFormatAction | DEAPCompilerSystem | UCA-192 |
| **SC-193** | System shall prevent Not providing during TokenizeStreamAction | DEAPCompilerSystem | UCA-193 |
| **SC-194** | System shall prevent Providing during TokenizeStreamAction | DEAPCompilerSystem | UCA-194 |
| **SC-195** | System shall prevent Too early / Too late / Out of order during TokenizeStreamAction | DEAPCompilerSystem | UCA-195 |
| **SC-196** | System shall prevent Stopped too soon / Applied too long during TokenizeStreamAction | DEAPCompilerSystem | UCA-196 |
| **SC-197** | System shall prevent Not providing during RecoverSyncAction | DEAPCompilerSystem | UCA-197 |
| **SC-198** | System shall prevent Providing during RecoverSyncAction | DEAPCompilerSystem | UCA-198 |
| **SC-199** | System shall prevent Too early / Too late / Out of order during RecoverSyncAction | DEAPCompilerSystem | UCA-199 |
| **SC-200** | System shall prevent Stopped too soon / Applied too long during RecoverSyncAction | DEAPCompilerSystem | UCA-200 |
| **SC-201** | System shall prevent Not providing during IngestSchemaAction | DEAPCompilerSystem | UCA-201 |
| **SC-202** | System shall prevent Providing during IngestSchemaAction | DEAPCompilerSystem | UCA-202 |
| **SC-203** | System shall prevent Too early / Too late / Out of order during IngestSchemaAction | DEAPCompilerSystem | UCA-203 |
| **SC-204** | System shall prevent Stopped too soon / Applied too long during IngestSchemaAction | DEAPCompilerSystem | UCA-204 |

## 2. STPA Unsafe Control Actions (UCA) Matrix
| UCA ID | Controller | Control Action | Guide Word | Hazard | Safety Constraint |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| UCA-001 | DEAPCompilerSystem | OA_01_Ingest_OEM_Artifacts | Not providing | H-1 | SC-001 |
| UCA-002 | DEAPCompilerSystem | OA_01_Ingest_OEM_Artifacts | Providing | H-1 | SC-002 |
| UCA-003 | DEAPCompilerSystem | OA_01_Ingest_OEM_Artifacts | Too early / Too late / Out of order | H-1 | SC-003 |
| UCA-004 | DEAPCompilerSystem | OA_01_Ingest_OEM_Artifacts | Stopped too soon / Applied too long | H-1 | SC-004 |
| UCA-005 | DEAPCompilerSystem | OA_01_01_Ingest_System_Vision | Not providing | H-2 | SC-005 |
| UCA-006 | DEAPCompilerSystem | OA_01_01_Ingest_System_Vision | Providing | H-2 | SC-006 |
| UCA-007 | DEAPCompilerSystem | OA_01_01_Ingest_System_Vision | Too early / Too late / Out of order | H-2 | SC-007 |
| UCA-008 | DEAPCompilerSystem | OA_01_01_Ingest_System_Vision | Stopped too soon / Applied too long | H-2 | SC-008 |
| UCA-009 | DEAPCompilerSystem | OA_01_02_Ingest_Universal_Schema | Not providing | H-3 | SC-009 |
| UCA-010 | DEAPCompilerSystem | OA_01_02_Ingest_Universal_Schema | Providing | H-3 | SC-010 |
| UCA-011 | DEAPCompilerSystem | OA_01_02_Ingest_Universal_Schema | Too early / Too late / Out of order | H-3 | SC-011 |
| UCA-012 | DEAPCompilerSystem | OA_01_02_Ingest_Universal_Schema | Stopped too soon / Applied too long | H-3 | SC-012 |
| UCA-013 | DEAPCompilerSystem | OA_01_1_Format_Detection | Not providing | H-4 | SC-013 |
| UCA-014 | DEAPCompilerSystem | OA_01_1_Format_Detection | Providing | H-4 | SC-014 |
| UCA-015 | DEAPCompilerSystem | OA_01_1_Format_Detection | Too early / Too late / Out of order | H-4 | SC-015 |
| UCA-016 | DEAPCompilerSystem | OA_01_1_Format_Detection | Stopped too soon / Applied too long | H-4 | SC-016 |
| UCA-017 | DEAPCompilerSystem | OA_01_2_Token_Lexing | Not providing | H-5 | SC-017 |
| UCA-018 | DEAPCompilerSystem | OA_01_2_Token_Lexing | Providing | H-5 | SC-018 |
| UCA-019 | DEAPCompilerSystem | OA_01_2_Token_Lexing | Too early / Too late / Out of order | H-5 | SC-019 |
| UCA-020 | DEAPCompilerSystem | OA_01_2_Token_Lexing | Stopped too soon / Applied too long | H-5 | SC-020 |
| UCA-021 | DEAPCompilerSystem | OA_01_3_Syntax_Normalization | Not providing | H-6 | SC-021 |
| UCA-022 | DEAPCompilerSystem | OA_01_3_Syntax_Normalization | Providing | H-6 | SC-022 |
| UCA-023 | DEAPCompilerSystem | OA_01_3_Syntax_Normalization | Too early / Too late / Out of order | H-6 | SC-023 |
| UCA-024 | DEAPCompilerSystem | OA_01_3_Syntax_Normalization | Stopped too soon / Applied too long | H-6 | SC-024 |
| UCA-025 | DEAPCompilerSystem | OA_02_Parse_SysML_AST | Not providing | H-7 | SC-025 |
| UCA-026 | DEAPCompilerSystem | OA_02_Parse_SysML_AST | Providing | H-7 | SC-026 |
| UCA-027 | DEAPCompilerSystem | OA_02_Parse_SysML_AST | Too early / Too late / Out of order | H-7 | SC-027 |
| UCA-028 | DEAPCompilerSystem | OA_02_Parse_SysML_AST | Stopped too soon / Applied too long | H-7 | SC-028 |
| UCA-029 | DEAPCompilerSystem | OA_02_01_Construct_Node_Arena | Not providing | H-8 | SC-029 |
| UCA-030 | DEAPCompilerSystem | OA_02_01_Construct_Node_Arena | Providing | H-8 | SC-030 |
| UCA-031 | DEAPCompilerSystem | OA_02_01_Construct_Node_Arena | Too early / Too late / Out of order | H-8 | SC-031 |
| UCA-032 | DEAPCompilerSystem | OA_02_01_Construct_Node_Arena | Stopped too soon / Applied too long | H-8 | SC-032 |
| UCA-033 | DEAPCompilerSystem | OA_02_02_Lower_Grammar_Constructs | Not providing | H-9 | SC-033 |
| UCA-034 | DEAPCompilerSystem | OA_02_02_Lower_Grammar_Constructs | Providing | H-9 | SC-034 |
| UCA-035 | DEAPCompilerSystem | OA_02_02_Lower_Grammar_Constructs | Too early / Too late / Out of order | H-9 | SC-035 |
| UCA-036 | DEAPCompilerSystem | OA_02_02_Lower_Grammar_Constructs | Stopped too soon / Applied too long | H-9 | SC-036 |
| UCA-037 | DEAPCompilerSystem | OA_02_1_KerML_AST_Build | Not providing | H-10 | SC-037 |
| UCA-038 | DEAPCompilerSystem | OA_02_1_KerML_AST_Build | Providing | H-10 | SC-038 |
| UCA-039 | DEAPCompilerSystem | OA_02_1_KerML_AST_Build | Too early / Too late / Out of order | H-10 | SC-039 |
| UCA-040 | DEAPCompilerSystem | OA_02_1_KerML_AST_Build | Stopped too soon / Applied too long | H-10 | SC-040 |
| UCA-041 | DEAPCompilerSystem | OA_02_2_Node_Allocation | Not providing | H-11 | SC-041 |
| UCA-042 | DEAPCompilerSystem | OA_02_2_Node_Allocation | Providing | H-11 | SC-042 |
| UCA-043 | DEAPCompilerSystem | OA_02_2_Node_Allocation | Too early / Too late / Out of order | H-11 | SC-043 |
| UCA-044 | DEAPCompilerSystem | OA_02_2_Node_Allocation | Stopped too soon / Applied too long | H-11 | SC-044 |
| UCA-045 | DEAPCompilerSystem | OA_02_3_Topological_Index | Not providing | H-12 | SC-045 |
| UCA-046 | DEAPCompilerSystem | OA_02_3_Topological_Index | Providing | H-12 | SC-046 |
| UCA-047 | DEAPCompilerSystem | OA_02_3_Topological_Index | Too early / Too late / Out of order | H-12 | SC-047 |
| UCA-048 | DEAPCompilerSystem | OA_02_3_Topological_Index | Stopped too soon / Applied too long | H-12 | SC-048 |
| UCA-049 | DEAPCompilerSystem | OA_03_Validate_Model_Semantics | Not providing | H-13 | SC-049 |
| UCA-050 | DEAPCompilerSystem | OA_03_Validate_Model_Semantics | Providing | H-13 | SC-050 |
| UCA-051 | DEAPCompilerSystem | OA_03_Validate_Model_Semantics | Too early / Too late / Out of order | H-13 | SC-051 |
| UCA-052 | DEAPCompilerSystem | OA_03_Validate_Model_Semantics | Stopped too soon / Applied too long | H-13 | SC-052 |
| UCA-053 | DEAPCompilerSystem | OA_03_01_Verify_Metrology_Flows | Not providing | H-14 | SC-053 |
| UCA-054 | DEAPCompilerSystem | OA_03_01_Verify_Metrology_Flows | Providing | H-14 | SC-054 |
| UCA-055 | DEAPCompilerSystem | OA_03_01_Verify_Metrology_Flows | Too early / Too late / Out of order | H-14 | SC-055 |
| UCA-056 | DEAPCompilerSystem | OA_03_01_Verify_Metrology_Flows | Stopped too soon / Applied too long | H-14 | SC-056 |
| UCA-057 | DEAPCompilerSystem | OA_03_02_Solve_State_Transitions | Not providing | H-15 | SC-057 |
| UCA-058 | DEAPCompilerSystem | OA_03_02_Solve_State_Transitions | Providing | H-15 | SC-058 |
| UCA-059 | DEAPCompilerSystem | OA_03_02_Solve_State_Transitions | Too early / Too late / Out of order | H-15 | SC-059 |
| UCA-060 | DEAPCompilerSystem | OA_03_02_Solve_State_Transitions | Stopped too soon / Applied too long | H-15 | SC-060 |
| UCA-061 | DEAPCompilerSystem | OA_03_1_Type_Checking | Not providing | H-16 | SC-061 |
| UCA-062 | DEAPCompilerSystem | OA_03_1_Type_Checking | Providing | H-16 | SC-062 |
| UCA-063 | DEAPCompilerSystem | OA_03_1_Type_Checking | Too early / Too late / Out of order | H-16 | SC-063 |
| UCA-064 | DEAPCompilerSystem | OA_03_1_Type_Checking | Stopped too soon / Applied too long | H-16 | SC-064 |
| UCA-065 | DEAPCompilerSystem | OA_03_2_Metrology_Verification | Not providing | H-17 | SC-065 |
| UCA-066 | DEAPCompilerSystem | OA_03_2_Metrology_Verification | Providing | H-17 | SC-066 |
| UCA-067 | DEAPCompilerSystem | OA_03_2_Metrology_Verification | Too early / Too late / Out of order | H-17 | SC-067 |
| UCA-068 | DEAPCompilerSystem | OA_03_2_Metrology_Verification | Stopped too soon / Applied too long | H-17 | SC-068 |
| UCA-069 | DEAPCompilerSystem | OA_03_3_State_Reachability | Not providing | H-18 | SC-069 |
| UCA-070 | DEAPCompilerSystem | OA_03_3_State_Reachability | Providing | H-18 | SC-070 |
| UCA-071 | DEAPCompilerSystem | OA_03_3_State_Reachability | Too early / Too late / Out of order | H-18 | SC-071 |
| UCA-072 | DEAPCompilerSystem | OA_03_3_State_Reachability | Stopped too soon / Applied too long | H-18 | SC-072 |
| UCA-073 | DEAPCompilerSystem | OA_04_Transpile_Safety_Artifacts | Not providing | H-19 | SC-073 |
| UCA-074 | DEAPCompilerSystem | OA_04_Transpile_Safety_Artifacts | Providing | H-19 | SC-074 |
| UCA-075 | DEAPCompilerSystem | OA_04_Transpile_Safety_Artifacts | Too early / Too late / Out of order | H-19 | SC-075 |
| UCA-076 | DEAPCompilerSystem | OA_04_Transpile_Safety_Artifacts | Stopped too soon / Applied too long | H-19 | SC-076 |
| UCA-077 | DEAPCompilerSystem | OA_04_01_Verify_Safety_Traceability | Not providing | H-20 | SC-077 |
| UCA-078 | DEAPCompilerSystem | OA_04_01_Verify_Safety_Traceability | Providing | H-20 | SC-078 |
| UCA-079 | DEAPCompilerSystem | OA_04_01_Verify_Safety_Traceability | Too early / Too late / Out of order | H-20 | SC-079 |
| UCA-080 | DEAPCompilerSystem | OA_04_01_Verify_Safety_Traceability | Stopped too soon / Applied too long | H-20 | SC-080 |
| UCA-081 | DEAPCompilerSystem | OA_04_02_Verify_ICD_Interconnect_Contracts | Not providing | H-21 | SC-081 |
| UCA-082 | DEAPCompilerSystem | OA_04_02_Verify_ICD_Interconnect_Contracts | Providing | H-21 | SC-082 |
| UCA-083 | DEAPCompilerSystem | OA_04_02_Verify_ICD_Interconnect_Contracts | Too early / Too late / Out of order | H-21 | SC-083 |
| UCA-084 | DEAPCompilerSystem | OA_04_02_Verify_ICD_Interconnect_Contracts | Stopped too soon / Applied too long | H-21 | SC-084 |
| UCA-085 | DEAPCompilerSystem | OA_04_1_STPA_Matrix_Synthesis | Not providing | H-22 | SC-085 |
| UCA-086 | DEAPCompilerSystem | OA_04_1_STPA_Matrix_Synthesis | Providing | H-22 | SC-086 |
| UCA-087 | DEAPCompilerSystem | OA_04_1_STPA_Matrix_Synthesis | Too early / Too late / Out of order | H-22 | SC-087 |
| UCA-088 | DEAPCompilerSystem | OA_04_1_STPA_Matrix_Synthesis | Stopped too soon / Applied too long | H-22 | SC-088 |
| UCA-089 | DEAPCompilerSystem | OA_04_2_UCA_Derivation | Not providing | H-23 | SC-089 |
| UCA-090 | DEAPCompilerSystem | OA_04_2_UCA_Derivation | Providing | H-23 | SC-090 |
| UCA-091 | DEAPCompilerSystem | OA_04_2_UCA_Derivation | Too early / Too late / Out of order | H-23 | SC-091 |
| UCA-092 | DEAPCompilerSystem | OA_04_2_UCA_Derivation | Stopped too soon / Applied too long | H-23 | SC-092 |
| UCA-093 | DEAPCompilerSystem | OA_04_3_FMECA_Generation | Not providing | H-24 | SC-093 |
| UCA-094 | DEAPCompilerSystem | OA_04_3_FMECA_Generation | Providing | H-24 | SC-094 |
| UCA-095 | DEAPCompilerSystem | OA_04_3_FMECA_Generation | Too early / Too late / Out of order | H-24 | SC-095 |
| UCA-096 | DEAPCompilerSystem | OA_04_3_FMECA_Generation | Stopped too soon / Applied too long | H-24 | SC-096 |
| UCA-097 | DEAPCompilerSystem | OA_05_Synthesize_Downstream_Projections | Not providing | H-25 | SC-097 |
| UCA-098 | DEAPCompilerSystem | OA_05_Synthesize_Downstream_Projections | Providing | H-25 | SC-098 |
| UCA-099 | DEAPCompilerSystem | OA_05_Synthesize_Downstream_Projections | Too early / Too late / Out of order | H-25 | SC-099 |
| UCA-100 | DEAPCompilerSystem | OA_05_Synthesize_Downstream_Projections | Stopped too soon / Applied too long | H-25 | SC-100 |
| UCA-101 | DEAPCompilerSystem | OA_05_01_Project_Agile_Backlog | Not providing | H-26 | SC-101 |
| UCA-102 | DEAPCompilerSystem | OA_05_01_Project_Agile_Backlog | Providing | H-26 | SC-102 |
| UCA-103 | DEAPCompilerSystem | OA_05_01_Project_Agile_Backlog | Too early / Too late / Out of order | H-26 | SC-103 |
| UCA-104 | DEAPCompilerSystem | OA_05_01_Project_Agile_Backlog | Stopped too soon / Applied too long | H-26 | SC-104 |
| UCA-105 | DEAPCompilerSystem | OA_05_02_Synthesize_Multi_Target_CodeGen | Not providing | H-27 | SC-105 |
| UCA-106 | DEAPCompilerSystem | OA_05_02_Synthesize_Multi_Target_CodeGen | Providing | H-27 | SC-106 |
| UCA-107 | DEAPCompilerSystem | OA_05_02_Synthesize_Multi_Target_CodeGen | Too early / Too late / Out of order | H-27 | SC-107 |
| UCA-108 | DEAPCompilerSystem | OA_05_02_Synthesize_Multi_Target_CodeGen | Stopped too soon / Applied too long | H-27 | SC-108 |
| UCA-109 | DEAPCompilerSystem | OA_05_1_Agile_Backlog_Projection | Not providing | H-28 | SC-109 |
| UCA-110 | DEAPCompilerSystem | OA_05_1_Agile_Backlog_Projection | Providing | H-28 | SC-110 |
| UCA-111 | DEAPCompilerSystem | OA_05_1_Agile_Backlog_Projection | Too early / Too late / Out of order | H-28 | SC-111 |
| UCA-112 | DEAPCompilerSystem | OA_05_1_Agile_Backlog_Projection | Stopped too soon / Applied too long | H-28 | SC-112 |
| UCA-113 | DEAPCompilerSystem | OA_05_2_Multi_Target_CodeGen | Not providing | H-29 | SC-113 |
| UCA-114 | DEAPCompilerSystem | OA_05_2_Multi_Target_CodeGen | Providing | H-29 | SC-114 |
| UCA-115 | DEAPCompilerSystem | OA_05_2_Multi_Target_CodeGen | Too early / Too late / Out of order | H-29 | SC-115 |
| UCA-116 | DEAPCompilerSystem | OA_05_2_Multi_Target_CodeGen | Stopped too soon / Applied too long | H-29 | SC-116 |
| UCA-117 | DEAPCompilerSystem | OA_05_3_Interface_Binding | Not providing | H-30 | SC-117 |
| UCA-118 | DEAPCompilerSystem | OA_05_3_Interface_Binding | Providing | H-30 | SC-118 |
| UCA-119 | DEAPCompilerSystem | OA_05_3_Interface_Binding | Too early / Too late / Out of order | H-30 | SC-119 |
| UCA-120 | DEAPCompilerSystem | OA_05_3_Interface_Binding | Stopped too soon / Applied too long | H-30 | SC-120 |
| UCA-121 | DEAPCompilerSystem | OA_06_Verify_Baseline_Parity | Not providing | H-31 | SC-121 |
| UCA-122 | DEAPCompilerSystem | OA_06_Verify_Baseline_Parity | Providing | H-31 | SC-122 |
| UCA-123 | DEAPCompilerSystem | OA_06_Verify_Baseline_Parity | Too early / Too late / Out of order | H-31 | SC-123 |
| UCA-124 | DEAPCompilerSystem | OA_06_Verify_Baseline_Parity | Stopped too soon / Applied too long | H-31 | SC-124 |
| UCA-125 | DEAPCompilerSystem | OA_06_01_Triage_Diagnostic_Catalog | Not providing | H-32 | SC-125 |
| UCA-126 | DEAPCompilerSystem | OA_06_01_Triage_Diagnostic_Catalog | Providing | H-32 | SC-126 |
| UCA-127 | DEAPCompilerSystem | OA_06_01_Triage_Diagnostic_Catalog | Too early / Too late / Out of order | H-32 | SC-127 |
| UCA-128 | DEAPCompilerSystem | OA_06_01_Triage_Diagnostic_Catalog | Stopped too soon / Applied too long | H-32 | SC-128 |
| UCA-129 | DEAPCompilerSystem | OA_06_02_Assure_Compiler_Performance_Gates | Not providing | H-33 | SC-129 |
| UCA-130 | DEAPCompilerSystem | OA_06_02_Assure_Compiler_Performance_Gates | Providing | H-33 | SC-130 |
| UCA-131 | DEAPCompilerSystem | OA_06_02_Assure_Compiler_Performance_Gates | Too early / Too late / Out of order | H-33 | SC-131 |
| UCA-132 | DEAPCompilerSystem | OA_06_02_Assure_Compiler_Performance_Gates | Stopped too soon / Applied too long | H-33 | SC-132 |
| UCA-133 | DEAPCompilerSystem | OA_06_1_Diagnostic_Catalog_Scan | Not providing | H-34 | SC-133 |
| UCA-134 | DEAPCompilerSystem | OA_06_1_Diagnostic_Catalog_Scan | Providing | H-34 | SC-134 |
| UCA-135 | DEAPCompilerSystem | OA_06_1_Diagnostic_Catalog_Scan | Too early / Too late / Out of order | H-34 | SC-135 |
| UCA-136 | DEAPCompilerSystem | OA_06_1_Diagnostic_Catalog_Scan | Stopped too soon / Applied too long | H-34 | SC-136 |
| UCA-137 | DEAPCompilerSystem | OA_06_2_Baseline_Parity_Audit | Not providing | H-35 | SC-137 |
| UCA-138 | DEAPCompilerSystem | OA_06_2_Baseline_Parity_Audit | Providing | H-35 | SC-138 |
| UCA-139 | DEAPCompilerSystem | OA_06_2_Baseline_Parity_Audit | Too early / Too late / Out of order | H-35 | SC-139 |
| UCA-140 | DEAPCompilerSystem | OA_06_2_Baseline_Parity_Audit | Stopped too soon / Applied too long | H-35 | SC-140 |
| UCA-141 | DEAPCompilerSystem | OA_06_3_Performance_Benchmarking | Not providing | H-36 | SC-141 |
| UCA-142 | DEAPCompilerSystem | OA_06_3_Performance_Benchmarking | Providing | H-36 | SC-142 |
| UCA-143 | DEAPCompilerSystem | OA_06_3_Performance_Benchmarking | Too early / Too late / Out of order | H-36 | SC-143 |
| UCA-144 | DEAPCompilerSystem | OA_06_3_Performance_Benchmarking | Stopped too soon / Applied too long | H-36 | SC-144 |
| UCA-145 | DEAPCompilerSystem | VerifyCompilerAssuranceAction | Not providing | H-37 | SC-145 |
| UCA-146 | DEAPCompilerSystem | VerifyCompilerAssuranceAction | Providing | H-37 | SC-146 |
| UCA-147 | DEAPCompilerSystem | VerifyCompilerAssuranceAction | Too early / Too late / Out of order | H-37 | SC-147 |
| UCA-148 | DEAPCompilerSystem | VerifyCompilerAssuranceAction | Stopped too soon / Applied too long | H-37 | SC-148 |
| UCA-149 | DEAPCompilerSystem | AllocateNodeAction | Not providing | H-38 | SC-149 |
| UCA-150 | DEAPCompilerSystem | AllocateNodeAction | Providing | H-38 | SC-150 |
| UCA-151 | DEAPCompilerSystem | AllocateNodeAction | Too early / Too late / Out of order | H-38 | SC-151 |
| UCA-152 | DEAPCompilerSystem | AllocateNodeAction | Stopped too soon / Applied too long | H-38 | SC-152 |
| UCA-153 | DEAPCompilerSystem | TriageDiagnosticAction | Not providing | H-39 | SC-153 |
| UCA-154 | DEAPCompilerSystem | TriageDiagnosticAction | Providing | H-39 | SC-154 |
| UCA-155 | DEAPCompilerSystem | TriageDiagnosticAction | Too early / Too late / Out of order | H-39 | SC-155 |
| UCA-156 | DEAPCompilerSystem | TriageDiagnosticAction | Stopped too soon / Applied too long | H-39 | SC-156 |
| UCA-157 | DEAPCompilerSystem | ProjectAgileSpecificationsAction | Not providing | H-40 | SC-157 |
| UCA-158 | DEAPCompilerSystem | ProjectAgileSpecificationsAction | Providing | H-40 | SC-158 |
| UCA-159 | DEAPCompilerSystem | ProjectAgileSpecificationsAction | Too early / Too late / Out of order | H-40 | SC-159 |
| UCA-160 | DEAPCompilerSystem | ProjectAgileSpecificationsAction | Stopped too soon / Applied too long | H-40 | SC-160 |
| UCA-161 | DEAPCompilerSystem | VerifySafetyTraceabilityAction | Not providing | H-41 | SC-161 |
| UCA-162 | DEAPCompilerSystem | VerifySafetyTraceabilityAction | Providing | H-41 | SC-162 |
| UCA-163 | DEAPCompilerSystem | VerifySafetyTraceabilityAction | Too early / Too late / Out of order | H-41 | SC-163 |
| UCA-164 | DEAPCompilerSystem | VerifySafetyTraceabilityAction | Stopped too soon / Applied too long | H-41 | SC-164 |
| UCA-165 | DEAPCompilerSystem | SynthesizeICDContractsAction | Not providing | H-42 | SC-165 |
| UCA-166 | DEAPCompilerSystem | SynthesizeICDContractsAction | Providing | H-42 | SC-166 |
| UCA-167 | DEAPCompilerSystem | SynthesizeICDContractsAction | Too early / Too late / Out of order | H-42 | SC-167 |
| UCA-168 | DEAPCompilerSystem | SynthesizeICDContractsAction | Stopped too soon / Applied too long | H-42 | SC-168 |
| UCA-169 | DEAPCompilerSystem | SynthesizeCodeAction | Not providing | H-43 | SC-169 |
| UCA-170 | DEAPCompilerSystem | SynthesizeCodeAction | Providing | H-43 | SC-170 |
| UCA-171 | DEAPCompilerSystem | SynthesizeCodeAction | Too early / Too late / Out of order | H-43 | SC-171 |
| UCA-172 | DEAPCompilerSystem | SynthesizeCodeAction | Stopped too soon / Applied too long | H-43 | SC-172 |
| UCA-173 | DEAPCompilerSystem | SolveFlowsAction | Not providing | H-44 | SC-173 |
| UCA-174 | DEAPCompilerSystem | SolveFlowsAction | Providing | H-44 | SC-174 |
| UCA-175 | DEAPCompilerSystem | SolveFlowsAction | Too early / Too late / Out of order | H-44 | SC-175 |
| UCA-176 | DEAPCompilerSystem | SolveFlowsAction | Stopped too soon / Applied too long | H-44 | SC-176 |
| UCA-177 | DEAPCompilerSystem | SolveStateDynamicsAction | Not providing | H-45 | SC-177 |
| UCA-178 | DEAPCompilerSystem | SolveStateDynamicsAction | Providing | H-45 | SC-178 |
| UCA-179 | DEAPCompilerSystem | SolveStateDynamicsAction | Too early / Too late / Out of order | H-45 | SC-179 |
| UCA-180 | DEAPCompilerSystem | SolveStateDynamicsAction | Stopped too soon / Applied too long | H-45 | SC-180 |
| UCA-181 | DEAPCompilerSystem | LowerSysMLAction | Not providing | H-46 | SC-181 |
| UCA-182 | DEAPCompilerSystem | LowerSysMLAction | Providing | H-46 | SC-182 |
| UCA-183 | DEAPCompilerSystem | LowerSysMLAction | Too early / Too late / Out of order | H-46 | SC-183 |
| UCA-184 | DEAPCompilerSystem | LowerSysMLAction | Stopped too soon / Applied too long | H-46 | SC-184 |
| UCA-185 | DEAPCompilerSystem | BootstrapSystemVisionAction | Not providing | H-47 | SC-185 |
| UCA-186 | DEAPCompilerSystem | BootstrapSystemVisionAction | Providing | H-47 | SC-186 |
| UCA-187 | DEAPCompilerSystem | BootstrapSystemVisionAction | Too early / Too late / Out of order | H-47 | SC-187 |
| UCA-188 | DEAPCompilerSystem | BootstrapSystemVisionAction | Stopped too soon / Applied too long | H-47 | SC-188 |
| UCA-189 | DEAPCompilerSystem | ClassifyFormatAction | Not providing | H-48 | SC-189 |
| UCA-190 | DEAPCompilerSystem | ClassifyFormatAction | Providing | H-48 | SC-190 |
| UCA-191 | DEAPCompilerSystem | ClassifyFormatAction | Too early / Too late / Out of order | H-48 | SC-191 |
| UCA-192 | DEAPCompilerSystem | ClassifyFormatAction | Stopped too soon / Applied too long | H-48 | SC-192 |
| UCA-193 | DEAPCompilerSystem | TokenizeStreamAction | Not providing | H-49 | SC-193 |
| UCA-194 | DEAPCompilerSystem | TokenizeStreamAction | Providing | H-49 | SC-194 |
| UCA-195 | DEAPCompilerSystem | TokenizeStreamAction | Too early / Too late / Out of order | H-49 | SC-195 |
| UCA-196 | DEAPCompilerSystem | TokenizeStreamAction | Stopped too soon / Applied too long | H-49 | SC-196 |
| UCA-197 | DEAPCompilerSystem | RecoverSyncAction | Not providing | H-50 | SC-197 |
| UCA-198 | DEAPCompilerSystem | RecoverSyncAction | Providing | H-50 | SC-198 |
| UCA-199 | DEAPCompilerSystem | RecoverSyncAction | Too early / Too late / Out of order | H-50 | SC-199 |
| UCA-200 | DEAPCompilerSystem | RecoverSyncAction | Stopped too soon / Applied too long | H-50 | SC-200 |
| UCA-201 | DEAPCompilerSystem | IngestSchemaAction | Not providing | H-51 | SC-201 |
| UCA-202 | DEAPCompilerSystem | IngestSchemaAction | Providing | H-51 | SC-202 |
| UCA-203 | DEAPCompilerSystem | IngestSchemaAction | Too early / Too late / Out of order | H-51 | SC-203 |
| UCA-204 | DEAPCompilerSystem | IngestSchemaAction | Stopped too soon / Applied too long | H-51 | SC-204 |

