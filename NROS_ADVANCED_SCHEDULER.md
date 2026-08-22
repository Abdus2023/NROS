# NROS Advanced Scheduler (Part CXI–CXX)

The scheduler is one of the most important correctness boundaries in NROS.

It transforms:

```text
Work
+
Policy
+
Dependencies
+
Resources
+
Agent capabilities
+
Current runtime state
```

into:

```text
Scheduling Decision
```

The scheduler therefore must be treated as a **state-transition engine**, not merely a queue consumer.

# 1. Scheduling Function

Conceptually:

```text
Schedule(
    authoritative_state,
    eligible_work,
    available_agents,
    resources,
    policies,
    current_epoch
)
→ decision
```

The decision should be explainable from its inputs.

# 2. Deterministic Scheduling

Where deterministic behavior is required:

```text
same state
+
same policy
+
same inputs
+
same epoch
```

should produce:

```text
same scheduling decision
```

unless an explicitly nondeterministic factor is part of the scheduling model.

# 3. Determinism Boundary

NROS should explicitly distinguish:

```text
DETERMINISTIC
```

from:

```text
EVENTUALLY CONSISTENT
```

and:

```text
INTENTIONALLY NONDETERMINISTIC
```

These must not be conflated.

# 4. Why Determinism Matters

Deterministic scheduling improves:

```text
reproducibility
debugging
testing
incident analysis
simulation
formal verification
auditability
```

It also makes scheduling decisions easier to explain.

# 5. Nondeterministic Inputs

Potential nondeterministic inputs include:

```text
wall-clock time
random numbers
network arrival order
concurrent event arrival
unordered map iteration
Agent discovery order
```

If any are intentionally used, their semantics must be explicit.

# 6. Stable Ordering

Whenever multiple candidates are otherwise equivalent, NROS should apply a stable tie-breaker.

Example:

```text
priority
→ enqueue sequence
→ Agent ID
```

This prevents accidental ordering differences.

# 7. Never Depend on Hash Iteration

A scheduler must not rely on implementation-specific ordering such as:

```text
HashMap iteration
```

to select a winner.

The selection order should be explicit.

# 8. Scheduling Inputs

A scheduling decision may depend on:

```text
work priority
creation sequence
deadline
dependencies
resource requirements
Agent capabilities
Agent health
Agent load
tenant quota
policy
affinity
anti-affinity
```

# 9. Input Snapshot

The scheduler should reason against a coherent state snapshot.

Conceptually:

```text
State S42
```

contains:

```text
eligible Work
available Agents
resource state
leases
policies
dependencies
```

The decision is then derived from `S42`.

# 10. Stale Decisions

A scheduler may compute:

```text
assign W42 → A7
```

but before committing the decision:

```text
A7 becomes unavailable
```

The original decision is now stale.

NROS must validate the decision against the current authority state before committing it.

# 11. Optimistic Scheduling

A useful model is:

```text
read state
   ↓
compute decision
   ↓
validate version
   ↓
commit if unchanged
```

If the state version changed:

```text
retry scheduling calculation
```

# 12. State Version

A scheduling state can expose:

```text
state_version = 842
```

A decision derived from version `842` must not silently commit against version `843` when the change affects scheduling correctness.

# 13. Compare-and-Swap

Conceptually:

```text
commit_decision(expected_version=842)
```

succeeds only if:

```text
current_version == 842
```

Otherwise the scheduler recomputes.

# 14. Scheduling Epoch

An epoch provides stronger authority semantics:

```text
epoch = 17
```

A decision produced under epoch `16` cannot be accepted after epoch `17` becomes authoritative.

# 15. Work Lifecycle

A Work item should have explicit lifecycle states.

For example:

```text
CREATED
   ↓
ADMITTED
   ↓
ELIGIBLE
   ↓
QUEUED
   ↓
ASSIGNED
   ↓
DISPATCHED
   ↓
RUNNING
   ↓
FINALIZING
   ↓
SUCCEEDED / FAILED / CANCELLED
```

# 16. Invalid Transitions

NROS should reject impossible transitions.

For example:

```text
SUCCEEDED → RUNNING
```

should not happen unless the domain explicitly models a new attempt or execution.

# 17. Work vs Execution

A Work item and an Execution should not be treated as the same entity.

Example:

```text
Work W42
   ├── Execution E1
   └── Execution E2
```

where E2 may be a retry.

# 18. Attempt Identity

An execution can contain attempts:

```text
Execution E1
   ├── Attempt A1
   ├── Attempt A2
   └── Attempt A3
```

This preserves retry history.

# 19. Retry Semantics

A retry should normally create a new attempt identity.

Do not overwrite:

```text
A1
```

with the state of:

```text
A2
```

# 20. Retry Policy

A retry policy can consider:

```text
failure category
attempt number
maximum attempts
backoff
deadline
resource availability
dependency health
```

# 21. Retry Eligibility

Not every failure should be retried.

Examples:

```text
transient network failure → possibly retry
capacity unavailable → retry later
authorization denied → do not retry automatically
invalid input → do not retry
integrity failure → quarantine/escalate
```

# 22. Deadline Semantics

A Work may have:

```text
deadline
```

The scheduler must define what happens when the deadline passes:

```text
reject
cancel
stop retrying
allow completion
```

depending on the contract.

# 23. Deadline vs Timeout

These are distinct.

A timeout usually means:

```text
one operation exceeded its duration
```

A deadline means:

```text
the overall Work must not proceed beyond a defined point
```

# 24. Deadline Propagation

A Work deadline may propagate into:

```text
execution
attempt
Agent command
external dependency
```

but each layer should retain its own semantic boundary.

# 25. Dependencies

A Work may depend on other Work:

```text
W1
 ↓
W2
 ↓
W3
```

W2 is not eligible until the dependency condition is satisfied.

# 26. Dependency States

A dependency can be:

```text
SATISFIED
UNSATISFIED
FAILED
CANCELLED
UNKNOWN
```

The scheduler must define how each affects eligibility.

# 27. Dependency Failure

If:

```text
W1 = FAILED
```

then W2 may:

```text
fail
skip
wait
retry
execute under degraded semantics
```

depending on policy.

# 28. Dependency Cycles

The scheduler must detect cycles.

Example:

```text
W1 → W2
W2 → W3
W3 → W1
```

This cannot become eligible without an explicit cycle-breaking mechanism.

# 29. DAG Model

Where dependencies form a DAG:

```text
       W1
      /  \
    W2    W3
      \  /
       W4
```

the scheduler can identify parallelizable Work.

# 30. Parallelism

Independent Work may execute concurrently.

However, concurrency must respect:

```text
resource limits
quotas
Agent capacity
dependency semantics
tenant policy
```

# 31. Resource Model

Resources should be represented explicitly.

Examples:

```text
CPU
memory
GPU
storage
network
device
capability
concurrency slot
```

# 32. Resource Requirement

A Work may specify:

```text
cpu >= 2
memory >= 4GiB
gpu = required
capability = "rust-toolchain"
```

The scheduler must evaluate these requirements against current Agent state.

# 33. Resource Reservation

There is a critical difference between:

```text
resource available
```

and:

```text
resource reserved
```

Two scheduling decisions must not reserve the same exclusive capacity.

# 34. Reservation Transaction

Conceptually:

```text
candidate selected
     ↓
reserve resources
     ↓
commit assignment
```

If reservation fails:

```text
discard candidate
```

rather than creating an invalid assignment.

# 35. Reservation Leases

Resource reservations may have:

```text
reservation_id
owner
epoch
expiration
resource set
```

to prevent stale ownership.

# 36. Resource Release

Resources should be released through explicit lifecycle transitions.

Example:

```text
RUNNING
   ↓
FINALIZING
   ↓
RESOURCE_RELEASED
   ↓
COMPLETED
```

The exact ordering must be defined.

# 37. Crash During Resource Ownership

If an Agent crashes while holding resources:

```text
Agent unavailable
   ↓
lease expires/fenced
   ↓
resources reconciled
   ↓
resources returned to pool
```

Do not immediately assume the workload disappeared without reconciliation.

# 38. Capability Matching

An Agent can advertise capabilities:

```text
linux
x86_64
gpu
docker
rust
network
special_device
```

The scheduler matches requirements against these capabilities.

# 39. Capability Versioning

Capabilities may change over time.

Therefore capability state should be associated with an Agent incarnation or version where necessary.

# 40. Affinity

Affinity prefers placing related Work on a particular:

```text
Agent
host
zone
resource pool
```

for locality or performance.

# 41. Anti-Affinity

Anti-affinity prevents related Work from sharing a failure domain.

Example:

```text
replica A → host 1
replica B → host 2
```

rather than placing both on the same host.

# 42. Failure-Domain Awareness

Placement may consider:

```text
host
rack
zone
region
Agent pool
```

depending on deployment architecture.

# 43. Tenant Quotas

A tenant may have:

```text
maximum concurrent executions
CPU quota
memory quota
queue quota
storage quota
```

Quota checks should happen before assignment.

# 44. Fairness

A scheduler should avoid allowing one tenant to monopolize the system indefinitely.

Possible models:

```text
FIFO
weighted fair queue
fair-share
quota-aware priority
aging
```

# 45. Priority

Priority should influence scheduling but must not bypass correctness.

A high-priority Work still cannot execute if:

```text
dependencies unsatisfied
authorization invalid
resources unavailable
Agent incompatible
```

# 46. Starvation

If low-priority Work can remain indefinitely behind higher-priority Work, the system exhibits starvation.

An aging mechanism can increase effective priority over time.

# 47. Aging

Conceptually:

```text
effective_priority =
base_priority + aging_factor
```

The formula should be deterministic and bounded.

# 48. Fairness vs Throughput

A scheduler often trades:

```text
fairness
```

against:

```text
maximum throughput
```

The chosen policy should be explicit rather than accidental.

# 49. Preemption

Preemption allows one Work to displace another.

It should require explicit support for:

```text
pause
checkpoint
terminate
resource reclamation
resume
```

# 50. Preemption Safety

Never assume a running workload can be safely killed.

Workloads may require:

```text
graceful shutdown
checkpoint
transaction rollback
cleanup
```

# 51. Preemption States

Example:

```text
RUNNING
   ↓
PREEMPT_REQUESTED
   ↓
CHECKPOINTING
   ↓
SUSPENDED
```

or:

```text
RUNNING
   ↓
PREEMPT_REQUESTED
   ↓
TERMINATING
   ↓
STOPPED
```

# 52. Cancellation

Cancellation should be represented separately from failure.

```text
FAILED
```

means execution failed.

```text
CANCELLED
```

means execution was intentionally terminated according to cancellation semantics.

# 53. Cancellation Authority

Not every actor should be able to cancel arbitrary Work.

Cancellation requires authorization and should produce an auditable event.

# 54. Cooperative Cancellation

Where possible:

```text
cancel request
   ↓
Agent acknowledges
   ↓
workload stops
   ↓
result committed
```

# 55. Forced Cancellation

If cooperative cancellation fails:

```text
timeout
   ↓
forced termination
```

The final state should retain evidence that termination was forced.

# 56. Scheduling Decision Record

A scheduling decision should be representable as a durable record when auditability is required.

Example:

```text
Decision {
    decision_id
    work_id
    state_version
    scheduler_epoch
    candidates
    selected_agent
    policy_version
    reason
}
```

# 57. Candidate Rejection Reasons

For each candidate:

```text
Agent A1
→ capability mismatch

Agent A2
→ insufficient memory

Agent A3
→ draining

Agent A4
→ selected
```

This makes placement explainable.

# 58. Decision Explainability

An operator should be able to ask:

```text
Why did W42 run on A4?
```

and obtain:

```text
A1 rejected: capability mismatch
A2 rejected: capacity
A3 rejected: draining
A4 selected: first valid candidate under policy P7
```

# 59. Policy Version

Scheduling decisions should identify the policy version that produced them.

Example:

```text
policy_version = scheduler-policy-12
```

Without this, reproducing historical decisions becomes difficult.

# 60. Policy Changes

A policy change should not silently mutate already committed historical decisions.

New decisions use the new policy version.

Historical records retain the previous version.

# 61. Scheduler Simulation

NROS should support offline simulation:

```text
state snapshot
+
policy
+
candidate Work
+
candidate Agents
```

→

```text
predicted scheduling decisions
```

This is valuable for testing policy changes before deployment.

# 62. Deterministic Simulation

Given identical inputs:

```text
simulation(S, P)
```

should produce identical output.

This provides a powerful regression mechanism.

# 63. Scheduling Test Matrix

Tests should cover:

```text
single Agent
multiple Agents
equal candidates
resource exhaustion
dependency blocking
priority
fairness
quota
affinity
anti-affinity
Agent failure
lease expiry
concurrent scheduler instances
restart
duplicate commands
stale decisions
```

# 64. Concurrency

Multiple scheduler workers may attempt to schedule simultaneously.

Correctness must not depend on only one scheduler thread unless single-threaded scheduling is an explicit architectural constraint.

# 65. Concurrent Scheduling

If multiple workers operate concurrently:

```text
Worker A → W42 → A7
Worker B → W42 → A8
```

only one assignment should commit.

Use:

```text
version checks
unique constraints
leases
transactional assignment
```

as appropriate.

# 66. Double Assignment Invariant

For Work that permits only one active execution:

```text
active_execution_count(W42) <= 1
```

must be enforced at the authoritative state boundary.

# 67. Exactly-One Assignment

If the domain requires one Agent owner:

```text
active_owner_count(resource) <= 1
```

must be enforced by authoritative state, not merely scheduler convention.

# 68. Queue Semantics

A queue should define:

```text
ordering
visibility
acknowledgement
retry
dead-letter behavior
expiration
```

# 69. FIFO

FIFO means:

```text
earlier eligible Work
```

is considered before:

```text
later eligible Work
```

but resource constraints can still prevent strict execution order.

# 70. Eligibility Before Ordering

The scheduler should generally:

```text
all queued Work
   ↓
filter eligible Work
   ↓
order eligible candidates
   ↓
select
```

rather than treating blocked Work as though it were executable.

# 71. Head-of-Line Blocking

Strict FIFO can cause:

```text
W1 = blocked
W2 = executable
```

with W2 unnecessarily waiting.

Whether NROS permits bypassing W1 should be an explicit queue policy.

# 72. Queue Visibility

A queued Work may be:

```text
VISIBLE
RESERVED
IN_FLIGHT
BLOCKED
DELAYED
```

These are distinct states.

# 73. Dead-Letter Queue

Repeatedly failing Work may eventually enter:

```text
DEAD_LETTERED
```

rather than retry forever.

The transition should be explicit and auditable.

# 74. Execution Start Boundary

A Work should not be marked:

```text
RUNNING
```

merely because the scheduler issued a command.

A stronger semantic boundary is:

```text
Agent acknowledged execution start
```

or another defined execution evidence event.

# 75. Completion Boundary

Similarly, completion requires a defined evidence boundary.

Possible sequence:

```text
process exited
   ↓
result collected
   ↓
result validated
   ↓
result committed
```

Only the defined commit point establishes durable completion.

# 76. Result Validation

Results may require validation of:

```text
exit status
output schema
artifact existence
artifact digest
resource cleanup
checkpoint state
```

before finalization.

# 77. Partial Results

A workload may produce useful output before failing.

NROS should distinguish:

```text
execution result
```

from:

```text
partial artifacts
```

and preserve them according to policy.

# 78. Artifact Integrity

Artifacts should be identified by:

```text
artifact_id
digest
size
producer_execution
```

where appropriate.

# 79. Checkpointing

Long-running Work may checkpoint state:

```text
Execution E1
   ↓
Checkpoint C1
   ↓
progress
   ↓
Checkpoint C2
```

A later retry may resume from a validated checkpoint.

# 80. Checkpoint Validity

A checkpoint should be associated with:

```text
execution_id
attempt_id
schema_version
software_version
input_digest
state_digest
```

where required for reproducibility.

# 81. Stale Checkpoints

A checkpoint from an incompatible execution environment should not automatically be resumed.

Compatibility must be validated.

# 82. Resume Semantics

The scheduler should know whether resume means:

```text
continue same attempt
```

or:

```text
create new attempt from checkpoint
```

These are semantically different.

# 83. Runtime State Machine

A robust execution state machine can be represented as:

```text
CREATED
  ↓
ADMITTED
  ↓
QUEUED
  ↓
ASSIGNED
  ↓
DISPATCHED
  ↓
STARTING
  ↓
RUNNING
  ├──→ CHECKPOINTING
  │        ↓
  │    CHECKPOINTED
  │
  ├──→ CANCELLING
  │        ↓
  │    CANCELLED
  │
  └──→ FINISHING
           ↓
       VALIDATING
           ↓
       COMMITTED
```

Failure transitions should be explicit rather than implicit.

# 84. Invalid State Recovery

If durable state contains an impossible transition:

```text
RUNNING
→ CREATED
```

NROS should detect the invariant violation rather than blindly continuing.

# 85. Corruption Response

Possible responses:

```text
halt affected resource
quarantine state
restore snapshot
replay events
raise operator alert
```

The system must not silently normalize corruption into a plausible but false state.

# 86. Runtime Assertions

Internal invariants should be asserted aggressively in development and appropriately monitored in production.

Examples:

```text
active execution has one owner
lease epoch is current
committed result has execution identity
terminal execution cannot become running
```

# 87. Safety vs Liveness

Correctness has two major dimensions.

**Safety:**

> Something bad never happens.

Examples:

```text
no double ownership
no stale authority
no invalid transition
no unauthorized execution
```

**Liveness:**

> Something good eventually happens.

Examples:

```text
eligible Work eventually executes
expired leases eventually clear
recovery eventually reaches READY
```

# 88. Safety Takes Precedence

When safety and liveness conflict:

```text
unsafe execution
```

should generally be prevented even if this temporarily reduces throughput.

# 89. Liveness Boundaries

Liveness claims require assumptions.

For example:

```text
Work eventually executes
```

is only meaningful if:

```text
resources eventually become available
Agent eventually becomes healthy
dependencies eventually resolve
```

# 90. Fairness Assumptions

A fairness guarantee should state its scope.

For example:

```text
Within a healthy scheduling domain, continuously eligible Work will not be starved indefinitely.
```

This is more precise than claiming universal fairness.

# 91. Scheduler Correctness Properties

A useful formal property set:

```text
No invalid assignment
No stale assignment
No duplicate active ownership
No unauthorized placement
No resource overcommit
No dependency violation
No impossible state transition
No silent state loss
```

# 92. Scheduler Liveness Properties

Where assumptions hold:

```text
Eligible Work is eventually considered.
Valid assignments are eventually committed.
Retries eventually stop according to policy.
Expired leases eventually become reclaimable.
Recovery eventually reaches READY.
```

# 93. Linearization Point

For important operations, define the point at which the operation logically takes effect.

For example:

```text
assignment linearization point
=
authoritative transaction commit
```

This makes concurrency reasoning substantially clearer.

# 94. Admission Linearization

Work admission may linearize at:

```text
durable creation transaction
```

rather than when an HTTP request first arrives.

# 95. Completion Linearization

Execution completion may linearize at:

```text
validated result commit
```

rather than process exit.

# 96. Cancellation Linearization

Cancellation may linearize when:

```text
authoritative cancellation state commits
```

while the actual workload termination can happen afterward.

# 97. Causal Ordering

Important transitions should preserve causal relationships:

```text
assignment
→ dispatch
→ start
→ finish
→ commit
```

A completion event should not appear authoritative before the corresponding execution exists.

# 98. Event Ordering vs Execution Ordering

Event timestamps do not necessarily define execution order.

Use explicit:

```text
causation_id
sequence
state version
```

where ordering matters.

# 99. Scheduler Observability

Every significant scheduling decision should be traceable to:

```text
state snapshot/version
policy
candidate set
decision
commit result
```

This connects scheduling semantics directly to the observability architecture established in Part CX.

# 100. Determinism Invariants

```text
1. Scheduling inputs are explicit.

2. Stable tie-breaking is mandatory where deterministic behavior is required.

3. Hash iteration order never defines scheduling behavior.

4. Decisions are derived from coherent state.

5. Stale decisions cannot commit silently.

6. State versions protect concurrent scheduling.

7. Epochs protect authority boundaries.

8. Work and Execution identities are distinct.

9. Retries create explicit attempt identities.

10. Resource reservations are authoritative.

11. Resource overcommit is prevented.

12. Dependency constraints are enforced before assignment.

13. Dependency cycles are detected.

14. Priority does not bypass safety constraints.

15. Fairness policy is explicit.

16. Starvation prevention is explicit where required.

17. Preemption semantics are explicit.

18. Cancellation and failure remain distinct.

19. Scheduling decisions are explainable.

20. Policy versions are recorded.

21. Historical decisions are not silently rewritten.

22. Scheduler restart cannot double-schedule Work.

23. Concurrent schedulers cannot create conflicting ownership.

24. Execution start has an explicit semantic boundary.

25. Completion has an explicit commit boundary.

26. Partial results are represented separately from final state.

27. Checkpoints are integrity- and compatibility-checked.

28. Impossible state transitions are rejected.

29. Safety properties are enforced independently of liveness.

30. Linearization points are defined for critical operations.
```

# 101. Canonical Scheduling Pipeline

The resulting model is:

```text
                 WORK
                   │
                   ▼
             VALIDATION
                   │
                   ▼
             ADMISSION
                   │
                   ▼
             DEPENDENCY
              CHECK
                   │
                   ▼
             ELIGIBILITY
                   │
                   ▼
          ┌─────────────────┐
          │ POLICY FILTER   │
          └────────┬────────┘
                   ▼
          RESOURCE FILTER
                   │
                   ▼
          CAPABILITY FILTER
                   │
                   ▼
          AFFINITY / QUOTA
                   │
                   ▼
            STABLE ORDER
                   │
                   ▼
            CANDIDATE
             SELECTION
                   │
                   ▼
          RESERVATION
                   │
                   ▼
          VERSION CHECK
                   │
                   ▼
        AUTHORITATIVE COMMIT
                   │
                   ▼
             DISPATCH
                   │
                   ▼
              EXECUTE
                   │
                   ▼
             VALIDATE
                   │
                   ▼
              COMMIT
```

# 102. Runtime Correctness Principle

The scheduler should never be evaluated solely by:

```text
"Did it eventually run something?"
```

A correct scheduler must also answer:

```text
Why was this Work eligible?

Why was this Agent selected?

Which resources were reserved?

Which policy version was used?

What prevented competing candidates?

What authority epoch was active?

Where is the authoritative commit?

What evidence proves execution occurred?
```

# 103. Final Principle

> **A scheduling decision is correct only when it is valid against authoritative state, respects policy and resource constraints, cannot be invalidated by stale authority, and remains explainable after the runtime is restarted.**

This gives NROS a precise bridge between:

```text
Scheduling
     +
Concurrency
     +
Authority
     +
Observability
     +
Recovery
```

rather than treating them as independent subsystems.

# Part CXII — Security Architecture, Identity, Authorization & Trust Boundaries

The next layer should formalize the security model around:

```text
users
services
Agents
Workloads
credentials
commands
leases
tenants
artifacts
policies
```

with particular attention to:

```text
authentication
authorization
capability delegation
credential lifecycle
least privilege
tenant isolation
command integrity
Agent trust
auditability
```

The central question becomes:

> **Who is allowed to cause which state transition, under which authority, with which credentials, and how can NROS prove that the action was authorized?**

# NROS — Part CXII: Security Architecture, Identity, Authorization & Trust Boundaries

Security in NROS must be modeled as a property of **state transitions and authority**, not merely as authentication at the API boundary.

The fundamental security question is:

> **Who is authorized to cause which transition, on which resource, under which policy, using which authority, and with what evidence?**

The security model therefore spans:

```text
Identity
   ↓
Authentication
   ↓
Authorization
   ↓
Capability / Scope
   ↓
Policy Evaluation
   ↓
Command Construction
   ↓
Execution Authority
   ↓
Audit
```

# 1. Security Boundaries

NROS should explicitly identify trust boundaries between:

```text
User
API
Control Plane
Scheduler
Agent
Workload
Artifact Store
State Store
External System
```

A component crossing a boundary must never inherit trust implicitly.

# 2. Principal Model

A principal represents an actor capable of requesting or performing an operation.

Possible principal classes:

```text
USER
SERVICE
AGENT
WORKLOAD
SYSTEM
ADMINISTRATOR
AUTOMATION
```

Each class should have explicit semantics.

# 3. Identity

Every security-sensitive principal should have a stable identity.

Examples:

```text
user_id
service_id
agent_id
workload_id
```

Identity should not be derived from:

```text
IP address
hostname
process ID
connection ID
```

because these values can change or be reused.

# 4. Authentication

Authentication establishes:

> **Who are you?**

Authorization establishes:

> **What may you do?**

These must remain separate concepts.

# 5. Authentication Factors

Depending on deployment, NROS may support:

```text
mTLS
signed tokens
OIDC/OAuth identity
API credentials
service identity
hardware-backed credentials
```

The exact mechanism can vary, but the authenticated identity must be normalized into the NROS principal model.

# 6. Service Identity

Internal services should authenticate independently.

For example:

```text
API → Controller
Controller → Scheduler
Scheduler → Agent
Agent → Artifact Store
```

A service should not authenticate merely because it is reachable on an internal network.

# 7. Zero-Trust Boundary

The default assumption should be:

```text
network reachability ≠ authorization
```

and:

```text
internal component ≠ automatically trusted component
```

# 8. Agent Identity

Every Agent should possess a stable identity:

```text
agent_id
```

and an identity credential.

The controller should be able to determine:

```text
which Agent
which incarnation
which credential
which authority epoch
```

is communicating.

# 9. Agent Incarnation

A restarted Agent must receive a new incarnation identity or equivalent epoch.

Example:

```text
Agent A42 / incarnation 17
```

then after restart:

```text
Agent A42 / incarnation 18
```

This prevents stale sessions from being confused with the new runtime.

# 10. Credential Rotation

Credentials must be replaceable without requiring permanent identity changes.

Conceptually:

```text
Agent A42
   ↓
credential C1
   ↓ rotation
credential C2
```

Both identity and credential lifecycle must therefore be modeled separately.

# 11. Credential Expiration

Credentials should have explicit validity periods where appropriate:

```text
issued_at
expires_at
issuer
subject
key_id
```

Expired credentials must not authorize new operations.

# 12. Revocation

NROS must have a defined strategy for revoked credentials.

Possible mechanisms:

```text
short-lived credentials
revocation lists
key rotation
issuer-side invalidation
session termination
```

# 13. Least Privilege

Every principal should receive only the permissions required for its function.

