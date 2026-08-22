# NROS Advanced Execution (Part CXXI–CXXX)

The Agent is the execution-side authority through which NROS interacts with real compute, devices, processes, and local resources.

An Agent is not merely a worker process.

It is a stateful participant with:

```text
identity
incarnation
authority
capabilities
resources
health
leases
executions
observations
```

The Controller must therefore distinguish:

```text
Agent exists
Agent is connected
Agent is authorized
Agent is healthy
Agent is capable
Agent is current
Agent is executing
```

These are different claims.

# 1. Agent Identity

Every Agent requires a stable logical identity:

```text
agent_id
```

The identity should survive ordinary process restarts.

# 2. Agent Incarnation

Every process/session incarnation receives a new identifier:

```text
agent_id = A42
incarnation = I17
```

After restart:

```text
agent_id = A42
incarnation = I18
```

This prevents stale messages from incarnation I17 being accepted by I18.

# 3. Agent Identity vs Session Identity

These must remain separate:

```text
Agent identity
    = durable participant

Session identity
    = current communication channel
```

A reconnect can create a new session without creating a new Agent.

# 4. Agent Registration

An Agent should explicitly register before becoming schedulable.

Conceptually:

```text
REGISTER
   ↓
AUTHENTICATE
   ↓
VALIDATE
   ↓
CAPABILITY_DISCOVERY
   ↓
RESOURCE_DISCOVERY
   ↓
READY
```

# 5. Registration Payload

Registration may include:

```text
agent_id
incarnation
software_version
protocol_version
capabilities
resources
labels
topology
health
supported_features
```

# 6. Registration Is Not Readiness

Successful registration does not necessarily mean:

```text
READY
```

The Agent may still require:

```text
resource validation
runtime initialization
security verification
health checks
```

# 7. Agent Lifecycle

A canonical lifecycle:

```text
UNKNOWN
   ↓
REGISTERING
   ↓
AUTHENTICATING
   ↓
VALIDATING
   ↓
READY
   ↓
DRAINING
   ↓
OFFLINE
```

Failure paths may include:

```text
SUSPECTED
FAILED
QUARANTINED
REVOKED
```

# 8. Ready State

An Agent should enter `READY` only when the Controller has sufficient evidence that it can safely receive Work.

# 9. Schedulability

The scheduler should derive:

```text
schedulable =
authorized
∧ current_incarnation
∧ healthy
∧ ready
∧ not_draining
∧ resources_valid
```

# 10. Capability Model

Capabilities describe what the Agent can execute.

Examples:

```text
linux
rust
wasm
gpu
container
shell
device.serial
```

# 11. Capability Identity

Each capability should have a stable identifier.

Avoid relying solely on human-readable descriptions.

# 12. Capability Version

Some capabilities are versioned:

```text
wasm.runtime = 2.4
```

# 13. Capability Constraints

A capability may expose constraints:

```text
gpu.compute >= 8.0
memory >= 16GiB
runtime = wasm
```

# 14. Capability Provenance

The Controller should know how a capability was established:

```text
declared
discovered
verified
attested
```

# 15. Declared Capability

An Agent can claim:

```text
capability = docker
```

But declaration alone may not be sufficient for high-trust scheduling.

# 16. Verified Capability

The Controller may require an actual validation:

```text
probe
→ capability confirmed
```

# 17. Capability Revocation

If a runtime disappears:

```text
docker capability
→
INVALID
```

The Agent should no longer receive Work requiring it.

# 18. Capability Freshness

Capabilities can become stale.

A capability record should have validity information:

```text
observed_at
expires_at
verification_state
```

where appropriate.

# 19. Resource Model Integration

Capabilities answer:

> What can this Agent do?

Resources answer:

> How much capacity does it currently have?

The scheduler requires both.

# 20. Agent Labels

Agents may expose labels:

```text
region=eu-west
zone=3
environment=production
hardware=arm64
```

Labels support placement policies.

# 21. Labels vs Capabilities

Do not conflate:

```text
label = descriptive metadata
capability = executable ability
```

# 22. Topology

An Agent may expose:

```text
region
zone
rack
host
NUMA topology
device topology
```

# 23. Topology Freshness

Topology changes should invalidate stale scheduling assumptions.

# 24. Resource Discovery

During registration, the Agent should report:

```text
CPU
memory
storage
GPU
devices
network
custom resources
```

# 25. Resource Verification

The Controller may validate reported resources against policy.

For example:

```text
declared CPU = 16
observed CPU = 16
```

# 26. Resource Drift

After registration:

```text
resource capacity
```

may change.

The Agent should therefore publish resource updates.

# 27. Resource Update

Example:

```text
ResourceChanged {
    resource_id
    previous_capacity
    new_capacity
    reason
}
```

# 28. Resource Update Authority

The Controller should determine whether Agent-reported capacity is:

```text
authoritative
advisory
verified
```

based on deployment trust.

# 29. Agent Health

Health should be multidimensional.

Possible dimensions:

```text
process health
transport health
resource health
execution health
system health
```

# 30. Process Health

Is the Agent process alive and responsive?

# 31. Transport Health

Can the Controller communicate with it?

# 32. Resource Health

Are its advertised resources actually usable?

# 33. Execution Health

Are active executions behaving correctly?

# 34. System Health

Is the underlying machine under severe pressure or otherwise unsafe?

# 35. Composite Health

A simple composite state might be:

```text
HEALTHY
DEGRADED
UNHEALTHY
UNKNOWN
```

But individual health dimensions should remain available.

# 36. Heartbeat

The Agent periodically emits a heartbeat.

Example:

```text
Heartbeat {
    agent_id
    incarnation
    session_id
    authority_epoch
    timestamp
    health
    resource_summary
}
```

# 37. Heartbeat Semantics

Heartbeat proves recent communication.

It does not by itself prove:

```text
resource correctness
execution correctness
command completion
```

# 38. Heartbeat Timeout

A missed heartbeat should transition through suspicion rather than immediately assuming permanent failure.

# 39. Suspicion State

```text
READY
 ↓
SUSPECTED
 ↓
UNREACHABLE
 ↓
FAILED
```

depending on configured detection policy.

# 40. False Positives

Network congestion can cause heartbeat loss without Agent failure.

The failure detector must account for this possibility.

# 41. Heartbeat Sequence

Heartbeats can include:

```text
heartbeat_sequence
```

to detect stale or reordered heartbeat messages.

# 42. Agent Lease

An Agent may maintain a lease with the Controller.

Conceptually:

```text
lease_expires_at
```

# 43. Lease Renewal

The Agent periodically renews:

```text
RENEW_LEASE
```

# 44. Lease Expiration

If renewal stops:

```text
lease expires
```

The Controller can stop considering the Agent authoritative.

# 45. Lease vs Heartbeat

Heartbeat:

> The Agent is communicating.

Lease:

> The Agent currently retains authority to participate.

These are related but not identical.

# 46. Agent Epoch

The Controller can assign an authority epoch:

```text
epoch = 72
```

A new registration or leadership transition may produce:

```text
epoch = 73
```

# 47. Stale Agent Protection

Messages from:

```text
epoch 72
```

should not mutate state governed by:

```text
epoch 73
```

when fencing semantics require it.

# 48. Agent Command Authorization

Before executing a command, the Agent should validate:

```text
sender authority
agent identity
incarnation
epoch
command identity
authorization
```

# 49. Command Targeting

A command should identify its intended target:

```text
target_agent_id
target_incarnation
```

where necessary.

# 50. Stale Command

A command intended for:

```text
Agent A42 / incarnation I17
```

must not accidentally execute on:

```text
Agent A42 / incarnation I18
```

unless the protocol explicitly allows migration.

# 51. Execution Namespace

An Agent should maintain an execution namespace:

```text
execution_id
```

Each execution should be uniquely identifiable.

# 52. Execution Identity

An execution should reference:

```text
work_id
attempt
agent_id
incarnation
execution_id
```

# 53. Attempt Identity

Retries should produce distinct attempts:

```text
Work W42
Attempt 1
Attempt 2
Attempt 3
```

# 54. Execution Lifecycle

A canonical lifecycle:

```text
CREATED
   ↓
DISPATCHED
   ↓
ACCEPTED
   ↓
STARTING
   ↓
RUNNING
   ↓
TERMINATING
   ↓
COMPLETED
```

Failure states:

```text
FAILED
CANCELLED
TIMED_OUT
UNKNOWN
LOST
```

# 55. Agent Acceptance

When an Agent accepts a command:

```text
execution record
```

should be established before claiming that execution is active.

# 56. Start Evidence

`STARTED` should correspond to meaningful evidence.

For example:

```text
process created
execution supervisor active
```

# 57. Running Evidence

A heartbeat saying:

```text
Agent alive
```

does not necessarily prove a particular execution remains alive.

Execution-specific evidence is preferable.

# 58. Completion Evidence

Completion should include:

```text
exit status
termination reason
finished_at
output/artifact references
```

where applicable.

# 59. Execution Result

Possible result categories:

```text
SUCCESS
FAILED
CANCELLED
TIMED_OUT
RESOURCE_LOST
UNKNOWN
```

# 60. Unknown Result

If the Agent disappears after starting an execution:

```text
result = UNKNOWN
```

may be the only correct immediate state.

# 61. Unknown Is Not Failure

NROS must not automatically transform:

```text
UNKNOWN
```

into:

```text
FAILED
```

without policy or evidence.

# 62. Reconciliation

Reconciliation attempts to resolve uncertainty.

Sources may include:

```text
Agent reconnect
persistent Agent journal
external process supervisor
resource state
execution artifact
```

# 63. Reconciliation Priority

Evidence should be ranked according to authority.

For example:

```text
durable execution record
>
authenticated Agent state
>
local observation
>
inferred state
```

The exact hierarchy depends on architecture.

# 64. Agent Reconnect

After reconnect, the Agent should report:

```text
current incarnation
active executions
completed executions
resource state
lease state
```

# 65. Reconciliation Handshake

Conceptually:

```text
Controller
   ↓
RECONCILE_REQUEST
   ↓
Agent
   ↓
RECONCILE_REPORT
   ↓
Controller
   ↓
state correction
```

# 66. Reconciliation Report

A report can include:

```text
agent_id
incarnation
epoch
execution_ids
execution_states
resource_state
command_results
last_sequence
```

# 67. Missing Execution

If Controller expects:

```text
E42
```

but Agent reports no such execution:

```text
E42 = missing
```

the Controller must apply explicit reconciliation policy.

# 68. Unexpected Execution

If Agent reports:

```text
E99
```

but Controller has no corresponding authoritative record:

```text
E99 = unexpected
```

The Agent should not automatically treat it as legitimate.

# 69. Orphan Execution

An execution without valid Controller authority is an orphan.

Policy may require:

```text
quarantine
terminate
inspect
adopt
```

# 70. Adoption

If NROS permits adoption, it must be explicit and auditable.

It must never occur merely because an execution happens to exist on an Agent.

# 71. Agent Drain

Draining means:

```text
no new Work
```

while existing Work may continue.

# 72. Drain Initiation

Reasons include:

```text
maintenance
software upgrade
resource degradation
operator action
shutdown
```

# 73. Drain Completion

An Agent can become fully offline after:

```text
active executions = 0
reservations released
commands settled
```

or after explicit force policy.

# 74. Forced Shutdown

Forced shutdown may terminate active Work.

The resulting execution states must remain explicit.

# 75. Agent Revocation

Revocation means:

> The Agent is no longer trusted to participate.

This is stronger than ordinary offline status.

# 76. Revocation Consequences

A revoked Agent should:

```text
receive no new commands
lose scheduling eligibility
have authority invalidated
require explicit re-registration
```

# 77. Quarantine

Quarantine is useful when the Agent is suspicious but not conclusively compromised or invalid.

# 78. Agent Software Version

The Agent should report:

```text
software_version
build_id
protocol_version
```

# 79. Version Policy

The Controller may require:

```text
minimum_agent_version
```

for certain Work.

# 80. Feature Capability

Instead of depending solely on version numbers:

```text
supports(feature_x)
```

is often more robust.

# 81. Agent Configuration

Configuration affecting execution semantics should be identifiable:

```text
configuration_version
```

# 82. Agent Policy Version

Security or scheduling policy may also have versions:

```text
policy_version
```

This makes historical decisions reproducible.

# 83. Agent Attestation

High-trust deployments may require evidence that the Agent is running an approved software/hardware configuration.

The architecture should represent:

```text
attestation_state
```

without assuming every deployment needs remote attestation.

# 84. Trust State

Possible states:

```text
UNTRUSTED
PENDING
TRUSTED
DEGRADED
REVOKED
```

# 85. Trust vs Health

An Agent can be:

```text
healthy but untrusted
```

or:

```text
trusted but currently unhealthy
```

These dimensions must remain separate.

# 86. Agent Resource Reservation

When Controller assigns Work:

```text
reservation
→
command
→
execution
```

The Agent should not independently create conflicting Controller-authoritative reservations.

# 87. Local Resource Manager

The Agent may maintain a local resource manager for execution safety.

It should enforce:

```text
local capacity
exclusive ownership
process limits
device access
```

# 88. Controller vs Agent Authority

The Controller generally owns global scheduling decisions.

The Agent owns local execution enforcement.

This creates a deliberate division:

```text
Controller:
global authority

Agent:
local enforcement
```

# 89. Local Enforcement

Even if Controller mistakenly sends:

```text
CPU = 1000
```

the Agent should reject an impossible local allocation.

# 90. Defense in Depth

The Agent should validate locally:

```text
resource limits
command authorization
execution identity
target identity
local safety constraints
```

# 91. Agent Journaling

For recovery, the Agent may maintain a local durable journal:

```text
command accepted
execution started
execution completed
```

This is particularly valuable during Controller disconnection.

# 92. Local Journal Authority

The Agent journal is evidence about local execution.

It does not automatically override Controller authority.

# 93. Journal Sequence

Local events should have monotonically increasing sequence numbers.

# 94. Replaying Agent Journal

After reconnect:

```text
Controller asks:
"Give me events after sequence 500."
```

The Agent can replay:

```text
501
502
503
```

# 95. Journal Retention

Retention should cover the maximum expected Controller outage plus reconciliation window.

# 96. Agent State Snapshot

The Agent may periodically snapshot:

```text
active executions
resource state
journal sequence
```

to accelerate local recovery.

# 97. Agent Crash Recovery

After Agent restart:

```text
load journal
→ reconstruct executions
→ discover local processes
→ reconcile process state
→ establish new incarnation
→ reconnect Controller
```

# 98. Process Discovery

If an execution supervisor survives Agent process restart, the new incarnation may rediscover the execution.

This must be explicitly modeled.

# 99. Process Ownership

Every managed process should carry enough identity to associate it with:

```text
agent_id
execution_id
attempt
```

where practical.

# 100. Orphan Process Protection

A process that cannot be associated with valid authority should enter explicit policy:

```text
ORPHANED
```

rather than being silently adopted.

# 101. Agent Metrics

Expose:

```text
heartbeat_age
lease_age
active_executions
resource_utilization
command_queue_depth
reconciliation_state
journal_lag
```

# 102. Agent Audit Events

Useful events:

```text
AgentRegistered
AgentAuthenticated
AgentValidated
AgentReady
AgentDraining
AgentDisconnected
AgentSuspected
AgentFailed
AgentRevoked
AgentReconciled
AgentRecovered
```

# 103. Agent Invariants

```text
1. Agent identity is stable across ordinary restarts.

2. Every process incarnation has a unique incarnation identity.

3. Session identity is distinct from Agent identity.

4. Registration precedes schedulability.

5. Registration does not automatically imply readiness.

6. Capability and resource information are distinct.

7. Capability provenance is explicit.

8. Capability freshness is bounded where required.

9. Resource state can change after registration.

10. Agent health is multidimensional.

11. Heartbeat proves communication, not execution correctness.

12. Lease and heartbeat semantics remain distinct.

13. Authority epochs fence stale participants.

14. Commands target explicit identities where required.

15. Execution attempts have unique identities.

16. Unknown execution outcomes remain UNKNOWN until resolved.

17. Communication loss does not automatically mean execution failure.

18. Reconciliation is explicit.

19. Unexpected executions are not silently adopted.

20. Orphan executions follow explicit policy.

21. Draining prevents new Work.

22. Revocation invalidates participation authority.

23. Trust and health remain separate dimensions.

24. Agent-local enforcement protects local resource safety.

25. Controller retains global scheduling authority.

26. Agent retains local execution enforcement.

27. Local journals provide execution evidence.

28. Local evidence does not silently override global authority.

29. Agent restart creates a new incarnation.

30. Recovery must reconcile surviving executions before normal operation.
```

# 104. Canonical Agent Lifecycle

```text
             REGISTER
                │
                ▼
           AUTHENTICATE
                │
                ▼
             VALIDATE
                │
       ┌────────┴────────┐
       ▼                 ▼
   CAPABILITIES       RESOURCES
       │                 │
       └────────┬────────┘
                ▼
              READY
                │
        ┌───────┴────────┐
        ▼                ▼
     DRAINING         SUSPECTED
        │                │
        ▼                ▼
      OFFLINE          FAILED
                           │
                           ▼
                       RECOVER
                           │
                           ▼
                      NEW INCARNATION
                           │
                           ▼
                      RECONCILE
                           │
                           ▼
                          READY
```

# 105. Canonical Agent Registration

```text
Agent
  │
  ├── identity
  ├── incarnation
  ├── software
  ├── protocol
  ├── capabilities
  ├── resources
  ├── labels
  ├── topology
  └── trust evidence
           │
           ▼
       Controller
           │
           ├── authenticate
           ├── validate
           ├── verify
           └── authorize
           │
           ▼
         READY
```

# 106. Canonical Execution Control

```text
Controller
    │
    ▼
RESERVATION
    │
    ▼
DISPATCH
    │
    ▼
Agent
    │
    ├── validate target
    ├── validate authority
    ├── validate resources
    ├── deduplicate command
    │
    ▼
ACCEPT
    │
    ▼
START
    │
    ▼
RUN
    │
    ▼
COMPLETE
    │
    ▼
REPORT
```

# 107. Canonical Reconciliation

```text
          CONTROLLER
               │
               │ expected state
               ▼
        RECONCILE_REQUEST
               │
               ▼
             AGENT
               │
        ┌──────┼───────┐
        ▼      ▼       ▼
    executions resources journal
        │      │       │
        └──────┼───────┘
               ▼
       RECONCILE_REPORT
               │
               ▼
          COMPARISON
               │
       ┌───────┼────────┐
       ▼       ▼        ▼
     MATCH   MISSING  EXTRA
       │       │        │
       ▼       ▼        ▼
     KEEP    POLICY   POLICY
```

# 108. Agent Decision Rule

Before NROS schedules Work to an Agent, it should be possible to establish:

```text
identity
+
current incarnation
+
authority
+
trust
+
health
+
capability
+
resource capacity
+
placement compatibility
```

# 109. Agent Failure Rule

When an Agent disappears:

```text
do not assume completion
do not assume failure
do not immediately reuse exclusive resources
do not silently discard reservations
```

Instead:

```text
fence
→
classify
→
reconcile
→
recover/retry/fail
```

according to explicit policy.

# 110. Agent Principle

> **An Agent is an independently identifiable, versioned, authority-scoped execution participant—not merely a network connection or worker process.**

# 111. Liveness Principle

> **Liveness evidence establishes communication; execution correctness requires execution-specific evidence.**

# 112. Recovery Principle

> **Agent restart creates a new incarnation, and every stale authority associated with the previous incarnation must be prevented from mutating current state.**

# 113. Reconciliation Principle

> **When Controller and Agent disagree, NROS must resolve the disagreement through explicit authority and evidence rules rather than choosing whichever observation arrived most recently.**

# 114. Final Architectural Rule

> **NROS must treat Agent identity, incarnation, capability, resource capacity, health, trust, lease, execution state, and reconciliation state as separate but composable dimensions, enabling safe scheduling and deterministic recovery across disconnects, crashes, restarts, resource drift, and stale commands.**

The next layer is:

# Part CXXII — Workflow & Dependency Engine, DAG Semantics, Conditions, Gates, Fan-Out/Fan-In, Retries, Compensation, Deadlines & Workflow Recovery

The central question becomes:

> **How does NROS represent and execute multi-step Work while preserving dependency correctness, conditional branching, concurrency, failure propagation, retries, compensation, and durable recovery?**

# NROS — Part CXXII: Workflow & Dependency Engine, DAG Semantics, Conditions, Gates, Fan-Out/Fan-In, Retries, Compensation, Deadlines & Workflow Recovery

The Workflow layer defines how NROS transforms a collection of Work items into an executable, observable, recoverable computation.

A Workflow is not simply a list of tasks.

It is a durable execution graph with:

```text
nodes
dependencies
conditions
resources
attempts
deadlines
outputs
failures
retries
compensation
state
```

The central invariant is:

> **A Work item becomes executable only when all required predecessor semantics permit execution.**

# 1. Workflow Identity

Every Workflow requires:

```text
workflow_id
```

This identity must remain stable across retries, recovery, and execution migration.

# 2. Workflow Definition

A Workflow Definition describes intended behavior.

Conceptually:

```text
WorkflowDefinition {
    workflow_id
    version
    nodes
    dependencies
    policies
    inputs
    outputs
}
```

# 3. Workflow Instance

A Definition is immutable or versioned.

An Instance represents one execution of that Definition.

```text
definition
    ↓
workflow_instance
```

# 4. Definition vs Instance

For example:

```text
Definition:
    build → test → package

Instance:
    workflow/run/842
```

Multiple instances can execute the same Definition simultaneously.

# 5. Workflow Version

Every definition should have an explicit:

```text
definition_version
```

An active instance must remain associated with the exact definition version from which it was created.

# 6. Immutable Execution Semantics

A running Workflow should not silently change because the underlying definition was edited.

Therefore:

```text
Workflow Instance
        ↓
Definition Version N
```

remains stable.

# 7. Workflow Node

A Workflow consists of nodes.

A node represents a logical unit of work.

Examples:

```text
Compile
UnitTest
IntegrationTest
Package
Deploy
Notify
```

# 8. Node Identity

Every node requires a stable identity within the Definition:

```text
node_id
```

The identity should not depend on display names.

# 9. Node Attempt

Execution attempts remain separate from logical node identity.

```text
node_id = test
attempt = 3
```

means:

> The third execution attempt of the same logical node.

# 10. Node State

A canonical state model:

```text
PENDING
BLOCKED
READY
DISPATCHED
RUNNING
SUCCEEDED
FAILED
CANCELLED
SKIPPED
TIMED_OUT
UNKNOWN
```

# 11. Pending

`PENDING` means the node has not yet become eligible for execution.

# 12. Blocked

`BLOCKED` means an explicit dependency, gate, condition, or policy currently prevents execution.

# 13. Ready

`READY` means all required execution prerequisites are satisfied.

# 14. Dispatched

`DISPATCHED` means the Controller has assigned execution to an Agent but execution has not yet been confirmed as started.

# 15. Running

`RUNNING` requires execution-specific evidence.

# 16. Terminal States

Typical terminal states:

```text
SUCCEEDED
FAILED
CANCELLED
SKIPPED
TIMED_OUT
```

`UNKNOWN` is deliberately non-terminal when reconciliation remains possible.

# 17. DAG Model

The simplest Workflow dependency graph is a directed acyclic graph:

```text
A → B → D
     ↘
       C →
```

The graph must not contain cycles unless the Workflow model explicitly introduces controlled looping semantics.

# 18. DAG Validation

A Workflow Definition should be validated before execution.

Required checks include:

```text
unique node IDs
valid references
no illegal cycles
valid dependency types
valid conditions
valid retry policies
valid resource requirements
```

# 19. Cycle Detection

An ordinary DAG Workflow must reject:

```text
A → B
B → C
C → A
```

at definition-validation time.

# 20. Dependency Edge

A dependency is more than:

```text
A must finish before B
```

It may define:

```text
condition
success semantics
data flow
failure propagation
ordering
```

# 21. Dependency Types

NROS can distinguish:

```text
SUCCESS
COMPLETION
FAILURE
ALWAYS
CONDITIONAL
DATA_AVAILABLE
```

depending on the required semantics.

# 22. Success Dependency

```text
A ──success──> B
```

B becomes eligible only if A succeeds.

# 23. Completion Dependency

```text
A ──complete──> B
```

B becomes eligible after A reaches any terminal completion state permitted by policy.

# 24. Failure Dependency

A recovery node may intentionally depend on failure:

```text
A ──failure──> RecoverA
```

# 25. Always Dependency

Some cleanup nodes should execute regardless of predecessor result:

```text
A ──always──> Cleanup
```

# 26. Conditional Dependency

A dependency may evaluate an expression:

```text
A
 │
 └── if output.status == "partial" ──> B
```

# 27. Dependency Evaluation

Eligibility should be evaluated against authoritative Workflow state, not transient in-memory observations alone.

# 28. Readiness Rule

A node is `READY` only when:

```text
all_required_dependencies_satisfied
∧ condition_true
∧ resources_available
∧ workflow_not_cancelled
∧ deadline_not_expired
∧ policy_allows_execution
```

# 29. Dependency Closure

Before executing a node, the scheduler should be able to determine its predecessor closure.

Example:

```text
D
↑
B
↑
A
```

D indirectly depends on A.

# 30. Parallelism

Independent nodes may execute concurrently.

Example:

```text
        ┌→ B ─┐
A ──────┤     ├→ E
        └→ C ─┘
```

B and C can run concurrently if resources permit.

# 31. Fan-Out

One node can unlock multiple nodes:

```text
A
├── B
├── C
└── D
```

# 32. Fan-In

Multiple nodes can converge:

```text
B ─┐
C ─┼→ E
D ─┘
```

# 33. Fan-In Semantics

The Workflow must define whether E requires:

```text
all predecessors
any predecessor
quorum
specific subset
```

# 34. Quorum Dependency

Example:

```text
5 parallel validation nodes
required quorum = 3
```

The downstream node can become eligible after three successful validations if policy permits.

# 35. Any-of Dependency

A race can be represented:

```text
A ─┐
B ─┼→ C
D ─┘
```

where the first successful predecessor unlocks C.

The remaining work must then follow cancellation or continuation policy.

# 36. Race Semantics

Race behavior must be deterministic from the perspective of Workflow state.

If A and B finish simultaneously, NROS must not produce inconsistent downstream activation.

# 37. Atomic Readiness Transition

Transition:

```text
BLOCKED → READY
```

should be persisted atomically with the relevant state version where concurrent schedulers exist.

# 38. Scheduler Concurrency

Multiple scheduler workers may evaluate the same Workflow.

Therefore:

```text
READY
```

must not imply that multiple schedulers can claim the node simultaneously.

# 39. Node Claim

A scheduler may atomically claim:

```text
node_id
execution_attempt
lease
```

before dispatch.

# 40. Work Lease

A node execution may carry a lease:

```text
execution_lease
```

protecting it against stale scheduler ownership.

# 41. Stale Scheduler

If scheduler S1 loses leadership and S2 takes over, S1 must not continue dispatching new attempts under obsolete authority.

# 42. Workflow Epoch

The Workflow instance may maintain:

```text
workflow_epoch
```

for fencing stale execution decisions.

# 43. Workflow State Version

A separate:

```text
state_version
```

can support optimistic concurrency control.

Example:

```text
Workflow W42
state_version = 104
```

Two updates against version 104 cannot both blindly commit.

# 44. Compare-and-Swap

A state transition may require:

```text
expected_version = 104
new_version = 105
```

If another writer already committed version 105, the transition fails and must be recomputed.

# 45. Durable Workflow State

At minimum, authoritative state should preserve:

```text
workflow_id
definition_version
node states
attempts
dependencies
outputs
errors
timestamps
deadline state
cancellation state
```

# 46. Workflow Input

Workflow input should be immutable once execution begins unless the definition explicitly permits dynamic input.

# 47. Workflow Output

Outputs should be versioned and associated with the producing node attempt.

# 48. Output Identity

A useful identity is:

```text
workflow_id
node_id
attempt
output_name
```

# 49. Artifact References

Large outputs should be represented through artifact references rather than embedding arbitrary data directly into Workflow state.

# 50. Data Dependencies

A node may depend on data produced by another node:

```text
Compile
   ↓ artifact
Package
```

The dependency must specify both:

