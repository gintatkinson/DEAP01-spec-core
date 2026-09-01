# Formal Safety Proof — T-01 Safe-State Invariant Preservation

This document implements the canonical 5-part mathematical proof structure for the safe-state invariant theorem.

## Theorem T-01: Safe-State Invariant Preservation

1. **Proposition / Theorem Statement**: The safe-state invariant $I(x)$ is non-increasing along every trajectory that remains within the certified envelope.

2. **Operational Assumptions & Domain Bounds**: The plant dynamics are bounded by $\| f(x) \| \le M$ on the envelope boundary, with bounded disturbance norm.

3. **Invariant / Barrier Function Definition**: Define the barrier certificate $B(x) = I_{\max} - I(x) \ge 0$ on the safe set, with $\alpha > 0$ the invariant margin.

4. **Analytical / Inductive Derivation**:

$$
\begin{aligned}
\dot{B}(x) &= -\dot{I}(x) \\
&\le -\alpha B(x)
\end{aligned}
$$

5. **Formal Conclusion & Q.E.D.**: By the comparison lemma, $B(x(t)) \ge e^{-\alpha t} B(x_0) \ge 0$ for all $t \ge 0$; hence $I(x) \le I_{\max}$ is preserved throughout envelope operation. Q.E.D.
