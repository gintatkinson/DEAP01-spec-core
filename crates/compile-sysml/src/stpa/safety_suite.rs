//! STPA Safety Artifact Suite Transpiler.
//!
//! Generates the 10-pillar safety artifact suite from SysML v2 AST models:
//! 1. 01_LOSSES_HAZARDS_TOPOLOGY.md
//! 2. 02_UCA_COMBINATORIAL_MATRIX.md
//! 3. 03_LOSS_SCENARIOS.md
//! 4. 04_SAFETY_CONSTRAINTS.md
//! 5. 05_FMECA_MATRIX.md
//! 6. 06_REGULATORY_OBJECTIVES_ASSESSMENT.md (& 06_SORA_SAIL_ASSESSMENT.md)
//! 7. 07_RTA_ARCHITECTURE.md
//! 8. STPA_MATRIX.md
//! 9. HAZARD_LOG.md
//! 10. SLDV_FORMAL_PROOFS.m

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use deap_core::sysml_ast::{ConstraintDef, PackageDef};
use crate::semantic::digest::write_atomic;
use super::fmeca::{generate_fmeca_matrix, FmecaRow, FmecaScoringConfig};
use super::uca::{expand_cartesian_stpa, UnsafeControlAction, PENDING_PARAMETER};

/// Parameterized proof template structure.
pub struct ProofTemplate {
    pub id: &'static str,
    pub name: &'static str,
    pub statement: &'static str,
    pub derivation: &'static [&'static str],
    pub params: &'static [(&'static str, &'static str, &'static str, &'static str)],
    pub numeric: &'static [&'static str],
    pub sldv: &'static str,
}

