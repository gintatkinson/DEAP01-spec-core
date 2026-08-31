package DEAP_Safety_Architecture {

    // =========================================================================
    // 1. Certification Requirements Package
    // Formal requirements for DO-178C (DAL A–E), DO-254, ARP4754A/4761,
    // and JARUS SORA v2.5 (SAIL I–VI)
    // =========================================================================
    package Certification_Requirements {

        // --- DO-178C Software Considerations ---
        requirement def DO178C_DAL_A {
            doc /* Software Level A: Catastrophic failure condition prevention. Requires 100% MC/DC coverage and verified zero heap allocation. */
            attribute level : String = "DAL A";
            attribute mcdc_coverage_required : Real = 100.0;
        }

        requirement def DO178C_DAL_B {
            doc /* Software Level B: Hazardous failure condition mitigation. Requires 100% Decision/State coverage. */
            attribute level : String = "DAL B";
            attribute decision_coverage_required : Real = 100.0;
        }

        requirement def DO178C_DAL_C {
            doc /* Software Level C: Major failure condition mitigation. Requires 100% Statement coverage. */
            attribute level : String = "DAL C";
            attribute statement_coverage_required : Real = 100.0;
        }

        requirement def DO178C_DAL_D {
            doc /* Software Level D: Minor failure condition mitigation. */
            attribute level : String = "DAL D";
        }

        requirement def DO178C_DAL_E {
            doc /* Software Level E: No safety effect. */
            attribute level : String = "DAL E";
        }

        // --- DO-254 Airborne Electronic Hardware ---
        requirement def DO254_DAL_A_Hardware {
            doc /* Hardware Design Assurance Level A for Flight Control Computers and Actuator ICs. */
            attribute target_device : String = "FPGA / ASIC";
            attribute single_event_upset_mitigated : Boolean = true;
        }

        // --- ARP4754A / ARP4761 System Safety Assessment ---
        requirement def ARP4754A_SafetyAssessment {
            doc /* Functional Hazard Assessment (FHA), Preliminary System Safety Assessment (PSSA), and SSA validation. */
            attribute max_allowable_catastrophic_probability : Real = 1.0e-9;
        }

        requirement def ARP4761_FMEA_PSSA {
            doc /* Failure Modes and Effects Analysis coupled with Fault Tree Analysis (FTA). */
            attribute quantitative_fta_required : Boolean = true;
        }

        // --- JARUS SORA v2.5 SAIL Requirements ---
        requirement def SORA_SAIL_I {
            doc /* Specific Assurance and Integrity Level I (Low Risk UAS Operation). */
            attribute sail_level : String = "SAIL I";
        }

        requirement def SORA_SAIL_II {
            doc /* Specific Assurance and Integrity Level II. */
            attribute sail_level : String = "SAIL II";
        }

        requirement def SORA_SAIL_III {
            doc /* Specific Assurance and Integrity Level III (Medium Risk UAS Operation). */
            attribute sail_level : String = "SAIL III";
        }

        requirement def SORA_SAIL_IV {
            doc /* Specific Assurance and Integrity Level IV (Controlled Airspace / BVLOS). */
            attribute sail_level : String = "SAIL IV";
        }

        requirement def SORA_SAIL_V {
            doc /* Specific Assurance and Integrity Level V (High Density Urban Operations). */
            attribute sail_level : String = "SAIL V";
        }

        requirement def SORA_SAIL_VI {
            doc /* Specific Assurance and Integrity Level VI (Complex Category / Commercial Airliner Proximity). */
            attribute sail_level : String = "SAIL VI";
            attribute target_integrity_level : String = "Equivalent to Manned Aviation (1e-9/hr)";
        }
    }

    // =========================================================================
    // 1b. System Losses and Hazards Package
    // Formal requirement definitions for top-down System Losses (L_1..L_4)
    // and System Hazards (H_1..H_6) per SAE ARP4761 & FAA AC 25.1309-1A
    // =========================================================================
    package System_Losses_And_Hazards {

        // --- System Losses (L_1 ... L_4) ---
        requirement def L_1 {
            doc /* Loss of Life / Aircraft Destruction: Complete hull loss and fatal injuries. */
            attribute severity : String = "Catastrophic";
            attribute target_probability_per_hr : Real = 1.0e-9;
        }

        requirement def L_2 {
            doc /* Loss of Mission / Severe Operational Failure: Total failure of flight objective, emergency divert required. */
            attribute severity : String = "Hazardous";
            attribute target_probability_per_hr : Real = 1.0e-7;
        }

        requirement def L_3 {
            doc /* Damage to Ground Infrastructure: Destruction of runway, airport structures, or ground equipment. */
            attribute severity : String = "Major";
            attribute target_probability_per_hr : Real = 1.0e-5;
        }

        requirement def L_4 {
            doc /* Loss of System Availability / Margins: Reduction in avionic functional capability or safety margins. */
            attribute severity : String = "Major";
            attribute target_probability_per_hr : Real = 1.0e-5;
        }

        // --- System Hazards (H_1 ... H_6) ---
        requirement def H_1 {
            doc /* Controlled Flight Into Terrain (CFIT): Airworthy aircraft flown into terrain under control or guidance. */
            attribute regulatory_baseline : String = "FAA AC 25.1309-1A §8.b / CS-25.1309(b)(1)";
            attribute severity : String = "Catastrophic";
        }

        requirement def H_2 {
            doc /* Loss of Control in Flight (LOC-I): Aircraft attitude, altitude, or aerodynamic state exceeds normal flight envelope. */
            attribute regulatory_baseline : String = "FAA AC 25.1309-1A §8.a / CS-25.1309(b)(1)";
            attribute severity : String = "Catastrophic";
        }

        requirement def H_3 {
            doc /* Mid-Air Collision (MAC): Loss of required horizontal or vertical separation between aircraft in controlled airspace. */
            attribute regulatory_baseline : String = "FAA AC 25.1309-1A §8.c / CS-25.1309(b)(2)";
            attribute severity : String = "Hazardous";
        }

        requirement def H_4 {
            doc /* Runway Incursion / Excursion: Aircraft departs runway surface during landing, takeoff, or ground taxi operations. */
            attribute regulatory_baseline : String = "FAA AC 25.1309-1A §8.d / CS-25.1309(b)(3)";
            attribute severity : String = "Major";
        }

        requirement def H_5 {
            doc /* Airframe Structural Overstress: Aerodynamic or structural loads exceed ultimate design limits. */
            attribute regulatory_baseline : String = "FAA AC 25.1309-1A §8.e / CS-25.1309(b)(1)";
            attribute severity : String = "Catastrophic";
        }

        requirement def H_6 {
            doc /* Uncommanded Engine Thrust Reversal: In-flight deployment of engine thrust reversers creating catastrophic asymmetric drag. */
            attribute regulatory_baseline : String = "FAA AC 25.1309-1A §8.f / CS-25.1309(b)(1)";
            attribute severity : String = "Catastrophic";
        }
    }

    // =========================================================================
    // 2. Aircraft State Space Package
    // Continuous State Vector x(t) definition
    // =========================================================================
    package Aircraft_State_Space {

        attribute def AircraftStateVector {
            attribute altitude : Real;         // Barometric/Radar Altitude (meters)
            attribute V_CAS : Real;            // Calibrated Airspeed (m/s)
            attribute alpha : Real;            // Angle of Attack (radians)
            attribute theta : Real;            // Pitch Angle (radians)
            attribute phi : Real;              // Roll Angle (radians)
            attribute r : Real;                // Yaw Rate (rad/s)
            attribute T_eng : Real;            // Engine Thrust Output (N)
            attribute WoW : Boolean;           // Weight-on-Wheels Sensor Flag
            attribute S_phase : String;        // Flight Phase (Preflight, Taxi, Takeoff, Climb, Cruise, Descent, Approach, Landing, RTH)
        }
    }

    // =========================================================================
    // 3. Avionic System Parts Package
    // Main hardware and software subsystem definitions
    // =========================================================================
    package Avionic_System_Parts {

        part def FlightControlComputer {
            doc /* Triple-modular redundant (TMR) primary flight control computer executing control loop laws. */
            attribute redundant_channels : Integer = 3;
            attribute max_loop_latency_ms : Real = 10.0;
        }

        part def AutopilotGuidanceEngine {
            doc /* Trajectory generation and guidance engine providing waypoints and inner-loop commands. */
            attribute dal_level : String = "DAL A";
        }

        part def ActuatorControlUnit {
            doc /* Control Surface and Motor ESC actuator driver unit with feedback sensing. */
            attribute refresh_rate_hz : Real = 400.0;
        }

        part def GroundControlStation {
            doc /* Telemetry command link terminal and operator interface station. */
            attribute link_frequency_mhz : Real = 2400.0;
        }

        part def DAASensorSuite {
            doc /* Detect-and-Avoid sensor payload including TCAS, ADS-B In, Radar, and Optical sensors. */
            attribute range_meters : Real = 10000.0;
        }
    }

    // =========================================================================
    // 4. Databus Port Definitions Package
    // Hardware and telemetry bus communication interfaces
    // =========================================================================
    package Databus_Port_Definitions {

        port def Arinc429Port {
            doc /* ARINC 429 high-speed avionics serial bus port (100 kbps). */
            attribute word_length_bits : Integer = 32;
            attribute parity : String = "Odd";
        }

        port def Mil1553Port {
            doc /* MIL-STD-1553B dual-redundant command/response multiplex databus port. */
            attribute bitrate_mbps : Real = 1.0;
        }

        port def C25GLinkPort {
            doc /* 3GPP 5G Command & Control link port for BVLOS telemetry and RTCA DO-365B DAA streams. */
            attribute max_latency_ms : Real = 20.0;
            attribute encrypted : Boolean = true;
        }

        port def CanBusPort {
            doc /* CAN Aerospace / UAVCAN / DroneCAN bus port for internal actuator sensor node networks. */
            attribute baudrate_kbps : Integer = 1000;
        }
    }

    // =========================================================================
    // 4b. STPA Unsafe Control Actions Package
    // Formal SysML v2 requirement definitions (UCA_01 ... UCA_16) for all 16 rows
    // of the STPA UCA matrix in DEAP_FLIGHT_SYSTEMS_SAFETY_CONCEPT_PAPER.md
    // =========================================================================
    package STPA_Unsafe_Control_Actions {

        requirement def UCA_01 {
            @TriggersHazard(H_1)
            doc /* Flight Control Computer fails to provide Pitch Nose-Up Recovery Command during low altitude speed decay. */
            attribute controller : String = "Flight Control Computer (FCC)";
            attribute control_action : String = "Pitch Nose-Up Recovery Command";
            attribute uca_type : String = "Not Provided";
            attribute context_vector : String = "h < 500 ft AGL, V_CAS < V_ref, alpha > 12.0 deg, WoW = False, S_phase = APPROACH";
            attribute triggered_hazard_id : String = "H_1";
            attribute severity : String = "Catastrophic";
            attribute dal_level : String = "DAL A";
        }

        requirement def UCA_02 {
            @TriggersHazard(H_2)
            doc /* Flight Control Computer provides Pitch High-Rate Elevator Command unsafely during high angle of attack state. */
            attribute controller : String = "Flight Control Computer (FCC)";
            attribute control_action : String = "Pitch High-Rate Elevator Command";
            attribute uca_type : String = "Provided Unsafely";
            attribute context_vector : String = "alpha > 14.5 deg, V_CAS < V_stall + 5 kts, theta > 18.0 deg, WoW = False, S_phase in {CLIMB, APPROACH}";
            attribute triggered_hazard_id : String = "H_2";
            attribute severity : String = "Catastrophic";
            attribute dal_level : String = "DAL A";
        }

        requirement def UCA_03 {
            @TriggersHazard(H_5)
            doc /* Flight Control Computer provides Roll Stabilization Aileron Command too late during severe turbulence gust. */
            attribute controller : String = "Flight Control Computer (FCC)";
            attribute control_action : String = "Roll Stabilization Aileron Command";
            attribute uca_type : String = "Provided Too Late";
            attribute context_vector : String = "Command processing latency t_latency > 50 ms, severe turbulence gust d(t), S_phase = CRUISE";
            attribute triggered_hazard_id : String = "H_5";
            attribute severity : String = "Hazardous";
            attribute dal_level : String = "DAL B";
        }

        requirement def UCA_04 {
            @TriggersHazard(H_2)
            doc /* Flight Control Computer applies Elevator Auto-Trim Torque Drive too long after manual pilot override signal. */
            attribute controller : String = "Flight Control Computer (FCC)";
            attribute control_action : String = "Elevator Auto-Trim Torque Drive";
            attribute uca_type : String = "Applied Too Long";
            attribute context_vector : String = "Continuous trim drive applied for t > 2.0 s after manual pilot override / AP disconnect signal";
            attribute triggered_hazard_id : String = "H_2";
            attribute severity : String = "Catastrophic";
            attribute dal_level : String = "DAL A";
        }

        requirement def UCA_05 {
            @TriggersHazard(H_1)
            doc /* Autopilot Mode Logic fails to provide Auto-Go-Around (GA) Mode Engage upon severe glideslope deviation. */
            attribute controller : String = "Autopilot Mode Logic";
            attribute control_action : String = "Auto-Go-Around (GA) Mode Engage";
            attribute uca_type : String = "Not Provided";
            attribute context_vector : String = "ILS glideslope deviation > 1.5 dots below decision height h < 200 ft AGL, S_phase = APPROACH";
            attribute triggered_hazard_id : String = "H_1";
            attribute severity : String = "Catastrophic";
            attribute dal_level : String = "DAL A";
        }

        requirement def UCA_06 {
            @TriggersHazard(H_1)
            doc /* Autopilot Mode Logic provides Autopilot Pitch Down Trim Command unsafely during radio altimeter lock loss. */
            attribute controller : String = "Autopilot Mode Logic";
            attribute control_action : String = "Autopilot Pitch Down Trim Command";
            attribute uca_type : String = "Provided Unsafely";
            attribute context_vector : String = "Radio altimeter sensor fault / lock loss at low altitude h < 400 ft AGL, S_phase = APPROACH";
            attribute triggered_hazard_id : String = "H_1";
            attribute severity : String = "Catastrophic";
            attribute dal_level : String = "DAL A";
        }

        requirement def UCA_07 {
            @TriggersHazard(H_3)
            doc /* Autopilot Mode Logic provides VNAV Descent Mode Transition too early prior to ATC clearance boundary. */
            attribute controller : String = "Autopilot Mode Logic";
            attribute control_action : String = "VNAV Descent Mode Transition";
            attribute uca_type : String = "Provided Too Early";
            attribute context_vector : String = "Executed 15 s prior to ATC altitude clearance boundary, h = 24,000 ft MSL, S_phase = CRUISE";
            attribute triggered_hazard_id : String = "H_3";
            attribute severity : String = "Hazardous";
            attribute dal_level : String = "DAL B";
        }

        requirement def UCA_08 {
            @TriggersHazard(H_2)
            doc /* Autopilot Mode Logic applies Nose-Up Pitch Hold Command too long after Go-Around mode disengagement. */
            attribute controller : String = "Autopilot Mode Logic";
            attribute control_action : String = "Nose-Up Pitch Hold Command";
            attribute uca_type : String = "Applied Too Long";
            attribute context_vector : String = "Pitch command maintained for t > 5.0 s after Go-Around mode disengagement, V_CAS < V_min";
            attribute triggered_hazard_id : String = "H_2";
            attribute severity : String = "Catastrophic";
            attribute dal_level : String = "DAL A";
        }

        requirement def UCA_09 {
            @TriggersHazard(H_2)
            doc /* Auto-Throttle System fails to provide Engine Thrust Increase Command during critical airspeed decay. */
            attribute controller : String = "Auto-Throttle System";
            attribute control_action : String = "Engine Thrust Increase Command";
            attribute uca_type : String = "Not Provided";
            attribute context_vector : String = "Airspeed decay V_CAS < V_stall-warning (V_CAS < 1.1 V_stall), h > 500 ft AGL, S_phase = APPROACH";
            attribute triggered_hazard_id : String = "H_2";
            attribute severity : String = "Catastrophic";
            attribute dal_level : String = "DAL A";
        }

        requirement def UCA_10 {
            @TriggersHazard(H_6)
            doc /* Auto-Throttle / Reverser provides Engine Thrust Reverser Deploy Command unsafely in-flight. */
            attribute controller : String = "Auto-Throttle / Reverser";
            attribute control_action : String = "Engine Thrust Reverser Deploy Command";
            attribute uca_type : String = "Provided Unsafely";
            attribute context_vector : String = "In-flight execution (WoW = False, h > 50 ft AGL, V_CAS = 250 kts, S_phase in {CLIMB, CRUISE})";
            attribute triggered_hazard_id : String = "H_6";
            attribute severity : String = "Catastrophic";
            attribute dal_level : String = "DAL A";
        }

        requirement def UCA_11 {
            @TriggersHazard(H_4)
            doc /* Auto-Throttle System provides Idle Thrust Retard Command too early prior to main landing gear touchdown. */
            attribute controller : String = "Auto-Throttle System";
            attribute control_action : String = "Idle Thrust Retard Command";
            attribute uca_type : String = "Provided Too Early";
            attribute context_vector : String = "Issued 5.0 s prior to main landing gear touchdown, h = 80 ft AGL, S_phase = APPROACH";
            attribute triggered_hazard_id : String = "H_4";
            attribute severity : String = "Major";
            attribute dal_level : String = "DAL C";
        }

        requirement def UCA_12 {
            @TriggersHazard(H_4)
            doc /* Auto-Throttle / Reverser applies Reverse Thrust Actuator Drive Power too long after low taxi speed. */
            attribute controller : String = "Auto-Throttle / Reverser";
            attribute control_action : String = "Reverse Thrust Actuator Drive Power";
            attribute uca_type : String = "Applied Too Long";
            attribute context_vector : String = "Reverse thrust applied for t > 3.0 s after ground taxi speed drops below V_CAS < 10 kts, S_phase = TAXI";
            attribute triggered_hazard_id : String = "H_4";
            attribute severity : String = "Major";
            attribute dal_level : String = "DAL C";
        }

        requirement def UCA_13 {
            @TriggersHazard(H_2)
            doc /* Rudder Yaw Damper fails to provide Asymmetric Thrust Compensation Yaw Command during single engine failure. */
            attribute controller : String = "Rudder Yaw Damper";
            attribute control_action : String = "Asymmetric Thrust Compensation Yaw Command";
            attribute uca_type : String = "Not Provided";
            attribute context_vector : String = "Single engine failure event (T_eng1 - T_eng2 > 40%), V_CAS > V_1, h > 100 ft AGL, S_phase = TAKEOFF";
            attribute triggered_hazard_id : String = "H_2";
            attribute severity : String = "Catastrophic";
            attribute dal_level : String = "DAL A";
        }

        requirement def UCA_14 {
            @TriggersHazard(H_5)
            doc /* Rudder Yaw Damper provides Full Scale Rudder Deflection Command unsafely above maneuvering speed. */
            attribute controller : String = "Rudder Yaw Damper";
            attribute control_action : String = "Full Scale Rudder Deflection Command";
            attribute uca_type : String = "Provided Unsafely";
            attribute context_vector : String = "Airspeed exceeds maneuvering speed (V_CAS > V_A = 270 kts), h = 15,000 ft MSL, S_phase = CRUISE";
            attribute triggered_hazard_id : String = "H_5";
            attribute severity : String = "Catastrophic";
            attribute dal_level : String = "DAL A";
        }

        requirement def UCA_15 {
            @TriggersHazard(H_2)
            doc /* ARINC 653 Partition Switcher fails to provide Executive Partition Context Switch Command on minor frame expiration. */
            attribute controller : String = "ARINC 653 Partition Switcher";
            attribute control_action : String = "Executive Partition Context Switch Command";
            attribute uca_type : String = "Not Provided";
            attribute context_vector : String = "Flight Control Partition 1 execution minor frame deadline expired (t > 20 ms), S_phase = ALL";
            attribute triggered_hazard_id : String = "H_2";
            attribute severity : String = "Catastrophic";
            attribute dal_level : String = "DAL A";
        }

        requirement def UCA_16 {
            @TriggersHazard(H_2)
            doc /* ARINC 653 Partition Switcher provides Partition Preemption Switch Command unsafely during active DAL A flight loop. */
            attribute controller : String = "ARINC 653 Partition Switcher";
            attribute control_action : String = "Partition Preemption Switch Command";
            attribute uca_type : String = "Provided Unsafely";
            attribute context_vector : String = "Preempting active DAL A Flight Control Partition 1 to service lower-criticality Partition 4 during maneuver";
            attribute triggered_hazard_id : String = "H_2";
            attribute severity : String = "Catastrophic";
            attribute dal_level : String = "DAL A";
        }
    }

    // =========================================================================
    // 5. STPA Safety Control Constraints Package
    // 32 Formal SysML v2 requirement definitions (SC_1 ... SC_32) with @SafetyRealises tags
    // =========================================================================
    package STPA_Safety_Control_Constraints {

        requirement def SC_1 {
            @SafetyRealises(UCA_01)
            doc /* FCC shall not command control surface deflection exceeding structural load limit +/- 30 deg. */
            attribute max_surface_deflection_deg : Real = 30.0;
        }

        requirement def SC_2 {
            @SafetyRealises(UCA_02)
            doc /* Autopilot shall inhibit arming of automatic land mode when Weight-on-Wheels (WoW) flag is false during cruise. */
        }

        requirement def SC_3 {
            @SafetyRealises(UCA_03)
            doc /* ActuatorControlUnit shall command motor shutdown within 50ms upon detecting persistent overcurrent condition (>45A). */
            attribute response_time_ms : Real = 50.0;
        }

        requirement def SC_4 {
            @SafetyRealises(UCA_04)
            doc /* DAASensorSuite shall generate well-clear conflict warning at least 30 seconds prior to predicted collision point. */
            attribute warning_lead_time_s : Real = 30.0;
        }

        requirement def SC_5 {
            @SafetyRealises(UCA_05)
            doc /* FlightControlComputer shall transition to Degraded Mode upon losing 2 out of 3 redundant sensor channels. */
        }

        requirement def SC_6 {
            @SafetyRealises(UCA_06)
            doc /* GroundControlStation shall enforce heartbeat acknowledgment every 500ms over C25GLinkPort. */
            attribute heartbeat_interval_ms : Real = 500.0;
        }

        requirement def SC_7 {
            @SafetyRealises(UCA_07)
            doc /* AutopilotGuidanceEngine shall trigger automatic Return-To-Home (RTH) when telemetry link loss exceeds 5 seconds. */
            attribute link_loss_threshold_s : Real = 5.0;
        }

        requirement def SC_8 {
            @SafetyRealises(UCA_08)
            doc /* FlightControlComputer shall maintain minimum airspeed above stall speed (V_CAS > 1.3 * V_stall) during automated maneuvers. */
        }

        requirement def SC_9 {
            @SafetyRealises(UCA_09)
            doc /* ActuatorControlUnit shall perform cyclic parity checks on CAN bus frames and discard corrupted frames. */
        }

        requirement def SC_10 {
            @SafetyRealises(UCA_10)
            doc /* DAASensorSuite shall prioritize TCAS Resolution Advisories (RA) over ground guidance trajectory commands. */
        }

        requirement def SC_11 {
            @SafetyRealises(UCA_11)
            doc /* FlightControlComputer shall limit pitch angle (theta) to -15 deg <= theta <= +20 deg in normal flight law. */
        }

        requirement def SC_12 {
            @SafetyRealises(UCA_12)
            doc /* FlightControlComputer shall limit roll angle (phi) to +/- 45 deg in normal flight law. */
        }

        requirement def SC_13 {
            @SafetyRealises(UCA_13)
            doc /* AutopilotGuidanceEngine shall reject waypoint upload if terrain clearance is less than 150m AGL. */
            attribute min_terrain_clearance_m : Real = 150.0;
        }

        requirement def SC_14 {
            @SafetyRealises(UCA_14)
            doc /* FlightControlComputer shall lock out manual override input if pilot input rate exceeds structural g-limit. */
        }

        requirement def SC_15 {
            @SafetyRealises(UCA_15)
            doc /* ActuatorControlUnit shall lock control surface in trim state upon total bus power failure. */
        }

        requirement def SC_16 {
            @SafetyRealises(UCA_16)
            doc /* GroundControlStation shall display emergency geofence breach alert within 100ms of boundary detection. */
        }

        requirement def SC_17 {
            @SafetyRealises(UCA_17)
            doc /* FlightControlComputer shall trigger ballistic parachute deployment if dual engine failure occurs below 300m AGL. */
            attribute min_parachute_altitude_m : Real = 300.0;
        }

        requirement def SC_18 {
            @SafetyRealises(UCA_18)
            doc /* DAASensorSuite shall maintain target tracking across 360-degree azimuth with minimum 99.9% detection probability. */
        }

        requirement def SC_19 {
            @SafetyRealises(UCA_19)
            doc /* FlightControlComputer shall suppress high-frequency actuator jitter above 25 Hz. */
        }

        requirement def SC_20 {
            @SafetyRealises(UCA_20)
            doc /* AutopilotGuidanceEngine shall calculate remaining fuel/battery reserve requirement (minimum 20% landing reserve). */
            attribute min_landing_reserve_percent : Real = 20.0;
        }

        requirement def SC_21 {
            @SafetyRealises(UCA_21)
            doc /* FlightControlComputer shall execute cross-channel voter consensus check every ARINC 653 partition cycle. */
        }

        requirement def SC_22 {
            @SafetyRealises(UCA_22)
            doc /* ActuatorControlUnit shall cross-compare commanded position against encoder feedback and flag mismatch > 2 deg. */
        }

        requirement def SC_23 {
            @SafetyRealises(UCA_23)
            doc /* DAASensorSuite shall reject invalid ADS-B frames failing CRC-24 checksum. */
        }

        requirement def SC_24 {
            @SafetyRealises(UCA_24)
            doc /* GroundControlStation shall mandate dual-operator authentication for mission plan modifications during flight. */
        }

        requirement def SC_25 {
            @SafetyRealises(UCA_25)
            doc /* FlightControlComputer shall bound maximum vertical descent rate to -5.0 m/s in precision approach mode. */
            attribute max_descent_rate_mps : Real = 5.0;
        }

        requirement def SC_26 {
            @SafetyRealises(UCA_26)
            doc /* AutopilotGuidanceEngine shall execute emergency loiter pattern when destination airport status is red. */
        }

        requirement def SC_27 {
            @SafetyRealises(UCA_27)
            doc /* FlightControlComputer shall monitor Pitot-static tube heating and alarm upon ice detection. */
        }

        requirement def SC_28 {
            @SafetyRealises(UCA_28)
            doc /* ActuatorControlUnit shall enforce zero-heap memory allocation during real-time execution loops. */
        }

        requirement def SC_29 {
            @SafetyRealises(UCA_29)
            doc /* DAASensorSuite shall log raw sensor telemetry to crash-survivable memory module. */
        }

        requirement def SC_30 {
            @SafetyRealises(UCA_30)
            doc /* GroundControlStation shall encrypt all C2 link frames using AES-256-GCM. */
        }

        requirement def SC_31 {
            @SafetyRealises(UCA_31)
            doc /* FlightControlComputer shall initiate automatic climb if ground proximity warning system (GPWS) alert triggers. */
        }

        requirement def SC_32 {
            @SafetyRealises(UCA_32)
            doc /* FlightControlComputer shall isolate babbling node on ARINC 429 / MIL-STD-1553 bus within 10ms of detection. */
            attribute max_isolation_delay_ms : Real = 10.0;
        }
    }

    // =========================================================================
    // 6. FMECA Risk Profiles Package
    // Hardware fault models (FMECA-HW-01 through FMECA-HW-08) with RPN attributes
    // =========================================================================
    package FMECA_Risk_Profiles {

        attribute def FMECA_HW_01 {
            doc /* FCC Processor Single-Event Upset (SEU) Stuck-at-Fault. */
            attribute severity : Integer = 10;
            attribute occurrence : Integer = 2;
            attribute detection : Integer = 2;
            attribute rpn : Integer = 40; // 10 * 2 * 2
            attribute mitigation : String = "TMR Lockstep Voting & Auto-Reset";
        }

        attribute def FMECA_HW_02 {
            doc /* Actuator Servo Driver MOSFET Short Circuit Failure. */
            attribute severity : Integer = 9;
            attribute occurrence : Integer = 3;
            attribute detection : Integer = 2;
            attribute rpn : Integer = 54; // 9 * 3 * 2
            attribute mitigation : String = "Dual-FET In-line Fuse Isolation";
        }

        attribute def FMECA_HW_03 {
            doc /* ARINC 429 Transmitter Babbling Line Transceiver Failure. */
            attribute severity : Integer = 8;
            attribute occurrence : Integer = 3;
            attribute detection : Integer = 1;
            attribute rpn : Integer = 24; // 8 * 3 * 1
            attribute mitigation : String = "Bus Controller Hardware Timeout & Bus Disconnect Switch";
        }

        attribute def FMECA_HW_04 {
            doc /* Pitot Tube Icing Blockage Signal Drift Fault. */
            attribute severity : Integer = 9;
            attribute occurrence : Integer = 4;
            attribute detection : Integer = 2;
            attribute rpn : Integer = 72; // 9 * 4 * 2
            attribute mitigation : String = "Triple Air Data Computer (ADC) Cross-Comparison & Pitot Heater";
        }

        attribute def FMECA_HW_05 {
            doc /* C2 5G Transceiver Power Amplifier Thermal Shutdown Fault. */
            attribute severity : Integer = 7;
            attribute occurrence : Integer = 3;
            attribute detection : Integer = 1;
            attribute rpn : Integer = 21; // 7 * 3 * 1
            attribute mitigation : String = "Dual Satellite / 5G Hybrid Modem Fallback";
        }

        attribute def FMECA_HW_06 {
            doc /* Power Distribution Unit Relay Contact Weld Fault. */
            attribute severity : Integer = 9;
            attribute occurrence : Integer = 2;
            attribute detection : Integer = 3;
            attribute rpn : Integer = 54; // 9 * 2 * 3
            attribute mitigation : String = "Dual High-Side Solid State Power Controllers (SSPC)";
        }

        attribute def FMECA_HW_07 {
            doc /* Inertial Measurement Unit (IMU) Accelerometer Bias Drift Fault. */
            attribute severity : Integer = 8;
            attribute occurrence : Integer = 3;
            attribute detection : Integer = 2;
            attribute rpn : Integer = 48; // 8 * 3 * 2
            attribute mitigation : String = "Extended Kalman Filter (EKF) Innovation Residual Monitoring";
        }

        attribute def FMECA_HW_08 {
            doc /* Ground Control Station Operator Workstation Power Loss Fault. */
            attribute severity : Integer = 6;
            attribute occurrence : Integer = 2;
            attribute detection : Integer = 1;
            attribute rpn : Integer = 12; // 6 * 2 * 1
            attribute mitigation : String = "Redundant GCS Hot-Standby Console & Autonomous Loiter Mode";
        }
    }

    // =========================================================================
    // 7. Stateflow Execution State Machines Package
    // Formal SysML v2 state definitions for ARINC 653 100ms major frame schedule
    // and UAS fail-safe state transitions.
    // =========================================================================
    package Stateflow_Execution_State_Machines {

        state def ARINC653_MajorFrameSchedule {
            doc /* 100ms Major Frame cyclic executive schedule for partitioned avionics. */

            state Partition1_FlightControl {
                doc /* Execution window 0ms - 30ms: Primary Control Law Computation. */
            }

            state Partition2_Navigation {
                doc /* Execution window 30ms - 55ms: EKF & Sensor Fusion. */
            }

            state Partition3_HealthMonitor {
                doc /* Execution window 55ms - 75ms: Built-In Test (BIT) & FMECA Monitors. */
            }

            state Partition4_Telemetry {
                doc /* Execution window 75ms - 100ms: C2 Link Packaging & Telemetry Tx. */
            }
        }

        state def Failsafe_StateMachine {
            doc /* UAS Failsafe Logic State Machine handling nominal, degraded, RTH, and parachute modes. */

            state NormalMode {
                doc /* Nominal Flight Operation with all TMR channels healthy. */
            }

            state DegradedMode {
                doc /* Single sensor failure or minor link degradation detected. System restricts flight envelope. */
            }

            state ReturnToHomeState {
                doc /* Persistent C2 link loss (>5s) or low fuel state. Automated waypoint navigation back to home base. */
            }

            state ParachuteDeploymentState {
                doc /* Unrecoverable dual engine or total flight computer failure below emergency threshold altitude. */
            }
        }
    }

    // =========================================================================
    // 8. Simulink Export Profile Package
    // Annotations and metadata for MATLAB Simulink & Stateflow synthesis
    // =========================================================================
    package Simulink_Export_Profile {

        attribute def SimulinkBlock {
            doc /* Annotates SysML part definition as a MATLAB Simulink block subsystem. */
            attribute block_type : String; // e.g. "Subsystem", "S-Function", "ModelReference"
            attribute model_file : String; // Target .slx filename
        }

        attribute def StateflowChart {
            doc /* Annotates SysML state definition as a MATLAB Stateflow chart element. */
            attribute chart_type : String; // e.g. "Classic", "Reactive", "Mealy", "Moore"
            attribute stateflow_file : String; // Target .sfx filename
        }

        attribute def SimulinkSignal {
            doc /* Annotates SysML port definition or attribute definition as a Simulink bus signal. */
            attribute signal_type : String; // e.g. "double", "single", "boolean", "BusObject"
            attribute sample_time_s : Real; // Sampling period in seconds
        }
    }

    // =========================================================================
    // 9. Evolved 3-Layer Semantic Chain Packages
    // Explicit 3-layer semantic annotations connecting Certification_Requirements,
    // Avionic_Control_Structure, Safety_Statecharts, and ARINC661_Display_Interfaces.
    // =========================================================================

    // --- Layer 1: Domain State & Signal Model ---
    package Avionic_Control_Structure {
        doc /* Layer 1 (Domain State & Signal Model): Defines primary flight control data structures, continuous state vectors, and telemetry signal models. Binds to Certification_Requirements. */

        part def FlightControlStructure {
            @Layer(1)
            @RealisesLayer("Domain State & Signal Model")
            doc /* Structural decomposition of flight control sensor inputs, state vector estimators, and command buffers. */
            attribute control_frequency_hz : Real = 400.0;
        }

        attribute def SignalVectorModel {
            @Layer(1)
            doc /* Raw and filtered telemetry signal vector parameters. */
            attribute sensor_validity_mask : Integer = 255;
        }
    }

    // --- Layer 2: Logic & Safety State Management ---
    package Safety_Statecharts {
        doc /* Layer 2 (Logic & Safety State Management): Formal finite state machines, safety statecharts, and Mode Logic transition laws. Binds Layer 1 to Layer 3. */

        state def FailsafeStatechart {
            @Layer(2)
            @RealisesLayer("Logic & Safety State Management")
            doc /* Hierarchical safety statechart governing flight mode transitions, interlocks, and emergency degraded states. */

            state NominalControl {
                doc /* Closed-loop flight control mode under DO-178C DAL A guidance laws. */
            }

            state EmergencyOverride {
                doc /* Automatic failsafe statechart execution upon hazard detection. */
            }
        }
    }

    // --- Layer 3: Display & Actuator Interface Binding ---
    package ARINC661_Display_Interfaces {
        doc /* Layer 3 (Display & Actuator Interface Binding): Binding CDS widgets, ARINC 661 binary parameter buffers, vector symbology renders, and actuator control interfaces. */

        port def ARINC661_WidgetBindingPort {
            @Layer(3)
            @RealisesLayer("Display & Actuator Interface Binding")
            doc /* ARINC 661 widget protocol parameter buffer port for Cockpit Display System (CDS) and actuator drivers. */
            attribute widget_id : Integer;
            attribute parameter_id : Integer;
        }

        part def DisplayActuatorBinding {
            @Layer(3)
            doc /* Physical interface binding display kernel rendering engines and actuator servo drivers. */
            attribute refresh_rate_hz : Real = 60.0;
        }
    }
}
