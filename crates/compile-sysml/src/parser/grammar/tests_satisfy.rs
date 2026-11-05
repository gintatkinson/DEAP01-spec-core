//! Unit verification tests for `satisfy [requirement]` statements, zero-padded IDs, and comment interleaving.
//!
//! Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-TESTS]
//! Contract: Verifies package-level and part-level `satisfy` statement parsing, syntax variants, duplicate deduplication, zero-padded hyphenated requirement IDs, and comment tolerance at every lexical boundary.
//! Bounds & Faults: Executes deterministic in-memory parsing and serialization round-trip assertions with zero panics.

#[cfg(test)]
mod satisfy_stmt_tests {
    use super::super::*;

    #[test]
    fn test_issue_460_parse_and_roundtrip_satisfy_requirements(
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crate::semantic::serializer::SysmlSerializable;

        let source = r#"
package UserInterfaceSystem {
    satisfy requirement REQ-UI-00;
    part def TouchScreenPanel {
        satisfy requirement REQ-UI-01;
        satisfy requirement REQ-UI-02 by PrimaryTouchSensor;
        port in touchEvent;
    }
    part def NavigationController {
        satisfy requirement REQ-NAV-001;
        satisfy requirement REQ-CORE-002;
    }
}
"#;
        let pkg = SysmlParser::parse_source(source, "UserInterfaceSystem")?;
        assert_eq!(pkg.satisfied_requirements, vec!["REQ-UI-00".to_string()]);

        let touch_panel = pkg.part_defs.iter().find(|p| p.name == "TouchScreenPanel").ok_or("missing TouchScreenPanel")?;
        assert_eq!(touch_panel.satisfied_requirements, vec!["REQ-UI-01".to_string(), "REQ-UI-02".to_string()]);

        let nav_ctrl = pkg.part_defs.iter().find(|p| p.name == "NavigationController").ok_or("missing NavigationController")?;
        assert_eq!(nav_ctrl.satisfied_requirements, vec!["REQ-NAV-001".to_string(), "REQ-CORE-002".to_string()]);

        let serialized = pkg.to_sysml(0);
        for expected_line in [
            "satisfy requirement REQ-UI-00;",
            "satisfy requirement REQ-UI-01;",
            "satisfy requirement REQ-UI-02;",
            "satisfy requirement REQ-NAV-001;",
            "satisfy requirement REQ-CORE-002;",
        ] {
            assert!(serialized.contains(expected_line));
        }

        let reparsed = SysmlParser::parse_source(&serialized, "UserInterfaceSystem")?;
        assert_eq!(reparsed.satisfied_requirements, pkg.satisfied_requirements);
        let reparsed_touch = reparsed.part_defs.iter().find(|p| p.name == "TouchScreenPanel").ok_or("missing reparsed TouchScreenPanel")?;
        assert_eq!(reparsed_touch.satisfied_requirements, touch_panel.satisfied_requirements);
        let reparsed_nav = reparsed.part_defs.iter().find(|p| p.name == "NavigationController").ok_or("missing reparsed NavigationController")?;
        assert_eq!(reparsed_nav.satisfied_requirements, nav_ctrl.satisfied_requirements);
        Ok(())
    }

    #[test]
    fn test_issue_460_parse_satisfy_syntax_variants_and_deduplication(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source = r#"
package SyntaxVariantsModel {
    part def DisplaySubsystem {
        // Syntax variant: without 'requirement' keyword
        satisfy REQ-UI-01;
        // Duplicate satisfy statement should be deduplicated
        satisfy requirement REQ-UI-01;
        // Quoted requirement identifier
        satisfy requirement "REQ-UI-02";
        // Hyphenated ID with multiple segments and zero-padding
        satisfy requirement REQ-FMS-DISP-0042;
    }
}
"#;
        let pkg = SysmlParser::parse_source(source, "SyntaxVariantsModel")?;
        let display = pkg
            .part_defs
            .iter()
            .find(|p| p.name == "DisplaySubsystem")
            .ok_or("missing DisplaySubsystem")?;

        assert_eq!(
            display.satisfied_requirements,
            vec![
                "REQ-UI-01".to_string(),
                "REQ-UI-02".to_string(),
                "REQ-FMS-DISP-0042".to_string(),
            ],
            "syntax variants and zero-padded multi-hyphen identifiers must be parsed with duplicate deduplication"
        );
        Ok(())
    }