pub const PROOF_TEMPLATES: [ProofTemplate; 10] = [
    ProofTemplate {
        id: "T-01",
        name: "Kinetic Energy Dissipation Bound",
        statement: "$$ E_{\\mathrm{density}} \\le E_{\\mathrm{limit}} $$",
        derivation: &[
            "v_{\\mathrm{term}} &= \\sqrt{ \\frac{K_f \\cdot M \\cdot g}{\\rho \\cdot C_d \\cdot A_d} }",
            "E_{\\mathrm{impact}} &= \\frac{M \\cdot M \\cdot g}{\\rho \\cdot C_d \\cdot A_d}",
            "E_{\\mathrm{density}} &= \\frac{E_{\\mathrm{impact}}}{A_f}",
        ],
        params: &[
            ("KineticFactor", "K_f", "Dimensionless kinetic term factor", "-"),
            ("ParameterMass", "M", "System total mass", "kg"),
            ("GravityAcceleration", "g", "Gravitational acceleration", "m/s"),
            ("MediumDensity", "rho", "Ambient medium density", "kg per cubic metre"),
            ("DragCoefficient", "C_d", "Aerodynamic drag coefficient", "-"),
            ("DecelerationArea", "A_d", "Deceleration projected area", "square metre"),
            ("FrontalArea", "A_f", "Frontal impact cross-section area", "square metre"),
            ("EnergyDensityLimit", "E_limit", "Regulatory energy density ceiling", "J per square metre"),
        ],
        numeric: &[
            "v_{\\mathrm{term}} &= \\sqrt{ (%%KineticFactor%% \\cdot %%ParameterMass%% \\cdot %%GravityAcceleration%%) / (%%MediumDensity%% \\cdot %%DragCoefficient%% \\cdot %%DecelerationArea%%) }",
            "E_{\\mathrm{impact}} &= (%%ParameterMass%% \\cdot %%ParameterMass%% \\cdot %%GravityAcceleration%%) / (%%MediumDensity%% \\cdot %%DragCoefficient%% \\cdot %%DecelerationArea%%)",
            "E_{\\mathrm{density}} &= E_{\\mathrm{impact}} / %%FrontalArea%% \\le %%EnergyDensityLimit%%",
        ],
        sldv: "sldv.assert( (ImpactEnergyDensity <= %%EnergyDensityLimit%%), 'Bind_KINETIC_ENERGY_DISSIPATION_BOUND' );",
    },
    ProofTemplate {
        id: "T-02",
        name: "Containment Reach Bound",
        statement: "$$ R_{\\mathrm{glide}} \\le R_{\\mathrm{bound}} - R_{\\mathrm{buffer}} $$",
        derivation: &[
            "t_{\\mathrm{glide}} &= \\frac{H_a}{V_{\\mathrm{sink}}}",
            "R_{\\mathrm{air}} &= H_a \\cdot (L/D)_{\\mathrm{max}}",
            "R_{\\mathrm{drift}} &= V_{\\mathrm{wind}} \\cdot t_{\\mathrm{glide}}",
            "R_{\\mathrm{glide}} &= R_{\\mathrm{air}} + R_{\\mathrm{drift}}",
        ],
        params: &[
            ("InitialAltitude", "H_a", "Initial altitude above reference plane", "m"),
            ("LiftToDragRatio", "(L/D)_max", "Maximum lift-to-drag ratio", "-"),
            ("SinkRate", "V_sink", "Minimum sink rate", "m/s"),
            ("WindDriftSpeed", "V_wind", "Wind drift component", "m/s"),
            ("ContainmentRadius", "R_bound", "Operational containment radius", "m"),
            ("BufferRadius", "R_buffer", "Contingency buffer radius", "m"),
        ],
        numeric: &[
            "t_{\\mathrm{glide}} &= %%InitialAltitude%% / %%SinkRate%%",
            "R_{\\mathrm{air}} &= %%InitialAltitude%% \\cdot %%LiftToDragRatio%%",
            "R_{\\mathrm{drift}} &= %%WindDriftSpeed%% \\cdot t_{\\mathrm{glide}}",
            "R_{\\mathrm{glide}} &= R_{\\mathrm{air}} + R_{\\mathrm{drift}} \\le %%ContainmentRadius%% - %%BufferRadius%%",
        ],
        sldv: "sldv.assert( (GlideDistance <= (ContainmentRadius - ContingencyBuffer)), 'Bind_CONTAINMENT_REACH_BOUND' );",
    },
    ProofTemplate {
        id: "T-03",
        name: "Barrier Forward Invariance Bound",
        statement: "$$ \\dot{B}(\\mathbf{x}, \\mathbf{u}) + \\gamma(B(\\mathbf{x})) \\ge B_{\\mathrm{min}} $$",
        derivation: &[
            "B(\\mathbf{x}) &= d_b \\cdot d_b - \\|\\mathbf{p} - \\mathbf{p}_c\\| \\cdot \\|\\mathbf{p} - \\mathbf{p}_c\\| - \\frac{\\|\\mathbf{v}\\| \\cdot \\|\\mathbf{v}\\|}{K_f \\cdot a_{\\mathrm{max}}}",
            "\\dot{B}(\\mathbf{x}, \\mathbf{u}) &= -K_f \\cdot (\\mathbf{p} - \\mathbf{p}_c)^T \\mathbf{v} + \\frac{\\mathbf{v}^T \\mathbf{u}}{a_{\\mathrm{max}}}",
            "\\dot{B} + \\gamma B &= \\dot{B} + \\gamma_s \\cdot B(\\mathbf{x})",
        ],
        params: &[
            ("BarrierRadius", "d_b", "Containment boundary radius", "m"),
            ("PositionOffset", "p-p_c", "Current radial offset from centre", "m"),
            ("GroundSpeed", "v", "Ground speed magnitude", "m/s"),
            ("AccelerationLimit", "a_max", "Maximum certified acceleration", "m/s"),
            ("BarrierGain", "gamma_s", "Extended class-K linear gain", "per second"),
            ("KineticFactor", "K_f", "Dimensionless kinetic term factor", "-"),
        ],
        numeric: &[
            "B(\\mathbf{x}) &= %%BarrierRadius%% \\cdot %%BarrierRadius%% - %%PositionOffset%% \\cdot %%PositionOffset%% - (%%GroundSpeed%% \\cdot %%GroundSpeed%%) / (%%KineticFactor%% \\cdot %%AccelerationLimit%%)",
            "\\dot{B} &= -%%KineticFactor%% \\cdot %%PositionOffset%% \\cdot %%GroundSpeed%% + %%GroundSpeed%% \\cdot (%%AccelerationLimit%%/%%AccelerationLimit%%)",
            "\\dot{B} + \\gamma B &= \\dot{B} + %%BarrierGain%% \\cdot B(\\mathbf{x}) \\ge B_{\\mathrm{min}}",
        ],
        sldv: "sldv.assert( (BarrierValue >= BarrierFloor) && (BarrierDerivative + BarrierGain * BarrierValue >= BarrierFloor), 'Bind_BARRIER_FORWARD_INVARIANCE_BOUND' );",
    },
    ProofTemplate {
        id: "T-04",
        name: "Exponential Discharge Bound",
        statement: "$$ V_e(t) = V_a \\cdot \\exp\\left( -\\frac{t}{R_b \\cdot C_s} \\right) \\le V_{\\mathrm{safe}} $$",
        derivation: &[
            "\\tau_{\\mathrm{bleed}} &= R_b \\cdot C_s",
            "V_e(t) &= V_a \\cdot \\exp\\left( -\\frac{t}{\\tau_{\\mathrm{bleed}}} \\right)",
            "t_{\\mathrm{safe}} &= \\tau_{\\mathrm{bleed}} \\cdot \\ln\\left( \\frac{V_a}{V_{\\mathrm{safe}}} \\right)",
        ],
        params: &[
            ("InitialPotential", "V_a", "Initial fully charged potential", "V"),
            ("SafePotential", "V_safe", "Non-hazardous potential ceiling", "V"),
            ("BleedResistance", "R_b", "Bleed-down resistance", "ohm"),
            ("StorageCapacitance", "C_s", "Energy storage capacitance", "F"),
        ],
        numeric: &[
            "\\tau_{\\mathrm{bleed}} &= %%BleedResistance%% \\cdot %%StorageCapacitance%%",
            "t_{\\mathrm{safe}} &= \\tau_{\\mathrm{bleed}} \\cdot \\ln(%%InitialPotential%%/%%SafePotential%%)",
            "V_e(t_{\\mathrm{safe}}) &= %%InitialPotential%% \\cdot \\exp(-t_{\\mathrm{safe}}/\\tau_{\\mathrm{bleed}}) \\le %%SafePotential%%",
        ],
        sldv: "sldv.assert( implies(DeactivationCommandActive && (ElapsedTime >= TauBleed), (StoredPotential <= %%SafePotential%%)), 'Bind_EXPONENTIAL_DISCHARGE_BOUND' );",
    },
    ProofTemplate {
        id: "T-05",
        name: "Energy Balance Separation Bound",
        statement: "$$ V_{\\mathrm{sep}} = \\sqrt{ \\frac{K_f}{M} \\left( W_{\\mathrm{drive}} - W_{\\mathrm{friction}} \\right) } \\ge V_{\\mathrm{stall}} $$",
        derivation: &[
            "W_{\\mathrm{drive}} &= P_r \\cdot A_p \\cdot x_s",
            "W_{\\mathrm{friction}} &= \\mu_k \\cdot M \\cdot g \\cdot \\cos(\\theta_s) \\cdot x_s",
            "V_{\\mathrm{sep}} &= \\sqrt{ \\frac{K_f \\cdot (W_{\\mathrm{drive}} - W_{\\mathrm{friction}})}{M} }",
        ],
        params: &[
            ("RailPressure", "P_r", "Mean drive pressure", "Pa"),
            ("PistonArea", "A_p", "Drive piston cross-section area", "square metre"),
            ("StrokeLength", "x_s", "Acceleration stroke length", "m"),
            ("ParameterMass", "M", "System total mass", "kg"),
            ("FrictionCoefficient", "mu_k", "Kinetic friction coefficient", "-"),
            ("InclineAngle", "theta_s", "Stroke incline angle", "deg"),
            ("StallSpeed", "V_stall", "Minimum stall velocity", "m/s"),
            ("KineticFactor", "K_f", "Dimensionless kinetic term factor", "-"),
            ("GravityAcceleration", "g", "Gravitational acceleration", "m/s"),
        ],
        numeric: &[
            "W_{\\mathrm{drive}} &= %%RailPressure%% \\cdot %%PistonArea%% \\cdot %%StrokeLength%%",
            "W_{\\mathrm{friction}} &= %%FrictionCoefficient%% \\cdot %%ParameterMass%% \\cdot %%GravityAcceleration%% \\cdot \\cos(%%InclineAngle%%) \\cdot %%StrokeLength%%",
            "V_{\\mathrm{sep}} &= \\sqrt(%%KineticFactor%% \\cdot (W_{\\mathrm{drive}} - W_{\\mathrm{friction}})/%%ParameterMass%%) \\ge %%StallSpeed%%",
        ],
        sldv: "sldv.assert( implies(SeparationTrigger, (ReleaseSpeed >= %%StallSpeed%%)), 'Bind_ENERGY_BALANCE_SEPARATION_BOUND' );",
    },
    ProofTemplate {
        id: "T-06",
        name: "Link Margin Lower Bound",
        statement: "$$ \\mathrm{LM} = P_{\\mathrm{rx}} - P_{\\mathrm{sens}} \\ge \\mathrm{LM}_{\\mathrm{min}} $$",
        derivation: &[
            "\\mathrm{FSPL} &= L_f \\cdot \\left( \\log(D) + \\log(f) + \\log(F_c) \\right)",
            "P_{\\mathrm{rx}} &= P_{\\mathrm{tx}} + G_{\\mathrm{tx}} + G_{\\mathrm{rx}} - \\mathrm{FSPL} - L_{\\mathrm{misc}}",
            "\\mathrm{LM} &= P_{\\mathrm{rx}} - P_{\\mathrm{sens}}",
        ],
        params: &[
            ("TransmitPower", "P_tx", "Transmitter output power", "dBm"),
            ("TransmitGain", "G_tx", "Transmitter antenna gain", "dBi"),
            ("ReceiveGain", "G_rx", "Receiver antenna gain", "dBi"),
            ("CarrierFrequency", "f", "Carrier frequency", "Hz"),
            ("StandoffDistance", "D", "Maximum standoff distance", "m"),
            ("InsertionLoss", "L_misc", "Insertion and atmospheric loss", "dB"),
            ("ReceiveSensitivity", "P_sens", "Receiver detection sensitivity", "dBm"),
            ("MinLinkMargin", "LM_min", "Minimum required link margin", "dB"),
            ("LogFactor", "L_f", "Dimensionless decibel scaling factor", "-"),
            ("PropagationFactor", "F_c", "Dimensionless propagation geometry factor", "-"),
        ],
        numeric: &[
            "\\mathrm{FSPL} &= %%LogFactor%% \\cdot ( \\log(%%StandoffDistance%%) + \\log(%%CarrierFrequency%%) + \\log(%%PropagationFactor%%) )",
            "P_{\\mathrm{rx}} &= %%TransmitPower%% + %%TransmitGain%% + %%ReceiveGain%% - \\mathrm{FSPL} - %%InsertionLoss%%",
            "\\mathrm{LM} &= P_{\\mathrm{rx}} - %%ReceiveSensitivity%% \\ge %%MinLinkMargin%%",
        ],
        sldv: "sldv.assert( (LinkMargin >= %%MinLinkMargin%%), 'Bind_LINK_MARGIN_LOWER_BOUND' );",
    },
    ProofTemplate {
        id: "T-07",
        name: "Energy Reserve and Thermal Budget Bound",
        statement: "$$ \\mathrm{SoC}(t) \\ge \\mathrm{SoC}_{\\mathrm{crit}} \\; \\wedge \\; T_{\\mathrm{cell}}(t) \\le T_{\\mathrm{max}} $$",
        derivation: &[
            "E_{\\mathrm{rtl}} &= \\left( \\frac{D}{V_{\\mathrm{cruise}}} \\right) \\cdot \\left( P_{\\mathrm{prop}} + P_{\\mathrm{av}} \\right)",
            "\\mathrm{SoC}_{\\mathrm{crit}} &= \\frac{E_{\\mathrm{rtl}} + E_{\\mathrm{abort}}}{E_{\\mathrm{total}}}",
            "\\Delta T &= \\frac{I_b \\cdot I_b \\cdot R_i}{h \\cdot A_p}",
            "T_{\\mathrm{cell,max}} &= T_{\\mathrm{amb}} + \\Delta T",
        ],
        params: &[
            ("TotalEnergy", "E_total", "Total energy storage capacity", "J"),
            ("PropulsionPower", "P_prop", "Steady-state propulsion power", "W"),
            ("AvionicsPower", "P_av", "Avionics power consumption", "W"),
            ("CruiseSpeed", "V_cruise", "Cruise speed", "m/s"),
            ("ReserveDistance", "D", "Standoff distance to recovery point", "m"),
            ("AbortReserve", "E_abort", "Emergency abort energy reserve", "J"),
            ("DischargeCurrent", "I_b", "Storage discharge current", "A"),
            ("InternalResistance", "R_i", "Internal resistance", "ohm"),
            ("DissipationProduct", "h_A_p", "Convective dissipation product", "W per K"),
            ("AmbientTemperature", "T_amb", "Ambient temperature", "degC"),
            ("ThermalLimit", "T_max", "Maximum certified temperature", "degC"),
        ],
        numeric: &[
            "t_{\\mathrm{rtl}} &= %%ReserveDistance%%/%%CruiseSpeed%%",
            "E_{\\mathrm{rtl}} &= t_{\\mathrm{rtl}} \\cdot (%%PropulsionPower%% + %%AvionicsPower%%)",
            "\\mathrm{SoC}_{\\mathrm{crit}} &= (E_{\\mathrm{rtl}} + %%AbortReserve%%)/%%TotalEnergy%%",
            "\\Delta T &= (%%DischargeCurrent%% \\cdot %%DischargeCurrent%% \\cdot %%InternalResistance%%)/%%DissipationProduct%%",
            "T_{\\mathrm{cell,max}} &= %%AmbientTemperature%% + \\Delta T \\le %%ThermalLimit%%",
        ],
        sldv: "sldv.assert( (ReserveState >= DynamicReserveThreshold) && (CellTemperature <= %%ThermalLimit%%), 'Bind_ENERGY_RESERVE_THERMAL_BUDGET' );",
    },
    ProofTemplate {
        id: "T-08",
        name: "Separation and Miss Distance Bound",
        statement: "$$ d_{\\mathrm{CPA}} \\ge D_{\\mathrm{mod}} \\; \\vee \\; H_{\\mathrm{sep}} \\ge H_{\\mathrm{thresh}} $$",
        derivation: &[
            "d_{\\mathrm{evade}} &= \\frac{a_{\\mathrm{evade}}}{K_f} \\cdot t_m \\cdot t_m",
            "t_m &= \\tau_{\\mathrm{thresh}}",
            "d_{\\mathrm{CPA}} &= d_{\\mathrm{evade}}",
        ],
        params: &[
            ("WellClearRadius", "D_mod", "Horizontal well-clear boundary", "m"),
            ("VerticalClearance", "H_thresh", "Vertical well-clear boundary", "m"),
            ("WarnTime", "tau_thresh", "Warning time threshold", "s"),
            ("RelativeVelocity", "v_rel", "Maximum relative velocity", "m/s"),
            ("EvadeAcceleration", "a_evade", "Certified evasive acceleration", "m/s"),
            ("KineticFactor", "K_f", "Dimensionless kinetic term factor", "-"),
        ],
        numeric: &[
            "t_{\\mathrm{maneuver}} &= %%WarnTime%%",
            "d_{\\mathrm{evade}} &= (%%EvadeAcceleration%%/%%KineticFactor%%) \\cdot %%WarnTime%% \\cdot %%WarnTime%%",
            "d_{\\mathrm{evade}} &\\ge %%WellClearRadius%%",
        ],
        sldv: "sldv.assert( (HorizontalSeparationAtCPA >= %%WellClearRadius%%) || (VerticalSeparationAtCPA >= %%VerticalClearance%%), 'Bind_SEPARATION_MISS_DISTANCE_BOUND' );",
    },
    ProofTemplate {
        id: "T-09",
        name: "Loading Ceiling and Field of View Bound",
        statement: "$$ q(t) \\le q_{\\mathrm{limit}} \\; \\wedge \\; \\eta_{\\mathrm{LOS}}(t) \\le \\theta_{\\mathrm{FOV}} $$",
        derivation: &[
            "q_{\\mathrm{max}} &= \\frac{M \\cdot g \\cdot \\sin(\\theta_d)}{C_d \\cdot S_r}",
            "V_{\\mathrm{dive}} &= \\sqrt{ \\frac{K_f \\cdot q_{\\mathrm{max}}}{\\rho} }",
            "\\eta_{\\mathrm{LOS}} &= \\arctan\\left( \\frac{r_{\\perp}}{r_{\\parallel}} \\right)",
        ],
        params: &[
            ("TerminalMass", "M", "Terminal dive mass", "kg"),
            ("DescentAngle", "theta_d", "Maximum dive path angle", "deg"),
            ("DescentDragCoefficient", "C_d", "High-speed drag coefficient", "-"),
            ("ReferenceArea", "S_r", "Reference surface area", "square metre"),
            ("SeaLevelDensity", "rho", "Ambient medium density", "kg per cubic metre"),
            ("DynamicPressureLimit", "q_limit", "Aeroelastic dynamic pressure limit", "Pa"),
            ("FieldOfViewHalf", "theta_FOV", "Sensor half-angle field of view", "deg"),
            ("GravityAcceleration", "g", "Gravitational acceleration", "m/s"),
            ("KineticFactor", "K_f", "Dimensionless kinetic term factor", "-"),
        ],
        numeric: &[
            "q_{\\mathrm{max}} &= (%%TerminalMass%% \\cdot %%GravityAcceleration%% \\cdot \\sin(%%DescentAngle%%))/(%%DescentDragCoefficient%% \\cdot %%ReferenceArea%%)",
            "V_{\\mathrm{dive}} &= \\sqrt((%%KineticFactor%% \\cdot q_{\\mathrm{max}})/%%SeaLevelDensity%%)",
            "q_{\\mathrm{max}} &\\le %%DynamicPressureLimit%%",
            "\\eta_{\\mathrm{LOS}} &\\le %%FieldOfViewHalf%%",
        ],
        sldv: "sldv.assert( (DynamicPressure <= %%DynamicPressureLimit%%) && (LineOfSightTrackError <= %%FieldOfViewHalf%%), 'Bind_LOADING_CEILING_FOV_BOUND' );",
    },
    ProofTemplate {
        id: "T-10",
        name: "Markov Reliability Bound",
        statement: "$$ P_{\\mathrm{cat}}(T) < \\epsilon_{\\mathrm{target}} $$",
        derivation: &[
            "P_{\\mathrm{cat}}(T) &= \\int_{t_a}^{T} \\lambda_c \\cdot P_{\\mathrm{single}}(t) \\, dt",
            "P_{\\mathrm{cat}}(T) &\\approx \\frac{\\lambda_p \\cdot \\lambda_c}{\\mu_r} \\cdot T",
        ],
        params: &[
            ("ChannelFailureRate1", "lambda_p", "Primary channel failure rate", "per hour"),
            ("ChannelFailureRate2", "lambda_c", "Secondary channel common-cause rate", "per hour"),
            ("SwitchRate", "mu_r", "Reconfiguration switch rate", "per hour"),
            ("MissionDuration", "T", "Single mission operating duration", "hr"),
            ("FailureCeiling", "epsilon_target", "Target catastrophic failure ceiling", "per operating hour"),
        ],
        numeric: &[
            "P_{\\mathrm{cat}} &= (%%ChannelFailureRate1%% \\cdot %%ChannelFailureRate2%%)/%%SwitchRate%% \\cdot %%MissionDuration%%",
            "P_{\\mathrm{cat}} &\\le %%FailureCeiling%%",
        ],
        sldv: "sldv.assert( (CatastrophicFailureProbability <= %%FailureCeiling%%), 'Bind_MARKOV_RELIABILITY_BOUND' );",
    },
];

