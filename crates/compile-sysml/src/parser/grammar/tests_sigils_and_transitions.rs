//! Type-sigil byte-span preservation, lifetime reference, and state-machine transition round-trip grammar tests.
//!
//! Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-TESTS]
//! Contract: Verifies 1-byte span invariance across `&`, `?`, and `'a` lifetime sigils, single-quoted unrestricted identifier disambiguation, structured `first`/`then` and `from`/`to` transition parsing, qualified endpoints, and state-action / doc-comment round-trips.
//! Bounds & Faults: Executes deterministic in-memory tokenization, parsing, and serialization round-trip assertions with zero panics.

#[cfg(test)]
mod sigil_and_transition_tests {
    use super::super::*;

    /// Verifies structured `source`, `target`, `name`, `trigger`, `guard`, and `effect` extraction on `TransitionDef` for both `first <src> then <tgt>` and `from <src> to <tgt>` forms.
    #[test]
    fn test_defect_06_parse_state_transitions_first_then_and_from_to(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source = r#"
package StateMachinePkg {
    part def FlightController {
        state def FlightMode {
            state Init;
            state Standby;
            state Armed;
            state Active;
            state SafeMode;

            transition first Init then Standby;
            transition t_arm first Standby accept ArmCmd if battery_pct > 20.0 do EnableServos then Armed;
            transition from Armed to Active;
            transition t_fault from Active to SafeMode accept FaultSignal if pitch_deg > 15.0 do EngageBackup;
        }
    }
}
"#;
        let pkg = SysmlParser::parse_source(source, "StateMachinePkg")?;
        let part = pkg.part_defs.first().ok_or("missing FlightController")?;
        let state_def = part.states.first().ok_or("missing FlightMode")?;
        assert_eq!(state_def.transitions.len(), 4);

        let tr0 = state_def.transitions.first().ok_or("missing transition 0")?;
        assert!(tr0.name.is_none(), "unnamed transition must have name == None, got {:?}", tr0.name);
        assert_eq!(tr0.source.as_deref(), Some("Init"));
        assert_eq!(tr0.target.as_deref(), Some("Standby"));
        assert_eq!((&tr0.trigger, &tr0.guard, &tr0.effect), (&None, &None, &None));

        let tr1 = state_def.transitions.get(1).ok_or("missing transition 1")?;
        assert_eq!(tr1.name.as_deref(), Some("t_arm"));
        assert_eq!(tr1.source.as_deref(), Some("Standby"));
        assert_eq!(tr1.target.as_deref(), Some("Armed"));
        assert_eq!(tr1.trigger.as_deref(), Some("ArmCmd"));
        assert_eq!(tr1.guard.as_deref(), Some("battery_pct > 20.0"));
        assert_eq!(tr1.effect.as_deref(), Some("EnableServos"));

        let tr2 = state_def.transitions.get(2).ok_or("missing transition 2")?;
        assert!(tr2.name.is_none(), "unnamed transition must have name == None, got {:?}", tr2.name);
        assert_eq!(tr2.source.as_deref(), Some("Armed"));
        assert_eq!(tr2.target.as_deref(), Some("Active"));
        assert_eq!((&tr2.trigger, &tr2.guard, &tr2.effect), (&None, &None, &None));

        let tr3 = state_def.transitions.get(3).ok_or("missing transition 3")?;
        assert_eq!(tr3.name.as_deref(), Some("t_fault"));
        assert_eq!(tr3.source.as_deref(), Some("Active"));
        assert_eq!(tr3.target.as_deref(), Some("SafeMode"));
        assert_eq!(tr3.trigger.as_deref(), Some("FaultSignal"));
        assert_eq!(tr3.guard.as_deref(), Some("pitch_deg > 15.0"));
        assert_eq!(tr3.effect.as_deref(), Some("EngageBackup"));
        Ok(())
    }

