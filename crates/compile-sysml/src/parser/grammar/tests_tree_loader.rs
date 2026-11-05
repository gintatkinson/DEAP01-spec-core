//! Multi-file SysML v2 tree discovery, iterative package merging, and `MAX_PARSE_DEPTH` stack-bound tests.
//!
//! Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-TESTS]
//! Contract: Verifies `discover_sysml_files` iterative directory traversal and `.`-root filtering, `parse_sysml_tree` / `parse_sysml_directory` package merging, `CalcDef` merge conservation, and `MAX_PARSE_DEPTH` (64) stack bounds under constrained thread stacks.
//! Bounds & Faults: Uses isolated PID/timestamp-scoped temporary directories cleaned up deterministically and fixed-stack scoped threads with zero panics.

#[cfg(test)]
mod tree_loader_and_depth_tests {
    use super::super::*;

    #[test]
    fn test_parse_multi_file_sysml_tree() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = std::env::temp_dir().join(format!(
            "test_sysml_tree_{}",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_nanos()
        ));
        let conops_dir = temp_dir.join("conops");
        let sub1_dir = temp_dir.join("subsystems/sub1");
        fs::create_dir_all(&conops_dir)?;
        fs::create_dir_all(&sub1_dir)?;

        fs::write(conops_dir.join("actors.sysml"), "package ConOps {\n    part def Operator;\n}\n")?;
        fs::write(
            sub1_dir.join("requirements.sysml"),
            "package Subsystem_1 {\n    requirement def SubsystemSafetyReq {\n        doc /* Subsystem shall fail safe */\n    }\n}\n",
        )?;
        fs::write(
            sub1_dir.join("architecture.sysml"),
            "package Subsystem_1 {\n    part def SubsystemController;\n}\n",
        )?;
        fs::write(
            temp_dir.join("model.sysml"),
            "package DEAP_Compiler_System {\n    import Subsystem_1::*;\n    part def MainSystem;\n}\n",
        )?;
        fs::write(temp_dir.join(".hidden.sysml"), "package Hidden {}")?;
        fs::write(temp_dir.join("#temp.sysml#"), "package Temp {}")?;

        let discovered = discover_sysml_files(&temp_dir);
        assert_eq!(discovered.len(), 4);
        assert!(!discovered.iter().any(|p| p.to_string_lossy().contains(".hidden")));
        assert!(!discovered.iter().any(|p| p.to_string_lossy().contains("#temp")));

        let root = parse_sysml_tree(&discovered)?;
        assert_eq!(root.packages.len(), 3);

        let conops = root.packages.iter().find(|p| p.name == "ConOps").ok_or("ConOps missing")?;
        assert_eq!(conops.part_defs.len(), 1);
        assert_eq!(conops.part_defs.first().ok_or("missing Operator")?.name, "Operator");

        let sub1 = root.packages.iter().find(|p| p.name == "Subsystem_1").ok_or("Subsystem_1 missing")?;
        assert_eq!(sub1.requirement_defs.len(), 1);
        assert_eq!(sub1.requirement_defs.first().ok_or("missing req")?.name, "SubsystemSafetyReq");
        assert_eq!(sub1.part_defs.len(), 1);
        assert_eq!(sub1.part_defs.first().ok_or("missing part")?.name, "SubsystemController");

        let deap = root.packages.iter().find(|p| p.name == "DEAP_Compiler_System").ok_or("DEAP missing")?;
        assert_eq!(deap.imports.len(), 1);
        let imp = deap.imports.first().ok_or("missing import")?;
        assert_eq!(imp.path, "Subsystem_1");
        assert!(imp.is_wildcard);
        assert_eq!(deap.part_defs.len(), 1);
        assert_eq!(deap.part_defs.first().ok_or("missing MainSystem")?.name, "MainSystem");

