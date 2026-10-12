doc /* ConOps Level 0 Operational Lifecycle State Machine */
package ConOps {
    doc /* Compiler operational lifecycle state machine */
    state def CompilerLifecycleModes {
        entry;
        state Bootstrapping;
        state Ingesting;
        state Compiling;
        state Verifying;
        state EmitSuccess;
        state FaultTerminated;

        transition InitialToBoot
            first entry
            then Bootstrapping;

        transition BootToIngest
            first Bootstrapping
            then Ingesting;

        transition IngestToCompile
            first Ingesting
            then Compiling;

        transition CompileToVerify
            first Compiling
            then Verifying;

        transition VerifyToEmit
            first Verifying
            then EmitSuccess;

        transition CompileToFault
            first Compiling
            then FaultTerminated;

        transition VerifyToFault
            first Verifying
            then FaultTerminated;
    }
}
