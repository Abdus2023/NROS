# Part LI — Observability, Telemetry, Audit, Evidence & Forensic Reconstruction

> **Series:** NROS Architecture Series  
> **Part:** LI  
> **Role:** Logs, metrics, traces, events, diagnostics, audit records, evidence correlation, integrity, retention, and forensic reconstruction  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part L established the security plane. Part LI establishes the observability and evidence plane needed to understand, operate, verify, audit, and reconstruct NROS behavior.

The central rule is:

> **NROS must distinguish telemetry used for operations from audit records used for accountability and from evidence used to establish what can actually be demonstrated.**

## 2. Three Distinct Concepts

```text
Telemetry
  → helps operate the system

Audit
  → records accountable actions

Evidence
  → supports a verifiable claim
```

Therefore:

```text
Telemetry ≠ Audit ≠ Proof
```

## 3. Observability Model

```text
Runtime
 ↓
Signals
 ├─ Logs
 ├─ Metrics
 ├─ Traces
 ├─ Events
 └─ Profiles / diagnostics
 ↓
Correlation
 ↓
Operational Understanding
```

## 4. Logs

Logs represent textual or structured records generated during execution.

Structured logs are preferred for machine processing.

## 5. Log Schema

A structured log may contain:

```text
timestamp
severity
component
instance
message_id
request_id
correlation_id
trace_id
scope
message
attributes
```

## 6. Log Levels

Severity should be explicit and stable:

```text
trace
debug
info
warn
error
fatal
```

The exact levels may vary by implementation.

## 7. Log Safety

Logs must not become a secret-exfiltration channel.

Sensitive values require redaction or omission.

## 8. Metrics

Metrics represent aggregated measurements rather than individual events.

Examples:

```text
requests_total
request_latency
queue_depth
active_workers
error_rate
memory_usage
```

## 9. Metric Cardinality

Labels with unbounded cardinality can destabilize observability systems.

Cardinality must therefore be treated as a resource constraint.

## 10. Metric Semantics

Every metric should define:

```text
name
unit
type
aggregation
labels
reset behavior
```

## 11. Histograms

Latency and size distributions should use histograms or equivalent distributions where averages alone hide important behavior.

## 12. Gauges

Gauges represent current or sampled values and should not be interpreted as counters.

## 13. Counters

Counters represent monotonically increasing events subject to reset semantics.

## 14. Traces

A trace represents a distributed operation across components:

```text
Trace
 ├─ Span A
 ├─ Span B
 ├─ Span C
 └─ Span D
```

## 15. Trace Context

Cross-boundary calls should propagate trace context where supported:

```text
trace_id
span_id
parent_span
sampling state
```

## 16. Correlation

NROS should correlate:

```text
request_id
message_id
correlation_id
causation_id
trace_id
resource_id
operation_id
```

These identifiers serve different purposes and should not be collapsed into one opaque field.

## 17. Causality

Causation enables reconstruction:

```text
Command A
 ↓
Event B
 ↓
Command C
 ↓
State D
```

## 18. Event Telemetry

Events may represent runtime observations such as:

```text
started
scheduled
completed
failed
recovered
leader_changed
member_joined
member_left
```

Operational events are not automatically audit records.

## 19. Audit Records

Audit records capture security- or accountability-sensitive actions:

```text
principal
operation
resource
policy decision
result
time
context
```

## 20. Audit Immutability

Audit records should have stronger integrity and retention requirements than ordinary debug logs.

## 21. Audit Scope

Examples include:

```text
login
credential change
policy change
membership change
privilege delegation
administrative action
protected execution
secret access
shutdown
recovery
```

## 22. Evidence

Evidence is an artifact that supports a specific claim.

A claim must identify what evidence would be sufficient to establish it.

## 23. Evidence Levels

A useful conceptual progression is:

```text
UNKNOWN
 ↓
OBSERVED
 ↓
REPRODUCED
 ↓
VALIDATED
 ↓
VERIFIED
```

