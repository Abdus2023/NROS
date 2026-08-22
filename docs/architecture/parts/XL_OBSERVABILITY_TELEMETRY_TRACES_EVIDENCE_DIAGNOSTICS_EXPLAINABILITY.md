# Part XL — Observability, Telemetry, Traces, Evidence, Diagnostics & Explainability

> **Series:** NROS Architecture Series  
> **Part:** XL  
> **Role:** Runtime observation, events, metrics, logs, traces, health signals, diagnostics, evidence, provenance, explainability, and observability governance  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part XXXIX established controlled runtime change. Part XL defines how NROS makes execution observable without confusing operational telemetry with formal proof.

The central rule is:

> **NROS separates observation from interpretation and evidence from proof: runtime signals must be attributable, time-aware, scoped, integrity-protected where required, privacy-aware, and useful for diagnosis, while no telemetry record is treated as formal assurance merely because it exists.**

## 2. Fundamental Distinctions

```text
observation
  ≠
telemetry
  ≠
metric
  ≠
log
  ≠
trace
  ≠
event
  ≠
evidence
  ≠
proof
```

## 3. Observation

An observation is a fact captured from or about runtime behavior.

Examples:

```text
state changed
request accepted
queue depth = 12
worker restarted
latency = 40 ms
```

## 4. Telemetry

Telemetry is the transport and representation of observations for operational use.

It may include:

```text
metrics
logs
events
traces
profiles
health signals
```

## 5. Event

An event represents a meaningful occurrence:

```text
Event
 ├─ identity
 ├─ type
 ├─ source
 ├─ time
 ├─ scope
 └─ payload
```

Events should be immutable once committed where auditability requires it.

## 6. Metric

A metric represents measured or aggregated numerical state:

```text
CPU utilization
queue depth
request rate
error rate
restart count
```

Metrics are useful for trends and thresholds but may lose individual-event detail.

## 7. Log

A log records diagnostic or operational information, often optimized for human or machine inspection.

Logs must not be assumed to be complete event history.

## 8. Trace

A trace links observations across execution boundaries:

```text
Request
 ↓
Workflow
 ↓
Task
 ↓
Worker
 ↓
Dependency
```

Trace context provides causal and diagnostic correlation, not automatic proof of causality.

## 9. Span

A span represents a bounded operation within a trace:

```text
Trace
 ├─ Span A
 ├─ Span B
 └─ Span C
```

Span relationships should distinguish parent-child execution from asynchronous or linked relationships.

## 10. Correlation Identity

Important observations should carry stable correlation identifiers:

```text
request_id
workflow_id
task_id
agent_id
worker_id
node_id
trace_id
span_id
```

Identifiers must be scoped to avoid accidental ambiguity.

## 11. Causality

Observability may record causal relationships:

```text
Event A
 ↓ caused / triggered
Event B
```

A recorded relation is an architectural assertion and should identify whether it is observed, inferred, or explicitly established by the runtime.

## 12. Logical Time

Part XXXVI defines temporal semantics. Observability should preserve the relevant ordering information rather than relying exclusively on wall-clock timestamps.

Useful fields can include:

```text
wall_time
monotonic_time
logical_time
epoch
sequence
```

## 13. Clock Uncertainty

Distributed timestamps may disagree.

```text
Node A: 10:00:01
Node B: 10:00:00
```

Diagnostics should avoid claiming precise ordering when the available clock evidence cannot support it.

## 14. Sequence Numbers

Per-source sequence numbers can detect gaps:

```text
100
101
103
```

The missing `102` should be detectable rather than silently ignored.

## 15. Event Integrity

Critical events may require:

```text
hash
signature
sequence
source identity
append-only storage
```

The required mechanism depends on the trust model.

## 16. Event Provenance

Every important observation should answer:

```text
Who produced it?
What produced it?
When?
Under which version?
For which scope?
```

## 17. Source Identity

Telemetry should identify its source:

```text
component
instance
node
agent
runtime version
```

Source identity must remain meaningful across restarts through explicit instance or epoch identifiers.

## 18. Runtime Epoch

A restarted component can receive a new epoch:

```text
worker instance
 epoch 7 → epoch 8
```

This helps distinguish current observations from stale observations.

## 19. Health Signals

Health telemetry should distinguish:

```text
liveness
readiness
progress
capacity
dependency health
state consistency
```

Part XXXVIII defines the corresponding supervision semantics.

## 20. Diagnostic State

A component may expose a diagnostic snapshot:

```text
state
health
queue depth
resource usage
active work
last error
restart count
epoch
configuration version
policy version
```

## 21. Explainability

Operational explainability answers:

```text
What happened?
Why was the action selected?
Which policy applied?
Which version was active?
What resources were available?
What evidence supports the diagnosis?
```

Explainability is an observability property, not necessarily formal proof.

## 22. Decision Provenance

Important decisions should be attributable to:

```text
input
policy version
configuration version
runtime version
resource state
decision
outcome
```

## 23. Agent Decision Records

For agentic execution, useful records include:

```text
observation set
selected action
policy constraints
tool invocation
result
termination reason
```

Sensitive internal reasoning should not be exposed merely because a diagnostic record exists; observability should capture actionable provenance without requiring unrestricted disclosure of private reasoning.

## 24. Evidence

Evidence is an observation or artifact that supports a claim.

```text
Claim
 ↓
Evidence
```

Evidence may be:

```text
runtime event
metric sample
log
trace
artifact
configuration snapshot
test result
```

## 25. Evidence ≠ Proof

```text
Telemetry exists
    ≠
Property proven
```

Formal proof, exhaustive verification, testing evidence, and runtime observations have different assurance levels.

## 26. Evidence Classification

A useful classification is:

```text
Observed
Derived
Reproduced
Verified
Formally Proven
```

The strongest label must not be assigned without the corresponding evidence.

## 27. Evidence Bundle

A diagnostic claim can be represented as:

```text
Evidence Bundle
 ├─ claim
 ├─ observations
 ├─ sources
 ├─ timestamps
 ├─ versions
 ├─ transformations
 └─ integrity metadata
```

## 28. Evidence Chain

```text
Runtime Event
 ↓
Collector
 ↓
Storage
 ↓
Query
 ↓
Analysis
 ↓
Claim
```

Each transformation should remain attributable where evidence integrity matters.

## 29. Sampling

High-volume telemetry may require sampling:

```text
10,000 events
 ↓ sampling
1,000 retained
```

Sampling introduces uncertainty and must be represented in analysis.

## 30. Tail Sampling

A system may retain traces selectively after observing their outcomes:

```text
Trace begins
 ↓
Outcome observed
 ↓
retain / discard
```

The policy must not imply complete trace coverage.

## 31. Aggregation

Metrics can aggregate events:

```text
100 requests
 ↓
error rate = 2%
```

Aggregation improves scalability but removes individual event detail.

## 32. Cardinality

Unbounded labels can create telemetry overload:

```text
request_id
user_id
random token
```

High-cardinality fields must be governed explicitly.

## 33. Telemetry Resource Budget

Observability consumes resources:

```text
CPU
memory
network
storage
I/O
```

Part XXXVII resource controls therefore apply to telemetry pipelines.

## 34. Telemetry Backpressure

```text
Producer
 ↓
Telemetry queue full
 ↓
Batch / sample / drop / block
```

The policy must define what happens when observability itself is under pressure.

## 35. Loss Semantics

Telemetry systems should distinguish:

```text
complete
sampled
aggregated
dropped
unknown
```

Silently treating dropped telemetry as absent events can corrupt diagnosis.

## 36. Critical Event Durability

Not every observation needs durable storage.

Critical events may require stronger guarantees:

```text
security event
state transition
authorization decision
recovery action
configuration activation
```

## 37. Audit vs Diagnostic Telemetry

Audit records and diagnostic telemetry have different purposes:

```text
Audit → accountability
Diagnostics → troubleshooting
```

A diagnostic stream should not automatically be treated as an authoritative audit trail.

## 38. Privacy

Telemetry can contain sensitive information.

Collection must follow data minimization:

```text
Need for diagnosis
        ↓
Minimum necessary data
```

## 39. Redaction

Sensitive fields may require:

```text
remove
mask
hash
tokenize
aggregate
```

Redaction must preserve enough structure for legitimate diagnostics where possible.

## 40. Secret Protection

Secrets should never be emitted merely because a configuration object is observable.

```text
Password
API key
private token
credential
```

must be explicitly protected.

## 41. Tenant Isolation

Multi-tenant telemetry must preserve tenant boundaries:

```text
Tenant A telemetry
      ║
   isolation
      ║
Tenant B telemetry
```

## 42. Access Control

Telemetry access should be governed by:

```text
identity
capability
role
scope
purpose
```

Observability does not bypass authorization.

## 43. Retention

Telemetry should have explicit retention classes:

```text
hot
warm
cold
expired
```

Retention should match diagnostic, audit, and compliance requirements.

## 44. Expiration

Expired telemetry must be removed or archived according to policy.

Retention is a lifecycle, not an implicit infinite-storage promise.

## 45. Storage Integrity

Important evidence stores may require:

```text
append-only semantics
checksums
signatures
versioned schemas
access audit
```

