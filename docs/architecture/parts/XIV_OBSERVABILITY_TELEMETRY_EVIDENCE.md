# Part XIV — Observability, Telemetry & Evidence

> **Series:** NROS Architecture Series  
> **Part:** XIV  
> **Role:** Logs, metrics, traces, diagnostics, health, telemetry, correlation, evidence, and verification  
> **Status:** Architectural design document — not implementation evidence

## 1. Purpose

Part XIII defined dataflow and flow control. Part XIV defines how NROS observes runtime behavior, records evidence, correlates events, exposes diagnostics, and distinguishes observation from verification.

The central rule is:

> **NROS treats observability as evidence production, not as proof by itself; an observation becomes verification evidence only when its source, scope, semantics, and acceptance criteria are defined.**

## 2. Observability Model

```text
Runtime behavior
      ↓
Observation
      ↓
Telemetry signal
      ↓
Record / aggregate
      ↓
Correlation
      ↓
Analysis
      ↓
Verification / diagnosis / operations
```

Observability should make relevant runtime behavior reconstructable without requiring every internal detail to be exposed.

## 3. Signals

The primary signal classes are:

```text
Logs
Metrics
Traces
Events
Profiles
Health status
Diagnostics
State snapshots
```

Each serves a different purpose.

## 4. Logs

A log records a discrete textual or structured observation.

```text
LogRecord
├── timestamp
├── severity
├── source
├── entity
├── generation
├── message / fields
└── correlation
```

Structured fields are preferred where machines must analyze the record.

## 5. Metrics

Metrics represent measured or aggregated quantities.

Examples:

```text
queue_depth
activation_latency
cpu_time
memory_usage
message_rate
fault_count
deadline_misses
recovery_attempts
```

A metric must define its unit, aggregation semantics, and measurement scope.

## 6. Counters

Counters represent cumulative occurrences where appropriate.

```text
faults_total = 37
```

Counter reset semantics must be explicit, especially across process restarts or generation changes.

## 7. Gauges

Gauges represent current or sampled values:

```text
queue_depth = 12
memory_used = 128 MiB
```

A gauge does not necessarily imply that the value is continuously observed.

## 8. Histograms

Latency and size distributions may require histograms rather than averages:

```text
latency
├── p50
├── p90
├── p99
└── max
```

An average alone can hide tail behavior that matters for temporal guarantees.

## 9. Traces

A trace represents causally related execution activity.

```text
Trace
└── Span A
    ├── Span B
    └── Span C
```

Trace context should propagate across supported communication boundaries where causality must be reconstructed.

## 10. Span Semantics

A span may represent:

```text
message handling
scheduler activation
storage operation
RPC
recovery action
resource acquisition
configuration application
```

A span's timing is evidence of observed execution, not automatically a formal timing guarantee.

## 11. Events

Runtime events provide structured state-transition or occurrence records.

Examples:

```text
ActivationStarted
MessageDelivered
ResourceGranted
FaultDetected
RecoveryStarted
ConfigurationApplied
```

Events should carry stable identifiers where correlation or replay requires them.

## 12. Health

Health represents an operational assessment according to a defined health contract.

```text
Health
├── status
├── checks
├── timestamp
├── generation
└── evidence references
```

Health status is policy-derived and should not be confused with raw telemetry.

## 13. Readiness vs Health

```text
Ready
  ≠
Healthy
```

An entity may be ready to accept work while operating in a degraded condition, depending on its contract.

## 14. Diagnostics

Diagnostics provide tools or information intended to explain runtime behavior.

Possible diagnostic surfaces:

```text
state dump
queue inspection
resource snapshot
fault report
trace query
configuration view
topology view
health report
```

Diagnostic access is subject to Part XI authorization rules.

## 15. Correlation

Observability records should support correlation across subsystems.

Useful identifiers include:

