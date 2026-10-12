//! # Heterogeneous Specification AST Translators
//!
//! ## 1. Safety Intent & Regulatory Scope
//! This module provides deterministic translator engines converting heterogeneous technical
//! specifications into canonical SysML v2 textual Abstract Syntax Tree (AST) representations.
//!
//! In DO-178C Level A and ISO 26262 ASIL D architectures, specification translators bridge
//! OEM-supplied interface definitions, contractual models, and requirements with the formal
//! SysML v2 compilation pipeline.
//!
//! ## 2. Supported Translators
//! - [`idl`]: Translates OMG IDL 3.x/4.x interface definitions into SysML packages, parts, and actions.
//! - [`markdown`]: Translates Level 0 OEM Markdown requirements, BDD criteria, and tables into SysML packages, parts, ports, and constraints.
//! - [`openapi`]: Translates OpenAPI 3.0/3.1 RESTful schemas and routes into SysML parts and action endpoints.
//! - [`yang`]: Translates RFC 6020/7950 YANG data models into SysML packages, parts, attributes, actions, and use cases.

pub mod idl;
pub mod markdown;
pub mod openapi;
pub mod yang;