For example:

```text
telemetry-reader
```

should not automatically possess:

```text
execution-admin
```

authority.

# 14. Permission Model

A permission can be represented conceptually as:

```text
principal
+
action
+
resource
+
scope
+
conditions
```

Example:

```text
Agent A42
may execute
Work W*
within
Pool P7
```

# 15. Actions

Actions should be explicit.

Examples:

```text
READ
CREATE
UPDATE
DELETE
EXECUTE
CANCEL
PAUSE
RESUME
ASSIGN
DEPLOY
ROTATE_CREDENTIAL
VIEW_SECRET
```

Avoid vague permissions such as:

```text
FULL_ACCESS
```

unless truly required.

# 16. Resource Scoping

Authorization should be scoped to concrete resources where possible.

Example:

```text
tenant:T1
workspace:W7
execution:E42
agent:A9
artifact:AR17
```

# 17. Tenant Isolation

A tenant must not be able to access another tenant's resources merely by knowing an identifier.

Every resource access should enforce:

```text
principal
→ tenant scope
→ resource ownership
→ permission
```

# 18. Cross-Tenant Access

Cross-tenant operations should require explicit privileged authorization.

For example:

```text
platform-admin
```

may inspect multiple tenants, while:

```text
tenant-user
```

may access only one.

# 19. Resource Ownership

Ownership should be represented explicitly.

For example:

```text
resource_id
owner_tenant
owner_principal
```

Ownership is not necessarily equivalent to authorization.

# 20. Delegation

A principal may delegate limited authority.

Example:

```text
User U1
   ↓ delegates
Service S1
   ↓ may execute
Work W42
```

The delegated authority should be narrower than or bounded by the original authority.

# 21. Authority Intersection

Effective permission should be constrained by every relevant boundary:

```text
User permission
∩
delegated permission
∩
tenant policy
∩
resource policy
∩
runtime safety policy
```

The resulting set is the effective authority.

# 22. Capability Model

NROS can represent temporary execution authority as a capability.

Conceptually:

```text
Capability {
    capability_id
    issuer
    subject
    action
    resource
    scope
    expiration
    epoch
}
```

# 23. Capability Non-Transferability

Where required, a capability should be bound to:

```text
principal
Agent
incarnation
execution
```

so that possession of a copied token does not automatically grant unrelated authority.

# 24. Capability Expiration

Execution capabilities should generally be time-bounded or lease-bounded.

This limits damage from:

```text
credential leakage
stale commands
replayed commands
```

# 25. Command Authorization

Every security-sensitive command should have an authorization context.

Conceptually:

```text
Command {
    command_id
    principal
    action
    resource
    authorization_context
    epoch
}
```

# 26. Authorization Before Dispatch

The controller should authorize before dispatching a command to an Agent.

The Agent may additionally verify the command if it represents an independent trust boundary.

# 27. Defense in Depth

Security should not depend on one authorization check.

For example:

```text
API
 ↓
Controller authorization
 ↓
Scheduler policy
 ↓
Agent command validation
 ↓
Workload sandbox
```

Each layer enforces its own relevant invariants.

# 28. Agent-Side Authorization

An Agent should reject commands that violate its own security boundary even if the controller claims they are valid.

This protects against:

```text
compromised controller
misconfiguration
stale authority
forged commands
```

depending on the deployment threat model.

# 29. Command Integrity

Commands should be protected against modification.

Possible mechanisms include:

```text
authenticated transport
message authentication
digital signatures
request digests
```

The mechanism should match the threat model.

# 30. Replay Protection

A valid command captured from a previous execution should not automatically remain executable.

Use:

```text
command_id
nonce
epoch
expiration
sequence
```

as appropriate.

# 31. Replay Detection

The receiving side should detect previously accepted command identities.

For example:

```text
command C42
status = COMPLETED
```

A duplicate C42 should return the existing result or an explicit duplicate response rather than executing again.

# 32. Freshness

Security-sensitive commands should have a freshness condition.

Conceptually:

```text
issued_at
+
expiration
+
current_epoch
```

must remain valid at execution time.

# 33. Stale Authority

A command can be cryptographically authentic but still unauthorized because it belongs to an obsolete authority epoch.

Therefore:

```text
authenticated ≠ currently authorized
```

# 34. Epoch Validation

Before accepting a command:

```text
command.epoch == current_authority_epoch
```

or another explicitly defined validity relation must hold.

# 35. Lease Security

A lease is both a reliability mechanism and a security boundary.

A valid lease should bind:

```text
resource
owner
epoch
expiration
```

# 36. Lease Theft

A principal must not be able to renew another principal's lease merely by knowing its identifier.

Renewal authorization must verify ownership and authority.

# 37. Secret Management

Secrets should not be embedded directly in:

```text
Work definitions
logs
events
command arguments
artifact metadata
```

unless explicitly protected.

# 38. Secret References

Prefer:

```text
secret_ref
```

over:

```text
secret_value
```

in durable Work state.

The actual secret can be resolved only when needed.

# 39. Secret Scope

Secret access should be scoped to:

```text
tenant
Work
Execution
Agent
operation
```

where practical.

# 40. Secret Exposure Boundary

A secret should not automatically become visible to every component participating in an execution.

For example:

```text
Scheduler
```

may know that a secret is required without receiving its plaintext value.

# 41. Secret Injection

Where possible:

```text
secret store
   ↓
Agent
   ↓
isolated workload
```

rather than:

```text
secret store
   ↓
Controller
   ↓
Scheduler
   ↓
logs
```

# 42. Secret Redaction

Security-sensitive values should be redacted from:

```text
logs
telemetry
diagnostics
error messages
trace attributes
audit records
```

# 43. Structured Logging

Structured logs should use explicit sensitivity classifications:

```text
PUBLIC
INTERNAL
CONFIDENTIAL
SECRET
```

The logger can then enforce redaction rules systematically.

# 44. Audit vs Logging

Operational logs answer:

> What happened operationally?

Audit records answer:

> Who performed which security-sensitive action, under what authority?

They should not be treated as interchangeable.

# 45. Audit Record

A security audit event can contain:

```text
event_id
timestamp
principal
action
resource
tenant
authorization_result
policy_version
command_id
causation_id
request_id
```

without storing unnecessary secret material.

# 46. Authorization Denials

Denied operations should be observable.

Example:

```text
DENIED
principal=U42
action=EXECUTE
resource=W99
reason=missing_permission
```

Sensitive information should still be minimized.

# 47. Audit Immutability

Security audit records should be protected against ordinary mutation.

Depending on deployment:

```text
append-only storage
integrity chaining
signed records
WORM storage
```

may be appropriate.

# 48. Audit Completeness

Security-sensitive state transitions should have corresponding audit evidence.

Examples:

```text
credential created
credential revoked
permission granted
permission removed
Work executed
Work cancelled
Agent enrolled
Agent disabled
secret accessed
```

# 49. Authorization Policy Versioning

Every important authorization decision should be attributable to a policy version.

Example:

```text
authorization_policy = P19
```

This makes historical decisions explainable after policy changes.

# 50. Policy Evaluation

Conceptually:

```text
authorize(
    principal,
    action,
    resource,
    context,
    policy_version
)
→ ALLOW | DENY
```

The evaluator should be deterministic for the same policy and context.

# 51. Policy Context

Authorization context may include:

```text
tenant
resource state
Agent identity
Agent incarnation
execution state
network trust level
time constraints
capability
delegation
```

Only relevant context should influence authorization.

# 52. Time-Based Authorization

Some permissions may be time-scoped:

```text
valid from
valid until
```

The time source should be trustworthy enough for the required security guarantee.

# 53. Clock Security

Wall-clock time should not be treated as perfectly trustworthy across distributed systems.

Security-sensitive expiry may require:

```text
server-side validation
monotonic local timers
short validity windows
```

depending on architecture.

# 54. Policy Precedence

When multiple policies apply, NROS must define precedence.

A secure default is generally:

```text
explicit deny
```

overrides:

```text
allow
```

when policies conflict.

# 55. Default Deny

Unknown permissions should produce:

```text
DENY
```

rather than:

```text
ALLOW
```

# 56. Fail-Closed

For high-risk operations, if authorization infrastructure is unavailable:

```text
authorization unavailable
```

should normally result in:

```text
operation blocked
```

rather than silently authorized execution.

# 57. Read vs Write

Security policies may distinguish:

```text
read
```

from:

```text
mutate
```

A principal might be allowed to inspect execution state without being allowed to cancel it.

# 58. Administrative Authority

Administrative operations should be strongly separated from ordinary execution authority.

Examples:

```text
rotate platform keys
change tenant quotas
disable Agent
modify authorization policy
```

should require elevated privileges.

# 59. Break-Glass Access

Emergency access may exist but should be:

```text
time-limited
strongly authenticated
highly audited
scope-limited
reviewable
```

Break-glass access must not become ordinary administrative access.

# 60. Agent Enrollment

An Agent should not become trusted merely by connecting.

Enrollment should establish:

```text
identity
credential
capabilities
policy scope
trust state
```

# 61. Agent Trust States

Possible lifecycle:

```text
UNREGISTERED
   ↓
PENDING
   ↓
ENROLLED
   ↓
ACTIVE
   ↓
QUARANTINED
   ↓
DISABLED
```

# 62. Quarantine

An Agent may be quarantined if:

```text
credential compromise suspected
unexpected capability change
protocol violation
integrity failure
malicious behavior
```

Quarantine should prevent unsafe operations while preserving evidence.

# 63. Capability Attestation

Where supported, an Agent may provide evidence about:

```text
software version
runtime version
platform
security configuration
```

This should be treated as evidence, not automatically as unquestionable truth.

# 64. Workload Isolation

Even a trusted Agent should isolate workloads according to the required threat model.

Possible isolation layers:

```text
process
container
VM
sandbox
OS user
filesystem namespace
network namespace
```

# 65. Privilege Separation

The Agent control process should not necessarily execute workloads with the same privileges.

Conceptually:

```text
Agent supervisor
       │
       ▼
restricted workload
```

# 66. Filesystem Isolation

A workload should receive only the filesystem access required for its execution.

Avoid exposing:

```text
Agent credentials
control-plane state
other tenant data
host secrets
```

to ordinary workloads.

# 67. Network Isolation

Network access should be policy-controlled where required.

A workload may need:

```text
no network
internal-only
specific service
restricted outbound
full network
```

These should be explicit execution capabilities.

# 68. Resource Isolation

Security and reliability overlap here.

A malicious or faulty workload should not trivially exhaust:

```text
CPU
memory
storage
file descriptors
processes
network connections
```

# 69. Artifact Security

Artifacts should have:

```text
owner
tenant
producer
digest
classification
access policy
```

# 70. Artifact Integrity

When an artifact is referenced later:

```text
artifact_id
+
digest
```

should identify exactly which content is intended.

# 71. Artifact Authorization

Possession of an artifact identifier must not automatically imply read permission.

Access requires:

```text
principal
→ artifact policy
→ authorization
```

# 72. Dependency Security

External dependencies should be identified and verified where supply-chain assurance matters.

Examples:

```text
container image
binary
package
script
plugin
model
artifact
```

# 73. Dependency Pinning

Security-sensitive workloads should prefer immutable references such as:

```text
digest
content hash
signed artifact identity
```

over mutable names alone.

# 74. Command Injection Boundary

Work parameters must not accidentally become control-plane commands.

Separate:

```text
data
```

from:

```text
control instructions
```

and validate both independently.

# 75. Serialization Security

Untrusted serialized data should not be deserialized into executable objects without validation.

Prefer explicit schemas and bounded parsers.

# 76. Protocol Validation

Every incoming protocol message should validate:

```text
version
message type
required fields
field ranges
identity
authorization
freshness
size limits
```

before processing.

# 77. Resource Exhaustion Attacks

Inputs should have limits for:

```text
message size
metadata count
string length
nested depth
batch size
request rate
concurrent requests
```

# 78. Rate Limiting

Rate limits may be applied per:

```text
principal
tenant
IP/network identity
API key
Agent
endpoint
operation class
```

depending on the threat model.

# 79. Authentication Abuse

Repeated failed authentication should trigger appropriate controls such as:

```text
rate limiting
temporary blocking
credential rotation
security alerts
```

without creating an easy denial-of-service vector against legitimate users.

# 80. Security Events

Security-relevant events should be first-class domain or audit events where required.

Examples:

```text
AuthenticationSucceeded
AuthenticationFailed
AuthorizationDenied
CredentialIssued
CredentialRevoked
AgentEnrolled
AgentQuarantined
CapabilityGranted
CapabilityRevoked
SecretAccessed
```

# 81. Security Event Causality

A security event should be correlated with:

```text
request_id
principal
command_id
execution_id
causation_id
```

where applicable.

# 82. Security Incident Containment

When compromise is suspected:

```text
detect
   ↓
classify
   ↓
revoke/fence
   ↓
quarantine
   ↓
preserve evidence
   ↓
recover
```

This parallels the reliability recovery model.

# 83. Credential Compromise

If an Agent credential is compromised:

```text
revoke credential
   ↓
invalidate sessions
   ↓
fence Agent incarnation
   ↓
quarantine Agent
   ↓
issue replacement credential
   ↓
re-enroll
```

# 84. Compromised Principal

A compromised user or service identity should be isolated without necessarily invalidating unrelated principals.

This reinforces failure-domain isolation at the security layer.

# 85. Security State Machine

Security posture can be modeled as:

```text
TRUSTED
   ↓
SUSPECTED
   ↓
QUARANTINED
   ↓
REVOKED
```

Recovery should require explicit evidence to return to a trusted state.

# 86. Trust Is Not Permanent

An Agent being trusted yesterday does not prove that:

```text
Agent
software
credential
configuration
```

remain trustworthy today.

Trust must therefore have explicit lifecycle semantics.

# 87. Cryptographic Identity

Where strong identity is required, identities may be bound to cryptographic keys.

Example:

```text
agent_id
   ↕
public_key
```

Credential rotation then changes the key material without necessarily changing the logical Agent identity.

# 88. Key Rotation

Key rotation should preserve:

```text
identity continuity
```

while invalidating:

```text
old credential authority
```

after the configured transition period.

# 89. Key Compromise Recovery

A compromised key requires stronger treatment than routine rotation.

The system may need to:

```text
revoke immediately
invalidate active sessions
change epoch
re-enroll identity
audit affected operations
```

# 90. Secure Defaults

NROS should default toward:

```text
authentication required
default deny
least privilege
encrypted transport
bounded credentials
short-lived execution authority
audit sensitive actions
redact secrets
fence stale authority
```

# 91. Security vs Availability

Some security decisions intentionally reduce availability.

For example:

```text
authorization store unavailable
```

may cause:

```text
new privileged execution blocked
```

This is preferable to silently executing unauthorized Work.

# 92. Security and Recovery Integration

Recovery must restore security state before normal scheduling resumes.

A safe order is:

```text
recover state
   ↓
validate identity infrastructure
   ↓
validate authority epoch
   ↓
revoke stale credentials
   ↓
reconcile Agents
   ↓
restore authorization state
   ↓
enable scheduling
```

# 93. Security Invariants

```text
1. Identity is distinct from authentication.

2. Authentication is distinct from authorization.

3. Network reachability does not imply trust.

4. Every security-sensitive action has an identified principal.

5. Permissions are explicit.

6. Default authorization is deny.

7. Least privilege is the baseline.

8. Authorization is resource-scoped.

9. Tenant boundaries are enforced at the authoritative state boundary.

10. Delegated authority cannot exceed the delegator's authority.

11. Capabilities are bounded by scope and lifetime.

12. Commands have stable identities.

13. Commands have replay protection.

14. Authenticated commands can still be rejected as stale.

15. Authority epochs fence obsolete control.

16. Leases are security-sensitive authority objects.

17. Secrets are referenced rather than unnecessarily copied.

18. Secrets are excluded from ordinary logs and telemetry.

19. Audit records are distinct from operational logs.

20. Security-sensitive decisions are attributable to policy versions.

21. Authorization failures are observable.

22. High-risk authorization failures default closed.

23. Agent enrollment establishes explicit trust.

24. Agent incarnations are distinct.

25. Compromised Agents can be quarantined.

26. Credentials can be rotated and revoked.

27. Workloads execute with bounded privileges.

28. Artifacts have explicit ownership and access policy.

29. Artifact identity can be integrity-verified.

30. Untrusted protocol input is validated and bounded.

31. Resource exhaustion is treated as a security concern.

32. Security incidents trigger containment before recovery.

33. Recovery restores security state before normal scheduling.

34. Trust is time- and evidence-dependent.

35. Security state transitions are auditable.
```

# 94. Canonical Security Flow

The complete authorization path becomes:

```text
                  REQUEST
                     │
                     ▼
               AUTHENTICATE
                     │
                     ▼
                IDENTIFY
                 PRINCIPAL
                     │
                     ▼
              CHECK CREDENTIAL
                     │
                     ▼
              CHECK FRESHNESS
                     │
                     ▼
              CHECK AUTHORITY
                     │
                     ▼
              CHECK TENANT
                     │
                     ▼
             CHECK RESOURCE
                     │
                     ▼
             CHECK CAPABILITY
                     │
                     ▼
              POLICY EVAL
                     │
             ┌───────┴───────┐
             │               │
           DENY             ALLOW
             │               │
             ▼               ▼
           AUDIT         ISSUE/USE
             │           AUTHORITY
             │               │
             │               ▼
             │            DISPATCH
             │               │
             │               ▼
             │            EXECUTE
             │               │
             └───────┬───────┘
                     ▼
                   AUDIT
```

# 95. Security Principle

The strongest NROS security model is not:

```text
"the request had a valid token"
```

It is:

```text
the principal is known
+
the credential is valid
+
the authority is current
+
the resource is in scope
+
the requested action is permitted
+
the policy permits it
+
the command is fresh
+
the execution authority is valid
+
the action is auditable
```

# 96. Final Architectural Rule

> **NROS must treat authorization as a property of the transition being attempted, not merely a property of the connection making the request.**

This connects:

```text
Identity
+
Authorization
+
Leases
+
Epochs
+
Scheduling
+
Execution
+
Audit
+
Recovery
```

into one coherent authority model.

# Part CXIII — Observability, Telemetry, Tracing & Evidence Architecture

The next layer should formalize how NROS proves what happened.

The observability model should distinguish:

```text
Metrics
Logs
Events
Traces
Audit records
State snapshots
Execution evidence
Security evidence
Recovery evidence
```

The central question becomes:

> **Can an operator reconstruct what NROS believed, what it actually did, why it did it, and which evidence supports that conclusion?**

# NROS — Part CXIII: Observability, Telemetry, Tracing & Evidence Architecture

Observability in NROS must be treated as an **evidence system**, not merely a collection of logs and metrics.

The objective is to make important runtime behavior reconstructable:

```text
What happened?
Why did it happen?
Who caused it?
Which state was authoritative?
Which policy was active?
What did the Agent actually execute?
What evidence proves the result?
```

# 1. Observability Model

NROS should distinguish at least:

```text
Metrics
Logs
Events
Traces
Audit Records
State Snapshots
Execution Evidence
Security Evidence
Recovery Evidence
```

Each answers a different question.

# 2. Metrics

Metrics answer:

> **How much? How often? How long?**

Examples:

```text
scheduler_decisions_total
executions_started_total
executions_completed_total
execution_failures_total
queue_depth
active_executions
resource_utilization
lease_expirations_total
authorization_denials_total
```

# 3. Logs

Logs answer:

> **What did a component observe or do?**

Examples:

```text
scheduler selected candidate
Agent received command
execution process exited
checkpoint created
authorization denied
```

Logs should remain operationally useful without becoming the authoritative state store.

# 4. Events

Events represent meaningful state or domain transitions.

Example:

```text
WorkAssigned
ExecutionStarted
ExecutionFailed
LeaseExpired
AgentQuarantined
ArtifactCommitted
```

Events should have explicit semantics.

# 5. Traces

Distributed traces answer:

> **How did one operation propagate across components?**

Example:

```text
API request
   ↓
Controller
   ↓
Scheduler
   ↓
Agent
   ↓
Workload
   ↓
Artifact Store
```

# 6. Audit Records

Audit records answer:

> **Who performed a security-sensitive action under which authority?**

They must not be confused with ordinary application logs.

# 7. State Snapshots

A snapshot answers:

> **What did the authoritative system state look like at a particular point?**

Snapshots are especially important for:

```text
recovery
debugging
simulation
incident analysis
reproduction
```

# 8. Execution Evidence

Execution evidence answers:

> **What proves that the requested operation actually occurred?**

Examples:

```text
Agent acknowledgement
process identity
exit status
output digest
artifact digest
execution timestamp
resource release
```

# 9. Evidence Hierarchy

Not all telemetry has equal authority.

A useful hierarchy is:

```text
AUTHORITATIVE STATE
       ↓
COMMITTED DOMAIN EVENT
       ↓
EXECUTION EVIDENCE
       ↓
AUDIT RECORD
       ↓
TRACE
       ↓
LOG
       ↓
METRIC
```

This does not mean lower layers are useless.

It means they should not silently override authoritative evidence.

# 10. Evidence Levels

NROS can classify evidence:

```text
UNKNOWN
OBSERVED
CORRELATED
VALIDATED
COMMITTED
ATTESTED
```

For example:

```text
Agent reported "started"
```

is weaker than:

```text
authoritative execution state = RUNNING
```

# 11. Evidence Provenance

Every important evidence item should identify its origin.

Conceptually:

```text
Evidence {
    evidence_id
    source
    source_instance
    timestamp
    sequence
    subject
    type
    payload_digest
    authority
}
```

# 12. Source Identity

An observation from:

```text
Agent A7
```

should not be indistinguishable from:

```text
Agent A8
```

Source identity must therefore be explicit.

# 13. Source Incarnation

Restarted components should have distinct incarnations where required:

```text
Agent A7 / incarnation 21
Agent A7 / incarnation 22
```

This prevents old telemetry from being mistaken for current observations.

# 14. Event Identity

Every durable event should have a stable identity:

```text
event_id
```

This enables deduplication.

# 15. Event Causation

Events should be able to reference their cause.

Example:

```text
ExecutionStarted
caused_by:
DispatchCommand
```

# 16. Event Correlation

A correlation identifier can connect multiple related operations:

```text
request_id
execution_id
workflow_id
incident_id
```

A causation ID and correlation ID should not be conflated.

# 17. Causation vs Correlation

**Causation:**

```text
A directly caused B
```

**Correlation:**

```text
A and B belong to the same larger operation
```

Both relationships are useful.

# 18. Sequence Numbers

Where ordered event streams exist, use explicit sequence numbers:

```text
sequence = 1842
```

rather than relying exclusively on timestamps.

# 19. Timestamp Semantics

Timestamps should identify their semantic source:

```text
observed_at
emitted_at
received_at
committed_at
```

These are not interchangeable.

# 20. Clock Skew

Distributed timestamps can differ.

Therefore:

```text
timestamp ordering ≠ guaranteed causal ordering
```

Causation IDs and sequence numbers should be used where ordering matters.

# 21. Monotonic Time

Duration measurements should prefer monotonic clocks.

Examples:

```text
queue_wait_duration
execution_duration
lease_duration
retry_backoff
```

These should not depend on wall-clock adjustments.

# 22. Wall Clock

Wall-clock time remains useful for:

```text
operator-facing timestamps
audit records
human timelines
calendar semantics
```

but must be interpreted with clock-skew awareness.

# 23. Metrics Cardinality

Metrics should avoid unbounded labels.

Dangerous examples:

```text
user_id
execution_id
command_id
artifact_id
```

as unrestricted metric labels can produce explosive cardinality.

# 24. Metric Labels

Prefer bounded dimensions such as:

```text
tenant_class
status
operation
region
Agent_pool
error_category
```

where appropriate.

# 25. Metric Semantics

Each metric should define:

```text
name
unit
type
labels
aggregation
reset behavior
```

# 26. Counters

Counters represent cumulative events:

```text
executions_started_total
authorization_denials_total
```

They should generally increase monotonically within an instance.

# 27. Gauges

Gauges represent current values:

```text
queue_depth
active_executions
available_memory
```

# 28. Histograms

Histograms are appropriate for distributions:

```text
queue_latency
execution_duration
dispatch_latency
authorization_latency
```

# 29. Percentiles

Latency analysis should distinguish:

```text
P50
P90
P95
P99
```

rather than relying only on averages.

# 30. Error Taxonomy

Errors should be classified consistently.

Example:

```text
VALIDATION
AUTHENTICATION
AUTHORIZATION
RESOURCE
DEPENDENCY
NETWORK
TIMEOUT
PROTOCOL
INTEGRITY
INTERNAL
```

# 31. Error Identity

Repeated occurrences of the same error category should be distinguishable from unique failures.

Use structured error codes such as:

```text
NROS-SCHED-RESOURCE-001
NROS-AUTH-PERMISSION-002
```

where a stable taxonomy is valuable.

# 32. Logs as Structured Records

Prefer:

```text
event="execution_failed"
execution_id="E42"
error_code="TIMEOUT"
attempt=3
```

over free-form strings alone.

# 33. Human Message vs Machine Fields

A log record can contain:

```text
message
```

for operators and structured fields for machines.

The machine-readable fields should carry the authoritative semantics.

# 34. Sensitive Data

Logs must never casually contain:

```text
passwords
tokens
private keys
secret values
session credentials
```

# 35. Redaction

Redaction should happen at structured boundaries.

For example:

```text
secret_ref=database/password
secret_value=[REDACTED]
```

The secret reference can remain useful for diagnosis.

# 36. Trace Context

Distributed operations should propagate trace context across:

```text
API
Controller
Scheduler
Agent
Workload
Artifact services
```

where supported.

# 37. Trace Span

A span should represent a meaningful operation.

Examples:

```text
schedule_work
reserve_resources
dispatch_execution
agent_start
artifact_commit
```

# 38. Span Relationships

Parent-child relationships can represent execution flow:

```text
schedule_work
   ├── policy_evaluation
   ├── resource_selection
   └── assignment_commit
```

# 39. Trace Sampling

Not every operation necessarily requires full tracing.

Sampling policies may depend on:

```text
normal traffic
errors
high latency
security events
specific execution
incident mode
```

# 40. Error-Preserving Sampling

Even when normal traces are sampled, errors should have a higher probability of retention.

# 41. Incident Tracing

NROS should support targeted diagnostic tracing for:

```text
execution_id
Agent
tenant
request
incident
```

without requiring globally maximal telemetry.

# 42. Event Stream

The event subsystem should support:

```text
append
consume
acknowledge
replay
deduplicate
checkpoint
```