    #[test]
    fn test_issue_460_parse_satisfy_with_interspersed_block_comments(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source = r#"
package CommentVariantsModel {
    part def CommentSubsystem {
        // Block comment between satisfy and requirement
        satisfy /* comment 1 */ requirement REQ-UI-01;
        // Block comment between satisfy and ID (no requirement keyword)
        satisfy /* comment 2 */ REQ-UI-02;
        // Block comment between requirement and ID
        satisfy requirement /* comment 3 */ REQ-UI-03;
        // Block comment before semicolon
        satisfy requirement REQ-UI-04 /* comment 4 */;
        // Multiple block comments interspersed
        satisfy /* c1 */ /* c2 */ requirement /* c3 */ REQ-UI-05 /* c4 */;
        // Block comment with 'by' clause
        satisfy requirement REQ-UI-06 /* comment 5 */ by SubsystemImpl;
        // Qualified path
        satisfy requirement Core::UI::REQ-07;
    }
}
"#;
        let pkg = SysmlParser::parse_source(source, "CommentVariantsModel")?;
        let subsystem = pkg
            .part_defs
            .iter()
            .find(|p| p.name == "CommentSubsystem")
            .ok_or("missing CommentSubsystem")?;

        assert_eq!(
            subsystem.satisfied_requirements,
            vec![
                "REQ-UI-01".to_string(),
                "REQ-UI-02".to_string(),
                "REQ-UI-03".to_string(),
                "REQ-UI-04".to_string(),
                "REQ-UI-05".to_string(),
                "REQ-UI-06".to_string(),
                "Core::UI::REQ-07".to_string(),
            ],
            "interspersed block comments must not leak <dynamic> or prevent requirement keyword recognition"
        );
        Ok(())
    }

    #[test]
    fn test_issue_460_comments_between_satisfy_and_requirement(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let test_cases = [
            (
                "satisfy // single-line comment\n requirement REQ-01;",
                "REQ-01",
            ),
            (
                "satisfy /* single-line block comment */ requirement REQ-02;",
                "REQ-02",
            ),
            (
                "satisfy /* multi\n   line\n   block comment */ requirement REQ-03;",
                "REQ-03",
            ),
            (
                "satisfy /* nested /* block */ comment */ requirement REQ-04;",
                "REQ-04",
            ),
            (
                "satisfy doc /* doc comment style */ requirement REQ-05;",
                "REQ-05",
            ),
            (
                "satisfy /* doc: comment style */ requirement REQ-06;",
                "REQ-06",
            ),
            ("satisfy /* c1 */ /* c2 */ requirement REQ-07;", "REQ-07"),
            (
                "satisfy /* c1 */ // line\n /* c2 */ requirement REQ-08;",
                "REQ-08",
            ),
            (
                "satisfy /* satisfy requirement by */ requirement REQ-09;",
                "REQ-09",
            ),
            ("satisfy /**/ requirement REQ-10;", "REQ-10"),
        ];

        for (stmt, expected_id) in test_cases {
            let src = format!("package P {{ part def C {{ {} }} }}", stmt);
            let pkg = SysmlParser::parse_source(&src, "P")?;
            assert_eq!(
                pkg.part_defs.first().ok_or("missing part")?.satisfied_requirements,
                vec![expected_id.to_string()],
                "Failed to parse satisfy statement with comment between satisfy and requirement: {}",
                stmt
            );
        }
        Ok(())
    }

    #[test]
    fn test_issue_460_comments_between_requirement_and_identifier(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let test_cases = [
            ("satisfy requirement // line comment\n REQ-01;", "REQ-01"),
            ("satisfy requirement /* single block */ REQ-02;", "REQ-02"),
            (
                "satisfy requirement /* multi\nline\nblock */ REQ-03;",
                "REQ-03",
            ),
            (
                "satisfy requirement doc /* doc comment */ REQ-04;",
                "REQ-04",
            ),
            ("satisfy requirement /* doc: docstring */ REQ-05;", "REQ-05"),
            ("satisfy requirement /* c1 */ /* c2 */ REQ-06;", "REQ-06"),
            (
                "satisfy requirement /* c1 */ // line\n /* c2 */ REQ-07;",
                "REQ-07",
            ),
            (
                "satisfy /* comment without req keyword */ REQ-08;",
                "REQ-08",
            ),
            ("satisfy // line without req keyword\n REQ-09;", "REQ-09"),
            ("satisfy /* multi\nline without req */ REQ-10;", "REQ-10"),
            ("satisfy requirement /* c1 */ \"REQ-UI-01\";", "REQ-UI-01"),
            ("satisfy requirement /* c1 */ 'REQ-UI-02';", "REQ-UI-02"),
            ("satisfy /* c1 */ \"REQ-UI-03\";", "REQ-UI-03"),
        ];

        for (stmt, expected_id) in test_cases {
            let src = format!("package P {{ part def C {{ {} }} }}", stmt);
            let pkg = SysmlParser::parse_source(&src, "P")?;
            assert_eq!(
                pkg.part_defs
                    .first()
                    .ok_or("missing part")?
                    .satisfied_requirements,
                vec![expected_id.to_string()],
                "Failed to parse satisfy statement with comment between requirement and ID: {}",
                stmt
            );
        }
        Ok(())
    }

