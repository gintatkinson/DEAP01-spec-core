doc /* Structural Architecture for Subsystem 8 Level 1C ICD Interconnect Contracts */
package InterconnectContracts {
    import DEAP_Compiler_System::*;

    doc /* Primary execution engine for Level 1C Interface Control Document (ICD) contracts */
    part def ICDEngine {
        port in safety_in : SafetyPort;
        port out icd_out : ICDInterconnectPort;

        part matrixBuilder : InterfaceMatrixBuilder;
        part compatChecker : PortCompatibilityChecker;
        part serializer : ContractSerializer;
    }

    doc /* Canonical N x N interface matrix builder and lexicographical coordinate assigner */
    part def InterfaceMatrixBuilder {
        port in safety_in : SafetyPort;
        port out icd_out : ICDInterconnectPort;
    }

    doc /* Port type, protocol, and signal dictionary compatibility validator */
    part def PortCompatibilityChecker {
        port in safety_in : SafetyPort;
        port out icd_out : ICDInterconnectPort;
    }

    doc /* Interface contract serializer emitting machine-readable and tabular ICD specifications */
    part def ContractSerializer {
        port in safety_in : SafetyPort;
        port out icd_out : ICDInterconnectPort;
    }
}