    /// Verifies that `&` and `?` type sigil tokenization preserves exact 1-byte source spans, prevents diagnostic span drift, and distinguishes raw sigils from `'&'` and `'?'` unrestricted identifiers.
    #[test]
    fn test_issue_495_type_sigil_tokenization_preserves_spans_and_unrestricted_idents(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let src = "package P {\n    attribute r : &T?;\n    attribute x : Integer;\n}\n";
        let tokens = SysmlParser::tokenize_source(src)?;
        for (ident, expected_span) in [
            ("&", Span { start: 30, end: 31, line: 2, col: 19 }),
            ("T", Span { start: 31, end: 32, line: 2, col: 20 }),
            ("?", Span { start: 32, end: 33, line: 2, col: 21 }),
            ("x", Span { start: 49, end: 50, line: 3, col: 15 }),
        ] {
            let tok = tokens
                .iter()
                .find(|t| t.kind == TokenKind::Ident(ident.to_string()))
                .ok_or("missing expected token")?;
            assert_eq!(tok.span, expected_span);
        }

        let bad_same_line = "package P {\n    part a : &T; part b : <>;\n}\n";
        let expected_lt_start = bad_same_line.find('<').ok_or("missing '<'")?;
        let err_same = SysmlParser::parse_source(bad_same_line, "P").err().ok_or("empty '<>' must fail")?;
        assert_eq!(err_same.span, Span { start: expected_lt_start, end: expected_lt_start + 1, line: 2, col: 27 });

        let bad_cross_line = "package P {\n    attribute a : &T?;\n    atribute b : Integer;\n}\n";
        let expected_typo_start = bad_cross_line.find("atribute").ok_or("missing typo")?;
        let err_cross = SysmlParser::parse_source(bad_cross_line, "P").err().ok_or("typo must fail")?;
        assert_eq!(
            err_cross.span,
            Span { start: expected_typo_start, end: expected_typo_start + "atribute".len(), line: 3, col: 5 }
        );

        let quoted_ident_src = "package P {\n    part def '&' {\n        attribute '?' : '&';\n    }\n}\n";
        let quoted_pkg = SysmlParser::parse_source(quoted_ident_src, "P")?;
        let quoted_part = quoted_pkg.part_defs.first().ok_or("missing part def '&'")?;
        assert_eq!(quoted_part.name, "&");
        let quoted_attr = quoted_part.attributes.first().ok_or("missing attribute '?'")?;
        assert_eq!(quoted_attr.name, "?");
        assert_eq!(quoted_attr.type_name, "&");

        assert!(SysmlParser::parse_source("package P { part def &; }", "P").is_err());
        assert!(SysmlParser::parse_source("package P { part def ?; }", "P").is_err());
        assert!(SysmlParser::parse_source("package P { attribute x : ?; }", "P").is_err());

        let expr_and_escape_src = "package P {\n    part def 'a\\'&b' {\n        attribute mask : Integer = a & b;\n        constraint def C {\n            (x > 0) ? a : b;\n        }\n    }\n}\n";
        let expr_pkg = SysmlParser::parse_source(expr_and_escape_src, "P")?;
        let escaped_part = expr_pkg.part_defs.first().ok_or("missing escaped part")?;
        assert_eq!(escaped_part.name, "a'&b");
        assert_eq!(escaped_part.attributes.first().and_then(|a| a.default_value.as_deref()), Some("a & b"));
        assert_eq!(escaped_part.constraints.first().map(|c| c.expression.as_str()), Some("( x > 0 ) ? a : b"));

        let expr_err_src = format!("{expr_and_escape_src}\npackage Bad {{\n    atribute z : Integer;\n}}\n");
        let z_typo_start = expr_err_src.find("atribute").ok_or("missing atribute")?;
        let expr_err = SysmlParser::parse_source(&expr_err_src, "P")
            .err()
            .ok_or("expected error on trailing package typo")?;
        assert_eq!(expr_err.span.start, z_typo_start);
        assert_eq!(expr_err.span.end, z_typo_start + "atribute".len());
        Ok(())
    }