        let root_from_dir = parse_sysml_directory(&temp_dir)?;
        assert_eq!(root_from_dir.packages.len(), 3);
        let _ = fs::remove_dir_all(&temp_dir);
        Ok(())
    }

    #[test]
    fn test_issue_444_merge_package_defs_preserves_calc_defs(
    ) -> Result<(), Box<dyn std::error::Error>> {
        use deap_core::sysml_ast::CalcDef;

        let mut dest = PackageDef { name: "TrajectorySubsystem".to_string(), ..Default::default() };
        let mut src = PackageDef { name: "TrajectorySubsystem".to_string(), ..Default::default() };
        src.calc_defs.push(CalcDef {
            name: "ComputeTargetTrajectory".to_string(),
            return_type: Some("TrajectoryVector".to_string()),
            ..Default::default()
        });

        merge_package_defs(&mut dest, src);
        assert_eq!(dest.calc_defs.len(), 1);
        assert_eq!(dest.calc_defs.first().map(|c| c.name.as_str()), Some("ComputeTargetTrajectory"));
        Ok(())
    }

    /// Verifies that `discover_sysml_files` allows a `.`-prefixed root directory at depth 0 while excluding `target/` and hidden directories at depth > 0.
    #[test]
    fn test_discover_sysml_files_allows_dot_root_and_excludes_target(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir =
            std::env::temp_dir().join(format!(".deap_test_discover_sysml_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        let schema_dir = temp_dir.join("schema");
        let target_dir = temp_dir.join("target").join("debug");
        let hidden_dir = temp_dir.join(".pipeline");
        fs::create_dir_all(&schema_dir)?;
        fs::create_dir_all(&target_dir)?;
        fs::create_dir_all(&hidden_dir)?;

        let valid_sysml = schema_dir.join("model.sysml");
        fs::write(&valid_sysml, "package P {}\n")?;
        fs::write(target_dir.join("cached.sysml"), "package Cached {}\n")?;
        fs::write(hidden_dir.join("schema.sysml"), "package Hidden {}\n")?;

        let discovered = discover_sysml_files(&temp_dir);
        let _ = fs::remove_dir_all(&temp_dir);
        assert_eq!(discovered, vec![valid_sysml]);
        Ok(())
    }

    fn build_nested_states(count: usize) -> String {
        let mut src = String::new();
        for idx in 0..count {
            src.push_str(&format!("state S_{idx} {{ "));
        }
        for _ in 0..count {
            src.push_str("} ");
        }
        src
    }

    fn assert_deep_parser_stack_probe() -> Result<(), String> {
        let deep_levels = 2_000usize;
        let (mut deep_pkgs, mut deep_parts) = (String::new(), String::new());
        let mut deep_tuple_type = String::from("attribute tup : ");
        for idx in 0..deep_levels {
            deep_pkgs.push_str(&format!("package P_{idx} {{ "));
            deep_parts.push_str(&format!("part def Part_{idx} {{ "));
            deep_tuple_type.push('(');
        }
        deep_tuple_type.push_str("u32");
        for _ in 0..deep_levels {
            deep_pkgs.push_str("} ");
            deep_parts.push_str("} ");
            deep_tuple_type.push(')');
        }
        deep_tuple_type.push(';');
        let deep_states = build_nested_states(deep_levels);

        std::thread::scope(|s| -> Result<(), String> {
            let handle = std::thread::Builder::new()
                .name("bounded_grammar_parser_probe".to_string())
                .stack_size(2 * 1024 * 1024)
                .spawn_scoped(s, || -> Result<(), String> {
                    for (label, src) in [
                        ("packages", deep_pkgs.as_str()),
                        ("parts", deep_parts.as_str()),
                        ("states", deep_states.as_str()),
                        ("tuple_types", deep_tuple_type.as_str()),
                    ] {
                        let toks = SysmlParser::tokenize_source(src)
                            .map_err(|e| format!("tokenize {label} failed: {e}"))?;
                        let mut p = SysmlParser::new(&toks);
                        let err = p.parse_package_or_file("Bounded").err().ok_or_else(|| {
                            format!("2000-level {label} must return Err(ParseError)")
                        })?;
                        if !err.message.contains("recursion depth") || p.depth != 0 {
                            return Err(format!("bad depth/error for {label}: {err:?} depth={}", p.depth));
                        }
                    }
                    Ok(())
                })
                .map_err(|e| format!("failed to spawn parser thread: {e}"))?;
            handle.join().map_err(|_| "bounded parser stack thread panicked".to_string())?
        })
    }

    fn assert_deep_merge_stack_probe() -> Result<(), String> {
        std::thread::scope(|s| -> Result<(), String> {
            let handle = std::thread::Builder::new()
                .name("bounded_grammar_merge_probe".to_string())
                .stack_size(256 * 1024)
                .spawn_scoped(s, || -> Result<(), String> {
                    let chain_len = 2_000usize;
                    let build_pkg_chain = |attr_prefix: &str| -> PackageDef {
                        let mut curr = PackageDef {
                            name: format!("Pkg_{}", chain_len.saturating_sub(1)),
                            ..Default::default()
                        };
                        for idx in (0..chain_len.saturating_sub(1)).rev() {
                            let attr = AttributeDef {
                                name: format!("{attr_prefix}_{idx}"),
                                type_name: "Integer".to_string(),
                                default_value: Some("1".to_string()),
                                doc: None,
                            };
                            curr = PackageDef {
                                name: format!("Pkg_{idx}"),
                                attribute_defs: vec![attr],
                                packages: vec![curr],
                                ..Default::default()
                            };
                        }
                        curr
                    };
                    let mut dest_chain = build_pkg_chain("dest_attr");
                    let src_chain = build_pkg_chain("src_attr");
                    merge_package_defs(&mut dest_chain, src_chain);
                    let mut cursor = &dest_chain;
                    for depth in 0..=MAX_PARSE_DEPTH {
                        let expected_attrs = if depth < MAX_PARSE_DEPTH { 2 } else { 1 };
                        if cursor.attribute_defs.len() != expected_attrs {
                            return Err(format!("at depth {depth}, got {}", cursor.attribute_defs.len()));
                        }
                        cursor = cursor.packages.first().ok_or_else(|| format!("missing child at {depth}"))?;
                    }
                    tree_loader::drop_packages_iteratively(std::mem::take(&mut dest_chain.packages));
                    Ok(())
                })
                .map_err(|e| format!("failed to spawn merge thread: {e}"))?;
            handle.join().map_err(|_| "256 KiB merge stack thread panicked".to_string())?
        })
    }

    /// Verifies that `SysmlParser` and `merge_package_defs` enforce `MAX_PARSE_DEPTH` (64),
    /// return `Err(ParseError)` cleanly without call-stack overflow when exceeded, and restore
    /// `SysmlParser::depth == 0` across all `Ok` and `Err` paths.
    #[test]
    fn test_max_parse_depth_bounds_recursion_and_merge() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(MAX_PARSE_DEPTH, 64);

        let exact_states_src = build_nested_states(MAX_PARSE_DEPTH.saturating_sub(1));
        let exact_state_tokens = SysmlParser::tokenize_source(&exact_states_src)?;
        let mut exact_state_parser = SysmlParser::new(&exact_state_tokens);
        assert_eq!(exact_state_parser.depth, 0);
        assert!(exact_state_parser.parse_package_or_file("ExactStates").is_ok());
        assert_eq!(exact_state_parser.depth, 0);

        let over_states_src = build_nested_states(MAX_PARSE_DEPTH);
        let over_state_tokens = SysmlParser::tokenize_source(&over_states_src)?;
        let mut over_state_parser = SysmlParser::new(&over_state_tokens);
        let over_state_err = over_state_parser
            .parse_package_or_file("OverStates")
            .err()
            .ok_or("64 nested states must fail")?;
        assert!(over_state_err.message.contains("recursion depth"));
        assert_eq!(over_state_parser.depth, 0);

        let build_nested_vec_attr = |wrappers: usize| -> String {
            format!(
                "attribute nested : {}u32{};",
                "Vec<".repeat(wrappers),
                ">".repeat(wrappers)
            )
        };
        let exact_type_tokens = SysmlParser::tokenize_source(&build_nested_vec_attr(MAX_PARSE_DEPTH.saturating_sub(2)))?;
        let mut exact_type_parser = SysmlParser::new(&exact_type_tokens);
        assert!(exact_type_parser.parse_package_or_file("ExactType").is_ok());
        assert_eq!(exact_type_parser.depth, 0);

        let over_type_tokens = SysmlParser::tokenize_source(&build_nested_vec_attr(MAX_PARSE_DEPTH.saturating_sub(1)))?;
        let mut over_type_parser = SysmlParser::new(&over_type_tokens);
        let over_type_err = over_type_parser.parse_package_or_file("OverType").err().ok_or("63 nested Vec<...> must fail")?;
        assert!(over_type_err.message.contains("recursion depth"));
        assert_eq!(over_type_parser.depth, 0);

        let malformed_tokens = SysmlParser::tokenize_source("package P0 { package P1 { part def Bad { port broken : ; } } }")?;
        let mut malformed_parser = SysmlParser::new(&malformed_tokens);
        assert!(malformed_parser.parse_package_or_file("Bad").is_err());
        assert_eq!(malformed_parser.depth, 0);

        let mut many_calcs_src = String::from("package CalcDepthLeakProbe {\n");
        for idx in 0..80usize {
            many_calcs_src.push_str(&format!("    calc def C_{idx} {{ return (a + b); }}\n"));
        }
        many_calcs_src.push('}');
        let calc_tokens = SysmlParser::tokenize_source(&many_calcs_src)?;
        let mut calc_parser = SysmlParser::new(&calc_tokens);
        let calc_pkg = calc_parser.parse_package_or_file("CalcDepthLeakProbe")?;
        assert_eq!(calc_pkg.calc_defs.len(), 80);
        assert_eq!(calc_parser.depth, 0);

        assert_deep_parser_stack_probe()?;
        assert_deep_merge_stack_probe()?;
        Ok(())
    }
}