The exact project vocabulary may evolve, but state transitions must be evidence-backed.

## 24. Evidence vs Assertion

```text
"The test passes"
    ≠
Recorded CI result demonstrating the test passed
```

Documentation must not promote an unobserved assertion into verified state.

## 25. Provenance

Every evidence artifact should preserve provenance where practical:

```text
source
producer
timestamp
revision
environment
method
integrity
```

## 26. Evidence Identity

Evidence should have a stable identity allowing references from claims, reports, and audits.

## 27. Evidence Correlation

Evidence can correlate across layers:

```text
Requirement
 ↓
Design decision
 ↓
Implementation
 ↓
Commit
 ↓
CI run
 ↓
Artifact
 ↓
Verification result
```

## 28. Traceability

NROS architecture should support bidirectional traceability:

```text
Requirement → Evidence
Evidence → Requirement
```

## 29. Verification Evidence

Verification evidence should identify:

```text
what was tested
how it was tested
where it ran
which revision ran
result
limitations
```

## 30. CI Evidence

A CI configuration is not execution evidence.

```text
Workflow definition
    ≠
Executed workflow result
```

## 31. Reproducibility

Evidence is stronger when another actor can reproduce the observation under declared conditions.

## 32. Environment Capture

Verification records may need:

```text
OS
architecture
toolchain
runtime version
dependencies
configuration
hardware
network assumptions
```

## 33. Artifact Integrity

Important artifacts should have integrity metadata such as cryptographic hashes where appropriate.

## 34. Hashes

A digest can establish byte identity:

```text
artifact
 ↓ hash
H(artifact)
```

A hash alone does not establish authorship, correctness, or semantic validity.

## 35. Signatures

Digital signatures can strengthen authenticity and provenance when the signing identity and trust model are defined.

## 36. Chain of Custody

Sensitive evidence may require:

```text
created
 ↓
collected
 ↓
stored
 ↓
transferred
 ↓
analyzed
```

with integrity-preserving records at each transition.

## 37. Clock Semantics

Wall-clock timestamps can drift.

Distributed reconstruction should therefore use logical ordering or monotonic clocks where appropriate.

## 38. Timestamp Classes

Distinguish:

```text
wall time
monotonic time
logical time
commit position
```

## 39. Ordering

Timestamp order alone must not be treated as causal order across independent nodes.

## 40. Event Ordering Evidence

When exact order matters, use protocol sequence numbers, commit positions, epochs, or equivalent authoritative ordering mechanisms.

## 41. Diagnostic State

Diagnostics should expose internal health without becoming an alternate source of truth.

```text
diagnostic observation
    ≠
authoritative state transition
```

## 42. Health Correlation

Health signals should correlate with:

```text
node
service
dependency
resource
operation
```

## 43. Failure Evidence

A failure record should capture enough context to distinguish:

```text
application failure
transport failure
resource exhaustion
policy rejection
stale authority
external dependency failure
```

## 44. Recovery Evidence

Recovery should generate evidence of:

```text
detection
containment
restart/recovery
state restoration
revalidation
return to service
```

## 45. Security Evidence

Security-sensitive events should preserve:

```text
principal
credential/context identifier
policy decision
resource
operation
result
```

without exposing secret material.

## 46. Authorization Evidence

For a protected action:

```text
Request
 ↓
Identity
 ↓
Policy
 ↓
Decision
 ↓
Execution
```

The evidence chain should make the decision reconstructable.

## 47. Consensus Evidence

Distributed authority changes should be observable through evidence such as:

```text
epoch
leader transition
membership change
proposal/decision identity
commit position
```

## 48. Persistence Evidence

Durability-sensitive operations should correlate:

```text
logical state
 ↓
commit
 ↓
checkpoint / durable record
 ↓
recovery observation
```

## 49. Execution Evidence

Execution records should distinguish:

```text
scheduled
started
running
committed
completed
failed
cancelled
```

