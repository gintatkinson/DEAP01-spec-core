doc /* ConOps Level 0 External Operational Actors */
package ConOps {
    doc /* External human systems engineer interacting with compiler tooling and schemas */
    part def HumanEngineer;

    doc /* Continuous Integration runner executing automated compilation and verification gates */
    part def CIContinuousIntegrationRunner;

    doc /* Remote artifact registry storing and serving upstream models and dependencies */
    part def RemoteArtifactRegistry;

    doc /* Downstream application host executing synthesized projections and runtime modules */
    part def DownstreamApplicationHost;
}