```text
execution ordering
data availability
```

# 51. Missing Output

If a required output is absent, the downstream node must not execute merely because the predecessor is marked successful.

# 52. Output Validation

Required outputs should be validated before declaring a node's contract satisfied.

# 53. Conditional Execution

Conditions can depend on:

```text
node outputs
workflow inputs
metadata
environment
policy
previous results
```

# 54. Condition Language

A condition language must be:

```text
deterministic
side-effect free
bounded
versioned
```

# 55. No Arbitrary Code

Conditions should not execute unrestricted arbitrary code inside the Controller.

This would introduce:

```text
security risk
nondeterminism
resource exhaustion
reproducibility problems
```

# 56. Condition Result

Evaluation should produce:

```text
TRUE
FALSE
UNKNOWN
ERROR
```

rather than only Boolean values when data may be unavailable.

# 57. Unknown Condition

An `UNKNOWN` condition should normally keep a node blocked rather than treating unknown as false.

# 58. Condition Error

A malformed condition should produce a deterministic Workflow error.

# 59. Gates

A Gate is an explicit barrier requiring an external or internal decision.

Examples:

```text
Approval
SecurityReview
MaintenanceWindow
ManualRelease
```

# 60. Gate State

Possible states:

```text
WAITING
APPROVED
REJECTED
EXPIRED
CANCELLED
```

# 61. Manual Approval

A manual gate should record:

```text
actor
timestamp
decision
reason
policy_version
```

# 62. Gate Expiration

An approval should not remain valid forever if the Workflow defines an expiry.

# 63. Gate Idempotency

Repeated approval requests must not produce conflicting Workflow transitions.

# 64. Workflow Cancellation

Cancellation is an explicit Workflow state transition.

It should not simply delete the Workflow.

# 65. Cancellation States

Possible:

```text
CANCEL_REQUESTED
CANCELLING
CANCELLED
```

# 66. Cancellation Propagation

When Workflow cancellation occurs:

```text
pending nodes
→ SKIPPED/CANCELLED
running nodes
→ cancellation requested
```

according to policy.

# 67. Cancellation Race

A node can transition to RUNNING at the same moment cancellation is requested.

The system must define which transition wins and under which state version.

# 68. Force Cancellation

Force cancellation is stronger:

```text
terminate execution
invalidate leases
release resources
```

It may leave the resulting execution state as `CANCELLED` or `UNKNOWN` depending on evidence.

# 69. Retry Policy

A node can have:

```text
max_attempts
backoff
retryable_errors
retry_on_timeout
retry_on_agent_loss
```

# 70. Retry Is Not Re-execution by Default

A retry creates a new attempt:

```text
attempt 1
attempt 2
```

while retaining the logical node identity.

# 71. Retry Eligibility

A failed node should be retried only if:

```text
retry_policy_allows
∧ attempts < max_attempts
∧ workflow_deadline_valid
∧ failure_class_retryable
```

# 72. Exponential Backoff

A policy may use:

```text
delay = base × 2^attempt
```

with a maximum cap and optional jitter.

# 73. Jitter

Jitter prevents synchronized retry storms when many nodes fail simultaneously.

# 74. Retry Budget

A Workflow can have a global retry budget:

```text
max_total_retries
```

preventing pathological retry amplification.

# 75. Retry Amplification

If:

```text
100 nodes
× 5 retries
× 3 downstream retries
```

are permitted without control, one failure can create an enormous execution storm.

# 76. Failure Domains

Retries should consider the failure domain.

Repeatedly retrying on the same failed Agent may be pointless.

# 77. Agent-Aware Retry

Policy may require:

```text
retry on different Agent
```

for infrastructure failures.

# 78. Deterministic Retry Classification

Failures should be classified:

```text
USER_ERROR
APPLICATION_ERROR
RESOURCE_ERROR
AGENT_ERROR
NETWORK_ERROR
TIMEOUT
CANCELLED
UNKNOWN
```

before retry decisions.

# 79. Unknown Failure

Unknown outcomes should not automatically be retried when the operation may have produced irreversible side effects.

Reconciliation may be required first.

# 80. Retry and Idempotency

Before retrying a non-idempotent node:

```text
reconcile previous attempt
```

must be possible or the Workflow must explicitly accept duplicate effects.

# 81. Timeout

Timeouts should exist at multiple levels:

```text
queue_timeout
dispatch_timeout
start_timeout
execution_timeout
node_deadline
workflow_deadline
```

# 82. Node Timeout

A node timeout limits its execution duration.

# 83. Workflow Deadline

The Workflow deadline bounds the complete execution.

# 84. Deadline Propagation

If:

```text
workflow deadline = T
```

then downstream nodes should not receive deadlines beyond T.

# 85. Deadline vs Timeout

A timeout measures duration.

A deadline specifies an absolute cutoff.

They should not be conflated.

# 86. Deadline Race

If a node completes exactly as its deadline expires, the authoritative state transition must follow a deterministic ordering rule.

# 87. Priority

Workflows may have priority:

```text
LOW
NORMAL
HIGH
CRITICAL
```

Priority must not bypass safety or authorization constraints.

# 88. Fairness

A scheduler should avoid starvation of lower-priority Work.

# 89. Quotas

Workflow execution can be constrained by:

```text
CPU quota
memory quota
concurrency quota
agent quota
tenant quota
```

# 90. Concurrency Limit

A Workflow may define:

```text
max_parallel_nodes = N
```

# 91. Global Concurrency

The Controller may additionally impose global limits.

The effective limit is:

```text
min(workflow_limit, tenant_limit, system_limit, resource_limit)
```

# 92. Resource Deadlock

Poor dependency/resource combinations can produce:

```text
nodes waiting for resources
resources reserved by blocked nodes
```

The scheduler must prevent or detect this class of deadlock.

# 93. Reservation Timing

Where possible, resources should not be reserved far earlier than required unless the Workflow explicitly requires reservation.

# 94. Reservation Expiration

Reservations need expiry semantics to prevent permanent leakage after scheduler failure.

# 95. Compensation

Some Workflows require compensating actions after partial success.

Example:

```text
Reserve
  ↓
Deploy
  ↓
Configure
```

If Configure fails:

```text
Compensate Deploy
  ↓
Release Reservation
```

# 96. Compensation Is Not Rollback

A distributed side effect cannot always be rolled back.

Compensation is a new action intended to restore an acceptable state.

# 97. Compensation Node

Compensation should be represented explicitly in the Workflow model.

# 98. Compensation Trigger

Possible triggers:

```text
node failure
workflow cancellation
timeout
operator request
policy violation
```

# 99. Compensation Ordering

Compensations may need reverse dependency order:

```text
A → B → C
```

compensates:

```text
C → B → A
```

where semantics permit.

# 100. Compensation Failure

Compensation can itself fail.

The Workflow must represent:

```text
COMPENSATION_FAILED
```

rather than pretending the original state was restored.

# 101. Recovery Workflow

A failed compensation may require an operator-visible recovery Workflow.

# 102. Saga Semantics

Complex distributed Workflows can follow saga-style semantics:

```text
forward actions
+
compensating actions
```

without requiring global distributed transactions.

# 103. Workflow Failure Policy

A Workflow should define what happens when a node fails:

```text
FAIL_FAST
CONTINUE_INDEPENDENT
RETRY
COMPENSATE
WAIT_FOR_OPERATOR
```

# 104. Fail-Fast

One critical failure can immediately stop downstream execution.

# 105. Continue-Independent

Independent branches may continue even if another branch fails.

# 106. Partial Success

A Workflow may legitimately finish with:

```text
PARTIALLY_SUCCEEDED
```

if the definition permits it.

# 107. Workflow Terminal States

Possible:

```text
SUCCEEDED
FAILED
PARTIALLY_SUCCEEDED
CANCELLED
TIMED_OUT
COMPENSATING
COMPENSATION_FAILED
UNKNOWN
```

# 108. Workflow UNKNOWN

Workflow-level `UNKNOWN` should be used when authoritative state cannot yet establish the final result.

# 109. Workflow Recovery

After Controller restart:

```text
load Workflow state
→ recover scheduler state
→ recover leases
→ inspect running attempts
→ reconcile Agents
→ rebuild ready queue
→ resume
```

# 110. Ready Queue Recovery

`READY` nodes must be reconstructed from durable state rather than trusting an ephemeral in-memory queue.

# 111. Duplicate Dispatch Prevention

After Controller restart, a node marked:

```text
DISPATCHED
```

may have been accepted by an Agent before the crash.

The Controller must reconcile before creating another attempt.

# 112. Scheduler Recovery Rule

```text
DISPATCHED
+
unknown acknowledgement
=
RECONCILE
```

not:

```text
DISPATCHED
+
unknown acknowledgement
=
RETRY IMMEDIATELY
```

# 113. Workflow Event Log

Important transitions should produce durable events:

```text
WorkflowCreated
NodeReady
NodeDispatched
NodeAccepted
NodeStarted
NodeSucceeded
NodeFailed
NodeRetried
NodeSkipped
WorkflowCancelled
WorkflowCompleted
```

# 114. Event-Sourced Reconstruction

If event sourcing is used, Workflow state can be reconstructed by replaying authoritative events.

# 115. Snapshotting

Large Workflows should support snapshots to avoid replaying an unbounded history on every recovery.

# 116. Snapshot Safety

A snapshot must correspond to a known state version.

# 117. Event Ordering

Events affecting one Workflow should have deterministic ordering.

# 118. Workflow Concurrency

Multiple state transitions can occur simultaneously:

```text
Node B succeeds
Node C fails
Cancellation requested
```

The persistence layer must serialize or safely merge these transitions.

# 119. Atomic Transition

A transition should validate:

```text
current state
expected version
transition preconditions
```

before commit.

# 120. Invalid Transition

Example:

```text
SUCCEEDED → RUNNING
```

should be rejected unless explicitly supported by the state machine.

# 121. State Machine

Workflow and node states should be modeled as explicit state machines rather than arbitrary mutable strings.

# 122. State Transition Record

Each transition should be attributable:

```text
previous_state
new_state
actor
reason
causation_id
timestamp
state_version
```

# 123. Actor

The transition actor may be:

```text
controller
scheduler
agent
operator
system_recovery
```

# 124. Workflow Observability

The system should expose:

```text
workflow age
critical path
ready nodes
blocked nodes
running nodes
retry counts
failure classes
resource waits
deadline slack
```

# 125. Critical Path

For DAGs, the scheduler can calculate the dependency critical path.

This identifies nodes whose delay most directly affects completion time.

# 126. Deadline Slack

For a node:

```text
slack = deadline - estimated_remaining_time
```

Low slack can influence scheduling priority.

# 127. Block Reason

A blocked node should expose a structured reason:

```text
WAITING_DEPENDENCY
WAITING_GATE
WAITING_RESOURCE
WAITING_RETRY
WAITING_CONDITION
WAITING_QUOTA
```

# 128. Explainability

Operators should be able to ask:

> Why is node B not running?

and receive machine-readable evidence:

```text
B blocked because:
A = FAILED
condition = false
resource = unavailable
```

# 129. Workflow Validation

Before accepting a definition:

```text
validate graph
validate references
validate conditions
validate resources
validate policies
validate deadlines
validate compensation
```

# 130. Static Unsatisfiability

The validator should detect obvious impossible definitions.

Example:

```text
Node requires GPU
Agent pool has no GPU-capable Agent
```

if the Workflow is constrained exclusively to that pool.

# 131. Runtime Unsatisfiability

Some impossibility can only be discovered at runtime:

```text
all eligible Agents unavailable
```

This should become explicit scheduler state rather than an infinite wait with no explanation.

# 132. Workflow Pause

A Workflow may support:

```text
PAUSE_REQUESTED
PAUSED
```

for operator intervention.

# 133. Pause Semantics

Pause should generally prevent new nodes from starting while allowing already-running nodes to follow explicit policy.

# 134. Resume

Resume reconstructs eligibility from durable state.

# 135. Workflow Priority Inversion

A high-priority Workflow may wait behind resources held by low-priority Work.

Resource scheduling must define whether preemption is permitted.

# 136. Preemption

If supported:

```text
running low-priority node
→ preempt requested
→ checkpoint/cancel
→ resources released
→ high-priority node starts
```

# 137. Preemption Safety

Preemption must not be assumed safe for arbitrary Work.

Nodes must declare whether they support:

```text
checkpoint
graceful cancellation
restart
compensation
```

# 138. Checkpointing

A checkpointable node can persist execution state:

```text
checkpoint_id
state
artifact references
```

allowing recovery or migration.

# 139. Migration

A node can potentially migrate:

```text
Agent A
   ↓ checkpoint
Agent B
   ↓ restore
continue
```

This is significantly stronger than ordinary retry.

# 140. Migration Identity

Migration must preserve:

```text
logical node identity
```

while creating a new execution attempt or incarnation according to the execution model.

# 141. Workflow Invariants

```text
1. Workflow Definitions are versioned.

2. Workflow Instances bind to a specific definition version.

3. Node identity is stable within a definition.

4. Attempts are separate from logical node identity.

5. DAG validity is checked before execution.

6. Cycles are rejected unless explicit loop semantics exist.

7. Dependency semantics are explicit.

8. Fan-in semantics are explicit.

9. Conditions are deterministic and side-effect free.

10. Unknown conditions do not silently become false.

11. Gates are explicit stateful objects.

12. Readiness is derived from authoritative state.

13. Concurrent schedulers cannot claim the same node twice.

14. Workflow state transitions are version-checked.

15. Duplicate dispatch is reconciled before retry.

16. Retry creates a distinct attempt.

17. Retry policy is explicit.

18. Non-idempotent retries require reconciliation or explicit semantics.

19. Deadlines propagate through downstream execution.

20. Workflow deadlines cannot be extended accidentally by retries.

21. Compensation is distinct from rollback.

22. Compensation failures remain observable.

23. Unknown execution results remain UNKNOWN until resolved.

24. Workflow recovery reconstructs ready work from durable state.

25. Ephemeral scheduler queues are never the sole source of truth.

26. Cancellation is explicit.

27. Pause is distinct from cancellation.

28. Resource reservations have bounded lifetime.

29. Blocked nodes expose structured reasons.

30. Workflow state remains explainable and auditable.
```

# 142. Canonical DAG

```text
                 ┌──────────────┐
                 │    START     │
                 └──────┬───────┘
                        │
                        ▼
                    ┌───────┐
                    │   A   │
                    └───┬───┘
                  ┌─────┴─────┐
                  ▼           ▼
              ┌───────┐   ┌───────┐
              │   B   │   │   C   │
              └───┬───┘   └───┬───┘
                  │           │
                  └─────┬─────┘
                        ▼
                    ┌───────┐
                    │   D   │
                    └───┬───┘
                        │
                        ▼
                    ┌───────┐
                    │  END  │
                    └───────┘
```

# 143. Canonical Retry

```text
           NODE ATTEMPT 1
                 │
          ┌──────┴──────┐
          │             │
       SUCCESS        FAILURE
          │             │
          ▼             ▼
        DONE       CLASSIFY ERROR
                        │
                 ┌──────┴──────┐
                 │             │
              RETRYABLE    PERMANENT
                 │             │
                 ▼             ▼
              BACKOFF         FAIL
                 │
                 ▼
           NODE ATTEMPT 2
```

# 144. Canonical Recovery

```text
Controller Restart
        │
        ▼
Load Durable Workflow State
        │
        ▼
Recover State Versions
        │
        ▼
Inspect Active Attempts
        │
        ▼
Reconcile Agents
        │
        ├──────────────┐
        ▼              ▼
     CONFIRMED      UNKNOWN
        │              │
        ▼              ▼
     RESUME        RECONCILE
                       │
                       ▼
                  RESOLVE STATE
                       │
                       ▼
                 REBUILD READY QUEUE
                       │
                       ▼
                     RESUME
```

# 145. Canonical Compensation

```text
Forward Path

Reserve
   ↓
Deploy
   ↓
Configure
   ↓
Validate
   ↓
SUCCESS


Failure

Configure
   ↓
FAILED
   ↓
Compensation
   ↓
Undeploy
   ↓
Release
   ↓
RESTORE ACCEPTABLE STATE
```

# 146. Workflow Decision Rule

Before dispatching a node, NROS should be able to answer:

```text
Which definition version?
Which Workflow instance?
Which node?
Which attempt?
Are dependencies satisfied?
Are conditions satisfied?
Are gates satisfied?
Are resources available?
Is the Workflow still active?
Is the deadline valid?
Is this attempt still authorized?
```

If any required answer is unknown, the node should not silently execute.

# 147. Recovery Decision Rule

After an uncertain execution:

```text
Do not immediately retry.
Do not immediately mark failed.
Do not discard the previous attempt.

First:

identify
→ reconcile
→ classify
→ decide
```

# 148. Workflow Principle

> **A Workflow is a durable state machine over a dependency graph, not an ephemeral queue of tasks.**

# 149. Dependency Principle

> **Dependencies define semantic eligibility, while the scheduler decides when eligible Work can actually consume resources.**

# 150. Retry Principle

> **Retries create new execution attempts; they do not erase the history or uncertainty of previous attempts.**

# 151. Recovery Principle

> **Workflow recovery must reconstruct execution authority from durable state and reconciliation evidence rather than from in-memory scheduler state.**

# 152. Final Architectural Rule

> **NROS must provide a versioned, durable Workflow engine capable of expressing dependency graphs, conditional execution, concurrency, gates, retries, deadlines, compensation, cancellation, and recovery while preserving deterministic state transitions and explicit semantics for uncertain execution outcomes.**

The next layer is:

# Part CXXIII — Scheduler Architecture, Placement, Resource Accounting, Fairness, Priority, Queues, Admission Control, Preemption, Capacity, Bin-Packing & Scheduling Correctness

The central question becomes:

> **Given a set of READY Work items and a changing fleet of Agents, how does NROS choose what runs, where it runs, when it runs, and how resources are allocated without starvation, oversubscription, priority inversion, or duplicate execution?**

# NROS — Part CXXIII: Scheduler Architecture

The Scheduler is the decision engine that converts eligible Workflow Work into concrete execution assignments.

Its fundamental responsibility is:

```text
READY WORK
    +
AVAILABLE CAPACITY
    +
PLACEMENT POLICY
    ↓
AUTHORIZED ASSIGNMENT
```

The Scheduler must not merely maximize utilization.

It must preserve:

```text
correctness
fairness
safety
capacity constraints
priority semantics
deadline guarantees
authority
```

# 1. Scheduler Responsibility

The Scheduler answers four primary questions:

```text
WHAT runs?
WHERE does it run?
WHEN does it run?
HOW MUCH capacity does it receive?
```

# 2. Scheduler Non-Responsibility

The Scheduler should not become the authority for unrelated concerns.

For example:

```text
authentication
business logic
artifact storage
execution implementation
```

belong elsewhere.

# 3. Scheduler Inputs

A scheduling decision may depend on:

```text
READY nodes
Agent state
capabilities
resource availability
Workflow priority
tenant quotas
placement constraints
deadlines
affinity
anti-affinity
failure domains
preemption policy
```

# 4. Scheduler Output

A successful decision produces an assignment:

```text
Assignment {
    workflow_id
    node_id
    attempt
    agent_id
    resource_request
    placement_reason
    scheduler_epoch
}
```

# 5. Scheduling Is a State Transition

The scheduler should not simply return:

```text
Agent A
```

It must persist the resulting ownership/claim semantics.

# 6. Atomic Assignment

A scheduling operation should ensure:

```text
Work claimed
+
resources reserved
+
assignment recorded
```

are mutually consistent.

Otherwise concurrent schedulers can oversubscribe the same capacity.

# 7. Scheduler Epoch

A scheduler leadership or authority epoch may fence stale scheduler instances:

```text
epoch 41
```

becomes invalid when:

```text
epoch 42
```

takes authority.

# 8. Scheduler Workers

NROS may use multiple scheduler workers.

For example:

```text
Scheduler
 ├── admission worker
 ├── placement worker
 ├── dispatch worker
 └── reconciliation worker
```

These components must not create conflicting assignments.

# 9. Global Scheduler vs Distributed Scheduler

Two broad models exist.

### Centralized

```text
Controller
   ↓
Scheduler
   ↓
Agents
```

### Distributed

```text
Scheduler A
Scheduler B
Scheduler C
   ↓
shared authoritative state
```

Both require fencing and atomic claims.

# 10. Ready Queue

The scheduler consumes eligible Work from a durable logical ready set.

The in-memory queue is only an optimization.

# 11. Ready Queue Entry

A queue entry should identify:

```text
workflow_id
node_id
attempt
priority
enqueue_time
deadline
resource_request
queue_class
```

# 12. Queue Ordering

A queue may combine:

```text
priority
deadline urgency
fairness
age
tenant policy
```

rather than relying on a single numeric priority.

# 13. Priority

Priority represents relative scheduling importance.

Example:

```text
CRITICAL
HIGH
NORMAL
LOW
```

# 14. Priority Must Not Bypass Safety

A CRITICAL Work item cannot:

```text
exceed capacity
bypass authorization
violate placement constraints
ignore resource isolation
```

# 15. Fairness

If priority dominates indefinitely, low-priority Work can starve.

The scheduler therefore needs fairness semantics.

# 16. Fairness Domains

Fairness can be enforced across:

```text
tenants
users
projects
workflows
queues
resource pools
```

# 17. Weighted Fairness

Tenants may receive weights:

```text
Tenant A = 2
Tenant B = 1
```

A can receive approximately twice the scheduling share under sustained contention.

# 18. Fairness vs Utilization

A perfectly fair scheduler can still leave resources idle if fairness rules are too rigid.

Therefore:

```text
fairness
+
work conservation
```

must be balanced.

# 19. Work Conservation

If eligible Work exists and compatible capacity is genuinely available, the scheduler should normally avoid unnecessary idleness.

# 20. Admission Control

Before Work enters active scheduling, NROS may apply admission control.

Checks include:

```text
authorization
quota
budget
workflow limits
resource feasibility
policy
deadline feasibility
cluster capacity policy
```

# 21. Admission Rejection

Rejected Work should receive a structured reason:

```text
QUOTA_EXCEEDED
UNAUTHORIZED
UNSATISFIABLE
POLICY_DENIED
DEADLINE_IMPOSSIBLE
```

# 22. Admission vs Scheduling

Admission asks:

> May this Work participate?

Scheduling asks:

> Which eligible Work runs now and where?

These should remain separate.

# 23. Resource Model

Resources should be represented explicitly.

Common dimensions:

```text
CPU
memory
storage
GPU
network
devices
custom resources
```

# 24. Resource Quantity

Each resource requires a defined unit.

Examples:

```text
CPU = cores
memory = bytes
storage = bytes
GPU = devices
```

# 25. Resource Capacity

An Agent exposes:

```text
capacity
allocated
available
```

with:

```text
available = capacity - allocated
```

subject to reservation semantics.

# 26. Resource Request

Every schedulable node should define required resources when resource-aware placement is needed.

Example:

```text
CPU: 4
memory: 8 GiB
GPU: 1
```

# 27. Resource Limits

A Work item may distinguish:

```text
request
limit
```

where execution semantics support this distinction.

# 28. Request vs Limit

Request influences placement.

Limit constrains execution.

The exact relationship must be defined by the Agent runtime.

# 29. Resource Reservation

A reservation prevents another scheduling decision from consuming the same capacity.

# 30. Reservation Lifecycle

```text
AVAILABLE
   ↓
RESERVED
   ↓
ALLOCATED
   ↓
RELEASED
```

# 31. Reservation Leak

If an assignment fails, reservation state must be released or reconciled.

# 32. Reservation Lease

Reservations should have bounded lifetime.

Otherwise a crashed scheduler can permanently consume capacity.

# 33. Reservation Reconciliation

After scheduler recovery:

```text
durable reservations
↔
Agent allocations
```

must be compared.

# 34. Oversubscription

If:

```text
allocated > capacity
```

the scheduler has violated a fundamental resource invariant unless explicit oversubscription policy exists.

# 35. Oversubscription Policy

Some resources can intentionally be oversubscribed.

For example:

```text
CPU = share-based
```

But this must be explicit.

# 36. Hard Resources

Some resources should generally remain non-overcommittable:

```text
exclusive device
physical GPU
hardware channel
serial interface
```

# 37. Consumable Resources

A consumable resource decreases as Work is allocated.

# 38. Non-Consumable Resources

Some capabilities are predicates:

```text
linux = true
arm64 = true
```

They determine eligibility but are not necessarily numerically consumed.

# 39. Resource Classes

The scheduler should distinguish:

```text
scalar resources
integer resources
exclusive resources
boolean capabilities
topological resources
```

# 40. Placement Constraints

A Work item can specify constraints:

```text
region = eu
architecture = arm64
gpu = true
```

# 41. Hard Constraints

A hard constraint must be satisfied.

If:

```text
architecture = arm64
```

then an x86 Agent is ineligible.

# 42. Soft Constraints

A soft preference affects ranking but does not necessarily prevent placement.

Example:

```text
prefer same region
```

# 43. Affinity

Affinity encourages co-location.

Example:

```text
B prefers the Agent running A
```

# 44. Anti-Affinity

Anti-affinity avoids co-location.

Example:

```text
replicas should not share a host
```

# 45. Failure-Domain Placement

Replicas may need separation across:

```text
host
rack
zone
region
```

# 46. Topology-Aware Scheduling

A placement algorithm should be able to consider topology when the Workflow requires it.

# 47. Placement Pipeline

A useful scheduling pipeline is:

```text
READY
 ↓
ADMISSION
 ↓
FILTER
 ↓
SCORE
 ↓
RESERVE
 ↓
ASSIGN
 ↓
DISPATCH
```

# 48. Filter Phase

The filter phase removes Agents violating hard constraints.

Example:

```text
Agent A → GPU absent → reject
Agent B → GPU present → candidate
Agent C → GPU present → candidate
```

# 49. Score Phase

Candidates are ranked according to soft objectives.

Possible factors:

```text
resource fit
locality
load
fairness
energy
cost
latency
failure-domain diversity
```

# 50. Score Must Not Override Filters

A highly scored Agent that violates a hard constraint remains ineligible.

# 51. Scoring Determinism

Given the same authoritative state and policy version, scoring should ideally produce the same result.

# 52. Tie-Breaking

Ties require deterministic rules.

For example:

```text
score
→ least-loaded
→ oldest reservation
→ stable agent_id
```

# 53. Stable Tie-Breaker

A stable identifier prevents scheduling results from changing unpredictably due only to map iteration order.

# 54. Bin-Packing

The scheduler may use bin-packing strategies.

### Best fit

Place Work where remaining capacity most closely matches the request.

### Worst fit

Spread Work across available capacity.

# 55. Packing Objective

Packing can improve:

```text
resource utilization
```

but may reduce:

```text
failure isolation
```

# 56. Spreading Objective

Spreading can improve resilience but may increase fragmentation.

# 57. Fragmentation

A fleet may have enough total capacity but no single Agent satisfying a large request.

Example:

```text
4 Agents × 2 CPU free
```

cannot satisfy:

```text
1 request × 8 CPU
```

# 58. Fragmentation Awareness

The scheduler should distinguish:

```text
total capacity
```

from:

```text
usable capacity for this request
```

# 59. Gang Scheduling

Some Workflows require several nodes to start together.

Example:

```text
A
B
C
```

must all receive resources before any begins.

# 60. Gang Scheduling Semantics

If all required resources cannot be reserved:

```text
do not partially commit
```

unless partial gang allocation is explicitly supported.

# 61. Gang Reservation

A gang should be reserved atomically when the backend permits it.

# 62. Scheduling Deadlock

Gang requirements can create deadlocks with fragmented reservations.

The scheduler should either detect or prevent these conditions.

# 63. Queue Classes

NROS can organize Work into queue classes:

```text
interactive
batch
background
maintenance
system
```

# 64. Queue Isolation

A noisy batch workload should not necessarily consume all scheduling capacity needed for interactive Work.

# 65. Queue Concurrency

Each queue may define:

```text
max_concurrent
weight
priority
```

# 66. Tenant Quota

A tenant may have:

```text
CPU quota
memory quota
concurrent Work quota
Agent quota
```

# 67. Hierarchical Quotas

Quota can be hierarchical:

```text
Organization
   ├── Team A
   │    ├── Project 1
   │    └── Project 2
   └── Team B
```

