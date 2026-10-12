doc /* ConOps Level 0 Operational Integrity Threats */
package ConOps {
    doc /* Syntactically invalid or semantically inconsistent AST payload causing parsing fault */
    item def MalformedASTPayload;

    doc /* Corrupted or tampered schema source input violating structural invariants */
    item def CorruptSchemaInput;

    doc /* Unauthorized modification or drift in downstream models bypassing AST primacy */
    item def UnauthorizedModelTampering;

    doc /* Resource exhaustion denial-of-service attempt via massive payload floods */
    item def ResourceExhaustionFlood;
}
