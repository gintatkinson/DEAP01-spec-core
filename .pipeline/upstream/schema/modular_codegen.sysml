// Authoritative Self-Hosting 4-Level Modular Multi-File Code Generator Metamodel (BP-07 / REQ-0607)
// Pure abstract MBSE compiler specification -- zero hardcoded domain concepts.

package ModularCodegen {
    package domain {
        part def ModularityBoundsSpec {
            @Layer("Domain");
            attribute max_file_lines : Integer = 400;
            attribute max_items_per_file : Integer = 12;
            attribute max_cyclomatic_complexity : Integer = 8;
            attribute max_action_steps_per_stage : Integer = 5;
            attribute max_transitions_per_handler : Integer = 6;

            constraint def ValidBoundsEnvelope(
                in max_file_lines : Integer,
                in max_items_per_file : Integer,
                in max_cyclomatic_complexity : Integer,
                in max_action_steps_per_stage : Integer,
                in max_transitions_per_handler : Integer
            ) {
                max_file_lines >= 50
                    and max_file_lines <= 400
                    and max_items_per_file >= 1
                    and max_items_per_file <= 12
                    and max_cyclomatic_complexity >= 1
                    and max_cyclomatic_complexity <= 8
                    and max_action_steps_per_stage >= 1
                    and max_action_steps_per_stage <= 8
                    and max_transitions_per_handler >= 1
                    and max_transitions_per_handler <= 8;
            }
        }

        part def LayerHierarchySpec {
            @Layer("Domain");
            attribute domain_tier : Integer = 1;
            attribute application_tier : Integer = 2;
            attribute presentation_tier : Integer = 3;

            constraint def NonUpwardLayerImport(
                in importer_tier : Integer,
                in imported_tier : Integer
            ) {
                importer_tier >= 1
                    and importer_tier <= 3
                    and imported_tier >= 1
                    and imported_tier <= 3
                    and importer_tier >= imported_tier;
            }
        }

        part def CompilationUnitSpec {
            @Layer("Domain");
            attribute relative_path : String = "mod.rs";
            attribute layer_tier : Integer = 1;
            attribute item_count : Integer = 1;
            attribute line_count : Integer = 10;
            attribute max_function_vg : Integer = 1;

            constraint def UnitObeysBounds(
                in line_count : Integer,
                in max_file_lines : Integer,
                in item_count : Integer,
                in max_items_per_file : Integer,
                in max_function_vg : Integer,
                in max_cyclomatic_complexity : Integer
            ) {
                line_count >= 1
                    and line_count <= max_file_lines
                    and item_count >= 1
                    and item_count <= max_items_per_file
                    and max_function_vg >= 1
                    and max_function_vg <= max_cyclomatic_complexity;
            }
        }

        calc def ComputeCyclomaticComplexity(in decision_points : Integer) : Integer {
            decision_points + 1;
        }
    }

    package lowering {
        port def SchemaPackageAstPort {
            attribute protocol_family : String = "SysmlPackageAst";
            flow in package_ast_stream : String;
        }

        port def VerifiedModuleGraphPort {
            attribute protocol_family : String = "VerifiedModuleGraph";
            flow out module_graph_stream : String;
        }

        part def FourLevelLoweringEngineSpec {
            @Layer("Application");
            attribute active_level_count : Integer = 4;
            attribute fail_closed_on_cycle : Boolean = true;
            attribute fail_closed_on_upward_layer : Boolean = true;
            part bounds_ref : ModularityBoundsSpec;
            part layer_ref : LayerHierarchySpec;
            port in schema_in : SchemaPackageAstPort;
            port out graph_out : VerifiedModuleGraphPort;

            constraint def LoweringLevelsComplete(in active_level_count : Integer) {
                active_level_count == 4;
            }
        }

        action def ExecuteFourLevelLoweringPipeline {
            in package_count : Integer;
            out unit_count : Integer;
            step resolve_package_hierarchy;
            step extract_part_layer_annotations;
            step build_directed_dependency_edges;
            step run_kahn_topological_cycle_check;
            step verify_non_upward_layer_edges;
            step partition_package_directories;
            step split_part_responsibility_facets;
            step outline_sequential_action_stages;
            step outline_source_state_handlers;
            step hoist_anf_boolean_subpredicates;
            step synthesize_barrel_reexport_facades;
            step verify_in_memory_modularity_bounds;
        }

        state def ModularCodegenLifecycle {
            state Idle;
            state ResolvingLayers;
            state VerifyingDag;
            state PartitioningPackages;
            state SplittingFacets;
            state OutliningBehaviors;
            state HoistingPredicates;
            state SynthesizingTree;
            state VerifyingBounds;
            state PersistingAtomic;
            state Verified;
            state AbortedFailClosed;

            transition Initial_to_Idle first Initial then Idle;
            transition Idle_to_ResolvingLayers first Idle accept start_compilation then ResolvingLayers;
            transition ResolvingLayers_to_VerifyingDag first ResolvingLayers accept layers_resolved then VerifyingDag;
            transition VerifyingDag_to_PartitioningPackages first VerifyingDag accept dag_verified then PartitioningPackages;
            transition VerifyingDag_to_AbortedOnCycle first VerifyingDag accept on_cycle_e0702 then AbortedFailClosed;
            transition VerifyingDag_to_AbortedOnUpwardLayer first VerifyingDag accept on_upward_layer_e0703 then AbortedFailClosed;
            transition PartitioningPackages_to_SplittingFacets first PartitioningPackages accept packages_partitioned then SplittingFacets;
            transition SplittingFacets_to_OutliningBehaviors first SplittingFacets accept facets_split then OutliningBehaviors;
            transition OutliningBehaviors_to_HoistingPredicates first OutliningBehaviors accept behaviors_outlined then HoistingPredicates;
            transition HoistingPredicates_to_SynthesizingTree first HoistingPredicates accept predicates_hoisted then SynthesizingTree;
            transition SynthesizingTree_to_VerifyingBounds first SynthesizingTree accept tree_synthesized then VerifyingBounds;
            transition VerifyingBounds_to_PersistingAtomic first VerifyingBounds accept bounds_verified then PersistingAtomic;
            transition VerifyingBounds_to_AbortedOnDrift first VerifyingBounds accept on_fixed_point_drift_e0705 then AbortedFailClosed;
            transition PersistingAtomic_to_Verified first PersistingAtomic accept atomic_commit_succeeded then Verified;
            transition Verified_to_Idle first Verified accept reset_lifecycle then Idle;
            transition AbortedFailClosed_to_Idle first AbortedFailClosed accept reset_lifecycle then Idle;
        }
    }

    package emitters {
        port def ModularRustEmitPort {
            attribute protocol_family : String = "AtomicRustTree";
            flow out rust_manifest_sha256 : String;
        }

        port def ModularFlutterEmitPort {
            attribute protocol_family : String = "AtomicDartTree";
            flow out dart_manifest_sha256 : String;
        }

        part def ModularEmitterSuiteSpec {
            @Layer("Presentation");
            attribute atomic_persistence : Boolean = true;
            attribute check36_docstrings_enabled : Boolean = true;
            attribute stage_mismatch_files : Integer = 0;
            part engine_ref : FourLevelLoweringEngineSpec;
            port out rust_tree_out : ModularRustEmitPort;
            port out flutter_tree_out : ModularFlutterEmitPort;

            constraint def FixedPointBootstrapParity(in stage_mismatch_files : Integer) {
                stage_mismatch_files == 0;
            }
        }
    }
}