Effective quota is constrained by every parent.

# 68. Quota Accounting

Quota accounting must be based on authoritative reservations/allocations rather than scheduler-local counters.

# 69. Quota Release

Quota must be released when:

```text
execution completes
execution is cancelled
reservation expires
allocation is reconciled away
```

# 70. Fairness Accounting

Fairness should account for actual resource consumption where appropriate, not merely number of tasks.

# 71. Weighted Resource Fairness

A tenant consuming:

```text
16 CPU
```

for a long duration should receive a larger fairness charge than one consuming:

```text
1 CPU
```

briefly.

# 72. Fairness Debt

A tenant that consumes more than its fair share accumulates scheduling debt.

The scheduler can reduce future priority until the system rebalances.

# 73. Aging

Aging increases effective scheduling priority as Work waits.

This prevents starvation.

# 74. Aging Bound

Aging should be capped to prevent a very old low-priority Work item from permanently dominating critical Work.

# 75. Deadline Scheduling

Work with approaching deadlines can receive urgency.

Example:

```text
slack ↓
priority ↑
```

# 76. Deadline Feasibility

If the scheduler can determine that:

```text
required work
>
remaining capacity before deadline
```

the Workflow should become explicitly deadline-infeasible.

# 77. Infeasible Deadline

Do not repeatedly schedule impossible Work while reporting it as normally healthy.

# 78. Admission Deadline Check

Admission control can reject or warn about obviously impossible deadlines.

# 79. Preemption

Preemption allows the scheduler to reclaim resources from lower-priority Work.

# 80. Preemption Preconditions

Preemption requires:

```text
policy permission
victim eligibility
safe termination/checkpoint
resource benefit
```

# 81. Preemption Victim

Candidate victims may be selected using:

```text
priority
age
checkpointability
resource footprint
fairness debt
deadline slack
```

# 82. Preemption Cost

The scheduler should account for the cost of:

```text
checkpointing
restart
lost progress
compensation
```

# 83. Non-Preemptible Work

Some Work must declare:

```text
preemptible = false
```

# 84. Preemption State

A node may enter:

```text
PREEMPT_REQUESTED
CHECKPOINTING
PREEMPTED
```

before eventual resumption or retry.

# 85. Priority Inversion

Priority inversion occurs when high-priority Work waits on resources held by low-priority Work.

# 86. Priority Inheritance

If dependencies require it, the scheduler may temporarily elevate the priority of the blocking Work.

# 87. Priority Inheritance Limits

Inheritance must be bounded and auditable.

# 88. Resource Locality

Placement may optimize locality:

```text
data locality
network locality
device locality
cache locality
```

# 89. Data Locality

If a large artifact exists on Agent A, placing dependent Work on A may reduce transfer cost.

# 90. Locality vs Resilience

The scheduler should not blindly prefer locality when it creates unacceptable concentration or failure risk.

# 91. Energy-Aware Scheduling

Where infrastructure exposes energy information, the scheduler may optimize:

```text
power consumption
thermal state
energy cost
```

# 92. Cost-Aware Scheduling

Cloud-like environments may expose:

```text
cost_per_cpu
cost_per_gpu
network_cost
```

# 93. Policy Objective

Multiple objectives can conflict:

```text
latency
cost
fairness
utilization
resilience
energy
```

The scheduler requires an explicit policy hierarchy or weighted objective.

# 94. Policy Version

Every scheduling decision should be attributable to:

```text
scheduler_policy_version
```

for reproducibility.

# 95. Scheduling Explanation

An assignment should be explainable.

Example:

```text
Selected Agent A because:

✓ GPU capability
✓ 8 CPU available
✓ region match
✓ quota available
✓ lowest placement cost
```

# 96. Rejection Explanation

A rejected Agent should have structured reasons:

```text
GPU_MISSING
MEMORY_INSUFFICIENT
ZONE_MISMATCH
QUOTA_CONFLICT
AGENT_DRAINING
```

# 97. Scheduling Trace

A scheduling decision can record:

```text
candidate_agents
filter_results
scores
selected_agent
reservation
policy_version
```

# 98. Trace Size

Full traces may be expensive.

NROS can retain summarized explanations while allowing detailed diagnostics when required.

# 99. Scheduling Event

Useful events include:

```text
WorkAdmitted
WorkQueued
WorkDequeued
PlacementEvaluated
AgentSelected
ReservationCreated
AssignmentCommitted
DispatchStarted
```

# 100. Scheduling Failure

A scheduling failure should identify whether it resulted from:

```text
no capacity
no compatible Agent
quota
policy
deadline
reservation race
scheduler failure
```

# 101. No-Capacity State

If Work cannot currently run:

```text
WAITING_FOR_CAPACITY
```

is preferable to treating it as failed.

# 102. No-Compatible-Agent State

If no Agent can ever satisfy the Work's hard constraints:

```text
UNSATISFIABLE_PLACEMENT
```

should be surfaced.

# 103. Temporary vs Permanent Unsatisfiability

The scheduler must distinguish:

```text
currently impossible
```

from:

```text
structurally impossible
```

# 104. Scheduler Backpressure

If dispatch cannot keep up with incoming Work, the system should apply backpressure.

# 105. Queue Growth

Metrics should expose:

```text
queue_depth
queue_age
arrival_rate
service_rate
```

# 106. Admission Backpressure

When queues exceed configured limits, new Work may be:

```text
delayed
rejected
rate-limited
```

# 107. Burst Handling

The scheduler should tolerate short bursts without creating unbounded memory or reservation growth.

# 108. Scheduling Storm

A failure affecting many Agents can cause thousands of Work items to become simultaneously retryable.

The scheduler should apply:

```text
retry backoff
queue fairness
admission limits
failure-domain awareness
```

# 109. Herd Prevention

After a common infrastructure failure, immediately retrying every Work item onto the same remaining Agents can recreate the failure.

# 110. Failure-Domain Diversification

Retry placement should prefer different failure domains when appropriate.

# 111. Scheduler Recovery

After Controller restart:

```text
load assignments
load reservations
load queue state
reconcile Agents
rebuild eligible queue
resume scheduling
```

# 112. Stale Reservation

If a reservation exists in Controller state but not on the Agent:

```text
RECONCILE
→ RELEASE
```

unless evidence indicates the assignment is still active.

# 113. Ghost Allocation

If an Agent reports allocation absent from Controller state:

```text
GHOST_ALLOCATION
```

must be investigated.

# 114. Scheduling Idempotency

A scheduling request should have an idempotency identity:

```text
schedule_decision_id
```

preventing duplicate commits during retries.

# 115. Dispatch Idempotency

Dispatch itself should also be idempotent:

```text
assignment_id
```

so network retries do not create duplicate execution.

# 116. Scheduler Lease

A scheduler worker may hold a lease over its scheduling partition.

If the lease expires:

```text
worker must stop authoritative scheduling
```

# 117. Partitioning

Large deployments may partition scheduling state by:

```text
tenant
resource pool
region
queue
workflow shard
```

# 118. Partition Ownership

Each partition should have a clear authority owner.

# 119. Partition Rebalancing

Ownership changes must use fencing to prevent old owners from continuing to commit assignments.

# 120. Cross-Partition Work

If a Workflow spans partitions, coordination must remain explicit.

# 121. Scheduler Invariants

```text
1. Scheduler decisions operate only on authoritative eligible state.

2. Hard placement constraints are never violated.

3. Resource reservations cannot exceed hard capacity.

4. Assignment and reservation state remain consistent.

5. Concurrent schedulers cannot claim the same Work attempt.

6. Stale scheduler epochs cannot commit new assignments.

7. Queue state is reconstructible from durable state.

8. In-memory queues are optimizations, not the source of truth.

9. Priority cannot bypass safety or authorization.

10. Fairness prevents indefinite starvation.

11. Aging is bounded.

12. Quotas are authoritative and recoverable.

13. Reservations have bounded lifetime.

14. Duplicate dispatch is prevented through assignment identity.

15. Unknown execution state is reconciled before unsafe retry.

16. Hard constraints and soft preferences remain distinct.

17. Candidate filtering precedes candidate scoring.

18. Tie-breaking is deterministic.

19. Structural placement impossibility is distinguished from temporary capacity shortage.

20. Deadline infeasibility is observable.

21. Preemption is policy-controlled.

22. Non-preemptible Work cannot be forcibly preempted without explicit override semantics.

23. Scheduler policy is versioned.

24. Scheduling decisions are explainable.

25. Scheduler recovery reconstructs authority before resuming dispatch.

26. Partition ownership is fenced.

27. Resource accounting is based on durable authoritative state.

28. Queue backpressure prevents unbounded growth.

29. Retry storms are controlled.

30. Scheduling must preserve correctness before optimization.
```

# 122. Canonical Scheduling Pipeline

```text
             READY WORK
                  │
                  ▼
             ADMISSION
                  │
          ┌───────┴────────┐
          │                │
       ACCEPT            REJECT
          │
          ▼
        QUEUE
          │
          ▼
      DEQUEUE
          │
          ▼
        FILTER
          │
          ▼
     CANDIDATES
          │
          ▼
        SCORE
          │
          ▼
     SELECT AGENT
          │
          ▼
       RESERVE
          │
          ▼
    COMMIT ASSIGNMENT
          │
          ▼
       DISPATCH
          │
          ▼
       RECONCILE
          │
          ▼
       EXECUTION
```

# 123. Canonical Placement

```text
Work Request
     │
     ├── CPU >= 4
     ├── Memory >= 8GiB
     ├── GPU = 1
     ├── Region = EU
     └── Anti-affinity = Host
              │
              ▼
        FILTER AGENTS
              │
       ┌──────┼──────┐
       ▼      ▼      ▼
      A       B      C
     GPU✓   GPU✓   GPU✗
     EU✓    EU✓    EU✓
     RAM✓   RAM✓   RAM✓
       │      │
       └──┬───┘
          ▼
        SCORE
          │
          ▼
       SELECT B
```

# 124. Canonical Resource Accounting

```text
Agent Capacity
      │
      ├── CPU 16
      ├── RAM 32G
      └── GPU 2
             │
             ▼
        Reservations
             │
      ┌──────┼──────┐
      ▼      ▼      ▼
     W1     W2     W3
    CPU4   CPU6   CPU2
    RAM8   RAM12  RAM4
    GPU1   GPU0   GPU1
      │      │      │
      └──────┼──────┘
             ▼
       Remaining Capacity
```

# 125. Canonical Fairness

```text
              READY
                │
       ┌────────┼────────┐
       ▼        ▼        ▼
    Tenant A  Tenant B  Tenant C
       │        │        │
       ▼        ▼        ▼
   usage=high usage=low usage=medium
       │        │        │
       ▼        ▼        ▼
  fairness↓ fairness↑ fairness↔
       │        │        │
       └────────┼────────┘
                ▼
          Scheduling Order
```

# 126. Canonical Preemption

```text
High Priority Work
       │
       ▼
No Capacity
       │
       ▼
Find Victim
       │
       ├── non-preemptible → reject victim
       │
       └── preemptible
              │
              ▼
       PREEMPT_REQUESTED
              │
              ▼
          CHECKPOINT
              │
              ▼
          RELEASE
              │
              ▼
       HIGH PRIORITY RUNS
```

# 127. Scheduling Decision Record

A durable scheduling record should conceptually contain:

```text
decision_id
scheduler_epoch
policy_version

workflow_id
node_id
attempt

candidate_agents
filter_results
scores

selected_agent

resource_request
reservation_id
assignment_id

created_at
committed_at
```

# 128. Scheduling Explanation Record

A compact explanation may contain:

```text
selected_agent = A42

hard_constraints:
    PASS

resource_fit:
    PASS

quota:
    PASS

fairness:
    PASS

locality_score:
    0.91

load_score:
    0.78

final_score:
    0.86
```

# 129. Scheduler Correctness Principle

> **A scheduling decision is correct only when the selected placement satisfies every hard constraint and the corresponding resource ownership is durably and atomically established.**

# 130. Fairness Principle

> **Fairness is a scheduling policy, not a license to violate resource safety, authorization, or explicit priority semantics.**

# 131. Recovery Principle

> **Scheduler recovery must reconstruct authority, reservations, assignments, and eligibility before issuing new execution decisions.**

# 132. Placement Principle

> **Placement should first eliminate invalid Agents, then rank valid candidates according to explicit and versioned policy.**

# 133. Resource Principle

> **Capacity must be represented and accounted for explicitly; total fleet capacity is not equivalent to capacity satisfying a particular Work request.**

# 134. Final Architectural Rule

> **NROS must implement scheduling as an authoritative, durable, policy-versioned decision process that combines admission control, resource accounting, hard constraint filtering, deterministic placement scoring, fairness, priority, quotas, preemption, reservations, backpressure, and reconciliation while ensuring that no stale or concurrent scheduler can create duplicate or unsafe execution.**

The next layer is:

# Part CXXIV — Persistence & State Store Architecture, Transactions, Event Log, Snapshots, Indexes, Concurrency Control, Recovery, Durability, Compaction & Consistency

The central question becomes:

> **What is the authoritative persistent state of NROS, how is it committed atomically, how do concurrent components coordinate against it, and how can the entire control plane recover deterministically after crashes or partial failures?**

# NROS — Part CXXIV: Persistence & State Store Architecture, Transactions, Event Log, Snapshots, Indexes, Concurrency Control, Recovery, Durability, Compaction & Consistency

Persistence is the authoritative memory of NROS.

Everything that determines whether Work may execute must ultimately be reconstructible from durable state.

The fundamental invariant is:

> **If a control-plane decision cannot be recovered from durable authoritative state, that decision is not a reliable system fact.**

# 1. Persistence Responsibility

The State Store provides durable representations of:

```text
Workflows
Workflow nodes
Attempts
Assignments
Agents
Resources
Reservations
Leases
Policies
Events
Artifacts
Checkpoints
Gates
Quotas
Scheduler state
Recovery state
```

# 2. Authoritative State

NROS must distinguish between:

```text
authoritative state
```

and:

```text
derived state
```

For example:

```text
Database:
    node_state = READY

Memory:
    ready_queue contains node
```

The database is authoritative.

The queue is derived.

# 3. Source-of-Truth Rule

A critical invariant:

```text
Durable State
      ↓
Derived Runtime State
```

never:

```text
Runtime State
      ↓
Durable State
```

as the only recovery mechanism.

# 4. State Store Requirements

The persistence layer should provide:

```text
atomicity
durability
consistency
concurrency control
transactions
queryability
recovery
```

The exact implementation may vary.

# 5. Logical Data Model

A conceptual NROS State Store contains:

```text
workflow
workflow_node
work_attempt
assignment
agent
agent_resource
reservation
lease
gate
quota
policy
event
snapshot
artifact_reference
checkpoint
```

# 6. Workflow Record

Conceptually:

```text
workflow_id
definition_id
definition_version
state
priority
created_at
updated_at
deadline
state_version
```

# 7. Workflow Node Record

```text
workflow_id
node_id
state
attempt_count
current_attempt
condition_state
blocked_reason
state_version
```

# 8. Attempt Record

Each execution attempt should be independently identifiable.

```text
workflow_id
node_id
attempt
execution_id
agent_id
state
started_at
finished_at
failure_class
```

# 9. Assignment Record

The assignment connects logical Work to execution placement.

```text
assignment_id
workflow_id
node_id
attempt
agent_id
resource_request
reservation_id
scheduler_epoch
state
```

# 10. Agent Record

An Agent record can include:

```text
agent_id
state
capabilities
capacity
last_heartbeat
registration_epoch
```

# 11. Reservation Record

```text
reservation_id
agent_id
assignment_id
resource_vector
state
expires_at
```

# 12. Lease Record

Leases protect ownership over time.

```text
lease_id
owner_id
resource_type
epoch
expires_at
```

# 13. Event Record

A durable event can contain:

```text
event_id
stream_id
sequence
event_type
payload
causation_id
correlation_id
created_at
schema_version
```

# 14. Event Stream

A Workflow can have an ordered stream:

```text
WorkflowCreated
NodeReady
NodeDispatched
NodeStarted
NodeSucceeded
WorkflowCompleted
```

# 15. Sequence Number

Each stream should have a monotonically increasing sequence where ordered event semantics are required.

Example:

```text
1 WorkflowCreated
2 NodeReady
3 NodeDispatched
4 NodeStarted
5 NodeSucceeded
```

# 16. Global Ordering

NROS should not assume that every event requires a globally ordered sequence.

Global ordering is expensive and often unnecessary.

Per-stream ordering is usually more useful.

# 17. Causation ID

A causation identifier answers:

> Which event or command caused this event?

Example:

```text
NodeFailed
caused_by = ExecutionFinished
```

# 18. Correlation ID

A correlation ID groups related operations across components.

Example:

```text
workflow request
→ scheduler decision
→ assignment
→ dispatch
→ execution
```

can share one correlation identity.

# 19. Command vs Event

NROS should distinguish:

```text
Command:
"dispatch node A"

Event:
"node A was dispatched"
```

A command represents intent.

An event represents committed fact.

# 20. Persistence Boundary

The system must avoid emitting a durable success event before the corresponding state change is safely committed.

# 21. Transaction

A transaction groups state changes that must succeed or fail together.

Example:

```text
BEGIN

update node → DISPATCHED
create assignment
create reservation
append event

COMMIT
```

# 22. Atomic Assignment

The following should be treated as one logical transaction:

```text
claim Work
+
reserve capacity
+
create assignment
```

when the underlying persistence system supports the required atomicity.

# 23. Transaction Failure

If commit fails:

```text
none of the transaction's authoritative changes
```

should be treated as committed.

# 24. Partial Failure

Distributed operations can produce:

```text
Controller committed
Agent did not receive message
```

This is why:

```text
durable assignment
+
reconciliation
```

is required.

# 25. Transaction vs Distributed Action

A database transaction cannot automatically roll back an external Agent action.

Therefore:

```text
DB transaction
≠
distributed transaction
```

# 26. Outbox Pattern

NROS can use an outbox:

```text
state change
+
outbox message
```

committed atomically.

A dispatcher later delivers the external command.

# 27. Outbox Lifecycle

```text
PENDING
   ↓
DELIVERING
   ↓
DELIVERED
```

Failures can return the record to retryable state.

# 28. Outbox Idempotency

Every outbound command should carry an idempotency key.

Example:

```text
assignment_id
```

# 29. Inbox Pattern

Agents or receiving components can persist received command identities.

This allows duplicate network deliveries to be safely ignored.

# 30. Exactly-Once Illusion

NROS should not assume that arbitrary distributed messaging provides exactly-once execution.

Instead, it should implement:

```text
at-least-once delivery
+
idempotent handling
+
durable identity
+
reconciliation
```

# 31. State Version

Every mutable authoritative object can maintain:

```text
state_version
```

incremented on successful state mutation.

# 32. Optimistic Concurrency

A transition can use:

```text
UPDATE ...
WHERE id = X
AND state_version = 42
```

and increment:

```text
42 → 43
```

# 33. Lost Update Prevention

Without version checks:

```text
Writer A reads READY
Writer B reads READY
A writes RUNNING
B writes CANCELLED
```

The final state may silently overwrite the first decision.

Version checks expose the conflict.

# 34. Pessimistic Locking

Some operations may instead require transactional locks.

Examples:

```text
resource reservation
exclusive assignment
quota mutation
```

# 35. Optimistic vs Pessimistic

Optimistic concurrency works well when conflicts are uncommon.

Pessimistic locking can be preferable when conflicts are frequent or correctness requires serialized ownership.

# 36. State Machine Validation

Persistence must not allow arbitrary state mutation.

For example:

```text
SUCCEEDED → RUNNING
```

should be rejected unless an explicit recovery transition exists.

# 37. Transactional State Machine

The transition should validate:

```text
current state
expected version
transition legality
authorization
policy
```

within the authoritative transaction.

# 38. Event and State Consistency

NROS should define whether:

```text
state
```

or:

```text
event log
```

is the primary source of truth.

A hybrid model is possible, but the relationship must be explicit.

# 39. State-First Model

In a state-first model:

```text
state = authority
events = audit/history
```

Events describe transitions but are not required to reconstruct every field.

# 40. Event-Sourced Model

In an event-sourced model:

```text
events = authority
state = projection
```

State is reconstructed from events.

# 41. Hybrid Model

NROS may use:

```text
durable current state
+
immutable event history
+
periodic snapshots
```

This is often practical for control-plane systems.

# 42. Snapshot

A snapshot captures current state at a known event sequence.

```text
snapshot_sequence = 10000
```

# 43. Snapshot Recovery

Recovery becomes:

```text
load snapshot
+
replay events after snapshot
```

instead of replaying the entire history.

# 44. Snapshot Atomicity

A snapshot must not combine incompatible versions of related entities.

# 45. Snapshot Metadata

A snapshot should record:

```text
snapshot_id
stream/version
created_at
schema_version
state_hash
```

# 46. State Hash

A deterministic hash can help verify snapshot integrity.

# 47. Event Integrity

Event records may include a checksum or hash.

This helps detect corruption.

# 48. Tamper Evidence

For security-sensitive deployments, events can form a hash chain:

```text
H(event_n)
=
hash(event_n_payload + H(event_n-1))
```

This provides tamper evidence.

# 49. Persistence Durability

A successful commit must satisfy the configured durability guarantee.

NROS must explicitly define what:

```text
COMMITTED
```

means.

# 50. Commit Semantics

Possible durability levels:

```text
local durable
replicated durable
quorum durable
```

The system must not claim stronger durability than the backend guarantees.

# 51. Crash Consistency

After a process crash, NROS must recover to a state satisfying all persistence invariants.

# 52. Power-Loss Recovery

If the storage backend can lose recently acknowledged writes under power failure, the NROS durability contract must reflect that behavior.

# 53. Replication

The State Store may replicate authoritative state across nodes.

Replication provides:

```text
availability
fault tolerance
```

but introduces consistency considerations.

# 54. Quorum

A replicated store may require a quorum before acknowledging a critical write.

# 55. Split Brain

Multiple controllers must not independently become authoritative.

Leadership requires:

```text
lease
epoch
fencing
```

# 56. Leader Epoch

Example:

```text
Leader A → epoch 12
Leader B → epoch 13
```

A must not commit new authoritative scheduler decisions after epoch 13 is established.

# 57. Fencing

Every authoritative write can carry the current leadership epoch.

The persistence layer rejects stale epochs where supported.

# 58. Lease Expiration

A controller must stop authoritative actions after its leadership lease expires unless it successfully renews it.

# 59. Clock Dependence

Lease correctness should avoid trusting an unsynchronized wall clock alone.

Monotonic timers and backend-enforced expiration semantics are preferable where available.

# 60. Indexes

The State Store requires indexes for common queries.

Examples:

```text
workflow(state)
node(workflow_id, state)
assignment(agent_id, state)
reservation(agent_id, state)
event(stream_id, sequence)
lease(owner_id)
```

# 61. Index Correctness

Indexes are derived structures.

They must never become more authoritative than the underlying records.

# 62. Queue Index

A ready queue can be represented by an index over:

```text
state
priority
deadline
enqueue_time
```

# 63. Efficient Readiness

Instead of scanning every Workflow repeatedly:

```text
index READY nodes
```

can provide efficient scheduling.

# 64. Blocked Index

Similarly:

```text
blocked_reason
```

can support operational diagnostics.

# 65. Garbage Collection

Completed Workflows and events may eventually become eligible for retention policies.

# 66. Retention Policy

Retention can depend on:

```text
age
workflow class
audit requirements
legal requirements
storage pressure
```

# 67. Event Compaction

If event history is retained indefinitely, storage can grow without bound.

Compaction can remove redundant history only when policy permits.

# 68. Compaction Safety

Never compact events required for:

```text
audit
recovery
compliance
active snapshots
```

# 69. Tombstones

Deleted entities may require tombstones to prevent stale replicas or caches from resurrecting them.

# 70. Entity Resurrection

A stale message must not recreate an object that has already been permanently deleted or superseded.

# 71. Schema Versioning

Persistent records require:

```text
schema_version
```

when schema evolution is expected.

# 72. Migration

Schema migrations should be:

```text
versioned
testable
rollback-aware where possible
```

# 73. Backward Compatibility

During rolling upgrades, different Controller versions may temporarily coexist.

The persistence contract must define compatibility between them.

# 74. Event Schema Evolution

Events require explicit schema versions.

Old consumers should either:

```text
understand old versions
```

or:

```text
perform controlled migration
```

# 75. Unknown Fields

Forward-compatible serialization should avoid breaking older readers when safe.

# 76. Serialization Determinism

For hashes, signatures, snapshots, or reproducibility, serialization must be canonical.

# 77. Idempotent Writes

Persistence APIs should support idempotency where callers may retry requests.

# 78. Idempotency Key

A mutation can include:

```text
idempotency_key
```

so repeated requests return the same logical result rather than create duplicate state.

# 79. Duplicate Event Prevention

If the same command is retried:

```text
Command X
Command X
```

the system must avoid producing contradictory duplicate transitions.

# 80. Transaction Boundary

A transaction should be as small as practical while preserving invariants.

Large transactions increase:

```text
lock contention
latency
failure surface
```

# 81. Long-Running Transactions

NROS should avoid holding database transactions across external Agent execution.

Never:

```text
BEGIN TRANSACTION
↓
dispatch Agent
↓
wait 30 minutes
↓
COMMIT
```

# 82. Correct Pattern

Instead:

```text
Transaction:
    persist assignment
    commit

External:
    dispatch

Later:
    reconcile result
```

# 83. Transactional Outbox

This pattern ensures the dispatch intent survives a Controller crash.

# 84. Inbox Deduplication

The Agent can persist:

```text
assignment_id
```

before acknowledging command processing.

# 85. At-Least-Once Control Plane

The preferred control-plane assumption should be:

```text
messages may duplicate
messages may be delayed
messages may be lost
```

Therefore state transitions must be idempotent and reconcilable.

# 86. Exactly-Once State Transition

The State Store can provide exactly-once semantics for a transaction within its own authoritative boundary even though external execution remains at-least-once.

# 87. Recovery Algorithm

After Controller restart:

```text
1. Acquire leadership.
2. Establish new epoch.
3. Load durable state.
4. Validate state integrity.
5. Recover active leases.
6. Reconcile Agent registrations.
7. Reconcile reservations.
8. Reconcile assignments.
9. Rebuild derived indexes/queues.
10. Resume eligible scheduling.
```

# 88. Recovery Ordering

Leadership must be established before new authoritative scheduling decisions are emitted.

# 89. Recovery Barrier

A Controller should expose:

```text
RECOVERING
```

until its authoritative state is safe to serve normal scheduling traffic.

# 90. Recovery Completion

Transition:

```text
RECOVERING → ACTIVE
```

should require explicit recovery invariants to pass.

# 91. Corrupt State

If state integrity checks fail:

```text
do not silently continue
```

The Controller should enter a degraded or recovery-required state.

# 92. Degraded Persistence

If the State Store becomes unavailable:

```text
do not continue making irreversible scheduling decisions
```

unless an explicitly safe degraded mode exists.

# 93. Read Availability vs Write Availability

The system may allow diagnostic reads while rejecting authoritative mutations.

# 94. Persistence Backpressure

If writes slow down:

```text
scheduler throughput
```

must eventually be throttled rather than allowing unbounded in-memory state.

# 95. Write Queue

An internal write queue can smooth short bursts but must have bounded capacity.

# 96. Persistence Metrics

Important metrics:

```text
transaction_latency
commit_latency
write_rate
read_rate
queue_depth
lock_wait
conflict_rate
snapshot_duration
replay_duration
event_lag
replication_lag
```

# 97. Recovery Metrics

Track:

```text
recovery_duration
reconciliation_duration
unknown_assignments
stale_leases
orphan_reservations
rebuild_duration
```

# 98. Consistency Model

NROS should document consistency guarantees for every major API.

Examples:

```text
strongly consistent
read-after-write
eventually consistent
best effort
```

# 99. Scheduling Reads

Scheduling-critical reads should use authoritative consistency.

A stale read must not cause an unsafe assignment.

# 100. Diagnostic Reads

Dashboards may tolerate eventual consistency where correctness is unaffected.

# 101. Cache

Caches can accelerate reads.

But:

```text
cache ≠ authority
```

# 102. Cache Invalidation

State changes should invalidate or version caches deterministically.