## 46. Schema Evolution

Telemetry schemas evolve:

```text
Event v1
 ↓
Event v2
```

Consumers need explicit compatibility rules.

## 47. Event Schema Registry

Where appropriate, schemas should be identifiable by:

```text
schema_id
version
producer
compatibility mode
```

## 48. Diagnostic Query

A diagnostic query should be reproducible:

```text
query
 + time range
 + filters
 + schema versions
 + data source
```

## 49. Reproducibility

A diagnostic conclusion should record enough context to reproduce the analysis when practical.

```text
Evidence
 ↓
Analysis version
 ↓
Result
```

## 50. Runtime Snapshots

A snapshot may capture:

```text
configuration
policy
health
resource state
active workflows
versions
```

Snapshots are useful for post-incident reconstruction.

## 51. Incident Timeline

Observability should support:

```text
T0: configuration change
T1: dependency degraded
T2: queue growth
T3: worker restart
T4: recovery
```

The timeline should preserve uncertainty where ordering cannot be established precisely.

## 52. Root-Cause Analysis

Telemetry can support hypotheses:

```text
Observation
 ↓
Correlation
 ↓
Hypothesis
 ↓
Validation
```

Correlation alone is not proof of root cause.

## 53. Causal Graph

A diagnostic system may represent:

```text
Config change
      ↓
Resource pressure
      ↓
Timeouts
      ↓
Retries
      ↓
Overload
```

Edges should identify whether they are observed, inferred, or model-derived.

## 54. Anomaly Detection

Anomaly detection may identify deviations from expected behavior:

```text
Baseline
 ↓
Deviation
 ↓
Alert
```

An anomaly is a signal for investigation, not necessarily a fault.

## 55. Alerting

Alerts should be actionable:

```text
condition
severity
scope
first observed
last observed
owner
recommended action
```

## 56. Alert Deduplication

Repeated manifestations of one underlying fault should not create uncontrolled alert storms.

```text
One cause
 ↓
Many symptoms
 ↓
Correlated alert
```

## 57. Severity

Severity should have explicit semantics, for example:

```text
info
warning
error
critical
```

Severity is not the same as confidence or evidence strength.

## 58. Confidence

A diagnostic conclusion may include confidence:

```text
hypothesis
confidence
supporting evidence
contradicting evidence
```

Confidence must not be confused with certainty.

## 59. Diagnostic State Machine

```text
Unknown
 ↓
Observed
 ↓
Investigating
 ↓
Correlated
 ↓
Validated
 ↓
Resolved
```

Not every incident reaches validated root cause.

## 60. Health and Telemetry Failure

If telemetry fails while workload continues:

```text
Workload → running
Telemetry → degraded
```

The architecture must define whether workload execution continues, degrades, or stops for critical observability dependencies.

## 61. Observability of the Observability Plane

Telemetry infrastructure itself needs:

```text
health
queue depth
loss rate
latency
storage capacity
collector failures
```

## 62. Telemetry Feedback Loops

Automated control may consume telemetry:

```text
Metric
 ↓
Policy
 ↓
Action
 ↓
New Metric
```

Control loops must be bounded to prevent oscillation and runaway reactions.

## 63. Metrics for Control

Metrics used for automatic control should have:

```text
defined semantics
bounded delay
known sampling behavior
failure handling
```

## 64. Missing Telemetry

Absence of a signal means:

```text
No observation
```

not automatically:

```text
No event occurred
```

This distinction is fundamental.

## 65. Evidence Freshness

Evidence can become stale:

```text
Evidence at T0
 ↓
Current state at T1
```

A claim must specify the temporal relationship between evidence and the property being claimed.

## 66. Evidence Scope

Evidence should identify its scope:

```text
task
workflow
agent
node
tenant
cluster
```

Evidence about one component cannot automatically establish a global property.

## 67. Evidence Completeness

Evidence claims should identify whether collection was:

```text
complete
partial
sampled
unknown
```

## 68. Evidence Integrity

Where evidence supports security or governance claims, integrity mechanisms should prevent undetected alteration.

## 69. Provenance Graph

```text
Source Event
 ↓
Derived Metric
 ↓
Diagnostic Result
 ↓
Claim
```

Derived artifacts should retain links to their inputs where feasible.

## 70. Explainability Boundary

Explainability should reveal operationally relevant provenance without exposing secrets or violating isolation boundaries.

## 71. Explainability and Agent Actions

An agent action should be explainable through:

```text
observed context
constraints
selected operation
tool/result status
policy/version context
outcome
```

The record should not imply that a textual explanation is a complete causal account of internal computation.

## 72. Diagnostic Export

An incident bundle may contain:

```text
timeline
logs
metrics
traces
snapshots
configuration versions
policy versions
restart history
evidence metadata
```

Sensitive material remains subject to access and redaction policy.

## 73. Evidence Rehydration

An exported diagnostic bundle should retain enough metadata to identify:

```text
source
schema
version
time basis
integrity status
```

## 74. Observability Contracts

Components should define what they guarantee to expose:

```text
required events
required metrics
health signals
error categories
correlation identifiers
```

Undocumented observability should not be treated as a stable API.

## 75. Minimum Diagnostic Contract

A critical component should expose at least, where applicable:

```text
identity
version
health
state
active work
resource pressure
last failure
configuration epoch
policy epoch
```

## 76. Sampling and Safety

Sampling must never silently remove mandatory security or audit events.

Critical telemetry classes require explicit loss policy.

## 77. Observability Cost Control

Observability policies should define:

```text
rate limits
sampling
retention
aggregation
storage quotas
export limits
```

## 78. Observability During Overload

During resource pressure, NROS should preserve the telemetry needed for:

```text
containment
recovery
security
control-plane operation
```

Lower-value diagnostics may be sampled or dropped first.

## 79. Observability During Recovery

Recovery actions should themselves be observable:

```text
fault detected
restart
recovery start
reconciliation
validation
admission
```

## 80. Observability During Rollout

Part XXXIX changes should emit sufficient signals to evaluate rollout gates:

```text
cohort
version
health
error rate
resource impact
invariant status
```

## 81. Formal Observation Invariant

```text
Observed(Event)
    ⇒
Source(Event) ∧ TimeContext(Event)
```

where those fields are required by the event contract.

## 82. Formal Evidence Invariant

```text
Evidence(Claim)
    ⇒
EvidenceScope ⊇ ClaimScope
```

and the evidence must be relevant to the claimed property.

## 83. Formal Sampling Invariant

```text
Sampled(Stream)
    ⇒
Completeness(Stream) = NotGuaranteed
```

unless an explicit completeness guarantee exists.

## 84. Formal Diagnostic Invariant

```text
DiagnosticConclusion
    ⇒
SupportingObservations are identifiable
```

where the diagnostic contract requires evidence provenance.

## 85. Formal Telemetry Security Invariant

```text
TelemetryAccess
    ⇒
Authorized(Actor, Scope, Purpose)
```

## 86. Verification Matrix

| Property | Verification question |
|---|---|
| Identity | Can observations be attributed to a source? |
| Time | Is temporal context explicit? |
| Ordering | Can sequence gaps and uncertainty be detected? |
| Correlation | Can related execution be reconstructed? |
| Health | Are liveness, readiness, and progress distinct? |
| Evidence | Is evidence separated from proof? |
| Integrity | Can critical evidence alteration be detected? |
| Sampling | Is incomplete telemetry clearly identified? |
| Privacy | Are sensitive fields minimized and protected? |
| Isolation | Are tenant and capability boundaries preserved? |
| Retention | Are lifecycle policies explicit? |
| Cost | Is telemetry resource usage bounded? |
| Diagnostics | Can incidents be reconstructed? |
| Provenance | Can derived claims trace to source observations? |
| Explainability | Are decisions operationally attributable? |
| Control loops | Are telemetry-driven actions bounded? |
| Recovery | Are recovery actions observable? |
| Rollout | Can change gates consume trustworthy signals? |
| Formal assurance | Are evidence-scope invariants explicit? |

## 87. What Part XL Does Not Claim

This Part does not claim that the current NROS implementation already has:

- complete distributed tracing;
- universal structured logging;
- production-grade telemetry storage;
- tamper-evident evidence storage;
- complete incident reconstruction;
- universal anomaly detection;
- complete agent decision provenance;
- formally verified observability semantics;
- complete privacy-preserving telemetry pipelines.

Those require implementation-specific evidence.

## 88. Transition to Part XLI

Part XL establishes observability and evidence.

Part XLI should define **security architecture, trust boundaries, identity, authentication, authorization, capabilities, secrets, isolation enforcement, secure communication, threat modeling, and security invariants**, connecting the observable runtime with explicit trust and authority semantics.

```text
Part XXXIX
Configuration + control + reconfiguration + rollout
        ↓
Part XL
Observability + telemetry + traces + evidence + diagnostics
        ↓
Part XLI
Security + identity + trust + authorization + capabilities
```

## Canonical rule

> **NROS makes runtime behavior observable and attributable while preserving the distinction between telemetry, evidence, and proof; observability must itself remain bounded, secure, privacy-aware, integrity-aware, and subordinate to the same resource, authority, and isolation constraints as the workload it observes.**