    /// Verifies that lifetime sigils (`&'a T?`, `&'static mut [u8]`) preserve 1-byte spans and `TransitionDef` clauses round-trip losslessly.
    #[test]
    fn test_issue_521_lifetime_sigils_and_transition_clause_roundtrip(
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crate::semantic::serializer::SysmlSerializable;

        let lifetime_src =
            "package P {\n    attribute r : &'a T?;\n    attribute buf : &'static mut [u8];\n}\n";
        let tokens = SysmlParser::tokenize_source(lifetime_src)?;
        let q_tok = tokens
            .iter()
            .find(|t| t.kind == TokenKind::Ident("?".to_string()))
            .ok_or("missing '?' token after lifetime reference &'a T")?;
        assert_eq!(q_tok.span, Span { start: 35, end: 36, line: 2, col: 24 });

        let pkg = SysmlParser::parse_source(lifetime_src, "P")?;
        assert_eq!(pkg.attribute_defs.first().map(|a| a.type_name.as_str()), Some("&'a T?"));
        assert_eq!(pkg.attribute_defs.get(1).map(|a| a.type_name.as_str()), Some("&'static mut [u8]"));

        let multi_lifetime_src = "package P {\n    attribute pair : (&'a T, &'b mut U?);\n}\n";
        let multi_pkg = SysmlParser::parse_source(multi_lifetime_src, "P")?;
        assert_eq!(multi_pkg.attribute_defs.first().map(|a| a.type_name.as_str()), Some("(&'a T, &'b mut U?)"));
        assert!(SysmlParser::parse_source("package P { part def 'a; }", "P").is_err());

        let state_src = "package ModePkg {\n    state def OperatingMode {\n        state Idle;\n        state Active;\n        transition first Idle accept EngageCmd if is_ready do init_actuators then Active;\n    }\n}\n";
        let state_pkg1 = SysmlParser::parse_source(state_src, "ModePkg")?;
        let t1 = state_pkg1.state_defs.first().and_then(|s| s.transitions.first()).ok_or("missing t1")?;
        assert_eq!(t1.source.as_deref(), Some("Idle"));
        assert_eq!(t1.target.as_deref(), Some("Active"));
        assert_eq!(t1.trigger.as_deref(), Some("EngageCmd"));
        assert_eq!(t1.guard.as_deref(), Some("is_ready"));
        assert_eq!(t1.effect.as_deref(), Some("init_actuators"));

        let emitted = state_pkg1.to_sysml(0);
        let state_pkg2 = SysmlParser::parse_source(&emitted, "ModePkg")?;
        let t2 = state_pkg2.state_defs.first().and_then(|s| s.transitions.first()).ok_or("missing t2")?;
        assert_eq!(t2.source.as_deref(), Some("Idle"));
        assert_eq!(t2.target.as_deref(), Some("Active"));
        assert_eq!(t2.trigger.as_deref(), Some("EngageCmd"));
        assert_eq!(t2.guard.as_deref(), Some("is_ready"));
        assert_eq!(t2.effect.as_deref(), Some("init_actuators"));

        for named_raw in [
            "t_arm first Idle accept ArmCmd if safety_ok do lock_interlock then Armed",
            "t_fault first Armed accept FaultDetected then Faulted",
        ] {
            let parsed = SysmlParser::parse_transition_clause(named_raw);
            let formatted = SysmlParser::format_transition_clause(&parsed);
            assert_eq!(formatted, named_raw);
            let reparsed = SysmlParser::parse_transition_clause(&formatted);
            assert_eq!(parsed, reparsed);
        }
        Ok(())
    }

    /// Verifies qualified/dotted state transition endpoints (`Mode::Standby`, `Sub.Armed`), keywords inside call arguments, and `StateDef::to_sysml` round-trips.
    #[test]
    fn test_issue_496_transition_tokens_qualified_endpoints_and_roundtrip(
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crate::semantic::serializer::SysmlSerializable;

        let source = r#"
package Issue496Pkg {
    state def FlightMode {
        state Init;
        state Standby;
        state Armed;

        transition first Init then Mode::Standby;
        transition first Mode::Standby accept Sig::Arm(from) if check(to, speed > 0) do sendCmd(x, y) then Sub.Armed;
        transition t_named first Sub.Armed accept Nav::Disarm if isSafe(first) do resetState(then) then Init;
    }
}
"#;
        let pkg = SysmlParser::parse_source(source, "Issue496Pkg")?;
        let st = pkg.state_defs.first().ok_or("missing FlightMode")?;
        assert_eq!(st.transitions.len(), 3);

        let t0 = st.transitions.first().ok_or("missing transition 0")?;
        assert_eq!(t0.name, None);
        assert_eq!(t0.source.as_deref(), Some("Init"));
        assert_eq!(t0.target.as_deref(), Some("Mode::Standby"));
        assert_eq!((&t0.trigger, &t0.guard, &t0.effect), (&None, &None, &None));

        let t1 = st.transitions.get(1).ok_or("missing transition 1")?;
        assert_eq!(t1.name, None);
        assert_eq!(t1.source.as_deref(), Some("Mode::Standby"));
        assert_eq!(t1.target.as_deref(), Some("Sub.Armed"));
        assert_eq!(t1.trigger.as_deref(), Some("Sig::Arm(from)"));
        assert_eq!(t1.guard.as_deref(), Some("check(to, speed > 0)"));
        assert_eq!(t1.effect.as_deref(), Some("sendCmd(x, y)"));

        let t2 = st.transitions.get(2).ok_or("missing transition 2")?;
        assert_eq!(t2.name.as_deref(), Some("t_named"));
        assert_eq!(t2.source.as_deref(), Some("Sub.Armed"));
        assert_eq!(t2.target.as_deref(), Some("Init"));
        assert_eq!(t2.trigger.as_deref(), Some("Nav::Disarm"));
        assert_eq!(t2.guard.as_deref(), Some("isSafe(first)"));
        assert_eq!(t2.effect.as_deref(), Some("resetState(then)"));

        let serialized = pkg.to_sysml(0);
        assert!(serialized.contains("transition first Init then Mode::Standby;"));
        assert!(serialized.contains("transition first Mode::Standby accept Sig::Arm(from) if check(to, speed > 0) do sendCmd(x, y) then Sub.Armed;"));
        assert!(serialized.contains("transition t_named first Sub.Armed accept Nav::Disarm if isSafe(first) do resetState(then) then Init;"));

        let reparsed = SysmlParser::parse_source(&serialized, "Issue496Pkg")?;
        assert_eq!(reparsed, pkg);
        Ok(())
    }