# 103. Versioned Cache

A cache entry can include:

```text
state_version
```

and be rejected if stale.

# 104. Read Repair

If a derived projection is inconsistent, NROS should be able to rebuild it from authoritative state.

# 105. Projection

Examples:

```text
ready queue
metrics
workflow summaries
Agent utilization
```

are derived projections.

# 106. Projection Rebuild

A projection must be rebuildable after corruption or loss.

# 107. Projection Lag

If event-driven projections lag behind authoritative state, they should expose their sequence/lag.

# 108. Event Consumer Offset

Consumers can maintain:

```text
stream_id
last_processed_sequence
```

# 109. Consumer Recovery

After restart:

```text
load last sequence
→ replay remaining events
→ resume
```

# 110. Event Poisoning

Malformed events should not permanently crash the event consumer.

The system should provide:

```text
dead-letter handling
quarantine
diagnostic visibility
```

where appropriate.

# 111. Audit Trail

Authoritative mutations should be attributable.

Audit records may include:

```text
actor
action
target
timestamp
reason
correlation_id
```

# 112. Security Boundary

Audit data should be protected from unauthorized modification.

# 113. Sensitive Data

Persistence should avoid storing secrets directly in ordinary Workflow state.

Use references to secure secret-management facilities where necessary.

# 114. Secret Redaction

Logs, events, traces, and snapshots should respect field-level redaction policies.

# 115. Persistence Encryption

Sensitive persisted data may require encryption at rest.

# 116. Backup

NROS must define backup procedures for authoritative state.

# 117. Restore

A restore must preserve semantic correctness, including:

```text
Workflow identities
event ordering
assignment identities
lease epochs
schema versions
```

# 118. Restore and External Reality

Restoring an old database snapshot does not automatically restore the external Agent fleet to the same state.

Therefore restore requires reconciliation.

# 119. Disaster Recovery

A disaster recovery procedure should be:

```text
restore
→ establish new authority
→ fence old authority
→ reconcile Agents
→ reconcile assignments
→ resume
```

# 120. Split-Brain Recovery

Old Controllers must be prevented from making authoritative changes after failover.

# 121. Disaster-Recovery Epoch

A new recovery generation can fence pre-disaster control-plane state.

# 122. Persistence Invariants

```text
1. Durable state is authoritative.

2. Derived queues and caches are reconstructible.

3. State transitions are validated.

4. Concurrent mutations cannot silently overwrite each other.

5. Critical transitions use atomic persistence boundaries.

6. External execution is never held inside a long database transaction.

7. Commands and committed events are distinct concepts.

8. Outbox delivery is idempotent.

9. External command delivery is assumed to be at least once.

10. Duplicate commands cannot create duplicate logical execution.

11. Unknown external state is reconciled.

12. Leadership epochs fence stale Controllers.

13. Leases have bounded lifetime.

14. Snapshots are internally consistent.

15. Event ordering is explicit.

16. Event schema versions are explicit.

17. Projections are rebuildable.

18. Caches are never authoritative.

19. Persistence failures prevent unsafe irreversible scheduling.

20. Recovery has an explicit barrier.

21. Restored state is reconciled with external reality.

22. Schema migrations are versioned.

23. Retention and compaction respect audit and recovery requirements.

24. Persistent sensitive data follows security policy.

25. Every authoritative mutation remains attributable.

26. State integrity can be verified.

27. Recovery is deterministic wherever the underlying evidence permits.

28. The system never claims stronger durability or consistency than its storage backend actually provides.
```

# 123. Canonical Persistence Architecture

```text
                    NROS CONTROL PLANE
                           │
                    ┌──────┴──────┐
                    │             │
                Commands       Queries
                    │             │
                    ▼             ▼
               State Machine   Read Model
                    │             ▲
                    ▼             │
              TRANSACTION        │
                    │             │
        ┌───────────┼─────────────┤
        ▼           ▼             ▼
   Current State   Event Log    Outbox
        │           │             │
        │           │             ▼
        │           │          Dispatcher
        │           │             │
        │           │             ▼
        │           │           Agent
        │           │
        ▼           ▼
     Snapshot   Event Consumers
                    │
                    ▼
                Projections
```

# 124. Canonical Assignment Transaction

```text
BEGIN

Validate:
    node.state == READY
    version == expected
    scheduler_epoch == current

Create:
    reservation
    assignment

Update:
    node.state = DISPATCHED
    state_version += 1

Append:
    NodeDispatched

Create:
    Outbox(DispatchAssignment)

COMMIT
```

After commit:

```text
Outbox
   ↓
Dispatcher
   ↓
Agent
```

# 125. Canonical Recovery

```text
                    CRASH
                      │
                      ▼
                 NEW CONTROLLER
                      │
                      ▼
                ACQUIRE LEASE
                      │
                      ▼
                 NEW EPOCH
                      │
                      ▼
               LOAD DURABLE STATE
                      │
                      ▼
                VALIDATE INTEGRITY
                      │
                      ▼
              RECONCILE EXTERNALS
                      │
          ┌───────────┼───────────┐
          ▼           ▼           ▼
       Agents     Reservations  Leases
          │           │           │
          └───────────┼───────────┘
                      ▼
               REBUILD PROJECTIONS
                      │
                      ▼
                RECOVERY BARRIER
                      │
                      ▼
                    ACTIVE
```

# 126. Canonical State Transition

```text
Read current state
       │
       ▼
Validate expected version
       │
       ▼
Validate transition
       │
       ▼
Apply mutation
       │
       ├── Append event
       ├── Update state
       └── Create outbox
       │
       ▼
     COMMIT
       │
       ▼
Authoritative Fact Exists
```

# 127. Canonical Consistency Model

```text
                 AUTHORITATIVE STATE
                         │
          ┌──────────────┼──────────────┐
          ▼              ▼              ▼
      Scheduler        Events         API Reads
          │              │
          ▼              ▼
     Assignments      Projections
          │              │
          ▼              ▼
        Agents        Dashboards
```

The authoritative state remains the reference point for correctness.

# 128. Persistence Correctness Principle

> **Durability is not merely storing bytes; it is preserving the semantic state machine across crashes, concurrency, retries, upgrades, and recovery.**

# 129. Transaction Principle

> **A transaction should atomically establish every control-plane fact that must remain mutually consistent, while external execution must remain outside the transaction and be reconciled through durable intent and identity.**

# 130. Recovery Principle

> **Recovery must establish new authority, reconstruct durable state, reconcile external reality, rebuild derived state, and only then resume scheduling.**

# 131. Consistency Principle

> **Every NROS component must know whether the data it consumes is authoritative, strongly consistent, eventually consistent, cached, or derived.**

# 132. Final Architectural Rule

> **NROS must provide a durable, transactional, versioned, recoverable State Store in which Workflow state, scheduling decisions, reservations, leases, events, and control-plane authority can be reconstructed deterministically, while external execution is coordinated through idempotent commands, durable intent, reconciliation, and explicit consistency guarantees.**

The next layer is:

# Part CXXV — Agent Runtime & Execution Protocol, Registration, Heartbeats, Capabilities, Command Delivery, Execution Lifecycle, Sandboxing, Process Supervision, Checkpointing, Result Reporting & Agent Recovery

The central question becomes:

> **How does an NROS Agent safely receive an assignment, establish execution authority, launch and supervise Work, report authoritative execution evidence, recover from local failures, and reconcile its state with the Controller without creating duplicate or orphaned execution?**

# NROS — Part CXXV: Agent Runtime & Execution Protocol

The Agent is the execution-side authority of NROS.

The Controller decides **what should execute and where**.

The Agent determines **whether the local execution environment can safely realize that assignment and supervises the resulting execution**.

The fundamental relationship is:

```text
Controller
   │
   │ Assignment
   ▼
Agent
   │
   ├── Validate
   ├── Reserve
   ├── Prepare
   ├── Launch
   ├── Supervise
   ├── Report
   └── Recover
```

The Agent must never interpret network delivery alone as proof that execution succeeded.

# 1. Agent Responsibility

An Agent is responsible for:

```text
registration
capability advertisement
heartbeat
assignment acceptance
local resource validation
execution preparation
process supervision
output capture
checkpointing
result reporting
cleanup
reconciliation
```

# 2. Agent Non-Responsibility

The Agent should not independently redefine Controller policy.

For example, an Agent should not decide:

```text
priority
tenant quota
global fairness
workflow dependency semantics
global scheduling
```

unless explicitly delegated such authority.

# 3. Agent Identity

Every Agent requires a stable identity:

```text
agent_id
```

This identity must survive process restart where the underlying installation remains the same.

# 4. Agent Instance Identity

An Agent process should additionally have an instance identity:

```text
agent_instance_id
```

This distinguishes:

```text
same Agent
different runtime incarnation
```

# 5. Registration

Agent startup begins with registration:

```text
STARTING
   ↓
REGISTERING
   ↓
REGISTERED
```

# 6. Registration Payload

Conceptually:

```text
AgentRegistration {
    agent_id
    agent_instance_id
    protocol_version
    runtime_version
    platform
    architecture
    capabilities
    capacity
    labels
    endpoint
}
```

# 7. Registration Validation

The Controller should validate:

```text
identity
protocol compatibility
authorization
capability schema
resource declaration
runtime compatibility
```

before accepting the Agent.

# 8. Registration Epoch

Each Agent instance should receive or establish a registration generation.

Example:

```text
agent_id = A42

instance 100 → old
instance 101 → current
```

Commands targeting instance 100 must not accidentally control instance 101.

# 9. Capability Advertisement

Agents advertise what they can execute.

Examples:

```text
linux
x86_64
rust
docker
wasm
gpu
network
device:x
```

# 10. Capability vs Resource

A capability answers:

> Can this Agent perform this class of operation?

A resource answers:

> How much capacity is available?

These are distinct concepts.

# 11. Capability Version

Capability definitions should be versioned.

Otherwise a Controller may interpret:

```text
gpu
```

differently from the Agent.

# 12. Dynamic Capabilities

Some capabilities may change during runtime.

For example:

```text
GPU available
```

may become:

```text
GPU unavailable
```

because of a hardware fault.

The Agent must report capability changes.

# 13. Static Capabilities

Other capabilities normally remain stable:

```text
architecture
OS family
installed runtime
hardware class
```

# 14. Resource Advertisement

The Agent reports:

```text
capacity
allocatable
allocated
reserved
```

where applicable.

# 15. Capacity Is Not Trust

The Controller should treat Agent-reported capacity as evidence subject to registration and policy.

A compromised Agent cannot be allowed to arbitrarily expand global quota.

# 16. Heartbeat

An Agent periodically reports liveness.

```text
REGISTERED
   ↓
HEARTBEAT
   ↓
HEARTBEAT
   ↓
HEARTBEAT
```

# 17. Heartbeat Contents

A heartbeat can include:

```text
agent_id
instance_id
timestamp/monotonic counter
load
active assignments
resource state
health
protocol version
```

# 18. Heartbeat Is Not Execution Evidence

A heartbeat proves that the Agent is responsive.

It does not prove:

```text
Work succeeded
```

# 19. Execution Evidence

Execution state requires explicit evidence:

```text
started
running
checkpointed
exited
result reported
```

# 20. Heartbeat Timeout

If heartbeats stop:

```text
REGISTERED
    ↓
SUSPECT
    ↓
UNREACHABLE
```

The Controller must not immediately assume that running Work has stopped.

# 21. Unknown Execution State

An unreachable Agent creates:

```text
EXECUTION_UNKNOWN
```

until sufficient evidence exists.

# 22. Why Unknown Matters

Immediately retrying Work can produce:

```text
old execution still running
+
new execution started
```

which creates duplicate execution.

# 23. Agent Recovery

When an Agent reconnects, it should report its active execution inventory.

Example:

```text
Assignment A1 → RUNNING
Assignment A2 → EXITED
Assignment A3 → UNKNOWN
```

# 24. Reconciliation

Controller and Agent compare:

```text
Controller assignments
↔
Agent execution inventory
```

# 25. Agent State Machine

A useful Agent lifecycle is:

```text
STARTING
   ↓
REGISTERING
   ↓
REGISTERED
   ↓
DRAINING
   ↓
STOPPING
   ↓
STOPPED
```

Failure states can overlay this lifecycle.

# 26. Agent Health

Health should be multidimensional:

```text
control-plane health
execution health
resource health
hardware health
storage health
network health
```

# 27. Ready vs Healthy

An Agent can be:

```text
HEALTHY
```

but not:

```text
SCHEDULABLE
```

For example, it may be draining.

# 28. Drain

Draining means:

```text
accept existing Work
do not accept new Work
```

# 29. Drain Completion

An Agent is drained when:

```text
active assignments = 0
```

or explicit force policy takes effect.

# 30. Force Drain

Force drain may terminate active Work.

This must be an explicit policy operation.

# 31. Command Protocol

Controller-to-Agent commands should have stable identities.

Example:

```text
Command {
    command_id
    assignment_id
    command_type
    protocol_version
    created_at
    payload
}
```

# 32. Command Types

Examples:

```text
PrepareAssignment
StartAssignment
CancelAssignment
CheckpointAssignment
TerminateAssignment
CollectDiagnostics
ReleaseAssignment
```

# 33. Command Ordering

Commands may require ordering.

For example:

```text
Prepare
   ↓
Start
   ↓
Checkpoint
   ↓
Terminate
```

# 34. Command Sequence

Each assignment can have a monotonically increasing command sequence:

```text
1 Prepare
2 Start
3 Checkpoint
4 Terminate
```

# 35. Stale Command Rejection

An Agent should reject:

```text
command_sequence < current_sequence
```

when ordering is required.

# 36. Duplicate Command

If:

```text
StartAssignment(command=17)
```

arrives twice, the Agent should recognize the duplicate.

# 37. Idempotent Start

A duplicate Start command should result in:

```text
already_started
```

rather than creating a second process.

# 38. Command Acknowledgement

Acknowledgement should distinguish:

```text
received
validated
accepted
started
completed
```

These are different facts.

# 39. Received

Means:

```text
network/message layer accepted command
```

# 40. Accepted

Means:

```text
Agent validated command and committed local intent
```

# 41. Started

Means:

```text
execution was actually launched
```

# 42. Completed

Means:

```text
execution reached terminal state
```

# 43. Command Result

A command response can contain:

```text
command_id
assignment_id
status
execution_id
reason
timestamp
```

# 44. Execution Identity

Every actual process/run must have an:

```text
execution_id
```

This is distinct from:

```text
workflow_id
node_id
attempt
assignment_id
```

# 45. Identity Hierarchy

```text
Workflow
   │
   └── Node
        │
        └── Attempt
             │
             └── Assignment
                  │
                  └── Execution
```

This separation is essential for retries.

# 46. Process Launch

Before launching execution, the Agent validates:

```text
assignment identity
resource reservation
runtime availability
security policy
filesystem requirements
network policy
```

# 47. Preparation

Preparation may include:

```text
workspace creation
artifact retrieval
environment setup
credential injection
resource isolation
filesystem mounting
```

# 48. Preparation Failure

If preparation fails:

```text
PREPARING
   ↓
PREPARATION_FAILED
```

No successful execution should be reported.

# 49. Artifact Integrity

Downloaded execution artifacts should be verified where integrity metadata exists.

Examples:

```text
digest
signature
version
```

# 50. Artifact Identity

An execution should reference immutable artifact identities where possible.

# 51. Workspace Isolation

Each execution should have an isolated workspace where required.

Example:

```text
/var/lib/nros/work/
    execution-A/
    execution-B/
```

# 52. Filesystem Isolation

Execution should not automatically receive unrestricted access to the Agent filesystem.

# 53. Process Isolation

Depending on deployment requirements, NROS may use:

```text
process isolation
containers
namespaces
sandbox runtimes
WASM
VMs
```

# 54. Security Boundary

The Agent runtime is a security boundary.

Untrusted Work must not gain arbitrary control over:

```text
Controller credentials
other Work
Agent secrets
host filesystem
host network
```

unless explicitly authorized.

# 55. Resource Enforcement

The Agent should enforce local limits where possible.

Examples:

```text
CPU
memory
process count
filesystem size
network bandwidth
GPU access
```

# 56. Controller vs Agent Resource Authority

The Controller performs global placement.

The Agent performs local enforcement.

Therefore:

```text
Controller:
    "4 CPU requested"

Agent:
    "execution cannot exceed local 4 CPU allocation"
```

# 57. Resource Drift

If actual local allocation differs from Controller state:

```text
reported allocation
≠
authoritative assignment
```

the Agent should report the discrepancy.

# 58. Execution Supervision

The Agent should supervise child processes.

It should detect:

```text
exit
crash
signal
timeout
resource violation
orphaning
```

# 59. Parent-Child Relationship

The Agent must ensure that execution does not silently escape supervision.

# 60. Orphan Process

If the Controller believes execution ended but a child process remains active, the Agent must detect and clean up or report it.

# 61. Process Group

Where supported, the Agent should manage the execution as a process group or equivalent isolation unit.

This simplifies:

```text
termination
cleanup
resource accounting
```

# 62. Exit Status

The Agent should capture:

```text
exit_code
signal
termination_reason
runtime_duration
```

# 63. Result Classification

Execution termination can be classified as:

```text
SUCCESS
FAILED
CANCELLED
TIMED_OUT
PREEMPTED
LOST
RESOURCE_VIOLATION
INFRASTRUCTURE_FAILURE
```

# 64. Exit Code Is Not Enough

An exit code of:

```text
1
```

does not explain whether the failure was:

```text
application failure
Agent failure
timeout
resource kill
```

The Agent should preserve structured termination context.

# 65. Standard Output

Execution output should be captured according to configured policy.

# 66. Output Limits

Unlimited output can exhaust Agent storage or Controller transport.

Therefore:

```text
max_stdout
max_stderr
max_log_bytes
```

may be required.

# 67. Log Streaming

Logs may be streamed incrementally.

But streamed logs are not necessarily durable execution state.

# 68. Log Backpressure

If the Controller cannot consume logs quickly enough, the Agent must apply bounded buffering.

# 69. Log Loss Policy

The execution contract should specify whether logs are:

```text
lossless
best-effort
sampled
truncated
```

# 70. Result Reporting

Terminal execution evidence should contain:

```text
execution_id
assignment_id
terminal_state
exit information
resource usage
artifact references
output references
timestamps
```

# 71. Result Authentication

Result messages should be authenticated as originating from the expected Agent identity.

# 72. Result Sequence

Execution updates can use sequence numbers:

```text
1 PREPARING
2 STARTED
3 RUNNING
4 EXITED
5 RESULT_REPORTED
```

# 73. Duplicate Result

A duplicate terminal result must not cause the Workflow to transition twice.

# 74. Conflicting Results

If the Agent reports:

```text
SUCCESS
```

after already reporting:

```text
FAILED
```

the Controller must reject or explicitly reconcile the conflict.

# 75. Terminal State Immutability

Normally:

```text
SUCCEEDED
FAILED
CANCELLED
```

are terminal.

Changing them requires explicit recovery semantics.

# 76. Checkpointing

Checkpointing captures resumable execution state.

Possible checkpoint contents:

```text
memory/state
progress
application metadata
workspace state
input position
```

# 77. Checkpoint Identity

Each checkpoint requires:

```text
checkpoint_id
execution_id
sequence
created_at
artifact/reference
```

# 78. Checkpoint Durability

A checkpoint should not be considered durable merely because it exists in Agent memory.

# 79. Durable Checkpoint

The system should distinguish:

```text
CHECKPOINT_CREATED
```

from:

```text
CHECKPOINT_DURABLE
```

# 80. Checkpoint Upload

For remote checkpoint storage:

```text
create
→ upload
→ verify
→ commit
```

# 81. Checkpoint Resume

A retry may use:

```text
latest valid checkpoint
```

rather than starting from zero.

# 82. Checkpoint Compatibility

A checkpoint must be compatible with:

```text
runtime version
execution version
application version
architecture
```

where required.

# 83. Checkpoint Corruption

A corrupt checkpoint must be rejected rather than silently resumed.

# 84. Local Crash Recovery

If the Agent process crashes and restarts:

```text
load local execution journal
→ discover surviving processes
→ discover checkpoints
→ reconcile with Controller
```

# 85. Surviving Child Process

If the process survives the Agent process:

```text
execution remains UNKNOWN
```

until it can be safely identified.

# 86. PID Is Not Execution Identity

A PID may be reused.

Therefore:

```text
PID ≠ authoritative execution identity
```

# 87. Execution Metadata

The Agent should persist enough metadata to map local processes to:

```text
execution_id
assignment_id
```

# 88. Local Execution Journal

A local journal can record:

```text
prepare
launch
pid/process identity
checkpoint
exit
cleanup
```

This assists crash recovery.

# 89. Local Journal Durability

The Agent should define which local journal entries are durable before acknowledging state to the Controller.

# 90. Network Partition

During Controller disconnection, an Agent may continue existing Work according to policy.

# 91. Offline Execution Policy

Possible policies:

```text
continue existing
pause
terminate
reject new Work
```

# 92. New Work During Partition

Unless explicitly authorized:

```text
no Controller connection
→ no new externally authorized Work
```

# 93. Lease-Based Execution

An assignment may carry a lease:

```text
assignment_lease
```

The Agent can continue execution only while lease semantics permit it.

# 94. Lease Expiration

Expiration does not automatically prove Work should be killed.

The Controller may need reconciliation before termination.

# 95. Fencing Execution

For highly sensitive Work, execution can be fenced using an assignment epoch.

# 96. Cancellation

Cancellation should be explicit:

```text
RUNNING
   ↓
CANCEL_REQUESTED
   ↓
TERMINATING
   ↓
CANCELLED
```

# 97. Graceful Cancellation

The Agent should first request graceful termination where policy allows.

# 98. Forced Termination

If graceful termination exceeds its deadline:

```text
FORCE_TERMINATE
```

may be applied.

# 99. Cancellation Reason

Cancellation should carry:

```text
actor
reason
policy
timestamp
```

# 100. Timeout

Timeout should distinguish:

```text
startup timeout
execution timeout
checkpoint timeout
cleanup timeout
```

# 101. Resource Violation

If execution exceeds a hard local resource boundary:

```text
RESOURCE_VIOLATION
```

should be reported rather than a generic failure.

# 102. Agent-Level Fault

An Agent crash should not be interpreted as application failure.

The Controller must classify:

```text
application failure
vs
infrastructure failure
```

# 103. Retry Classification

A failure class should determine whether retry is appropriate.

Example:

```text
APPLICATION_FAILURE
    → usually no automatic retry

AGENT_FAILURE
    → potentially retry

NETWORK_FAILURE
    → reconcile first

RESOURCE_VIOLATION
    → policy-dependent
```

# 104. Retry Safety

Retry must not begin until the previous execution's terminal or fenced state is sufficiently established.

# 105. Duplicate Execution Prevention

The Controller should require:

```text
old execution terminal
OR
old execution fenced
```

before launching a replacement where duplicate execution is unsafe.

# 106. Agent Reconciliation Protocol

On reconnect:

```text
Controller → inventory request

Agent → execution inventory

Controller → compare

Mismatch → reconcile

Agreement → resume normal operation
```

# 107. Inventory

The Agent inventory should include:

```text
execution_id
assignment_id
state
resource usage
process identity
checkpoint
start time
```

# 108. Controller-Only Assignment

If Controller has:

```text
assignment A
```

but Agent reports:

```text
nothing
```

the assignment becomes:

```text
MISSING_EXECUTION
```

and requires reconciliation.

# 109. Agent-Only Execution

If Agent reports execution absent from Controller state:

```text
ORPHAN_EXECUTION
```

The Agent should not automatically continue indefinitely.

# 110. Reconciliation Outcomes

Possible results:

```text
CONFIRMED
REATTACHED
TERMINATE
RETRY
QUARANTINE
MANUAL_REVIEW
```

# 111. Agent Quarantine

An Agent may be quarantined when it repeatedly violates protocol invariants.

# 112. Quarantine Conditions

Examples:

```text
duplicate execution
invalid result
resource accounting corruption
authentication failure
protocol violation
```

# 113. Agent Protocol Version

The Agent and Controller negotiate a compatible protocol version.

# 114. Compatibility Matrix

A deployment may support:

```text
Controller v3
Agent v2
```

if explicitly compatible.

Otherwise registration fails.

# 115. Feature Negotiation

Optional protocol features can be negotiated:

```text
checkpointing
streaming logs
GPU isolation
remote artifacts
```

# 116. Protocol Extension

Unknown optional fields should not break compatible implementations.

# 117. Authentication

Agent communication must authenticate:

```text
Agent → Controller
Controller → Agent
```

# 118. Authorization

Authentication establishes identity.

Authorization determines:

```text
what that identity may do
```

# 119. Agent Credential Scope

Agent credentials should be limited to the operations required for that Agent.

# 120. Credential Rotation

Agent credentials should support rotation without requiring unnecessary execution downtime.

# 121. Secret Injection

Secrets required by Work should not automatically become visible to:

```text
Agent logs
diagnostics
Controller events
other Work
```

# 122. Environment Isolation

Each execution should receive only the environment it requires.

# 123. Network Isolation

Execution network access should be policy-controlled.

Possible policies:

```text
none
restricted
internal
full
```

# 124. Host Access

Host-level operations should require explicit authorization.

# 125. Device Access

Devices such as:

```text
GPU
USB
serial
camera
```

must be explicitly allocated.

# 126. Device Exclusivity

Exclusive devices require reservation semantics equivalent to other non-shareable resources.

# 127. Agent Storage

The Agent requires controlled local storage for:

```text
workspace
temporary files
logs
checkpoints
execution journal
```

# 128. Storage Pressure

If local storage approaches a critical threshold:

```text
Agent → DEGRADED
```

and scheduling eligibility may be reduced.

# 129. Cleanup

After terminal execution:

```text
processes
temporary files
mounts
network namespaces
resource reservations
```

must be cleaned.

# 130. Cleanup Failure

Cleanup failure should be observable.

The Agent should not report fully released resources while cleanup is incomplete.

# 131. Resource Release

Correct sequence:

```text
execution terminal
   ↓
cleanup
   ↓
verify resources released
   ↓
report release
```

# 132. Agent Observability

Metrics should include:

```text
active_executions
execution_start_rate
execution_failure_rate
heartbeat_latency
command_latency
resource_usage
cleanup_failures
checkpoint_rate
reconciliation_count
```

# 133. Execution Trace

Each execution can expose:

```text
prepared_at
started_at
first_output_at
checkpointed_at
finished_at
cleanup_completed_at
```

# 134. Agent Diagnostics

Diagnostics should expose:

```text
registration state
Controller connectivity
active assignments
resource health
execution processes
protocol errors
```

without exposing secrets.

# 135. Agent State Machine

A complete execution state machine may be:

```text
RECEIVED
   ↓
VALIDATING
   ↓
PREPARING
   ↓
PREPARED
   ↓
STARTING
   ↓
RUNNING
   ├──────────────┐
   │              │
   ▼              ▼
CHECKPOINTING   CANCEL_REQUESTED
   │              │
   ▼              ▼
RUNNING       TERMINATING
   │              │
   └──────┬───────┘
          ▼
       TERMINAL
          │
          ▼
       CLEANING
          │
          ▼
       RELEASED
```

# 136. Agent Protocol Invariants

```text
1. Every Agent has a stable identity.

2. Every Agent process has an instance identity.

3. Registration validates protocol compatibility and authorization.

4. Capabilities and resources are distinct concepts.

5. Heartbeat proves liveness, not execution success.

6. Execution has a unique execution_id.

7. Assignment identity and execution identity remain distinct.

8. Commands have stable identities.

9. Duplicate commands are idempotently handled.

10. Command ordering is explicit where required.

11. Stale commands cannot override newer assignment state.

12. Process supervision prevents uncontrolled orphan execution where possible.

13. PID alone is never authoritative execution identity.

14. Terminal execution results are idempotent.

15. Conflicting terminal results are explicitly rejected or reconciled.

16. Resource limits are enforced locally where technically possible.

17. Controller placement does not replace local enforcement.

18. Execution state remains observable after Agent restart.

19. Agent restart triggers reconciliation.

20. Controller restart triggers Agent reconciliation.

21. Unknown execution state is not silently treated as success or failure.

22. Retry does not create unsafe duplicate execution.

23. Checkpoints have explicit identity and durability semantics.

24. Corrupt checkpoints cannot be silently resumed.

25. Cancellation is explicit and auditable.

26. Forced termination is policy-controlled.

27. Cleanup completion precedes final resource release.

28. An Agent can enter draining mode.

29. Draining prevents new assignments while allowing existing Work to finish.

30. Network partitions do not grant implicit authorization for new Work.

31. Agent credentials are scoped and rotatable.

32. Secrets are isolated from ordinary diagnostics and logs.

33. Protocol versions are explicit.

34. Authentication and authorization are separate concepts.

35. Agent inventory is sufficient for reconciliation.

36. Orphan executions are explicitly classified.

37. Agent-only and Controller-only state mismatches are observable.

38. Resource drift is reported.

39. Protocol violations can trigger quarantine.

40. Agent health and schedulability are distinct.

41. The Agent never silently changes global scheduling policy.

42. External execution evidence is represented separately from control-plane intent.

43. Execution failure classification distinguishes application failure from infrastructure failure.

44. Local durable state is used to recover from Agent crashes.

45. No successful result is reported without corresponding execution evidence.
```