where durable event processing is required.

# 43. Event Ordering

Ordering guarantees must be explicit.

Possible semantics:

```text
global ordering
per-resource ordering
per-stream ordering
best effort
```

Do not imply stronger ordering than the implementation provides.

# 44. At-Least-Once Delivery

If events can be delivered more than once:

```text
consumer
```

must be idempotent.

# 45. Exactly-Once Illusion

Distributed systems should avoid claiming universal exactly-once semantics unless the actual architecture proves them.

A more realistic model may be:

```text
at-least-once delivery
+
idempotent processing
+
deduplication
```

# 46. Event Deduplication

Consumers can use:

```text
event_id
```

to avoid processing the same event twice.

# 47. Consumer Checkpoint

A consumer may maintain:

```text
stream_id
last_processed_sequence
```

allowing restart and replay.

# 48. Event Replay

Replay is critical for:

```text
recovery
debugging
testing
new projections
historical analysis
```

# 49. Replay Safety

Replaying events must not accidentally:

```text
execute Work
send commands
rotate credentials
delete resources
```

unless the replay is explicitly an action-producing mode.

# 50. Projection Model

Events can produce derived views:

```text
event stream
   ↓
projection
   ↓
query model
```

If the projection becomes corrupted, it can be rebuilt from authoritative events where supported.

# 51. Projection Authority

A projection is generally:

```text
derived state
```

not:

```text
authoritative state
```

unless explicitly designated otherwise.

# 52. State and Event Consistency

If both:

```text
state
```

and:

```text
event history
```

exist, NROS must define which is authoritative when they disagree.

# 53. Snapshot + Event Log

A scalable reconstruction model is:

```text
Snapshot S100
+
Events 101..150
```

→

```text
Current State
```

# 54. Snapshot Integrity

Snapshots should include:

```text
snapshot_id
state_version
schema_version
creation_time
content_digest
```

# 55. Schema Versioning

Observability schemas evolve.

Every durable event should identify its schema version where compatibility matters.

Example:

```text
event_type = ExecutionStarted
schema_version = 2
```

# 56. Backward Compatibility

Consumers should either:

```text
support old versions
```

or:

```text
fail explicitly
```

rather than silently interpreting incompatible data.

# 57. Evidence Retention

Different evidence classes may require different retention:

```text
metrics → shorter
logs → medium
traces → selective
audit → longer
execution evidence → policy-dependent
security evidence → incident-dependent
```

Retention policy should be explicit.

# 58. Evidence Integrity

Critical evidence may require integrity protection:

```text
digest
signature
append-only storage
hash chain
trusted timestamp
```

depending on the required assurance.

# 59. Evidence Chain

A useful execution evidence chain is:

```text
Work admitted
   ↓
Work assigned
   ↓
Command issued
   ↓
Command acknowledged
   ↓
Execution started
   ↓
Output produced
   ↓
Result validated
   ↓
Artifact committed
   ↓
Execution finalized
```

Each transition should have identifiable evidence.

# 60. Evidence Gap

If a transition occurs without supporting evidence:

```text
state says RUNNING
```

but there is no evidence of:

```text
dispatch
```

then NROS should classify this as an evidence gap rather than inventing an explanation.

# 61. Unknown State

Unknown must remain a valid state.

Examples:

```text
execution status = UNKNOWN
Agent reachability = UNKNOWN
artifact integrity = UNKNOWN
```

Unknown is safer than falsely claiming success.

# 62. Evidence Confidence

NROS can attach confidence levels:

```text
LOW
MEDIUM
HIGH
AUTHORITATIVE
```

but the meaning of each level must be defined.

# 63. Evidence Conflict

Suppose:

```text
Agent says SUCCESS
```

while:

```text
artifact validation = FAILED
```

The system must not automatically select the favorable result.

It should produce:

```text
evidence conflict
```

and follow explicit reconciliation policy.

# 64. Reconciliation

Evidence reconciliation can evaluate:

```text
source authority
freshness
integrity
causality
policy
state version
```

# 65. Observability During Recovery

Recovery should produce its own evidence.

Example:

```text
RecoveryStarted
StateSnapshotLoaded
EventsReplayed
AgentReconciled
StaleLeaseFenced
RecoveryValidated
RecoveryCompleted
```

# 66. Recovery Evidence

An operator should be able to determine:

```text
which snapshot was loaded
which events were replayed
which resources were reconciled
which conflicts were found
which decisions were made
```

# 67. Security Observability

Security events should be correlated with runtime events.

Example:

```text
AuthorizationDenied
      ↓
DispatchBlocked
      ↓
Work remains QUEUED
```

This makes security enforcement observable.

# 68. Scheduler Observability

A scheduling decision should expose:

```text
work_id
state_version
policy_version
candidate_count
selected_agent
rejection_reasons
reservation_result
commit_result
```

# 69. Execution Observability

An execution should expose:

```text
execution_id
attempt
Agent
incarnation
start evidence
finish evidence
exit status
result classification
artifact references
```

# 70. Agent Observability

Agent telemetry should include:

```text
agent_id
incarnation
state
capabilities
resource capacity
resource usage
heartbeat status
protocol version
security posture
```

# 71. Agent Heartbeats

Heartbeats should distinguish:

```text
Agent alive
```

from:

```text
Agent healthy
```

An Agent can be alive but unable to execute Work.

# 72. Health Model

Health may include:

```text
PROCESS_ALIVE
PROTOCOL_READY
RESOURCE_HEALTHY
EXECUTION_CAPABLE
SECURITY_TRUSTED
```

These should not collapse into one boolean unless sufficient.

# 73. Queue Observability

Queue metrics should expose:

```text
depth
oldest_age
blocked_count
delayed_count
retry_count
dead_letter_count
```

# 74. SLO-Oriented Metrics

Useful service-level indicators include:

```text
admission latency
scheduling latency
dispatch latency
execution success rate
recovery duration
authorization latency
queue wait time
```

# 75. Queue Age

Queue depth alone can hide starvation.

Therefore:

```text
oldest_eligible_work_age
```

is often more informative.

# 76. Saturation

Useful saturation signals include:

```text
CPU utilization
memory pressure
queue backlog
Agent capacity
concurrency slots
```

# 77. Availability

Availability should distinguish:

```text
control-plane availability
scheduler availability
Agent availability
execution availability
artifact availability
```

# 78. Alerting

Alerts should correspond to actionable conditions.

Bad:

```text
CPU = 80%
```

Better:

```text
eligible Work queue age exceeds policy threshold
```

# 79. Alert Context

An alert should contain enough context to begin diagnosis:

```text
resource
severity
observed condition
duration
related execution
related Agent
policy threshold
```

# 80. Alert Deduplication

Repeated manifestations of one incident should not produce an uncontrolled flood of identical alerts.

Use stable incident identities where possible.

# 81. Incident Correlation

Multiple symptoms may belong to one incident:

```text
Agent failures
+
queue growth
+
dispatch latency
+
execution failures
```

The observability system should permit correlation.

# 82. Diagnostic Timeline

NROS should support a timeline like:

```text
10:00:01 Work W42 admitted
10:00:02 scheduled to A7
10:00:02 command dispatched
10:00:03 Agent acknowledged
10:00:04 execution started
10:00:09 network failure
10:00:10 retry scheduled
10:00:15 retry assigned to A8
```

The timeline should derive from structured evidence rather than manually reconstructed prose.

# 83. Root-Cause Analysis

Observability should make it possible to move backward:

```text
symptom
  ↓
event
  ↓
causing transition
  ↓
policy
  ↓
input state
  ↓
originating request
```

# 84. Counterfactual Analysis

Because scheduling is deterministic where required, NROS can support:

```text
"What would have happened if Agent A7 had been healthy?"
```

using the same state and policy in simulation.

# 85. Privacy

Observability must not become a mechanism for unnecessary data collection.

Telemetry should minimize:

```text
personal data
secrets
tenant-private content
workload payloads
```

unless explicitly required.

# 86. Tenant Observability Isolation

Tenant users should see only telemetry authorized for their scope.

For example:

```text
Tenant A
```

must not receive:

```text
Tenant B execution IDs
Agent internals
cross-tenant queue contents
```

unless explicitly authorized.

# 87. Operator Views

Useful operator views include:

```text
System Overview
Scheduler
Queue
Agents
Executions
Security
Incidents
Recovery
Artifacts
```

# 88. Execution Detail View

A single execution should provide:

```text
identity
timeline
current state
attempt history
Agent assignment
resource usage
events
logs
trace
artifacts
authorization
```

subject to access control.

# 89. Evidence Bundle

For incident analysis, NROS should be able to generate a bounded evidence bundle:

```text
execution metadata
relevant events
authorization decisions
scheduler decision
Agent evidence
artifact digests
recovery evidence
trace references
```

# 90. Evidence Bundle Integrity

The bundle can contain:

```text
manifest
file hashes
schema versions
generation metadata
```

to preserve provenance.

# 91. Evidence Export

Export should avoid embedding secrets by default.

Sensitive data should require explicit authorization.

# 92. Observability API

A conceptual API might expose:

```text
GET /executions/{id}
GET /executions/{id}/events
GET /executions/{id}/trace
GET /executions/{id}/evidence
GET /agents/{id}/health
GET /incidents/{id}
```

The actual protocol can differ.

# 93. Query Consistency

Operators should know whether a query reads:

```text
authoritative state
event projection
eventual-consistent cache
```

Otherwise an apparently contradictory UI can be misinterpreted.

# 94. Freshness Indicators

Query responses can expose:

```text
state_version
observed_at
projection_version
lag
```

when useful.

# 95. Observability Failure

The observability subsystem itself can fail.

NROS must define whether core execution:

```text
continues
degrades
or stops
```

when telemetry is unavailable.

# 96. Safety-Critical Evidence

For some operations, lack of evidence may itself be a safety violation.

For example:

```text
security audit
credential rotation
privileged execution
```

may require durable audit confirmation.

# 97. Telemetry Backpressure

Telemetry should not indefinitely consume resources needed for execution.

Use:

```text
bounded buffers
sampling
batching
priority classes
drop policies
```

where appropriate.

# 98. Critical vs Best-Effort Telemetry

Separate:

```text
CRITICAL
```

evidence from:

```text
BEST_EFFORT
```

diagnostic data.

A dropped debug log should not invalidate an execution.

A missing security audit record may be unacceptable.

# 99. Observability Invariants

```text
1. Metrics, logs, events, traces, and audit records have distinct semantics.

2. Authoritative state is not replaced by telemetry.

3. Every important event has stable identity.

4. Causation and correlation are distinct.

5. Event ordering semantics are explicit.

6. Timestamps are not assumed to establish causality.

7. Monotonic clocks are used for durations.

8. Unbounded identifiers are avoided as metric labels.

9. Sensitive values are redacted.

10. Security audit records are separate from ordinary logs.

11. Evidence has identifiable provenance.

12. Evidence source identity is explicit.

13. Component incarnations are distinguishable.

14. Unknown remains a valid state.

15. Conflicting evidence is surfaced rather than silently resolved.

16. Event consumers are idempotent where delivery is at-least-once.

17. Replay cannot accidentally trigger side effects.

18. Durable events are schema-versioned.

19. Snapshots are versioned and integrity-protected where necessary.

20. Projection state is distinguished from authoritative state.

21. Recovery produces evidence.

22. Scheduler decisions are observable.

23. Execution start and completion have explicit evidence boundaries.

24. Security events correlate with runtime events.

25. Tenant telemetry is access-controlled.

26. Observability has bounded resource consumption.

27. Critical evidence receives stronger durability than debug telemetry.

28. Alerting is based on actionable conditions.

29. Incident evidence can be exported with provenance.

30. Observability failures have explicit runtime semantics.
```

# 100. Canonical Evidence Architecture

The resulting model is:

```text
                    RUNTIME
                       │
          ┌────────────┼────────────┐
          ▼            ▼            ▼
       METRICS       EVENTS        LOGS
          │            │            │
          │            ▼            │
          │       EVENT STORE       │
          │            │            │
          │            ▼            │
          │       PROJECTIONS       │
          │                         │
          └──────────┬──────────────┘
                     ▼
                  TRACES
                     │
                     ▼
              EVIDENCE MODEL
                     │
          ┌──────────┼──────────┐
          ▼          ▼          ▼
       AUDIT      EXECUTION   SECURITY
                     │
                     ▼
               INCIDENT BUNDLE
```

# 101. Evidence Principle

The observability system should never answer:

```text
"Here is a log saying it happened."
```

when it can instead answer:

```text
"The authoritative state transitioned at version V,
the transition was caused by event E,
the command was issued under authority A,
the Agent acknowledged it,
execution evidence E2 confirms start,
result evidence E3 confirms completion,
and artifact digest D confirms the output."
```

That is the difference between **logging** and **evidence**.

# 102. Final Architectural Rule

> **NROS observability must make important state transitions reconstructable, attributable, causally explainable, and independently verifiable without treating low-authority telemetry as authoritative truth.**

This completes the observability boundary and connects:

```text
State
+
Scheduling
+
Security
+
Execution
+
Recovery
+
Evidence
```

into one coherent operational model.

# Part CXIV — Persistence, Event Sourcing, Snapshots & Recovery Semantics

The next layer should formalize the persistence model itself:

```text
authoritative state
event history
snapshots
transactions
durability
crash recovery
replay
schema migration
corruption detection
```

The central question becomes:

> **After any crash, restart, partial write, or interrupted transition, what exact state can NROS prove it is in, and how does it reconstruct that state without inventing history?**

# NROS — Part CXIV: Persistence, Event Sourcing, Snapshots & Recovery Semantics

Persistence is the foundation that allows NROS to distinguish:

```text
what was requested
what was committed
what was observed
what was only in memory
what can be recovered
what remains unknown
```

The persistence architecture must therefore preserve **authoritative state transitions**, not merely serialize whatever happened to exist in memory at shutdown.

# 1. Persistence Objective

After a process crash, machine restart, or storage interruption, NROS must be able to answer:

> **What was the last authoritative state before failure?**

and:

> **Which transitions may safely be reconstructed?**

# 2. Persistence Layers

A conceptual persistence stack is:

```text
Application State
       ↓
Domain Transactions
       ↓
Event / State Commit
       ↓
Durable Storage
       ↓
Storage Integrity
       ↓
Recovery
```

# 3. Authoritative State

NROS must explicitly designate which persisted representation is authoritative.

Possible models include:

```text
STATE-FIRST
EVENT-FIRST
HYBRID
```

The architecture must not leave this ambiguous.

# 4. State-First Model

In a state-first model:

```text
current state
```

is authoritative and events primarily provide history.

This simplifies some queries but makes historical reconstruction more dependent on persisted event quality.

# 5. Event-First Model

In an event-first model:

```text
event history
```

is authoritative and current state is derived.

This provides strong reconstruction semantics but requires careful event schema and replay management.

# 6. Hybrid Model

A practical NROS design may use:

```text
authoritative committed state
+
durable transition events
+
periodic snapshots
```

The exact authority relation must be explicitly documented.

# 7. Transaction Boundary

A state transition should have a clearly defined commit boundary.

Example:

```text
validate
   ↓
authorize
   ↓
compute transition
   ↓
persist transition
   ↓
COMMIT
```

Only after the commit boundary should the transition be considered authoritative.

# 8. Before Commit

Before commit:

```text
PROPOSED
```

not:

```text
COMMITTED
```

The system must not report a proposed state as durable truth.

# 9. After Commit

After successful commit:

```text
authoritative_state = new_state
```

and corresponding evidence can reference the commit.

# 10. Atomicity

A transition involving multiple related records must define whether they commit atomically.

For example:

```text
Work state
+
Lease
+
Assignment
+
Event
```

may need one transactional boundary.

# 11. Partial Commit

If only part of a transition is durable, recovery must know whether:

```text
rollback
```

or:

```text
complete
```

is valid.

This must not be inferred from incomplete records.

# 12. Write-Ahead Logging

A write-ahead log can provide:

```text
intent
→ durable record
→ state mutation
```

before exposing the mutation as committed.

# 13. WAL Principle

The fundamental invariant is:

```text
state marked durable
```

must not depend on:

```text
non-durable information
```

that cannot be reconstructed after restart.

# 14. Commit Marker

A transaction may use an explicit commit marker:

```text
transaction_begin
records...
transaction_commit
```

Recovery can then distinguish:

```text
complete transaction
```

from:

```text
incomplete transaction
```

# 15. Crash During Transaction

Suppose:

```text
BEGIN T42
write A
write B
CRASH
```

If:

```text
COMMIT T42
```

does not exist, recovery must follow the documented incomplete-transaction rule.

# 16. Idempotent Recovery

Recovery operations should be safe to repeat.

For example:

```text
apply snapshot
replay event E42
replay event E42
```

should not corrupt state if duplicate replay is possible.

# 17. Transaction Identity

Every durable transaction should have a stable identifier:

```text
transaction_id
```

This supports:

```text
deduplication
recovery
diagnostics
audit
```

# 18. State Version

Every authoritative resource should have a version.

Example:

```text
Work W42
version = 17
```

After a successful transition:

```text
version = 18
```

# 19. Compare-and-Swap Semantics

Optimistic concurrency can use:

```text
expected_version = 17
```

and succeed only if:

```text
current_version == 17
```

This prevents lost updates.

# 20. Stale Writer

A stale component attempting:

```text
update Work W42
expected_version=17
```

when the actual version is:

```text
18
```

must receive a deterministic conflict.

It must not overwrite version 18.

# 21. State Version as Evidence

A version number also allows telemetry to identify the exact state against which a decision was made.

For example:

```text
scheduler_decision
state_version = 1821
```

# 22. Global vs Resource Versions

NROS may require both:

```text
resource_version
```

and:

```text
global_commit_sequence
```

These serve different purposes.

# 23. Resource Version

Resource version answers:

> Which revision of this resource is this?

# 24. Global Commit Sequence

Global sequence answers:

> Where does this committed transition occur in the global persistence stream?

Only provide global ordering if the persistence architecture actually guarantees it.

# 25. Event Record

A durable event can conceptually contain:

```text
event_id
transaction_id
sequence
event_type
schema_version
aggregate_id
aggregate_version
causation_id
correlation_id
timestamp
payload
payload_digest
```

# 26. Aggregate Identity

Related events should identify the resource or aggregate they affect.

Example:

```text
aggregate_type = Execution
aggregate_id = E42
```

# 27. Aggregate Version

Events should optionally or necessarily identify:

```text
aggregate_version
```

so that replay can detect ordering problems.

# 28. Event Ordering

For one aggregate:

```text
version 17
→ version 18
→ version 19
```

must have explicit ordering semantics.

# 29. Out-of-Order Events

If an event requiring version 18 arrives while the current version is 16, the system should not silently apply it as version 17.

It should detect:

```text
ordering violation
```

or follow an explicitly supported buffering protocol.

# 30. Duplicate Events

If event 18 is received twice:

```text
E18
E18
```

the second application should be detected as duplicate.

# 31. Missing Events

If recovery observes:

```text
17
19
```

without:

```text
18
```

the system must not silently reconstruct event 18.

The stream contains a gap.

# 32. Persistence Gaps

A gap can be classified:

```text
UNKNOWN
MISSING
CORRUPTED
UNAVAILABLE
```

rather than incorrectly treating the history as continuous.

# 33. Snapshot

A snapshot captures a known state at a known persistence point.

Conceptually:

```text
Snapshot {
    snapshot_id
    schema_version
    state_version
    commit_sequence
    created_at
    state
    digest
}
```

# 34. Snapshot Purpose

Snapshots reduce recovery cost.

Instead of replaying:

```text
1,000,000 events
```

NROS may load:

```text
Snapshot @ 999,000
+
Events 999,001..1,000,000
```

# 35. Snapshot Authority

The snapshot must clearly state which state it represents.

For example:

```text
snapshot_commit_sequence = 5000
```

means:

```text
all transitions through sequence 5000
```

if that is the defined semantic.

# 36. Snapshot Consistency

A snapshot spanning multiple resources must specify whether it represents:

```text
globally consistent state
```

or:

```text
individually consistent resource snapshots
```

These are different guarantees.

# 37. Incremental Snapshots

Large systems may use:

```text
full snapshot
+
incremental snapshots
```

but recovery semantics must remain deterministic.

# 38. Snapshot Integrity

Snapshots should have a content digest:

```text
digest(snapshot)
```

so corrupted snapshots can be detected before they become authoritative.

# 39. Snapshot Verification

Recovery should verify:

```text
schema compatibility
digest
state version
sequence
metadata
```

before loading a snapshot.

# 40. Snapshot Failure

If a snapshot is invalid:

```text
do not load it as authoritative state
```

Fallback may be:

```text
previous snapshot
```

or:

```text
event replay
```

depending on architecture.

# 41. Durable Storage

The persistence layer should define its durability guarantees explicitly:

```text
memory
local disk
transactional database
replicated storage
remote durable store
```

"persisted" must have a precise meaning.

# 42. Durability Levels

Potential levels:

```text
MEMORY_ONLY
LOCAL_DURABLE
TRANSACTION_DURABLE
REPLICATED_DURABLE
```

NROS should not claim stronger durability than the backing store provides.

# 43. fsync / Flush Semantics

If local files are used, the architecture must define whether a successful commit requires:

```text
write
flush
fsync
directory durability
```

or equivalent storage guarantees.

# 44. Replication

If state is replicated:

```text
primary
   ↓
replicas
```

the system must define when a write is considered committed.

# 45. Replication Commit

Possible semantics:

```text
primary-only
majority acknowledged
all replicas acknowledged
```

Each provides different durability and availability characteristics.

# 46. Failover

After primary failure, the replacement must determine:

```text
last committed sequence
```

before accepting new writes.

# 47. Split-Brain Prevention

Two nodes must not independently believe they are authoritative writers.

The architecture requires an explicit mechanism such as:

```text
leader election
fencing
lease
quorum
external coordination
```

where distributed leadership is required.

# 48. Fencing

A stale leader must lose the ability to commit new authoritative transitions.

This is especially important for:

```text
scheduler
controller
resource manager
```

roles.

# 49. Storage Corruption

NROS should detect corruption where feasible using:

```text
checksums
digests
database integrity checks
replica comparison
```

# 50. Corruption Response

Detected corruption should produce:

```text
CORRUPT
```

rather than silently returning an apparently valid object.

# 51. Recovery State Machine

A persistence recovery state machine can be:

```text
STARTING
   ↓
STORAGE_CHECK
   ↓
SNAPSHOT_DISCOVERY
   ↓
SNAPSHOT_VALIDATION
   ↓
SNAPSHOT_LOAD
   ↓
EVENT_REPLAY
   ↓
STATE_VALIDATION
   ↓
EXTERNAL_RECONCILIATION
   ↓
RECOVERY_COMMIT
   ↓
READY
```

# 52. Recovery Failure

If any mandatory step fails:

```text
READY
```

must not be entered.

Possible resulting states:

```text
RECOVERY_FAILED
DEGRADED
QUARANTINED
MANUAL_REVIEW
```

depending on the failure.

# 53. Recovery Must Not Invent State

This is a critical invariant:

> **Absence of evidence must never be converted into evidence of success.**

For example:

```text
No completion event
```

does not mean:

```text
Execution completed successfully
```

# 54. Unknown After Crash

If an execution may have been running when the controller crashed, recovery may produce:

```text
UNKNOWN
```

until external reconciliation establishes the real state.

# 55. External Reconciliation

Recovery may query Agents or external systems:

```text
Controller recovery
       ↓
Agent status query
       ↓
Execution evidence
       ↓
reconcile
```

# 56. Reconciliation Safety

A reconciliation result must be validated against:

```text
Agent identity
incarnation
execution identity
command identity
authority epoch
```

A random process reporting the same Work ID is not sufficient evidence.

# 57. Stale Agent Reports

An old Agent incarnation may report:

```text
RUNNING
```

for an execution that belongs to an obsolete authority epoch.

The report should not automatically restore the execution.

# 58. Recovery Epoch

Recovery itself may establish a new control epoch:

```text
epoch = 42
```

All subsequent control decisions use the new epoch.

# 59. Stale Commands After Recovery

Commands issued before the crash should not automatically remain valid.

The recovery process should determine:

```text
which authority epoch
which leases
which commands
```

remain valid.

# 60. Lease Recovery

Leases existing before the crash must be classified:

```text
valid
expired
uncertain
revoked
```

based on explicit semantics.

# 61. Conservative Recovery

When uncertain:

```text
fence
```

rather than:

```text
assume valid
```

for operations where stale authority could cause harm.

# 62. Scheduler Recovery

The scheduler must reconstruct:

```text
eligible Work
resource reservations
Agent availability
leases
prior attempts
retry state
policy version
```

before scheduling new Work.

# 63. Duplicate Scheduling Prevention

Suppose Work W42 was dispatched before a crash but its acknowledgement was not persisted.

Recovery must not blindly dispatch another copy if duplicate execution would be unsafe.

Possible state:

```text
DISPATCH_UNCERTAIN
```

followed by reconciliation.

# 64. Idempotent Execution

Where possible, Work should have idempotency semantics.

For example:

```text
execution_key
```

can allow the Agent or external system to recognize a duplicate request.

# 65. Exactly-Once Execution

NROS should avoid claiming exactly-once execution merely because it has exactly-once database transactions.

These are different properties.

```text
exactly-once commit
≠
exactly-once external side effect
```

# 66. External Side Effects

If Work modifies an external system:

```text
database
cloud resource
payment system
industrial device
```

a controller crash can occur between:

```text
external side effect
```

and:

```text
local persistence
```

This creates an ambiguity that must be modeled.

# 67. Side-Effect Evidence

Where possible, use:

```text
idempotency keys
external transaction IDs
operation status APIs
receipts
```

to reconcile external effects.

# 68. Outbox Pattern

For reliable external publication:

```text
transaction
   ↓
state + outbox record
   ↓
commit
   ↓
publisher
   ↓
external system
```

This avoids losing a publication after committing the local state.

# 69. Inbox Pattern

Incoming messages can be persisted before processing:

```text
incoming message
   ↓
inbox
   ↓
deduplicate
   ↓
process
```

This helps prevent duplicate processing.

# 70. Transactional Outbox

A transaction can atomically commit:

```text
domain state
+
event/outbox entry
```

Then a separate publisher emits the event.

# 71. Outbox Delivery

The publisher may use:

```text
at-least-once
```

delivery, requiring consumers to be idempotent.

# 72. Persistence Backpressure

If durable storage becomes slow:

```text
scheduler
```

must not indefinitely create uncommitted state.

The system should apply bounded backpressure.