    /// Verifies token-aware formatting for `entry`/`do`/`exit` actions, contextual `action` identifier preservation, and inner state/transition `doc` round-trips.
    #[test]
    fn test_issue_509_state_action_clauses_and_doc_comments_roundtrip(
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crate::semantic::serializer::SysmlSerializable;

        let source = r#"
package Issue509Pkg {
    doc /* Issue 509 package */
    state def GuidanceState {
        doc /* Primary guidance state machine */
        entry action Nav::initFilter(x, y); // inline comment must not become <dynamic>
        do ctrl.runStep(a[0], b);
        exit action Actuator::shutdown();

        doc /* Nominal standby sub-state */
        state Standby;

        doc /* Active guidance sub-state */
        state Active {
            entry action;
            do action::step(1);
        }

        doc /* Transition from Standby to Active */
        transition t_engage first Standby accept EngageCmd if navReady(x) do action EnableServos(1, 2) then Active;
    }
}
"#;
        let pkg = SysmlParser::parse_source(source, "Issue509Pkg")?;
        assert_eq!(pkg.doc.as_deref(), Some("Issue 509 package"));
        let st = pkg.state_defs.first().ok_or("missing GuidanceState")?;
        assert_eq!(st.name, "GuidanceState");
        assert_eq!(st.doc.as_deref(), Some("Primary guidance state machine"));
        assert_eq!(st.entry_action.as_deref(), Some("Nav::initFilter(x, y)"));
        assert_eq!(st.do_action.as_deref(), Some("ctrl.runStep(a[0], b)"));
        assert_eq!(st.exit_action.as_deref(), Some("Actuator::shutdown()"));

        assert_eq!(st.sub_states.len(), 2);
        let sub0 = st.sub_states.first().ok_or("missing Standby sub-state")?;
        assert_eq!(sub0.name, "Standby");
        assert_eq!(sub0.doc.as_deref(), Some("Nominal standby sub-state"));

        let sub1 = st.sub_states.get(1).ok_or("missing Active sub-state")?;
        assert_eq!(sub1.name, "Active");
        assert_eq!(sub1.doc.as_deref(), Some("Active guidance sub-state"));
        assert_eq!(sub1.entry_action.as_deref(), Some("action"));
        assert_eq!(sub1.do_action.as_deref(), Some("action::step(1)"));

        assert_eq!(st.transitions.len(), 1);
        let tr = st.transitions.first().ok_or("missing transition t_engage")?;
        assert_eq!(tr.name.as_deref(), Some("t_engage"));
        assert_eq!(tr.doc.as_deref(), Some("Transition from Standby to Active"));
        assert_eq!(tr.source.as_deref(), Some("Standby"));
        assert_eq!(tr.target.as_deref(), Some("Active"));
        assert_eq!(tr.trigger.as_deref(), Some("EngageCmd"));
        assert_eq!(tr.guard.as_deref(), Some("navReady(x)"));
        assert_eq!(tr.effect.as_deref(), Some("EnableServos(1, 2)"));

        let serialized = pkg.to_sysml(0);
        assert!(serialized.contains("entry Nav::initFilter(x, y);"));
        assert!(serialized.contains("do ctrl.runStep(a[0], b);"));
        assert!(serialized.contains("exit Actuator::shutdown();"));
        assert!(serialized.contains("doc /* Transition from Standby to Active */"));

        let reparsed = SysmlParser::parse_source(&serialized, "Issue509Pkg")?;
        assert_eq!(reparsed, pkg);
        Ok(())
    }
}