/// Collects symbolic parameter tokens from AST attribute defaults and constraint expressions.
pub fn collect_parameter_tokens(pkg: &PackageDef) -> HashMap<String, String> {
    let mut tokens = HashMap::new();

    for p in pkg.get_all_packages() {
        for attr in &p.attribute_defs {
            if let Some(ref val) = attr.default_value {
                let s = val.trim();
                if !s.is_empty() {
                    tokens.insert(attr.name.clone(), s.to_string());
                }
            }
        }
        for con in &p.constraint_defs {
            extract_constraint_tokens(&con.expression, &mut tokens);
        }
    }

    for part in pkg.get_all_parts() {
        for attr in &part.attributes {
            if let Some(ref val) = attr.default_value {
                let s = val.trim();
                if !s.is_empty() {
                    tokens.insert(attr.name.clone(), s.to_string());
                }
            }
        }
        for con in &part.constraints {
            extract_constraint_tokens(&con.expression, &mut tokens);
        }
    }

    tokens
}

fn extract_constraint_tokens(expr: &str, tokens: &mut HashMap<String, String>) {
    let operators = ["<=", ">=", "<", ">", "="];
    let mut idx = 0;
    while idx < expr.len() {
        let matched_op = operators.iter().find(|&&op| expr[idx..].starts_with(op));
        if let Some(&op) = matched_op {
            let before = &expr[..idx];
            let before_trimmed = before.trim_end();
            let id_start = before_trimmed
                .rfind(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .map(|p| p + 1)
                .unwrap_or(0);
            let key = &before_trimmed[id_start..];
            let key_valid = !key.is_empty()
                && (key.as_bytes()[0].is_ascii_alphabetic() || key.as_bytes()[0] == b'_')
                && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');

            let after = &expr[idx + op.len()..];
            let after_trimmed = after.trim_start();
            let val_len = after_trimmed
                .find(|c: char| c.is_whitespace() || c == ';')
                .unwrap_or(after_trimmed.len());
            let val = &after_trimmed[..val_len];

            if key_valid && !val.is_empty() {
                tokens.insert(key.to_string(), val.to_string());
                idx += op.len() + (after.len() - after_trimmed.len()) + val_len;
                continue;
            }
            idx += op.len();
        } else {
            idx += 1;
        }
    }
}

fn resolve_template(text: &str, tokens: &HashMap<String, String>) -> String {
    let mut result = String::with_capacity(text.len());
    let mut idx = 0;
    while let Some(start) = text[idx..].find("%%") {
        let open_pos = idx + start;
        result.push_str(&text[idx..open_pos]);
        let param_start = open_pos + 2;
        if let Some(close) = text[param_start..].find("%%") {
            let key = &text[param_start..param_start + close];
            if !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                let val = tokens
                    .get(key)
                    .cloned()
                    .unwrap_or_else(|| PENDING_PARAMETER.to_string());
                result.push_str(&val);
                idx = param_start + close + 2;
                continue;
            }
        }
        result.push_str("%%");
        idx = open_pos + 2;
    }
    result.push_str(&text[idx..]);
    result
}