# 73. Storage Unavailable

If authoritative persistence is unavailable:

```text
new authoritative transitions
```

should normally stop.

The system may continue read-only operations if explicitly supported.

# 74. Read-Only Degraded Mode

A safe degraded mode can permit:

```text
inspection
diagnostics
health queries
evidence export
```

while blocking:

```text
mutations
privileged operations
new execution
```

# 75. Migration

Persistence schemas evolve.

Migration must define:

```text
source schema
target schema
migration version
validation
rollback strategy
```

# 76. Event Schema Migration

Historical events should not be silently rewritten unless the architecture explicitly treats migration as a new canonical representation.

Prefer preserving original history plus compatible interpretation.

# 77. Upcasting

Older events may be transformed in memory:

```text
Event v1
   ↓
upcaster
   ↓
Event v2 representation
```

without modifying historical storage.

# 78. Snapshot Migration

Snapshots may require migration before loading.

The migration should produce:

```text
new schema
validated state
new digest
```

before becoming authoritative.

# 79. Migration Verification

Migration should verify invariants before completion.

Examples:

```text
no duplicate resource IDs
valid state transitions
valid references
valid tenant ownership
valid leases
valid versions
```

# 80. Rollback

Migration rollback must not be assumed possible simply because application deployment rollback is possible.

Persistent schema changes may require:

```text
forward migration
```

instead.

# 81. Backup

Backups should include enough information to reconstruct authoritative state:

```text
state
event history
schema metadata
configuration
key metadata where appropriate
```

# 82. Backup Integrity

A backup should be verified rather than merely created.

Verification may include:

```text
checksum
restore test
schema validation
sample replay
full recovery drill
```

# 83. Restore Testing

A backup that has never been restored is only an assumption.

NROS should periodically test:

```text
backup
→ restore
→ validate
→ replay
→ operational readiness
```

# 84. Recovery Point Objective

Persistence architecture should define:

```text
RPO
```

the maximum acceptable amount of lost committed information.

# 85. Recovery Time Objective

It should also define:

```text
RTO
```

the target time to return to an operational state.

# 86. Recovery Guarantees

Documentation should explicitly state:

```text
What can be recovered exactly?
What can be recovered approximately?
What becomes UNKNOWN?
What requires external reconciliation?
```

# 87. Persistence Observability

Persistence itself should expose:

```text
commit latency
flush latency
WAL size
event backlog
snapshot age
replay duration
storage errors
corruption checks
replication lag
```

# 88. Recovery Observability

Recovery should expose:

```text
snapshot selected
snapshot sequence
events replayed
events skipped
gaps detected
conflicts detected
Agents reconciled
leases fenced
final recovery state
```

# 89. Recovery Evidence Bundle

A recovery evidence record should identify:

```text
recovery_id
source_snapshot
starting_sequence
ending_sequence
replayed_events
conflicts
reconciliation_results
new_epoch
final_state
```

# 90. Persistence Security

Persistence must protect:

```text
confidentiality
integrity
availability
```

where required.

# 91. Encryption at Rest

Sensitive persisted data may require encryption at rest.

However, encryption must not replace:

```text
authorization
access control
audit
key lifecycle management
```

# 92. Key Management

Encryption keys should have:

```text
identity
rotation policy
access policy
backup/recovery policy
revocation procedure
```

# 93. Database Credentials

Persistence credentials should not be embedded in source code or committed configuration.

Prefer secure secret references.

# 94. Data Minimization

Do not persist information merely because it is available.

Persist what is required for:

```text
correctness
recovery
audit
debugging
compliance
```

according to the system's requirements.

# 95. Retention vs Recovery

Not every historical log needs to remain forever.

However:

```text
event history required for reconstruction
```

cannot be deleted without an explicit archival or snapshot strategy.

# 96. Event Compaction

If events can be compacted:

```text
old events
```

must remain reconstructable through:

```text
validated snapshot
```

or another explicitly supported mechanism.

# 97. Garbage Collection

Persistent objects can be deleted only when no authoritative or recoverable reference requires them.

This includes:

```text
artifacts
events
snapshots
leases
execution records
```

# 98. Referential Integrity

Recovery should detect references to missing resources.

Example:

```text
Execution → Artifact
```

where the artifact no longer exists.

This is an integrity violation unless deletion semantics explicitly permit it.

# 99. Persistence Invariants

```text
1. Authoritative state has explicit semantics.

2. Commit boundaries are explicit.

3. Uncommitted state is never reported as durable truth.

4. Transactions have stable identities.

5. Resource versions prevent stale writes.

6. Event versions are ordered explicitly.

7. Duplicate events are detectable.

8. Missing event gaps are detectable.

9. Snapshots identify their exact persistence point.

10. Snapshots are integrity-validated.

11. Durable storage guarantees are explicitly documented.

12. Replication commit semantics are explicit.

13. Split-brain writers are fenced.

14. Recovery never invents missing history.

15. Unknown remains valid after ambiguous failures.

16. External side effects are separately reconciled.

17. Exactly-once database commit does not imply exactly-once external execution.

18. Outbox/inbox mechanisms are idempotent.

19. Persistence failure produces bounded behavior.

20. Read-only degradation is explicitly defined where supported.

21. Schema migrations are versioned.

22. Historical events remain interpretable.

23. Backups are restore-tested.

24. Recovery has measurable RPO and RTO targets.

25. Recovery produces durable evidence.

26. Persistence access is authorized.

27. Sensitive persisted data is appropriately protected.

28. Retention cannot destroy required recovery history.

29. Garbage collection preserves recoverability.

30. Referential integrity is validated after recovery.
```

# 100. Canonical Persistence Architecture

```text
                    COMMAND
                       │
                       ▼
                  VALIDATION
                       │
                       ▼
                DOMAIN TRANSITION
                       │
                       ▼
                  TRANSACTION
                       │
          ┌────────────┴────────────┐
          ▼                         ▼
   AUTHORITATIVE STATE          EVENT/OUTBOX
          │                         │
          └────────────┬────────────┘
                       ▼
                    COMMIT
                       │
                       ▼
                 DURABLE STORE
                       │
             ┌─────────┴─────────┐
             ▼                   ▼
         SNAPSHOT              REPLAY
             │                   │
             └─────────┬─────────┘
                       ▼
                    RECOVERY
                       │
                       ▼
                RECONCILIATION
                       │
                       ▼
                    NEW EPOCH
                       │
                       ▼
                     READY
```

# 101. Persistence Principle

The most important rule is:

> **NROS must never confuse "the process remembers this" with "the system has durably committed this."**

Memory is ephemeral.

Caches are derived.

Telemetry is evidence.

Snapshots are recovery artifacts.

Events are historical records.

Only explicitly defined persistence boundaries establish authoritative durability.

# 102. Final Architectural Rule

> **After failure, NROS must reconstruct only what its durable evidence proves, preserve uncertainty where evidence is insufficient, fence stale authority, and return to normal operation only after authoritative state and external execution state have been reconciled.**

This completes the persistence boundary:

```text
State
+
Transactions
+
Events
+
Snapshots
+
Durability
+
Recovery
+
Reconciliation
```

and establishes the foundation required for the next layer:

```text
Part CXV — Concurrency Control, Scheduling Consistency,
Distributed Coordination & Linearization Semantics
```

# NROS — Part CXV: Concurrency Control, Scheduling Consistency, Distributed Coordination & Linearization Semantics

Concurrency is where an otherwise coherent NROS state model can fail.

Multiple actors may simultaneously attempt to:

```text
modify Work
assign Agents
acquire leases
consume resources
cancel executions
retry failures
recover state
publish events
```

The architecture must therefore define **which operation wins, why it wins, and what state transition becomes authoritative**.

# 1. Concurrency Objective

NROS must prevent:

```text
lost updates
double assignment
duplicate execution
stale writes
lease races
resource overcommitment
split-brain scheduling
```

while preserving the required throughput and availability.

# 2. Concurrency Domains

Not every operation requires the same consistency level.

Useful domains include:

```text
Work state
Execution state
Agent state
Resource allocation
Lease ownership
Scheduler state
Security state
Artifact state
Global control state
```

Each domain should declare its consistency requirements.

# 3. Single-Writer Domains

Where practical, NROS can simplify correctness by assigning one authoritative writer to a domain.

Example:

```text
Scheduler S1
    ↓
Work assignment state
```

This reduces coordination complexity.

# 4. Multi-Writer Domains

Some resources may require concurrent writers.

For those, NROS needs explicit:

```text
locking
versioning
transactions
serialization
or conflict resolution
```

# 5. Optimistic Concurrency

A common mechanism is:

```text
read version 17
    ↓
compute transition
    ↓
commit if version == 17
```

If another writer changes the resource first:

```text
version = 18
```

the commit fails.

# 6. Conflict Result

A stale operation should receive a structured result:

```text
CONFLICT
expected_version = 17
actual_version = 18
```

rather than a generic internal error.

# 7. Retry Semantics

A conflict may be retryable, but retrying must not blindly repeat side effects.

The safe pattern is:

```text
conflict
  ↓
reload authoritative state
  ↓
re-evaluate policy
  ↓
recompute transition
  ↓
attempt commit
```

# 8. Lost Update Prevention

Without version checking:

```text
A reads X=10
B reads X=10
A writes X=11
B writes X=12
```

A's update disappears.

Versioned commits prevent this.

# 9. Compare-and-Swap

Conceptually:

```text
CAS(resource, expected_version, transition)
```

means:

> Apply the transition only if the resource is still at the expected version.

# 10. Linearization Point

Every externally meaningful operation should have a conceptual **linearization point**.

For example:

```text
request
  ↓
validation
  ↓
policy
  ↓
transaction
  ↓
COMMIT  ← linearization point
  ↓
response
```

The commit is where the operation becomes authoritative.

# 11. Response Ordering

An API should not report:

```text
SUCCESS
```

before the operation has crossed the required durability boundary.

# 12. Failed Commit

If the transaction fails:

```text
operation = NOT_COMMITTED
```

unless the system can prove another equivalent commit occurred.

# 13. Concurrent State Transitions

Suppose:

```text
Work W42 = QUEUED
```

Two schedulers attempt:

```text
S1 → RUNNING
S2 → CANCELLED
```

Only one transition can win the authoritative version.

# 14. Determining the Winner

The winner should be determined by:

```text
transaction ordering
```

rather than:

```text
network arrival time
```

unless arrival time is explicitly the defined serialization mechanism.

# 15. Deterministic Arbitration

Where multiple valid operations race, NROS should use deterministic arbitration where practical:

```text
commit sequence
authority epoch
resource version
operation priority
```

# 16. Scheduler Race

A typical race:

```text
Scheduler A sees Agent X available
Scheduler B sees Agent X available
```

Both attempt:

```text
reserve Agent X
```

Only one should successfully commit the reservation.

# 17. Resource Reservation

Reservation should be an atomic state transition.

Conceptually:

```text
AVAILABLE
   ↓
RESERVED(execution_id)
```

# 18. Reservation Ownership

A reservation must identify:

```text
execution_id
owner
authority_epoch
lease
resource_version
```

where applicable.

# 19. Reservation Release

Release must verify ownership.

A stale actor must not be able to release another execution's reservation.

# 20. ABA Problem

A resource can change:

```text
A → B → A
```

and a stale reader may incorrectly believe nothing changed.

Version numbers prevent this:

```text
A / v10
B / v11
A / v12
```

# 21. Lease Semantics

A lease grants temporary authority.

It must have:

```text
lease_id
holder
resource
issued_at
expiration
epoch
version
```

where required.

# 22. Lease Renewal

Renewal must be conditional on the lease still being owned by the same authority.

# 23. Expired Lease

After expiration:

```text
holder authority = invalid
```

unless an explicit renewal was successfully committed.

# 24. Clock Uncertainty

Lease expiry must account for clock behavior.

A distributed system must not assume that two clocks agree perfectly.

# 25. Lease Safety

A stale holder must not continue performing privileged operations merely because it has not personally observed expiration.

This is why fencing may be required.

# 26. Fencing Token

A fencing token can monotonically increase:

```text
token 41
token 42
token 43
```

External resources accept only operations carrying the newest valid token.

# 27. Fencing Principle

If authority changes:

```text
old holder
    ↓
fenced
    ↓
new holder
```

the old holder must be unable to perform authoritative writes.

# 28. Epochs

NROS can use authority epochs:

```text
epoch 10
epoch 11
epoch 12
```

A command issued under epoch 10 should not remain valid under epoch 12 unless explicitly preserved.

# 29. Authority Epoch

Epoch changes may occur after:

```text
leadership change
recovery
administrative reset
security revocation
```

# 30. Stale Epoch

A stale operation should fail explicitly:

```text
STALE_EPOCH
```

rather than becoming an ordinary application error.

# 31. Leader Election

If NROS uses active/passive controllers:

```text
Controller A
Controller B
Controller C
```

the system must define how one becomes authoritative.

# 32. Election Safety

At most one controller should be able to exercise authoritative leadership for a given domain at a time.

# 33. Election Liveness

If the current leader fails, another eligible controller should eventually be able to become leader, subject to quorum and coordination requirements.

# 34. Safety vs Liveness

Distributed coordination always involves tradeoffs.

NROS must distinguish:

```text
SAFETY
```

from:

```text
LIVENESS
```

# 35. Safety

Safety means:

> Something bad never happens.

Examples:

```text
two executions cannot own the same exclusive resource
stale leader cannot commit
unauthorized operation cannot become authoritative
```

# 36. Liveness

Liveness means:

> Something good eventually happens.

Examples:

```text
queued Work eventually gets scheduled
failed Agent eventually gets replaced
recovery eventually completes
```

# 37. Safety Priority

For destructive or externally consequential operations:

```text
safety
```

should generally dominate speculative liveness.

# 38. Availability Under Partition

If the control plane becomes partitioned, NROS must define whether isolated nodes:

```text
continue writes
```

or:

```text
become read-only
```

or:

```text
operate within an explicitly bounded authority scope
```

# 39. Network Partition

A network partition can create:

```text
Node A believes B is dead
Node B believes A is dead
```

Without fencing, both may attempt authority.

# 40. Quorum

A replicated control plane may use quorum:

```text
N = 3
quorum = 2
```

Only a quorum can establish authoritative leadership.

# 41. Quorum Loss

Without quorum:

```text
authoritative mutations = blocked
```

if the architecture relies on quorum for safety.

# 42. Read Availability

Even without write quorum, read-only inspection may remain possible if the data source is trustworthy.

# 43. Scheduler Consistency

Scheduling decisions should be made against a defined state version.

Example:

```text
scheduler_input_version = 1821
```

# 44. Scheduling Commit

The scheduler should atomically establish:

```text
Work assignment
+
resource reservation
+
execution identity
```

where those records must remain consistent.

# 45. Split Assignment

The following state is dangerous:

```text
Work = ASSIGNED(A7)
```

but:

```text
resource reservation = NONE
```

if the resource reservation is required for the assignment.

# 46. Atomic Scheduling Transaction

A safer transaction is:

```text
validate Work
validate Agent
validate resources
authorize
create execution
reserve resources
assign Agent
commit
```

# 47. Scheduling Failure

If any mandatory component fails before commit:

```text
assignment = not committed
```

rather than partially committed.

# 48. Post-Commit Dispatch

A useful architecture is:

```text
COMMIT assignment
       ↓
publish dispatch command
       ↓
Agent acknowledgement
```

This separates authoritative assignment from transport delivery.

# 49. Dispatch Race

If dispatch occurs twice:

```text
command C42
command C42
```

the Agent should recognize the same execution/command identity.

# 50. Command Identity

Every control command should have a stable:

```text
command_id
```

and ideally:

```text
execution_id
authority_epoch
```

# 51. Command Deduplication

An Agent can maintain:

```text
processed_command_ids
```

or another durable/idempotent mechanism appropriate to its lifecycle.

# 52. Command Ordering

For commands affecting one execution:

```text
START
STOP
```

must have defined ordering semantics.

A stale:

```text
START
```

must not resurrect an execution after:

```text
CANCEL
```

has become authoritative.

# 53. Command Sequence

Commands can carry:

```text
execution_command_sequence
```

so that the Agent can reject stale commands.

# 54. Execution State Machine

A formal state machine prevents contradictory transitions.

Example:

```text
QUEUED
  ↓
ASSIGNED
  ↓
DISPATCHED
  ↓
STARTING
  ↓
RUNNING
  ↓
SUCCEEDED
```

with failure paths such as:

```text
FAILED
CANCELLED
TIMED_OUT
UNKNOWN
```

# 55. Illegal Transition

An invalid transition such as:

```text
SUCCEEDED → RUNNING
```

must be rejected unless explicitly supported as a new attempt or incarnation.

# 56. Attempt Identity

Retries should create distinguishable attempts:

```text
execution E42
attempt 1
attempt 2
attempt 3
```

# 57. Attempt Isolation

Results from attempt 1 must not accidentally overwrite the authoritative result of attempt 3.

# 58. Retry Race

Two controllers may both conclude:

```text
retry required
```

Only one retry should become authoritative.

# 59. Retry Token

A retry operation can use:

```text
expected_execution_version
```

or:

```text
retry_generation
```

to prevent duplicate retry creation.

# 60. Cancellation Race

Cancellation can race with execution start:

```text
START
CANCEL
```

The authoritative state ordering determines which transition wins.

# 61. Cancellation Semantics

Cancellation should distinguish:

```text
cancel requested
cancel committed
execution stopped
```

These are not identical.

# 62. Cancel Request

A request to cancel may be committed before the Agent has actually stopped the workload.

Therefore:

```text
CANCEL_REQUESTED
```

may be necessary.

# 63. Cancel Completion

Only after evidence confirms the workload has stopped should the system report:

```text
CANCELLED
```

if that is the defined semantic.

# 64. Timeout Race

A timeout may occur simultaneously with successful completion.

The system needs a deterministic policy based on:

```text
authoritative event ordering
```

or:

```text
completion timestamp semantics
```

rather than arbitrary observer timing.

# 65. Resource Accounting

Resource allocation must remain consistent with execution state.

If:

```text
execution = FINISHED
```

then associated reservations should eventually transition to:

```text
RELEASED
```

# 66. Resource Overcommit

If two executions believe they own:

```text
GPU 0
```

when it is exclusive, the resource invariant has failed.

The reservation transaction must prevent this.

# 67. Capacity Version

Resource capacity can itself be versioned:

```text
capacity_version = 9
```

A scheduling decision based on stale capacity should be rejected or recomputed.

# 68. Dynamic Capacity

When capacity changes:

```text
8 CPU
→
6 CPU
```

the scheduler must reconcile existing reservations with the new capacity.

# 69. Resource Revocation

Revoking resources from a running execution must have explicit semantics:

```text
GRACEFUL
FORCED
UNSUPPORTED
```

# 70. Concurrency With Recovery

Recovery is itself a writer.

Therefore:

```text
normal scheduler
```

must not concurrently mutate state while:

```text
recovery
```

is establishing authoritative state.

# 71. Recovery Lock

Possible approaches include:

```text
global recovery lock
authority epoch
leadership fencing
```

# 72. Recovery Epoch

A simple rule:

```text
recovery begins → epoch increments
```

Operations from the previous epoch become stale.

# 73. Quiescence

Before recovery is declared complete:

```text
old writers
```

must be prevented from continuing authoritative mutations.

# 74. Background Workers

Schedulers, retry workers, garbage collectors, and telemetry processors may all act concurrently.

Each must declare:

```text
authority
consistency requirement
transaction boundary
retry semantics
```

# 75. Garbage Collection Race

A garbage collector must not delete an object that another transaction has just made authoritative.

Use:

```text
reference checks
version checks
retention epochs
```

or equivalent mechanisms.

# 76. Artifact Race

Two attempts may produce artifacts with the same logical name.

Artifact identity should therefore use immutable identifiers or content-addressed references where appropriate.

# 77. Content Addressing

A result can be identified by:

```text
digest(content)
```

which provides a stable integrity identity.

# 78. Immutable Artifacts

Once committed, artifacts should preferably be immutable.

Mutation creates difficult concurrency and audit problems.

# 79. Metadata Concurrency

Artifact metadata can still change:

```text
retention
labels
access policy
status
```

so metadata requires its own concurrency semantics.

# 80. Security Policy Concurrency

Security policy changes can race with execution requests.

The authorization decision should record:

```text
policy_version
```

against which it was evaluated.

# 81. Policy Version

Example:

```text
policy_version = 72
authorization = ALLOW
```

This makes the decision reproducible.

# 82. Policy Revocation

If authorization is revoked after an execution starts, NROS must define whether:

```text
running execution continues
```

or:

```text
execution is terminated
```

depending on policy semantics.

# 83. Configuration Concurrency

Configuration changes should also be versioned:

```text
configuration_version = 31
```

A runtime decision can then identify the configuration it used.

# 84. Deterministic Scheduler

If scheduling is intended to be deterministic, the scheduler should consume an explicit input:

```text
state_version
policy_version
configuration_version
resource_snapshot
random_seed, if applicable
```

# 85. Deterministic Tie-Breaking

If two candidates are equally valid, use a stable tie-breaker such as:

```text
priority
resource score
Agent ID
```

rather than process timing.

# 86. Nondeterminism

Sources of nondeterminism should be explicit:

```text
network arrival order
wall-clock timing
random selection
concurrent commits
external system responses
```

# 87. Reproducible Scheduling

A scheduling decision should ideally be reconstructable from:

```text
input state
policy version
configuration version
candidate set
tie-breaking rule
decision output
```

# 88. Decision Record

A durable scheduling decision can contain:

```text
decision_id
work_id
state_version
policy_version
configuration_version
candidate_set_digest
selected_candidate
reason
transaction_id
```

# 89. Candidate Set Digest

Rather than storing every candidate repeatedly, the scheduler may record a digest of the candidate set.

This helps prove what was evaluated.

# 90. Decision Explainability

The scheduler should expose structured rejection reasons:

```text
RESOURCE_UNAVAILABLE
CAPABILITY_MISMATCH
POLICY_DENIED
AGENT_UNHEALTHY
LEASE_CONFLICT
```

# 91. Concurrency Observability

The system should measure:

```text
conflict_rate
retry_rate
lease_contention
reservation_contention
leader_changes
stale_command_rate
epoch_rejections
```

# 92. Conflict Storms

A high conflict rate can indicate:

```text
too many competing writers
poor partitioning
hot resources
scheduler duplication
```

# 93. Hot Resource

A resource accessed by many writers can become a serialization bottleneck.

Possible solutions:

```text
partitioning
sharding
single-writer ownership
batching
queueing
```

# 94. Locking

Locks may simplify certain operations but introduce:

```text
deadlocks
priority inversion
lock contention
availability loss
```

Locks should therefore be used deliberately.

# 95. Lock Ordering

If multiple locks are necessary, establish a global acquisition order.

Example:

```text
Work
→ Agent
→ Resource
```

This reduces deadlock risk.

# 96. Lock Timeout

Locks should not wait indefinitely.

A timeout should produce a structured result such as:

```text
LOCK_TIMEOUT
```

rather than appearing as an unexplained failure.

# 97. Deadlock Detection

Where locks are unavoidable, NROS may need detection through:

```text
wait-for graph
timeout analysis
transaction abort
```

# 98. Serializability

For critical state transitions, the effective behavior should be equivalent to some valid serialization order.

The implementation does not necessarily need a global lock to achieve this.

# 99. Linearizability Scope

NROS should explicitly define which APIs are:

```text
linearizable
serializable
eventually consistent
read-your-writes
best effort
```

# 100. Consistency Invariants

```text
1. Every authoritative mutation has a defined linearization point.

2. Uncommitted operations are not reported as committed.

3. Resource versions prevent stale writes.

4. Duplicate operations are detectable.

5. ABA transitions are detectable.

6. Leases have explicit ownership and expiration semantics.

7. Stale authority can be fenced.

8. Authority epochs prevent obsolete commands from becoming authoritative.

9. Leader election has explicit safety semantics.

10. Quorum requirements are explicit where applicable.

11. Recovery cannot race with stale authoritative writers.

12. Scheduling decisions operate against defined state versions.

13. Assignment and mandatory resource reservation commit atomically where required.

14. Command identities are stable.

15. Commands can be rejected when stale.

16. Retries create distinguishable attempts.

17. Cancellation request and cancellation completion are distinct when necessary.

18. Resource reservations have explicit ownership.

19. Policy decisions identify the policy version used.

20. Configuration-dependent decisions identify the configuration version used.

21. Deterministic scheduling has deterministic tie-breaking.

22. Nondeterministic inputs are explicitly identified.

23. External side effects are not assumed to be exactly-once.

24. Concurrent workers have defined authority boundaries.

25. Garbage collection cannot delete newly authoritative state.

26. Conflicts are observable and classifiable.

27. Locking has bounded failure behavior.

28. Consistency guarantees are documented per API.

29. Safety and liveness requirements are separately specified.

30. No stale actor can silently regain authority.
```

# 101. Canonical Coordination Model

```text
                       AUTHORITATIVE STORE
                              │
                    ┌─────────┴─────────┐
                    │                   │
               STATE VERSION        EPOCH
                    │                   │
                    └─────────┬─────────┘
                              ▼
                         SCHEDULER
                              │
                 ┌────────────┼────────────┐
                 ▼            ▼            ▼
               WORK         AGENT       RESOURCE
                 │            │            │
                 └────────────┼────────────┘
                              ▼
                         TRANSACTION
                              │
                    ┌─────────┴─────────┐
                    ▼                   ▼
                ASSIGNMENT          RESERVATION
                    │                   │
                    └─────────┬─────────┘
                              ▼
                            COMMIT
                              │
                              ▼
                       DISPATCH COMMAND
                              │
                              ▼
                            AGENT
                              │
                              ▼
                         EXECUTION
```

# 102. Failure-Safe Race Resolution

A race should resolve according to:

```text
authoritative state
        +
version
        +
epoch
        +
transaction ordering
```

not according to:

```text
which process happened to respond first
```

unless response order is itself the explicitly defined serialization mechanism.

# 103. Concurrency Principle

> **Every concurrent mutation must have an explicit authority, conflict rule, commit boundary, and observable outcome.**

# 104. Final Architectural Rule

> **NROS must make concurrency deterministic at the state-transition boundary even when the surrounding system is asynchronous, distributed, delayed, duplicated, or partially failed.**

This establishes the coordination foundation for:

```text
State
+
Persistence
+
Concurrency
+
Leases
+
Epochs
+
Scheduling
+
Execution
```

The next architectural layer is:

# Part CXVI — Agent Lifecycle, Capability Negotiation, Heartbeats, Liveness, Failure Detection & Reconciliation