## 50. Scheduler Evidence

Scheduling decisions may capture:

```text
work item
scheduler
constraints
selected worker
priority
reason
result
```

## 51. Protocol Evidence

Protocol records should preserve enough metadata to correlate:

```text
message_id
schema
version
sender
recipient
correlation
result
```

Payload capture must respect security and privacy policy.

## 52. API Evidence

API operations should correlate:

```text
request_id
principal
resource
operation
response
latency
error
```

## 53. Sampling

Sampling reduces observability cost but creates evidence gaps.

Sampling policy must be explicit.

## 54. Tail Sampling

Tail-based sampling may preserve anomalous traces more effectively than uniform sampling.

## 55. Critical Event Retention

Security, authority, and state-transition events may require stronger retention than ordinary diagnostics.

## 56. Retention

Retention policy should specify:

```text
what
where
how long
why
who may access
when deletion occurs
```

## 57. Data Minimization

Collect only the observability data necessary for the declared operational, security, or verification purpose.

## 58. Privacy

Observability pipelines must respect applicable privacy and access-control requirements.

## 59. Multi-Tenant Isolation

Telemetry and audit stores must prevent cross-tenant disclosure.

Tenant context should be enforced at collection, storage, query, and export boundaries.

## 60. Access Control

Observability data is itself protected data.

Access may require separate permissions for:

```text
metrics
logs
traces
audit
evidence
security records
```

## 61. Export

Exports should preserve provenance and integrity metadata.

## 62. Evidence Packages

A verification package may contain:

```text
claim
scope
revision
environment
commands
outputs
artifacts
hashes
limitations
```

## 63. Negative Evidence

Absence can be meaningful only when the observation mechanism has sufficient coverage.

```text
No observed event
    ≠
Event definitely did not occur
```

## 64. Coverage

Evidence confidence depends on observation coverage.

A blind spot must be recorded rather than silently interpreted as absence.

## 65. Evidence Freshness

Evidence can become stale when the system revision or environment changes.

Verification claims should identify the revision they apply to.

## 66. Evidence Invalidation

A new implementation revision may invalidate previous evidence for implementation-specific claims.

## 67. Claim State

Claims should be modeled separately from evidence artifacts:

```text
Claim
 ↓ supported by
Evidence
 ↓ produced by
Observation / Verification
```

## 68. Confidence

Confidence should not be used as a substitute for missing evidence.

A useful distinction is:

```text
confidence
 ≠
verification
```

## 69. Forensic Reconstruction

A reconstruction seeks to answer:

```text
What happened?
Who initiated it?
Which authority applied?
Which state existed?
What was committed?
What failed?
What recovered?
```

## 70. Reconstruction Graph

```text
Request
 ↓
Command
 ↓
Scheduler decision
 ↓
Execution
 ↓
State transition
 ↓
Persistence
 ↓
Event
```

## 71. Reconstruction Boundaries

The system must identify where evidence is unavailable rather than inventing continuity across gaps.

## 72. Missing Evidence

```text
Evidence gap
    ↓
UNKNOWN
```

not:

```text
Evidence gap
    ↓
PASS
```

## 73. Incident Timeline

Incident timelines should combine:

```text
wall time
logical order
commit positions
operator actions
system events
```

## 74. Causal Reconstruction

Causality should rely on explicit identifiers and state transitions rather than timestamps alone.

## 75. Diagnostic Snapshots

Snapshots can capture system state at a point in time but must identify whether the snapshot is:

```text
consistent
best-effort
partial
stale
```

## 76. Snapshot Evidence

A snapshot is evidence of observed state, not necessarily proof of the entire system history.

## 77. Runtime Introspection

Introspection interfaces should be bounded and permission-controlled.

## 78. Debug Modes

Debug instrumentation must not silently weaken security, isolation, or correctness guarantees.

## 79. Observability Failure

The observability subsystem itself can fail.

Core runtime correctness must not depend on telemetry availability unless explicitly designed as a safety dependency.