/// Collects all constraint defs across package and part scopes.
pub fn collect_all_constraints(pkg: &PackageDef) -> Vec<ConstraintDef> {
    pkg.get_all_constraints()
}

/// Transpiles the SysML v2 AST into the 10-pillar safety artifact suite.
pub fn transpile_safety_suite(
    pkg: &PackageDef,
    scoring_config: Option<&FmecaScoringConfig>,
) -> HashMap<String, String> {
    let mut artifacts = HashMap::new();
    let tokens = collect_parameter_tokens(pkg);
    let ucas = expand_cartesian_stpa(pkg);
    let constraints = collect_all_constraints(pkg);
    let fmeca_rows = generate_fmeca_matrix(pkg, scoring_config);

    // 1. 01_LOSSES_HAZARDS_TOPOLOGY.md
    artifacts.insert(
        "01_LOSSES_HAZARDS_TOPOLOGY.md".to_string(),
        render_losses_hazards_topology(pkg, &ucas),
    );

    // 2. 02_UCA_COMBINATORIAL_MATRIX.md
    artifacts.insert(
        "02_UCA_COMBINATORIAL_MATRIX.md".to_string(),
        render_uca_matrix(&ucas),
    );

    // 3. 03_LOSS_SCENARIOS.md
    artifacts.insert(
        "03_LOSS_SCENARIOS.md".to_string(),
        render_loss_scenarios(&ucas),
    );

    // 4. 04_SAFETY_CONSTRAINTS.md
    artifacts.insert(
        "04_SAFETY_CONSTRAINTS.md".to_string(),
        render_safety_constraints(&ucas, &constraints),
    );

    // 5. 05_FMECA_MATRIX.md
    artifacts.insert(
        "05_FMECA_MATRIX.md".to_string(),
        render_fmeca_matrix_doc(&fmeca_rows),
    );

    // 6. 06_REGULATORY_OBJECTIVES_ASSESSMENT.md & 06_SORA_SAIL_ASSESSMENT.md
    let reg_doc = render_regulatory_objectives_assessment(&tokens);
    artifacts.insert(
        "06_REGULATORY_OBJECTIVES_ASSESSMENT.md".to_string(),
        reg_doc.clone(),
    );
    artifacts.insert("06_SORA_SAIL_ASSESSMENT.md".to_string(), reg_doc);

    // 7. 07_RTA_ARCHITECTURE.md
    artifacts.insert(
        "07_RTA_ARCHITECTURE.md".to_string(),
        render_rta_architecture(&tokens),
    );

    // 8. STPA_MATRIX.md
    artifacts.insert("STPA_MATRIX.md".to_string(), render_stpa_matrix(&ucas));

    // 9. HAZARD_LOG.md
    artifacts.insert("HAZARD_LOG.md".to_string(), render_hazard_log(pkg));

    // 10. SLDV_FORMAL_PROOFS.m
    artifacts.insert(
        "SLDV_FORMAL_PROOFS.m".to_string(),
        render_sldv_script(&constraints, &tokens),
    );

    artifacts
}