The next question becomes:

> **How does NROS establish that an Agent is eligible, reachable, capable, authorized, and still the same Agent before assigning or continuing execution?**

# NROS — Part CXVI: Agent Lifecycle, Capability Negotiation, Heartbeats, Liveness, Failure Detection & Reconciliation

Agents are the execution boundary between NROS control-plane intent and real-world computation.

An Agent is therefore not simply:

```text
process = alive
```

It is an identity-bearing, capability-bearing, authority-scoped execution participant whose state can become uncertain independently of the controller.

The Agent model must answer:

```text
Who is this Agent?
Which incarnation is active?
What can it execute?
Is it authorized?
Is it healthy?
Is it reachable?
Can it safely receive Work?
Which executions does it own?
What happens when communication fails?
```

# 1. Agent Identity

Every Agent requires a stable identity:

```text
agent_id
```

The identity should survive ordinary process restarts where the same logical Agent is intended to remain the same participant.

# 2. Agent Incarnation

Every runtime instance should have an incarnation identity:

```text
agent_id = A7
incarnation = 42
```

A restart may therefore produce:

```text
A7 / incarnation 42
A7 / incarnation 43
```

This prevents stale telemetry and commands from one process instance being confused with another.

# 3. Identity vs Incarnation

These represent different concepts:

```text
agent_id
    = logical participant

incarnation
    = particular runtime lifetime
```

Both are required when restart ambiguity matters.

# 4. Agent Registration

An Agent should establish its presence through an explicit registration protocol.

Conceptually:

```text
Agent starts
    ↓
identity established
    ↓
credentials verified
    ↓
capabilities advertised
    ↓
protocol negotiated
    ↓
health established
    ↓
Agent becomes eligible
```

Registration alone should not imply execution eligibility.

# 5. Registration State

Possible states:

```text
UNKNOWN
REGISTERING
REGISTERED
VALIDATING
READY
DRAINING
QUARANTINED
REVOKED
OFFLINE
```

The exact state machine may differ, but transitions must be explicit.

# 6. Eligibility vs Presence

An Agent can be:

```text
REGISTERED
```

while not being:

```text
ELIGIBLE
```

For example:

```text
registered = yes
authorized = yes
health = degraded
capacity = zero
```

means the Agent exists but should not receive new Work.

# 7. Agent State Machine

A useful lifecycle is:

```text
DISCOVERED
    ↓
AUTHENTICATING
    ↓
REGISTERED
    ↓
CAPABILITY_NEGOTIATION
    ↓
HEALTH_CHECK
    ↓
READY
    ↓
DRAINING
    ↓
OFFLINE
```

Exceptional states:

```text
QUARANTINED
REVOKED
FAILED
```

# 8. Discovery

Discovery identifies a candidate Agent.

Discovery evidence should not automatically grant authority.

The sequence should remain:

```text
discovered
≠
authenticated
≠
authorized
≠
healthy
≠
eligible
```

# 9. Authentication

The control plane must establish that the Agent possesses valid credentials.

Authentication answers:

> **Who are you?**

It does not answer:

> **What are you allowed to do?**

# 10. Authorization

Authorization determines:

```text
which Work
which resources
which operations
which tenants
which capabilities
```

the Agent may execute.

# 11. Authentication Context

Agent identity should be associated with:

```text
credential identity
agent_id
incarnation
authority epoch
protocol session
```

where relevant.

# 12. Capability Advertisement

An Agent should advertise what it can actually perform.

Examples:

```text
cpu
memory
gpu
architecture
operating_system
runtime
container_support
filesystem_features
network_features
device_access
```

# 13. Capability Identity

Capabilities should use stable identifiers.

Example:

```text
linux.x86_64
container.exec
gpu.cuda
wasm.runtime
```

rather than arbitrary free-form strings wherever machine matching is required.

# 14. Capability Version

Capabilities may have versions:

```text
container.exec = v2
```

because:

```text
supports capability
```

does not imply:

```text
supports every version of capability
```

# 15. Capability Constraints

A capability can include constraints:

```text
gpu.cuda
memory >= 16 GiB
architecture = x86_64
```

The scheduler must evaluate these constraints explicitly.

# 16. Capability Freshness

Capabilities should have a freshness boundary.

An Agent that advertised:

```text
GPU = available
```

five hours ago may no longer have that resource.

Therefore:

```text
capability advertisement
```

and:

```text
current capacity
```

should not be treated as identical.

# 17. Static vs Dynamic Capabilities

Static:

```text
architecture
OS
installed runtime
```

Dynamic:

```text
available memory
free CPU
GPU availability
current load
```

The two classes require different update semantics.

# 18. Capability Negotiation

When the controller and Agent support different protocol versions, they should negotiate a mutually supported version.

Example:

```text
controller:
v1, v2, v3

Agent:
v2, v3

selected:
v3
```

# 19. Unsupported Capability

An Agent must not accept Work merely because it understands the outer protocol.

It must also satisfy the Work's execution requirements.

# 20. Capability Proof

For sensitive capabilities, NROS may require stronger evidence than self-reporting.

Examples:

```text
trusted runtime
attested environment
verified binary
approved hardware
```

The assurance level should be explicit.

# 21. Capability Trust

A capability record can include:

```text
source
observed_at
verified_at
verification_method
confidence
```

This distinguishes:

```text
self-declared
```

from:

```text
verified
```

capabilities.

# 22. Agent Readiness

Readiness should be derived from multiple conditions:

```text
identity valid
+
protocol compatible
+
authorized
+
health acceptable
+
required capabilities available
+
resource capacity available
+
not draining
+
not quarantined
```

# 23. Heartbeat

Heartbeats provide liveness information.

A heartbeat should identify:

```text
agent_id
incarnation
sequence
timestamp
state
capacity
protocol/session information
```

where needed.

# 24. Heartbeat Sequence

Each heartbeat should preferably carry a monotonically increasing sequence:

```text
1
2
3
4
```

This helps detect duplicates and stale messages.

# 25. Heartbeat ≠ Health

A heartbeat proves that an Agent can communicate.

It does not necessarily prove that:

```text
execution subsystem works
filesystem works
runtime works
required capability works
```

# 26. Health Probe

NROS may perform deeper probes:

```text
liveness
readiness
execution readiness
resource health
dependency health
```

# 27. Health Levels

A useful model is:

```text
ALIVE
RESPONSIVE
READY
EXECUTION_CAPABLE
HEALTHY
```

Each level represents stronger evidence.

# 28. Heartbeat Timeout

A controller can classify an Agent as suspect after missing expected heartbeats.

However:

```text
missing heartbeat
```

does not prove:

```text
Agent process terminated
```

It proves only that expected evidence was not received.

# 29. Failure Detector

The controller can maintain:

```text
HEALTHY
SUSPECT
UNREACHABLE
FAILED
```

but these states must have documented semantics.

# 30. Suspect State

A useful intermediate state is:

```text
SUSPECT
```

rather than immediately declaring:

```text
FAILED
```

after one missed heartbeat.

# 31. Failure Detection Is an Inference

Distributed failure detection is inherently imperfect.

Therefore:

> **"Agent failed" may be a control-plane conclusion, not direct physical knowledge.**

The distinction matters during reconciliation.

# 32. Failure Detection Policy

The detector can use:

```text
heartbeat interval
timeout
miss threshold
network health
recent command responses
external signals
```

# 33. Adaptive Detection

Different environments may require different thresholds.

For example:

```text
LAN Agent
```

may tolerate shorter detection windows than:

```text
remote Agent
```

# 34. Failure Detector Evidence

When marking an Agent unavailable, retain:

```text
last heartbeat sequence
last heartbeat timestamp
miss count
detection policy version
detection decision timestamp
```

# 35. Network Partition

If the controller cannot reach an Agent, the Agent may still be running.

This creates:

```text
controller view:
UNREACHABLE

Agent view:
RUNNING
```

This is not necessarily a contradiction.

It is a partition.

# 36. Partition Safety

During partition, NROS must prevent both sides from independently exercising conflicting authority.

This is where:

```text
epochs
leases
fencing
```

become critical.

# 37. Agent Authority Epoch

An Agent session should be associated with an authority epoch:

```text
Agent A7
epoch = 51
```

After controller recovery:

```text
epoch = 52
```

Old session authority becomes stale.

# 38. Session Identity

The Agent connection should have a session identity:

```text
session_id
```

so that old connections cannot be confused with current sessions.

# 39. Session Lifecycle

```text
SESSION_CREATED
    ↓
AUTHENTICATED
    ↓
NEGOTIATED
    ↓
ACTIVE
    ↓
STALE
    ↓
CLOSED
```

# 40. Stale Session

If an Agent reconnects:

```text
old session = S1
new session = S2
```

the controller must explicitly invalidate S1 where required.

# 41. Reconnect

A reconnect should not automatically restore previous execution authority.

The Agent must re-establish:

```text
identity
incarnation
session
epoch
capabilities
health
execution state
```

# 42. Re-registration

A restarted Agent may register as:

```text
same agent_id
new incarnation
```

The controller should treat the new incarnation as a distinct runtime participant.

# 43. Incarnation Collision

Two processes must not legitimately claim the same:

```text
agent_id + incarnation
```

If this occurs, NROS should detect an identity conflict.

# 44. Agent Quarantine

An Agent may be quarantined when:

```text
identity anomaly
protocol violation
security event
corrupt state
inconsistent execution evidence
```

is detected.

# 45. Quarantine Semantics

Quarantine should normally block:

```text
new Work
privileged operations
resource claims
```

while preserving enough access for:

```text
diagnostics
recovery
evidence collection
```

where safe.

# 46. Revocation

Revocation is stronger than temporary unavailability.

A revoked Agent should not regain authorization simply by reconnecting.

It requires explicit reauthorization.

# 47. Drain Mode

An Agent can enter:

```text
DRAINING
```

to stop receiving new Work while allowing existing executions to finish.

# 48. Drain Sequence

```text
READY
  ↓
DRAINING
  ↓
wait for active executions
  ↓
release resources
  ↓
OFFLINE
```

# 49. Forced Drain

If graceful draining exceeds a policy boundary:

```text
DRAINING
```

may transition to:

```text
FORCED_STOP
```

with explicit evidence.

# 50. Agent Capacity

Capacity should be represented separately from identity and health.

Example:

```text
cpu_total
cpu_available
memory_total
memory_available
gpu_total
gpu_available
concurrency_limit
```

# 51. Capacity Reservation

The scheduler should reserve capacity before dispatching Work where required.

Example:

```text
available CPU = 8
Work requires CPU = 4
reservation = 4
remaining = 4
```

# 52. Capacity Race

If two schedulers see:

```text
available = 4
```

and each attempt to reserve:

```text
4
```

only one should commit.

# 53. Capacity Version

Dynamic capacity should be versioned:

```text
capacity_version = 104
```

so stale scheduling decisions can be detected.

# 54. Resource Report

Agent resource reports should distinguish:

```text
physical capacity
allocatable capacity
reserved capacity
currently used capacity
```

# 55. Resource Accounting

A useful invariant is:

```text
used + reserved + available <= allocatable
```

subject to explicitly defined overcommit policies.

# 56. Overcommit

If overcommit is supported, it must be explicit.

For example:

```text
logical CPU capacity = 16
physical CPU capacity = 8
```

The scheduler must know which capacity model it is using.

# 57. Agent Execution Slots

Some Agents may expose:

```text
max_concurrent_executions
```

This should be enforced transactionally where exclusive.

# 58. Execution Admission

An Agent should not accept execution merely because the controller requested it.

It should validate:

```text
command identity
authority
capabilities
resource availability
execution state
```

# 59. Agent-Side Validation

Agent-side validation provides a second safety boundary.

It should not replace controller authorization, but it can prevent unsafe execution caused by stale or malformed commands.

# 60. Command Acceptance

A command can be accepted only if:

```text
identity valid
+
session valid
+
epoch valid
+
command not stale
+
capability satisfied
+
resource available
```

where applicable.

# 61. Command Rejection

Rejections should be structured:

```text
STALE_SESSION
STALE_EPOCH
UNKNOWN_COMMAND
CAPABILITY_MISMATCH
RESOURCE_UNAVAILABLE
INVALID_STATE
UNAUTHORIZED
```

# 62. Agent Acknowledgement

Acknowledgement should distinguish:

```text
RECEIVED
VALIDATED
ACCEPTED
STARTED
COMPLETED
```

# 63. Acknowledgement Semantics

A message:

```text
command received
```

must not be interpreted as:

```text
execution started
```

# 64. Execution Start Evidence

The Agent should provide evidence appropriate to the execution model:

```text
process ID
container ID
runtime task ID
device transaction ID
```

where meaningful.

# 65. Process Identity

A process ID alone may be insufficient because PIDs can be reused.

Prefer a stronger identity such as:

```text
process start time
execution ID
runtime instance ID
```

# 66. Completion Evidence

Completion should include:

```text
exit status
termination reason
execution duration
artifact references
resource release
```

where applicable.

# 67. Agent Crash

If the Agent crashes during execution:

```text
controller view:
execution = UNKNOWN
```

may be safer than immediately declaring:

```text
FAILED
```

until reconciliation establishes whether external work survived.

# 68. Agent Restart

After restart:

```text
new incarnation
```

must reconcile previous executions.

# 69. Execution Reconciliation

The controller asks:

```text
Which executions from the previous incarnation still exist?
Which completed?
Which failed?
Which are unknown?
```

# 70. Reconciliation Evidence

The Agent should identify:

```text
execution_id
attempt
previous authority epoch
current incarnation
runtime identity
state
evidence timestamp
```

# 71. Reconciliation Conflict

If controller state says:

```text
E42 = FAILED
```

while Agent reports:

```text
E42 = RUNNING
```

the system must produce an explicit conflict.

It must not silently overwrite one state with the other.

# 72. Conflict Resolution

Resolution may use:

```text
authority epoch
execution attempt
commit sequence
Agent incarnation
external evidence
policy
```

# 73. Orphaned Execution

An execution can become orphaned when:

```text
controller assignment exists
```

but:

```text
no valid Agent authority exists
```

The state should be explicit:

```text
ORPHANED
```

or:

```text
RECONCILIATION_REQUIRED
```

# 74. Orphan Recovery

The controller can then:

```text
reconcile
→ fence
→ retry
→ cancel
```

according to policy.

# 75. Agent Retirement

Retirement should be an explicit lifecycle operation.

Example:

```text
READY
 ↓
DRAINING
 ↓
OFFLINE
 ↓
RETIRED
```

# 76. Retirement Semantics

A retired Agent should no longer become eligible merely because it reconnects.

Reactivation requires explicit policy.

# 77. Agent Replacement

Replacing Agent A7 with A8 must not accidentally transfer execution identity.

Instead:

```text
execution E42
attempt 1 → A7
attempt 2 → A8
```

where a retry is actually created.

# 78. Agent Pool

Agents can be grouped into pools:

```text
general
gpu
high-memory
trusted
edge
isolated
```

Pool membership should be derived from authoritative configuration/capability state.

# 79. Pool Membership

An Agent can belong to multiple pools if policy permits:

```text
A7:
general
gpu
trusted
```

# 80. Pool Capacity

Scheduling against a pool should account for actual Agent-level reservations.

Pool capacity must not become an independent fiction.

# 81. Capability Matching

A Work requirement:

```text
gpu.cuda >= 12
memory >= 32 GiB
```

should be evaluated against current verified Agent capabilities.

# 82. Capability Mismatch

If a capability disappears after assignment:

```text
GPU removed
```

the scheduler must detect that the Agent is no longer suitable.

# 83. Dynamic Capability Loss

Dynamic loss may produce:

```text
DEGRADED
```

or:

```text
UNAVAILABLE
```

depending on impact.

# 84. Agent Health and Scheduling

Health should be part of eligibility:

```text
healthy
+
authorized
+
capable
+
capacity available
```

is stronger than:

```text
heartbeat received
```

alone.

# 85. Heartbeat Payload Size

Heartbeats should remain bounded.

Large diagnostics should use separate channels rather than embedding everything in the heartbeat.

# 86. Heartbeat Frequency

Frequency should balance:

```text
failure detection speed
network overhead
battery/CPU consumption
scale
```

# 87. Heartbeat Jitter

Agents may add controlled jitter to avoid synchronized heartbeat bursts:

```text
A1 → 10.1s
A2 → 9.8s
A3 → 10.3s
```

rather than all transmitting simultaneously.

# 88. Heartbeat Backpressure

If the control plane is overloaded, heartbeat processing must not consume all available capacity.

Priority handling may be required.

# 89. Liveness vs Availability

An Agent may be alive but unavailable for scheduling:

```text
alive = true
ready = false
```

This distinction should be visible.

# 90. Readiness Conditions

Readiness may require:

```text
protocol established
security valid
runtime loaded
required dependencies available
capacity valid
clock state acceptable
```

depending on the environment.

# 91. Clock Health

If execution deadlines depend on time, Agent clock health may become part of readiness.

Clock quality should not be assumed merely because the process is responsive.

# 92. Agent Metadata

Useful metadata includes:

```text
agent_id
incarnation
software_version
protocol_version
OS
architecture
runtime_version
capabilities
labels
health
capacity
```

# 93. Metadata Freshness

Metadata should carry:

```text
observed_at
```

so stale information is distinguishable from current information.

# 94. Software Version

The control plane should know which Agent software version is active where compatibility matters.

This enables:

```text
upgrade policy
feature gating
security enforcement
protocol compatibility
```

# 95. Rolling Upgrade

An Agent pool should support controlled replacement:

```text
A7 old
A8 new
A9 old
...
```

while preserving safety.

# 96. Upgrade Eligibility

New Agent versions should pass:

```text
authentication
protocol compatibility
health checks
capability checks
```

before receiving production Work.

# 97. Version Skew

The protocol should define which combinations are supported:

```text
controller v5
Agent v4 → supported
Agent v3 → unsupported
```

# 98. Capability Gating

New capabilities should not be used merely because an Agent reports them if the controller does not understand their semantics.

# 99. Agent Invariants

```text
1. Agent identity is stable within its intended logical lifetime.

2. Runtime incarnations are distinguishable.

3. Discovery does not imply authorization.

4. Authentication does not imply authorization.

5. Registration does not imply readiness.

6. Heartbeat does not imply health.

7. Liveness does not imply execution capability.

8. Dynamic capacity is freshness-sensitive.

9. Capability advertisements have explicit provenance.

10. Capability versions are explicit where required.

11. Sessions have unique identities.

12. Stale sessions can be invalidated.

13. Authority epochs fence obsolete control operations.

14. Quarantine blocks unsafe authority.

15. Revocation requires explicit reauthorization.

16. Draining prevents new assignments.

17. Resource reservations are authoritative.

18. Command acknowledgement levels are distinct.

19. Command identity is stable.

20. Stale commands are rejected.

21. Execution attempts are distinguishable.

22. Agent restart creates a new incarnation.

23. Restart triggers reconciliation.

24. Agent failure does not automatically prove workload failure.

25. Unknown remains valid after ambiguous execution loss.

26. Reconciliation conflicts are explicit.

27. Orphaned executions have explicit states.

28. Retired Agents cannot silently reactivate.

29. Pool membership is derived from authoritative Agent state.

30. Agent-side validation provides an independent safety boundary.
```

# 100. Canonical Agent Lifecycle

```text
                       DISCOVERY
                           │
                           ▼
                     AUTHENTICATION
                           │
                           ▼
                       REGISTERED
                           │
                           ▼
                 CAPABILITY NEGOTIATION
                           │
                           ▼
                      HEALTH CHECK
                           │
                    ┌──────┴──────┐
                    ▼             ▼
                 REJECT          READY
                                  │
                       ┌──────────┴──────────┐
                       ▼                     ▼
                    EXECUTE               DRAIN
                       │                     │
                       │                     ▼
                       │                  OFFLINE
                       │
                       ▼
                 RECONCILIATION
                       │
             ┌─────────┴─────────┐
             ▼                   ▼
          HEALTHY             CONFLICT
             │                   │
             ▼                   ▼
           READY              QUARANTINE
```

# 101. Canonical Failure Path

```text
READY
  ↓
heartbeat missing
  ↓
SUSPECT
  ↓
additional evidence
  ↓
UNREACHABLE
  ↓
authority fenced
  ↓
execution reconciliation
  ↓
┌───────────────┬───────────────┐
▼               ▼               ▼
RECOVERED     RETRY          UNKNOWN
```

The system should not jump directly from:

```text
heartbeat missing
```

to:

```text
execution failed
```

without evidence or policy justification.

# 102. Agent Readiness Function

Conceptually:

```text
eligible(agent) =
    authenticated
    ∧ authorized
    ∧ protocol_compatible
    ∧ capabilities_satisfied
    ∧ health_acceptable
    ∧ capacity_available
    ∧ session_current
    ∧ epoch_current
    ∧ ¬draining
    ∧ ¬quarantined
    ∧ ¬revoked
```

This should be treated as a logical model, not necessarily one literal function.

# 103. Failure Detection Principle

> **Failure detection is an evidence-based control decision, not omniscient knowledge of the remote process.**

# 104. Reconciliation Principle

> **After an Agent disappears and returns, NROS must reconcile identity, incarnation, authority, capabilities, resources, and executions before restoring eligibility.**

# 105. Final Architectural Rule

> **An Agent becomes schedulable only when NROS can establish sufficient evidence that the Agent is the correct runtime participant, is authorized under the current authority epoch, satisfies the required capabilities, possesses sufficient resources, and can safely accept new execution.**

This completes the Agent boundary:

```text
Identity
+
Incarnation
+
Authentication
+
Authorization
+
Capabilities
+
Health
+
Heartbeat
+
Liveness
+
Leases
+
Fencing
+
Reconciliation
```

The next layer is:

# Part CXVII — Work Model, Job/Task Semantics, Dependencies, DAG Execution, Priorities, Deadlines, Retries & Cancellation

The central question becomes:

> **What exactly is a unit of Work in NROS, how does it move through its lifecycle, and how are dependencies, retries, deadlines, priorities, and cancellation represented without ambiguity?**

# NROS — Part CXVII: Work Model, Job/Task Semantics, Dependencies, DAG Execution, Priorities, Deadlines, Retries & Cancellation

Work is the primary object that NROS schedules and executes.

A robust Work model must distinguish:

```text
intent
definition
instance
attempt
assignment
execution
result
```

Without these distinctions, retries, cancellation, dependency resolution, and recovery become ambiguous.

# 1. Work Hierarchy

A useful hierarchy is:

```text
Workflow
   ↓
Job
   ↓
Task
   ↓
Execution
   ↓
Attempt
```

These terms must have precise semantics.

# 2. Workflow

A Workflow represents a logical collection of related Work.

Example:

```text
Workflow W100
 ├── Task A
 ├── Task B
 ├── Task C
 └── Task D
```

The Workflow may define dependencies and shared policy.

# 3. Job

A Job is a logical unit submitted for execution.

It may contain:

```text
tasks
dependencies
policy
priority
deadline
retry configuration
resource requirements
```

# 4. Task

A Task represents a schedulable unit of Work.

Example:

```text
Task T42
command = build
resources = 4 CPU
```

# 5. Execution

An Execution represents a concrete controller-level execution instance of a Task.

Example:

```text
Task T42
    ↓
Execution E91
```

# 6. Attempt

An Attempt represents one concrete execution try.

Example:

```text
Execution E91
 ├── Attempt 1 → Agent A7
 ├── Attempt 2 → Agent A8
 └── Attempt 3 → Agent A9
```

This distinction is essential for retries.

# 7. Why Execution and Attempt Differ

An execution can represent the logical lifecycle:

```text
E91
```

while attempts represent individual runtime realizations.

This allows NROS to preserve:

```text
same Work identity
+
multiple execution attempts
```

without overwriting history.

# 8. Immutable Identity

Every Work object should have a stable identifier.

Example:

```text
work_id = W42
```

Identity must not change merely because:

```text
priority changes
retry occurs
Agent changes
configuration changes
```

# 9. Work Definition

A Work definition should contain enough information to determine what should execute.

Conceptually:

```text
WorkDefinition {
    work_id
    command
    inputs
    requirements
    policy
    priority
    deadlines
    retry_policy
    dependency_policy
}
```

# 10. Work Instance

The submitted Work instance binds a definition to an execution context:

```text
tenant
principal
submission
configuration
policy version
```

# 11. Work Immutability

Certain fields should become immutable after submission:

```text
work_id
principal
original command
original dependency identity
```

Mutable metadata should be versioned separately.

# 12. Mutable Work Attributes

Potentially mutable:

```text
priority
annotations
administrative state
cancellation state
resource reservation
```

Every mutation should have explicit version semantics.

# 13. Work State Machine

A baseline lifecycle:

```text
SUBMITTED
   ↓
VALIDATING
   ↓
QUEUED
   ↓
ELIGIBLE
   ↓
ASSIGNED
   ↓
DISPATCHED
   ↓
STARTING
   ↓
RUNNING
   ↓
┌──────────┬──────────┬──────────┐
▼          ▼          ▼
SUCCEEDED  FAILED   CANCELLED
```

Additional states may include:

```text
BLOCKED
WAITING_DEPENDENCY
RETRY_WAIT
UNKNOWN
EXPIRED
REJECTED
```

# 14. Submitted vs Accepted

Submission means:

> NROS received the request.

Acceptance means:

> NROS validated and committed the Work as a recognized object.

These should not be conflated.

# 15. Rejected Work

A Work request can be rejected before entering the scheduling system because of:

```text
invalid specification
authorization failure
unsupported capability
resource policy
quota
malformed dependency graph
```

# 16. Validation Boundary

Validation should occur before scheduling.

Example:

```text
submit
 ↓
schema validation
 ↓
authorization
 ↓
dependency validation
 ↓
resource validation
 ↓
commit Work
```

# 17. Dependency Model

Dependencies define when Work becomes eligible.

Example:

```text
A → B → C
```

means:

```text
B waits for A
C waits for B
```

# 18. DAG

A Workflow dependency graph should normally be a directed acyclic graph:

```text
A ──→ C
│
└──→ B ──→ D
```

# 19. Cycle Detection

A cycle such as:

```text
A → B → C → A
```

must be rejected unless NROS explicitly supports cyclic workflows.

# 20. Dependency Identity

Dependencies should reference stable Work identities, not mutable names.

# 21. Dependency States

A dependency can be:

```text
PENDING
SATISFIED
FAILED
CANCELLED
SKIPPED
UNKNOWN
```

# 22. Dependency Satisfaction

A Task should become eligible only when its dependency policy evaluates to true.