# 137. Canonical Agent Architecture

```text
                    NROS AGENT
                        │
        ┌───────────────┼────────────────┐
        │               │                │
   Control Client   Supervisor      Resource Manager
        │               │                │
        │               ▼                │
        │          Process Manager       │
        │               │                │
        │               ▼                │
        │          Executions            │
        │               │                │
        └───────────────┼────────────────┘
                        │
                  Local Journal
                        │
                        ▼
                 Reconciliation
```

# 138. Canonical Command Flow

```text
Controller
    │
    ▼
Durable Assignment
    │
    ▼
Outbox
    │
    ▼
Command Dispatcher
    │
    ▼
Agent
    │
    ├── validate command
    ├── deduplicate
    ├── reserve local resources
    ├── prepare workspace
    └── persist launch intent
              │
              ▼
           Execute
              │
              ▼
          Supervise
              │
              ▼
        Report Evidence
              │
              ▼
          Controller
```

# 139. Canonical Reconciliation

```text
             CONTROLLER
                 │
          expected assignments
                 │
                 ▼
             COMPARE
                 ▲
                 │
          actual executions
                 │
              AGENT

       ┌─────────┼──────────┐
       ▼         ▼          ▼
   MATCHED     AGENT-ONLY  CTRL-ONLY
       │           │          │
       ▼           ▼          ▼
   CONFIRMED    QUARANTINE   RECOVER
```

# 140. Canonical Execution Identity

```text
Workflow ID
     │
     ▼
Node ID
     │
     ▼
Attempt #3
     │
     ▼
Assignment ID
     │
     ▼
Execution ID
     │
     ├── Process group
     ├── Checkpoints
     ├── Logs
     └── Result
```

This hierarchy prevents retry identity from being confused with the logical Work identity.

# 141. Canonical Failure Classification

```text
                    TERMINATION
                         │
          ┌──────────────┼──────────────┐
          ▼              ▼              ▼
     Application      Infrastructure    Policy
        │                 │              │
        ▼                 ▼              ▼
      FAILED          AGENT_LOST       CANCELLED
                      NETWORK_LOST
                      RESOURCE_LOST
```

# 142. Agent Correctness Principle

> **An Agent must report execution facts based on local evidence, not infer success merely from command acceptance or Controller intent.**

# 143. Reconciliation Principle

> **When Controller state and Agent state disagree, NROS must enter an explicit reconciliation path rather than silently choosing one side.**

# 144. Execution Identity Principle

> **Workflow identity, attempt identity, assignment identity, and execution identity must remain distinct so retries and recovery cannot accidentally become duplicate logical execution.**

# 145. Security Principle

> **The Agent is an execution security boundary and must enforce local isolation even when the Controller has already authorized the Work globally.**

# 146. Final Architectural Rule

> **NROS Agents must provide authenticated, versioned, idempotent, resource-aware execution endpoints that can survive process crashes, network partitions, duplicate commands, Controller failover, and execution ambiguity while maintaining durable execution identity, explicit evidence, safe cleanup, checkpoint semantics, and deterministic reconciliation with Controller state.**

The next layer is:

# Part CXXVI — Workflow Definition & DSL Architecture, Schema, Versioning, Validation, DAG Semantics, Expressions, Parameters, Templates, Conditions, Loops, Fan-Out/Fan-In, Dynamic Graphs & Compilation

The central question becomes:

> **How is an NROS Workflow represented as a formally valid executable graph, how is that definition versioned and compiled, and how can dynamic control-flow constructs remain deterministic, auditable, schedulable, and safely executable?**

# NROS — Part CXXVI: Workflow Definition & DSL Architecture

The Workflow Definition is the declarative contract from which NROS derives executable Work.

It must separate:

```text
definition
execution
state
```

A definition describes **what may execute**.

Runtime state describes **what is currently happening**.

Execution records describe **what actually happened**.

The fundamental invariant is:

> **An executable Workflow must be derived from an immutable, versioned, validated definition.**

# 1. Workflow Definition Responsibility

The definition layer describes:

```text
Workflow identity
metadata
parameters
nodes
dependencies
conditions
resources
policies
artifacts
outputs
timeouts
retry policies
concurrency
control-flow constructs
```

# 2. Definition vs Instance

These must remain distinct.

```text
Workflow Definition
        │
        ├── version 1
        ├── version 2
        └── version 3
```

A running Workflow is an instance:

```text
Workflow Instance
    ↓
Definition Version 3
```

# 3. Immutable Definition

Once a definition version is published:

```text
definition_version = 3
```

it should be immutable.

A modification creates:

```text
definition_version = 4
```

# 4. Why Immutability Matters

A running Workflow must not silently change semantics because someone edited its source.

Without immutability:

```text
running instance
      ↓
definition changes
      ↓
runtime meaning changes
```

This destroys reproducibility.

# 5. Definition Identity

A definition can be identified by:

```text
workflow_type
definition_id
version
digest
```

# 6. Definition Digest

A canonical serialization can produce:

```text
definition_digest
```

This allows exact identification of the compiled definition.

# 7. Source vs Compiled Definition

NROS may maintain:

```text
Source DSL
    ↓
Parser
    ↓
AST
    ↓
Validator
    ↓
Intermediate Representation
    ↓
Compiler
    ↓
Executable Workflow Definition
```

# 8. AST

The Abstract Syntax Tree represents syntactic structure.

Example:

```text
workflow build {
    task compile
    task test
}
```

becomes conceptually:

```text
Workflow
 ├── Task(compile)
 └── Task(test)
```

# 9. Semantic IR

The semantic intermediate representation should remove irrelevant syntax and expose executable semantics.

Example:

```text
Node:
    id = "compile"
    dependencies = 
    executor = "shell"
```

# 10. Graph Representation

The compiled Workflow is normally represented as a directed graph.

```text
A ──→ B ──→ C
```

# 11. DAG

For ordinary dependency execution:

```text
Directed Acyclic Graph
```

is the fundamental structure.

# 12. DAG Invariant

A normal Workflow DAG must satisfy:

```text
no directed cycle
```

unless the DSL explicitly models iteration through a controlled construct.

# 13. Cycle Detection

Compilation must reject:

```text
A → B
B → C
C → A
```

# 14. Topological Ordering

A valid DAG has at least one topological order.

This supports deterministic scheduling analysis.

# 15. Node Identity

Each node requires a stable identifier:

```text
node_id
```

within the Workflow definition.

# 16. Node Identity Stability

Changing the node identifier can change:

```text
state identity
retry identity
artifact references
observability
```

Therefore node IDs should be treated as semantic identifiers.

# 17. Node Type

Examples:

```text
task
condition
parallel
join
map
reduce
subworkflow
approval
checkpoint
```

# 18. Task Node

A task describes executable Work:

```text
Task {
    id
    executor
    command
    inputs
    outputs
    resources
    timeout
    retry
}
```

# 19. Executor

The executor identifies the execution mechanism.

Examples:

```text
shell
container
wasm
python
remote
custom
```

# 20. Executor Contract

Every executor must define:

```text
input model
environment
resource model
result model
failure semantics
cleanup semantics
```

# 21. Parameters

Workflow definitions can declare parameters.

Example:

```text
parameter:
    name: environment
    type: string
    required: true
```

# 22. Parameter Types

Possible types:

```text
string
integer
float
boolean
duration
bytes
enum
list
map
object
```

# 23. Parameter Validation

Values should be validated before scheduling begins.

# 24. Defaults

Parameters can define defaults:

```text
retries = 3
```

But defaults must be part of the immutable definition.

# 25. Required Parameters

Missing required values must cause validation failure before execution.

# 26. Parameter Scope

Parameters can have scopes:

```text
workflow
node
subworkflow
iteration
```

# 27. Parameter Resolution

A reference may resolve through:

```text
explicit input
workflow parameter
node output
environment
secret reference
```

The precedence must be deterministic.

# 28. Expression Language

Conditions and parameter interpolation require an expression system.

The expression language should be:

```text
deterministic
bounded
typed
side-effect free
```

# 29. No Arbitrary Evaluation

A Workflow expression should not implicitly execute arbitrary host code.

Avoid:

```text
eval(user_string)
```

# 30. Expression Types

Expressions may support:

```text
literals
comparisons
boolean operators
arithmetic
string operations
collection operations
references
```

# 31. Expression Example

Conceptually:

```text
inputs.environment == "production"
```

# 32. Expression Result

Every expression should have a defined type.

For example:

```text
condition → boolean
timeout → duration
replicas → integer
```

# 33. Type Checking

The compiler should reject:

```text
timeout = "hello"
```

when timeout requires a duration.

# 34. Null Semantics

The DSL must define whether values can be null and how comparisons behave.

# 35. Undefined Values

A missing reference should not silently become an empty string.

It should produce an explicit evaluation result:

```text
UNDEFINED
```

or a validation error, according to context.

# 36. Secret References

Secrets should be referenced symbolically:

```text
secret("database_password")
```

rather than embedded in Workflow source.

# 37. Secret Evaluation

Secret values should be resolved as late as practical.

# 38. Secret Leakage

The execution engine must prevent secret values from appearing in:

```text
logs
diagnostics
events
error messages
Workflow metadata
```

unless explicitly authorized.

# 39. Templates

Templates allow reusable Workflow structures.

Conceptually:

```text
template deploy(service, environment)
```

# 40. Template Expansion

Templates should be expanded deterministically during compilation or instantiation.

# 41. Template Identity

Expanded nodes require stable generated identities.

Example:

```text
deploy.api.task
deploy.worker.task
```

# 42. Template Hygiene

Template variables must not accidentally capture unrelated identifiers.

# 43. Template Recursion

Unbounded recursive template expansion must be rejected.

# 44. Static Expansion Limit

Compilation should enforce bounded expansion:

```text
max_template_depth
max_generated_nodes
```

# 45. Conditions

A condition controls whether a node becomes executable.

Example:

```text
if tests_passed
    deploy
```

# 46. Conditional State

A skipped node must be distinguished from a node that never became eligible.

Possible states:

```text
PENDING
READY
RUNNING
SUCCEEDED
FAILED
SKIPPED
CANCELLED
```

# 47. Skip Semantics

A condition evaluating false can produce:

```text
SKIPPED
```

rather than:

```text

```

# 48. Dependency Semantics

A node can depend on:

```text
success
completion
failure
specific result
```

# 49. Dependency Policies

Examples:

```text
all_success
all_complete
any_success
any_complete
custom_condition
```

# 50. Fan-Out

Fan-out creates multiple Work items from one logical operation.

Example:

```text
input = [A, B, C]

        ↓

task(A)
task(B)
task(C)
```

# 51. Fan-Out Identity

Each generated execution needs deterministic identity.

Example:

```text
map_node
  item_index = 0
  item_key = A
```

# 52. Fan-In

Fan-in combines multiple branches.

```text
A ─┐
B ─┼──→ Join
C ─┘
```

# 53. Join Semantics

A Join must define:

```text
wait for all
wait for any
wait for threshold
collect successful
collect all results
```

# 54. Dynamic Fan-Out

Runtime-generated Work requires controlled limits.

Example:

```text
max_items = 1000
```

# 55. Unbounded Fan-Out

Unbounded expansion must be prohibited.

Otherwise one Workflow could create:

```text
millions of tasks
```

and overwhelm the scheduler.

# 56. Dynamic Graph

A dynamic Workflow can generate nodes during execution.

This creates a distinction:

```text
static definition graph
runtime expansion graph
```

# 57. Dynamic Graph Authority

Runtime-generated nodes must be persisted as authoritative Work definitions.

They must not exist only in Controller memory.

# 58. Dynamic Node Identity

Generated nodes require deterministic identity.

For example:

```text
parent_node
+
expansion_key
```

can derive a stable identity.

# 59. Loop Semantics

Loops should be explicit.

Example:

```text
repeat until condition
```

# 60. Loop Bound

Every loop must have a bounded or policy-controlled iteration limit.

```text
max_iterations = 100
```

# 61. Loop State

The runtime should persist:

```text
iteration_number
condition
result
```

# 62. Infinite Loop Protection

A Workflow must not be able to create infinite scheduling activity without explicit policy authorization.

# 63. Retry vs Loop

These are distinct.

Retry means:

```text
same logical Work
new execution attempt
```

Loop means:

```text
new logical iteration
```

# 64. Subworkflow

A node can invoke another Workflow definition.

```text
Parent Workflow
      │
      ▼
Subworkflow Instance
```

# 65. Subworkflow Identity

The child Workflow requires its own:

```text
workflow_instance_id
definition_version
```

# 66. Parent-Child Relationship

Persist:

```text
parent_workflow_id
parent_node_id
child_workflow_id
```

# 67. Subworkflow Completion

The parent node should complete based on the child Workflow's terminal result.

# 68. Subworkflow Failure

Failure semantics must be explicit:

```text
propagate
capture
retry
compensate
```

# 69. Compensation

Some Workflows require compensating operations.

Example:

```text
create resource
    ↓
deploy
    ↓
failure
    ↓
destroy resource
```

# 70. Compensation Is Not Rollback

External side effects cannot generally be rolled back atomically.

Compensation is a new Workflow action.

# 71. Resource Specification

A node may request:

```text
cpu
memory
storage
gpu
devices
network
custom resources
```

# 72. Resource Expressions

Resource requirements can depend on parameters.

Example:

```text
cpu = replicas * 2
```

The expression must resolve before scheduling.

# 73. Resource Bounds

Dynamic resource expressions must have safe bounds.

# 74. Timeout Specification

Nodes can define:

```text
queue_timeout
startup_timeout
execution_timeout
cleanup_timeout
```

# 75. Retry Policy

A retry policy may include:

```text
max_attempts
backoff
retryable_failures
jitter
```

# 76. Backoff

Retry delays should use controlled backoff.

Example:

```text
1s
2s
4s
8s
...
```

with a maximum.

# 77. Retry Classification

The definition should identify which failure classes are retryable.

# 78. Retry Identity

Each attempt increments:

```text
attempt
```

while retaining:

```text
workflow_id
node_id
```

# 79. Concurrency

Workflow definitions may limit:

```text
parallel nodes
parallel map items
subworkflow instances
resource usage
```

# 80. Concurrency Scope

Concurrency limits can apply to:

```text
workflow
tenant
node
template
resource class
```

# 81. Mutual Exclusion

Some nodes may require exclusive access to a named resource.

# 82. Resource Lock

Example:

```text
lock = production_database
```

The scheduler must serialize conflicting operations.

# 83. Approval Node

A Workflow may require external approval.

State:

```text
WAITING_APPROVAL
```

# 84. Approval Identity

An approval must identify:

```text
request_id
actor
decision
timestamp
```

# 85. Manual Intervention

Manual decisions should become durable events rather than undocumented UI state.

# 86. Workflow Metadata

Metadata can include:

```text
name
description
owner
labels
documentation
classification
```

# 87. Metadata vs Execution Inputs

Metadata should not be confused with mutable execution inputs.

# 88. Outputs

Workflow outputs should be declared.

Example:

```text
outputs:
    image_digest
    test_report
```

# 89. Output References

Large outputs should generally be referenced by durable artifact identity rather than embedded directly into state.

# 90. Artifact Contract

An artifact reference can contain:

```text
artifact_id
digest
media_type
size
storage_reference
```

# 91. Determinism

The compiler should produce deterministic output for identical:

```text
source
parameters
compiler version
dependency versions
```

# 92. Compiler Version

The compiled definition should record:

```text
compiler_version
```

# 93. Compiler Reproducibility

A definition digest should be reproducible from the same canonical inputs.

# 94. Canonical Serialization

Fields should have deterministic ordering where hashes or signatures depend on serialization.

# 95. Definition Signature

Production systems may sign published Workflow definitions.

This provides provenance.

# 96. Definition Trust

The Controller can require:

```text
trusted signer
```

for production Workflows.

# 97. Validation Phases

Validation should occur in layers:

```text
1 syntax
2 schema
3 types
4 graph
5 semantics
6 policy
7 resource
8 security
```

# 98. Syntax Validation

Reject malformed DSL.

# 99. Schema Validation

Reject structurally invalid objects.

# 100. Type Validation

Reject invalid expression and parameter types.

# 101. Graph Validation

Reject:

```text
cycles
unknown dependencies
duplicate node IDs
unreachable required nodes
invalid joins
```

# 102. Semantic Validation

Examples:

```text
retry policy incompatible with executor
invalid output reference
missing required parameter
invalid loop condition
```

# 103. Policy Validation

Examples:

```text
forbidden executor
forbidden network access
excessive resource request
unauthorized secret
```

# 104. Resource Validation

Verify that resource expressions resolve to valid values and respect configured maximums.

# 105. Security Validation

Reject unsafe combinations before execution.

# 106. Compile Errors

Errors should include:

```text
location
node
field
error_code
message
suggestion
```

# 107. Stable Error Codes

Example:

```text
NROS-WF-E001
NROS-WF-E002
```

Stable codes improve automation.

# 108. Source Locations

DSL compiler diagnostics should preserve:

```text
file
line
column
```

where applicable.

# 109. Definition Lifecycle

A definition may transition:

```text
DRAFT
   ↓
VALIDATED
   ↓
PUBLISHED
   ↓
DEPRECATED
   ↓
RETIRED
```

# 110. Draft

Draft definitions may be edited.

# 111. Validated

Validated means the definition passed compiler and policy checks.

# 112. Published

Published means it is available for Workflow instantiation.

# 113. Deprecated

Deprecated means new instances may be prohibited while existing instances continue.

# 114. Retired

Retired means the definition can no longer be instantiated.

# 115. Version Selection

Workflow creation must specify an immutable definition version.

Never rely on:

```text
"latest"
```

for reproducible execution.

# 116. Latest Alias

A UI may expose:

```text
latest
```

but it must resolve to an exact version before creating the Workflow instance.

# 117. Definition Migration

Changing definition version during execution is not equivalent to editing the definition.

It requires an explicit migration protocol.

# 118. Workflow Migration

A migration must define:

```text
old definition
new definition
state mapping
node mapping
compatibility
```

# 119. Migration Safety

A migration should not silently reinterpret completed execution history.

# 120. Runtime Expansion Record

Dynamic constructs should persist their expansion:

```text
expansion_id
source_node
expansion_input
generated_nodes
definition_digest
```

# 121. Dynamic Expansion Determinism

Given the same expansion input and definition version, expansion should produce the same logical graph.

# 122. Runtime Data Dependency

If expansion depends on external data, that input must become part of the durable execution evidence.

# 123. External Lookup

A Workflow should not assume:

```text
external lookup
```

is deterministic.

If its result affects graph structure, persist the result or digest.

# 124. Time-Dependent Expressions

Expressions such as:

```text
now()
```

can destroy reproducibility.

Such values should be explicitly modeled as runtime inputs.

# 125. Randomness

Random values must be controlled by a persisted seed if deterministic replay is required.

# 126. Environment Dependence

Implicit environment state should be minimized.

Prefer explicit:

```text
parameters
artifacts
runtime constraints
```

# 127. Compilation Boundary

The compiler converts:

```text
human-authored definition
```

into:

```text
machine-executable representation
```

The runtime should not repeatedly reinterpret raw source unless explicitly designed to do so.

# 128. Compiled IR

The IR can contain:

```text
node table
dependency table
expression bytecode/AST
resource requirements
retry policies
timeouts
security requirements
output declarations
```

# 129. IR Stability

The IR format should be versioned.

# 130. Compiler/Runtime Compatibility

The runtime must reject unsupported IR versions rather than guessing.

# 131. Workflow Admission

Before a Workflow instance is created:

```text
definition exists
definition published
version supported
parameters valid
policy valid
```

must all hold.

# 132. Instantiation

Instantiation binds:

```text
definition version
parameter values
execution identity
tenant/context
```

# 133. Immutable Instance Inputs

The resolved inputs used to instantiate a Workflow should be preserved.

# 134. Input Digest

A canonical digest can identify the exact input set.

# 135. Reproducibility Tuple

A Workflow execution can be identified by:

```text
definition_digest
compiler_version
parameter_digest
runtime_constraints
```

# 136. Workflow Graph Invariants

```text
1. Node IDs are unique within a definition.

2. Dependencies reference existing nodes.

3. Ordinary dependency graphs are acyclic.

4. Every executable node has an executor or control-flow semantics.

5. Every dynamic expansion has bounded growth.

6. Every loop has explicit termination semantics.

7. Every expression has defined typing.

8. Every external data dependency is explicit.

9. Every published definition is immutable.

10. Every Workflow instance binds to an exact definition version.

11. Definition digests are deterministic.

12. Compiler versions are recorded.

13. Generated node identities are deterministic.

14. Runtime-generated graph state is durable.

15. Secrets are referenced rather than embedded.

16. Resource expressions are validated before scheduling.

17. Retry and loop semantics are distinct.

18. Output references are typed and durable.

19. Invalid definitions cannot enter execution.

20. Policy validation precedes admission.

21. Security constraints are checked before execution.

22. Dynamic graph expansion cannot bypass admission policy.

23. Runtime data affecting graph semantics becomes durable evidence.

24. Time and randomness are explicitly modeled where reproducibility matters.

25. Definition migration requires explicit state mapping.
```

# 137. Canonical Compilation Pipeline

```text
              DSL SOURCE
                  │
                  ▼
               LEXER
                  │
                  ▼
               PARSER
                  │
                  ▼
                 AST
                  │
                  ▼
          SCHEMA VALIDATION
                  │
                  ▼
             TYPE CHECKER
                  │
                  ▼
           GRAPH VALIDATOR
                  │
                  ▼
          SEMANTIC VALIDATOR
                  │
                  ▼
           POLICY VALIDATOR
                  │
                  ▼
         SECURITY VALIDATOR
                  │
                  ▼
          RESOURCE VALIDATOR
                  │
                  ▼
              SEMANTIC IR
                  │
                  ▼
          CANONICAL SERIALIZE
                  │
                  ▼
            DEFINITION DIGEST
                  │
                  ▼
             PUBLISHED
```

# 138. Canonical Runtime Graph

```text
                    START
                      │
          ┌───────────┴───────────┐
          ▼                       ▼
       Prepare                  Validate
          │                       │
          └───────────┬───────────┘
                      ▼
                   Execute
                      │
             ┌────────┴────────┐
             ▼                 ▼
          Success            Failure
             │                 │
             ▼                 ▼
           Join              Retry?
             │                 │
             └───────┬─────────┘
                     ▼
                    END
```

# 139. Canonical Fan-Out/Fan-In

```text
                 MAP
                  │
       ┌──────────┼──────────┐
       ▼          ▼          ▼
      A-0        A-1        A-2
       │          │          │
       └──────────┼──────────┘
                  ▼
                 JOIN
                  │
                  ▼
                NEXT
```

# 140. Canonical Dynamic Expansion

```text
Static Node
    │
    ▼
Evaluate Expansion Input
    │
    ▼
Validate Bounds
    │
    ▼
Generate Graph
    │
    ▼
Persist Expansion
    │
    ▼
Schedule Generated Nodes
```

# 141. Canonical Definition Identity

```text
Source
  +
Compiler Version
  +
Dependencies
  +
Canonical Serialization
        │
        ▼
Definition Digest
        │
        ▼
Immutable Definition Version
```

# 142. Canonical Workflow Identity

```text
Definition
     │
     ├── version
     └── digest
          │
          ▼
Workflow Instance
     │
     ├── parameter digest
     ├── runtime context
     └── instance ID
```

# 143. DSL Design Principle

> **The DSL should describe intent, not expose arbitrary access to the Controller's implementation internals.**

# 144. Compiler Principle

> **Compilation is an admission-control boundary: malformed, ambiguous, cyclic, unsafe, unbounded, or unauthorized Workflow definitions must fail before execution.**

# 145. Determinism Principle

> **Identical definitions, compiler inputs, and explicit runtime inputs must produce identical executable semantics.**

# 146. Dynamic Graph Principle

> **Dynamic Workflow expansion must remain bounded, durable, identifiable, policy-validated, and reproducible from recorded inputs.**

# 147. Final Architectural Rule

> **NROS Workflow definitions must be immutable, versioned, typed, validated, canonically represented, and compiled into a stable executable graph whose dependencies, conditions, retries, loops, fan-out/fan-in, resource requirements, security constraints, and dynamic expansions are explicit and auditable before Work reaches the scheduler.**

The next layer is:

# Part CXXVII — Scheduler Architecture, Admission Control, Queueing, Priority, Fairness, Resource Matching, Reservations, Backfilling, Preemption, Deadlines, Quotas, Multi-Tenancy & Scheduling Correctness

The central question becomes:

> **Given a validated executable Workflow graph and a fleet of Agents with finite resources, how does NROS deterministically decide which Work is admitted, which Work runs next, where it runs, and how fairness, priority, quotas, reservations, deadlines, and preemption interact without violating safety invariants?**

# NROS — Part CXXVII: Scheduler Architecture

The Scheduler is the decision engine between **validated Work** and **available execution capacity**.

Its responsibility is not merely to select the next task.

It must continuously answer:

```text
What Work is eligible?
What Work is allowed?
What Work should run next?
Where can it run safely?
What resources must be reserved?
What policy constraints apply?
Can the decision be committed atomically?
```

The central invariant is:

> **A scheduling decision is valid only when eligibility, policy, resource availability, ownership, and persistence state all agree.**

# 1. Scheduler Responsibility

The Scheduler coordinates:

```text
admission
eligibility
prioritization
queueing
resource matching
reservation
placement
fairness
quotas
deadlines
preemption
concurrency
backpressure
```

# 2. Scheduler Non-Responsibility

The Scheduler should not:

```text
interpret arbitrary Workflow source
execute application code
invent resource capacity
bypass authorization
directly mutate Agent process state
```

It consumes compiled Workflow semantics and authoritative cluster state.

# 3. Scheduler Inputs

A scheduling cycle may consume:

```text
READY nodes
Workflow state
Agent state
Agent capabilities
resource capacity
reservations
leases
quotas
priorities
deadlines
tenant policy
concurrency limits
preemption policy
```

# 4. Scheduler Output

The Scheduler produces a proposed decision:

```text
Assignment {
    workflow_id
    node_id
    attempt
    agent_id
    resource_request
    reservation
    scheduler_epoch
}
```

# 5. Decision vs Commit

The Scheduler first computes:

```text
decision
```

Then persistence commits:

```text
assignment
+
reservation
+
state transition
```

The decision is not authoritative until committed.

# 6. Scheduling Cycle

Conceptually:

```text
Observe
   ↓
Filter
   ↓
Rank
   ↓
Match
   ↓
Reserve
   ↓
Commit
   ↓
Dispatch
   ↓
Observe again
```

# 7. Scheduler Triggering

Scheduling can be triggered by:

```text
Work becoming READY
Agent registration
Agent heartbeat
resource release
reservation expiration
priority change
deadline change
Workflow event
reconciliation
```

# 8. Event-Driven Scheduling

Event-driven scheduling avoids continuously scanning the entire system.

# 9. Periodic Scheduling

A periodic reconciliation cycle should still exist.

This catches:

```text
missed events
stale queues
projection corruption
expired reservations
```

# 10. Scheduler Queue

The ready queue is a derived structure.

```text
Authoritative State
       ↓
READY projection
       ↓
Scheduler Queue
```

# 11. Queue Correctness

If the queue becomes corrupted:

```text
rebuild from authoritative READY state
```

must be possible.

# 12. Eligibility

A node is eligible only if all required conditions hold.

Conceptually:

```text
READY
AND dependencies satisfied
AND policy allows
AND quota available
AND concurrency allows
AND required resources can be considered
AND Workflow not suspended
```

# 13. Eligibility vs Feasibility

These are different.

Eligibility asks:

> May this Work run?

Feasibility asks:

> Can it run now on at least one valid Agent?

# 14. Eligible but Not Feasible

Example:

```text
GPU task
+
no GPU capacity
```

The task remains eligible but cannot currently be placed.

# 15. Admission Control

Admission prevents the scheduler from accepting Work that cannot be safely supported.

# 16. Admission Checks

Potential checks:

```text
tenant quota
Workflow quota
resource maximum
security policy
executor availability
definition validity
deadline feasibility
cluster capacity policy
```

# 17. Hard Admission Failure

An admission failure should be explicit:

```text
REJECTED
```

rather than silently leaving the Work indefinitely queued.

# 18. Admission vs Queueing

A Work item can be:

```text
accepted
but waiting
```

This is fundamentally different from:

```text
rejected
```

# 19. Queue States

Useful conceptual states:

```text
PENDING
READY
QUEUED
BLOCKED
RESERVED
DISPATCHING
RUNNING
```

# 20. Priority

Every schedulable Work item may have a priority.

Example:

```text
priority = 100
```

Higher priority may normally receive earlier consideration.

# 21. Priority Is Not Absolute

Priority must not automatically override:

```text
quota
authorization
resource constraints
fairness policy
safety
```

# 22. Priority Bands

A system may define:

```text
critical
high
normal
low
```

# 23. Priority Inversion

A high-priority task can be blocked by a resource held by low-priority Work.

The scheduler should recognize this explicitly.

# 24. Aging

Waiting Work can receive increasing effective priority.

Conceptually:

```text
effective_priority =
base_priority + aging_factor × wait_time
```

# 25. Aging Bounds

Aging must be bounded so that priorities remain predictable.

# 26. Fairness

Fairness prevents one tenant or Workflow from monopolizing capacity.

# 27. Tenant Fairness

Example:

```text
Tenant A: 50 running
Tenant B: 1 running
```

If both are continuously eligible, the Scheduler should apply configured fairness.

# 28. Fair-Share

A fair-share model can assign each tenant a dynamic score.

Example:

```text
share_score =
desired_share - recent_usage
```

Lower usage can increase scheduling preference.

# 29. Weighted Fairness

Tenants can have weights:

```text
Tenant A = weight 2
Tenant B = weight 1
```

This does not mean A always receives exactly twice the resources.

It means scheduling preference reflects configured weights.

# 30. Hierarchical Fairness

Fairness can operate at:

```text
organization
  ↓
tenant
  ↓
project
  ↓
workflow
  ↓
node
```

# 31. Quotas

A quota limits resource consumption.

Examples:

```text
max_cpu
max_memory
max_gpu
max_running_tasks
max_storage
```

# 32. Hard Quota

Hard quota:

```text
cannot be exceeded
```

# 33. Soft Quota

Soft quota can be exceeded temporarily under defined conditions.

This must be explicit.

# 34. Quota Scope

Quota may apply to:

```text
tenant
project
Workflow
Agent pool
resource class
```

# 35. Hierarchical Quota

A Work item must satisfy every applicable quota:

```text
organization quota
AND
tenant quota
AND
project quota
AND
Workflow quota
```

# 36. Quota Reservation

Quota consumption should be reserved atomically with resource reservation where required.

# 37. Quota Release

Quota should be released only after the corresponding resource ownership is safely released.

# 38. Resource Model

NROS resources can be modeled as vectors:

```text
CPU
Memory
Storage
GPU
Network
Custom
```

# 39. Resource Request

Example:

```text
CPU = 4
Memory = 8 GiB
GPU = 1
```

# 40. Resource Capacity

Agent capacity:

```text
CPU = 16
Memory = 32 GiB
GPU = 2
```

# 41. Resource Feasibility

A placement is feasible if:

```text
request <= available capacity
```

for all required dimensions.

# 42. Scalar vs Vector Resources

CPU and memory are divisible.

Some resources are effectively discrete:

```text
GPU
device
license
```

# 43. Exclusive Resources

Exclusive resources require ownership rather than simple subtraction.

# 44. Custom Resources

NROS can support named resources:

```text
license:ansys = 2
device:serial0 = 1
```

# 45. Resource Labels

Agents can expose attributes:

```text
region=eu
arch=x86_64
gpu=nvidia
tier=high-memory
```

# 46. Placement Constraints

A node can specify:

```text
region == "eu"
gpu == "nvidia"
arch == "x86_64"
```

# 47. Affinity

Affinity prefers certain placements.

Example:

```text
prefer same Agent as previous node
```

# 48. Anti-Affinity

Anti-affinity prevents colocating certain Work.

Example:

```text
replica A
replica B
```

must run on different Agents.

# 49. Topology Awareness

Resources can be grouped by:

```text
Agent
rack
zone
region
cluster
```

# 50. Fault Domains

Scheduling replicas across fault domains reduces correlated failure.

# 51. Reservation

A reservation establishes future ownership of resources.

Example:

```text
Agent A
CPU 4
reserved for Assignment X
```

# 52. Reservation vs Allocation

Reservation means:

```text
held for intended Work
```

Allocation means:

```text
actively consumed by running Work
```

# 53. Reservation Lifecycle

```text
PROPOSED
   ↓
RESERVED
   ↓
ALLOCATED
   ↓
RELEASED
```

# 54. Reservation Expiration

Reservations must not live forever.

They require:

```text
expiration
renewal
or explicit release
```

# 55. Stale Reservation

A stale reservation can block capacity indefinitely.

The Controller must reconcile and expire it safely.

# 56. Scheduling Commit

The reservation and assignment should be committed atomically when they form one correctness boundary.

# 57. Placement Algorithm

A conceptual placement algorithm:

```text
for each eligible Work:
    candidates = compatible Agents
    filter unavailable Agents
    filter capability mismatch
    filter resource mismatch
    filter affinity violations
    filter quota violations
    rank candidates
    select candidate
    commit reservation
```

# 58. Candidate Filtering

Filtering should occur before expensive ranking.

# 59. Candidate Ranking

Possible factors:

```text
priority
fairness
locality
fragmentation
deadline
load
energy
cost
```

# 60. Scoring

A scheduler may calculate:

```text
score =
priority_score
+ fairness_score
+ locality_score
+ deadline_score
- fragmentation_penalty
```

The exact formula must be documented and deterministic enough for testing.

# 61. Deterministic Tie-Breaking

If two candidates have equal scores, use a deterministic tie-breaker.

Example:

```text
Agent ID ascending
```

# 62. Why Tie-Breaking Matters

Without deterministic tie-breaking:

```text
same state
→ different scheduling decisions
```

may occur across runs.

# 63. Scheduling Epoch

Every scheduler leadership generation should carry an epoch:

```text
scheduler_epoch
```

# 64. Stale Scheduler Decision

A decision created under epoch 17 must not commit under epoch 18 if fencing rules prohibit it.

# 65. Concurrent Schedulers

Multiple scheduler workers may operate concurrently.

They must coordinate through:

```text
transactional claims
version checks
leases
epochs
```

# 66. Work Claim

A scheduler worker can temporarily claim a queue item.

Example:

```text
READY
   ↓
CLAIMED
```

# 67. Claim Expiration

Claims must expire or be recoverable if the scheduler worker crashes.

# 68. Duplicate Scheduling

Two scheduler workers may observe the same Work.

Only one should successfully commit the assignment.

# 69. Optimistic Scheduling

A worker can optimistically compute a placement and let the State Store reject stale commits.

# 70. Scheduling Race

Example:

```text
Scheduler A sees 4 CPU free
Scheduler B sees 4 CPU free

A reserves 4 CPU
B tries reserve 4 CPU
```

B must fail atomically.

# 71. Resource Overcommit

Overcommit must be explicit.

For example:

```text
memory overcommit = 1.2
```

should never be accidental.

# 72. Hard Resource Guarantees

For non-overcommittable resources:

```text
allocated + requested <= capacity
```

must hold.

# 73. Soft Resources

Some resources can be oversubscribed according to policy.

The scheduler must distinguish:

```text
guaranteed
best-effort
burstable
```

# 74. Backfilling

Backfilling allows small Work to run while a larger reserved Work waits.

Example:

```text
Large Job:
needs 16 CPU
starts at T=10

Small Job:
needs 2 CPU
can finish before T=10
```

The small job may be admitted without delaying the reservation.

# 75. Backfill Safety

Backfilling must preserve:

```text
reservation guarantee
deadline guarantee
fairness policy
```

# 76. Reservation Horizon

The scheduler may calculate when reserved capacity becomes required.

# 77. Fragmentation

Resource fragmentation can cause:

```text
total capacity sufficient
but no single Agent can satisfy request
```

# 78. Fragmentation Awareness

The scheduler may prefer placements that preserve large contiguous resource blocks.

# 79. Bin Packing

Scheduling can use algorithms such as:

```text
best-fit
first-fit
worst-fit
dominant-resource fairness
```

depending on requirements.

# 80. Multi-Resource Fairness

When resources differ substantially, CPU-only fairness is insufficient.

# 81. Dominant Resource

A tenant's dominant resource is its largest proportional share.

Example:

```text
CPU share = 20%
GPU share = 70%

dominant share = 70%
```

# 82. Preemption

Preemption allows higher-priority Work to reclaim resources.

# 83. Preemption Is Dangerous

Preemption can create:

```text
lost progress
checkpoint cost
duplicate execution
thrashing
```

# 84. Preemption Eligibility

Only explicitly preemptible Work should be preempted unless emergency policy permits otherwise.

# 85. Preemption Cost

The scheduler should consider:

```text
checkpoint availability
restart cost
progress
cleanup time
priority difference
```

# 86. Graceful Preemption

Preferred sequence:

```text
PREEMPT_REQUESTED
   ↓
CHECKPOINT
   ↓
TERMINATE
   ↓
RELEASE
   ↓
NEW WORK
```

# 87. Forced Preemption

If graceful preemption fails:

```text
FORCE_TERMINATE
```

may be applied according to policy.

# 88. Preemption Debt

Repeatedly preempted Work can accumulate starvation.

Fairness logic should account for this.

# 89. Starvation

A Work item starves when it remains eligible but repeatedly loses scheduling decisions.

# 90. Anti-Starvation

Mechanisms include:

```text
aging
fair-share
minimum service guarantees
priority boosts
reservation
```

# 91. Deadline Scheduling

Work may have deadlines.

The scheduler can calculate:

```text
slack =
deadline - estimated_completion_time
```

# 92. Deadline Urgency

Lower slack generally means greater urgency.

# 93. Impossible Deadline

If the scheduler can prove:

```text
minimum feasible completion > deadline
```

the Workflow should become:

```text
DEADLINE_INFEASIBLE
```

rather than remaining silently queued.

# 94. Deadline vs Priority

Deadline urgency and static priority are separate scheduling dimensions.

# 95. Deadline Policy

Possible policy:

```text
priority first
deadline first
weighted combination
strict deadline class
```

# 96. Concurrency Limits

Workflow definitions may specify:

```text
max_parallel = 10
```

The Scheduler must enforce this globally for the Workflow instance.

# 97. Distributed Concurrency

Concurrency counters must be authoritative.

Local scheduler memory is insufficient.

# 98. Global Mutex

Some Workflows require serialized execution.

Example:

```text
max_parallel = 1
```

This is effectively a distributed concurrency constraint.

# 99. Tenant Concurrency

A tenant may have:

```text
max_running = 100
```

regardless of how many Workflow instances exist.

# 100. Queue Isolation

One noisy tenant should not prevent another tenant from making progress.

# 101. Scheduler Pools

Resources can be partitioned into pools:

```text
general
gpu
high-memory
trusted
edge
```

# 102. Pool Membership

An Agent can belong to multiple logical pools according to policy.

# 103. Pool Quotas

Each pool may have independent limits.

# 104. Queue Class

Work may target a queue class:

```text
interactive
batch
latency-sensitive
offline
```

# 105. Queue Policy

Different queues can use different:

```text
priority
fairness
preemption
deadline
backfill
```

policies.

# 106. Multi-Tenancy

Every scheduling decision should preserve tenant isolation.

# 107. Tenant Context

A Work item should carry:

```text
tenant_id
project_id
authorization_context
```

# 108. Tenant Policy

The Scheduler consumes tenant policy but should not independently invent authorization semantics.

# 109. Tenant Fairness

Fairness should be measured against the configured tenant hierarchy.

# 110. Noisy Neighbor Protection

One tenant consuming excessive capacity should trigger:

```text
throttling
quota enforcement
fairness reduction
```

rather than uncontrolled cluster domination.

# 111. Admission Backpressure

If the system reaches a safe operational limit:

```text
new Work → QUEUED or REJECTED
```

depending on policy.

# 112. Scheduler Backpressure

The Scheduler should bound:

```text
queue memory
decision rate
dispatch rate
database writes
Agent command rate
```

# 113. Dispatch Rate

A Scheduler should not flood an Agent with commands beyond its control-plane capacity.

# 114. Agent Saturation

If an Agent becomes saturated:

```text
scheduler preference ↓
```

unless policy requires otherwise.

# 115. Queue Latency

Important metrics:

```text
time_ready_to_queued
time_queued_to_reserved
time_reserved_to_started
```

# 116. Scheduling Metrics

Track:

```text
scheduling_cycles
placement_attempts
placement_failures
reservation_conflicts
queue_depth
fairness_score
preemptions
backfills
deadline_misses
```

# 117. Placement Failure Reason

Do not report only:

```text
"no capacity"
```

Prefer structured reasons:

```text
NO_GPU
QUOTA_EXCEEDED
CAPABILITY_MISMATCH
AFFINITY_CONFLICT
POOL_EXHAUSTED
CONCURRENCY_LIMIT
POLICY_DENIED
```

# 118. Explainability

Every scheduling decision should be explainable.

Example:

```text
Selected Agent A because:
    capability matched
    quota available
    priority highest
    fairness threshold satisfied
    resource fit optimal
```

# 119. Scheduling Decision Record

A durable or reconstructible decision record can contain:

```text
decision_id
scheduler_epoch
workflow_id
node_id
candidate_set
selected_agent
score
policy_version
resource_snapshot/version
```

# 120. Decision Reproducibility

For critical scheduling modes, the same input state should produce the same decision.

# 121. Policy Version

Scheduling policy should be versioned.

# 122. Policy Change

Changing fairness or priority rules should not silently reinterpret historical scheduling decisions.

# 123. Scheduler Configuration

Configuration should be:

```text
versioned
validated
auditable
```

# 124. Configuration Rollout

Policy changes should support controlled rollout.

# 125. Scheduler Failure

If the Scheduler process crashes:

```text
authoritative assignments survive
```

and a replacement scheduler reconstructs pending Work.

# 126. In-Flight Decision

An uncommitted decision is not authoritative.

The new scheduler may recompute it.

# 127. Committed Assignment

A committed assignment survives Scheduler restart.

# 128. Dispatch Failure

If:

```text
assignment committed
dispatch failed
```

the assignment becomes pending reconciliation rather than being silently discarded.

# 129. Dispatch Retry

The outbox/command mechanism can retry delivery using the same:

```text
assignment_id
command_id
```

# 130. Scheduler Reconciliation

The scheduler should periodically compare:

```text
READY state
assignments
reservations
Agent inventories
```

to detect drift.

# 131. Stale Assignment

An assignment with an expired lease or unreachable Agent requires explicit recovery.

# 132. Resource Leak

If an assignment terminates but reservation remains:

```text
resource leak
```

must be detected.

# 133. Scheduling Safety Boundary

The Scheduler must never:

```text
assign Work without authorization
over-allocate hard resources
ignore expired authority
create duplicate assignment identity
bypass concurrency limits
```

# 134. Scheduler Liveness

The Scheduler should eventually make progress when:

```text
eligible Work exists
+
compatible capacity exists
+
policy permits execution
+
persistence is healthy
```

# 135. Safety vs Liveness

These must remain distinct.

Safety:

> Nothing invalid happens.

Liveness:

> Valid Work eventually progresses.

# 136. Scheduler Invariants

```text
1. Only eligible Work enters placement.

2. Authorization precedes scheduling.

3. Hard quotas cannot be exceeded.

4. Hard resource capacity cannot be over-allocated.

5. Reservations and assignments obey transactional consistency.

6. Scheduler decisions are not authoritative until committed.

7. Stale scheduler epochs cannot create authoritative assignments.

8. Duplicate scheduling attempts cannot create duplicate assignments.

9. Queue state is rebuildable.

10. Agent capability mismatches prevent placement.

11. Resource mismatches prevent placement.

12. Concurrency limits are enforced authoritatively.

13. Tenant isolation is preserved.

14. Fairness policy cannot bypass safety constraints.

15. Priority cannot bypass authorization.

16. Preemption is policy-controlled.

17. Preemption does not silently create duplicate execution.

18. Backfilling cannot violate reservations.

19. Deadline handling is explicit.

20. Impossible deadlines become observable states.

21. Reservation expiration is recoverable.

22. Scheduling decisions are explainable.

23. Policy versions are recorded.

24. Configuration changes are auditable.

25. Scheduler memory is never the sole source of scheduling truth.

26. Scheduler crashes do not erase committed assignments.

27. Dispatch failures remain visible.

28. Resource leaks are detectable.

29. Starvation is monitored.

30. Queue backpressure is bounded.

31. Derived queues can be reconstructed.

32. Scheduling progress is eventually possible when all required conditions are satisfied.

33. A scheduling decision never assumes capacity that the authoritative state does not support.

34. External Agent execution remains separately reconciled.
```

# 137. Canonical Scheduler Architecture

```text
                    SCHEDULER
                        │
             ┌──────────┼──────────┐
             ▼          ▼          ▼
         Eligibility  Policy    Fairness
             │          │          │
             └──────────┼──────────┘
                        ▼
                 Candidate Filter
                        │
                        ▼
                  Resource Match
                        │
                        ▼
                    Ranking
                        │
                        ▼
                 Placement Decision
                        │
                        ▼
                 Transaction Commit
                        │
             ┌──────────┴──────────┐
             ▼                     ▼
        Assignment              Reservation
             │                     │
             └──────────┬──────────┘
                        ▼
                     Outbox
                        │
                        ▼
                      Agent
```

# 138. Canonical Scheduling Loop

```text
       ┌──────────────────────────┐
       │ Observe authoritative    │
       │ state                    │
       └────────────┬─────────────┘
                    ▼
             Find eligible Work
                    │
                    ▼
              Apply policy
                    │
                    ▼
              Apply fairness
                    │
                    ▼
           Find valid candidates
                    │
                    ▼
             Rank candidates
                    │
                    ▼
            Commit placement
                    │
             ┌──────┴──────┐
             ▼             ▼
          SUCCESS        CONFLICT
             │             │
             ▼             ▼
          Dispatch      Recompute
```

# 139. Canonical Priority/Fairness Relationship

```text
              SAFETY
                 │
                 ▼
          AUTHORIZATION
                 │
                 ▼
             QUOTAS
                 │
                 ▼
           ELIGIBILITY
                 │
        ┌────────┴────────┐
        ▼                 ▼
    PRIORITY          FAIRNESS
        │                 │
        └────────┬────────┘
                 ▼
             PLACEMENT
```

Safety constraints remain dominant.

# 140. Canonical Resource Matching

```text
Work Request
     │
     ├── CPU
     ├── Memory
     ├── GPU
     ├── Capabilities
     ├── Labels
     ├── Affinity
     └── Security
          │
          ▼
     Candidate Agents
          │
          ▼
      Filter invalid
          │
          ▼
       Rank valid
          │
          ▼
       Select Agent
```

# 141. Canonical Preemption

```text
High Priority Work
       │
       ▼
Insufficient Capacity
       │
       ▼
Find Preemptible Work
       │
       ▼
Evaluate Cost
       │
       ▼
Request Checkpoint
       │
       ▼
Terminate Safely
       │
       ▼
Release Resources
       │
       ▼
Run Higher Priority Work
```

# 142. Canonical Backfill

```text
Reserved Large Work
        │
        ▼
Reservation Horizon
        │
        ▼
Available Capacity Before Horizon
        │
        ▼
Small Eligible Work
        │
        ▼
Can Finish Safely?
       / \
     YES  NO
      │    │
      ▼    ▼
   BACKFILL WAIT
```

# 143. Canonical Multi-Tenant Scheduling

```text
Organization
    │
    ├── Tenant A
    │      ├── Project A1
    │      └── Project A2
    │
    └── Tenant B
           ├── Project B1
           └── Project B2

                ↓

         Fairness / Quota
                ↓
            Scheduler
                ↓
             Agents
```

# 144. Scheduler Correctness Principle

> **Scheduling is a constrained state transition, not merely a ranking algorithm.**

# 145. Placement Principle

> **A placement is valid only when the selected Agent satisfies capability, resource, policy, authorization, quota, concurrency, topology, and ownership constraints at commit time.**

# 146. Fairness Principle

> **Fairness influences which valid Work is preferred; it must never authorize an otherwise invalid placement.**

# 147. Preemption Principle

> **Preemption is a controlled state transition requiring explicit policy, safe termination semantics, resource reclamation, and protection against duplicate execution.**

# 148. Liveness Principle

> **When eligible Work and compatible capacity exist, NROS should make eventual progress without requiring scheduler-local state to survive process failure.**

# 149. Final Architectural Rule

> **The NROS Scheduler must transform validated, eligible Work into durable assignments through deterministic policy evaluation, fair and resource-aware placement, transactional reservation, explicit quota and concurrency enforcement, controlled preemption/backfilling, bounded queueing, and recoverable reconciliation—while never treating an uncommitted scheduling decision as authoritative.**

The next layer is:

# Part CXXVIII — Policy Engine & Governance Architecture, Authorization, Admission Policies, RBAC/ABAC, Tenancy, Resource Governance, Security Policies, Approval Gates, Policy Versioning, Evaluation Semantics, Overrides, Auditability & Enforcement

The central question becomes:

> **How does NROS formally determine whether a Workflow, Actor, Agent, resource request, execution action, or state transition is permitted—and how can those decisions remain versioned, deterministic, explainable, auditable, and impossible to bypass through alternate control paths?**

# NROS — Part CXXVIII: Policy Engine & Governance Architecture

The Policy Engine is the authorization and governance boundary of NROS.

The Scheduler answers:

> **What can run next?**

The Policy Engine answers:

> **Is this action permitted at all?**

These questions must remain separate.

The central invariant is:

> **No execution authority may be obtained merely because a Work item is schedulable.**

# 1. Policy Responsibility

The Policy subsystem governs:

```text
identity
authentication context
authorization
admission
resource limits
tenant isolation
security constraints
executor permissions
network permissions
secret access
Agent eligibility
approval requirements
Workflow publication
execution actions
administrative operations
```

# 2. Policy Non-Responsibility

The Policy Engine should not directly:

```text
launch processes
select arbitrary Agents
modify Workflow source
consume application output
perform scheduling itself
```

It produces policy decisions consumed by authoritative state transitions.

# 3. Policy Decision

A policy evaluation can produce:

```text
ALLOW
DENY
REQUIRE_APPROVAL
DEFER
```

# 4. Allow

Means:

```text
the requested action satisfies applicable policy
```

It does not necessarily mean resources are available.

# 5. Deny

Means:

```text
the action must not proceed
```

# 6. Require Approval

Means:

```text
policy allows progression only after an explicit approval event
```

# 7. Defer

Means the decision cannot yet be finalized because required policy context is unavailable.

A defer must not be treated as an implicit allow.

# 8. Policy Subject

A subject is the actor requesting or causing an action.

Examples:

```text
human user
service identity
Workflow
Agent
scheduler
administrator
```

# 9. Policy Resource

The resource being acted upon may be:

```text
Workflow Definition
Workflow Instance
Node
Assignment
Agent
Secret
Artifact
Tenant
Project
```

# 10. Policy Action

Examples:

```text
create
read
publish
execute
cancel
approve
delete
administer
assign
register
```

# 11. Policy Context

A decision can depend on:

```text
subject
resource
action
tenant
time
environment
Workflow metadata
resource request
Agent attributes
security classification
```

# 12. Authorization Tuple

Conceptually:

```text
(subject, action, resource, context)
```

produces:

```text
decision
```

# 13. RBAC

Role-Based Access Control assigns permissions through roles.

Example:

```text
developer
operator
auditor
administrator
```

# 14. Role Permissions

Example:

```text
developer:
    create Workflow
    execute development Workflow

operator:
    cancel execution
    inspect runtime

auditor:
    read audit records
```

# 15. Role Assignment

Role assignment should be explicit and scoped.

# 16. Scoped Roles

A user may be:

```text
operator
```

for:

```text
Tenant A
```

without becoming an operator for the entire NROS deployment.

# 17. ABAC

Attribute-Based Access Control evaluates attributes.

Examples:

```text
user.department
Workflow.classification
Agent.trust_level
resource.region
```

# 18. RBAC + ABAC

NROS can combine:

```text
Role
+
Attributes
+
Resource policy
```

# 19. Example Authorization

Conceptually:

```text
ALLOW execute
IF
    subject.role = operator
AND
    resource.tenant = subject.tenant
AND
    resource.classification <= subject.clearance
```

# 20. Deny by Default

The default should be:

```text
DENY
```

unless an applicable policy explicitly permits the operation.

# 21. Explicit Deny

Explicit deny should normally override allow rules.

Example:

```text
ALLOW execute production
DENY execute production from untrusted Agent
```

Result:

```text
DENY
```

# 22. Policy Precedence

A deterministic precedence model is required.

A common ordering is:

```text
system deny
   ↓
security deny
   ↓
tenant deny
   ↓
resource policy
   ↓
role allow
   ↓
default deny
```

The exact ordering must be part of the policy specification.

# 23. Policy Composition

Policies should compose predictably.

Avoid ambiguous rules such as:

```text
Rule A says allow
Rule B says deny
```

without defined precedence.

# 24. Policy Scope

Policies may apply at:

```text
system
organization
tenant
project
Workflow
node
Agent
resource
```

# 25. Policy Inheritance

Child scopes can inherit parent policies.

Example:

```text
System
  ↓
Tenant
  ↓
Project
  ↓
Workflow
```

# 26. Policy Override

Overrides must be explicit and bounded.

An override should identify:

```text
who
what
why
scope
expiration
approval
```

# 27. Temporary Override

Production exceptions may require temporary authorization.

Example:

```text
expires_at
```

# 28. No Permanent Emergency Override

Emergency mechanisms should not silently create permanent privileges.

# 29. Break-Glass Access

A break-glass mechanism may provide exceptional authority.

It must require:

```text
strong authentication
explicit reason
scope
expiration
audit record
```

# 30. Policy Version

Every policy set should have:

```text
policy_version
```

# 31. Policy Immutability

Published policy versions should be immutable.

Changes create a new version.

# 32. Decision Versioning

Every policy decision should record the policy version that produced it.

# 33. Policy Decision Record

A decision record may contain:

```text
decision_id
subject
action
resource
decision
policy_version
context_digest
timestamp
reason_codes
```

# 34. Explainability

A denial should provide structured reasons.

Example:

```text
POLICY_DENIED
reason:
    EXECUTOR_NOT_ALLOWED
```

# 35. Sensitive Policy Details

Not every internal policy rule should be exposed to untrusted users.

Therefore diagnostics may distinguish:

```text
user-visible reason
internal evaluation reason
```

# 36. Policy Evaluation

Evaluation should be deterministic given:

```text
policy_version
input context
```

# 37. Time

Time-dependent policies require explicit evaluation time.

For reproducibility:

```text
evaluation_timestamp
```

should be part of the decision context.

# 38. External Attributes

If policy depends on an external identity provider or directory:

```text
resolved attributes
```

should be represented in the evaluation context or referenced by durable evidence.

# 39. Stale Attributes

Authorization must account for attribute freshness.

Example:

```text
user role changed
```

A cached role should not grant indefinite access.

# 40. Policy Cache

Policy decisions may be cached only with explicit validity semantics.

# 41. Cache Key

A cache key should include all decision-relevant inputs.