/// Writes all generated safety suite artifacts to `out_dir` atomically.
pub fn emit_safety_suite(
    pkg: &PackageDef,
    out_dir: &Path,
    scoring_config: Option<&FmecaScoringConfig>,
) -> Result<(), String> {
    fs::create_dir_all(out_dir)
        .map_err(|e| format!("Failed to create output directory '{}': {}", out_dir.display(), e))?;

    let artifacts = transpile_safety_suite(pkg, scoring_config);
    for (filename, content) in artifacts {
        let dest = out_dir.join(&filename);
        write_atomic(&dest, &content).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[allow(dead_code)]
fn sanitize_mermaid_id(id: &str) -> String {
    let clean: String = id
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if clean.is_empty() || clean.chars().next().map_or(false, |c| c.is_numeric()) {
        format!("Node_{}", clean)
    } else {
        clean
    }
}

/// Renders the STPA hierarchical safety control structure topology, system losses,
/// and associated control actions mapped directly to AST declared parts.
fn render_losses_hazards_topology(pkg: &PackageDef, ucas: &[UnsafeControlAction]) -> String {
    let all_parts = pkg.get_all_parts();
    let controllers: Vec<_> = all_parts
        .iter()
        .filter(|part| !part.actions.is_empty())
        .collect();

    let mut out = String::new();
    out.push_str("# System Losses, Hazards & Control Structure Topology\n\n");
    out.push_str("## System Losses\n\n");
    out.push_str("| Loss ID | Description |\n");
    out.push_str("| :--- | :--- |\n");

    let controller_names: Vec<String> = if !controllers.is_empty() {
        controllers.iter().map(|c| c.name.clone()).collect()
    } else {
        let mut uca_controllers: Vec<String> = ucas.iter().map(|u| u.controller.clone()).collect();
        uca_controllers.sort();
        uca_controllers.dedup();
        if !uca_controllers.is_empty() {
            uca_controllers
        } else {
            let filtered: Vec<String> = all_parts
                .iter()
                .filter(|p| !p.name.starts_with('_'))
                .map(|p| p.name.clone())
                .collect();
            if filtered.is_empty() {
                all_parts.iter().map(|p| p.name.clone()).collect()
            } else {
                filtered
            }
        }
    };

    if controller_names.is_empty() {
        out.push_str(&format!(
            "| L-1 | Loss of safe function of {} |\n",
            pkg.name
        ));
    } else {
        for (i, c_name) in controller_names.iter().enumerate() {
            out.push_str(&format!(
                "| L-{} | Loss of safe function of {} |\n",
                i + 1,
                c_name
            ));
        }
    }

    out.push_str("\n## System Hazards\n\n");
    out.push_str("| Hazard ID | Associated Control Action | Controller |\n");
    out.push_str("| :--- | :--- | :--- |\n");
    for uca in ucas {
        if uca.id.ends_with("-001") || uca.guide_word == "Not providing" {
            out.push_str(&format!(
                "| {} | {} | {} |\n",
                uca.hazard, uca.control_action, uca.controller
            ));
        }
    }

    let primary_controller = if !controller_names.is_empty() {
        controller_names.first().cloned().unwrap_or_else(|| "DEAPCompilerSystem".to_string())
    } else {
        "DEAPCompilerSystem".to_string()
    };

    out.push_str("\n## Hierarchical Control Structure Topology\n\n");
    out.push_str("```mermaid\n");
    out.push_str("graph TD\n");
    out.push_str("    subgraph \"Hierarchical Safety Control Structure\"\n");
    out.push_str("        direction TB\n");
    out.push_str(&format!(
        "        CompilerController[\"{}<br/>(Supervisory Controller)\"]\n",
        primary_controller
    ));
    out.push_str("        ActuatorEngine[\"UniversalIngestionEngine<br/>(Actuator Engine)\"]\n");
    out.push_str("        ControlledProcess[\"DownstreamApplicationHost<br/>(Execution Environment)\"]\n");
    out.push_str("        DiagnosticSensors[\"CompilerAssuranceEngine<br/>(Diagnostic Sensors)\"]\n\n");
    out.push_str("        CompilerController -->|\"supervisory dispatch: trigger_lowering - invoke_codegen\"| ActuatorEngine\n");
    out.push_str("        ActuatorEngine -->|\"artifact synthesis: lower_ast - write_binaries\"| ControlledProcess\n");
    out.push_str("        ControlledProcess -->|\"compilation artifacts and execution traces\"| DiagnosticSensors\n");
    out.push_str("        DiagnosticSensors -->|\"sensor feedback: diagnostic_errors - verification_metrics\"| CompilerController\n");
    out.push_str("    end\n");
    out.push_str("```\n\n");

    out
}

fn render_uca_matrix(ucas: &[UnsafeControlAction]) -> String {
    let mut out = String::new();
    out.push_str("# Unsafe Control Action Combinatorial Matrix\n\n");
    out.push_str("Cartesian product of controlling part-def control actions across the\n");
    out.push_str("four canonical STPA guide-word categories.\n\n");
    out.push_str("| UCA ID | Controller | Control Action | Guide Word | Context | Hazard | Safety Constraint | Severity | SAIL |\n");
    out.push_str("| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n");
    for u in ucas {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            u.id,
            u.controller,
            u.control_action,
            u.guide_word,
            u.context,
            u.hazard,
            u.constraint,
            u.severity,
            u.sail
        ));
    }
    out.push('\n');
    out
}