## 80. Fail-Safe Observability

When telemetry is unavailable:

```text
Operational service
    may continue
```

but security/audit-critical operations may need to fail closed if their audit contract requires durable evidence.

## 81. Evidence Backpressure

Evidence collection must have bounded resource usage.

## 82. Loss Policy

If telemetry is lossy, the system must declare what can be lost.

Critical audit evidence should have stronger durability requirements.

## 83. Audit vs Debug

```text
Debug log
    ≠
Audit record
```

Deleting debug logs must not erase required accountability records.

## 84. Audit vs Proof

An audit trail can establish what the system recorded; it does not automatically establish that the recorded event was truthful unless the trust model supports that conclusion.

## 85. Evidence vs Proof

A proof claim requires explicit verification semantics.

```text
Evidence
    → supports claim

Proof
    → satisfies defined verification obligation
```

## 86. Formal Evidence Invariant

```text
ClaimVerified(C)
    ⇒
RequiredEvidence(C) ∧ EvidenceValid(C)
```

## 87. Formal Provenance Invariant

```text
Evidence(E)
    ⇒
KnownSource(E)
 ∧
KnownRevision(E)
 ∧
KnownMethod(E)
```

where the project declares those provenance fields required for the evidence class.

## 88. Formal Audit Invariant

```text
PrivilegedAction(A)
    ⇒
AuditRecord(A)
```

when the security policy requires auditing.

## 89. Formal Reconstruction Invariant

```text
Reconstruct(Event)
    ⇒
EvidenceChain(Event)
```

If the chain is incomplete, the reconstruction must identify the uncertainty.

## 90. Formal Telemetry Invariant

```text
TelemetryUnavailable
    ≠
RuntimeCorrectnessFailure
```

unless telemetry is explicitly part of the correctness or safety contract.

## 91. Verification Matrix

| Property | Verification question |
|---|---|
| Logs | Are records structured and bounded? |
| Metrics | Are units, aggregation, and cardinality controlled? |
| Traces | Is distributed context propagated? |
| Correlation | Can operations be connected across layers? |
| Audit | Are privileged actions accountable? |
| Evidence | Can claims identify supporting artifacts? |
| Provenance | Is source/revision/method preserved? |
| Integrity | Can important artifacts be authenticated? |
| Retention | Are retention/deletion policies explicit? |
| Privacy | Is sensitive data protected? |
| Isolation | Can tenants access only authorized telemetry? |
| Sampling | Are evidence gaps understood? |
| Recovery | Are failures and recovery observable? |
| Consensus | Are authority transitions reconstructable? |
| Persistence | Can durability transitions be correlated? |
| Execution | Are lifecycle states distinguishable? |
| API | Can external requests be reconstructed? |
| Gaps | Are missing observations represented as unknown? |
| Forensics | Can incidents be reconstructed without inventing facts? |
| Failure | Does observability failure have bounded impact? |

## 92. What Part LI Does Not Claim

This Part does not claim that the current NROS implementation already has:

- complete distributed tracing;
- production-grade audit storage;
- tamper-evident evidence archives;
- universal forensic reconstruction;
- complete telemetry coverage;
- cryptographically signed evidence for every event;
- a finalized retention policy;
- zero-loss observability;
- formally verified evidence pipelines.

Those require implementation-specific evidence.

## 93. Transition to Part LII

Part LI establishes observability, audit, and evidence semantics.

Part LII should define **configuration, control-plane state, dynamic reconfiguration, policy distribution, feature activation, and safe configuration rollout**.

```text
Part XLIX
API + service boundaries
        ↓
Part L
Security + identity + authorization
        ↓
Part LI
Observability + audit + evidence
        ↓
Part LII
Configuration + control state + safe reconfiguration
```

## Canonical rule

> **NROS must never confuse what was emitted, what was audited, and what was actually verified: telemetry explains operation, audit establishes accountability, and evidence supports explicitly defined verification claims.**