    #[test]
    fn test_issue_460_comments_within_identifier_tokens_and_hyphens(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let test_cases = [
            ("satisfy requirement REQ /* c */ -01;", "REQ-01"),
            ("satisfy requirement REQ- /* c */ 01;", "REQ-01"),
            (
                "satisfy requirement REQ /* c1 */ - /* c2 */ UI /* c3 */ - /* c4 */ 01;",
                "REQ-UI-01",
            ),
            ("satisfy requirement REQ- // line comment\n 01;", "REQ-01"),
            ("satisfy requirement REQ /* multi\nline */ -01;", "REQ-01"),
            ("satisfy requirement REQ /* nested /* c */ */ -01;", "REQ-01"),
            (
                "satisfy requirement REQ /* c1 */ - /* c2 */ FMS /* c3 */ - /* c4 */ DISP /* c5 */ - /* c6 */ 0042;",
                "REQ-FMS-DISP-0042",
            ),
            ("satisfy requirement Core /* c1 */ ::UI::REQ-01;", "Core::UI::REQ-01"),
            ("satisfy requirement Core:: /* c2 */ UI::REQ-01;", "Core::UI::REQ-01"),
            ("satisfy requirement Core::UI /* c3 */ ::REQ-01;", "Core::UI::REQ-01"),
            ("satisfy requirement Core::UI:: /* c4 */ REQ-01;", "Core::UI::REQ-01"),
            (
                "satisfy requirement Core /* c1 */ :: /* c2 */ UI /* c3 */ :: /* c4 */ REQ /* c5 */ - /* c6 */ 01;",
                "Core::UI::REQ-01",
            ),
        ];

        for (stmt, expected_id) in test_cases {
            let src = format!("package P {{ part def C {{ {} }} }}", stmt);
            let pkg = SysmlParser::parse_source(&src, "P")?;
            assert_eq!(
                pkg.part_defs
                    .first()
                    .ok_or("missing part")?
                    .satisfied_requirements,
                vec![expected_id.to_string()],
                "Failed to parse satisfy statement with comments within identifier tokens: {}",
                stmt
            );
        }
        Ok(())
    }

    #[test]
    fn test_issue_460_comments_before_terminating_semicolon(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let test_cases = [
            ("satisfy requirement REQ-01 // line comment\n ;", "REQ-01"),
            ("satisfy requirement REQ-02 /* block comment */;", "REQ-02"),
            (
                "satisfy requirement REQ-03 /* multi\nline\ncomment */;",
                "REQ-03",
            ),
            (
                "satisfy requirement REQ-04 /* nested /* comment */ */;",
                "REQ-04",
            ),
            ("satisfy requirement REQ-05 /* c1 */ /* c2 */;", "REQ-05"),
            (
                "satisfy requirement REQ-06 /* c1 */ // line\n /* c2 */;",
                "REQ-06",
            ),
            ("satisfy requirement REQ-07 /* ;;; */;", "REQ-07"),
            ("satisfy requirement REQ-08 /* { } */;", "REQ-08"),
            (
                "satisfy requirement \"REQ-09\" /* block comment */;",
                "REQ-09",
            ),
            (
                "satisfy requirement 'REQ-10' /* block comment */;",
                "REQ-10",
            ),
        ];

        for (stmt, expected_id) in test_cases {
            let src = format!("package P {{ part def C {{ {} }} }}", stmt);
            let pkg = SysmlParser::parse_source(&src, "P")?;
            assert_eq!(
                pkg.part_defs
                    .first()
                    .ok_or("missing part")?
                    .satisfied_requirements,
                vec![expected_id.to_string()],
                "Failed to parse satisfy statement with comments before terminating semicolon: {}",
                stmt
            );
        }
        Ok(())
    }
}