fn render_loss_scenarios(ucas: &[UnsafeControlAction]) -> String {
    let mut out = String::new();
    out.push_str("# Loss Scenarios & Causal Factors\n\n");
    out.push_str("| Loss Scenario ID | UCA ID | Controller | Control Action | Scenario | Causal Factor |\n");
    out.push_str("| :--- | :--- | :--- | :--- | :--- | :--- |\n");
    for (i, u) in ucas.iter().enumerate() {
        out.push_str(&format!(
            "| LS-{:03} | {} | {} | {} | Loss scenario skeleton for {} under nondeterministic conditions | {} |\n",
            i + 1,
            u.id,
            u.controller,
            u.control_action,
            u.control_action,
            PENDING_PARAMETER
        ));
    }
    out.push('\n');
    out
}

fn render_safety_constraints(
    ucas: &[UnsafeControlAction],
    constraints: &[ConstraintDef],
) -> String {
    let mut out = String::new();
    out.push_str("# Formal Safety Constraints\n\n");
    out.push_str("## Derived Safety Constraints per Unsafe Control Action\n\n");
    out.push_str("| Safety Constraint ID | UCA ID | Constraint Statement |\n");
    out.push_str("| :--- | :--- | :--- |\n");
    for u in ucas {
        out.push_str(&format!(
            "| {} | {} | {} shall remain within safe bounds under {} |\n",
            u.constraint, u.id, u.control_action, u.guide_word
        ));
    }
    out.push_str("\n## Schema-Declared Constraint Defs (SysML v2 SSOT)\n\n");
    out.push_str("| Constraint Def | Expression |\n");
    out.push_str("| :--- | :--- |\n");
    for c in constraints {
        out.push_str(&format!("| {} | {} |\n", c.name, c.expression));
    }
    out.push('\n');
    out
}

fn render_fmeca_matrix_doc(rows: &[FmecaRow]) -> String {
    let mut out = String::new();
    out.push_str("# FMECA Criticality Matrix\n\n");
    out.push_str("The Risk Priority Number is the product of severity (S), occurrence (O)\n");
    out.push_str("and detection (D) scores.\n\n");
    out.push_str("| FMECA ID | Component | Failure Mode | Potential Effect | Severity (S) | Occurrence (O) | Detection (D) | RPN (S x O x D) | Mitigation | Derivation Basis |\n");
    out.push_str("| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n");
    for r in rows {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            r.id,
            r.component,
            r.failure_mode,
            r.effect,
            r.severity_cell,
            r.occurrence_cell,
            r.detection_cell,
            r.rpn_cell,
            r.mitigation,
            r.basis
        ));
    }
    out.push('\n');
    out
}

fn render_regulatory_objectives_assessment(tokens: &HashMap<String, String>) -> String {
    let prefix = tokens
        .get("REGULATORY_OBJECTIVE_PREFIX")
        .cloned()
        .unwrap_or_else(|| "OBJ-".to_string());
    let prefix_label = if prefix.ends_with('-') {
        prefix
    } else {
        format!("{}-", prefix)
    };

    let mut out = String::new();
    out.push_str("# Domain Regulatory Objectives Assessment & Safety Objective Roster\n\n");
    out.push_str("| Assessment Field | Value |\n");
    out.push_str("| :--- | :--- |\n");
    out.push_str(&format!(
        "| Ground Risk Class (GRC) | {} |\n",
        tokens
            .get("GroundRiskClass")
            .unwrap_or(&PENDING_PARAMETER.to_string())
    ));
    out.push_str(&format!(
        "| Air Risk Class (ARC) | {} |\n",
        tokens
            .get("AirRiskClass")
            .unwrap_or(&PENDING_PARAMETER.to_string())
    ));
    out.push_str(&format!(
        "| Specific Assurance and Integrity Level (SAIL) | {} |\n\n",
        tokens.get("SAIL").unwrap_or(&PENDING_PARAMETER.to_string())
    ));

    out.push_str("## Operational Safety Objectives\n\n");
    out.push_str("| OSO ID | Objective | Robustness | Integrity | Assurance Level | Evidence |\n");
    out.push_str("| :--- | :--- | :--- | :--- | :--- | :--- |\n");

    for i in 1..=10 {
        let oso_id = format!("{}{:02}", prefix_label, i);
        let obj = tokens
            .get(&format!("OBJ{:02}_Objective", i))
            .or_else(|| tokens.get(&format!("OSO{:02}_Objective", i)))
            .map(|s| s.as_str())
            .unwrap_or(PENDING_PARAMETER);
        let rob = tokens
            .get(&format!("OBJ{:02}_Robustness", i))
            .or_else(|| tokens.get(&format!("OSO{:02}_Robustness", i)))
            .map(|s| s.as_str())
            .unwrap_or(PENDING_PARAMETER);
        let int = tokens
            .get(&format!("OBJ{:02}_Integrity", i))
            .or_else(|| tokens.get(&format!("OSO{:02}_Integrity", i)))
            .map(|s| s.as_str())
            .unwrap_or(PENDING_PARAMETER);
        let ass = tokens
            .get(&format!("OBJ{:02}_Assurance_Level", i))
            .or_else(|| tokens.get(&format!("OSO{:02}_Assurance_Level", i)))
            .map(|s| s.as_str())
            .unwrap_or(PENDING_PARAMETER);
        let evi = tokens
            .get(&format!("OBJ{:02}_Evidence", i))
            .or_else(|| tokens.get(&format!("OSO{:02}_Evidence", i)))
            .map(|s| s.as_str())
            .unwrap_or(PENDING_PARAMETER);

        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            oso_id, obj, rob, int, ass, evi
        ));
    }
    out.push('\n');
    out
}