```text
trace_id
span_id
correlation_id
request_id
message_id
activation_id
fault_id
recovery_id
entity_id
generation
```

Correlation identifiers should have documented scope and uniqueness assumptions.

## 16. Causality

Correlation does not automatically prove causality.

```text
A occurred before B
      ≠
A caused B
```

Causal relationships should be established from explicit runtime relationships, trace context, or other evidence.

## 17. Time

Telemetry timestamps must specify their clock semantics.

```text
wall-clock time
monotonic time
logical sequence
```

Latency measurement should generally use an appropriate monotonic clock rather than assuming wall-clock subtraction is safe.

## 18. Sampling

High-volume telemetry may require sampling:

```text
100000 events
      ↓ sampling
1000 retained records
```

Sampling policy must preserve critical events that cannot safely be discarded.

## 19. Aggregation

Telemetry may be aggregated:

```text
raw events
   ↓
window
   ↓
count / rate / histogram
```

Aggregation loses information and therefore must not be mistaken for the original evidence when the original detail is required.

## 20. Retention

Telemetry retention requires explicit policy:

```text
hot retention
cold retention
archive
expiration
delete
```

Retention must consider incident investigation, verification, storage limits, and security requirements.

## 21. Redaction

Sensitive data may need redaction before telemetry leaves a trust boundary.

Possible categories:

```text
credentials
secret values
personal data
security tokens
private payloads
sensitive topology
```

Redaction should preserve enough metadata for diagnosis without exposing protected content.

## 22. Telemetry Backpressure

Telemetry itself can overload a system.

```text
Runtime
  ↓
Telemetry producer
  ↓
Buffer
  ↓
Exporter
```

The system needs an explicit policy for overload:

```text
sample
aggregate
drop low-priority telemetry
block
spill
fail exporter
```

Critical runtime execution should not become indefinitely blocked by noncritical telemetry unless the system contract explicitly requires synchronous audit evidence.

## 23. Telemetry Isolation

Observability should be isolated sufficiently that telemetry failure does not automatically become application failure.

Possible mechanisms:

```text
separate queues
bounded buffers
independent worker
resource quotas
priority classes
```

## 24. Evidence Levels

Evidence can be classified by strength:

```text
UNOBSERVED
OBSERVED
RECORDED
CORRELATED
MEASURED
REPRODUCED
VERIFIED
```

The exact vocabulary may evolve, but architectural documents must not collapse these levels.

## 25. Verification Evidence

A verification record should identify:

```text
claim
requirement
test / measurement
inputs
environment
observed result
acceptance criteria
artifact
timestamp
commit / version
```

This allows a claim to be traced back to concrete evidence.

## 26. Claim vs Evidence

The architecture explicitly separates:

```text
Claim
  ≠
Implementation
  ≠
Observation
  ≠
Test result
  ≠
Verification
```

A design document cannot substitute for execution evidence.

## 27. Deterministic Diagnostics

Diagnostics should prefer deterministic representations where possible.

For example:

```text
same runtime state
      ↓
canonical diagnostic representation
```

This improves comparison, regression analysis, and automated verification.

## 28. State Snapshots

Diagnostic snapshots may capture:

```text
lifecycle state
entity generation
scheduler state
resource usage
active dependencies
configuration version
channel state
fault state
```

Snapshots are point-in-time observations and may become stale immediately after capture.

## 29. Profiling

Profiling can identify resource or execution behavior:

```text
CPU
memory
I/O
scheduler
allocation
lock contention
message processing
```

Profiles are observational artifacts and should not be interpreted as universal performance guarantees.

## 30. Deadline and Budget Evidence

Parts VI–VIII require telemetry for temporal/resource claims:

```text
release time
start time
completion time
deadline
execution time
budget
consumption
miss reason
```

A claimed deadline guarantee requires evidence appropriate to the guarantee being claimed.

## 31. Fault and Recovery Evidence

Parts IX–XII can emit:

```text
FaultDetected
ContainmentStarted
RecoveryStarted
RecoveryAttempted
RecoveryVerified
RecoveryFailed
CheckpointCreated
StateRestored
StateCorruptionDetected
```

This allows incident reconstruction across fault and persistence boundaries.

## 32. Dataflow Evidence

Part XIII dataflow can expose:

```text
published
accepted
delivered
processed
acknowledged
dropped
expired
replayed
queue_full
backpressure
```

These events help distinguish delivery failure from processing failure.

## 33. Configuration Evidence

Configuration operations should produce evidence:

```text
configuration loaded
validation passed/failed
configuration applied
rollback
configuration version changed
```

Sensitive values must remain protected.

## 34. Discovery Evidence

Discovery can expose:

```text
registration
lease renewal
lease expiration
resolution attempt
resolution success/failure
endpoint change
capability change
```

This supports reconstruction of dynamic topology.

## 35. Security Evidence

Part XI security decisions should be observable without exposing secrets:

```text
authentication success/failure
authorization allow/deny
credential lifecycle event
capability grant/revoke
policy version
security boundary transition
```

## 36. Export

Telemetry may be exported to external systems:

```text
local collector
file
remote collector
metrics backend
trace backend
log backend
```

Export semantics should define buffering, retry, security, and failure behavior.

## 37. Offline Evidence

Evidence may need to remain useful when external observability infrastructure is unavailable.

Possible mechanisms:

```text
local ring buffer
persistent incident journal
checkpoint-linked diagnostics
bounded local storage
```

## 38. Evidence Integrity

Where evidence is used for verification or audit, integrity mechanisms may include:

```text
hashes
sequence numbers
authenticated records
signed artifacts
immutable storage
```

Integrity mechanisms must match the required threat model.

## 39. Observability and Performance

Instrumentation has cost.

```text
More telemetry
    ↓
more CPU / memory / I/O
```

Instrumentation overhead should therefore be measurable and controlled.

## 40. Verification Matrix

| Property | Verification question |
|---|---|
| Signals | Are logs, metrics, traces, and events semantically distinct? |
| Metrics | Are units and aggregation scopes defined? |
| Timing | Are clock semantics appropriate for latency measurements? |
| Correlation | Can related runtime operations be reconstructed? |
| Causality | Are causal claims supported beyond mere temporal correlation? |
| Sampling | Are critical records protected from inappropriate sampling? |
| Retention | Is evidence retained for its required lifetime? |
| Redaction | Are sensitive values protected? |
| Backpressure | Can telemetry overload be bounded? |
| Isolation | Can telemetry failure avoid destabilizing runtime execution? |
| Evidence | Can claims be traced to concrete artifacts? |
| Verification | Are acceptance criteria explicit? |
| Reproducibility | Is environment/version information retained? |
| Integrity | Can critical evidence tampering be detected? |
| Performance | Is instrumentation overhead measurable? |

## 41. What Part XIV Does Not Claim

This Part does not claim that the current NROS implementation already provides:

- complete distributed tracing;
- universal metrics collection;
- tamper-proof audit logs;
- lossless telemetry under arbitrary overload;
- deterministic observability overhead;
- automatic root-cause analysis;
- formal verification from telemetry alone;
- permanent evidence retention.

Those properties require implementation and verification evidence.

## 42. Transition to Part XV

Part XIV defines observability and evidence.

Part XV should define **deployment, composition, isolation, and multi-node execution**, connecting runtime entities to actual hosts, processes, containers, devices, and distributed deployments.

```text
Part XIII
Dataflow + flow control
        ↓
Part XIV
Observability + evidence
        ↓
Part XV
Deployment + composition + isolation
```

## Canonical rule

> **NROS treats observability as structured evidence production and keeps observation, measurement, correlation, verification, and proof distinct; telemetry must remain bounded, secure, and sufficiently reproducible for the claims it supports.**