For example:

```text
all dependencies succeeded
```

is different from:

```text
all dependencies completed
```

# 23. Dependency Policy

Possible policies:

```text
ALL_SUCCEEDED
ALL_COMPLETED
ANY_SUCCEEDED
ANY_COMPLETED
EXPLICIT_CONDITION
```

# 24. Failed Dependency

If:

```text
A = FAILED
```

and:

```text
B requires A = SUCCEEDED
```

then:

```text
B = BLOCKED
```

rather than:

```text
B = READY
```

# 25. Dependency Propagation

Failure propagation should be explicit.

Possible result:

```text
A FAILED
   ↓
B BLOCKED
   ↓
C BLOCKED
```

# 26. Conditional Dependencies

A Workflow may support:

```text
if A succeeds → B
if A fails → C
```

This requires a formal condition model.

# 27. Condition Evaluation

Conditions should be deterministic and versioned.

Example:

```text
condition_version = 3
```

# 28. Dependency Evidence

The system should record why a dependency became satisfied:

```text
dependency_id
source_work
source_state
source_version
evaluation
policy_version
```

# 29. DAG Scheduling

A scheduler can identify ready nodes:

```text
Graph
 ↓
dependency evaluation
 ↓
READY set
 ↓
resource matching
 ↓
priority ordering
 ↓
assignment
```

# 30. Ready Set

The ready set should contain only Work whose mandatory dependencies are satisfied.

# 31. Priority

Priority determines ordering among eligible Work.

Example:

```text
P0 > P1 > P2 > P3
```

# 32. Priority Semantics

Priority should not bypass:

```text
authorization
resource constraints
dependency constraints
safety policy
```

# 33. Priority Inversion

A low-priority Work holding a scarce resource can block high-priority Work.

The scheduler may require:

```text
priority inheritance
preemption
resource partitioning
```

depending on policy.

# 34. Fairness

A scheduler that always selects the highest priority may starve lower-priority Work.

Therefore fairness should be explicit.

# 35. Aging

One strategy:

```text
effective_priority =
base_priority + waiting_time_factor
```

The formula must be deterministic and bounded.

# 36. Fair Queuing

Work can be divided into queues:

```text
tenant A
tenant B
tenant C
```

with explicit weights.

# 37. Tenant Fairness

If multi-tenancy exists, one tenant should not consume all shared scheduling capacity unless policy explicitly permits it.

# 38. Quotas

A tenant may have:

```text
CPU quota
memory quota
GPU quota
concurrency quota
Workflow quota
```

# 39. Quota Accounting

Quota usage must be tied to authoritative reservations, not merely observed process usage.

# 40. Deadline

Work may have:

```text
deadline
```

which represents the latest acceptable completion time.

# 41. Deadline Types

Distinguish:

```text
submission deadline
start deadline
completion deadline
lease deadline
execution timeout
```

These are different constraints.

# 42. Deadline Semantics

A deadline should specify:

```text
absolute timestamp
```

or:

```text
relative duration
```

and the clock domain used.

# 43. Start Deadline

If Work has:

```text
start_deadline = T
```

and has not started by T:

```text
EXPIRED
```

may become authoritative.

# 44. Completion Deadline

If Work is still running at its completion deadline, policy may require:

```text
cancel
continue
fail
escalate
```

# 45. Timeout vs Deadline

A timeout measures execution duration:

```text
runtime <= 30 min
```

A deadline measures wall-clock completion:

```text
finish by 18:00
```

They must not be conflated.

# 46. Deadline Race

If completion and timeout occur near the same boundary, the authoritative event ordering determines the result.

# 47. Clock Uncertainty

Distributed Agents may have different clocks.

Deadline enforcement should therefore use a well-defined clock source or conservative bounds.

# 48. Retry Policy

Retry configuration can include:

```text
max_attempts
retryable_failures
backoff
jitter
max_backoff
retry_deadline
```

# 49. Retryable vs Non-Retryable

Not every failure should trigger retry.

Examples:

```text
CAPABILITY_MISMATCH → maybe non-retryable on same pool
TRANSIENT_NETWORK → retryable
INVALID_INPUT → non-retryable
AGENT_CRASH → retryable
AUTHORIZATION_DENIED → non-retryable
```

The policy must be explicit.

# 50. Attempt Number

Every attempt receives:

```text
attempt_number
```

starting at:

```text
1
```

# 51. Retry Generation

A retry may also carry:

```text
retry_generation
```

to prevent concurrent controllers from creating duplicate retries.

# 52. Backoff

A retry can use:

```text
delay = min(base × multiplier^attempt, max_delay)
```

# 53. Jitter

To avoid synchronized retry storms, a bounded jitter can be applied.

The jitter strategy should remain observable.

# 54. Retry Storm

If thousands of failed Work items retry simultaneously:

```text
failure
 ↓
retry
 ↓
resource pressure
 ↓
more failure
 ↓
retry storm
```

NROS should apply global or tenant-level retry controls.

# 55. Retry Budget

A Workflow or tenant can have:

```text
retry_budget
```

limiting retry amplification.

# 56. Retry Preservation

Retry history should remain immutable:

```text
Attempt 1 = FAILED
Attempt 2 = FAILED
Attempt 3 = SUCCEEDED
```

Do not overwrite Attempt 1 with Attempt 3.

# 57. Cancellation

Cancellation should be modeled as a state transition, not merely a transport message.

# 58. Cancellation Request

```text
CANCEL_REQUESTED
```

means the system has committed the cancellation intent.

It does not necessarily mean execution has stopped.

# 59. Cancellation Delivery

The controller sends:

```text
cancel(command_id)
```

to the Agent.

# 60. Cancellation Acknowledgement

The Agent may report:

```text
CANCEL_RECEIVED
```

then:

```text
CANCELING
```

then:

```text
CANCELLED
```

# 61. Forced Cancellation

If graceful cancellation fails:

```text
CANCEL_TIMEOUT
```

may lead to:

```text
FORCED_TERMINATION
```

subject to authorization.

# 62. Cancellation Failure

If the Agent cannot stop the external operation, the result may become:

```text
UNKNOWN
```

rather than falsely claiming:

```text
CANCELLED
```

# 63. Cancellation Race

If:

```text
CANCEL
```

and:

```text
SUCCESS
```

race, the final state must be determined using explicit commit semantics.

# 64. Administrative Cancellation

An administrator may cancel Work regardless of the submitting principal's wishes if policy permits.

The authorization record should identify:

```text
actor
reason
policy
timestamp
```

# 65. User Cancellation

A user cancellation should identify:

```text
principal
request_id
work_id
```

and be authorized against the current Work state.

# 66. Force Flag

If forced cancellation exists, it should be explicit:

```text
force = true
```

rather than inferred from transport failure.

# 67. Preemption

Preemption means one Work is intentionally interrupted to allow another Work to run.

It differs from cancellation.

```text
CANCEL
    = terminate Work

PREEMPT
    = temporarily interrupt Work
```

# 68. Preemption Semantics

A preempted Work may become:

```text
PAUSED
```

or:

```text
PREEMPTED
```

with a later continuation or retry.

# 69. Checkpointing

Preemption is safer when the Work supports checkpointing:

```text
running
 ↓
checkpoint
 ↓
pause
 ↓
resume
```

# 70. Non-Checkpointable Work

If Work cannot safely resume, preemption may require:

```text
terminate
+
retry
```

with a new attempt.

# 71. Resource Preemption

Preemption must atomically update:

```text
execution state
resource reservation
Agent assignment
```

where required.

# 72. Priority + Preemption

A high-priority Work may preempt lower-priority Work only if policy explicitly permits it.

# 73. Work Resource Requirements

A Work definition may specify:

```text
cpu
memory
gpu
storage
network
devices
custom resources
```

# 74. Resource Expressions

Requirements may use:

```text
minimum
maximum
exact
range
count
affinity
anti-affinity
```

# 75. Capability + Resource Matching

Scheduling requires both:

```text
capability_match
```

and:

```text
resource_capacity
```

A capable Agent with insufficient capacity is not eligible.

# 76. Affinity

Work may prefer Agents with certain attributes:

```text
same region
same data locality
GPU family
specific device
```

# 77. Anti-Affinity

Work may require separation:

```text
Task A ≠ same host as Task B
```

for resilience or isolation.

# 78. Placement Constraints

Placement policies should be declarative and versioned.

# 79. Data Locality

If input data exists on Agent A, scheduling there may reduce transfer cost.

The scheduler can model:

```text
data locality score
```

without violating hard placement constraints.

# 80. Work Inputs

Inputs should be immutable references where possible:

```text
artifact_id
content_digest
version
```

rather than mutable filesystem paths.

# 81. Work Outputs

Outputs should similarly become immutable artifacts with stable identity.

# 82. Output Commit

Completion should distinguish:

```text
process exited
```

from:

```text
outputs durably committed
```

# 83. Partial Output

If execution fails after producing partial output:

```text
PARTIAL
```

should be represented explicitly where relevant.

# 84. Output Atomicity

Where possible:

```text
temporary output
    ↓
validate
    ↓
atomic publish
```

prevents consumers from observing incomplete artifacts.

# 85. Work Result

A result can include:

```text
status
exit_code
termination_reason
attempt
artifacts
metrics
logs
evidence
```

# 86. Result Provenance

Every result should be traceable to:

```text
work_id
execution_id
attempt
agent_id
incarnation
command_id
policy_version
```

where applicable.

# 87. Work Event Stream

Important lifecycle transitions should produce durable events:

```text
SUBMITTED
ACCEPTED
QUEUED
ASSIGNED
DISPATCHED
STARTED
RUNNING
COMPLETED
FAILED
CANCEL_REQUESTED
CANCELLED
RETRY_SCHEDULED
```

# 88. Event Ordering

Events for one Work should be ordered by authoritative transition sequence.

# 89. Event Idempotency

Consumers must be able to recognize duplicate lifecycle events.

# 90. Work Observability

A Work inspection API should expose:

```text
current_state
state_version
dependencies
assignment
execution
attempts
deadline
retry_status
resource_reservation
events
```

# 91. Explainability

The scheduler should answer:

> Why is this Work not running?

Possible answers:

```text
WAITING_DEPENDENCY
NO_CAPABLE_AGENT
NO_CAPACITY
QUOTA_EXCEEDED
POLICY_DENIED
DEADLINE_EXPIRED
RETRY_BACKOFF
PREEMPTED
```

# 92. Explainability Evidence

Each blocking reason should identify the relevant evidence and policy version.

# 93. Work Priority Mutation

Changing priority after submission should create a versioned mutation:

```text
priority v1 = 5
priority v2 = 10
```

The scheduler should know which version it consumed.

# 94. Deadline Mutation

Changing deadlines should likewise be versioned and authorized.

# 95. Dependency Mutation

Changing a dependency graph after execution has started can be dangerous.

Such mutations should normally be restricted or create a new Workflow version.

# 96. Workflow Version

A Workflow can use:

```text
workflow_id = W100
version = 4
```

so historical executions remain associated with the exact graph they used.

# 97. Immutable Workflow Graph

Once execution begins, the graph version should preferably be immutable.

Changes create:

```text
version 5
```

rather than mutating version 4.

# 98. DAG Versioning

Dependency evaluation should record:

```text
workflow_version
dependency_policy_version
```

# 99. Work Invariants

```text
1. Work identity is stable.

2. Definition, execution, and attempt are distinct.

3. Attempts are never silently overwritten.

4. Submission and acceptance are distinct.

5. Validation precedes scheduling.

6. Dependency graphs are acyclic unless cycles are explicitly supported.

7. Dependency satisfaction has explicit semantics.

8. Failed dependencies do not silently become successful dependencies.

9. Priority does not bypass hard safety constraints.

10. Fairness policy is explicit.

11. Quotas are authoritative.

12. Deadline types are distinct.

13. Timeouts and deadlines are not conflated.

14. Retryability is policy-driven.

15. Retry attempts are bounded.

16. Retry storms are controlled.

17. Cancellation request is distinct from cancellation completion.

18. Forced cancellation is explicit.

19. UNKNOWN remains valid when execution outcome cannot be established.

20. Preemption is distinct from cancellation.

21. Resource requirements are explicit.

22. Capability matching and capacity matching are both required.

23. Placement constraints are versioned.

24. Inputs are immutable references where possible.

25. Outputs are integrity-verified.

26. Partial output is explicit.

27. Results retain provenance.

28. Lifecycle events are ordered.

29. Lifecycle events are idempotently consumable.

30. Every blocked Work item has an explainable reason.
```

# 100. Canonical Work Lifecycle

```text
                SUBMIT
                   │
                   ▼
               VALIDATE
                   │
          ┌────────┴────────┐
          ▼                 ▼
       REJECT            ACCEPT
                              │
                              ▼
                           QUEUED
                              │
                              ▼
                     DEPENDENCY CHECK
                              │
                    ┌─────────┴─────────┐
                    ▼                   ▼
                 BLOCKED              READY
                                        │
                                        ▼
                                  RESOURCE MATCH
                                        │
                                        ▼
                                    ASSIGN
                                        │
                                        ▼
                                   DISPATCH
                                        │
                                        ▼
                                     START
                                        │
                                        ▼
                                    RUNNING
                                        │
                    ┌───────────────────┼───────────────────┐
                    ▼                   ▼                   ▼
                 SUCCESS              FAILURE            CANCEL
                    │                   │                   │
                    ▼                   ▼                   ▼
                COMPLETE             RETRY?              STOPPED
                                        │
                              ┌─────────┴─────────┐
                              ▼                   ▼
                           RETRY_WAIT            FAILED
                              │
                              ▼
                            READY
```

# 101. Canonical DAG Execution

```text
              A
             / \
            ▼   ▼
            B   C
             \ /
              ▼
              D
              │
              ▼
              E
```

Execution semantics:

```text
A succeeds
   ↓
B and C become eligible
   ↓
B + C succeed
   ↓
D becomes eligible
   ↓
D succeeds
   ↓
E becomes eligible
```

If:

```text
B fails
```

then D's eligibility depends on the declared dependency policy.

# 102. Scheduler Decision Function

Conceptually:

```text
eligible(work) =
    accepted
    ∧ dependencies_satisfied
    ∧ deadline_valid
    ∧ retry_policy_allows
    ∧ quota_allows
    ∧ placement_constraints_satisfied
```

Then:

```text
schedulable(work) =
    eligible(work)
    ∧ capable_agent_exists
    ∧ sufficient_capacity_exists
    ∧ authorization_valid
```

# 103. Scheduling Order

A conceptual scheduler pipeline:

```text
ALL WORK
   ↓
filter accepted
   ↓
filter dependency-ready
   ↓
filter deadline-valid
   ↓
filter quota-valid
   ↓
filter retry-eligible
   ↓
rank by policy
   ↓
match Agents
   ↓
reserve resources
   ↓
commit assignment
   ↓
dispatch
```

# 104. Work State Principle

> **A Work item's state must describe authoritative lifecycle state, not merely the latest observation received from an Agent.**

# 105. Retry Principle

> **A retry is a new execution attempt belonging to the same logical Work, never an overwrite of the previous attempt.**

# 106. Cancellation Principle

> **Cancellation intent, cancellation delivery, and confirmed termination are separate facts and must remain distinguishable whenever they can diverge.**

# 107. Dependency Principle

> **A dependent Work item becomes eligible only when its declared dependency policy evaluates against authoritative dependency state and returns true.**

# 108. Final Architectural Rule

> **NROS must represent Work as an immutable logical identity evolving through versioned state transitions, explicit dependency semantics, bounded retries, deterministic scheduling policy, and evidence-backed execution outcomes.**

This completes the Work boundary:

```text
Workflow
+
Job
+
Task
+
Execution
+
Attempt
+
Dependencies
+
Resources
+
Priority
+
Deadline
+
Retry
+
Cancellation
+
Result
```

The next layer is:

# Part CXVIII — Resource Model, Capacity Accounting, Reservations, Pools, Affinity, Placement, Quotas & Multi-Tenant Scheduling

The central question becomes:

> **How does NROS represent finite resources precisely enough to prevent overcommitment while still supporting dynamic capacity, heterogeneous Agents, quotas, affinity, locality, and fair multi-tenant scheduling?**

# NROS — Part CXVIII: Resource Model, Capacity Accounting, Reservations, Pools, Affinity, Placement, Quotas & Multi-Tenant Scheduling

Resources are the physical and logical constraints that determine whether Work can actually execute.

NROS must therefore distinguish:

```text
what exists
what is allocatable
what is reserved
what is currently consumed
what remains available
who is allowed to use it
```

A resource model that collapses these concepts will eventually produce incorrect scheduling decisions.

# 1. Resource Hierarchy

A useful abstraction is:

```text
Resource Domain
   ↓
Resource Pool
   ↓
Agent
   ↓
Resource
   ↓
Reservation
   ↓
Execution
```

# 2. Resource Identity

Every schedulable resource should have a stable identity.

Examples:

```text
cpu
memory
gpu
storage
network
device
custom.foo
```

For physical resources, identity may additionally include:

```text
agent_id
device_id
topology
```

# 3. Resource Types

NROS should distinguish resource semantics.

Common classes:

```text
CPU
memory
GPU
storage
network bandwidth
device
concurrency slot
custom resource
```

# 4. Scalar Resources

Some resources are naturally numeric:

```text
CPU = 8
memory = 32 GiB
bandwidth = 1 Gbps
```

These can use scalar accounting.

# 5. Integer Resources

Some resources should be treated as indivisible units:

```text
GPU = 2
license = 5
execution_slot = 8
```

# 6. Exclusive Resources

Some resources can be owned by only one execution at a time:

```text
serial_port
physical_device
exclusive_GPU
hardware_controller
```

Their allocation is effectively binary:

```text
FREE
→
RESERVED
→
RELEASED
```

# 7. Fractional Resources

Other resources can be divided:

```text
CPU = 0.5
GPU = 0.25
bandwidth = 100 Mbps
```

Fractional allocation requires explicit precision rules.

# 8. Resource Units

Every resource must have a canonical unit.

Examples:

```text
CPU → logical cores
memory → bytes
storage → bytes
bandwidth → bits/sec
```

The scheduler must never compare incompatible units.

# 9. Unit Normalization

Inputs such as:

```text
1 GiB
1024 MiB
1073741824 bytes
```

must normalize to the same canonical quantity.

# 10. Precision

Floating-point arithmetic should be avoided for critical resource accounting where exact integer units are possible.

Prefer:

```text
integer base units
fixed-point quantities
```

# 11. Capacity

A resource should expose:

```text
capacity
```

meaning the total amount that is allocatable under the current policy.

# 12. Physical Capacity

Physical capacity may be:

```text
8 CPU cores
32 GiB RAM
```

# 13. Allocatable Capacity

Allocatable capacity may differ:

```text
physical = 8 CPU
system_reserved = 1 CPU
allocatable = 7 CPU
```

# 14. Reserved Capacity

Reservations represent capacity committed to Work:

```text
allocatable = 7
reserved = 4
```

# 15. Actual Usage

Actual usage may differ from reservation:

```text
reserved = 4
used = 2.7
```

The scheduler should generally use reservation semantics for admission control rather than assuming observed usage equals available capacity.

# 16. Available Capacity

Conceptually:

```text
available =
    allocatable
    - committed_reservations
```

subject to the system's overcommit policy.

# 17. Overcommit

NROS may explicitly support overcommit.

Example:

```text
physical memory = 16 GiB
logical allocatable = 24 GiB
```

But overcommit must be represented as policy, not an accounting bug.

# 18. Overcommit Risk

If actual demand exceeds physical capacity:

```text
pressure
→ throttling
→ eviction
→ failure
```

may occur.

The scheduler must know these consequences.

# 19. Reservation as Authority

A reservation should be authoritative evidence that capacity has been committed.

A mere scheduling estimate is not a reservation.

# 20. Reservation Identity

Each reservation should have:

```text
reservation_id
work_id
execution_id
attempt
resource_id
amount
owner
state
version
```

where applicable.

# 21. Reservation Lifecycle

```text
REQUESTED
   ↓
VALIDATING
   ↓
RESERVED
   ↓
ACTIVE
   ↓
RELEASED
```

Exceptional states:

```text
EXPIRED
REVOKED
FAILED
```

# 22. Reservation Commit

A reservation should become authoritative through the same transactional model used elsewhere:

```text
check version
check capacity
authorize
reserve
commit
```

# 23. Reservation Race

Two requests:

```text
R1 → reserve 4 CPU
R2 → reserve 4 CPU
```

against:

```text
available = 4
```

cannot both succeed.

# 24. Reservation Ownership

Every reservation should identify its owner.

This prevents:

```text
stale worker releases current reservation
```

or:

```text
one execution modifies another execution's reservation
```

# 25. Reservation Fencing

Reservation mutations should validate:

```text
reservation_id
owner
epoch
version
```

where required.

# 26. Reservation Expiration

Temporary reservations may expire.

Expiration should be explicit:

```text
reservation
→
EXPIRED
```

rather than silently disappearing.

# 27. Lease vs Reservation

These concepts differ:

```text
Lease
    = temporary authority

Reservation
    = committed resource allocation
```

A reservation may itself be protected by a lease, but they should not be conflated.

# 28. Resource Pools

Agents can be grouped into resource pools:

```text
CPU pool
GPU pool
high-memory pool
trusted pool
edge pool
```

# 29. Pool Membership

Membership should derive from authoritative properties:

```text
capability
labels
topology
policy
health
```

# 30. Pool Capacity

Pool capacity should be computed from current eligible Agents.

If an Agent is:

```text
OFFLINE
```

its capacity should not remain schedulable merely because historical metadata says it has resources.

# 31. Dynamic Pool Membership

When an Agent changes:

```text
READY → DRAINING
```

it should cease receiving new Work even if it remains physically online.

# 32. Resource Labels

Resources can have labels:

```text
gpu.vendor = nvidia
gpu.memory = 24GiB
region = eu-west
storage.type = nvme
```

Labels support placement policies.

# 33. Hard Constraints

A hard constraint means:

> The Work cannot execute unless the condition is satisfied.

Example:

```text
gpu.vendor = nvidia
```

# 34. Soft Constraints

A soft constraint represents preference:

```text
prefer region = eu-west
```

The scheduler may violate it if necessary.

# 35. Constraint Priority

Placement policies can therefore be modeled:

```text
HARD
>
SOFT
```

with explicit ordering among soft preferences.

# 36. Affinity

Affinity means:

> Prefer or require co-location.

Examples:

```text
same Agent
same region
same rack
same data domain
```

# 37. Anti-Affinity

Anti-affinity means:

> Prefer or require separation.

Examples:

```text
different Agents
different zones
different failure domains
```

# 38. Failure Domains

A failure domain identifies infrastructure sharing a common failure source:

```text
host
rack
zone
region
power domain
```

# 39. High Availability Placement

For replicas:

```text
Replica A → Zone 1
Replica B → Zone 2
Replica C → Zone 3
```

may be required.

# 40. Replica Identity

Replicas should have stable identities:

```text
replica_set = R1
replica = 1
replica = 2
replica = 3
```

# 41. Resource Topology

Some resources are topology-sensitive.

Example:

```text
GPU 0
CPU NUMA node 0
memory NUMA node 0
```

A scheduler that ignores topology can allocate technically valid but practically inefficient placements.

# 42. NUMA-Aware Scheduling

Where supported, Work may require:

```text
CPU + memory
```

from the same NUMA domain.

# 43. Device Affinity

A task using:

```text
GPU 0
```

may require access to associated:

```text
PCI device
memory
driver
runtime
```

# 44. Device Exclusivity

Some devices cannot safely be shared.

The resource model must mark:

```text
sharing = EXCLUSIVE
```

rather than assuming every resource is fractional.

# 45. Shareable Resources

Other resources can be shared:

```text
CPU
network
read-only storage
```

subject to policy.

# 46. Resource Classes

A resource class can define default semantics:

```text
SCALAR
INTEGER
EXCLUSIVE
SHARED
COUNTED
CAPACITY
```

# 47. Custom Resources

NROS should support custom resource types without changing the scheduler core.

Example:

```text
fpga
camera
serial_port
license.foo
hardware_channel
```

# 48. Resource Schema

Custom resources should define:

```text
resource_type
unit
allocation_mode
sharing_mode
precision
constraints
```

# 49. Resource Discovery

Agents should report resources with provenance:

```text
source
observed_at
verified_at
capacity
allocatable
```

# 50. Resource Freshness

A stale capacity report must not indefinitely authorize new scheduling decisions.

# 51. Capacity Snapshot

The scheduler may operate on:

```text
resource_snapshot_id
```

containing the capacities used during a scheduling cycle.

# 52. Snapshot Versioning

Each snapshot should identify:

```text
snapshot_id
timestamp
source versions
```

# 53. Snapshot Staleness

A scheduler must define the maximum acceptable age of a capacity snapshot.

# 54. Scheduling Against Stale State

If a snapshot becomes stale before commit:

```text
revalidate
```

rather than blindly committing the old decision.

# 55. Resource Reconciliation

Actual resource state can diverge from controller reservations.

Example:

```text
controller:
GPU 0 reserved by E42

Agent:
GPU 0 actually free
```

The discrepancy must be reconciled explicitly.

# 56. Resource Drift

Drift can occur due to:

```text
Agent crash
manual intervention
external processes
hardware failure
controller failure
```

# 57. Drift Detection

NROS should periodically compare:

```text
authoritative reservations
```

against:

```text
observed resource state
```

# 58. Drift Resolution

Possible actions:

```text
reconcile
release stale reservation
quarantine Agent
rebuild reservation state
```

depending on policy.

# 59. Resource Accounting Invariant

For non-overcommitted scalar resources:

```text
reserved + system_reserved + available
=
allocatable
```

subject to rounding rules.

# 60. Exclusive Resource Invariant

For an exclusive resource:

```text
owners(resource) ≤ 1
```

# 61. Reservation/Execution Invariant

If execution requires a reservation:

```text
RUNNING execution
⇒
valid reservation exists
```

unless the resource policy explicitly permits reservationless execution.

# 62. Release Invariant

After authoritative completion:

```text
resources eventually become releasable
```

and cannot remain permanently charged to the completed execution without an explicit retention state.

# 63. Quotas

Quotas constrain resource consumption by administrative domain.

Possible dimensions:

```text
tenant
project
user
workflow
queue
```

# 64. Quota Hierarchy

A hierarchy may be:

```text
Organization
   ↓
Tenant
   ↓
Project
   ↓
User
```

# 65. Effective Quota

A child quota cannot necessarily exceed a parent quota.

Conceptually:

```text
effective_limit =
min(parent_limit, child_limit)
```

where hierarchy semantics require it.

# 66. Quota Usage

Quota usage should be based on authoritative reservations.

Example:

```text
tenant quota = 100 CPU
reserved = 80 CPU
available quota = 20 CPU
```

# 67. Quota Race

Two schedulers must not both believe:

```text
20 CPU remaining
```

and commit:

```text
20 + 20
```

This requires atomic quota accounting.

# 68. Quota Transaction

Scheduling may require atomic validation of:

```text
Agent capacity
+
tenant quota
+
project quota
+
resource reservation
```

# 69. Hierarchical Quota Transaction

For hierarchical quotas, all relevant levels must be validated consistently.

# 70. Quota Borrowing

Some systems permit one tenant to temporarily consume unused quota belonging to another pool.

If supported, borrowing must have:

```text
authorization
limits
expiration
accounting
```

# 71. Quota Reclamation

When Work completes:

```text
reservation released
→
quota usage decreases
```

The release must be idempotent.

# 72. Fair Scheduling

Fairness can be defined using:

```text
weighted fair share
dominant resource fairness
queue fairness
round-robin
aging
```

# 73. Dominant Resource Fairness

For multi-resource systems, a tenant's dominant resource can determine its effective share.

Example:

```text
Tenant A:
CPU 20%
Memory 60%

dominant share = 60%
```

This helps prevent one resource dimension from being ignored.

# 74. Fairness vs Priority

Priority and fairness are separate dimensions.

A high-priority tenant can still be subject to quota and fairness constraints.

# 75. Starvation Prevention

A scheduler should define whether low-priority Work can wait indefinitely.

If not, aging or fairness mechanisms are required.

# 76. Queue Model

Work may enter queues such as:

```text
critical
interactive
batch
background
```

Each queue can have policy.

# 77. Queue Admission

Admission should check:

```text
quota
authorization
resource feasibility
dependency state
deadline
```

before consuming queue capacity.

# 78. Queue Backpressure

When resources are saturated:

```text
new Work
→
queued
```

rather than causing uncontrolled retries or repeated scheduling attempts.

# 79. Scheduler Backpressure

The scheduler itself should avoid repeatedly attempting impossible assignments.

Example:

```text
NO_CAPABLE_AGENT
```

can pause scheduling until:

```text
capability/resource state changes
```

# 80. Resource Failure

If a resource becomes unhealthy:

```text
GPU failure
```

its allocatable capacity must immediately reflect the new state.

# 81. Partial Resource Failure

An Agent may retain:

```text
CPU
memory
```

while losing:

```text
GPU
```

Resource availability must be represented per resource.

# 82. Resource Health

Each resource may have:

```text
HEALTHY
DEGRADED
UNAVAILABLE
UNKNOWN
```

# 83. Resource Quarantine

A faulty resource can be removed from scheduling without taking the entire Agent offline.

# 84. Resource Recovery

After repair:

```text
UNAVAILABLE
→
VALIDATING
→
AVAILABLE
```

# 85. Resource Allocation and Agent Lifecycle

If an Agent enters:

```text
DRAINING
```

existing reservations may remain valid, but new reservations should normally be blocked.

# 86. Agent Failure

When an Agent becomes unreachable:

```text
new allocations = blocked
```

existing reservations become:

```text
RECONCILIATION_REQUIRED
```

until execution state is established.

# 87. Resource Reclamation After Failure

Resources should not be immediately reused if doing so could result in double ownership.

Safe reclamation may require:

```text
authority fencing
+
lease expiration
+
reconciliation
```

# 88. Resource Reuse Safety

The core rule is:

> Never reuse an exclusive resource while an old authority could still legitimately affect it.

# 89. Resource Lease

An exclusive resource reservation may have an associated lease:

```text
reservation
+
lease
+
fencing token
```

# 90. Fencing

When a reservation is reassigned:

```text
token 10
→
token 11
```

old holders carrying token 10 are rejected.

# 91. Resource Ownership

Ownership should be attributable:

```text
resource
→
reservation
→
execution
→
work
→
principal
```

# 92. Auditability

Every allocation mutation should produce an auditable record:

```text
who
what
resource
amount
previous_state
new_state
reason
policy_version
```

# 93. Resource Allocation Events

Useful events include:

```text
RESERVATION_CREATED
RESERVATION_COMMITTED
RESERVATION_ACTIVATED
RESERVATION_RELEASED
RESERVATION_EXPIRED
RESERVATION_REVOKED
RESOURCE_DEGRADED
RESOURCE_RECOVERED
```

# 94. Resource Metrics

NROS should expose:

```text
capacity
allocatable
reserved
used
available
fragmentation
reservation_conflicts
quota_rejections
placement_failures
```

# 95. Fragmentation

A pool may have sufficient total capacity but still be unable to satisfy a Work requirement.

Example:

```text
total free = 16 CPU
```

but:

```text
Work requires 16 CPU on one Agent
```

while every Agent has fewer than 16 free CPUs.

# 96. Fragmentation Awareness

The scheduler should distinguish:

```text
TOTAL_CAPACITY_INSUFFICIENT
```

from:

```text
PLACEMENT_FRAGMENTATION
```

# 97. Bin Packing

Resource placement may use:

```text
best fit
first fit
worst fit
dominant-resource strategies
```

The exact algorithm is implementation-specific.

# 98. Scheduler Determinism

Given the same:

```text
resource snapshot
Work set
policy
configuration
```

a deterministic scheduler should produce the same placement result.

# 99. Scheduling Decision Record

Each allocation decision should record:

```text
decision_id
work_id
resource_snapshot
candidate_agents
selected_agent
reservation_ids
policy_version
configuration_version
reason
```

# 100. Resource Invariants

```text
1. Every resource has a stable identity.

2. Every resource has a canonical unit.

3. Allocation semantics are explicit.

4. Physical capacity and allocatable capacity are distinct.

5. Reserved capacity is authoritative.

6. Observed usage is not automatically equivalent to reservation.

7. Overcommit is explicit.

8. Exclusive resources have at most one valid owner.

9. Reservations have stable identities.

10. Reservation ownership is explicit.

11. Reservation mutation is version-checked.

12. Reservation expiration is explicit.

13. Lease and reservation semantics remain distinct.

14. Resource pools derive from authoritative Agent/resource state.

15. Hard placement constraints cannot be violated for convenience.

16. Soft placement constraints are preferences, not guarantees.

17. Affinity and anti-affinity are explicit.

18. Failure domains are explicit where high availability requires them.

19. Topology-sensitive resources preserve topology information.

20. Dynamic resource state has freshness semantics.

21. Stale snapshots cannot silently authorize unsafe allocation.

22. Resource drift is detectable.

23. Resource reconciliation is explicit.

24. Quota accounting is atomic with reservation where required.

25. Hierarchical quotas have deterministic semantics.

26. Fairness does not bypass hard limits.

27. Priority does not bypass quota or safety constraints.

28. Resource failure can be represented independently of Agent failure.

29. Failed Agents do not immediately release exclusive resources without fencing/reconciliation.

30. Every allocation decision is auditable.
```

# 101. Canonical Resource Lifecycle

```text
              DISCOVER
                 │
                 ▼
              VERIFY
                 │
                 ▼
            ALLOCATABLE
                 │
                 ▼
             RESERVE
                 │
                 ▼
              ACTIVE
                 │
        ┌────────┼────────┐
        ▼        ▼        ▼
     RELEASE   EXPIRE   REVOKE
        │
        ▼
     AVAILABLE
```

# 102. Canonical Multi-Tenant Scheduling

```text
                    WORK
                      │
                      ▼
                 AUTHORIZATION
                      │
                      ▼
                    QUOTA
                      │
                      ▼
                DEPENDENCIES
                      │
                      ▼
                  PRIORITY
                      │
                      ▼
                   FAIRNESS
                      │
                      ▼
               AGENT FILTERING
                      │
              ┌───────┴────────┐
              ▼                ▼
          CAPABILITY         CAPACITY
              │                │
              └───────┬────────┘
                      ▼
                  PLACEMENT
                      │
                      ▼
                  RESERVATION
                      │
                      ▼
                    COMMIT
```

# 103. Resource Decision Function

Conceptually:

```text
allocatable(resource) =
    healthy
    ∧ current
    ∧ authorized
    ∧ capacity_remaining
    ∧ reservation_valid
```

For Work:

```text
placement_valid(work, agent) =
    capability_match
    ∧ hard_constraints
    ∧ anti_affinity
    ∧ quota
    ∧ capacity
    ∧ health
    ∧ authority
```

# 104. Scheduling Failure Taxonomy

NROS should distinguish:

```text
NO_CAPACITY
NO_CAPABLE_AGENT
QUOTA_EXCEEDED
HARD_CONSTRAINT_UNSATISFIABLE
AFFINITY_UNSATISFIED
ANTI_AFFINITY_UNSATISFIED
RESOURCE_UNHEALTHY
RESOURCE_STALE
RESERVATION_CONFLICT
AUTHORIZATION_DENIED
```

This makes scheduler behavior diagnosable.

# 105. Capacity Principle

> **A resource is schedulable only when its capacity, health, authority, ownership, and freshness are sufficiently established for the intended allocation.**

# 106. Reservation Principle

> **A scheduling decision becomes a resource commitment only when the corresponding reservation is atomically and authoritatively committed.**

# 107. Multi-Tenant Principle

> **Quota, fairness, and priority are independent policy dimensions; none may silently override hard authorization, dependency, placement, or resource-safety constraints.**

# 108. Final Architectural Rule

> **NROS must treat resources as versioned, attributable, policy-governed capacity rather than static numbers, allowing scheduling decisions to remain correct under concurrency, dynamic capacity changes, Agent failure, quota pressure, topology constraints, and multi-tenant contention.**

The next layer is:

# Part CXIX — Persistence, Event Sourcing, State Materialization, Transactions, Snapshots, Recovery, Compaction & Durable History

The central question becomes:

> **How does NROS persist authoritative state so that crashes, retries, failover, replay, reconciliation, and long-term auditability never require guessing what actually happened?**

# NROS — Part CXIX: Persistence, Event Sourcing, State Materialization, Transactions, Snapshots, Recovery, Compaction & Durable History

Persistence is the foundation that turns NROS from an in-memory scheduler into a recoverable distributed system.

The persistence layer must preserve the distinction between:

```text
intent
event
state
observation
reservation
execution evidence
```

A crash must not force NROS to reconstruct authority from memory, logs, or assumptions.

# 1. Persistence Goals

The persistence subsystem must provide:

```text
durability
consistency
ordering
recovery
idempotency
auditability
replayability
```

# 2. Authoritative State

NROS must define which persisted records are authoritative.

Examples:

```text
Work lifecycle state
Agent identity
Agent incarnation
resource reservations
leases
quota usage
workflow versions
execution attempts
```

# 3. Observation vs Authority

An observation:

```text
Agent says E42 is RUNNING
```

is not necessarily equivalent to authoritative state:

```text
controller says E42 = RUNNING
```

The persistence model must preserve that distinction.

# 4. Event

An event records that a state transition or durable fact occurred.

Example:

```text
WorkAccepted
```

# 5. State

Materialized state represents the current interpretation of events.

Example:

```text
work_id = W42
state = RUNNING
```

# 6. Event + State

A robust architecture can use:

```text
durable event log
        ↓
state projection
        ↓
materialized state
```

# 7. Event Sourcing

Under event sourcing, authoritative lifecycle transitions are represented as an ordered event history.

Example:

```text
Submitted
Accepted
Queued
Assigned
Started
Completed
```

# 8. Event Immutability

Once committed, an event should not be silently modified.

Corrections should be represented by new events.

# 9. Event Identity

Every event requires a stable identity:

```text
event_id
```

This enables deduplication.

# 10. Event Sequence

Events should have an authoritative sequence:

```text
1
2
3
4
```

The sequence establishes ordering within the relevant consistency domain.

# 11. Global vs Aggregate Sequence

NROS may use:

```text
global sequence
```

or:

```text
per-aggregate sequence
```

or both.

# 12. Aggregate

An aggregate is a consistency boundary.

Examples:

```text
Work
Agent
Reservation
Workflow
Tenant
```

# 13. Aggregate Version

Each aggregate should have a monotonically increasing version:

```text
Work W42
version 7
```

# 14. Optimistic Concurrency

A state mutation can require:

```text
expected_version = 7
```

and succeed only if the current version remains 7.

# 15. Lost Update Prevention

Without version checks:

```text
Scheduler A reads version 7
Scheduler B reads version 7

A writes change X
B writes change Y
```

B may accidentally overwrite A.

Version checking prevents this.

# 16. Transaction

A transaction groups mutations that must commit atomically.

Example:

```text
assign Work
+
create reservation
+
update quota
+
emit AssignmentCommitted
```

# 17. Atomic Scheduling Commit

A scheduling decision should ideally commit:

```text
Work assignment
resource reservation
quota accounting
scheduler decision
```

as one atomic state transition.

# 18. Transaction Boundary

The transaction boundary must be explicit.

A transaction should not include arbitrary external side effects such as:

```text
remote process execution
```

unless the system has a distributed transaction protocol, which NROS should generally avoid.

# 19. External Side Effects

A safer pattern is:

```text
transaction
  ↓
durable command
  ↓
commit
  ↓
dispatcher
  ↓
Agent
```

# 20. Transactional Outbox

The outbox pattern can connect durable state to asynchronous delivery.

```text
DB transaction
 ├── state mutation
 └── outbox command
          ↓
       dispatcher
          ↓
        Agent
```

# 21. Outbox Guarantees

If the transaction commits:

```text
the command becomes durably discoverable
```

even if the dispatcher crashes immediately afterward.

# 22. Outbox Delivery

The dispatcher may deliver the same command more than once.

Therefore commands must be idempotent or deduplicated.

# 23. Command Identity

Every command requires:

```text
command_id
```

The Agent can reject duplicate command IDs that it has already committed.

# 24. Inbox / Deduplication

The Agent may maintain:

```text
processed_command_id
```

to implement idempotent command handling.

# 25. Exactly-Once Illusion

Distributed systems should avoid claiming universal:

```text
exactly-once execution
```

unless it is formally guaranteed within a clearly defined boundary.

Prefer:

```text
at-least-once delivery
+
idempotent command semantics
```

where appropriate.

# 26. Event Ordering

Events belonging to the same aggregate should have deterministic ordering.

Example:

```text
WorkAccepted
version 1

Assigned
version 2

Started
version 3
```

# 27. Invalid Event Order

The system should reject impossible transitions such as:

```text
Completed
→
Started
```

unless an explicit state model permits it.

# 28. State Machine Validation

Every event should be validated against the current aggregate state.

Example:

```text
RUNNING
→
COMPLETED
```

valid.

```text
QUEUED
→
COMPLETED
```

may be invalid unless execution semantics permit direct completion.

# 29. Materialized Views

Frequently queried state should be materialized:

```text
work_current
agent_current
resource_current
reservation_current
tenant_usage
workflow_current
```

# 30. Projection

A projection consumes events:

```text
Event
 ↓
Projector
 ↓
Materialized state
```

# 31. Projection Determinism

Given the same event stream:

```text
same events
+
same projection version
```

the resulting materialized state should be deterministic.

# 32. Projection Version

Projection logic evolves.

Therefore:

```text
projection_version
```

should be tracked.

# 33. Projection Rebuild

If materialized state becomes corrupted:

```text
delete projection
 ↓
replay events
 ↓
rebuild
```

# 34. Snapshot

Long event histories may be expensive to replay.

A snapshot stores a known aggregate state at a known event version.

Example:

```text
Work W42
snapshot_version = 100
```

# 35. Snapshot + Replay

Recovery can use:

```text
snapshot @ 100
+
events 101..125
=
current state
```

# 36. Snapshot Integrity

A snapshot should contain:

```text
aggregate_id
aggregate_version
projection_version
created_at
state_digest
```

where appropriate.

# 37. Snapshot Validation

The system should verify that:

```text
snapshot version
```

matches the event history from which it claims to derive.

# 38. Snapshot Frequency

Snapshot frequency balances:

```text
write overhead
storage
recovery time
```

# 39. Snapshot Failure

A corrupt snapshot should not destroy the event history.

The system should fall back to an earlier trusted snapshot or replay from the beginning.

# 40. Event Log Retention

Not every event needs to remain forever in hot storage.

Retention policies can distinguish:

```text
hot history
cold archive
compliance archive
expired history
```

# 41. Compaction

Compaction reduces storage requirements while preserving the required recovery and audit semantics.

# 42. Safe Compaction

Compaction must never remove events required to reconstruct:

```text
authoritative state
audit history
legal/compliance evidence
```

unless policy explicitly permits it.

# 43. Event Tombstones

Deletion should often be represented by a tombstone:

```text
WorkDeleted
```

rather than physically erasing all evidence immediately.

# 44. Redaction

Sensitive data may require redaction.

Redaction should itself be represented as an auditable operation.

# 45. Immutable History vs Privacy

The persistence architecture must define how immutable event history interacts with:

```text
retention
privacy
deletion requests
secret rotation
```

# 46. Secret Handling

Secrets should not normally be stored directly in event payloads.

Prefer:

```text
secret reference
```

rather than:

```text
plaintext secret
```

# 47. Content Addressing

Large immutable payloads can be referenced by digest:

```text
sha256:<digest>
```

while the event stores the reference.

# 48. Event Payload Size

Events should remain bounded.

Large logs, artifacts, and outputs should live in dedicated artifact storage.

# 49. Artifact Reference

Example:

```text
artifact_id
content_digest
storage_class
```

# 50. Durable History

A complete Work history should permit reconstruction of:

```text
who submitted it
when it was accepted
which policy applied
which Agent was selected
which resources were reserved
which attempts occurred
what happened
why it failed
how it was retried
when it completed
```

# 51. Audit Trail

The audit trail should be separate conceptually from operational telemetry.

Operational metrics answer:

> What is happening?

Audit history answers:

> What authoritative decision occurred, by whom, under which policy?

# 52. Audit Event

An audit event may include:

```text
actor
action
target
timestamp
authorization_context
previous_state
new_state
reason
policy_version
```

# 53. Administrative Mutation

Examples:

```text
quota changed
Agent revoked
Work cancelled
priority changed
policy changed
```

should be auditable.

# 54. Transaction ID

Mutations spanning several aggregates may carry:

```text
transaction_id
```

to correlate related changes.

# 55. Correlation ID

A broader operational chain may use:

```text
correlation_id
```

to connect:

```text
API request
→ transaction
→ command
→ Agent execution
→ result
```

# 56. Causation ID

Each event may reference the event or command that caused it:

```text
causation_id
```

This creates an explicit causal chain.

# 57. Causal Metadata

A useful event envelope:

```text
event_id
aggregate_id
aggregate_type
aggregate_version
event_type
timestamp
actor
causation_id
correlation_id
payload
schema_version
```

# 58. Schema Version

Event schemas evolve.

Every persisted event should therefore carry:

```text
schema_version
```

# 59. Event Upcasting

Older events may need to be interpreted using newer schema semantics.

An upcaster can transform:

```text
v1 event
→
v2 representation
```

without mutating history.

# 60. Backward Compatibility

The persistence layer should define which historical event versions remain readable.

# 61. Migration

Database schema migration and event schema migration are different problems.

Both require explicit versioning.

# 62. State Migration

Materialized state may be migrated directly when safe.

Alternatively:

```text
rebuild from events
```

may be safer.

# 63. Crash During Transaction

If a process crashes during a transaction:

```text
partial transaction
```

must not become visible as authoritative state.

# 64. Crash After Commit

If the transaction committed before process failure:

```text
state must survive restart
```

even if the process never emitted an application-level confirmation.

# 65. Crash Before Commit

If the transaction never committed:

```text
no authoritative state transition
```

should be assumed.

# 66. Recovery

Recovery should proceed from durable state:

```text
open database
 ↓
validate storage
 ↓
recover committed transactions
 ↓
load snapshots
 ↓
replay events
 ↓
rebuild projections
 ↓
reconcile external Agents
 ↓
resume scheduling
```

# 67. Recovery Phase

NROS should have an explicit:

```text
RECOVERING
```

state.

It should not immediately claim:

```text
READY
```

before reconstruction completes.

# 68. Recovery Ordering

A safe sequence is:

```text
1. Persistence recovery
2. State validation
3. Projection recovery
4. Lease/epoch recovery
5. Agent reconciliation
6. Resource reconciliation
7. Work reconciliation
8. Scheduler activation
```

# 69. Scheduler During Recovery

New scheduling should generally be blocked until authoritative resource and execution state is sufficiently reconstructed.

# 70. Recovery Fence

A recovered controller may issue a new authority epoch:

```text
old epoch = 51
new epoch = 52
```

This fences stale controller sessions.

# 71. Leader Failover

If multiple controllers exist, a new leader must acquire authority before scheduling.

# 72. Leader Epoch

Leadership can be represented with:

```text
leader_epoch
```

so stale leaders cannot continue committing decisions.

# 73. Split-Brain Protection

Two controllers must not both believe they are authoritative.

This requires a consensus or equivalent exclusive-authority mechanism.

# 74. Consensus Boundary

The persistence layer should clearly identify where consensus is required.

Examples:

```text
leader election
global ordering
exclusive ownership
```

# 75. Local Transactions vs Distributed Consensus

A local database transaction provides atomicity inside one persistence authority.

It does not automatically solve:

```text
multi-controller consensus
```

# 76. Durable Scheduler State

The scheduler should persist enough state to recover:

```text
queued Work
reservations
assignments
retry timers
deadlines
fairness state
```

where the semantics require persistence.

# 77. Retry Timer Persistence

A retry scheduled for:

```text
12:00
```

must survive controller restart.

# 78. Deadline Persistence

Deadlines must remain authoritative across restart.

They must not reset because a scheduler process restarted.

# 79. Lease Persistence

Lease expiration state must be recoverable.

A restarted controller must not accidentally extend an expired lease simply because it lost in-memory timers.

# 80. Timer Reconstruction

Timers can be reconstructed from durable timestamps:

```text
next_retry_at
lease_expires_at
deadline
```

# 81. Event Replay

Replay should be deterministic.

Given:

```text
same event history
+
same projection version
```

the system should reach the same materialized state.

# 82. Replay Safety

Replay must not trigger external side effects such as:

```text
starting a process
sending a command
charging a tenant
```

unless explicitly operating through a safe side-effect-aware mechanism.

# 83. Projection vs Dispatcher

The projection layer reconstructs facts.

The dispatcher performs external effects.

They should remain separate.

# 84. Recovered Outbox

After restart:

```text
unprocessed outbox records
```

must be rediscovered and delivered.

# 85. Duplicate Delivery After Recovery

A command may have been delivered before the crash but not marked delivered.

Therefore:

```text
redelivery
```

must be safe.

# 86. Agent Idempotency After Controller Recovery

The Agent must use command identity and execution semantics to prevent accidental duplicate execution.

# 87. Persistence Integrity

Storage should provide integrity checks where appropriate:

```text
checksums
digests
transaction verification
sequence validation
```

# 88. Corruption Detection

If the event log contains:

```text
sequence 100
sequence 102
```

with no valid explanation, the system must detect the gap.

# 89. Event Gap

An event gap should produce:

```text
RECOVERY_ERROR
```

or equivalent rather than silently continuing with incomplete state.

# 90. Durable Sequence

Event sequences must be persisted with their ordering guarantees.

# 91. Clock and Event Ordering

Timestamps alone must not determine authoritative ordering.

Prefer:

```text
sequence
version
causal metadata
```

for ordering decisions.

# 92. Timestamp Semantics

Timestamps are useful for:

```text
observability
deadlines
retention
human interpretation
```

but should not be treated as perfect causal ordering.

# 93. Event Store Availability

If durable storage is unavailable:

```text
new authoritative state transitions
```

should generally stop.

The system should not continue pretending that unpersisted state is durable.

# 94. Read-Only Degradation

NROS may optionally remain available for read-only inspection during persistence degradation.

# 95. Write Degradation

If writes cannot be durably committed:

```text
scheduler = PAUSED
```

is safer than making ephemeral assignments.

# 96. Backpressure on Persistence

A slow database can create:

```text
event backlog
outbox backlog
scheduler latency
```

The system should expose these conditions explicitly.

# 97. Event Backlog

Metrics should include:

```text
uncommitted events
projection lag
outbox lag
replay position
```

# 98. Projection Lag

If:

```text
event stream = version 100
projection = version 97
```

queries against the projection may be stale.

The API should make consistency expectations explicit.

# 99. Strong vs Eventual Reads

NROS may expose:

```text
strongly consistent read
eventually consistent read
```

where useful.

# 100. Persistence Invariants

```text
1. Authoritative state is durable.

2. Events have stable identities.

3. Event ordering is explicit.

4. Aggregate versions are monotonic.

5. Concurrent mutations use version checks where required.

6. Atomic state changes use transactions.

7. External side effects are not assumed transactional with persistence.

8. Outbox records connect durable state to asynchronous commands.

9. Commands have stable identities.

10. Duplicate delivery is safe.

11. Materialized state is reconstructible.

12. Projection versions are explicit.

13. Snapshots identify their event version.

14. Corrupt snapshots do not destroy history.

15. Compaction preserves required recovery semantics.

16. Secrets are not unnecessarily embedded in durable events.

17. Large artifacts are referenced rather than embedded.

18. Historical events are schema-versioned.

19. Replay is deterministic.

20. Replay does not accidentally perform external side effects.

21. Recovery has an explicit lifecycle state.

22. Scheduler activation waits for required reconciliation.

23. Leadership has an explicit epoch.

24. Stale leaders cannot commit authoritative state.

25. Persistence failure does not silently produce false durability.

26. Event gaps are detected.

27. Projection lag is observable.

28. Retry timers survive restart.

29. Deadlines survive restart.

30. Lease state survives restart.
```

# 101. Canonical Persistence Pipeline

```text
                COMMAND
                   │
                   ▼
               VALIDATE
                   │
                   ▼
               TRANSACTION
              /          \
             ▼            ▼
       STATE CHANGE      OUTBOX
             │            │
             └──────┬─────┘
                    ▼
                  COMMIT
                    │
          ┌─────────┴─────────┐
          ▼                   ▼
      EVENT LOG           DISPATCHER
          │                   │
          ▼                   ▼
     PROJECTION             AGENT
          │                   │
          ▼                   ▼
   MATERIALIZED STATE      OBSERVATION
```

# 102. Canonical Recovery Pipeline