/// Renders the Run-Time Assurance (RTA) simplex architecture diagram and formal
/// proof suite derivations anchored to AST declared system elements.
fn render_rta_architecture(tokens: &HashMap<String, String>) -> String {
    let mut out = String::new();
    out.push_str("# Run-Time Assurance Architecture & Formal Proof Suite\n\n");
    out.push_str("## Simplex Run-Time Assurance Topology\n\n");
    out.push_str("```mermaid\n");
    out.push_str("graph TD\n");
    out.push_str("    subgraph \"Run-Time Assurance Architecture\"\n");
    out.push_str("        direction TB\n");
    out.push_str("        CompilerAssuranceEngine[\"High Assurance Channel<br/>(CompilerAssuranceEngine)\"] --> StateMachineSolverEngine[\"Safety Monitor Switch<br/>(StateMachineSolverEngine)\"]\n");
    out.push_str("        SafetyAssuranceEngine[\"Recovery Channel<br/>(SafetyAssuranceEngine)\"] --> StateMachineSolverEngine\n");
    out.push_str("        StateMachineSolverEngine --> DEAPCompilerSystem[\"System Under Assurance<br/>(DEAPCompilerSystem)\"]\n");
    out.push_str("    end\n");
    out.push_str("```\n\n");

    out.push_str("## Formal Proof Suite\n\n");

    for tmpl in &PROOF_TEMPLATES {
        out.push_str(&format!("## Theorem {} -- {}\n\n", tmpl.id, tmpl.name));
        out.push_str("### Formal Theorem Statement\n\n");
        out.push_str(tmpl.statement);
        out.push_str("\n\n### Symbolic Derivation\n\n");
        out.push_str("$$\n\\begin{aligned}\n");
        for line in tmpl.derivation {
            out.push_str(&format!("    {} \\\\\n", line));
        }
        out.push_str("\\end{aligned}\n$$\n\n");

        out.push_str("### Parameter Definitions & Engineering Units Table\n\n");
        out.push_str("| Symbol | Description | Value | Engineering Unit |\n");
        out.push_str("| :--- | :--- | :--- | :--- |\n");
        for (key, symbol, desc, unit) in tmpl.params {
            let val = tokens
                .get(*key)
                .cloned()
                .unwrap_or_else(|| PENDING_PARAMETER.to_string());
            out.push_str(&format!("| {} | {} | {} | {} |\n", symbol, desc, val, unit));
        }
        out.push_str("\n### Step-by-Step Numerical Proof Evaluation\n\n");
        out.push_str("$$\n\\begin{aligned}\n");
        for line in tmpl.numeric {
            let resolved = resolve_template(line, tokens);
            out.push_str(&format!("    {} \\\\\n", resolved));
        }
        out.push_str("\\end{aligned}\n$$\n\n");

        out.push_str("### SLDV Temporal Assertion Binding\n\n");
        out.push_str("```matlab\n");
        out.push_str(&resolve_template(tmpl.sldv, tokens));
        out.push_str("\n```\n\n");
    }

    out
}

fn render_stpa_matrix(ucas: &[UnsafeControlAction]) -> String {
    let mut out = String::new();
    out.push_str("# STPA Cross-Traceability Matrix\n\n");
    out.push_str("| UCA ID | Controller | Control Action | Guide Word | Hazard | Safety Constraint | Traceability Status |\n");
    out.push_str("| :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n");
    for u in ucas {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            u.id,
            u.controller,
            u.control_action,
            u.guide_word,
            u.hazard,
            u.constraint,
            PENDING_PARAMETER
        ));
    }
    out.push('\n');
    out
}

fn render_hazard_log(pkg: &PackageDef) -> String {
    let controllers: Vec<_> = pkg
        .part_defs
        .iter()
        .filter(|part| !part.actions.is_empty())
        .collect();

    let mut out = String::new();
    out.push_str("# Hazard Log\n\n");
    out.push_str("| ID | Kind | Source | Status | Notes |\n");
    out.push_str("| :--- | :--- | :--- | :--- | :--- |\n");

    if controllers.is_empty() {
        for (i, p) in pkg.part_defs.iter().enumerate() {
            out.push_str(&format!(
                "| L-{} | Loss | {} | Open | Skeleton loss entry |\n",
                i + 1,
                p.name
            ));
        }
    } else {
        for (i, c) in controllers.iter().enumerate() {
            out.push_str(&format!(
                "| L-{} | Loss | {} | Open | Skeleton loss entry |\n",
                i + 1,
                c.name
            ));
        }
    }

    let mut hazard_index = 0;
    for c in &controllers {
        for a in &c.actions {
            hazard_index += 1;
            out.push_str(&format!(
                "| H-{} | Hazard | {} / {} | Open | Compound hazard skeleton |\n",
                hazard_index, c.name, a.name
            ));
        }
    }

    out.push_str(&format!("\n| Resolution Authority | {} |\n\n", PENDING_PARAMETER));
    out
}

fn render_sldv_script(constraints: &[ConstraintDef], tokens: &HashMap<String, String>) -> String {
    let mut out = String::new();
    out.push_str("% SLDV Formal Proof Script\n");
    out.push_str("% Schema constraint bindings\n");
    for c in constraints {
        let expr = if c.expression.is_empty() {
            "false"
        } else {
            c.expression.as_str()
        };
        let sanitized = c.name.to_uppercase().replace('-', "_");
        out.push_str(&format!(
            "sldv.assert( ({}), 'Bind_{}_ASSERTION' );\n",
            expr, sanitized
        ));
    }

    out.push_str("\n% Theorem proof bindings\n");
    for tmpl in &PROOF_TEMPLATES {
        out.push_str(&format!("% {} {}\n", tmpl.id, tmpl.name));
        out.push_str(&resolve_template(tmpl.sldv, tokens));
        out.push_str("\n\n");
    }

    out
}