# 42. Cache Expiration

Time-sensitive policies require bounded cache lifetimes.

# 43. Revocation

Revoked authorization must invalidate relevant cached decisions.

# 44. Authentication vs Authorization

Authentication establishes:

```text
Who are you?
```

Authorization establishes:

```text
What may you do?
```

They must not be conflated.

# 45. Agent Authentication

An Agent must authenticate before receiving authorized execution commands.

# 46. Agent Authorization

An authenticated Agent may still be prohibited from executing certain Work.

Example:

```text
Agent trust_level < required trust
```

# 47. Agent Trust

Agents can have trust classifications:

```text
trusted
standard
restricted
quarantined
```

# 48. Workflow Trust

Workflow definitions may have classifications:

```text
development
internal
production
restricted
```

# 49. Trust Matching

A Workflow can require:

```text
minimum_agent_trust = trusted
```

# 50. Executor Policy

Policies may restrict executors.

Example:

```text
container executor allowed
shell executor denied
```

for a restricted tenant.

# 51. Network Policy

Execution may require explicit network access.

Example:

```text
network = none
```

or:

```text
network = internal-only
```

# 52. Secret Policy

A Workflow may request:

```text
secret:database/password
```

Policy determines whether the requesting identity may access it.

# 53. Secret Scope

Secrets should be scoped to:

```text
tenant
project
Workflow
environment
```

where required.

# 54. Artifact Policy

Artifact access can be controlled similarly.

Example:

```text
read artifact
publish artifact
delete artifact
```

are separate permissions.

# 55. Environment Policy

Production environments can require additional controls.

Example:

```text
production deployment
→ approval required
```

# 56. Approval Gates

An approval gate is a policy-controlled state transition.

```text
READY
   ↓
APPROVAL_REQUIRED
   ↓
APPROVED
   ↓
SCHEDULABLE
```

# 57. Approval Identity

Approval records must identify:

```text
approver
decision
scope
timestamp
policy_version
```

# 58. Approval Separation of Duties

For sensitive operations:

```text
requester ≠ approver
```

may be required.

# 59. Approval Expiration

An approval can expire if the associated context changes.

# 60. Approval Binding

An approval should be bound to:

```text
Workflow instance
definition digest
requested action
parameter digest
```

where appropriate.

# 61. Approval Reuse

A production approval should not automatically authorize a different Workflow instance.

# 62. Resource Governance

Policy can define maximum resource requests.

Example:

```text
max_cpu = 64
max_memory = 256GiB
```

# 63. Resource Class Authorization

A tenant may be allowed to use:

```text
standard
```

but not:

```text
GPU
high-memory
trusted
```

# 64. Scheduling vs Authorization

The Scheduler may find:

```text
GPU available
```

but Policy may say:

```text
tenant not authorized for GPU
```

Therefore the Work remains unschedulable.

# 65. Policy-Aware Scheduling

The Scheduler should consume policy-derived constraints before candidate selection.

# 66. Policy Precomputation

Static policy constraints can be compiled into the Workflow admission representation.

# 67. Runtime Policy

Dynamic policy conditions must still be evaluated at execution time.

# 68. Defense in Depth

Even if the Scheduler filters an Agent:

```text
Agent-level authorization
```

should still reject unauthorized execution.

# 69. No Single Enforcement Point

Security-critical rules should not depend exclusively on:

```text
UI
Scheduler
```

when they can be enforced at the authoritative transition boundary.

# 70. Policy Enforcement Boundary

The final state mutation should verify the required policy decision.

# 71. TOCTOU

Avoid:

```text
check policy
    ↓
long delay
    ↓
execute
```

if policy can change during the delay.

# 72. Policy Lease

For some actions, authorization can be represented as a short-lived lease.

# 73. Lease Binding

A policy lease should bind to:

```text
subject
resource
action
policy version
expiration
```

# 74. Revocation

Critical policy changes may require active leases to be revoked.

# 75. Policy Evaluation Failure

If the Policy Engine is unavailable:

```text
DENY
```

is appropriate for security-critical operations unless an explicitly defined fail-open policy exists.

# 76. Fail-Open

Fail-open should be extremely limited.

Example:

```text
non-sensitive telemetry
```

may tolerate degraded policy availability.

# 77. Fail-Closed

Security-sensitive actions should normally fail closed:

```text
execute production
publish artifact
read secret
```

# 78. Policy Availability

The Policy Engine itself should be highly available because fail-closed authorization can otherwise become an operational bottleneck.

# 79. Policy Store

Policies require durable storage.

The store should support:

```text
versioning
immutability
history
activation
rollback
```

# 80. Policy Activation

A new policy can move through:

```text
DRAFT
   ↓
VALIDATED
   ↓
STAGED
   ↓
ACTIVE
   ↓
RETIRED
```

# 81. Policy Validation

Validation should detect:

```text
syntax errors
conflicting rules
unreachable rules
invalid references
privilege escalation
unsafe defaults
```

# 82. Policy Testing

Policy should be testable before activation.

# 83. Simulation

A proposed policy can be evaluated against historical contexts:

```text
Would this policy deny any currently valid Workflow?
Would it grant new privileges?
```

# 84. Shadow Mode

A policy can operate in shadow mode:

```text
current policy → enforcement
new policy → observation
```

# 85. Policy Diff

Policy changes should produce semantic diffs:

```text
Tenant A:
    GPU access added

Production:
    approval requirement added
```

# 86. Privilege Escalation Analysis

The policy compiler should identify potentially dangerous transitions.

# 87. Least Privilege

Every identity should receive only the permissions necessary for its responsibilities.

# 88. Service Identity

Internal services should use distinct identities.

For example:

```text
scheduler
policy-engine
agent
artifact-service
audit-service
```

# 89. Service-to-Service Authorization

Each service identity should have explicit permissions.

# 90. Agent Command Authorization

An Agent should verify that incoming commands originate from an authorized Controller identity.

# 91. Controller Action Authorization

The Controller should verify that the requested operation is authorized before emitting commands.

# 92. Audit

Every security-relevant policy decision should be auditable.

# 93. Audit Events

Examples:

```text
POLICY_ALLOW
POLICY_DENY
APPROVAL_REQUESTED
APPROVAL_GRANTED
APPROVAL_REJECTED
OVERRIDE_CREATED
OVERRIDE_EXPIRED
ROLE_ASSIGNED
ROLE_REVOKED
SECRET_ACCESS
```

# 94. Audit Integrity

Audit records should be tamper-evident.

Possible mechanisms:

```text
append-only storage
hash chains
signatures
immutable object storage
```

# 95. Audit Context

A security event should reference:

```text
actor
tenant
action
resource
decision
policy_version
request_id
correlation_id
```

# 96. Correlation

Policy events should be correlatable with:

```text
Workflow
Assignment
Execution
Scheduler decision
Agent command
```

# 97. Policy Observability

Metrics can include:

```text
allow_count
deny_count
approval_count
evaluation_latency
cache_hit_rate
policy_errors
override_count
```

# 98. Denial Rate

A sudden increase in denial rate can indicate:

```text
policy regression
identity failure
configuration error
attack
```

# 99. Policy Error vs Denial

An evaluation error must not be silently reported as an ordinary denial.

Use separate states:

```text
DENIED
EVALUATION_ERROR
```

# 100. Policy Decision Cache

A cache can improve performance, but must respect:

```text
policy version
subject attributes
resource state
expiration
revocation
```

# 101. Policy Context Digest

A canonical digest of policy inputs can support decision traceability.

# 102. Governance Metadata

Sensitive Workflows may require:

```text
owner
reviewer
classification
retention policy
compliance tags
```

# 103. Compliance Policy

Policy can require:

```text
approved region
approved Agent class
mandatory logging
retention
human approval
```

# 104. Data Residency

A Workflow may specify:

```text
allowed_regions = ["EU"]
```

Policy must enforce this before placement.

# 105. Classification

Data or Workflows may carry classifications:

```text
public
internal
confidential
restricted
```

# 106. Classification Propagation

Derived Work should inherit required security classification from its inputs where policy requires.

# 107. Classification Escalation

A task processing restricted data cannot automatically produce an unrestricted output classification.

# 108. Cross-Tenant Access

Cross-tenant operations require explicit policy authorization.

# 109. Tenant Boundary

The absence of explicit cross-tenant authorization should result in denial.

# 110. Administrative Authority

Administrators are still subject to audit.

Administrative privilege must not imply invisibility.

# 111. Policy Governance

Policy changes should require controlled authority.

Possible roles:

```text
policy_author
policy_reviewer
policy_activator
auditor
```

# 112. Four-Eyes Policy

Highly sensitive policy changes can require two independent approvals.

# 113. Policy Rollback

Rollback must restore a known policy version, not reconstruct rules manually.

# 114. Rollback Safety

Existing executions should retain references to the policy versions under which their critical authorization decisions were made.

# 115. Historical Decisions

Audit systems must be able to answer:

> Why was this action allowed?

using the historical policy context.

# 116. Policy Decision Replay

Given:

```text
policy_version
decision_context
```

the engine should be able to reproduce the decision where deterministic replay is supported.

# 117. Policy Conflict

If multiple policy domains conflict:

```text
security
tenant
Workflow
Agent
```

the precedence model must determine the result.

# 118. Policy Monotonicity

Where possible, adding restrictive policy should not accidentally create new permissions.

This is a valuable property for policy analysis.

# 119. Privilege Monotonicity

Adding an allow rule should not unintentionally bypass a higher-priority deny rule.

# 120. Policy Dependency

A policy should explicitly declare external dependencies.

Example:

```text
identity_provider
risk_service
region_database
```

# 121. Dependency Failure

External dependency failure should produce:

```text
EVALUATION_ERROR
```

rather than silent authorization.

# 122. Policy Timeouts

Policy evaluation must have bounded latency.

# 123. Policy Circuit Breaker

Repeated dependency failures may trigger a circuit breaker to prevent system-wide overload.

# 124. Governance and Scheduler Boundary

The interaction should be:

```text
Workflow
   ↓
Policy
   ↓
Eligible
   ↓
Scheduler
   ↓
Placement
```

not:

```text
Scheduler
   ↓
"maybe authorized"
```

# 125. Policy and Agent Boundary

The interaction should be:

```text
Controller authorization
        ↓
Assignment
        ↓
Agent authorization
        ↓
Execution
```

# 126. Policy Invariants

```text
1. Default authorization is deny.

2. Authentication and authorization are separate.

3. Every security-sensitive action has an explicit policy decision.

4. Policy decisions reference a policy version.

5. Published policies are immutable.

6. Policy precedence is deterministic.

7. Explicit deny cannot be accidentally overridden.

8. Role scope is explicit.

9. Tenant boundaries are enforced.

10. Cross-tenant access requires explicit authorization.

11. Resource authorization is distinct from resource availability.

12. Scheduler eligibility does not imply authorization.

13. Agent authentication does not imply permission to execute every Workflow.

14. Secrets require explicit access authorization.

15. Production operations can require approval.

16. Approval is bound to the intended operation.

17. Temporary overrides have expiration.

18. Break-glass operations are auditable.

19. Policy evaluation failures are distinct from ordinary denial.

20. Security-sensitive evaluation fails closed by default.

21. Policy caches have bounded validity.

22. Revocation invalidates affected authorization.

23. Policy changes are versioned.

24. Policy activation is auditable.

25. Historical decisions remain attributable to their policy version.

26. Policy decisions are explainable at least through stable reason codes.

27. Policy dependencies are explicit.

28. Policy evaluation is bounded.

29. Privilege escalation is detectable.

30. Least privilege is the default design principle.

31. Internal service identities are distinct.

32. Service-to-service permissions are explicit.

33. Administrative actions are audited.

34. Policy rollback uses immutable known versions.

35. Security classification is preserved through derived Work where required.

36. Data residency constraints are enforced before placement.

37. Policy state is durable.

38. Policy evaluation does not depend solely on UI enforcement.

39. Authoritative state transitions revalidate required policy authority.

40. No alternate control path can bypass security-critical policy enforcement.
```

# 127. Canonical Policy Architecture

```text
                    POLICY SYSTEM
                         │
          ┌──────────────┼──────────────┐
          ▼              ▼              ▼
       Identity       Policy Store    Context
          │              │              │
          └──────────────┼──────────────┘
                         ▼
                  Policy Evaluator
                         │
          ┌──────────────┼──────────────┐
          ▼              ▼              ▼
        ALLOW           DENY      REQUIRE APPROVAL
          │              │              │
          ▼              ▼              ▼
      Continue         Reject       Approval State
```

# 128. Canonical Authorization Flow

```text
Request
  │
  ▼
Authenticate
  │
  ▼
Resolve Subject
  │
  ▼
Resolve Resource
  │
  ▼
Resolve Context
  │
  ▼
Load Policy Version
  │
  ▼
Evaluate
  │
  ├───────────┬──────────────┐
  ▼           ▼              ▼
ALLOW       DENY       REQUIRE APPROVAL
  │           │              │
  ▼           ▼              ▼
Continue     Reject       Approval
```

# 129. Canonical Governance Flow

```text
Workflow Definition
        │
        ▼
Policy Validation
        │
        ▼
Publication
        │
        ▼
Instantiation
        │
        ▼
Execution Admission
        │
        ▼
Scheduler
        │
        ▼
Agent
        │
        ▼
Runtime Enforcement
        │
        ▼
Audit
```

# 130. Canonical Production Approval

```text
Execution Request
       │
       ▼
Policy Evaluation
       │
       ▼
APPROVAL_REQUIRED
       │
       ▼
Approval Request
       │
       ├───────┐
       ▼       ▼
    APPROVE   REJECT
       │       │
       ▼       ▼
   SCHEDULABLE DENIED
```

# 131. Canonical Policy Decision Record

```text
Decision ID
    │
    ├── Subject
    ├── Action
    ├── Resource
    ├── Tenant
    ├── Policy Version
    ├── Context Digest
    ├── Decision
    ├── Reason Codes
    ├── Timestamp
    └── Correlation ID
```

# 132. Canonical Enforcement Boundary

```text
             REQUEST
                │
                ▼
          POLICY CHECK
                │
        ┌───────┴───────┐
        ▼               ▼
      DENY             ALLOW
        │               │
        X               ▼
                   STATE TRANSITION
                         │
                         ▼
                      OUTBOX
                         │
                         ▼
                       AGENT
```

The important property is:

> **The state transition itself must not rely solely on an earlier UI-level policy check.**

# 133. Policy Design Principle

> **Authorization is a first-class state-machine guard, not a convenience layer around the API.**

# 134. Governance Principle

> **Every exceptional authority must be explicit, scoped, time-bounded where possible, and auditable.**

# 135. Security Principle

> **Fail-closed authorization is the default for security-sensitive operations; fail-open behavior must be explicitly justified and narrowly scoped.**

# 136. Explainability Principle

> **Every policy decision should be reconstructible from its policy version and decision context, subject to intentional protection of sensitive policy internals.**

# 137. Final Architectural Rule

> **The NROS Policy Engine must provide deterministic, versioned, deny-by-default authorization and governance across identities, tenants, Workflows, Agents, resources, secrets, artifacts, and execution actions, with explicit policy precedence, scoped roles, attribute evaluation, approval gates, bounded overrides, fail-closed security semantics, immutable policy history, and auditable decision records. No scheduler, Agent, API, or alternate control path may bypass a security-critical policy boundary.**

The next layer is:

# Part CXXIX — State Store & Consistency Architecture, Event Sourcing, Durable State Machines, Transactions, Optimistic Concurrency, Leases, Locks, Snapshots, Projections, Idempotency, Exactly-Once Effects, Recovery & Reconciliation

The central question becomes:

> **What is the authoritative state of NROS, how is every important state transition made durable and concurrency-safe, and how can Controllers, Schedulers, Agents, and recovery processes reconstruct the same truth after crashes, retries, partitions, and partial failures?**

# NROS — Part CXXIX: State Store & Consistency Architecture

The State Store is the authoritative persistence boundary of NROS.

Every important subsystem depends on it:

```text
Policy
Scheduler
Controller
Workflow Engine
Agent Registry
Execution Manager
Recovery
Audit
Observability
```

The fundamental principle is:

> **If a fact affects correctness, it must have an authoritative representation that survives process failure.**

# 1. State Store Responsibility

The State Store provides:

```text
durable state
atomic transitions
concurrency control
versioning
leases
idempotency
event persistence
snapshots
projections
recovery
reconciliation
```

# 2. State Store Non-Responsibility

The State Store should not become:

```text
business workflow engine
scheduler
policy interpreter
Agent executor
UI database
```

It stores and protects authoritative state.

# 3. Authoritative State

Examples:

```text
Workflow state
Node state
Assignment state
Execution state
Agent registration
Agent lease
Resource reservation
Quota consumption
Approval state
Policy activation
Checkpoint metadata
```

# 4. Derived State

Examples:

```text
ready queues
dashboard counters
search indexes
metrics projections
materialized views
```

Derived state may be rebuilt.

# 5. The Authority Rule

```text
Authoritative state
       ↓
Derived projections
```

Never the reverse.

# 6. State Identity

Every durable entity requires a stable identifier.

Examples:

```text
workflow_id
node_id
execution_id
assignment_id
agent_id
reservation_id
decision_id
lease_id
```

# 7. Version

Mutable entities should have a monotonically increasing version.

Example:

```text
version = 41
```

# 8. Optimistic Concurrency

A mutation can require:

```text
expected_version = 41
```

If the stored version is already:

```text
42
```

the mutation fails with a concurrency conflict.

# 9. Why Versioning Matters

Without version checks:

```text
Controller A reads state 41
Controller B reads state 41

A writes update
B writes update
```

B may accidentally overwrite A.

# 10. Compare-and-Swap

Conceptually:

```text
UPDATE entity
SET state = new_state,
    version = version + 1
WHERE id = ?
AND version = expected_version
```

Exactly one concurrent writer succeeds.

# 11. Transaction

Related changes should be committed atomically.

Example:

```text
Assignment
+
Resource Reservation
+
Quota Consumption
+
Workflow Node State
```

may form one consistency boundary.

# 12. Atomicity

Either:

```text
all required changes commit
```

or:

```text
none commit
```

# 13. Partial Commit Hazard

Dangerous state:

```text
Assignment = CREATED
Reservation = MISSING
```

If resource reservation is mandatory, this state must never become authoritative.

# 14. Transaction Boundary

Transaction boundaries should follow invariants rather than subsystem ownership.

# 15. Event Sourcing

NROS may represent state through an append-only event history.

Conceptually:

```text
Event 1
Event 2
Event 3
...
Event N
    ↓
State
```

# 16. Example Event Sequence

```text
WorkflowCreated
WorkflowValidated
WorkflowStarted
NodeReady
AssignmentCreated
ExecutionStarted
ExecutionCompleted
```

# 17. Event as Fact

Events should describe facts that happened.

Prefer:

```text
ExecutionStarted
```

over:

```text
StartExecution
```

The former is historical fact.

# 18. Commands vs Events

Command:

```text
StartExecution
```

Event:

```text
ExecutionStarted
```

A command expresses intent.

An event records a committed fact.

# 19. Command Processing

```text
Command
  ↓
Validate
  ↓
Authorize
  ↓
Check State
  ↓
Commit
  ↓
Event
```

# 20. Event Immutability

Committed events should never be edited.

Corrections are represented through new events.

# 21. Event Ordering

Events belonging to one aggregate require a deterministic sequence.

Example:

```text
sequence = 0
sequence = 1
sequence = 2
```

# 22. Aggregate

An aggregate is a consistency boundary.

Potential aggregates:

```text
Workflow
Workflow Instance
Execution
Agent
Reservation
Tenant
```

# 23. Aggregate Version

Each aggregate has a version corresponding to its committed history.

# 24. Aggregate Command

A command can declare:

```text
expected aggregate version
```

# 25. Event Append

The event store should reject:

```text
append event
```

when the expected aggregate version does not match.

# 26. Event Sequence

Example:

```text
ExecutionCreated    seq=0
ExecutionStarted    seq=1
ExecutionCompleted  seq=2
```

# 27. Invalid Event Order

The state machine must reject:

```text
ExecutionCompleted
```

when no valid execution exists.

# 28. State Machine

Each authoritative entity should have explicit transitions.

Example:

```text
PENDING
   ↓
READY
   ↓
ASSIGNED
   ↓
RUNNING
   ↓
SUCCEEDED
```

# 29. Invalid Transition

Example:

```text
SUCCEEDED → RUNNING
```

must be rejected unless explicitly defined as a legal recovery transition.

# 30. Terminal States

Terminal states should be explicit:

```text
SUCCEEDED
FAILED
CANCELLED
REJECTED
EXPIRED
```

# 31. Terminal State Mutation

A terminal state should not be silently changed.

Corrections require an explicit transition.

# 32. Event Store vs State Store

NROS can use either:

```text
event-sourced authoritative state
```

or:

```text
transactional current-state store
+
append-only event log
```

The architecture should not require event sourcing everywhere.

# 33. Hybrid Model

A practical design is:

```text
Current State
      +
Event History
      +
Derived Projections
```

# 34. Current State

Optimized for:

```text
fast reads
state validation
transactional mutation
```

# 35. Event History

Optimized for:

```text
audit
replay
debugging
reconciliation
integration
```

# 36. Projection

A projection transforms authoritative events/state into a read model.

Example:

```text
Execution events
      ↓
execution_dashboard
```

# 37. Projection Failure

If a projection fails:

```text
authoritative state remains valid
```

The projection can be rebuilt.

# 38. Projection Version

Every projection should track:

```text
projection_version
```

# 39. Projection Rebuild

Conceptually:

```text
Drop Projection
      ↓
Replay Authoritative History
      ↓
Rebuild
      ↓
Validate
      ↓
Activate
```

# 40. Snapshot

Long event histories may be expensive to replay.

A snapshot stores:

```text
state at sequence N
```

# 41. Snapshot Replay

Recovery can use:

```text
Snapshot N
+
Events N+1...M
```

instead of replaying from zero.

# 42. Snapshot Validity

A snapshot must identify:

```text
aggregate_id
aggregate_version
schema_version
snapshot_timestamp
```

# 43. Snapshot Corruption

If snapshot validation fails:

```text
discard snapshot
replay from earlier trusted point
```

# 44. Schema Version

Persistent state and events require schema versioning.

# 45. Migration

State migrations should be:

```text
explicit
tested
observable
reversible where practical
```

# 46. Event Compatibility

Older events may need to remain readable after software upgrades.

# 47. Upcasting

An event reader can transform an older event representation into the current semantic representation.

# 48. Idempotency

Every externally retryable operation should have an idempotency identity.

Example:

```text
command_id
```

# 49. Duplicate Command

If the same command arrives twice:

```text
command_id = abc
```

the second invocation should not produce a second logical effect.

# 50. Idempotency Record

The State Store can retain:

```text
command_id
result
status
```

# 51. Idempotent Create

Creating an Assignment with the same request identity should return the existing result rather than creating a duplicate Assignment.

# 52. Idempotent Dispatch

Agent commands should carry stable:

```text
command_id
assignment_id
```

# 53. Agent Retry

If the Controller sends:

```text
ExecuteAssignment(command=42)
```

and receives no response, it can safely retry command 42.

# 54. Exactly-Once Effects

True exactly-once execution across distributed systems is difficult.

NROS should avoid depending on an impossible global guarantee.

# 55. Practical Semantics

Prefer:

```text
at-least-once delivery
+
idempotent command handling
+
durable assignment identity
+
reconciliation
```

# 56. Exactly-Once Logical Effect

NROS can provide:

> **Exactly-once logical state transition**

even when transport delivery is at-least-once.

# 57. Execution Side Effects

External application side effects may still occur more than once unless the application itself supports idempotency or transactional integration.

# 58. Idempotency Key Propagation

A request identity should propagate through:

```text
API
 ↓
Controller
 ↓
Assignment
 ↓
Outbox
 ↓
Agent
 ↓
Execution
```

# 59. Outbox Pattern

State changes and outbound commands should be coordinated.

Example:

```text
Transaction:
    create Assignment
    create Outbox Command
```

Then:

```text
Outbox Worker
    ↓
send Agent command
```

# 60. Why Outbox

Without an outbox:

```text
DB commit succeeds
Agent command lost
```

or:

```text
Agent command sent
DB commit fails
```

may occur.

# 61. Outbox State

Example:

```text
PENDING
DISPATCHING
SENT
ACKNOWLEDGED
FAILED
```

# 62. Outbox Retry

Retryable failure should return the command to a retryable state.

# 63. Dead Letter

Repeated permanent failures can move to:

```text
DEAD_LETTER
```

with explicit operator visibility.

# 64. Inbox Pattern

Agents can maintain an inbox/deduplication record:

```text
command_id
received_at
execution_result
```

# 65. Transactional Inbox

Processing can be:

```text
receive command
+
record command identity
+
apply state transition
```

atomically where supported.

# 66. Lease

A lease grants temporary ownership.

Examples:

```text
Agent registration lease
Scheduler leadership lease
Assignment execution lease
Reservation lease
```

# 67. Lease Structure

```text
lease_id
holder_id
resource_id
issued_at
expires_at
epoch
```

# 68. Lease Renewal

The holder periodically renews the lease.

# 69. Lease Expiration

If renewal stops:

```text
lease expires
```

and ownership becomes reclaimable according to policy.

# 70. Lease Fencing

Expiration alone is insufficient if an old holder continues operating.

Use:

```text
fencing_token
```

# 71. Fencing Token

Each new ownership generation receives a higher token:

```text
generation 41
generation 42
```

Older generation 41 operations are rejected.

# 72. Split-Brain Protection

Fencing prevents:

```text
old Controller
+
new Controller
```

from both believing they own the same authority.

# 73. Leader Election

Controller or Scheduler leadership may use a lease.

# 74. Leadership Epoch

Every leader receives:

```text
leader_epoch
```

# 75. Stale Leader

A stale leader cannot commit authoritative changes after losing leadership.

# 76. Lock

Locks can protect short-lived critical sections.

# 77. Distributed Lock Caution

Long-lived distributed locks are dangerous.

Prefer:

```text
transactions
version checks
leases
fencing
```

where possible.

# 78. Lock Scope

If a lock is unavoidable, scope it narrowly.

# 79. Deadlock

Multiple locks can create deadlocks.

Avoid inconsistent acquisition order.

# 80. Transaction Isolation

The State Store requires explicit transaction isolation semantics.

# 81. Serializable Operations

Critical invariants may require serializable transactions or equivalent compare-and-swap guarantees.

# 82. Snapshot Isolation

Snapshot isolation can improve concurrency but may not prevent all write anomalies.

# 83. Write Skew

Example:

```text
Transaction A sees quota available
Transaction B sees quota available
both commit
quota exceeded
```

The transaction model must prevent this when quota is a hard invariant.

# 84. Invariant-Based Concurrency

Concurrency control should be selected based on the invariant being protected.

# 85. Read Consistency

Different reads may have different requirements.

Examples:

```text
strongly consistent
eventually consistent
read-your-writes
monotonic
```

# 86. Strong Reads

Use strong consistency for:

```text
authorization
assignment ownership
resource reservation
quota enforcement
lease validation
```

# 87. Eventual Reads

Dashboards can generally tolerate:

```text
eventual consistency
```

# 88. Read Model Lag

Every eventually consistent projection should expose or internally track lag.

# 89. Consistency Contract

Each API should define its consistency semantics rather than leaving them implicit.

# 90. Transaction Retry

Transient transaction conflicts can be retried.

# 91. Retry Safety

A transaction retry must not duplicate external side effects.

Use:

```text
idempotency
outbox
```

# 92. Backoff

Retries should use bounded exponential backoff with jitter where appropriate.

# 93. Hot Aggregates

Some entities may become contention hotspots.

Examples:

```text
global quota
single Workflow mutex
large tenant counter
```

# 94. Contention Reduction

Possible techniques:

```text
sharding
partitioned counters
batched updates
hierarchical quotas
```

# 95. Counter Correctness

Counters that affect correctness must not rely solely on eventually consistent metrics.

# 96. Resource Accounting

Resource accounting should have an authoritative representation.

Example:

```text
capacity
reserved
allocated
available
```

# 97. Accounting Invariant

For hard resources:

```text
available + reserved + allocated <= capacity
```

subject to explicitly defined accounting states.

# 98. Reservation Release