```text
            PROCESS START
                  │
                  ▼
          STORAGE RECOVERY
                  │
                  ▼
          EVENT INTEGRITY
                  │
                  ▼
             SNAPSHOT
                  │
                  ▼
             EVENT REPLAY
                  │
                  ▼
          PROJECTION READY
                  │
                  ▼
          LEASE/EPOCH RECOVERY
                  │
                  ▼
         AGENT RECONCILIATION
                  │
                  ▼
       RESOURCE RECONCILIATION
                  │
                  ▼
          WORK RECONCILIATION
                  │
                  ▼
          OUTBOX RESUMPTION
                  │
                  ▼
         SCHEDULER ACTIVATION
```

# 103. Transactional Scheduling Example

Conceptually:

```text
BEGIN

  verify Work version
  verify Agent version
  verify capacity
  verify quota

  create reservation
  create assignment
  increment quota usage
  append AssignmentCommitted
  append CommandQueued

COMMIT
```

After commit:

```text
dispatcher
    ↓
send command
    ↓
Agent
```

# 104. Recovery Example

Before crash:

```text
Work W42
Attempt 2
Agent A7
reservation R91
command C44
```

Controller crashes.

After restart:

```text
load durable state
→ discover R91
→ discover C44
→ inspect Agent A7
→ reconcile Attempt 2
→ determine whether C44 was executed
→ continue / retry / mark UNKNOWN
```

No step should depend on an in-memory assumption that disappeared during the crash.

# 105. Persistence Principle

> **Durability is part of correctness, not merely storage.**

# 106. Recovery Principle

> **Recovery reconstructs authority from durable evidence first and consults remote observations second.**

# 107. Event Principle

> **Events describe authoritative transitions; observations describe what another participant reports. The two must never be silently conflated.**

# 108. Transaction Principle

> **A state transition that requires multiple local invariants must commit atomically or remain uncommitted.**

# 109. Final Architectural Rule

> **NROS must maintain a durable, versioned, replayable history from which authoritative state can be reconstructed after crashes, failover, retries, and partial failures without relying on undocumented in-memory assumptions.**

This completes the persistence boundary:

```text
Events
+
Aggregates
+
Transactions
+
Outbox
+
Idempotency
+
Materialized State
+
Snapshots
+
Replay
+
Compaction
+
Recovery
+
Leadership Epochs
+
Durable Timers
+
Audit History
```

The next layer is:

# Part CXX — Protocol & Messaging Model, Commands, Events, Envelopes, Correlation, Delivery Semantics, Backpressure, Ordering, Idempotency & Wire Compatibility

The central question becomes:

> **How do NROS components communicate with explicit message semantics so that retries, duplicates, reordering, disconnections, version skew, and partial delivery cannot silently corrupt system state?**

# NROS — Part CXX: Protocol & Messaging Model, Commands, Events, Envelopes, Correlation, Delivery Semantics, Backpressure, Ordering, Idempotency & Wire Compatibility

The protocol layer defines how NROS components communicate across process and machine boundaries.

Its responsibility is not merely serialization.

It must define:

```text
identity
authority
ordering
delivery
acknowledgement
versioning
correlation
failure
retry
compatibility
```

The protocol therefore forms a contract between:

```text
Controller
Scheduler
Agent
Persistence
Dispatcher
CLI
API clients
Observability consumers
```

# 1. Protocol Boundary

NROS should distinguish:

```text
domain model
    ↓
protocol model
    ↓
transport
```

The domain determines meaning.

The protocol determines representation.

The transport determines delivery mechanics.

These layers should not be conflated.

# 2. Message

A Message is the fundamental protocol unit.

Conceptually:

```text
Message {
    envelope
    payload
}
```

# 3. Envelope

The envelope carries protocol metadata.

A conceptual envelope:

```text
Envelope {
    message_id
    message_type
    schema_version
    sender
    recipient
    timestamp
    correlation_id
    causation_id
    sequence
    delivery
}
```

# 4. Payload

The payload contains domain-specific content.

Examples:

```text
WorkSubmitted
AssignWork
StartExecution
CancelExecution
ExecutionStarted
ExecutionCompleted
ResourceChanged
AgentHeartbeat
```

# 5. Message Identity

Every message requires:

```text
message_id
```

It must be globally unique within its required scope.

# 6. Command Identity

Commands should additionally carry:

```text
command_id
```

because:

```text
message
```

and:

```text
requested operation
```

are conceptually different.

# 7. Event Identity

Events should carry:

```text
event_id
```

even if they are transported inside messages.

# 8. Why Multiple IDs?

Example:

```text
message_id = M900
command_id = C42
correlation_id = R7
causation_id = E91
```

These answer different questions:

```text
M900 → Which transport message?
C42  → Which requested operation?
R7   → Which request chain?
E91  → What caused this message?
```

# 9. Sender Identity

The sender should be explicit:

```text
sender_id
```

Examples:

```text
controller-1
agent-17
cli-client-4
```

# 10. Receiver Identity

A message can target:

```text
recipient_id
```

or a logical destination:

```text
queue
topic
resource pool
service
```

# 11. Component Incarnation

A process identity alone may be insufficient.

An Agent restart should produce a new incarnation:

```text
agent_id = A7
incarnation = 42
```

This prevents old messages from being confused with messages from a restarted Agent.

# 12. Epoch

Authority changes may require:

```text
epoch
```

Example:

```text
controller epoch 100
```

A message from epoch 99 can then be recognized as stale.

# 13. Fencing

Messages can carry authority information:

```text
authority_epoch
fencing_token
```

so stale controllers cannot continue issuing valid commands.

# 14. Message Timestamp

Messages may include:

```text
sent_at
```

for observability and timeout calculations.

But timestamps should not automatically determine causal ordering.

# 15. Logical Ordering

Ordering can use:

```text
sequence
aggregate_version
causation_id
```

depending on the consistency boundary.

# 16. Message Types

NROS should distinguish at minimum:

```text
COMMAND
EVENT
QUERY
RESPONSE
ACKNOWLEDGEMENT
ERROR
```

# 17. Commands

A Command requests an action.

Examples:

```text
AssignWork
StartExecution
CancelExecution
ReleaseReservation
DrainAgent
```

# 18. Events

An Event reports that an authoritative state transition occurred.

Examples:

```text
WorkAccepted
ExecutionStarted
ExecutionCompleted
ReservationReleased
AgentDraining
```

# 19. Query

A Query requests information without necessarily changing state.

Examples:

```text
GetWork
ListAgents
GetResourceState
GetWorkflow
```

# 20. Response

A Response carries the result of a Query or request/response interaction.

# 21. Acknowledgement

An acknowledgement should answer a precise question.

For example:

```text
RECEIVED
ACCEPTED
COMMITTED
EXECUTED
```

These must not be collapsed into one generic ACK.

# 22. Receipt vs Acceptance

```text
RECEIVED
```

means:

> The recipient obtained the message.

```text
ACCEPTED
```

means:

> The recipient validated and accepted the requested operation.

# 23. Commit Acknowledgement

```text
COMMITTED
```

means:

> The requested state transition is durably committed.

# 24. Execution Acknowledgement

```text
EXECUTED
```

means:

> The requested operation was actually executed.

These semantics are fundamentally different.

# 25. Command Lifecycle

A command may therefore progress:

```text
CREATED
  ↓
SENT
  ↓
RECEIVED
  ↓
ACCEPTED
  ↓
COMMITTED
  ↓
EXECUTED
```

Not every command requires every stage.

# 26. Command State

Command state should be persisted when required for recovery and idempotency.

# 27. Event Delivery

Events should normally be treated as:

```text
at-least-once
```

unless stronger guarantees are explicitly implemented.

# 28. Duplicate Events

Consumers must tolerate:

```text
Event E42
Event E42
```

without applying the transition twice.

# 29. Event Deduplication

Deduplication can use:

```text
event_id
aggregate_id + version
```

depending on protocol semantics.

# 30. Duplicate Commands

A command may also be delivered more than once.

Example:

```text
AssignWork C42
AssignWork C42
```

The recipient must not create two assignments.

# 31. Command Idempotency

A command is idempotent if applying it repeatedly produces the same authoritative result.

Example:

```text
ReleaseReservation(R42)
```

can safely become:

```text
already released
```

without producing a second release.

# 32. Non-Idempotent Commands

Some commands are inherently non-idempotent:

```text
increment counter
charge account
append arbitrary record
```

These require explicit operation identity and deduplication.

# 33. Idempotency Key

Clients may supply:

```text
idempotency_key
```

for operations that need deduplication.

# 34. Idempotency Scope

The system must define whether the key is unique:

```text
per user
per tenant
per API
per aggregate
globally
```

# 35. Idempotency Retention

Deduplication records must remain available long enough to cover possible redelivery.

# 36. Message Schema

A protocol schema should define:

```text
field name
type
required/optional
default
constraints
semantic meaning
version
```

# 37. Required Fields

Required fields should be minimal but sufficient for safe interpretation.

# 38. Optional Fields

Optional fields enable protocol evolution.

Unknown optional fields should generally be ignored when safe.

# 39. Unknown Required Fields

A message containing unsupported required semantics should be rejected rather than silently misinterpreted.

# 40. Schema Version

Every message type should have an explicit schema version.

Example:

```text
WorkAccepted.v2
```

or:

```text
message_type = WorkAccepted
schema_version = 2
```

# 41. Protocol Version

Schema version and protocol version are different.

```text
protocol_version
```

describes the broader wire contract.

```text
schema_version
```

describes a specific message representation.

# 42. Capability Negotiation

Peers can advertise:

```text
supported_protocol_versions
supported_message_types
supported_features
```

# 43. Feature Flags

New protocol behavior may be negotiated through:

```text
feature_id
```

rather than inferred from version numbers alone.

# 44. Compatibility Matrix

NROS should explicitly define:

```text
sender version
receiver version
message version
supported features
```

and the compatibility result.

# 45. Backward Compatibility

A newer Controller should ideally continue communicating with an older Agent when the required feature set overlaps.

# 46. Forward Compatibility

An older Agent should safely reject or ignore unsupported optional functionality rather than corrupting state.

# 47. Compatibility Failure

A protocol incompatibility should produce a structured error:

```text
PROTOCOL_VERSION_UNSUPPORTED
```

rather than a generic transport failure.

# 48. Error Envelope

Errors should be structured.

Example:

```text
Error {
    code
    message
    retryable
    details
    correlation_id
}
```

# 49. Error Codes

Error codes should be stable machine-readable identifiers.

Examples:

```text
INVALID_MESSAGE
UNAUTHORIZED
NOT_FOUND
CONFLICT
STALE_VERSION
UNSUPPORTED_FEATURE
RESOURCE_UNAVAILABLE
DEADLINE_EXCEEDED
```

# 50. Human Message

Human-readable error text should not be the sole machine contract.

Clients should depend on:

```text
error_code
```

not prose.

# 51. Retryability

Errors should explicitly indicate whether retry may be appropriate.

But:

```text
retryable = true
```

does not mean:

> retry immediately.

Backoff policy remains separate.

# 52. Permanent vs Transient Errors

Permanent:

```text
INVALID_INPUT
UNAUTHORIZED
UNSUPPORTED_CAPABILITY
```

Transient:

```text
TEMPORARY_UNAVAILABLE
CONNECTION_LOST
LEADER_CHANGED
```

# 53. Transport Failure

A transport failure does not prove that the command was not executed.

This is one of the most important protocol rules.

# 54. Ambiguous Outcome

Example:

```text
Controller
   ↓
StartExecution
   ↓
Agent
   ↓
execution starts
   X
connection fails
```

The Controller cannot conclude:

```text
START_FAILED
```

merely from connection loss.

The outcome may be:

```text
UNKNOWN
```

until reconciliation.

# 55. Reconciliation

After ambiguity:

```text
query Agent
```

or:

```text
observe durable execution state
```

to establish the result.

# 56. Message Delivery Semantics

NROS should explicitly document whether a channel provides:

```text
at-most-once
at-least-once
effectively-once
ordered
unordered
```

# 57. At-Most-Once

The message is delivered zero or one time.

Failure may mean loss.

# 58. At-Least-Once

The message is eventually delivered one or more times.

Duplicates are expected.

# 59. Effectively-Once

Exactly-once *effect* is achieved through:

```text
at-least-once delivery
+
idempotency
+
durable deduplication
```

rather than assuming the network itself guarantees exactly once.

# 60. Ordering Guarantees

Ordering may be:

```text
none
per connection
per sender
per aggregate
per partition
global
```

The protocol must state which one applies.

# 61. Aggregate Ordering

For Work W42:

```text
v1
v2
v3
```

should not be processed as:

```text
v1
v3
v2
```

without explicit buffering or conflict handling.

# 62. Sequence Gap

If a consumer receives:

```text
sequence 10
sequence 12
```

it should detect the missing sequence 11 where ordering guarantees require it.

# 63. Gap Handling

Possible responses:

```text
REQUEST_MISSING
RECONNECT
REPLAY
RESYNC
FAIL
```

# 64. Resynchronization

A peer can request:

```text
state snapshot
+
events after snapshot
```

to recover from missed messages.

# 65. Snapshot Protocol

Conceptually:

```text
RESYNC_REQUEST
      ↓
SNAPSHOT
      ↓
EVENTS_AFTER(snapshot_version)
      ↓
RESYNC_COMPLETE
```

# 66. Flow Control

A fast sender can overwhelm a slow receiver.

NROS therefore needs explicit backpressure.

# 67. Backpressure

The receiver can advertise:

```text
available_window
```

or equivalent flow-control information.

# 68. Message Window

A sender may be permitted:

```text
N unacknowledged messages
```

before it must wait.

# 69. Per-Channel Backpressure

Backpressure can be scoped to:

```text
connection
stream
tenant
Agent
message class
```

# 70. Priority Under Backpressure

Critical control messages should not necessarily wait behind an unlimited backlog of low-priority telemetry.

# 71. Control Plane vs Data Plane

NROS should distinguish:

```text
CONTROL
```

from:

```text
DATA
```

traffic.

# 72. Control Plane

Examples:

```text
AssignWork
CancelExecution
RenewLease
Heartbeat
Reconcile
```

# 73. Data Plane

Examples:

```text
logs
large artifacts
metrics streams
bulk output
```

Large data should not congest critical control messages.

# 74. Message Size

Protocol messages should have explicit maximum sizes.

# 75. Large Payloads

Large payloads should be externalized:

```text
message
   ↓
artifact reference
```

rather than embedded directly.

# 76. Compression

Compression may be negotiated per message class or transport.

Compression must not alter message semantics.

# 77. Fragmentation

If transport-level fragmentation exists, the application protocol should still preserve a single logical message identity.

# 78. Streaming

Long-running operations may use streaming:

```text
START
DATA*
END
```

# 79. Stream Identity

Every stream requires:

```text
stream_id
```

and should remain associated with its parent command/execution.

# 80. Stream Termination

A stream should terminate explicitly:

```text
COMPLETED
CANCELLED
FAILED
ABORTED
```

# 81. Half-Open Streams

Network failures can leave one side believing a stream remains active.

Keepalive and timeout semantics must therefore be explicit.

# 82. Heartbeats

Agents can periodically send:

```text
Heartbeat
```

containing enough information to establish liveness.

# 83. Heartbeat Is Not Execution Proof

A heartbeat proves:

```text
Agent is responsive
```

It does not necessarily prove:

```text
Execution E42 is healthy
```

Execution state requires separate evidence.

# 84. Heartbeat Payload

Possible fields:

```text
agent_id
incarnation
epoch
timestamp
capacity_summary
active_execution_ids
health
```

# 85. Heartbeat Frequency

Heartbeat frequency should balance:

```text
failure detection latency
network overhead
battery/CPU cost
```

# 86. Failure Detector

The Controller should classify Agent state using explicit policy:

```text
HEALTHY
SUSPECTED
UNREACHABLE
FAILED
```

# 87. Suspicion

Temporary packet loss should not immediately imply permanent Agent failure.

# 88. Lease Renewal

An Agent may renew its operational lease:

```text
renew(agent_id, incarnation, epoch)
```

Failure to renew may eventually invalidate its authority.

# 89. Protocol Authentication

Messages crossing trust boundaries require authentication.

Possible mechanisms depend on deployment.

# 90. Authorization

Authentication answers:

> Who sent this?

Authorization answers:

> Is this sender allowed to perform this action?

They are separate.

# 91. Message Integrity

Messages should provide integrity protection appropriate to the transport/security model.

# 92. Replay Protection

A captured command must not be safely replayed indefinitely.

Use:

```text
message_id
command_id
timestamp/expiry where appropriate
epoch
nonce
```

according to protocol design.

# 93. Authorization Context

A command may carry or resolve:

```text
principal
tenant
roles/claims
policy_version
```

# 94. Delegation

If Controller acts on behalf of a user:

```text
principal = user
actor = controller
```

should remain distinguishable.

# 95. Audit Causality

A command should be traceable:

```text
user request
→ API command
→ controller decision
→ Agent command
→ execution
→ result
```

# 96. Correlation

All related messages should carry a common:

```text
correlation_id
```

where appropriate.

# 97. Causation

A response/event should identify its cause:

```text
causation_id
```

when useful.

# 98. Distributed Trace Context

NROS can optionally carry tracing metadata separately from domain identity.

Tracing must not become the authoritative source of protocol semantics.

# 99. Transport Independence

The protocol model should not require one transport.

Possible transports:

```text
Unix socket
TCP
QUIC
HTTP
WebSocket
message broker
```

depending on deployment.

# 100. Transport Adapter

Conceptually:

```text
Protocol Message
      ↓
Transport Adapter
      ↓
Wire
```

# 101. Wire Contract

The wire format should define:

```text
framing
encoding
length
message type
version
payload
integrity
```

# 102. Framing

A stream transport needs explicit message boundaries.

Possible approaches:

```text
length-prefixed
record framing
self-delimiting serialization
```

# 103. Encoding

The encoding must be deterministic enough for interoperability requirements.

Possible formats depend on NROS goals:

```text
JSON
CBOR
MessagePack
Protobuf
custom binary
```

The choice should follow measured requirements rather than preference alone.

# 104. Canonical Serialization

Where signatures, hashes, or deterministic digests depend on serialized data, canonical encoding must be specified.

# 105. Schema Evolution

Fields should be added in ways that preserve older readers when possible.

Avoid reusing field identifiers for incompatible meanings.

# 106. Removed Fields

A removed field's identifier should not be casually reused for a different semantic meaning.

# 107. Enum Evolution

Adding enum values can break older exhaustive consumers.

Consumers should therefore handle unknown values safely.

# 108. Unknown Message Types

An endpoint should respond with:

```text
UNSUPPORTED_MESSAGE_TYPE
```

when required by the protocol rather than silently interpreting an unknown message.

# 109. Protocol Handshake

A connection may begin:

```text
HELLO
   ↓
CAPABILITIES
   ↓
NEGOTIATE
   ↓
READY
```

# 110. Handshake Contents

Potential fields:

```text
protocol_versions
schema_versions
features
compression
authentication
maximum_message_size
flow_control
```

# 111. Handshake Failure

A failed negotiation should terminate the incompatible session cleanly.

# 112. Connection State

A connection can use:

```text
CONNECTING
AUTHENTICATING
NEGOTIATING
READY
DRAINING
CLOSED
```

# 113. Drain

During shutdown:

```text
DRAINING
```

means:

> Stop accepting new work while allowing defined in-flight protocol operations to complete.

# 114. Graceful Shutdown

A peer should communicate:

```text
DRAIN
```

before closing when possible.

# 115. Abrupt Disconnect

A disconnect is not equivalent to:

```text
cancel all operations
```

Remote state must be reconciled.

# 116. Reconnect

After reconnect:

```text
HELLO
→ authenticate
→ negotiate
→ resynchronize
→ resume
```

# 117. Session Identity

A connection/session should have:

```text
session_id
```

so that stale session messages can be distinguished.

# 118. Session Incarnation

A reconnect creates:

```text
new session_id
```

even if:

```text
agent_id
```

remains unchanged.

# 119. Message Replay After Reconnect

The sender should know which messages remain unacknowledged and whether they need retransmission.

# 120. Durable Delivery State

For important commands, delivery state should be persisted:

```text
QUEUED
SENT
ACKNOWLEDGED
COMPLETED
```

# 121. Delivery Timeout

A delivery timeout means:

```text
no expected protocol acknowledgement received
```

It does not prove that the remote operation did not occur.

# 122. Retry After Timeout

Before retrying a non-idempotent command:

```text
query/reconcile
```

may be required.

# 123. Protocol-Level Cancellation

A command may itself be cancelled before delivery:

```text
CANCEL_COMMAND
```

This is distinct from cancelling the execution that the command would have created.

# 124. Dead-Letter Handling

Messages that cannot be delivered after policy-defined retries may enter:

```text
DEAD_LETTER
```

# 125. Dead-Letter Semantics

Dead-lettering must preserve:

```text
message_id
command_id
failure reason
attempt count
timestamps
```

so operators can investigate.

# 126. Poison Messages

Malformed or semantically invalid messages should not cause infinite retry loops.

They should be rejected and classified.

# 127. Message Validation Order

A receiver should conceptually perform:

```text
decode
 ↓
schema validation
 ↓
authentication
 ↓
authorization
 ↓
version compatibility
 ↓
idempotency check
 ↓
state validation
 ↓
execute/commit
 ↓
acknowledge
```

The exact order may vary where security architecture requires it.

# 128. Message Rejection

Every rejection should be attributable to a stable error code.

# 129. Protocol Metrics

NROS should expose:

```text
messages_sent
messages_received
duplicates
retries
acks
timeouts
protocol_errors
schema_errors
bytes_in
bytes_out
queue_depth
backpressure_events
```

# 130. Per-Message Latency

Useful timings include:

```text
send latency
receive latency
processing latency
commit latency
execution latency
```

These must not be confused.

# 131. Protocol Tracing

A trace should be able to connect:

```text
message
→ command
→ transaction
→ event
→ Agent execution
```

using correlation metadata.

# 132. Protocol Invariants

```text
1. Every message has a stable identity.

2. Commands have explicit command identity.

3. Events have explicit event identity.

4. Sender and receiver identities are explicit.

5. Agent incarnations distinguish restarts.

6. Authority epochs fence stale controllers.

7. Receipt is distinct from acceptance.

8. Acceptance is distinct from durable commit.

9. Commit is distinct from execution.

10. Delivery semantics are explicitly documented.

11. Duplicate delivery is expected where at-least-once transport is used.

12. Idempotency semantics are explicit.

13. Non-idempotent operations use durable deduplication or equivalent safeguards.

14. Transport failure does not imply operation failure.

15. Ambiguous outcomes remain UNKNOWN until reconciled.

16. Ordering guarantees are explicit.

17. Sequence gaps are detectable.

18. Resynchronization is supported where required.

19. Backpressure prevents unbounded receiver overload.

20. Control traffic can remain functional under data-plane pressure.

21. Large payloads are externalized.

22. Protocol and schema versions are distinct.

23. Unknown required semantics are rejected.

24. Compatibility is negotiated explicitly where needed.

25. Authentication and authorization are distinct.

26. Replay protection is explicit.

27. Session identity changes on reconnect.

28. Graceful drain is distinct from abrupt disconnect.

29. Dead-letter handling prevents infinite retry loops.

30. Protocol failures are observable and machine-readable.
```

# 133. Canonical Command Pipeline

```text
              CLIENT
                │
                ▼
             COMMAND
                │
                ▼
           AUTHENTICATE
                │
                ▼
           AUTHORIZE
                │
                ▼
          IDEMPOTENCY
                │
                ▼
          STATE VALIDATE
                │
                ▼
             COMMIT
                │
                ▼
              ACK
                │
                ▼
           DISPATCH
                │
                ▼
              AGENT
                │
                ▼
             RESULT
```

# 134. Canonical Event Pipeline

```text
             STATE CHANGE
                  │
                  ▼
              TRANSACTION
                  │
                  ▼
             EVENT COMMIT
                  │
                  ▼
              EVENT STORE
                  │
            ┌─────┴─────┐
            ▼           ▼
       PROJECTION    SUBSCRIBER
            │           │
            ▼           ▼
       QUERY STATE    EVENT HANDLER
```

# 135. Canonical Reconnect Pipeline

```text
DISCONNECT
    │
    ▼
RECONNECT
    │
    ▼
AUTHENTICATE
    │
    ▼
NEGOTIATE
    │
    ▼
NEW SESSION
    │
    ▼
RESYNC
    │
    ▼
REPLAY/ACK
    │
    ▼
READY
```

# 136. Canonical Message Envelope

```text
Envelope
├── message_id
├── message_type
├── protocol_version
├── schema_version
├── sender_id
├── sender_incarnation
├── recipient
├── authority_epoch
├── timestamp
├── correlation_id
├── causation_id
├── sequence
├── delivery_semantics
└── payload
```

# 137. Protocol Decision Rule

Before processing a message, NROS should be able to determine:

```text
Who sent it?
Which incarnation sent it?
Which authority epoch?
What operation/message is this?
Which version defines its semantics?
Is it authorized?
Is it a duplicate?
Is it ordered correctly?
Can it safely be applied?
```

If these questions cannot be answered where required, the message should not silently mutate authoritative state.

# 138. Protocol Principle

> **A transport connection is not an authority boundary. Protocol identity, authorization, ordering, and state semantics must remain explicit above the transport.**

# 139. Delivery Principle

> **NROS must assume that messages may be duplicated, delayed, reordered, or lost unless the specific transport contract proves otherwise.**

# 140. Failure Principle

> **Loss of communication is evidence of communication failure, not proof of operation failure.**

# 141. Compatibility Principle

> **Protocol evolution must preserve semantic compatibility explicitly; version numbers alone are not a substitute for capability negotiation and schema discipline.**

# 142. Final Architectural Rule

> **NROS must provide a versioned, identity-rich, idempotent and failure-aware protocol layer in which commands, events, acknowledgements, observations, and errors have distinct semantics and remain safe under retries, reconnects, duplicates, reordering, and partial delivery.**

This completes the protocol boundary:

```text
Message
+
Envelope
+
Command
+
Event
+
Acknowledgement
+
Correlation
+
Causation
+
Idempotency
+
Ordering
+
Backpressure
+
Handshake
+
Compatibility
+
Authentication
+
Authorization
+
Resynchronization
+
Failure Semantics
```

The next layer is:

# Part CXXI — Agent Model, Registration, Identity, Incarnation, Capabilities, Health, Heartbeats, Leases, Execution Control & Reconciliation

The central question becomes:

> **How does NROS establish that an Agent is real, authorized, capable, alive, current, and still entitled to execute Work—and how does the Controller safely recover when that Agent disappears or returns?**