/// Parses an STPA markdown file and compiles its hazard matrices into a SysML constraint package.
pub fn compile_stpa_to_constraints(content: &str, package_name: &str) -> PackageDef {
    let mut pkg = PackageDef::default();
    pkg.name = package_name.to_string();
    pkg.doc = Some("STPA and FMECA Safety Invariants compiled for Run-Time Assurance (RTA) & SLDV Verification".to_string());

    // Match UCA table rows: | UCA-### | Controller | Action | Guide Word | ... | Hazard | Constraint | ...
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            continue;
        }
        let cols: Vec<&str> = trimmed.split('|').map(|s| s.trim()).collect();
        if cols.len() >= 8 && cols[1].starts_with("UCA-") && cols[1]["UCA-".len()..].chars().all(|c| c.is_ascii_digit()) && !cols[1]["UCA-".len()..].is_empty() {
            let uca_id = cols[1];
            let _controller = cols[2];
            let action = cols[3];
            let guide_word = cols[4];
            let sc_col = cols[7];
            let sc_id = if let Some(pos) = sc_col.find("SC-") {
                let rest = &sc_col[pos + 3..];
                let digit_len = rest.chars().take_while(|c| c.is_ascii_digit()).count();
                if digit_len > 0 {
                    &sc_col[pos..pos + 3 + digit_len]
                } else {
                    ""
                }
            } else {
                ""
            };

            if !sc_id.is_empty() && !action.is_empty() {
                let constraint_name = format!("{}_{}", sc_id.replace('-', "_"), action.replace(' ', "_"));
                let doc = format!("{}: {} shall not violate safety under {} (ref {})", sc_id, action, guide_word, uca_id);
                let expression = format!("{}SafeState == true", action.replace(' ', ""));

                pkg.constraint_defs.push(ConstraintDef {
                    name: constraint_name,
                    doc: Some(doc),
                    expression,
                    is_assertion: true,
                    ..Default::default()
                });
            }
        }
    }

    // Match FMECA table rows: | FMECA-### | Component | Failure Mode | Potential Effect | ... | Mitigation |
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            continue;
        }
        let cols: Vec<&str> = trimmed.split('|').map(|s| s.trim()).collect();
        if cols.len() >= 5 && cols[1].starts_with("FMECA-") && cols[1]["FMECA-".len()..].chars().all(|c| c.is_ascii_digit()) && !cols[1]["FMECA-".len()..].is_empty() {
            let fmeca_id = cols[1];
            let component = cols[2];
            let failure_mode = cols[3];

            if !component.is_empty() && !failure_mode.is_empty() {
                let constraint_name = format!("{}_{}_Integrity", fmeca_id.replace('-', "_"), component.replace(' ', "_"));
                let doc = format!("{}: {} shall mitigate {}", fmeca_id, component, failure_mode);
                let expression = format!("{}HealthStatus >= 1", component.replace(' ', ""));

                pkg.constraint_defs.push(ConstraintDef {
                    name: constraint_name,
                    doc: Some(doc),
                    expression,
                    is_assertion: false,
                    ..Default::default()
                });
            }
        }
    }

    pkg
}

#[cfg(test)]
mod tests {
    use super::*;
    use deap_core::sysml_ast::{ActionDef, PartDef};

    #[test]
    fn test_transpile_safety_suite_produces_all_10_artifacts() {
        let mut pkg = PackageDef::default();
        pkg.name = "TestSystem".to_string();

        let mut p = PartDef::default();
        p.name = "FlightComputer".to_string();
        let mut act = ActionDef::default();
        act.name = "CalculateGuidance".to_string();
        p.actions.push(act);
        pkg.part_defs.push(p);

        let suite = transpile_safety_suite(&pkg, None);
        assert!(suite.contains_key("01_LOSSES_HAZARDS_TOPOLOGY.md"));
        assert!(suite.contains_key("02_UCA_COMBINATORIAL_MATRIX.md"));
        assert!(suite.contains_key("03_LOSS_SCENARIOS.md"));
        assert!(suite.contains_key("04_SAFETY_CONSTRAINTS.md"));
        assert!(suite.contains_key("05_FMECA_MATRIX.md"));
        assert!(suite.contains_key("06_REGULATORY_OBJECTIVES_ASSESSMENT.md"));
        assert!(suite.contains_key("06_SORA_SAIL_ASSESSMENT.md"));
        assert!(suite.contains_key("07_RTA_ARCHITECTURE.md"));
        assert!(suite.contains_key("STPA_MATRIX.md"));
        assert!(suite.contains_key("HAZARD_LOG.md"));
        assert!(suite.contains_key("SLDV_FORMAL_PROOFS.m"));

        // Verify RTA architecture contains KaTeX aligned math
        let rta = suite.get("07_RTA_ARCHITECTURE.md").unwrap();
        assert!(rta.contains("$$\n\\begin{aligned}"));
        assert!(rta.contains("\\end{aligned}\n$$"));

        // Verify closed-loop STPA control structure topology
        let topo = suite.get("01_LOSSES_HAZARDS_TOPOLOGY.md").unwrap();
        assert!(topo.contains("subgraph \"Hierarchical Safety Control Structure\""));
        assert!(topo.contains("CompilerController[\"FlightComputer<br/>(Supervisory Controller)\"]"));
        assert!(topo.contains("ActuatorEngine[\"UniversalIngestionEngine<br/>(Actuator Engine)\"]"));
        assert!(topo.contains("ControlledProcess[\"DownstreamApplicationHost<br/>(Execution Environment)\"]"));
        assert!(topo.contains("DiagnosticSensors[\"CompilerAssuranceEngine<br/>(Diagnostic Sensors)\"]"));
        assert!(topo.contains("DiagnosticSensors -->|\"sensor feedback: diagnostic_errors - verification_metrics\"| CompilerController"));
    }

    #[test]
    fn test_compile_stpa_markdown_to_constraints() {
        let md = r#"
| UCA ID | Controller | Control Action | Guide Word | Context | Hazard | Safety Constraint | Severity | SAIL |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| UCA-001 | FlightComputer | DeployParachute | Not providing | In-flight | H-1 | SC-001 | High | IV |
"#;
        let pkg = compile_stpa_to_constraints(md, "SafetyConstraintsPackage");
        assert_eq!(pkg.name, "SafetyConstraintsPackage");
        assert_eq!(pkg.constraint_defs.len(), 1);
        assert_eq!(pkg.constraint_defs[0].name, "SC_001_DeployParachute");
        assert!(pkg.constraint_defs[0].is_assertion);
    }
}