Release must be idempotent.

# 99. Double Release

A reservation cannot reduce resource accounting twice.

# 100. Recovery

Recovery reconstructs valid state after failure.

# 101. Recovery Sources

Possible sources:

```text
current state
event history
snapshots
leases
outbox
Agent heartbeats
external observations
```

# 102. Recovery Order

A safe conceptual order:

```text
1. Load authoritative state
2. Validate schema
3. Recover transactions
4. Evaluate leases
5. Rebuild projections
6. Reconcile assignments
7. Reconcile Agents
8. Resume scheduling
```

# 103. Crash Recovery

A process crash must not require manual database editing.

# 104. Restart Recovery

After restart:

```text
in-flight local memory
```

may be lost.

Durable state must determine what happens next.

# 105. Partial Failure

Distributed failure may produce:

```text
DB says RUNNING
Agent says process absent
```

This is not necessarily corruption.

It is a reconciliation condition.

# 106. Reconciliation

The system compares independent observations and repairs state according to authoritative rules.

# 107. Reconciliation Loop

```text
Observe
  ↓
Compare
  ↓
Classify drift
  ↓
Apply safe transition
  ↓
Record result
  ↓
Repeat
```

# 108. Drift Classes

Examples:

```text
MISSING
STALE
DUPLICATE
UNKNOWN
CONFLICTING
ORPHANED
```

# 109. Agent Drift

Example:

```text
State Store:
assignment RUNNING

Agent:
process absent
```

Possible result:

```text
EXECUTION_LOST
```

followed by retry policy.

# 110. Unknown Agent

An Agent reporting an unknown assignment must not automatically become authoritative.

# 111. Unknown Execution

External execution not represented in NROS state should be quarantined or ignored according to policy.

# 112. Orphaned Resource

A resource reservation with no valid owner is:

```text
ORPHANED
```

and requires controlled recovery.

# 113. Reconciliation Authority

Reconciliation must have explicit authority to perform recovery transitions.

# 114. Recovery Events

Recovery actions should be auditable:

```text
LeaseExpired
AssignmentRecovered
ReservationReleased
ExecutionMarkedLost
ProjectionRebuilt
```

# 115. Idempotent Reconciliation

Running reconciliation twice must not produce different logical outcomes.

# 116. Reconciliation Concurrency

Only one authoritative repair should win when multiple Controllers detect the same drift.

# 117. Versioned Repair

Repair operations should use optimistic concurrency.

# 118. Database Failure

If the State Store becomes unavailable:

```text
security-sensitive mutations
```

must stop rather than operate from stale local assumptions.

# 119. Read-Only Degradation

Some read-only operations may continue from cached projections.

# 120. Write Degradation

Correctness-critical writes should not be simulated locally when durable storage is unavailable.

# 121. Partition

Network partitions can cause stale observations.

NROS must avoid allowing both sides to independently commit contradictory authority.

# 122. Quorum

If the persistence layer uses consensus, authoritative writes may require quorum.

# 123. Consensus

Consensus may be used for:

```text
leader election
strongly consistent metadata
```

but not every application read requires consensus.

# 124. CAP Tradeoff

For correctness-critical authority, NROS should prefer consistency over availability during partitions.

# 125. Data Retention

State and events need retention policies.

# 126. Event Retention

Events required for audit or recovery must not be deleted before their retention obligations expire.

# 127. Archival

Old events can be archived while maintaining reconstructibility.

# 128. Deletion

Deletion must distinguish:

```text
logical deletion
physical deletion
retention expiration
```

# 129. Tombstones

Deleted entities may require tombstones to prevent stale clients from recreating old state.

# 130. Garbage Collection

Derived data can be garbage-collected if it is reconstructible.

# 131. State Store Backup

Backups must cover all authoritative information required for recovery.

# 132. Restore

A restore operation must validate:

```text
schema
integrity
sequence continuity
cross-reference consistency
```

# 133. Disaster Recovery

Recovery objectives should define:

```text
RPO
RTO
```

# 134. RPO

Recovery Point Objective determines acceptable data loss.

# 135. RTO

Recovery Time Objective determines acceptable recovery duration.

# 136. Backup Verification

A backup that has never been restored is not sufficient evidence of recoverability.

# 137. Restore Testing

Periodic restore tests should verify:

```text
state reconstruction
event integrity
projection rebuild
lease recovery
outbox recovery
```

# 138. Audit Preservation

Security and governance history may have longer retention requirements than operational state.

# 139. Cryptographic Integrity

Critical event histories can use:

```text
hash chaining
signatures
checksums
```

to detect tampering.

# 140. State Integrity

Stored state should validate cross-entity references.

Example:

```text
Assignment.agent_id
```

must reference an existing valid Agent or an explicitly retained historical identity.

# 141. Referential Integrity

Database constraints should enforce relationships where appropriate.

# 142. State Invariants

Examples:

```text
RUNNING execution
→ valid assignment

ASSIGNED node
→ valid assignment

RESERVED resource
→ valid reservation owner

ACTIVE lease
→ valid lease holder

APPROVED execution
→ valid approval record
```

# 143. Cross-Aggregate Invariants

Some invariants span aggregates.

These require:

```text
transaction
or
explicit coordination protocol
```

# 144. Saga

For operations that cannot be atomically committed across systems, NROS may use a saga:

```text
Step A
 ↓
Step B
 ↓
Step C
```

with compensating actions.

# 145. Saga Limitation

Compensation is not equivalent to rollback.

External side effects may remain observable.

# 146. Durable Workflow State

The Workflow Engine should persist state transitions rather than relying on process memory.

# 147. Durable Scheduler State

The Scheduler should reconstruct queues from authoritative state.

# 148. Durable Agent State

Agent registration and leases must survive Controller restarts.

# 149. Durable Approval State

Approval cannot exist only in an in-memory UI session.

# 150. Durable Policy State

Active policy version must be recoverable.

# 151. Canonical State Architecture

```text
                    NROS STATE
                         │
             ┌───────────┼───────────┐
             ▼           ▼           ▼
        Current State   Events    Metadata
             │           │           │
             └──────┬────┴───────────┘
                    ▼
               Transactions
                    │
          ┌─────────┼─────────┐
          ▼         ▼         ▼
       Leases    Outbox    Versions
          │         │         │
          └─────────┼─────────┘
                    ▼
              Reconciliation
                    │
                    ▼
               Projections
```

# 152. Canonical Mutation Flow

```text
Command
  │
  ▼
Authentication
  │
  ▼
Authorization
  │
  ▼
Load State
  │
  ▼
Validate Version
  │
  ▼
Validate State Transition
  │
  ▼
Commit Transaction
  │
  ├── Current State
  ├── Event
  └── Outbox
  │
  ▼
Committed
```

# 153. Canonical Retry Flow

```text
Command
  │
  ▼
command_id exists?
  │
 ┌┴─────────────┐
 │              │
YES             NO
 │              │
 ▼              ▼
Return       Process
Existing         │
Result           ▼
             Commit
                │
                ▼
             Record ID
```

# 154. Canonical Lease Flow

```text
Acquire
  ↓
ACTIVE
  ↓
Renew
  ↓
ACTIVE
  ↓
Expiration
  ↓
EXPIRED
  ↓
Recovery / Reassignment
```

# 155. Canonical Reconciliation

```text
Authoritative State
        │
        ├──────────────┐
        ▼              ▼
Controller View    Agent View
        │              │
        └──────┬───────┘
               ▼
             Compare
               │
       ┌───────┼────────┐
       ▼       ▼        ▼
     Match   Drift    Unknown
       │       │        │
       ▼       ▼        ▼
     Keep    Repair   Quarantine
```

# 156. Canonical Recovery

```text
Crash
  │
  ▼
Restart
  │
  ▼
Load durable state
  │
  ▼
Recover transactions
  │
  ▼
Validate leases
  │
  ▼
Recover outbox
  │
  ▼
Rebuild projections
  │
  ▼
Reconcile external state
  │
  ▼
Resume operations
```

# 157. Canonical Consistency Hierarchy

```text
CRITICAL AUTHORITY
      │
      ▼
Strong Consistency
      │
      ▼
Authoritative State
      │
      ▼
Events
      │
      ▼
Projections
      │
      ▼
Caches
      │
      ▼
UI
```

The farther down the hierarchy, the less suitable the data is as an authority for correctness.

# 158. Consistency Principle

> **No cache, projection, queue, dashboard, or Agent-local observation may silently become authoritative merely because the primary State Store is temporarily unavailable.**

# 159. Idempotency Principle

> **Every retryable distributed command must possess a stable identity and produce at most one logical state transition.**

# 160. Concurrency Principle

> **Every correctness-critical mutation must be protected against concurrent writers through transactions, version checks, leases, fencing, or another explicitly defined consistency mechanism.**

# 161. Recovery Principle

> **Any committed state must be reconstructible after process failure without relying on volatile memory belonging to the process that originally created it.**

# 162. Reconciliation Principle

> **When authoritative state and external observations disagree, NROS must classify the discrepancy and apply a deterministic, auditable recovery rule rather than silently choosing whichever observation arrived last.**

# 163. Event Principle

> **Events represent immutable historical facts; commands represent requested actions; neither concept should be conflated with the other.**

# 164. Projection Principle

> **Every derived projection must be disposable and rebuildable from authoritative state or retained event history.**

# 165. Final Architectural Rule

> **The NROS State Store must provide a durable, versioned, transactionally protected source of truth for every correctness-critical fact, complemented by immutable event history, rebuildable projections, idempotent commands, durable outbox/inbox processing, lease fencing, explicit consistency semantics, crash recovery, and deterministic reconciliation. Distributed retries, crashes, partitions, stale Controllers, and duplicate messages must never create two authoritative interpretations of the same state transition.**

The next layer is:

# Part CXXX — Event Bus, Messaging & Distributed Coordination Architecture

The central question becomes:

> **How do NROS components exchange commands, events, observations, acknowledgements, failures, and coordination signals reliably across process and machine boundaries while preserving ordering, delivery semantics, backpressure, correlation, idempotency, partition tolerance, and recovery?**

# NROS — Part CXXX: Event Bus, Messaging & Distributed Coordination Architecture

The messaging layer connects the NROS control plane, execution plane, persistence layer, Agents, schedulers, policy engine, and observability systems.

Its responsibility is not merely to transport bytes.

It must preserve the semantic distinction between:

```text
command
event
query
response
observation
acknowledgement
heartbeat
coordination signal
```

The central invariant is:

> **A message is not authoritative merely because it was delivered. Authority comes from the durable state transition that validates and commits its semantic effect.**

# 1. Messaging Responsibilities

The messaging subsystem provides:

```text
delivery
routing
correlation
ordering
acknowledgement
retry
deduplication
backpressure
dead-letter handling
partitioning
fan-out
replay
coordination
```

# 2. Messaging Non-Responsibilities

The Event Bus must not become the authoritative owner of:

```text
Workflow state
authorization state
resource ownership
execution state
quota state
```

Those remain under the State Store.

# 3. Message Classes

NROS should distinguish at least:

```text
COMMAND
EVENT
QUERY
RESPONSE
ACK
NACK
HEARTBEAT
OBSERVATION
CONTROL
```

# 4. Command

A command expresses requested action.

Example:

```text
ExecuteAssignment
CancelExecution
PauseWorkflow
RegisterAgent
RenewLease
```

# 5. Event

An event describes a committed fact.

Example:

```text
ExecutionStarted
ExecutionCompleted
AgentRegistered
LeaseExpired
WorkflowCancelled
```

# 6. Query

A query requests information without intending to mutate authoritative state.

# 7. Response

A response contains the result of a query or request.

# 8. Observation

An observation reports externally observed state.

Example:

```text
Agent reports process exited.
```

This is evidence, not automatically an authoritative transition.

# 9. Heartbeat

A heartbeat indicates liveness.

It must not implicitly authorize actions.

# 10. Acknowledgement

An ACK means:

> **The receiver accepted responsibility for processing the message according to the defined protocol.**

It does not necessarily mean the requested business action completed.

# 11. Completion

Completion should be represented independently.

Example:

```text
ACK
≠
ExecutionCompleted
```

# 12. Negative Acknowledgement

A NACK indicates that the message could not be accepted or processed.

Reasons should be structured.

# 13. Message Envelope

Every durable message should contain a common envelope.

Conceptually:

```text
message_id
message_type
schema_version
producer_id
subject
tenant_id
correlation_id
causation_id
created_at
sequence
payload
```

# 14. Message ID

Every message requires a globally unique or sufficiently scoped identifier.

# 15. Correlation ID

Correlation connects related operations.

Example:

```text
API Request
    ↓
Command
    ↓
Assignment
    ↓
Agent Command
    ↓
Execution Event
```

All may share a correlation identity.

# 16. Causation ID

`causation_id` identifies the message that caused the current message.

This permits causal reconstruction.

# 17. Trace Context

Distributed tracing metadata should be propagated independently of business identifiers.

# 18. Tenant Context

Multi-tenant messages should carry explicit tenant identity where applicable.

A consumer must not infer tenant ownership merely from a topic name.

# 19. Schema Version

Message schemas require explicit versions.

# 20. Schema Evolution

Compatible evolution should prefer:

```text
additive fields
optional fields
new message versions
```

over silently changing the meaning of existing fields.

# 21. Semantic Compatibility

A schema can remain syntactically compatible while becoming semantically incompatible.

Therefore compatibility must include:

```text
field meaning
state semantics
ordering assumptions
required invariants
```

# 22. Serialization

The transport format should provide:

```text
deterministic encoding
versioning
validation
bounded size
```

# 23. Message Validation

Consumers should validate:

```text
schema
required fields
tenant scope
message type
version
authorization context
```

before applying effects.

# 24. Routing

Messages may be routed by:

```text
message type
tenant
Workflow
Agent
partition key
resource class
```

# 25. Topic Model

Conceptually:

```text
commands.*
events.*
observations.*
heartbeats.*
coordination.*
audit.*
```

Actual implementation may use another topology.

# 26. Command Routing

Commands should normally have a single authoritative recipient.

Example:

```text
ExecuteAssignment
      ↓
Agent-X
```

# 27. Event Routing

Events may have multiple subscribers.

Example:

```text
ExecutionCompleted
      ├── Scheduler
      ├── Workflow Engine
      ├── Audit
      ├── Metrics
      └── Notification
```

# 28. Consumer Independence

One consumer's failure must not prevent unrelated consumers from processing an event where the messaging architecture supports independent subscriptions.

# 29. Delivery Semantics

NROS should explicitly define:

```text
at-most-once
at-least-once
effectively-once
```

per message category.

# 30. At-Most-Once

Message is delivered zero or one time.

Advantages:

```text
low duplication
low coordination
```

Disadvantage:

```text
possible loss
```

# 31. At-Least-Once

Messages may be delivered repeatedly but should not be silently lost.

This is appropriate for many critical commands and events when consumers are idempotent.

# 32. Effectively-Once

NROS can provide:

```text
at-least-once transport
+
deduplication
+
idempotent state transitions
```

to achieve exactly-once logical effects.

# 33. Exactly-Once Limitation

The bus cannot guarantee that an arbitrary external side effect happens exactly once.

Example:

```text
send email
charge payment
invoke external API
```

may require external idempotency support.

# 34. Ordering

Ordering must be defined at a scope.

Possible scopes:

```text
global
tenant
Workflow
execution
Agent
partition
```

# 35. Global Ordering

Global ordering is expensive and usually unnecessary.

# 36. Aggregate Ordering

A better default is ordering per aggregate.

Example:

```text
Execution-42:
    event 1
    event 2
    event 3
```

# 37. Sequence Numbers

Ordered streams should provide sequence numbers.

# 38. Gap Detection

If a consumer receives:

```text
sequence 41
sequence 43
```

it should detect missing sequence 42.

# 39. Gap Recovery

The consumer may:

```text
request replay
pause processing
load authoritative state
```

depending on protocol.

# 40. Duplicate Sequence

A duplicate sequence should be recognized and handled idempotently.

# 41. Partition Key

Messages belonging to one ordering domain should use a stable partition key.

Example:

```text
execution_id
```

# 42. Partition Balance

Partitioning should prevent a single hot aggregate from overwhelming one partition where possible.

# 43. Hot Key

A globally active Workflow can create a messaging hotspot.

Possible mitigation:

```text
sub-partitioning
sharded work
aggregated events
```

provided semantic ordering remains correct.

# 44. Backpressure

Consumers must be able to signal inability to process incoming work.

# 45. Backpressure Sources

Examples:

```text
CPU saturation
memory pressure
database contention
Agent capacity
external dependency latency
```

# 46. Queue Growth

Unbounded queues are dangerous.

The system should monitor:

```text
queue depth
oldest message age
processing latency
retry volume
```

# 47. Queue Limits

Queues should have bounded or explicitly governed retention.

# 48. Producer Backpressure

When downstream capacity is exhausted, producers should receive a controlled signal rather than continuing unlimited publication.

# 49. Scheduler Interaction

Backpressure must influence admission.

If Agents cannot consume Work:

```text
new execution admission
```

may need throttling.

# 50. Flow Control

A command stream can use:

```text
credits
window size
max in-flight
```

to control producer behavior.

# 51. In-Flight Commands

Track:

```text
sent
acknowledged
completed
timed_out
```

# 52. Timeout

A command timeout means:

> **The expected response did not arrive within the protocol deadline.**

It does not necessarily mean the command failed.

# 53. Timeout Ambiguity

The Agent may have successfully executed the command while its response was lost.

Therefore timeout recovery requires reconciliation.

# 54. Retry

Retrying a timed-out command must use the same logical command identity when the operation is idempotent.

# 55. Retry Storm

A network partition can cause thousands of commands to retry simultaneously.

Mitigation:

```text
exponential backoff
jitter
retry budgets
circuit breakers
```

# 56. Retry Budget

Each operation should have a bounded retry policy.

# 57. Permanent Failure

After retry exhaustion, the message should enter an explicit failure path.

# 58. Dead Letter Queue

Dead-lettered messages should retain:

```text
original message
failure reason
attempt count
timestamps
consumer
correlation ID
```

# 59. Dead Letter Is Not Deletion

A dead-letter message remains an operational artifact.

# 60. Replay

Authorized operators or recovery processes may replay messages.

# 61. Replay Safety

Replay must not blindly reapply side effects.

Consumers must use:

```text
message identity
sequence
idempotency
state validation
```

# 62. Event Replay

Events can rebuild projections.

# 63. Command Replay

Commands should not generally be replayed automatically without verifying their intended effect.

# 64. Observation Replay

Historical observations may be replayed for diagnostics, but should not automatically become current authority.

# 65. Poison Message

A poison message repeatedly fails processing.

It must not block an entire partition indefinitely.

# 66. Poison Handling

Possible strategy:

```text
retry
   ↓
quarantine
   ↓
continue unrelated messages
```

subject to ordering requirements.

# 67. Ordering vs Availability

Strict ordering may require stopping processing after a failed message.

The choice must be explicit.

# 68. Consumer Offset

Consumers require durable progress markers.

Example:

```text
partition
offset
```

# 69. Offset Commit

Offset advancement should occur only after the consumer has safely completed the required logical effect.

# 70. Offset + State Transaction

Where possible:

```text
consume message
+
commit state
+
record processed message
```

should share a consistency boundary.

# 71. Cross-System Offset

When the bus and State Store cannot share a transaction, use:

```text
idempotency
deduplication
reconciliation
```

# 72. Consumer Restart

After restart, a consumer may re-read messages.

This is expected.

# 73. Consumer Idempotency

Consumers must tolerate duplicate delivery.

# 74. Message Authentication

Messages crossing trust boundaries must be authenticated.

# 75. Message Authorization

Authentication answers:

```text
Who sent this?
```

Authorization answers:

```text
Was this sender allowed to publish this message?
```

Both are required where trust boundaries exist.

# 76. Producer Identity

Every producer should have a stable service or Agent identity.

# 77. Topic Authorization

A producer should have explicit permissions for topics or message classes.

Example:

```text
Agent:
    publish observations
    publish heartbeat

Agent:
    cannot publish authoritative Workflow events
```

# 78. Event Authority

Only designated authoritative components may emit certain state events.

# 79. Event Forgery Prevention

A compromised client must not be able to publish:

```text
ExecutionCompleted
```

and have NROS accept it as authoritative merely because the event schema is valid.

# 80. Event Provenance

Critical events should include producer identity and causal context.

# 81. Tenant Isolation

Messaging infrastructure must prevent unauthorized cross-tenant consumption.

# 82. Topic Isolation

Tenant-specific streams can be physically or logically isolated.

# 83. Payload Isolation

Consumers should receive only the data required for their role.

# 84. Sensitive Payloads

Secrets should generally not be embedded directly in ordinary messages.

Prefer references:

```text
secret_reference
```

with authorization evaluated at access time.

# 85. Message Size

Messages should have bounded size.

Large payloads should use object storage or artifact references.

# 86. Blob Pattern

Instead of:

```text
Message
  └── 500 MB artifact
```

use:

```text
Message
  └── artifact_id
```

# 87. Artifact Integrity

Referenced artifacts should include a content digest.

# 88. Compression

Compression may reduce transport overhead but must not create unbounded decompression risk.

# 89. Resource Limits

Consumers must enforce:

```text
maximum payload size
maximum nesting depth
maximum processing time
```

where appropriate.

# 90. Message Schema Registry

A registry can track:

```text
message type
version
compatibility
owner
documentation
```

# 91. Schema Governance

Production message schemas should require controlled changes.

# 92. Breaking Changes

A breaking change should use:

```text
new schema version
```

rather than silently mutating the old contract.

# 93. Consumer Compatibility

Before activation:

```text
existing consumers
```

must be checked for compatibility.

# 94. Blue/Green Messaging

During migration:

```text
old producer
new producer
old consumer
new consumer
```

may temporarily coexist.

# 95. Dual Publishing

Dual publishing can support migrations but creates duplicate-event risks.

Deduplication must therefore remain explicit.

# 96. Event Translation

A compatibility adapter can translate:

```text
EventV1
→
EventV2
```

without modifying historical events.

# 97. Control Plane vs Data Plane

Messaging can distinguish:

```text
control plane
data plane
```

# 98. Control Plane

Contains:

```text
assignments
commands
leases
coordination
state transitions
```

# 99. Data Plane

Contains:

```text
execution streams
logs
artifacts
high-volume telemetry
```

# 100. Isolation

High-volume data-plane traffic must not starve critical control-plane messages.

# 101. Priority

Critical messages may have higher priority.

Example:

```text
CancelExecution
```

may need priority over:

```text
debug telemetry
```

# 102. Priority Inversion

Priority must not permit starvation of lower-priority traffic.

# 103. Cancellation

Cancellation should itself be a durable command.

# 104. Cancellation Race

A cancellation may race with execution completion.

Example:

```text
Cancel
Complete
```

The State Store determines the authoritative result according to the state machine.

# 105. Cancellation Delivery

Failure to deliver cancellation does not automatically imply cancellation succeeded.

# 106. Emergency Cancellation

Emergency cancellation may require separate high-priority routing.

# 107. Heartbeat Architecture

Agents should periodically report liveness.

# 108. Heartbeat Semantics

A heartbeat proves:

```text
communication and local liveness
```

not necessarily:

```text
correct execution of every assigned task
```

# 109. Heartbeat Payload

May include:

```text
agent_id
lease_epoch
timestamp
capacity
active_assignments
health_state
```

# 110. Heartbeat Staleness

The State Store should track last observed heartbeat.

# 111. Agent Failure

If heartbeat expiry occurs:

```text
AgentUnavailable
```

may trigger reconciliation.

# 112. Clock Skew

Distributed timestamps must not assume perfectly synchronized clocks.

# 113. Logical Ordering

For correctness-critical ordering, prefer:

```text
sequence
epoch
causation
```

over wall-clock time alone.

# 114. Clock Usage

Wall-clock time remains useful for:

```text
expiration
timeouts
retention
observability
```

but should be used carefully for causal ordering.

# 115. Coordination

Distributed coordination may include:

```text
leader election
resource ownership
assignment ownership
maintenance mode
drain state
```

# 116. Coordination Authority

Coordination state must be durable.

# 117. Leader Election

A leader election result should be represented by:

```text
leader_id
epoch
lease
```

# 118. Stale Leader Defense

Every leader mutation should prove its current epoch or fencing token.

# 119. Maintenance Mode

Maintenance mode can prevent new assignments while allowing active executions to drain.

# 120. Drain Protocol

```text
ACTIVE
   ↓
DRAINING
   ↓
DRAINED
```

# 121. Agent Drain

A draining Agent should stop accepting new Work while existing Work follows its defined policy.

# 122. Bus Drain

Messaging consumers may need:

```text
stop intake
finish in-flight
commit offsets
shutdown
```

# 123. Shutdown

Graceful shutdown should preserve:

```text
durable state
unprocessed messages
outbox records
leases
```

# 124. Unclean Shutdown

An abrupt process termination should be recoverable through durable state and retry semantics.

# 125. Event Bus Observability

Metrics should include:

```text
publish_rate
consume_rate
queue_depth
consumer_lag
delivery_latency
retry_rate
dead_letter_rate
duplicate_rate
```

# 126. Consumer Lag

Consumer lag is a first-class operational signal.

# 127. Oldest Message Age

Queue depth alone is insufficient.

Track:

```text
oldest_unprocessed_message_age
```

# 128. Delivery Latency

Measure:

```text
publish → delivery
```

and separately:

```text
delivery → processing completion
```

# 129. Correlation Observability

Operators should be able to trace:

```text
Request
→ Command
→ Event
→ Assignment
→ Agent
→ Execution
```

using correlation identifiers.

# 130. Message Audit

Security-sensitive messages should produce audit evidence.

# 131. Replay Audit

Replays should themselves be auditable.

# 132. Manual Message Injection

Manual command injection must require explicit authorization.

# 133. Administrative Messaging

Administrative commands should be distinguishable from normal runtime commands.

# 134. Unsafe Injection

An operator should not be able to inject an arbitrary serialized message and bypass domain validation.

# 135. Domain Validation

Injected commands still pass:

```text
authentication
authorization
schema validation
state validation
```

# 136. Bus Failure

If the messaging system becomes unavailable:

```text
durable state remains authoritative
```

# 137. Deferred Delivery

Outbox records remain pending until messaging recovers.

# 138. Messaging Recovery

After bus recovery:

```text
outbox resumes
consumer offsets resume
dead-letter policies resume
reconciliation validates external state
```

# 139. Duplicate Delivery After Recovery

Expected and harmless if idempotency is correct.

# 140. Event Bus Security Principle

> **A valid message is not automatically a valid state transition.**

# 141. Delivery Principle

> **At-least-once delivery combined with idempotent processing is generally safer than pretending distributed delivery is exactly once.**

# 142. Ordering Principle

> **Ordering guarantees must be specified per semantic scope; global ordering should not be assumed.**

# 143. Backpressure Principle

> **When downstream capacity is exhausted, NROS must reduce admission rather than allow unbounded queue growth.**

# 144. Replay Principle

> **Replay must preserve message identity and must never silently duplicate non-idempotent logical effects.**

# 145. Provenance Principle

> **Every security-sensitive command or event must be attributable to an authenticated producer and an explicit causal chain.**

# 146. Coordination Principle

> **Distributed authority must be fenced by durable epochs or tokens so that stale leaders cannot continue making authoritative mutations.**

# 147. Isolation Principle

> **Control-plane traffic must remain operationally isolated from high-volume data-plane traffic.**

# 148. Final Architectural Rule

> **The NROS Event Bus must provide reliable, authenticated, versioned, observable, and recoverable communication between distributed components while explicitly separating commands from events, observations from authority, acknowledgement from completion, and delivery from successful state transition. The transport may operate with at-least-once semantics, but every correctness-critical consumer must enforce idempotency, ordering where required, durable progress, bounded retries, backpressure, dead-letter handling, and reconciliation. No message may bypass the authoritative State Store, Policy Engine, or state-machine invariants.**

The next layer is:

# Part CXXXI — Agent Runtime & Execution Architecture

The central question becomes:

> **How does an NROS Agent securely acquire an authorized Assignment, prepare an isolated execution environment, execute Work, stream observations, maintain heartbeats, checkpoint progress, handle cancellation, report completion, survive reconnects, and recover from process or machine failure without violating the global state model?**
