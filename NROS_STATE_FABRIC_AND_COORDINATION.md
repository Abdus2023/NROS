# NROS State Fabric & Coordination (Part LI–LX)

The previous layer established the autonomous execution loop:

```text
Observe
 → Understand
 → Intend
 → Plan
 → Authorize
 → Allocate
 → Execute
 → Verify
 → Update
```

But an autonomous system needs something else:

> **It must remember what happened, why it happened, what changed, and what evidence supports the resulting state.**

This is where NROS should go beyond the traditional ROS logging/bagging model.

# 1. Event Is the Primitive

NROS should treat an **event** as a first-class runtime object.

```text
Event
├── event_id
├── event_type
├── timestamp
├── producer
├── subject
├── payload
├── causality
├── correlation
└── provenance
```

Examples:

```text
AgentStarted
GoalCreated
PlanGenerated
ResourceAllocated
WorkStarted
WorkCompleted
StateChanged
PolicyDenied
SafetyTriggered
ObservationReceived
GoalAchieved
```

# 2. Message ≠ Event

A message means:

> "Here is some data."

An event means:

> "Something happened."

For example:

```text
Message:
    battery = 18%
```

versus:

```text
Event:
    BatteryThresholdCrossed
```

The second carries semantic significance.

NROS should support both.

# 3. Event Identity

Every event needs a globally meaningful identity:

```text
EventId
```

Example:

```text
evt_01J...
```

This permits references from:

```text
state
work
plans
evidence
memory
diagnostics
```

# 4. Event Ordering

Distributed systems cannot always rely on wall-clock timestamps.

NROS should therefore distinguish:

```text
event_time
ingest_time
commit_time
```

and, where needed:

```text
logical_sequence
```

This prevents ambiguity when events arrive late or out of order.

# 5. Causality

Events should be able to reference causes.

Example:

```text
ObstacleDetected
    ↓
PlanInvalidated
    ↓
ReplanRequested
    ↓
PlanGenerated
    ↓
WorkStarted
```

Represent this explicitly:

```text
caused_by
```

rather than reconstructing causality from timestamps.

# 6. Correlation

Causality and correlation are different.

Multiple events may belong to one mission:

```text
Mission M42
├── Goal G1
├── Plan P1
├── Work W1
├── Work W2
└── Evidence E1
```

All can carry:

```text
correlation_id = M42
```

This gives NROS an execution-wide trace.

# 7. Event Graph

The result is:

```text
        Goal
         │
         ▼
       Plan
         │
    ┌────┴────┐
    ▼         ▼
  Work A    Work B
    │         │
    ▼         ▼
Observation  Observation
    │         │
    └────┬────┘
         ▼
      State
```

This is a **causal runtime graph**.

# 8. Event Immutability

Once committed, an event should normally be immutable.

If something was wrong:

```text
Event A
```

should not silently change.

Instead:

```text
Event A
   ↓
Correction Event B
```

This preserves historical integrity.

# 9. Event Retraction

Some domains require explicit retraction.

Example:

```text
Observation:
    obstacle = present
```

later determined invalid.

NROS can record:

```text
ObservationRetracted
```

while preserving the original observation.

# 10. Event Journal

The runtime can maintain an append-oriented journal:

```text
EventJournal
────────────────────────
E100
E101
E102
E103
E104
...
```

This becomes the authoritative historical sequence for the relevant scope.

# 11. Journal Scope

A journal may exist at multiple levels:

```text
agent
robot
domain
fleet
deployment
system
```

This aligns naturally with NROS domains.

# 12. Memory

Events are history.

Memory is **usable retained information**.

Therefore:

```text
Event
   ↓
Processing
   ↓
Memory
```

Memory may retain:

```text
facts
episodes
procedures
experiences
summaries
relationships
```

# 13. Event → Memory Is Not Automatic

Not every event deserves long-term memory.

For example:

```text
1,000,000 sensor samples
```

should not necessarily become:

```text
1,000,000 permanent memories
```

NROS should support memory policies.

# 14. Memory Policy

Example:

```text
retain:
    safety events indefinitely

retain:
    navigation events for 30 days

retain:
    raw telemetry for 24 hours

retain:
    summaries indefinitely
```

Retention is therefore policy-controlled.

# 15. Memory Classes

A useful NROS model:

```text
WORKING
EPISODIC
SEMANTIC
PROCEDURAL
EVIDENCE
```

Each has different lifecycle semantics.

# 16. Working Memory

Working memory contains current context:

```text
current_goal
current_plan
active_work
recent_observations
current_constraints
```

It is highly dynamic.

# 17. Episodic Memory

An episode records something that happened.

Example:

```text
Episode:
    Mission M42

    started:
        10:00

    encountered:
        blocked corridor

    replanned:
        Route B

    completed:
        10:27
```

This is useful for future planning and diagnosis.

# 18. Semantic Memory

Semantic memory represents persistent knowledge:

```text
ChargingStation-3
    located_at:
        Zone-A

    connector:
        Type-2

    normal_power:
        7kW
```

It should not depend on one particular mission.

# 19. Procedural Memory

Procedural memory represents reusable behavior:

```text
Procedure:
    emergency_docking
```

with:

```text
preconditions
steps
constraints
verification
recovery
```

This can become reusable planning knowledge.

# 20. Memory Provenance

Every memory item should retain:

```text
source_events
created_by
created_at
confidence
validity
version
```

Thus:

```text
Memory M17
    ← E101
    ← E107
    ← E121
```

The system can explain where its knowledge originated.

# 21. Evidence

Evidence is stronger than ordinary memory.

Evidence answers:

> **What concrete artifact supports this claim?**

Examples:

```text
sensor capture
execution trace
state snapshot
measurement
command response
test result
human approval
simulation result
```

# 22. Evidence Object

Conceptually:

```text
Evidence
├── evidence_id
├── type
├── source
├── timestamp
├── subject
├── artifact
├── integrity
└── provenance
```

# 23. Evidence vs Log

A log might say:

```text
"motor started"
```

Evidence might contain:

```text
command issued
hardware acknowledgement
motor current
encoder response
state transition
timestamp
```

Evidence supports verification.

Logs primarily support observation and debugging.

# 24. Evidence Levels

NROS can define evidence strength:

```text
UNKNOWN
OBSERVED
REPORTED
CORRELATED
VERIFIED
ATTESTED
```

For example:

```text
"motor is running"
```

may initially be:

```text
REPORTED
```

then:

```text
VERIFIED
```

after encoder feedback confirms motion.

# 25. Evidence Chains

Evidence should be composable.

```text
Command
   ↓
Acknowledgement
   ↓
Sensor Observation
   ↓
State Transition
   ↓
Verification
```

Together these form:

```text
EvidenceChain
```

# 26. Claim

A particularly powerful abstraction is the **claim**.

Example:

```text
Claim:
    robot reached destination B
```

Supported by:

```text
localization evidence
navigation result
sensor observation
state snapshot
```

The claim can have:

```text
status = VERIFIED
```

# 27. Claim Structure

```text
Claim
├── claim_id
├── proposition
├── subject
├── status
├── confidence
├── evidence
├── derived_from
└── validity
```

This gives NROS an epistemic layer.

# 28. Verification Graph

We can therefore represent:

```text
Claim:
    Goal achieved
       │
       ├── Evidence E1
       │      └── localization
       │
       ├── Evidence E2
       │      └── action result
       │
       └── Evidence E3
              └── sensor observation
```

This is extremely useful for safety and auditing.

# 29. Evidence Integrity

Evidence should support integrity metadata:

```text
hash
signature
source identity
sequence
timestamp
```

For critical environments, evidence may be cryptographically authenticated.

# 30. Evidence Storage

NROS should not mandate one storage engine.

Possible backends:

```text
memory
filesystem
database
object storage
remote evidence store
distributed ledger
```

The protocol defines semantics.

The implementation chooses storage.

# 31. Hot / Warm / Cold Memory

A runtime may organize memory into tiers:

```text
HOT
    active working state

WARM
    recent episodes

COLD
    archival history
```

This allows constrained robots to operate with limited local storage.

# 32. Memory Compaction

Millions of events can be summarized.

For example:

```text
10,000 navigation events
        ↓
Episode summarization
        ↓
Mission history
```

The original events can remain archived while the active memory stores a compact representation.

# 33. Summaries Must Preserve Provenance

A summary:

```text
"Mission completed successfully"
```

should reference:

```text
E100 ... E842
```

or an equivalent evidence range.

Otherwise summarization destroys traceability.

# 34. Replay

The event journal enables replay:

```text
Event E1
 ↓
Event E2
 ↓
Event E3
 ↓
...
```

reconstructing:

```text
State S0
 ↓
S1
 ↓
S2
 ↓
S3
```

This is one of the strongest capabilities NROS can provide.

# 35. Deterministic Replay

For deterministic components:

```text
same initial state
+
same events
+
same configuration
=
same result
```

This should be an explicit NROS design objective.

Not every external process can guarantee it, but the runtime should make deterministic replay possible wherever feasible.

# 36. Replay Modes

NROS can support:

```text
REALTIME
FAST
STEP
PAUSED
TIME_TRAVEL
```

For example:

```text
replay until event E742
```

then inspect state.

# 37. Simulation From Reality

A particularly powerful capability:

```text
Real Mission
     ↓
Recorded Event/Evidence Stream
     ↓
Simulation
     ↓
Alternative Plan
```

The system can ask:

> What would have happened if Plan B had been executed?

This connects operational history with simulation.

# 38. Counterfactual Execution

Conceptually:

```text
Actual:
    Plan A → outcome X

Counterfactual:
    Plan B → simulated outcome Y
```

The distinction must remain explicit:

```text
OBSERVED
vs
SIMULATED
```

A simulation must never masquerade as physical evidence.

# 39. Evidence Classification

Every artifact should identify its epistemic source:

```text
OBSERVED
SIMULATED
INFERRED
REPORTED
PREDICTED
REPLAYED
```

This is critical for autonomous reasoning.

# 40. Memory Retrieval

An agent should be able to ask:

```text
retrieve memories relevant to goal G
```

rather than loading the entire historical database.

Queries can consider:

```text
semantic similarity
time
entity
mission
location
confidence
causal relationship
```

# 41. Memory Relevance

A memory can carry:

```text
relevance
```

but relevance should not be treated as truth.

For example:

```text
Memory A:
    highly relevant
    low confidence
```

is still different from:

```text
Memory B:
    moderately relevant
    verified
```

# 42. Memory Conflicts

Suppose memory contains:

```text
Door A:
    requires badge
```

and newer evidence shows:

```text
Door A:
    no badge required
```

NROS should preserve both histories while marking:

```text
older fact = superseded
new fact = current
```

# 43. Temporal Knowledge

Facts should support temporal qualification:

```text
valid_from
valid_until
observed_at
```

Therefore:

```text
Door requires badge
```

may have been true:

```text
2025–2026
```

but no longer true.

# 44. Memory Ownership

Memory should also have authority.

For example:

```text
FleetRegistry
```

may be authoritative for:

```text
robot identity
```

while:

```text
Localization
```

is authoritative for:

```text
robot pose
```

This avoids uncontrolled semantic conflicts.

# 45. Memory Access Control

Not all memory should be globally visible.

A robot may have:

```text
private memory
```

while the fleet has:

```text
shared memory
```

Access can be controlled through:

```text
capability
authority
scope
policy
```

# 46. Memory Replication

A robot can synchronize selected memory:

```text
Robot A
   ↓
Fleet Memory
   ↓
Robot B
```

But synchronization should preserve:

```text
origin
timestamp
version
authority
confidence
```

# 47. Conflict-Free Synchronization

For distributed robots, some state can be merged using CRDT-like semantics or domain-specific reconciliation.

But NROS should not assume that every robotics state is mergeable.

For safety-critical state:

```text
explicit authority
```

is often preferable.

# 48. Event Delivery Guarantees

NROS should define delivery semantics explicitly.

Potential levels:

```text
BEST_EFFORT
AT_LEAST_ONCE
DURABLE
EXACTLY_ONCE_SEMANTIC
```

The last should be treated carefully: network systems often cannot provide literal exactly-once delivery cheaply.

Instead, NROS can achieve:

```text
at-least-once transport
+
idempotent event identity
+
deduplication
```

to provide effectively-once processing.

# 49. Idempotency

If:

```text
Event E42
```

arrives twice:

```text
E42
E42
```

the consumer should be able to recognize the duplicate.

Therefore:

```text
event_id
```

is not optional infrastructure metadata.

It is part of the semantic model.

# 50. Checkpoint

Agents should periodically create:

```text
Checkpoint
```

containing:

```text
state
goal
plan
active work
memory references
resource allocations
```

A checkpoint provides a recovery boundary.

# 51. Recovery

After process or machine failure:

```text
Checkpoint
    ↓
restore state
    ↓
validate resources
    ↓
validate assumptions
    ↓
resume / replan / abort
```

This is much stronger than simply restarting a ROS node.

# 52. Crash Recovery

Suppose:

```text
Work W193 = RUNNING
```

and the agent crashes.

On restart:

```text
runtime discovers W193
```

It must determine:

```text
Did W193 actually complete?
Did it partially execute?
Is the resource still allocated?
Is the external world consistent?
```

Evidence and event history provide the answer.

# 53. Exactly This Is Why History Matters

Without evidence:

```text
restart
 ↓
guess
```

With NROS history:

```text
restart
 ↓
recover checkpoint
 ↓
inspect events
 ↓
inspect evidence
 ↓
reconcile state
 ↓
resume safely
```

# 54. Memory → Learning

Historical experience can eventually feed learning systems:

```text
Episodes
   ↓
Pattern extraction
   ↓
Policy improvement
   ↓
New planner behavior
```

But NROS should maintain a strict distinction:

```text
runtime truth
vs
learned prediction
```

A learned model cannot silently overwrite authoritative state.

# 55. Prediction

Predictions should be first-class:

```text
Prediction
├── proposition
├── predicted_at
├── expected_time
├── model
├── confidence
└── evidence
```

Example:

```text
battery will reach 10% in 4 minutes
```

This is not current state.

It is a forecast.

# 56. Prediction → Planning

The planner can then use:

```text
Current State
+
Predictions
+
Goals
```

to make decisions.

For example:

```text
battery:
    15%

prediction:
    8% in 5 minutes

goal:
    complete mission in 12 minutes
```

This may trigger:

```text
recharge intent
```

before the battery becomes critically low.

# 57. The Complete Knowledge Lifecycle

We now have:

```text
Observation
    ↓
Claim / Interpretation
    ↓
State
    ↓
Knowledge
    ↓
Intent
    ↓
Goal
    ↓
Plan
    ↓
Work
    ↓
Execution
    ↓
Evidence
    ↓
Event
    ↓
Memory
    ↓
Future Planning
```

This is a closed autonomous knowledge loop.

# 58. NROS vs ROS Bags

The conceptual difference is important.

ROS bagging primarily answers:

> What messages were recorded?

NROS Evidence/Memory should answer:

> What happened, what state changed, what decision was made, why was it made, what work executed, and what evidence verifies the outcome?

Therefore:

```text
ROS:
    message history

NROS:
    semantic execution history
```

# 59. NROS Semantic Ledger

The combined event/evidence system can be thought of as a:

# **Semantic Runtime Ledger**

Not a financial ledger.

A ledger of:

```text
facts
events
decisions
authority
actions
state transitions
evidence
```

with immutable history and explicit corrections.

# 60. The NROS Causal Ledger

At the highest level:

```text
WHY
 ↓
Intent

WHAT
 ↓
Goal

HOW
 ↓
Plan

WITH WHAT
 ↓
Resources

WHO
 ↓
Authority

WHAT HAPPENED
 ↓
Events

WHAT CHANGED
 ↓
State

HOW DO WE KNOW
 ↓
Evidence

WHAT DO WE REMEMBER
 ↓
Memory
```

This gives NROS a coherent semantic architecture.

# 61. Thirteen-Fabric Architecture

The architecture now expands to:

```text
┌─────────────────────────────────────────────┐
│ Domain & Deployment                         │
├─────────────────────────────────────────────┤
│ Resource & Allocation                       │
├─────────────────────────────────────────────┤
│ Intent & Planning                           │
├─────────────────────────────────────────────┤
│ Knowledge & State                           │
├─────────────────────────────────────────────┤
│ Memory, Event & Evidence                    │
├─────────────────────────────────────────────┤
│ Protocol & Type                             │
├─────────────────────────────────────────────┤
│ Observability                               │
├─────────────────────────────────────────────┤
│ Capability & Authority                      │
├─────────────────────────────────────────────┤
│ Supervision                                 │
├─────────────────────────────────────────────┤
│ Temporal                                    │
├─────────────────────────────────────────────┤
│ Execution                                   │
├─────────────────────────────────────────────┤
│ Communication                               │
└─────────────────────────────────────────────┘
```

# 62. The Next Architectural Boundary

At this point, NROS has:

```text
communication
execution
time
supervision
authority
capabilities
resources
knowledge
state
intent
planning
memory
events
evidence
```

But these capabilities operate across a **distributed system**.

That introduces the next fundamental question:

> **How does NROS establish trust across agents, machines, robots, domains, and external services?**

This leads to:

# Part LII — NROS Identity, Trust & Security Fabric

The progression becomes:

```text
Identity
   ↓
Authentication
   ↓
Capability
   ↓
Authorization
   ↓
Trust
   ↓
Secure Communication
   ↓
Attested Execution
   ↓
Evidence
```

The crucial principle will be:

> **An NROS entity should never be trusted merely because it can communicate.**

Communication establishes reachability.

**Identity establishes who.**

**Authorization establishes what.**

**Attestation establishes what actually ran.**

**Evidence establishes what actually happened.**

# NROS — Part LII: Identity, Trust & Security Fabric

The Memory/Event/Evidence Fabric established **historical accountability**.

The next question is:

> **Who is allowed to create, observe, modify, execute, authorize, or trust something inside NROS?**

Traditional ROS deployments often treat connectivity and identity relatively lightly. NROS should instead make **identity and authority explicit runtime primitives**.

# 1. Security Is Not a Single Layer

NROS should separate:

```text
Identity
   ↓
Authentication
   ↓
Authorization
   ↓
Capability
   ↓
Trust
   ↓
Attestation
   ↓
Evidence
```

These answer different questions.

### Identity

> Who are you?

### Authentication

> Can you prove it?

### Authorization

> Are you allowed to do this?

### Capability

> What authority has explicitly been granted?

### Trust

> How much should I rely on this entity?

### Attestation

> What actually executed?

### Evidence

> What proves what happened?

# 2. NROS Principal

The fundamental security identity should be a:

```text
Principal
```

A principal may represent:

```text
Human
Agent
Robot
Process
Device
Service
Controller
Deployment
Organization
```

Each gets a stable identity.

# 3. Principal Identity

Conceptually:

```text
PrincipalId
├── namespace
├── identifier
└── identity-version
```

Examples:

```text
robot/01
agent/navigation/01
controller/motor/left
operator/42
service/map-server
```

Identity should be independent of process IDs.

A process can restart.

Its principal identity should not necessarily change.

# 4. Authentication

NROS must distinguish:

```text
"I claim to be X"
```

from:

```text
"I proved I am X"
```

Authentication mechanisms can vary by deployment:

```text
cryptographic keys
certificates
hardware identity
platform credentials
federated identity
local trust roots
```

The core protocol should not force one authentication technology.

# 5. Identity Lifecycle

Principals can move through:

```text
REGISTERED
   ↓
ACTIVE
   ↓
SUSPENDED
   ↓
REVOKED
   ↓
EXPIRED
```

A revoked identity should not simply disappear.

Its historical references must remain valid.

# 6. Stable Identity vs Runtime Instance

NROS should distinguish:

```text
Principal:
    agent/navigation/01
```

from:

```text
Runtime Instance:
    process instance 0x...
```

This permits:

```text
restart
 ↓
new process instance
 ↓
same principal
```

while still preserving instance-level evidence.

# 7. Session Identity

Communication can additionally have:

```text
SessionId
```

So the hierarchy becomes:

```text
Principal
   ↓
Runtime Instance
   ↓
Session
   ↓
Operations
```

This is useful for security auditing.

# 8. Authentication Context

Every security-sensitive operation can carry:

```text
principal
session
authentication method
authentication time
credential identity
```

The operation can therefore be traced back to a specific authenticated context.

# 9. Authorization

Authentication answers:

> Who?

Authorization answers:

> What may they do?

Example:

```text
agent/navigation
```

may be authorized to:

```text
read:
    localization

request:
    navigation

not allowed:
    motor.direct_write
```

# 10. Permission Is Not Enough

NROS should prefer **capabilities** for runtime authority.

Instead of:

```text
"agent A has permission to do X"
```

the runtime can issue:

```text
Capability C42
    subject = agent A
    operation = navigation.execute
    scope = robot/01
    expiry = T
```

The capability itself becomes an authority object.

# 11. Capability

A capability can contain:

```text
Capability
├── capability_id
├── issuer
├── subject
├── operation
├── resource
├── scope
├── constraints
├── validity
└── delegation
```

# 12. Least Authority

The capability model naturally supports:

> Give an entity only the authority it actually needs.

For example:

```text
camera-agent
```

gets:

```text
camera.read
```

but not:

```text
motor.control
```

This sharply limits compromise impact.

# 13. Resource-Scoped Authority

Capabilities should be able to target specific resources:

```text
motor.control
    resource = robot/01/motor/left
```

rather than:

```text
motor.control
    resource = *
```

The latter should require exceptional authority.

# 14. Operation-Scoped Authority

Similarly:

```text
read
write
configure
execute
stop
inspect
allocate
delegate
```

can be independently authorized.

# 15. Temporal Authority

Capabilities can expire:

```text
valid_from
valid_until
```

Example:

```text
navigation.execute
valid:
    14:00–14:30
```

After expiration:

```text
authorization denied
```

# 16. Contextual Authority

Authorization may depend on runtime conditions.

For example:

```text
motor.control
```

only permitted if:

```text
robot.mode == MAINTENANCE
```

This means authorization can reference state/policy.

# 17. Safety Authority

Some operations should have special security semantics:

```text
emergency_stop
```

A safety authority should not necessarily be equivalent to ordinary application permissions.

For example:

```text
SafetyController
    can:
        STOP
```

while:

```text
NavigationAgent
    can:
        REQUEST_STOP
```

The distinction is intentional.

# 18. Delegation

An authority holder may need to delegate limited authority.

Example:

```text
FleetManager
   ↓
RobotManager
   ↓
NavigationAgent
```

Delegation should be:

```text
explicit
scoped
bounded
auditable
revocable
```

# 19. No Unlimited Delegation

A delegated capability should not automatically grant:

```text
everything the parent can do
```

Instead:

```text
Parent:
    capabilities = {A, B, C, D}

Delegate:
    capabilities = {A, C}
```

This prevents privilege escalation.

# 20. Authority Chains

A capability can retain:

```text
issued_by
delegated_by
delegated_to
```

producing:

```text
Root Authority
    ↓
Fleet Manager
    ↓
Robot Supervisor
    ↓
Agent
```

This becomes part of evidence.

# 21. Revocation

Capabilities may need immediate revocation:

```text
Capability C42
      ↓
REVOKED
```

Consumers must be able to determine whether an authority is still valid.

# 22. Trust

Authorization answers:

> Is this operation permitted?

Trust answers:

> How much confidence should I place in this entity or its output?

These must remain separate.

For example:

```text
Sensor A:
    authorized = YES
    trust = LOW
```

It may legally publish observations while still being considered unreliable.

# 23. Trust Is Contextual

A principal can be trusted for one operation but not another.

Example:

```text
camera-service:
    perception → HIGH trust
    motor-control → NOT AUTHORIZED
```

Therefore trust should be associated with:

```text
principal
capability
resource
operation
context
```

where appropriate.

# 24. Trust Sources

Trust may derive from:

```text
identity verification
history
attestation
certification
operator approval
runtime health
evidence quality
policy
```

NROS should represent these sources rather than collapsing them into an opaque score.

# 25. Trust Score

A deployment may optionally derive:

```text
trust_score
```

but the core semantics should preserve the underlying reasons.

For example:

```text
trust = 0.91
```

is less useful than:

```text
identity verified
software attested
recent failures = 0
sensor consistency = high
```

# 26. Attestation

Authentication proves:

> This entity possesses the expected identity credential.

Attestation can answer:

> What software/hardware configuration is actually running?

Conceptually:

```text
Principal
    ↓
Runtime
    ↓
Measurement
    ↓
Attestation
```

# 27. Runtime Measurement

An execution environment can expose measurements such as:

```text
software version
binary digest
configuration digest
hardware identity
container/image identity
policy version
```

This creates a runtime identity beyond a simple process name.

# 28. Attested Component

A component can therefore be represented as:

```text
ComponentIdentity
├── principal
├── runtime_instance
├── software_digest
├── configuration_digest
├── environment
└── attestation
```

# 29. Why This Matters

Suppose evidence says:

```text
NavigationAgent generated Plan P42
```

Security-aware NROS can establish:

```text
which identity?
which binary?
which configuration?
which authority?
which policy?
```

That makes the evidence substantially stronger.

# 30. Secure Communication

Communication should bind:

```text
message
+
sender identity
+
session
+
authorization context
```

The receiving component should be able to determine:

```text
who sent this?
```

rather than trusting only a topic or endpoint address.

# 31. Message Authenticity

For security-sensitive traffic, NROS should support integrity/authenticity mechanisms.

Conceptually:

```text
Payload
   +
Identity context
   +
Integrity protection
```

A modified message must be detectable.

# 32. Confidentiality

Not every ROS-like topic needs encryption.

But NROS should support confidentiality where policy requires it:

```text
private telemetry
credentials
operator data
security state
mission-sensitive information
```

The security fabric defines policy; transport implements it.

# 33. Secure Discovery

Discovery itself can leak information.

A secure NROS deployment may need to control:

```text
who can discover
what exists
which capabilities exist
which resources exist
```

Therefore:

```text
Discovery
```

should be authorization-aware.

# 34. Capability Discovery

A client may ask:

```text
"What can you do?"
```

The answer should be filtered by authority.

An untrusted principal should not necessarily receive:

```text
full capability inventory
```

# 35. Security Domains

NROS can establish security boundaries:

```text
SecurityDomain
```

Examples:

```text
production
simulation
maintenance
development
emergency
```

Different domains may have different trust roots and policies.

# 36. Environment Separation

A particularly important case:

```text
SIMULATION
```

must not accidentally receive authority equivalent to:

```text
PHYSICAL_ROBOT
```

A simulation agent should never be able to issue real actuator commands merely because it uses the same protocol.

# 37. Capability Boundaries

The runtime should make dangerous crossings explicit:

```text
SIMULATION
      │
      ✕
      │
PHYSICAL ACTUATION
```

unless an explicit bridge capability exists.

# 38. Policy

Security decisions should be policy-driven.

Conceptually:

```text
Policy
├── subject
├── action
├── resource
├── conditions
├── effect
└── priority
```

Example:

```text
IF
    subject = navigation-agent
AND
    action = motor.control
AND
    robot.mode = autonomous
THEN
    DENY
```

unless an additional capability is present.

# 39. Policy Versioning

Security decisions need reproducibility.

Therefore:

```text
policy_version
```

should be attached to authorization decisions.

If an action occurred under:

```text
Policy P17
```

the evidence should preserve that fact.

# 40. Authorization Decision

An authorization decision can become an event:

```text
AuthorizationDecision
├── subject
├── operation
├── resource
├── result
├── policy
├── capability
└── timestamp
```

Example:

```text
DENIED
reason:
    capability expired
```

# 41. Security Events

Security events should enter the same Event Fabric:

```text
AuthenticationSucceeded
AuthenticationFailed
CapabilityIssued
CapabilityRevoked
AuthorizationGranted
AuthorizationDenied
AttestationVerified
AttestationFailed
PolicyChanged
SecurityDomainChanged
```

Security therefore becomes part of the causal history.

# 42. Security Evidence

For a critical operation:

```text
MotorStart
```

NROS can retain:

```text
who requested it
which capability authorized it
which policy allowed it
which runtime executed it
which hardware responded
what state resulted
```

This is an **attested execution chain**.

# 43. Attested Execution Chain

Conceptually:

```text
Principal
   ↓
Authentication
   ↓
Capability
   ↓
Authorization
   ↓
Plan
   ↓
Work
   ↓
Runtime
   ↓
Attestation
   ↓
Hardware
   ↓
Observed Result
```

This is one of the strongest architectural differences between NROS and traditional middleware.

# 44. Security Failure

Suppose:

```text
NavigationAgent
```

requests:

```text
motor.control
```

but its capability expired.

NROS should produce:

```text
AuthorizationDenied
```

and then the planner may:

```text
replan
request authority
or enter safe state
```

Security failure becomes part of normal runtime semantics.

# 45. Security Is Part of Planning

Planning must consider authority.

A plan may be technically feasible:

```text
Plan A:
    drive through restricted zone
```

but impossible under current authority:

```text
permission = DENIED
```

Therefore planning should consume:

```text
capabilities
policies
authority
```

as constraints.

# 46. Capability-Aware Planning

The planner can generate:

```text
Plan A:
    requires capability C1

Plan B:
    requires capability C2

Plan C:
    requires no restricted capability
```

Then select an executable plan.

This avoids discovering authorization failure only after execution begins.

# 47. Security-Aware Recovery

If a capability is revoked while work is running:

```text
W193 RUNNING
     ↓
capability revoked
     ↓
policy evaluation
     ↓
pause / stop / safe transition
```

The appropriate behavior depends on operation safety.

# 48. Capability Lease

Long-running work can use a lease:

```text
CapabilityLease
```

which must periodically be renewed.

If renewal fails:

```text
lease expires
```

and the runtime executes the configured safety policy.

# 49. Secure Checkpoint

Earlier we introduced checkpoints.

Now a checkpoint can include:

```text
identity
authority
policy version
capability state
attestation
state
plan
work
memory
```

This prevents recovery from silently inheriting obsolete authority.

# 50. Security-Aware Recovery

Recovery becomes:

```text
restore checkpoint
      ↓
authenticate runtime
      ↓
validate identity
      ↓
validate capabilities
      ↓
validate policy
      ↓
validate state
      ↓
resume / replan / safe-stop
```

This is substantially safer than blindly restoring a process snapshot.

# 51. Zero-Trust Principle

A useful NROS security principle is:

> **No component is trusted merely because it is inside the robot or network.**

Trust must derive from:

```text
identity
authentication
authorization
capability
attestation
policy
evidence
```

# 52. Security and Evidence Converge

We can now connect the previous fabrics:

```text
Security
   ↓
Authorization Event
   ↓
Execution Event
   ↓
State Change
   ↓
Evidence
```

Thus security decisions become part of the historical explanation of autonomous behavior.

# 53. NROS Security Model

The resulting abstraction is:

```text
WHO?
  Principal

PROVE WHO?
  Authentication

WHAT MAY THEY DO?
  Capability

IS THIS OPERATION ALLOWED?
  Authorization

HOW MUCH SHOULD WE TRUST IT?
  Trust

WHAT ACTUALLY RAN?
  Attestation

WHAT ACTUALLY HAPPENED?
  Evidence
```

# 54. Fourteen-Fabric Architecture

NROS now has:

```text
┌─────────────────────────────────────────────┐
│ Domain & Deployment                         │
├─────────────────────────────────────────────┤
│ Identity, Trust & Security                 │
├─────────────────────────────────────────────┤
│ Resource & Allocation                       │
├─────────────────────────────────────────────┤
│ Intent & Planning                           │
├─────────────────────────────────────────────┤
│ Knowledge & State                           │
├─────────────────────────────────────────────┤
│ Memory, Event & Evidence                    │
├─────────────────────────────────────────────┤
│ Protocol & Type                             │
├─────────────────────────────────────────────┤
│ Observability                               │
├─────────────────────────────────────────────┤
│ Capability & Authority                      │
├─────────────────────────────────────────────┤
│ Supervision                                 │
├─────────────────────────────────────────────┤
│ Temporal                                    │
├─────────────────────────────────────────────┤
│ Execution                                   │
├─────────────────────────────────────────────┤
│ Communication                               │
└─────────────────────────────────────────────┘
```

There is an important architectural observation here:

**Identity/Trust/Security and Capability/Authority are closely related, but they should not be collapsed.**

Identity answers *who*.

Capability answers *what authority was granted*.

Policy answers *whether that authority is usable here and now*.

# 55. The Next Problem: Distributed Coordination

NROS is now capable of having:

```text
Agent A
Agent B
Robot A
Robot B
Fleet Controller
Edge Node
Cloud Service
Human Operator
```

all acting within the same larger system.

That immediately creates another problem:

> **How do multiple autonomous actors coordinate without corrupting shared state, duplicating work, fighting over resources, or making incompatible decisions?**

This leads to:

# Part LIII — NROS Coordination, Consensus & Distributed Orchestration Fabric

The next progression is:

```text
Multiple Agents
      ↓
Shared Goals
      ↓
Coordination
      ↓
Negotiation
      ↓
Resource Arbitration
      ↓
Distributed Execution
      ↓
Conflict Resolution
      ↓
Global Outcome
```

And this is where NROS must confront one of the deepest differences from ROS:

> ROS primarily connects distributed processes.

> **NROS must coordinate distributed autonomous actors.**

# NROS — Part LIII: Coordination, Consensus & Distributed Orchestration Fabric

With Identity, Trust, Capability, and Evidence established, NROS can safely move from **single-agent autonomy** to **multi-agent autonomy**.

The central transition is:

```text
ROS:
    distributed computation

NROS:
    distributed autonomous coordination
```

The hard problem is no longer merely:

> "How do agents communicate?"

It becomes:

> **"How do independent agents reach compatible decisions while respecting authority, resources, timing, safety, and partial knowledge?"**

# 1. Coordination Is Not Communication

Two agents can communicate perfectly and still conflict.

Example:

```text
Agent A:
    wants corridor C

Agent B:
    wants corridor C
```

Both publish valid messages.

Both have valid capabilities.

Both execute.

The system fails because **communication did not establish coordination**.

Therefore NROS needs a separate coordination fabric.

# 2. Coordination Object

Introduce:

```text
CoordinationSession
```

Conceptually:

```text
CoordinationSession
├── session_id
├── participants
├── objective
├── resources
├── constraints
├── authority
├── protocol
├── deadline
└── outcome
```

# 3. Participants

Participants can include:

```text
robot
agent
human
service
planner
supervisor
resource-manager
safety-controller
```

A participant is not automatically equal to every other participant.

Its role matters.

# 4. Roles

For example:

```text
Coordinator
Planner
Executor
Observer
Approver
Arbiter
SafetyAuthority
```

One entity can hold multiple roles, but each role should remain explicit.

# 5. Shared Objective

Coordination should begin with an explicit objective:

```text
Objective O17:
    transport package P
    from A → B
    before deadline T
```

Participants can then reason against the same objective.

# 6. Shared Goal vs Local Goal

This distinction is critical.

```text
Shared Goal:
    package reaches B
```

while:

```text
Agent A:
    minimize travel time

Agent B:
    minimize energy

Agent C:
    maintain safety margin
```

NROS must support both:

```text
global objective
+
local objectives
```

without pretending they are identical.

# 7. Goal Ownership

A goal needs an owner or authority source:

```text
Goal
├── goal_id
├── issuer
├── owner
├── participants
├── constraints
└── lifecycle
```

Otherwise multiple agents may believe they independently own the same mission.

# 8. Goal Lifecycle

A distributed goal can transition through:

```text
PROPOSED
   ↓
ACCEPTED
   ↓
COORDINATING
   ↓
COMMITTED
   ↓
EXECUTING
   ↓
VERIFIED
   ↓
COMPLETED
```

Alternative outcomes:

```text
REJECTED
ABORTED
EXPIRED
SUPERSEDED
FAILED
```

# 9. Proposal

An agent may propose:

```text
Proposal P42
```

containing:

```text
objective
plan
required resources
estimated cost
constraints
authority requirements
deadline
confidence
```

Other participants can evaluate it.

# 10. Negotiation

Instead of immediately executing:

```text
Agent A → Execute
```

NROS can support:

```text
Proposal
   ↓
Counterproposal
   ↓
Evaluation
   ↓
Agreement
```

Example:

```text
A:
    I can transport P using corridor C.

B:
    Corridor C unavailable until 12:10.

A:
    I can use corridor D.

B:
    D requires additional energy allocation.

A:
    accepted.
```

# 11. Negotiation Must Be Structured

Avoid relying on free-form conversational messages for critical coordination.

Use typed objects:

```text
Proposal
CounterProposal
Constraint
Commitment
Reservation
Agreement
Rejection
```

Natural language may exist at the interface layer, but the runtime needs deterministic semantics.

# 12. Commitment

Once agents agree:

```text
Commitment C17
```

records:

```text
participant
obligation
resource
deadline
conditions
verification
```

Example:

```text
Robot A
    commits to:
        deliver package P
        before 12:30
```

# 13. Commitment Is Not Execution

This distinction is essential.

```text
Commitment:
    "I promise to do X."

Execution:
    "I am currently doing X."

Evidence:
    "X actually happened."
```

These must never be conflated.

# 14. Commitment State

A commitment can become:

```text
PROPOSED
ACCEPTED
ACTIVE
FULFILLED
BREACHED
CANCELLED
EXPIRED
```

A breach becomes an event:

```text
CommitmentBreached
```

and may trigger replanning.

# 15. Reservation

Resources should be reservable.

Example:

```text
Resource:
    charging station 3
```

Reservation:

```text
R88
    owner = Robot A
    interval = 12:00–12:20
```

Robot B should not independently assume availability.

# 16. Reservation ≠ Allocation

Reservation means:

> "This resource is promised for a future operation."

Allocation means:

> "The system has actually assigned the resource."

This distinction matters in distributed systems.

```text
Proposal
   ↓
Reservation
   ↓
Allocation
   ↓
Execution
```

# 17. Resource Arbitration

If two agents request:

```text
Resource X
```

NROS needs an arbiter.

Possible policies:

```text
priority
deadline
safety
fairness
cost
authority
distance
energy
mission criticality
```

# 18. Arbitration Result

The result should itself be explicit:

```text
ArbitrationDecision
├── resource
├── contenders
├── winner
├── policy
├── reason
└── validity
```

This creates an auditable decision.

# 19. Priority

Priority should not simply mean:

```text
higher number = wins
```

NROS can model priority as a policy input:

```text
mission-critical
safety-critical
deadline-critical
normal
background
```

# 20. Safety Dominates Optimization

Suppose:

```text
Agent A:
    fastest route

Agent B:
    safest route
```

The coordination system should never allow optimization to override safety constraints.

The conceptual ordering is:

```text
Safety constraints
        ↓
Authority constraints
        ↓
Hard resource constraints
        ↓
Mission constraints
        ↓
Optimization objectives
```

# 21. Hard vs Soft Constraints

NROS should explicitly distinguish:

```text
Hard constraint:
    MUST NOT violate

Soft constraint:
    SHOULD satisfy
```

Example:

```text
battery >= safety threshold
    HARD

minimize travel time
    SOFT
```

This prevents planners from trading away mandatory safety conditions.

# 22. Distributed State

Multiple agents may maintain partial views:

```text
Agent A:
    knows X, Y

Agent B:
    knows Y, Z

Agent C:
    knows X, Z
```

There is no guarantee that all agents have identical state at every instant.

NROS therefore needs:

```text
state version
origin
timestamp
authority
validity
```

# 23. State Version

Example:

```text
Robot A pose
version = 184
```

A newer observation:

```text
version = 185
```

can supersede it.

But different domains may have different versions simultaneously.

# 24. State Authority

A critical concept:

```text
Who is authoritative for this state?
```

For example:

```text
Robot:
    physical battery level
        → battery controller

Fleet:
    mission assignment
        → fleet coordinator

Map:
    canonical map
        → map authority
```

Authority prevents arbitrary agents from overwriting shared truth.

# 25. Distributed State Does Not Mean Global Consensus

Not every piece of state requires consensus.

For example:

```text
sensor temperature
```

doesn't necessarily need a distributed consensus protocol.

But:

```text
who owns charging station 3?
```

may require strong coordination.

NROS should therefore classify state by consistency requirement.

# 26. Consistency Classes

A useful model:

```text
LOCAL
EVENTUAL
CAUSAL
COORDINATED
STRONG
```

Each incurs different cost.

# 27. Local State

Only one component needs the state.

Example:

```text
motor controller internal temperature
```

No distributed agreement required.

# 28. Eventual State

Some information can converge later:

```text
robot telemetry
```

Temporary divergence is acceptable.

# 29. Causal State

The ordering of related events matters:

```text
command
   ↓
acknowledgement
   ↓
result
```

Consumers need causal consistency.

# 30. Coordinated State

Multiple agents must agree before changing state.

Example:

```text
resource ownership
```

This requires an explicit coordination protocol.

# 31. Strong State

Some safety-critical state may require stronger guarantees:

```text
emergency mode
actuator authority
safety interlock
```

The implementation should choose the appropriate consensus/coordination mechanism.

NROS should define the semantics rather than force one algorithm everywhere.

# 32. Consensus

Consensus answers a narrow question:

> Which value or decision should the participants agree upon?

It should not become a universal mechanism for all NROS communication.

For example:

```text
sensor stream
```

doesn't need consensus.

But:

```text
leader election
resource ownership
mission commitment
```

may.

# 33. Consensus Object

A consensus instance can be represented as:

```text
Consensus
├── instance_id
├── proposal
├── participants
├── quorum
├── decision
├── term
└── evidence
```

# 34. Quorum

The runtime may define a quorum appropriate to the coordination domain.

For example:

```text
3 coordinators
quorum = 2
```

The important point is that the decision semantics must be explicit.

# 35. Consensus Is Not Authority

A majority vote does not automatically make something safe.

For example:

```text
2 agents:
    "activate motor"

1 safety controller:
    "unsafe"
```

The majority must not override an independent safety authority if policy says safety has veto power.

Thus:

```text
Consensus
```

operates **inside authority boundaries**.

# 36. Safety Veto

NROS should support explicit veto semantics:

```text
Proposal
   ↓
Consensus
   ↓
Safety Check
   ↓
VETO
```

or:

```text
Proposal
   ↓
Safety Check
   ↓
Consensus
   ↓
Commit
```

The exact order depends on the domain.

# 37. Leader

Some coordination protocols need a leader:

```text
Coordinator
```

But NROS should treat leadership as a role, not an identity property.

A robot may be leader for:

```text
mission M1
```

without being globally superior.

# 38. Leader Election

Leadership may be:

```text
ELECTED
ASSIGNED
DESIGNATED
EMERGENCY
```

and have:

```text
term
scope
expiry
authority
```

# 39. Leader Failure

If a leader disappears:

```text
Leader
   ↓
failure
   ↓
detection
   ↓
election / reassignment
   ↓
new leader
```

The system must prevent split-brain execution.

# 40. Fencing

A particularly important mechanism is:

```text
Fencing
```

When authority changes:

```text
old leader
```

must no longer be capable of performing conflicting operations.

Example:

```text
Leader term = 7

new leader:
    term = 8
```

Commands from term 7 can be rejected.

# 41. Epoch

NROS can generalize this through:

```text
Epoch
```

An epoch identifies the current coordination authority/configuration.

This helps reject stale commands.

# 42. Stale Work

Suppose:

```text
Plan P42
epoch = 7
```

Later:

```text
epoch = 8
```

The old plan may no longer be valid.

NROS should require:

```text
revalidation
```

before resuming it.

# 43. Split-Brain Prevention

A distributed robot fleet must avoid:

```text
Coordinator A:
    Robot 1 belongs to me.

Coordinator B:
    Robot 1 belongs to me.
```

Both sending conflicting commands.

Epochs, leases, fencing, and authority boundaries are mechanisms for preventing this.

# 44. Lease-Based Coordination

A coordination lease can say:

```text
Coordinator A
owns resource R
until T
```

If it fails to renew:

```text
lease expires
```

and ownership becomes available for reassignment.

# 45. Clock Uncertainty

Lease semantics require careful temporal handling.

NROS must account for:

```text
clock drift
network delay
processing delay
```

This reinforces the importance of the Temporal Fabric introduced earlier.

# 46. Distributed Deadlines

A commitment might state:

```text
deliver by 12:30
```

but each participant has different clocks.

Therefore NROS should represent deadlines semantically and avoid assuming perfectly synchronized wall clocks.

# 47. Coordination Protocol

A generalized coordination protocol can be:

```text
1. Discover participants
2. Establish identities
3. Validate capabilities
4. Create objective
5. Publish proposal
6. Evaluate constraints
7. Negotiate
8. Reserve resources
9. Reach agreement
10. Commit
11. Execute
12. Verify
13. Release resources
14. Record evidence
```

# 48. Failure During Negotiation

If an agent disappears before commitment:

```text
Proposal
   ↓
Participant lost
   ↓
Negotiation invalidated
   ↓
Reconfigure
```

No partial agreement should silently become authoritative.

# 49. Failure During Execution

If an agent fails after commitment:

```text
Commitment C17
     ↓
Agent failure
     ↓
Commitment at risk
     ↓
Coordinator evaluates
     ↓
reassign / recover / abort
```

This is where NROS's earlier Work and Recovery models become essential.

# 50. Reassignment

A failed work item:

```text
W193
```

may be reassigned:

```text
Agent A → failed

Agent B → takeover
```

But B should inherit:

```text
goal
constraints
evidence
partial state
resource context
```

rather than blindly restarting.

# 51. Handoff

Introduce:

```text
Handoff
```

with:

```text
from
to
work
state
evidence
authority
conditions
```

This gives takeover a formal semantic representation.

# 52. Shared Blackboard

NROS may support a coordination-oriented shared state abstraction:

```text
Coordination Blackboard
```

containing:

```text
goals
proposals
commitments
reservations
resource state
alerts
coordination state
```

It is not equivalent to the ROS1 parameter server.

It represents dynamic coordination state rather than merely configuration parameters.

# 53. Coordination Graph

A mission can be represented as:

```text
Mission M
│
├── Goal G1
│
├── Participants
│   ├── Robot A
│   ├── Robot B
│   └── Agent C
│
├── Resources
│   ├── Corridor X
│   └── Charger Y
│
├── Commitments
│   ├── C1
│   └── C2
│
└── Evidence
    ├── E1
    └── E2
```

This is a **mission coordination graph**.

# 54. From Computation Graph to Coordination Graph

This is another major architectural transition.

Traditional ROS:

```text
Node → Topic → Node
```

NROS:

```text
Principal
   ↓
Intent
   ↓
Goal
   ↓
Coordination
   ↓
Commitment
   ↓
Work
   ↓
Evidence
```

The graph now represents **agency**, not merely computation.

# 55. Multi-Agent Planning

NROS planners should eventually support:

```text
single-agent planning
multi-agent planning
hierarchical planning
distributed planning
negotiated planning
contingency planning
```

But the runtime should not require one universal planner.

# 56. Hierarchical Coordination

A large mission can decompose:

```text
Mission
 ├── Subgoal A
 │    ├── Work A1
 │    └── Work A2
 │
 ├── Subgoal B
 │    ├── Work B1
 │    └── Work B2
 │
 └── Subgoal C
```

Different agents can own different subtrees.

# 57. Dependency Graph

Tasks can express dependencies:

```text
A ──→ C
B ──→ C
```

meaning:

```text
C cannot begin until A and B satisfy their completion conditions.
```

This creates a DAG-like execution structure.

# 58. Conditional Dependencies

Dependencies may be conditional:

```text
A succeeds → C
A fails → D
```

The coordination layer should support such branching explicitly.

# 59. Barrier

Multiple agents may need to reach a synchronization point:

```text
Robot A ──┐
Robot B ──┼──→ Barrier → Continue
Robot C ──┘
```

A barrier should have:

```text
participants
condition
deadline
failure policy
```

# 60. Synchronization Policy

If one participant never arrives:

```text
Barrier timeout
```

the system may:

```text
abort
continue with degraded mode
reassign
retry
escalate
```

This must be policy-driven.

# 61. Distributed Event Ordering

The Event Fabric introduced earlier must now handle events from multiple agents:

```text
A:E1
B:E1
C:E1
A:E2
...
```

NROS should preserve:

```text
local order
causal order
correlation
origin
```

without pretending that one universal physical ordering always exists.

# 62. Causal Coordination

If:

```text
Agent A:
    obstacle detected

Agent B:
    replans based on A's observation
```

the event chain should preserve:

```text
A:ObstacleDetected
       ↓
B:PlanInvalidated
       ↓
B:Replan
```

This provides distributed causal traceability.

# 63. Conflict Detection

NROS should detect conflicts such as:

```text
resource conflict
goal conflict
authority conflict
temporal conflict
spatial conflict
state conflict
plan conflict
```

before execution when possible.

# 64. Conflict Object

```text
Conflict
├── conflict_id
├── participants
├── objects
├── type
├── severity
├── detected_at
├── resolution
└── evidence
```

# 65. Conflict Resolution

Possible mechanisms:

```text
priority
negotiation
arbitration
preemption
replanning
escalation
safety stop
```

The runtime should make the chosen mechanism explicit.

# 66. Preemption

Suppose:

```text
Robot A:
    normal mission
```

and:

```text
Emergency Mission:
    higher priority
```

NROS may preempt:

```text
normal work
```

but preemption itself must be authorized.

# 67. Graceful Preemption

Not every operation can be immediately terminated.

Work should declare:

```text
preemption_mode
```

such as:

```text
IMMEDIATE
SAFE_POINT
CHECKPOINT
NON_PREEMPTIBLE
```

This integrates coordination with execution safety.

# 68. Escalation

If distributed agents cannot resolve a conflict:

```text
Agent A
   ↕
Agent B
   ↕
No agreement
   ↓
Supervisor
   ↓
Human / Safety Authority
```

Escalation should itself be a first-class event.

# 69. Human-in-the-Loop

Humans can participate as principals:

```text
Human Operator
```

with explicit capabilities:

```text
approve
deny
override
pause
resume
authorize
```

A human decision becomes:

```text
DecisionEvent
```

with provenance.

# 70. Human Authority Must Be Bounded

"Human override" should not mean:

```text
human can do absolutely anything
```

It should be:

```text
human principal
+
specific capability
+
specific scope
+
specific context
```

# 71. The Coordination Loop

The complete multi-agent loop becomes:

```text
Observe
   ↓
Interpret
   ↓
Generate Intent
   ↓
Discover Participants
   ↓
Evaluate Authority
   ↓
Propose
   ↓
Negotiate
   ↓
Reserve
   ↓
Commit
   ↓
Execute
   ↓
Observe
   ↓
Verify
   ↓
Record Evidence
   ↓
Update Shared Knowledge
   ↓
Re-coordinate
```

This is the beginning of a genuinely distributed autonomous runtime.

# 72. Fifteen-Fabric NROS Architecture

The architecture now becomes:

```text
┌──────────────────────────────────────────────┐
│ Domain & Deployment                          │
├──────────────────────────────────────────────┤
│ Identity, Trust & Security                   │
├──────────────────────────────────────────────┤
│ Coordination & Distributed Orchestration     │
├──────────────────────────────────────────────┤
│ Resource & Allocation                        │
├──────────────────────────────────────────────┤
│ Intent & Planning                            │
├──────────────────────────────────────────────┤
│ Knowledge & State                            │
├──────────────────────────────────────────────┤
│ Memory, Event & Evidence                     │
├──────────────────────────────────────────────┤
│ Protocol & Type                              │
├──────────────────────────────────────────────┤
│ Observability                                │
├──────────────────────────────────────────────┤
│ Capability & Authority                       │
├──────────────────────────────────────────────┤
│ Supervision                                  │
├──────────────────────────────────────────────┤
│ Temporal                                     │
├──────────────────────────────────────────────┤
│ Execution                                    │
├──────────────────────────────────────────────┤
│ Communication                                │
└──────────────────────────────────────────────┘
```

# 73. The Critical NROS Invariant

At this point we can formulate a fundamental invariant:

> **No autonomous work should become externally consequential merely because an agent generated it.**

The transition must be:

```text
Intent
  ↓
Plan
  ↓
Authority
  ↓
Coordination
  ↓
Commitment
  ↓
Execution
  ↓
Verification
```

Each boundary is observable and auditable.

# 74. The Next Boundary

Distributed coordination solves:

```text
who works with whom?
who owns what?
who commits?
who arbitrates?
```

But NROS still needs to answer:

> **How are physical resources represented, reserved, allocated, preempted, and released under real-time and safety constraints?**

That takes us to:

# Part LIV — NROS Resource, Scheduling & Real-Time Execution Fabric

The next transition is:

```text
Resource
   ↓
Availability
   ↓
Reservation
   ↓
Allocation
   ↓
Scheduling
   ↓
Execution
   ↓
Preemption
   ↓
Deadline
   ↓
Verification
```

This is where NROS moves from an **agent coordination model** toward a genuine **robotics runtime architecture**.

# NROS — Part LIV: Resource, Scheduling & Real-Time Execution Fabric

The Coordination Fabric answers:

> **Who should do what, with whom?**

The next layer answers:

> **When can it run, on which resource, under which timing guarantees, and what happens when resources become constrained?**

This is where NROS must go beyond the traditional ROS computation graph.

## 1. Resource Is a First-Class Runtime Concept

In a conventional middleware model, resources are often implicit:

```text
node
  ↓
CPU
  ↓
memory
  ↓
network
```

NROS should make them explicit:

```text
Resource
├── CPU
├── Memory
├── Storage
├── GPU / NPU
├── Network
├── Sensor
├── Actuator
├── Power
├── Spatial region
├── Communication channel
└── Exclusive physical equipment
```

The important insight is:

> **A robot's resources are not merely computing resources.**

They include the physical world.

# 2. Resource Identity

Every managed resource can have an identity:

```text
ResourceId
```

Examples:

```text
cpu/0
gpu/0
camera/front
lidar/main
arm/joint/3
motor/left
battery/main
charger/station/04
workspace/zone/A
network/link/0
```

This gives NROS a common resource model across software and hardware.

# 3. Resource Ownership

A resource can have:

```text
owner
```

but ownership should be distinguished from current use.

For example:

```text
Robot A
    owns:
        arm/01

Agent B
    currently uses:
        arm/01
```

through an authorized allocation.

# 4. Resource State

A resource can transition through:

```text
AVAILABLE
RESERVED
ALLOCATED
IN_USE
DEGRADED
FAULTED
QUARANTINED
OFFLINE
```

These states become observable runtime facts.

# 5. Resource Availability

Availability is not binary.

Instead:

```text
ResourceAvailability
├── capacity
├── current_usage
├── reservations
├── constraints
├── health
└── validity_interval
```

For example:

```text
CPU:
    capacity = 8 cores
    available = 3 cores
```

# 6. Resource Capacity

Some resources are divisible:

```text
CPU
RAM
GPU
bandwidth
battery
```

Others are exclusive:

```text
motor
camera
charging station
workspace
```

NROS should therefore support different allocation models.

# 7. Exclusive Resource

Example:

```text
charger/04
```

may have:

```text
capacity = 1
```

Two agents cannot simultaneously hold exclusive allocation.

# 8. Fractional Resource

CPU could be:

```text
capacity = 100%
```

and allocations:

```text
Agent A = 40%
Agent B = 30%
Agent C = 20%
```

subject to scheduling policy.

# 9. Consumable Resource

Some resources decrease with use:

```text
battery
fuel
storage
bandwidth quota
```

For these:

```text
initial capacity
      ↓
consumption
      ↓
remaining capacity
```

must be represented explicitly.

# 10. Renewable Resource

Others recover:

```text
CPU time
network bandwidth
battery charging
```

The model should distinguish:

```text
consumable
renewable
exclusive
shareable
```

# 11. Resource Requirement

A Work item should declare what it requires:

```text
Work W42
├── CPU >= 20%
├── camera/front
├── network >= 5 Mbps
└── battery >= safety threshold
```

The scheduler can therefore reason before execution.

# 12. Requirement vs Allocation

Again:

```text
Requirement:
    "I need a camera."

Allocation:
    "camera/front is assigned to me."
```

These must not be conflated.

# 13. Resource Reservation

Coordination can produce:

```text
Reservation R42
```

with:

```text
resource
owner
interval
capacity
priority
conditions
expiry
```

Example:

```text
camera/front
reserved:
    Agent A
    14:00–14:05
```

# 14. Reservation Is a Promise

Reservation means:

> "The system intends to keep this resource available."

It does not necessarily mean the resource is currently executing work.

Therefore:

```text
Reservation
      ↓
Allocation
      ↓
Execution
```

remains a meaningful sequence.

# 15. Allocation

Allocation is the authoritative assignment:

```text
Allocation A91
    resource = camera/front
    subject = perception-agent
    validity = ...
```

Other agents should respect it unless policy explicitly permits preemption.

# 16. Scheduling

Now we reach the scheduler.

The scheduler decides:

```text
what
runs
where
when
for how long
with what priority
```

But NROS should support **multiple schedulers**.

# 17. Scheduler Hierarchy

A deployment may contain:

```text
Global Scheduler
      ↓
Robot Scheduler
      ↓
Process Scheduler
      ↓
Hardware Scheduler
```

Different layers solve different problems.

# 18. Global Scheduler

Handles:

```text
fleet allocation
mission scheduling
robot assignment
large resource conflicts
```

# 19. Robot Scheduler

Handles:

```text
CPU
GPU
sensor pipelines
actuator workloads
local tasks
```

# 20. Process Scheduler

Handles:

```text
threads
executors
callbacks
work queues
```

This is closer to operating-system scheduling.

# 21. Hardware Scheduler

Handles hardware-specific constraints:

```text
motor cycle
sensor sampling
DMA
FPGA pipeline
bus arbitration
```

# 22. Real-Time Boundary

NROS should explicitly distinguish:

```text
real-time critical
near-real-time
soft real-time
best effort
background
```

Not every operation requires hard deadlines.

# 23. Hard Real-Time

A hard real-time task has:

```text
deadline
```

where missing it is considered a failure of correctness.

Example:

```text
motor control loop
```

Conceptually:

```text
period = 1 ms
deadline = 1 ms
```

# 24. Soft Real-Time

For perception:

```text
target:
    30 Hz
```

Missing occasional frames may degrade quality without constituting a system failure.

# 25. Best Effort

Examples:

```text
telemetry
logging
diagnostics
analytics
```

These should not consume resources needed by safety-critical work merely because they happen to be active.

# 26. Deadline

Every scheduled Work item may declare:

```text
deadline
```

but NROS should distinguish:

```text
start deadline
completion deadline
response deadline
validity deadline
```

# 27. Periodic Work

A periodic task:

```text
Work W
period = 10 ms
```

can generate:

```text
W1
W2
W3
W4
...
```

Each instance can have its own execution evidence.

# 28. Sporadic Work

Some work occurs irregularly:

```text
obstacle detected
```

which creates:

```text
urgent avoidance work
```

The scheduler should be able to accommodate event-triggered execution.

# 29. Aperiodic Work

Other work has no predictable period:

```text
map export
diagnostic report
model update
```

This can be scheduled opportunistically.

# 30. Priority

NROS priorities should reflect semantic importance.

For example:

```text
SAFETY
CONTROL
REALTIME
MISSION
PERCEPTION
PLANNING
TELEMETRY
BACKGROUND
```

But priority alone is insufficient.

A high-priority task without authorization should still be denied.

# 31. Priority Is Not Authority

This is a critical invariant:

```text
priority ≠ permission
```

An unauthorized task cannot become authorized merely by having a higher scheduler priority.

# 32. Scheduling Decision

A scheduler decision can be represented as:

```text
ScheduleDecision
├── work
├── resource
├── start
├── expected_duration
├── priority
├── constraints
├── scheduler
└── decision_version
```

This makes scheduling auditable.

# 33. Admission Control

Before scheduling work, NROS can ask:

```text
Can this work safely enter execution?
```

Checks include:

```text
authority
resource availability
timing feasibility
safety constraints
memory
power
dependencies
```

If not:

```text
ADMISSION_DENIED
```

# 34. Admission vs Scheduling

These are different.

```text
Admission:
    "May this work enter the system?"

Scheduling:
    "When should it run?"
```

This separation is valuable.

# 35. Feasibility

Suppose:

```text
Task A:
    execution = 8 ms
    deadline = 10 ms

Task B:
    execution = 8 ms
    deadline = 10 ms
```

If only one processor exists, both cannot necessarily satisfy their deadlines.

The scheduler should detect infeasibility instead of silently accepting impossible work.

# 36. Deadline Failure

When a deadline becomes impossible:

```text
Work W42
    ↓
deadline infeasible
```

NROS should produce an explicit state:

```text
DEADLINE_MISSED
```

and trigger policy:

```text
retry
degrade
drop
replan
escalate
abort
```

# 37. Temporal Contracts

A Work item can declare:

```text
TemporalContract
├── release_time
├── start_deadline
├── completion_deadline
├── period
├── jitter
└── duration_budget
```

This turns timing requirements into machine-readable constraints.

# 38. Jitter

For control systems, average latency is insufficient.

NROS should be capable of representing:

```text
expected latency
worst-case latency
jitter
```

because:

```text
average = 1 ms
```

does not imply:

```text
worst case = 1 ms
```

# 39. Execution Budget

A Work item can have:

```text
CPU budget
memory budget
energy budget
network budget
time budget
```

If the work exceeds its budget:

```text
BudgetExceeded
```

is emitted.

# 40. Budget Policy

Possible responses:

```text
WARN
THROTTLE
PREEMPT
RESTART
ABORT
DEGRADE
ESCALATE
```

The policy depends on the Work type.

# 41. Resource Preemption

A resource may need to be reassigned:

```text
Agent A
    currently owns resource X

Emergency Work
    requires X
```

NROS can invoke:

```text
preemption
```

but only if:

```text
authority
+
resource policy
+
preemption semantics
```

allow it.

# 42. Safe Preemption

A motor-control task may require:

```text
SAFE_POINT
```

while a telemetry task may be:

```text
IMMEDIATE
```

Therefore resource preemption should use the Work's declared semantics.

# 43. Scheduler Domains

Different resources may use different scheduling domains:

```text
CPU domain
GPU domain
actuator domain
network domain
mission domain
```

NROS can coordinate across domains without forcing them into a single scheduling algorithm.

# 44. Cross-Domain Scheduling

Example:

```text
Perception Work
    requires:
        camera
        GPU
        CPU
        network
```

All four resources must be compatible.

This becomes a **multi-resource scheduling problem**.

# 45. Resource Bundle

Represent this explicitly:

```text
ResourceBundle
├── camera/front
├── gpu/0
├── cpu/2
└── network/link/1
```

The scheduler can allocate the bundle atomically where necessary.

# 46. Partial Allocation

Sometimes partial allocation is useful.

Example:

```text
GPU allocated
camera unavailable
```

The work should not automatically execute.

It can remain:

```text
WAITING_FOR_RESOURCE
```

rather than entering an invalid partial state.

# 47. Resource Wait Graph

NROS can detect deadlocks using dependencies:

```text
Agent A
   waits for X

Agent B
   waits for Y

X held by B
Y held by A
```

Graph:

```text
A → X → B → Y → A
```

Cycle detected.

# 48. Deadlock Detection

A coordination runtime should identify:

```text
resource deadlock
coordination deadlock
commitment deadlock
```

and apply policy:

```text
preempt
rollback
cancel
reassign
escalate
```

# 49. Starvation

Another problem:

```text
Low-priority Work
```

may never execute because higher-priority work continually arrives.

NROS can support:

```text
aging
fairness
quota
reservation windows
```

to prevent indefinite starvation.

# 50. Fairness

For multi-agent fleets:

```text
Robot A
Robot B
Robot C
```

should not necessarily allow one participant to monopolize:

```text
charger
corridor
compute cluster
```

unless policy explicitly requires it.

# 51. Energy-Aware Scheduling

Robotics makes energy a scheduling dimension.

Example:

```text
Robot A:
    battery = 20%

Robot B:
    battery = 80%
```

A mission requiring long travel should prefer B if other constraints permit.

Energy becomes:

```text
resource
constraint
optimization variable
```

simultaneously.

# 52. Thermal-Aware Scheduling

Similarly:

```text
GPU temperature = high
```

may cause:

```text
throttling
migration
degradation
cooldown
```

This is a runtime scheduling event, not merely a diagnostic.

# 53. Fault-Aware Scheduling

If:

```text
camera/front = DEGRADED
```

the scheduler may:

```text
switch sensor
reduce perception rate
move work
change planner
```

The resource model therefore feeds directly into autonomy.

# 54. Resource Failure

A resource failure should produce:

```text
ResourceFault
```

followed by:

```text
ImpactAnalysis
      ↓
Affected Work
      ↓
Affected Commitments
      ↓
Replanning
```

This creates a causal chain.

# 55. Scheduling + Evidence

Every consequential scheduling decision should be explainable:

```text
Why did Robot B receive charger 4?
```

NROS should be able to answer:

```text
because:
    priority = mission-critical
    battery = 12%
    deadline = 14:30
    policy = fleet-scheduling-v3
    competing requests = A,C
```

This is precisely where the Evidence Fabric becomes valuable.

# 56. Scheduling Trace

A scheduling trace might look like:

```text
Request R17
   ↓
AdmissionCheck
   ↓
ResourceDiscovery
   ↓
ConflictDetected
   ↓
Arbitration
   ↓
Allocation
   ↓
Execution
   ↓
Completion
```

Every stage can generate evidence.

# 57. Real-Time Execution Boundary

NROS should not attempt to make every part of the runtime hard real-time.

Instead:

```text
                    NROS
                      │
          ┌───────────┴───────────┐
          │                       │
   Deterministic Plane       Autonomous Plane
          │                       │
     control loops          planning
     actuator timing        negotiation
     deadlines              reasoning
     safety                  memory
```

This separation is fundamental.

# 58. Deterministic Plane

The deterministic plane should minimize:

```text
dynamic allocation
unbounded latency
blocking I/O
unpredictable scheduling
garbage collection
```

where the deployment requires hard real-time guarantees.

# 59. Autonomous Plane

The autonomous plane can tolerate:

```text
variable latency
dynamic memory
complex planning
network calls
model inference
negotiation
```

provided its outputs cross controlled boundaries before affecting real-time execution.

# 60. The Critical Bridge

The bridge should be:

```text
Autonomous Decision
       ↓
Validation
       ↓
Authority
       ↓
Temporal Check
       ↓
Safety Check
       ↓
Real-Time Command
```

An AI planner should never directly bypass this boundary.

# 61. Real-Time Command

A command entering the deterministic plane should carry enough information to establish:

```text
issuer
authority
deadline
sequence
epoch
resource
safety context
```

# 62. Stale Command Protection

Suppose:

```text
Command C42
epoch = 18
```

but the robot is now:

```text
epoch = 19
```

The command can be rejected:

```text
STALE_COMMAND
```

This protects against delayed network packets and obsolete plans.

# 63. Sequence Numbers

Commands may also carry:

```text
sequence_number
```

so the executor can reject:

```text
duplicate
reordered
replayed
```

messages when required.

# 64. Watchdogs

Safety-critical execution should support:

```text
watchdog
```

Example:

```text
Planner must renew control authority every 100 ms.
```

If renewal disappears:

```text
safe transition
```

is triggered.

# 65. Heartbeat ≠ Authority

A heartbeat only says:

> "I am alive."

It does not prove:

> "I am authorized."

NROS should keep these semantics separate.

# 66. Execution Lease

A stronger model:

```text
ExecutionLease
├── work
├── executor
├── authority
├── epoch
├── expiry
└── renewal
```

The executor must maintain the lease during execution.

# 67. Safe Stop

When authority or timing becomes invalid:

```text
Lease Lost
   ↓
Execution Policy
   ↓
SAFE_STOP
```

The safe-stop behavior is resource-specific.

For example:

```text
motor → controlled deceleration
arm → hold / retract
navigation → brake
camera → stop acquisition
```

# 68. Scheduler as a Policy Engine

The NROS scheduler is therefore not merely:

```text
queue → CPU
```

It evaluates:

```text
authority
priority
deadline
resource
energy
safety
health
dependencies
coordination
```

before choosing execution.

# 69. Unified Runtime Decision

A consequential execution decision can conceptually be modeled as:

```text
ExecutionAdmission {
    principal,
    capability,
    work,
    resources,
    temporal_contract,
    policy,
    coordination_state,
    safety_state
}
```

The result:

```text
ALLOW
DEFER
DENY
PREEMPT
DEGRADE
ESCALATE
```

# 70. ROS → NROS Transformation

The architectural evolution is now increasingly clear:

| ROS concept | NROS evolution |
|---|---|
| Node | Principal / Runtime |
| Topic | Typed Event/Data Channel |
| Service | Operation |
| Action | Intent → Work |
| Parameter Server | Configuration + State |
| roslaunch | Deployment/Orchestration |
| rosbag | Event/Evidence Journal |
| TF | Spatial/Temporal State |
| Package | Capability-bearing Component |
| Master/Discovery | Federated Discovery |
| Executor | Execution Runtime |
| Scheduler | Resource/Temporal Scheduler |

The goal is **not** to rename ROS APIs.

The goal is to change the underlying semantic model.

# 71. The NROS Execution Pipeline

We can now describe the complete pipeline:

```text
┌───────────────┐
│ Observation   │
└───────┬───────┘
        ↓
┌───────────────┐
│ Interpretation│
└───────┬───────┘
        ↓
┌───────────────┐
│ Intent        │
└───────┬───────┘
        ↓
┌───────────────┐
│ Planning      │
└───────┬───────┘
        ↓
┌───────────────┐
│ Authority     │
└───────┬───────┘
        ↓
┌───────────────┐
│ Coordination  │
└───────┬───────┘
        ↓
┌───────────────┐
│ Resources     │
└───────┬───────┘
        ↓
┌───────────────┐
│ Scheduling    │
└───────┬───────┘
        ↓
┌───────────────┐
│ Admission     │
└───────┬───────┘
        ↓
┌───────────────┐
│ Execution     │
└───────┬───────┘
        ↓
┌───────────────┐
│ Verification  │
└───────┬───────┘
        ↓
┌───────────────┐
│ Evidence      │
└───────┬───────┘
        ↓
┌───────────────┐
│ Replanning    │
└───────────────┘
```

This is beginning to look much less like a conventional middleware stack and much more like an **autonomous execution substrate**.

# 72. Sixteen-Fabric NROS Architecture

At this stage:

```text
┌───────────────────────────────────────────────┐
│ Domain & Deployment                           │
├───────────────────────────────────────────────┤
│ Identity, Trust & Security                    │
├───────────────────────────────────────────────┤
│ Coordination & Distributed Orchestration      │
├───────────────────────────────────────────────┤
│ Resource, Scheduling & Real-Time Execution     │
├───────────────────────────────────────────────┤
│ Intent & Planning                             │
├───────────────────────────────────────────────┤
│ Knowledge & State                             │
├───────────────────────────────────────────────┤
│ Memory, Event & Evidence                      │
├───────────────────────────────────────────────┤
│ Protocol & Type                               │
├───────────────────────────────────────────────┤
│ Observability                                 │
├───────────────────────────────────────────────┤
│ Capability & Authority                        │
├───────────────────────────────────────────────┤
│ Supervision                                   │
├───────────────────────────────────────────────┤
│ Temporal                                      │
├───────────────────────────────────────────────┤
│ Execution                                     │
├───────────────────────────────────────────────┤
│ Communication                                 │
└───────────────────────────────────────────────┘
```

# 73. The Next Major Problem

We now have:

```text
Identity
Authority
Coordination
Resources
Scheduling
Execution
Evidence
```

But a robot cannot reason over raw events forever.

It needs a **world model**.

It must answer:

```text
Where am I?
What exists?
What changed?
What do I know?
How certain am I?
What does another agent know?
Which state is authoritative?
What is merely predicted?
```

That leads directly to:

# Part LV — NROS World Model, Knowledge Graph & Belief-State Fabric

The next transition is:

```text
Raw Sensors
     ↓
Observations
     ↓
Perception
     ↓
Facts
     ↓
Entities
     ↓
Relations
     ↓
Beliefs
     ↓
World Model
     ↓
Planning
     ↓
Action
```

And this is where NROS can move from **event-driven robotics middleware** toward a runtime capable of maintaining a **persistent, uncertainty-aware model of the world**.

# NROS — Part LV: World Model, Knowledge Graph & Belief-State Fabric

The previous layer established **when and where work can execute**.

Now we need to establish **what the system believes exists and what it believes is happening**.

This is a fundamental transition:

```text
ROS:
    sensor messages → processing nodes

NROS:
    observations → evidence → beliefs → world state → decisions
```

The distinction is crucial because an observation is **not automatically truth**.

# 1. Observation ≠ Fact ≠ Belief

Consider a lidar observation:

```text
Observation:
    obstacle detected at (4.2, 1.8)
```

This does not necessarily mean:

```text
Fact:
    obstacle definitely exists at (4.2, 1.8)
```

The runtime may instead maintain:

```text
Belief:
    obstacle exists
    probability = 0.94
    observed_by = lidar/front
    timestamp = T
```

NROS should preserve these distinctions.

# 2. Three Knowledge Layers

A useful model is:

```text
Observation
     ↓
Fact
     ↓
Belief
```

### Observation

Something was measured.

### Fact

A proposition accepted as sufficiently established.

### Belief

A proposition currently considered likely, possibly uncertain or contradictory.

# 3. World Model

Introduce:

```text
WorldModel
```

Conceptually:

```text
WorldModel
├── entities
├── properties
├── relations
├── locations
├── events
├── beliefs
├── predictions
├── constraints
└── provenance
```

The world model is therefore not merely a database.

It is the runtime's structured representation of the environment.

# 4. Entity

Everything relevant to autonomy can become an entity:

```text
Robot
Human
Object
Room
Door
Road
Machine
Sensor
Resource
Mission
Region
Obstacle
```

Each gets:

```text
EntityId
```

# 5. Entity Identity

An object observed multiple times should ideally maintain identity:

```text
Observation 1
    ↓
Object #42

Observation 2
    ↓
Object #42

Observation 3
    ↓
Object #42
```

rather than creating:

```text
Object #42
Object #87
Object #113
```

for every observation.

This introduces **entity tracking**.

# 6. Entity Lifecycle

Entities can move through:

```text
UNKNOWN
DISCOVERED
ACTIVE
INACTIVE
LOST
REMOVED
MERGED
SPLIT
```

Example:

```text
Person #17
    ↓
detected
    ↓
tracked
    ↓
temporarily lost
    ↓
reacquired
```

The history should remain available.

# 7. Provenance

Every important world-model assertion should answer:

> Where did this information come from?

Example:

```text
Fact F42:
    door/7 = OPEN
```

with provenance:

```text
source:
    camera/front

observation:
    O8172

algorithm:
    vision-model-v4

confidence:
    0.96
```

# 8. Confidence

NROS should support uncertainty without forcing one mathematical representation.

Possible forms:

```text
probability
confidence
interval
distribution
qualitative confidence
unknown
```

The semantic model should allow uncertainty while the implementation chooses the appropriate representation.

# 9. Unknown Is a First-Class State

A dangerous design is:

```text
UNKNOWN → FALSE
```

because missing information becomes negative knowledge.

NROS should explicitly distinguish:

```text
TRUE
FALSE
UNKNOWN
UNCERTAIN
CONFLICTING
```

# 10. Contradictory Knowledge

Suppose:

```text
Camera:
    door = OPEN

Sensor:
    door = CLOSED
```

The system should not silently overwrite one with the other.

Instead:

```text
BeliefConflict
├── proposition
├── source A
├── source B
├── timestamps
├── confidence
└── resolution policy
```

# 11. Belief Resolution

Possible strategies:

```text
source priority
sensor reliability
recency
consensus
cross-validation
human confirmation
keep both hypotheses
```

The chosen strategy should be explicit.

# 12. World State Is Time-Dependent

An entity property is not timeless.

Example:

```text
door/7:
    OPEN at T1
    CLOSED at T2
    OPEN at T3
```

Therefore:

```text
WorldState(t)
```

is conceptually more accurate than a single mutable dictionary.

# 13. Temporal Validity

A fact can have:

```text
valid_from
valid_until
observed_at
recorded_at
```

These are not necessarily the same.

# 14. Event Time vs Processing Time

Suppose a sensor observes something at:

```text
10:00:00.100
```

but the message arrives at:

```text
10:00:00.250
```

NROS should preserve both:

```text
event_time
processing_time
```

This is important for causal reconstruction.

# 15. Spatial Knowledge

Robotics requires spatial relations:

```text
inside
outside
near
far
above
below
left_of
right_of
connected_to
blocked_by
```

Example:

```text
Robot A
    inside
Room 3
```

# 16. Coordinate Frames

The traditional ROS `tf/tf2` concept evolves into a broader spatial model:

```text
SpatialFrame
├── frame_id
├── parent
├── transform
├── validity
└── provenance
```

But NROS should allow more than pure coordinate transforms.

It should represent semantic spatial relations as well.

# 17. Spatial Graph

Example:

```text
World
│
├── Building
│    ├── Floor 1
│    │    ├── Room A
│    │    └── Room B
│    │
│    └── Floor 2
│
└── Outdoor Area
```

Entities become nodes.

Relations become edges.

This is a **world knowledge graph**.

# 18. Knowledge Graph

Conceptually:

```text
Entity
   │
   ├── property
   ├── relation
   ├── event
   └── belief
```

Example:

```text
Robot-01
   ├── located_in → Room-A
   ├── powered_by → Battery-01
   ├── carrying → Package-17
   └── controlled_by → Agent-4
```

# 19. Semantic Relationships

The graph can represent:

```text
located_in
contains
owns
controls
observes
depends_on
blocks
connected_to
assigned_to
carrying
approaching
```

This enables higher-level reasoning.

# 20. State Transition

Suppose:

```text
Package-17
    located_in Room-A
```

then robot picks it up.

The resulting state may be:

```text
Robot-01
    carrying Package-17
```

This should not merely be a changed database row.

It should produce:

```text
StateTransitionEvent
```

with provenance.

# 21. World Model as Derived State

This gives an important architecture:

```text
Events
   ↓
Projection
   ↓
World Model
```

The world model can therefore be reconstructed from authoritative event history.

This complements the Evidence Fabric.

# 22. Event-Sourced World Model

Conceptually:

```text
E1: RobotCreated
E2: RobotLocated
E3: PackageDetected
E4: PackagePickedUp
E5: RobotMoved
```

Projection:

```text
Current World State
```

This makes state reconstruction possible.

# 23. Multiple Projections

The same event history can generate different views:

```text
World Projection
Mission Projection
Resource Projection
Security Projection
Spatial Projection
Diagnostic Projection
```

This is powerful because NROS does not need one monolithic state database.

# 24. Agent-Specific Belief

Different agents may have different beliefs.

```text
Agent A:
    door = OPEN (0.9)

Agent B:
    door = CLOSED (0.7)
```

The global system should not necessarily force immediate convergence.

Instead:

```text
Belief(agent, proposition)
```

can be represented explicitly.

# 25. Shared Knowledge

Some facts become sufficiently authoritative to be shared:

```text
Canonical Map
Fleet Assignment
Safety State
Resource Ownership
```

These should have designated authorities.

# 26. Knowledge Authority

For each domain:

```text
Authority
    ↓
Canonical State
```

Example:

```text
Battery Controller
    → battery state

Fleet Coordinator
    → mission assignment

Localization System
    → robot pose estimate

Safety Controller
    → safety mode
```

# 27. Belief Fusion

Multiple observations can contribute to one belief:

```text
Camera ───┐
Lidar ────┼──→ Fusion → Belief
Radar ────┘
```

The fusion algorithm can be:

```text
Bayesian
Kalman
particle-based
rule-based
neural
custom
```

NROS should provide the semantic interface, not mandate the algorithm.

# 28. Belief Update

A belief update should record:

```text
BeliefUpdate
├── entity
├── proposition
├── previous belief
├── new belief
├── evidence
├── method
└── timestamp
```

This gives reasoning provenance.

# 29. Prediction

A world model should also represent predictions:

```text
Robot A
    predicted_position(t+1)
```

Prediction is different from observation.

So:

```text
Observed:
    position = X

Predicted:
    position ≈ Y
```

must remain separate.

# 30. Prediction Horizon

Predictions can have:

```text
horizon
confidence
model
assumptions
validity
```

Example:

```text
predicted:
    obstacle position in 2 seconds

confidence:
    0.81
```

# 31. Counterfactual State

Advanced planning may ask:

> What would the world look like if action A occurred?

NROS can model:

```text
Current World
      │
      ├── actual
      │
      ├── hypothetical A
      │
      └── hypothetical B
```

These branches must never be confused with actual state.

# 32. Simulation Branch

This creates a natural relationship with simulation:

```text
Actual World
     ↓
Snapshot
     ↓
Simulation Branch
     ↓
Plan Evaluation
```

The result:

```text
SimulationPrediction
```

can inform planning without becoming physical-world fact.

# 33. Belief State vs World State

This distinction should remain explicit:

```text
World State:
    what is actually the case

Belief State:
    what an agent/system currently believes
```

In reality the former may be unknowable with perfect certainty.

Therefore the runtime should avoid pretending its model is omniscient.

# 34. Epistemic State

NROS can represent:

```text
known
unknown
believed
suspected
predicted
confirmed
contradicted
```

This gives agents an explicit model of **what they know and do not know**.

# 35. Knowledge Gap

Suppose an agent needs:

```text
door state
```

but current knowledge is:

```text
UNKNOWN
```

The planner can generate an information-gathering task:

```text
Observe Door
```

before generating physical action.

This is a major transition.

# 36. Information as a Resource

A sensor observation has value.

Therefore:

```text
Information Gain
```

can become a planning objective.

Example:

```text
Option A:
    immediately move

Option B:
    spend 2 seconds scanning
    reduce uncertainty
```

The planner can trade time against knowledge.

# 37. Active Perception

The agent may intentionally act to obtain information:

```text
Move camera
     ↓
observe object
     ↓
update belief
     ↓
replan
```

Thus perception becomes part of action planning.

# 38. World Model API

A conceptual API could expose:

```text
query_entity()
query_property()
query_relation()
query_region()
query_belief()
query_history()
query_prediction()
subscribe_state()
```

But importantly, access should respect:

```text
identity
authorization
security domain
information sensitivity
```

# 39. Knowledge Subscriptions

An agent could subscribe to:

```text
"objects entering Room A"
```

rather than merely:

```text
/topic/object
```

This is a semantic subscription.

The underlying implementation could still use efficient event streams.

# 40. Semantic Query

NROS could support structured queries such as:

```text
Find:
    all robots
    inside Building-1
    with battery < 20%
    capable of transport
```

The result becomes actionable coordination information.

# 41. Knowledge → Coordination

This connects the layers:

```text
World Model
     ↓
Resource State
     ↓
Coordination
     ↓
Planning
```

Example:

```text
Three robots detected.

Robot A:
    battery 12%

Robot B:
    battery 78%

Robot C:
    battery 51%
```

The planner may select B.

# 42. Knowledge → Safety

Similarly:

```text
Human detected
    ↓
restricted-zone belief
    ↓
safety constraint
    ↓
planner update
    ↓
motion restriction
```

The world model therefore directly influences safety.

# 43. Knowledge → Authority

Context can also modify authorization:

```text
Robot enters maintenance zone
       ↓
environment state changes
       ↓
policy condition changes
       ↓
capability becomes valid/invalid
```

This is why security cannot be isolated from world state.

# 44. Knowledge → Scheduling

Resource conditions can affect scheduling:

```text
GPU overheating
      ↓
resource degraded
      ↓
scheduler migrates workload
```

Again, the fabrics form a connected system.

# 45. Knowledge → Evidence

Every important belief should retain provenance:

```text
Belief
   ↓
Evidence
   ↓
Observation
   ↓
Source
```

This enables:

> "Why does NROS believe the door is closed?"

to receive a concrete answer.

# 46. Explainable State

NROS should be able to answer:

```text
Why is robot R considered unavailable?
```

For example:

```text
battery < threshold
AND
charger reservation exists
AND
maintenance capability active
```

The result is an **explanation graph**, not merely a status code.

# 47. Knowledge Conflict

If two authoritative systems disagree:

```text
Localization:
    Robot = Room A

Map authority:
    Robot = Room B
```

NROS should create:

```text
KnowledgeConflict
```

rather than silently selecting whichever message arrived last.

# 48. Conflict Resolution

Resolution can use:

```text
authority hierarchy
confidence
recency
cross-validation
temporal consistency
sensor health
human intervention
```

The chosen method must be observable.

# 49. Knowledge Expiration

Some knowledge becomes stale:

```text
door state
traffic state
battery estimate
human location
```

A fact can therefore have:

```text
freshness requirement
```

Example:

```text
HumanLocation:
    valid for 500 ms
```

After that:

```text
STALE
```

rather than:

```text
TRUE forever
```

# 50. Staleness Is Different From Falsehood

Important distinction:

```text
STALE
```

means:

> "We don't know whether this remains true."

It does not mean:

```text
FALSE
```

This prevents dangerous reasoning from outdated information.

# 51. World Model Consistency

NROS can define invariants such as:

```text
A resource cannot be simultaneously allocated exclusively
to two principals.

An object cannot simultaneously occupy mutually exclusive
locations under the same world-state version.

A revoked authority cannot create new authorized work.

A hypothetical state cannot become actual state without
an explicit transition.
```

These become runtime verification rules.

# 52. World Model Verification

The runtime can continuously evaluate:

```text
Invariant
    ↓
Valid
    or
Violation
```

Violation generates:

```text
WorldModelInvariantViolation
```

which can trigger safety or recovery mechanisms.

# 53. Persistent World Model

NROS should support persistence across restarts.

After restart:

```text
Runtime lost
```

does not necessarily mean:

```text
World knowledge lost
```

The system can reconstruct:

```text
events
+
snapshots
+
authoritative state
```

to recover the world model.

# 54. Snapshot + Event Log

For efficiency:

```text
Event Log:
    E1 ... E1000000

Snapshot:
    State at E999000
```

Recovery:

```text
Snapshot
   ↓
Replay E999001...E1000000
   ↓
Current World Model
```

This integrates directly with the Evidence/Event Fabric.

# 55. World Model as NROS Memory

The Memory Fabric can therefore be divided conceptually:

```text
Episodic Memory
    what happened

Semantic Memory
    what things mean

World State
    what is currently believed

Procedural Knowledge
    how to perform operations
```

This becomes particularly important for agentic systems.

# 56. The Agentic Loop

The complete NROS agent loop now becomes:

```text
Observe
   ↓
Update Beliefs
   ↓
Update World Model
   ↓
Detect Knowledge Gaps
   ↓
Form Intent
   ↓
Plan
   ↓
Check Authority
   ↓
Coordinate
   ↓
Acquire Resources
   ↓
Schedule
   ↓
Execute
   ↓
Observe Outcome
   ↓
Verify
   ↓
Record Evidence
   ↓
Update World Model
   ↓
Repeat
```

This is substantially richer than:

```text
callback → callback → callback
```

# 57. Seventeen-Fabric Architecture

NROS now has a coherent stack:

```text
┌──────────────────────────────────────────────┐
│ Domain & Deployment                          │
├──────────────────────────────────────────────┤
│ Identity, Trust & Security                   │
├──────────────────────────────────────────────┤
│ Coordination & Distributed Orchestration     │
├──────────────────────────────────────────────┤
│ Resource, Scheduling & Real-Time Execution    │
├──────────────────────────────────────────────┤
│ World Model, Knowledge & Belief               │
├──────────────────────────────────────────────┤
│ Intent & Planning                            │
├──────────────────────────────────────────────┤
│ Knowledge & State                             │
├──────────────────────────────────────────────┤
│ Memory, Event & Evidence                      │
├──────────────────────────────────────────────┤
│ Protocol & Type                               │
├──────────────────────────────────────────────┤
│ Observability                                 │
├──────────────────────────────────────────────┤
│ Capability & Authority                        │
├──────────────────────────────────────────────┤
│ Supervision                                   │
├──────────────────────────────────────────────┤
│ Temporal                                      │
├──────────────────────────────────────────────┤
│ Execution                                     │
├──────────────────────────────────────────────┤
│ Communication                                 │
└──────────────────────────────────────────────┘
```

There is some conceptual overlap between **World Model/Knowledge/Belief** and the earlier **Knowledge & State** fabric. That should eventually be normalized into a precise NROS architecture rather than duplicated.

# 58. The Deeper Transformation

At this point, the transformation can be summarized as:

```text
ROS
│
├── Nodes
├── Topics
├── Services
├── Parameters
├── Actions
└── Packages
        ↓
NROS
│
├── Principals
├── Intent
├── Goals
├── Beliefs
├── World State
├── Capabilities
├── Commitments
├── Resources
├── Schedules
├── Work
├── Evidence
└── Policies
```

The fundamental runtime object is no longer merely a **node**.

It is an **authorized autonomous participant operating over a changing world**.

# 59. The Next Problem

The World Model tells NROS:

> **What does the system currently believe about the world?**

But an autonomous system still needs to answer:

> **What should it do next?**

That brings us to the central intelligence layer:

# Part LVI — NROS Intent, Planning, Policy & Decision Fabric

The progression becomes:

```text
World State
     ↓
Beliefs
     ↓
Goals
     ↓
Constraints
     ↓
Policies
     ↓
Candidate Plans
     ↓
Evaluation
     ↓
Decision
     ↓
Commitment
     ↓
Execution
```

The critical architectural question will be:

> **How can NROS incorporate classical planners, behavior trees, state machines, optimization, probabilistic reasoning, and modern AI/LLM agents without making any one of them the definition of the runtime itself?**

# NROS — Part LVI: Intent, Planning, Policy & Decision Fabric

The previous part established the **World Model**:

```text
What exists?
What happened?
What is believed?
What is uncertain?
What is predicted?
```

Now NROS must answer:

```text
What should we do?
Why?
Under which constraints?
With what authority?
What happens if the plan fails?
```

This is the **Decision Fabric**.

The central design principle is:

> **NROS should provide a common execution semantics for decisions, without requiring a single planning algorithm, AI model, or agent framework.**

# 1. Decision Is Not Execution

A crucial separation:

```text
Decision
   ↓
Plan
   ↓
Commitment
   ↓
Work
   ↓
Execution
```

A planner saying:

```text
"Move to Room B"
```

does not mean the robot has permission to move.

It means a candidate decision exists.

# 2. Intent

Introduce:

```text
Intent
```

An Intent represents a desired state or outcome.

Examples:

```text
Reach(Room-B)

Deliver(Package-17, Dock-4)

Inspect(Machine-7)

Charge(Robot-3)

Avoid(Restricted-Zone)

Maintain(Battery > 30%)
```

Intent is therefore higher-level than an individual command.

# 3. Intent ≠ Goal

A useful distinction:

```text
Intent:
    desired direction/outcome

Goal:
    formally evaluable condition
```

For example:

```text
Intent:
    "Deliver package."

Goal:
    package/17.location == destination/4
```

The goal gives the runtime something that can be evaluated.

# 4. Goal Representation

A Goal can contain:

```text
Goal
├── condition
├── priority
├── deadline
├── owner
├── authority
├── constraints
├── success criteria
└── failure criteria
```

Example:

```text
Goal G42

condition:
    Package17 at Dock4

deadline:
    15:00

priority:
    mission-critical
```

# 5. Goal Lifecycle

A goal can move through:

```text
PROPOSED
ADMITTED
ACTIVE
SATISFIED
FAILED
CANCELLED
EXPIRED
SUPERSEDED
```

This prevents the system from treating goals as immutable strings.

# 6. Goal Ownership

Every operational goal should have an owner:

```text
owner = Mission-Controller
```

or:

```text
owner = Agent-17
```

Ownership establishes responsibility.

# 7. Goal Authority

Ownership alone does not imply permission.

For example:

```text
Agent A
    owns Goal G

Agent A
    lacks capability to execute G
```

Therefore:

```text
Goal ownership
≠
execution authority
```

# 8. Constraints

Planning does not occur in a vacuum.

A goal can have:

```text
Constraints
├── safety
├── temporal
├── resource
├── spatial
├── energy
├── authorization
├── environmental
└── mission
```

Example:

```text
Deliver Package17
```

subject to:

```text
battery >= 20%
human-free corridor
deadline < 15:00
payload secured
```

# 9. Hard vs Soft Constraints

NROS should distinguish:

```text
Hard Constraint
    must never be violated

Soft Constraint
    should preferably be satisfied
```

Example:

```text
Hard:
    never enter restricted zone

Soft:
    minimize travel distance
```

# 10. Policy

Policy determines how decisions are evaluated.

A policy can define:

```text
IF
    battery < 15%
AND
    charger available

THEN
    charging has priority
```

Policy is therefore different from planning.

# 11. Policy ≠ Plan

A plan says:

```text
how to achieve a goal
```

A policy says:

```text
which decisions are acceptable/preferred
```

This distinction allows multiple planners to operate under common system rules.

# 12. Planner Independence

NROS should support:

```text
Classical Planner
Behavior Tree
Finite State Machine
Task Planner
Motion Planner
Optimization Solver
Probabilistic Planner
Neural Policy
LLM Agent
Human Operator
```

without making any of them mandatory.

# 13. Planner Contract

Every planner should expose something conceptually like:

```text
Planner
├── accepts
│   ├── world state
│   ├── goals
│   ├── constraints
│   └── capabilities
│
└── produces
    ├── candidate plan
    └── rationale/evidence
```

# 14. Candidate Plan

A planner does not directly modify the world.

It produces:

```text
CandidatePlan
```

Example:

```text
Plan P7

1. Navigate → Room B
2. Acquire → Package17
3. Navigate → Dock4
4. Release → Package17
```

The plan is still hypothetical.

# 15. Plan Validation

Before execution:

```text
Candidate Plan
      ↓
Validation
```

Check:

```text
authority
resources
constraints
timing
safety
dependencies
world-state assumptions
```

Only then can the plan become executable.

# 16. Plan Status

A plan can be:

```text
DRAFT
CANDIDATE
VALIDATED
AUTHORIZED
COMMITTED
EXECUTING
COMPLETED
FAILED
INVALIDATED
CANCELLED
```

This creates a strong lifecycle.

# 17. Plan Assumptions

A plan should explicitly state what it assumes.

Example:

```text
Plan P7 assumptions:

door/4 = OPEN
battery > 30%
corridor/A = CLEAR
robot localization error < 20 cm
```

This is extremely important.

# 18. Plan Invalidation

If:

```text
corridor/A = BLOCKED
```

then the plan may become:

```text
INVALIDATED
```

without necessarily meaning that the goal failed.

The planner can generate a new plan.

# 19. Replanning

NROS should treat replanning as normal:

```text
Plan A
   ↓
World changes
   ↓
Plan invalidated
   ↓
Replan
   ↓
Plan B
```

This is fundamental for robotics.

# 20. Plan Versioning

Every plan should have a version:

```text
Goal G42

Plan 1
Plan 2
Plan 3
```

and the runtime should preserve relationships:

```text
Plan 2 supersedes Plan 1
Plan 3 supersedes Plan 2
```

# 21. Decision Record

A consequential decision should generate:

```text
DecisionRecord
├── decision_id
├── goal
├── selected_plan
├── rejected_alternatives
├── decision_policy
├── world_state_version
├── authority
├── planner
└── evidence
```

This is where NROS becomes explainable.

# 22. Why Was This Plan Selected?

NROS should be able to answer:

```text
Why Plan B?
```

Example:

```text
Plan A:
    shortest
    but violates energy constraint

Plan B:
    slightly longer
    satisfies deadline
    satisfies battery constraint

Plan C:
    unsafe under current world state
```

Therefore:

```text
Plan B selected.
```

# 23. Decision Score

A planner may evaluate candidates using:

```text
utility
cost
risk
energy
time
distance
confidence
```

Conceptually:

```text
Score(plan) =
    utility
    - cost
    - risk
    - energy
    - delay
```

NROS should not prescribe the equation.

It should provide the semantic container for the result.

# 24. Multi-Objective Planning

Robotics frequently requires simultaneous optimization:

```text
minimize:
    energy
    time
    risk
    distance

maximize:
    reliability
    information
    mission value
```

Different deployments can use different optimization strategies.

# 25. Risk

Risk deserves explicit representation.

A candidate action may have:

```text
probability_of_failure
severity
uncertainty
recoverability
```

The system can then distinguish:

```text
low probability + low consequence
```

from:

```text
low probability + catastrophic consequence
```

# 26. Safety Policy

A safety policy can override optimization.

For example:

```text
Planner:
    "Shortest path through corridor X."

Safety:
    "Human detected in corridor X."

Decision:
    REJECT
```

Thus:

```text
Optimization
      ↓
Safety Gate
```

not the reverse.

# 27. Decision Gate

A robust NROS decision pipeline:

```text
Candidate Plan
      ↓
Policy Check
      ↓
Authority Check
      ↓
Safety Check
      ↓
Resource Check
      ↓
Temporal Check
      ↓
World-State Check
      ↓
Decision
```

# 28. Commitment

Once a plan is accepted, the system can create:

```text
Commitment
```

A Commitment means:

> The system has accepted responsibility to attempt a specified outcome under defined conditions.

This is stronger than a plan.

# 29. Plan vs Commitment

```text
Plan:
    "This is how we could do it."

Commitment:
    "We have accepted this course of action."
```

This distinction becomes extremely valuable in multi-agent systems.

# 30. Commitment Structure

Conceptually:

```text
Commitment
├── commitment_id
├── principal
├── goal
├── plan
├── conditions
├── deadline
├── resources
├── authority
├── obligations
└── cancellation policy
```

# 31. Commitment Failure

If the system cannot continue:

```text
Commitment
    ↓
Failure
```

it should explicitly record:

```text
CommitmentFailed
```

rather than silently disappearing.

# 32. Commitment Renegotiation

In distributed robotics:

```text
Robot A:
    "I will deliver Package17."

Robot A:
    battery failure

Robot A:
    "I can no longer fulfill commitment."

Fleet:
    assign Robot B
```

This is not merely task retry.

It is **commitment renegotiation**.

# 33. Multi-Agent Planning

Now consider:

```text
Goal:
    transport package
```

Possible allocation:

```text
Robot A:
    perception

Robot B:
    transport

Robot C:
    docking
```

The resulting plan becomes distributed.

# 34. Joint Plan

NROS can represent:

```text
JointPlan
├── participant A
├── participant B
├── participant C
├── shared dependencies
├── synchronization points
└── failure policies
```

# 35. Synchronization Point

Example:

```text
Robot A
    reaches pickup location
          ↓
Robot B
    begins loading
```

The transition creates:

```text
Barrier / Synchronization
```

rather than relying on timing guesses.

# 36. Negotiated Planning

Agents can negotiate:

```text
A:
    "I need charger 4."

B:
    "I need charger 4 first."

Coordinator:
    compare deadlines and energy

Result:
    A gets charger 4
    B receives charger 2
```

The decision should become an explicit coordination artifact.

# 37. Human-in-the-Loop Decision

Not every decision should be fully autonomous.

A policy may require:

```text
HumanApprovalRequired
```

for:

```text
high-risk action
irreversible operation
restricted-area access
large resource commitment
```

The plan pauses at:

```text
WAITING_FOR_AUTHORIZATION
```

# 38. AI/LLM Integration

This architecture allows an LLM agent to participate safely.

The LLM can produce:

```text
Intent
CandidatePlan
Hypothesis
Explanation
```

but it does not directly gain:

```text
motor control
```

The NROS decision gates remain authoritative.

# 39. LLM as Planner

For example:

```text
LLM Agent
   ↓
"Inspect machine 7."
   ↓
Intent
   ↓
Planner
   ↓
Candidate Plan
   ↓
Validation
   ↓
Authority
   ↓
Execution
```

The model becomes a **planning participant**, not the runtime itself.

# 40. Tool Use

An agent may request:

```text
inspect_machine()
get_robot_pose()
reserve_camera()
navigate()
```

These are represented as capabilities and operations.

The agent does not bypass the capability system.

# 41. Planning Context

A planner should receive a bounded context:

```text
PlanningContext
├── relevant world state
├── beliefs
├── active goals
├── policies
├── capabilities
├── available resources
├── temporal constraints
└── historical evidence
```

This avoids blindly exposing the entire runtime state.

# 42. Planning Context Version

Every plan should record:

```text
world_version = 812
policy_version = 17
capability_snapshot = 42
```

Then NROS can determine whether the plan is still valid.

# 43. Stale Plan Detection

If the world becomes:

```text
version 813
```

while a plan was created against:

```text
version 812
```

the runtime evaluates whether the difference is relevant.

Possible outcomes:

```text
STILL_VALID
REVALIDATE
INVALID
```

# 44. Incremental Replanning

Not every change requires complete replanning.

Example:

```text
Plan:
A → B → C → D → E
```

If only the path B→C becomes blocked:

```text
A → B → [replan] → D → E
```

NROS should permit partial plan replacement.

# 45. Hierarchical Planning

A mission can be decomposed:

```text
Mission
 ├── Task A
 │    ├── Action A1
 │    └── Action A2
 │
 ├── Task B
 │    ├── Action B1
 │    └── Action B2
 │
 └── Task C
```

This naturally maps onto NROS:

```text
Goal
 ↓
Work Graph
 ↓
Execution Graph
```

# 46. Work Graph

A Plan can compile into a graph:

```text
A ──→ B ──→ C
      │
      └──→ D
```

where nodes are Work items and edges represent:

```text
dependency
temporal ordering
resource dependency
data dependency
condition
```

# 47. Conditional Execution

Example:

```text
IF object_detected
    inspect
ELSE
    search
```

This should be represented explicitly.

Not hidden inside arbitrary application code.

# 48. Retry Semantics

A Work item can declare:

```text
retry_policy
```

such as:

```text
max_attempts = 3
backoff = exponential
retry_on = transient_network_failure
```

But retrying an irreversible physical operation may be forbidden.

# 49. Idempotence

NROS should know whether an operation is:

```text
idempotent
non-idempotent
conditionally-idempotent
irreversible
```

Example:

```text
read_sensor()
```

is usually safe to retry.

But:

```text
release_gripper()
```

may not be.

# 50. Compensation

For operations that cannot simply be rolled back, the system can define:

```text
Compensation
```

Example:

```text
ReserveResource
    ↓
Work fails
    ↓
ReleaseResource
```

or:

```text
MoveObject
    ↓
Later failure
    ↓
Attempt return-to-safe-position
```

# 51. Saga-Like Execution

A complex mission can therefore use:

```text
Step A
  ↓
Step B
  ↓
Step C
```

with compensating actions:

```text
Compensation C'
Compensation B'
Compensation A'
```

if recovery is required.

This is useful for long-running physical workflows.

# 52. Decision + Evidence

Every consequential decision should create evidence:

```text
Goal
 ↓
Candidate Plans
 ↓
Evaluation
 ↓
Policy Checks
 ↓
Decision
 ↓
Commitment
```

The evidence chain allows later reconstruction:

```text
"What did the robot decide?"

"Why?"

"Using which world state?"

"Under which policy?"

"With which authority?"
```

# 53. Decision Provenance

A complete provenance chain can look like:

```text
Observation O
   ↓
Belief B
   ↓
World State W
   ↓
Goal G
   ↓
Plan P
   ↓
Policy Evaluation
   ↓
Decision D
   ↓
Commitment C
   ↓
Work Wk
   ↓
Execution E
   ↓
Outcome O'
```

This is one of the strongest differentiators of NROS.

# 54. The NROS Cognitive Loop

The architecture now supports:

```text
OBSERVE
   ↓
BELIEVE
   ↓
UNDERSTAND
   ↓
INTEND
   ↓
PLAN
   ↓
DECIDE
   ↓
COMMIT
   ↓
ACT
   ↓
VERIFY
   ↓
LEARN / UPDATE
```

This is much closer to an **agent-native robotics runtime** than traditional message middleware.

# 55. Eighteen-Fabric View

The architecture is converging toward:

```text
┌──────────────────────────────────────────────┐
│ Domain & Deployment                          │
├──────────────────────────────────────────────┤
│ Identity / Trust / Security                  │
├──────────────────────────────────────────────┤
│ Coordination / Multi-Agent                  │
├──────────────────────────────────────────────┤
│ Resource / Scheduling / Real-Time            │
├──────────────────────────────────────────────┤
│ World Model / Knowledge / Belief             │
├──────────────────────────────────────────────┤
│ Intent / Planning / Decision                 │
├──────────────────────────────────────────────┤
│ Memory / Event / Evidence                    │
├──────────────────────────────────────────────┤
│ Capability / Authority                       │
├──────────────────────────────────────────────┤
│ Supervision / Recovery                       │
├──────────────────────────────────────────────┤
│ Temporal                                    │
├──────────────────────────────────────────────┤
│ Execution                                   │
├──────────────────────────────────────────────┤
│ Communication / Protocol / Types             │
└──────────────────────────────────────────────┘
```

The exact number of "fabrics" should eventually be normalized. What matters now is the semantic separation.

# 56. The Core NROS Transformation

We can now state the architectural shift more precisely:

```text
ROS:
    Communication-centered

NROS:
    State + Intent + Authority + Decision +
    Coordination + Resource + Execution-centered
```

Or even more compactly:

```text
ROS = distributed robotics middleware

NROS = autonomous execution substrate
```

# 57. The Next Missing Piece

There is still one major question.

NROS can now:

```text
observe
believe
plan
decide
authorize
coordinate
schedule
execute
verify
```

But what happens when something goes wrong?

For a serious autonomous runtime, failure is not an exceptional afterthought.

It is a **first-class execution condition**.

Therefore the next layer is:

# Part LVII — NROS Supervision, Fault Model, Recovery & Resilience Fabric

The next progression is:

```text
Failure
   ↓
Detection
   ↓
Classification
   ↓
Containment
   ↓
Recovery
   ↓
Verification
   ↓
Replanning
   ↓
Resumption
```

with explicit treatment of:

```text
process crashes
hardware failures
network partitions
stale state
lost agents
resource exhaustion
deadline violations
authority revocation
sensor disagreement
planner failure
unsafe physical states
partial mission completion
distributed recovery
```

That layer is what turns NROS from an **agent-capable architecture** into a **fault-tolerant autonomous runtime**.

# NROS — Part LVII: Supervision, Fault Model, Recovery & Resilience Fabric

The previous layer gave NROS the ability to:

```text
Observe
→ Believe
→ Intend
→ Plan
→ Decide
→ Commit
→ Execute
→ Verify
```

But physical systems operate under failure.

A robot can lose:

- a sensor,
- a motor,
- a network link,
- a process,
- a CPU,
- a power source,
- localization,
- an authorization lease,
- a planner,
- another robot,
- or confidence in its own world model.

Therefore:

> **Failure must be part of the normal NROS state machine, not an exception path.**

# 1. Failure Is a First-Class Event

Traditional application logic often treats failure as:

```text
error
```

NROS needs a richer model:

```text
Failure
├── what failed?
├── when?
├── where?
├── why?
├── severity?
├── affected resources?
├── affected commitments?
├── affected goals?
├── containment?
└── recovery strategy?
```

# 2. Fault vs Failure

These should not be conflated.

```text
Fault:
    abnormal condition exists.

Failure:
    required behavior can no longer be provided.
```

Example:

```text
Camera temperature high
```

may be a:

```text
FAULT
```

while:

```text
Perception pipeline can no longer produce valid observations
```

is a:

```text
FAILURE
```

# 3. Fault Lifecycle

A fault can move through:

```text
DETECTED
→ CLASSIFIED
→ ACKNOWLEDGED
→ CONTAINED
→ RECOVERING
→ VERIFIED
→ CLEARED
```

or:

```text
DETECTED
→ UNRECOVERABLE
→ ISOLATED
```

# 4. Fault Identity

Every significant fault gets:

```text
FaultId
```

Example:

```text
fault://robot-07/motor-left/2026-08-21/1842
```

The identity allows all downstream evidence to reference the same incident.

# 5. Fault Classification

NROS should distinguish at least:

```text
TRANSIENT
INTERMITTENT
PERSISTENT
PERMANENT
UNKNOWN
```

Example:

```text
network timeout
    → TRANSIENT

loose connector
    → INTERMITTENT

dead motor
    → PERMANENT
```

# 6. Fault Domain

Failures may originate in:

```text
Software
Hardware
Network
Power
Environment
Human
Configuration
Security
Planning
Coordination
Timing
Knowledge
```

This classification is useful for automated recovery.

# 7. Failure Severity

A simple model:

```text
INFO
WARNING
DEGRADED
CRITICAL
EMERGENCY
```

But severity should describe **impact**, not merely technical abnormality.

# 8. Fault Scope

A fault may affect:

```text
component
process
node
robot
fleet
mission
domain
entire deployment
```

For example:

```text
camera failure
```

might affect only:

```text
perception
```

while:

```text
network partition
```

could affect:

```text
fleet coordination
```

# 9. Fault Propagation

NROS should explicitly model impact:

```text
Sensor Failure
      ↓
Perception Degraded
      ↓
World Model Uncertain
      ↓
Planner Invalidated
      ↓
Mission Delayed
```

This is a **fault propagation graph**.

# 10. Dependency Graph

Every Work or component can declare dependencies:

```text
Navigation
├── localization
├── map
├── lidar
├── planner
└── actuator
```

If:

```text
localization = FAILED
```

NROS can calculate affected functionality.

# 11. Blast Radius

For every failure:

```text
BlastRadius(Fault)
```

should answer:

```text
What becomes unsafe?
What becomes unavailable?
Which commitments are affected?
Which goals become impossible?
Which resources must be quarantined?
```

# 12. Containment

Before recovery, NROS may need to contain the fault.

Examples:

```text
disable component
isolate network link
revoke capability
quarantine resource
stop affected Work
freeze plan
switch operating mode
```

Containment prevents a local failure from becoming systemic.

# 13. Quarantine

A resource can enter:

```text
QUARANTINED
```

meaning:

> It may physically exist, but the runtime must not allocate it normally.

Example:

```text
motor/left
    → suspicious behavior
    → QUARANTINED
```

The scheduler must exclude it.

# 14. Fault Containment Region

A deployment can define:

```text
FaultContainmentRegion
```

such as:

```text
Robot-07
├── perception
├── navigation
├── manipulation
└── telemetry
```

A perception failure should not automatically terminate the entire robot runtime.

# 15. Graceful Degradation

A resilient robot should not have only:

```text
WORKING
FAILED
```

Instead:

```text
FULL
DEGRADED
MINIMAL
SAFE
EMERGENCY
```

Example:

```text
Full navigation:
    lidar + camera + localization

Degraded navigation:
    lidar + odometry

Minimal mode:
    controlled stop

Emergency:
    actuator-safe state
```

# 16. Capability Degradation

A capability can lose quality without disappearing:

```text
Navigation:
    nominal = 100%

Navigation:
    degraded = 60%
```

This should be represented explicitly.

# 17. Functional Redundancy

If:

```text
camera/front
```

fails:

```text
camera/rear
```

might partially compensate.

NROS should represent:

```text
Capability A
    provided_by:
        Resource X
        Resource Y
```

with different quality levels.

# 18. Redundancy Classes

Resources can be classified:

```text
PRIMARY
SECONDARY
REDUNDANT
HOT_STANDBY
COLD_STANDBY
```

This allows deterministic failover policies.

# 19. Failover

Example:

```text
Primary Localization
        ↓
failure
        ↓
Secondary Localization
        ↓
validation
        ↓
continue
```

Failover itself should produce evidence.

# 20. Recovery Is Not Restart

This is important.

A process restart is only one recovery mechanism.

NROS recovery strategies may include:

```text
RETRY
RESTART
RESET
FAILOVER
REPLAN
ROLLBACK
COMPENSATE
DEGRADE
ISOLATE
ESCALATE
ABORT
```

# 21. Recovery Policy

Each failure class can map to a recovery policy:

```text
Fault
  ↓
Policy Lookup
  ↓
Recovery Strategy
```

Example:

```text
temporary network timeout
    → retry

sensor corruption
    → isolate + failover

unsafe actuator
    → immediate safe stop
```

# 22. Recovery Attempt

Every attempt gets an identity:

```text
RecoveryAttemptId
```

with:

```text
fault
strategy
start
end
result
evidence
```

This prevents invisible recovery behavior.

# 23. Recovery State Machine

```text
FAULT
  ↓
ASSESS
  ↓
CONTAIN
  ↓
RECOVER
  ↓
VERIFY
  ├── SUCCESS → RESUME
  │
  └── FAILURE → ESCALATE
```

# 24. Verification Is Mandatory

Recovery should never mean:

```text
"I restarted it, therefore it works."
```

Instead:

```text
Recovery
   ↓
Health Check
   ↓
Functional Check
   ↓
Safety Check
   ↓
Restore Capability
```

# 25. Health vs Functionality

A process can be alive while being useless.

For example:

```text
camera process = running
```

but:

```text
camera frames = corrupted
```

Therefore NROS needs:

```text
liveness
readiness
health
functional validity
```

as distinct concepts.

# 26. Supervision

The **Supervisor** watches runtime entities.

Conceptually:

```text
Supervisor
├── processes
├── resources
├── capabilities
├── commitments
├── goals
├── plans
└── safety state
```

It detects abnormal transitions and applies policy.

# 27. Supervisor Hierarchy

Similar to scheduling:

```text
Fleet Supervisor
      ↓
Robot Supervisor
      ↓
Subsystem Supervisor
      ↓
Component Supervisor
```

Each level can contain failures locally.

# 28. Supervision Tree

A useful structure:

```text
Robot-01
│
├── Sensors
│   ├── Camera
│   └── Lidar
│
├── Perception
│
├── Localization
│
├── Planning
│
├── Control
│
└── Communications
```

If:

```text
Camera
```

fails, the supervisor decides whether only Camera should restart or whether a larger subtree must be affected.

# 29. Parent Responsibility

A supervisor should own the lifecycle of its children:

```text
Parent
  ├── start
  ├── monitor
  ├── recover
  └── stop
```

This creates deterministic lifecycle semantics.

# 30. Restart Strategies

NROS should support:

```text
restart immediately
restart with backoff
restart N times
restart only after dependency recovery
never restart
```

Example:

```text
attempt 1 → 1 s
attempt 2 → 2 s
attempt 3 → 4 s
```

# 31. Restart Storm Prevention

Distributed systems can accidentally create:

```text
failure
→ restart
→ failure
→ restart
→ failure
→ ...
```

NROS should detect this pattern.

Possible policy:

```text
max_attempts
cooldown
circuit_breaker
escalation
```

# 32. Circuit Breaker

A failing dependency can be temporarily disabled:

```text
NORMAL
  ↓ repeated failures
OPEN
  ↓ cooldown
HALF_OPEN
  ↓ successful test
CLOSED
```

This prevents continuous resource consumption.

# 33. Recovery Budget

Recovery itself consumes:

```text
time
CPU
energy
network
human attention
```

Therefore recovery should have budgets.

Example:

```text
Recovery budget:
    3 attempts
    30 seconds
```

After exhaustion:

```text
ESCALATE
```

# 34. Mission-Aware Recovery

Recovery should understand mission context.

Example:

```text
Mission:
    deliver medicine
```

A localization fault might justify:

```text
switch localization
```

whereas during:

```text
routine mapping
```

the system could simply:

```text
pause mission
```

Recovery is therefore contextual.

# 35. Commitment-Aware Recovery

Suppose:

```text
Robot A
    committed to Package17 delivery
```

and fails.

NROS must determine:

```text
Can another robot assume the commitment?
```

This creates:

```text
Commitment Transfer
```

# 36. Commitment Transfer

```text
Robot A
   ↓ failure
Commitment C42
   ↓
Fleet Coordinator
   ↓
Robot B
   ↓
new execution lease
```

The commitment history remains intact.

# 37. Partial Completion

Physical missions cannot always roll back.

Example:

```text
Package moved from A → B
```

then Robot failure.

The correct state is:

```text
Package is at B
```

not:

```text
Package operation failed → pretend nothing happened
```

Therefore recovery must account for **physical side effects**.

# 38. Physical Transaction

A physical operation can be modeled as:

```text
Intent
 ↓
Preconditions
 ↓
Action
 ↓
Physical Effects
 ↓
Verification
```

If failure occurs after the physical effect:

```text
World Model
```

must be updated before recovery planning continues.

# 39. Recovery from Reality

This leads to a key invariant:

> **Recovery must begin from observed physical state, not from assumed software state.**

Example:

```text
Software:
    "gripper closed"

Sensor:
    "gripper actually open"
```

Recovery trusts verified physical evidence according to authority policy.

# 40. Network Partition

Distributed robotics introduces another major fault:

```text
Robot A  X  Robot B
```

The communication channel disappears.

NROS should distinguish:

```text
temporary partition
persistent partition
unknown connectivity
```

rather than treating every missed packet as total failure.

# 41. Partition Mode

A robot may enter:

```text
CONNECTED
DEGRADED_CONNECTED
PARTITIONED
RECOVERING
```

During partition:

```text
local authority
cached policy
local safety
local commitments
```

may continue operating.

# 42. Autonomous Continuation

The robot needs an explicit policy:

```text
ON_PARTITION:
    continue
    pause
    return-home
    safe-stop
    complete-current-action
```

This must never be implicit.

# 43. Split-Brain Prevention

Two disconnected coordinators might both believe they own:

```text
charger/4
```

or command:

```text
robot/7
```

NROS therefore needs:

```text
epochs
leases
fencing
authority terms
```

to prevent stale control.

# 44. Fencing

A previous controller may hold:

```text
epoch = 12
```

while a new controller receives:

```text
epoch = 13
```

Commands carrying epoch 12 become invalid.

This is especially important for physical control.

# 45. Recovery Epoch

Every major recovery can advance an execution epoch:

```text
epoch 41
   ↓ fault
epoch 42
```

Old Work and commands can then be rejected automatically.

# 46. Temporal Faults

Not all failures are hardware failures.

Examples:

```text
deadline missed
clock uncertainty
timestamp drift
jitter exceeded
lease expired
plan stale
```

These belong to the fault model.

# 47. Knowledge Faults

The world model can also fail.

Examples:

```text
localization divergence
sensor contradiction
stale map
inconsistent entity identity
unknown object classification
```

NROS should be able to enter:

```text
KNOWLEDGE_DEGRADED
```

rather than falsely claiming normal operation.

# 48. Planner Fault

A planner may:

```text
timeout
produce invalid plan
produce unsafe plan
fail to converge
return contradictory output
```

The runtime should classify:

```text
PLANNER_FAULT
```

and potentially switch to another planner.

# 49. Planner Redundancy

For critical decisions:

```text
Planner A
Planner B
Verifier
```

can be combined:

```text
A → candidate
B → candidate
Verifier → accept/reject
```

This is especially useful for high-risk systems.

# 50. Runtime Invariant Failure

NROS itself may detect:

```text
authorization invariant violated
resource ownership conflict
world-state inconsistency
execution lease violation
```

These are **runtime integrity faults**.

They deserve stronger escalation than ordinary application errors.

# 51. Fault Evidence

Every important fault should connect to:

```text
Observation
→ Fault
→ Impact
→ Recovery
→ Result
```

Example:

```text
O817:
    motor current abnormal

F42:
    motor overload

I17:
    manipulation degraded

R9:
    switched to safe mode

V4:
    safe state verified
```

# 52. Incident

Multiple related faults form an:

```text
Incident
```

Example:

```text
Incident I7

F1: network degradation
F2: localization timeout
F3: planner invalidation
F4: mission delay
```

The incident is the higher-level operational object.

# 53. Incident Timeline

NROS should produce:

```text
T0  normal
T1  anomaly
T2  fault detected
T3  containment
T4  recovery
T5  verification
T6  mission resumed
```

This becomes invaluable for debugging and certification.

# 54. Recovery Evidence

The runtime should answer:

```text
Did recovery actually work?
```

with evidence:

```text
before:
    capability degraded

action:
    failover to sensor B

after:
    valid observations restored

verification:
    PASS
```

# 55. Safe-State Model

Every critical subsystem should declare one or more safe states.

```text
SafeState
├── actuator behavior
├── communication behavior
├── energy behavior
├── authority behavior
└── exit conditions
```

# 56. Safe State Is Contextual

There is no universal:

```text
STOP EVERYTHING
```

For example:

```text
Drone:
    controlled descent

Mobile robot:
    controlled stop

Industrial arm:
    hold/retract

Autonomous vehicle:
    minimum-risk maneuver
```

NROS provides the mechanism; the domain defines the safe state.

# 57. Emergency Mode

Emergency behavior should have stronger semantics:

```text
EMERGENCY
```

can override normal:

```text
planning
optimization
mission priorities
```

but still obey:

```text
hardware safety
physical constraints
certified emergency policy
```

# 58. Recovery Priority

A useful hierarchy:

```text
Safety
   ↓
Containment
   ↓
Physical integrity
   ↓
Authority integrity
   ↓
Service restoration
   ↓
Mission continuation
   ↓
Optimization
```

This hierarchy prevents the runtime from optimizing mission completion while the robot is unsafe.

# 59. Resilience Model

NROS resilience can therefore be expressed as:

```text
Detect
  ↓
Understand
  ↓
Contain
  ↓
Adapt
  ↓
Recover
  ↓
Verify
  ↓
Resume
```

or, if necessary:

```text
Detect
  ↓
Contain
  ↓
Escalate
  ↓
Safe State
```

# 60. NROS Recovery Loop

The full runtime loop becomes:

```text
┌──────────────┐
│   OBSERVE    │
└──────┬───────┘
       ↓
┌──────────────┐
│    BELIEVE   │
└──────┬───────┘
       ↓
┌──────────────┐
│    INTEND    │
└──────┬───────┘
       ↓
┌──────────────┐
│     PLAN     │
└──────┬───────┘
       ↓
┌──────────────┐
│    DECIDE    │
└──────┬───────┘
       ↓
┌──────────────┐
│    COMMIT    │
└──────┬───────┘
       ↓
┌──────────────┐
│    EXECUTE   │
└──────┬───────┘
       ↓
┌──────────────┐
│    VERIFY    │
└──────┬───────┘
       │
       ├──────── success ───────→ next cycle
       │
       ↓
┌──────────────┐
│    FAULT     │
└──────┬───────┘
       ↓
┌──────────────┐
│   CONTAIN    │
└──────┬───────┘
       ↓
┌──────────────┐
│   RECOVER    │
└──────┬───────┘
       ↓
┌──────────────┐
│   REVERIFY   │
└──────┬───────┘
       │
       ├── success → resume
       │
       └── failure → replan / safe state
```

# 61. ROS → NROS: Failure Semantics

The transformation can now be extended:

| ROS | NROS |
|---|---|
| process crash | supervised component failure |
| node restart | lifecycle recovery |
| topic timeout | temporal/communication fault |
| hardware error | resource fault |
| unavailable node | capability degradation |
| disconnected robot | partition state |
| action failure | commitment/work failure |
| sensor disagreement | belief conflict |
| stale message | epoch/temporal violation |
| rosbag debugging | evidence-backed incident reconstruction |
| launch restart | policy-driven supervision |
| shutdown | controlled lifecycle transition |

# 62. What NROS Is Becoming

The architecture now has enough pieces to see the larger picture:

```text
                    NROS
                     │
       ┌─────────────┼─────────────┐
       ↓             ↓             ↓
     WORLD         DECISION      EXECUTION
     MODEL          FABRIC        FABRIC
       │             │             │
   beliefs        plans         resources
   entities       policies      scheduling
   evidence       goals         real-time
   predictions    commitments   control
       │             │             │
       └─────────────┼─────────────┘
                     ↓
                SUPERVISION
                     │
                recovery
                containment
                resilience
```

The key difference from classic middleware is that the runtime now understands **state, intention, responsibility, resources, execution, and failure** as related semantic objects.

# 63. The Remaining Foundational Problem

There is one layer that now becomes unavoidable:

**Time.**

Not merely timestamps.

NROS needs a unified temporal model covering:

```text
event time
logical time
physical time
deadlines
leases
epochs
ordering
causality
synchronization
clock uncertainty
replay
deterministic execution
```

Without this, the preceding layers cannot be made rigorously composable.

For example:

```text
Which observation came first?

Was the plan generated before the world changed?

Was the authorization still valid when execution began?

Did the lease expire before the actuator command?

Was this event replayed or genuinely new?

Can two distributed events be causally ordered?
```

These are not logging questions.

They are **runtime correctness questions**.

# Part LVIII — NROS Temporal & Causality Fabric

The next layer therefore establishes:

```text
Physical Time
      +
Logical Time
      +
Causal Time
      +
Execution Time
      ↓
Unified Temporal Semantics
```

with:

```text
Clock Domains
Timestamping
Monotonicity
Deadlines
Timers
Leases
Epochs
Causal Ordering
Vector/Logical Clocks
Time Windows
Temporal Constraints
Deterministic Replay
Distributed Synchronization
```

This is the layer that can finally connect **real-time execution** with **distributed autonomous reasoning** without confusing wall-clock time, event ordering, and causal ordering.

# NROS — Part LVIII: Temporal & Causality Fabric

The previous layer established **supervision, fault detection, containment, recovery, and resilience**.

Now we address a deeper problem:

> **An autonomous distributed runtime must know not only what happened, but when it happened, in which temporal domain, and what caused what.**

This is where NROS must move beyond ordinary timestamps.

# 1. Time Is a Runtime Primitive

In a conventional middleware system, time often looks like:

```text
timestamp = now()
```

For NROS, that is insufficient.

Time affects:

```text
planning
scheduling
deadlines
leases
authorization
sensor fusion
control
recovery
coordination
replay
causality
```

Therefore:

```text
Time
```

must become a first-class runtime service.

# 2. Multiple Kinds of Time

NROS should distinguish at least:

```text
Wall Time
Monotonic Time
Logical Time
Event Time
Execution Time
Simulation Time
Mission Time
```

These must not be silently interchangeable.

# 3. Wall Time

Wall time represents an external calendar:

```text
2026-08-21 01:42:10
```

Useful for:

```text
logs
human interaction
deadlines tied to real-world schedules
certificates
auditing
```

But wall clocks can jump.

Therefore wall time must not be the sole basis for runtime ordering.

# 4. Monotonic Time

Monotonic time guarantees:

```text
t2 >= t1
```

for events observed within the same clock domain.

It is appropriate for:

```text
timeouts
durations
backoff
watchdogs
execution budgets
latency measurement
```

Example:

```text
start = monotonic_now()
...
elapsed = monotonic_now() - start
```

# 5. Logical Time

Logical time represents ordering rather than physical time.

For example:

```text
Event A
   ↓
Event B
   ↓
Event C
```

Even if their wall-clock timestamps are uncertain, NROS can know:

```text
A happened-before B
B happened-before C
```

# 6. Event Time

An event should carry the time associated with the phenomenon it represents.

For example:

```text
Sensor captured image
    event_time = 12:00:01.200
```

The message may reach NROS at:

```text
receive_time = 12:00:01.245
```

These are different.

# 7. Ingest Time

NROS should record:

```text
event_time
ingest_time
processing_time
completion_time
```

Example:

```text
camera capture:
    10.000 ms

runtime receives:
    10.025 ms

perception starts:
    10.030 ms

perception finishes:
    10.140 ms
```

This makes latency observable.

# 8. Temporal Provenance

Every important event can therefore contain:

```text
TemporalMetadata
├── event_time
├── observed_time
├── ingest_time
├── processing_start
├── processing_end
├── clock_domain
└── uncertainty
```

This becomes part of evidence.

# 9. Clock Domains

Different components may have different clocks:

```text
Robot CPU
Camera
Lidar
Motor controller
GPU
External server
Simulation
```

NROS should explicitly identify:

```text
ClockDomainId
```

rather than assuming all timestamps are directly comparable.

# 10. Clock Uncertainty

A timestamp is not necessarily exact.

Represent:

```text
t = 10.000 ± 2 ms
```

rather than falsely claiming:

```text
t = 10.000000
```

This is particularly important in distributed robotics.

# 11. Temporal Interval

Some events are better represented as intervals:

```text
[10.000, 10.120]
```

For example:

```text
object_visible
```

may be true throughout an interval rather than at one instant.

# 12. Temporal Validity

A state can have validity:

```text
valid_from
valid_until
```

Example:

```text
BatteryState
    value = 72%
    valid_from = 10:00:00
    valid_until = 10:00:05
```

After that point, the runtime may consider the state stale.

# 13. Staleness

NROS should distinguish:

```text
unknown
```

from:

```text
known but stale
```

For example:

```text
robot_pose:
    last update = 2 seconds ago
```

Whether that is acceptable depends on the capability.

# 14. Freshness Requirements

Capabilities can specify:

```text
max_state_age
```

Example:

```text
ObstacleAvoidance:
    lidar data <= 100 ms old
```

If the data is older:

```text
capability = DEGRADED
```

# 15. Deadlines

A Goal may have:

```text
deadline = T
```

A Work item may have:

```text
execution_deadline = T
```

A communication operation may have:

```text
response_timeout = Δ
```

These are different concepts and should not be conflated.

# 16. Temporal Constraint

A plan may require:

```text
A before B
B within 2 s of C
D after E
F during G
```

NROS therefore needs a temporal constraint model.

# 17. Temporal Relations

Useful relations include:

```text
BEFORE
AFTER
MEETS
OVERLAPS
DURING
STARTS
FINISHES
WITHIN
UNTIL
```

This allows plans to express temporal structure explicitly.

# 18. Temporal Plan

Example:

```text
Mission M1

NavigateToPickup
    BEFORE
PickupPackage

PickupPackage
    WITHIN 10s
    AFTER arrival

DeliverPackage
    BEFORE deadline
```

This is richer than a simple ordered list.

# 19. Deadline Semantics

Missing a deadline can have different meanings:

```text
SOFT_DEADLINE
HARD_DEADLINE
SAFETY_DEADLINE
MISSION_DEADLINE
```

A soft deadline may reduce utility.

A safety deadline may require immediate intervention.

# 20. Temporal Priority

A scheduler should understand:

```text
deadline
priority
duration
period
jitter
release time
```

This enables proper real-time scheduling semantics.

# 21. Periodic Work

Many robotic operations are periodic:

```text
control loop:
    every 1 ms

localization:
    every 20 ms

perception:
    every 33 ms

diagnostics:
    every 1 s
```

NROS should model periodicity directly.

# 22. Sporadic Work

Other work occurs unpredictably but has timing constraints:

```text
collision detected
```

Then:

```text
emergency response
    deadline = 5 ms
```

This is different from periodic execution.

# 23. Aperiodic Work

Examples:

```text
user request
mission update
diagnostic query
configuration change
```

These can use ordinary event-driven scheduling.

# 24. Execution-Time Budget

A Work item can declare:

```text
budget:
    CPU = 20 ms
    memory = 4 MB
    energy = 5 J
```

If execution exceeds its budget:

```text
BudgetExceeded
```

becomes an explicit runtime event.

# 25. Temporal Failure

The following can all be faults:

```text
DeadlineMissed
ExecutionOverrun
LeaseExpired
ClockUncertaintyExceeded
DataTooOld
SynchronizationLost
TemporalConstraintViolated
```

This connects time directly to the fault model.

# 26. Leases

A lease is a time-bounded authority.

Example:

```text
Robot A
    holds charger/4

lease:
    [10:00, 10:05]
```

After expiration:

```text
authority = revoked
```

unless renewed.

# 27. Why Leases Matter

Leases solve several distributed problems:

```text
stale ownership
dead controllers
network partitions
resource reservation
temporary authority
```

A dead controller cannot hold a resource forever if its lease expires.

# 28. Lease Renewal

```text
Lease
  ↓
renew
  ↓
renew
  ↓
renew
```

If renewal fails:

```text
lease expires
```

The resource can become available again according to policy.

# 29. Lease + Epoch

For physical resources, combine:

```text
lease
+
epoch
```

Example:

```text
resource = actuator/7
epoch = 42
lease = valid
```

A command must match the currently valid epoch.

This protects against stale commands.

# 30. Causality

NROS must distinguish:

```text
A happened before B
```

from:

```text
A and B happened around the same wall-clock time
```

Causality is about dependency.

# 31. Happens-Before

For example:

```text
SensorObservation
      ↓
WorldModelUpdate
      ↓
PlanGeneration
      ↓
Decision
      ↓
Actuation
```

The runtime should be able to reconstruct this causal chain.

# 32. Event Causality

Every derived event should be able to reference its causes:

```text
Event E42
    caused_by:
        E39
        E40
```

This creates a causal graph.

# 33. Causal Graph

```text
O1 ──→ B1 ──→ G1
          │
          └──→ P1 ──→ D1 ──→ A1
```

Where:

```text
O = observation
B = belief
G = goal
P = plan
D = decision
A = action
```

This is substantially more useful than a flat log.

# 34. Logical Clocks

For distributed causality, NROS can support:

```text
Lamport clocks
```

or equivalent logical ordering.

The key property is:

```text
if A → B
then
clock(A) < clock(B)
```

without requiring perfectly synchronized physical clocks.

# 35. Vector Clocks

Where concurrency matters, NROS can optionally support:

```text
VectorClock
```

This permits distinguishing:

```text
A happened before B
```

from:

```text
A || B
```

where `||` means concurrent.

# 36. Why Concurrency Matters

Suppose:

```text
Robot A sees obstacle X
```

while simultaneously:

```text
Robot B sees obstacle Y
```

Neither observation necessarily caused the other.

The system should preserve this distinction.

# 37. Causal Merge

When distributed state converges:

```text
Robot A state
       +
Robot B state
       ↓
Causal Merge
       ↓
Fleet World Model
```

The merge must preserve conflicting and concurrent evidence.

# 38. Temporal + World Model

The World Model should therefore become:

```text
Entity
├── state
├── confidence
├── source
├── event_time
├── validity
├── provenance
└── causal_context
```

Now a statement such as:

```text
Robot-7 is at (x,y)
```

becomes:

```text
Robot-7
position = (x,y)
confidence = 0.94
observed_at = T
valid_until = T+100ms
source = lidar+odometry
caused_by = E812
```

That is a much stronger semantic object.

# 39. Temporal Reasoning

NROS agents can then reason about:

```text
"What was true?"

"What is true?"

"What was probably true?"

"What changed?"

"When did it change?"

"What caused the change?"

"Is the information still valid?"
```

# 40. Deterministic Replay

The temporal model enables:

```text
Replay
```

A recorded execution can reconstruct:

```text
world state
events
decisions
plans
faults
recovery
```

in causal order.

# 41. Replay Is Not Playback

Playback merely reproduces messages.

NROS replay should reconstruct:

```text
runtime state
```

and allow:

```text
pause
step
inspect
branch
re-run
compare
```

# 42. Time Travel Debugging

An engineer should be able to ask:

```text
At T=8123:
    What did Robot-7 believe?

At T=8124:
    Which plan was active?

At T=8125:
    Why was actuator command A issued?
```

This is **temporal debugging**.

# 43. Deterministic Execution

For deterministic components, replay should ideally produce:

```text
same inputs
→ same decisions
→ same outputs
```

when the relevant environment and nondeterministic sources are controlled.

This is crucial for testing autonomous systems.

# 44. Sources of Nondeterminism

NROS should explicitly identify:

```text
randomness
thread scheduling
network ordering
hardware timing
clock drift
external environment
concurrent writes
AI model nondeterminism
```

Not every nondeterministic source can be eliminated, but it should be observable.

# 45. Execution Trace

An NROS trace can therefore look like:

```text
T100 O1 Sensor
T101 B1 BeliefUpdate
T102 G1 Goal
T103 P1 Plan
T104 D1 Decision
T105 C1 Commitment
T106 W1 WorkStart
T107 A1 Actuation
T108 O2 Observation
T109 V1 Verification
```

with causal edges:

```text
O1 → B1 → P1 → D1 → W1 → A1
                     ↓
                     V1
```

# 46. Temporal Consistency

The runtime can validate invariants such as:

```text
decision_time >= plan_time
execution_time >= commitment_time
verification_time >= execution_time
```

Violations become runtime integrity faults.

# 47. Temporal Authority

Authority itself can have validity:

```text
Authorization
    valid_from
    valid_until
    epoch
```

Therefore:

```text
authorized yesterday
```

does not imply:

```text
authorized now
```

# 48. Temporal Capability

Capabilities can also be conditional:

```text
Capability:
    operate elevator

valid:
    08:00–18:00

requires:
    supervisor lease
```

This becomes useful in industrial and safety-critical environments.

# 49. Time-Aware Scheduling

NROS scheduling now becomes:

```text
Work
├── release_time
├── deadline
├── period
├── duration_budget
├── priority
├── dependencies
└── temporal constraints
```

This allows the scheduler to reason about both resources and time.

# 50. Temporal Resource Reservation

A resource reservation becomes:

```text
Resource:
    camera-1

Reserved:
    [10:00, 10:05]

Owner:
    Agent-7

Epoch:
    19
```

This is much stronger than:

```text
camera = busy
```

# 51. Temporal Coordination

Two robots can coordinate:

```text
Robot A:
    arrive at loading bay by T

Robot B:
    open gate during [T, T+5s]
```

The coordination system can verify whether the temporal contract is satisfiable.

# 52. Time Uncertainty Propagation

Suppose:

```text
Sensor:
    ±5 ms
```

and:

```text
Network:
    ±10 ms
```

Then downstream reasoning should not pretend:

```text
event_time = exact
```

Uncertainty can propagate through the evidence chain.

# 53. Temporal Confidence

A belief may therefore contain:

```text
confidence = 0.91
temporal_uncertainty = ±20ms
```

This allows planners to distinguish:

```text
high-confidence recent fact
```

from:

```text
high-confidence but stale fact
```

# 54. Temporal Semantics of Events

NROS events should answer:

```text
When did it occur?
When was it observed?
When was it processed?
What did it depend on?
How long was it valid?
```

This becomes the canonical event contract.

# 55. The NROS Temporal Stack

We can now define:

```text
┌─────────────────────────────┐
│ Mission Time                │
├─────────────────────────────┤
│ Temporal Constraints        │
├─────────────────────────────┤
│ Deadlines / Periods / Leases│
├─────────────────────────────┤
│ Logical / Causal Time       │
├─────────────────────────────┤
│ Event Time                  │
├─────────────────────────────┤
│ Monotonic Runtime Time      │
├─────────────────────────────┤
│ Physical Clock Domains      │
├─────────────────────────────┤
│ Hardware / Timer Sources    │
└─────────────────────────────┘
```

# 56. ROS → NROS: Temporal Evolution

Classic ROS commonly exposes:

```text
timestamp
timer
rate
message ordering
```

NROS expands this into:

```text
ROS
 ↓
timestamps
 ↓
NROS
 ↓
temporal semantics
 + causality
 + leases
 + epochs
 + deadlines
 + validity
 + uncertainty
 + replay
```

# 57. The Deeper Transformation

We can now describe NROS as a runtime with four fundamental dimensions:

```text
STATE
    What exists?

TIME
    When?

INTENT
    What should happen?

AUTHORITY
    Who may cause it?
```

Then:

```text
DECISION
    Which action should be selected?

EXECUTION
    How is it performed?

EVIDENCE
    What actually happened?

RECOVERY
    What happens when reality diverges?
```

This gives us a coherent semantic architecture.

# 58. NROS Core Equation

A useful conceptual model is:

```text
NROS State(t)
=
World(t)
+
Beliefs(t)
+
Goals(t)
+
Plans(t)
+
Commitments(t)
+
Resources(t)
+
Authority(t)
+
Temporal Context(t)
+
Execution(t)
+
Faults(t)
```

The runtime continuously evolves this state.

# 59. The Autonomous Runtime Loop

The complete conceptual loop is now:

```text
          ┌───────────────┐
          │    OBSERVE    │
          └───────┬───────┘
                  ↓
          ┌───────────────┐
          │    UPDATE     │
          │  WORLD MODEL  │
          └───────┬───────┘
                  ↓
          ┌───────────────┐
          │     INTEND    │
          └───────┬───────┘
                  ↓
          ┌───────────────┐
          │     PLAN      │
          └───────┬───────┘
                  ↓
          ┌───────────────┐
          │     DECIDE    │
          └───────┬───────┘
                  ↓
          ┌───────────────┐
          │    AUTHORIZE  │
          └───────┬───────┘
                  ↓
          ┌───────────────┐
          │    COMMIT     │
          └───────┬───────┘
                  ↓
          ┌───────────────┐
          │    SCHEDULE   │
          └───────┬───────┘
                  ↓
          ┌───────────────┐
          │    EXECUTE    │
          └───────┬───────┘
                  ↓
          ┌───────────────┐
          │    VERIFY     │
          └───────┬───────┘
                  ↓
            ┌─────┴─────┐
            │           │
          SUCCESS     FAILURE
            │           │
            ↓           ↓
          LEARN       CONTAIN
            │           ↓
            │        RECOVER
            │           ↓
            │        REPLAN
            │           │
            └─────┬─────┘
                  ↓
                LOOP
```

# 60. What Remains

At this point NROS has a semantic model for:

```text
Communication
Types
Events
State
World Model
Belief
Intent
Goals
Planning
Decision
Authority
Capabilities
Resources
Scheduling
Execution
Supervision
Faults
Recovery
Time
Causality
Replay
```

The next question is no longer merely:

> **How do NROS components communicate?**

It becomes:

> **How are these semantics represented as a concrete Rust architecture without collapsing them back into a monolithic middleware core?**

That brings us to the next layer:

# Part LIX — NROS Kernel Architecture: Crates, Traits, State Machines & Runtime Boundaries

We will map the conceptual architecture into a concrete Rust workspace:

```text
nros-core
nros-event
nros-time
nros-world
nros-belief
nros-intent
nros-plan
nros-decision
nros-policy
nros-authority
nros-capability
nros-resource
nros-scheduler
nros-executor
nros-supervisor
nros-recovery
nros-transport
nros-protocol
nros-agent
nros-observability
nros-replay
```

and, most importantly, determine **what belongs inside the NROS kernel and what must remain replaceable at the edge**.

# NROS — Part LIX: Kernel Architecture — From Semantic Model to Rust Runtime

We now cross an important boundary.

The previous parts defined **what NROS means**:

```text
world
belief
intent
goal
plan
decision
authority
resource
execution
fault
recovery
time
causality
```

This part defines **where those semantics live**.

The central architectural principle is:

> **NROS should be a small semantic kernel surrounded by replaceable execution, transport, intelligence, and hardware adapters.**

# 1. The Kernel Must Not Become ROS Rewritten

A dangerous design would be:

```text
ROS
  ↓
rewrite everything in Rust
  ↓
NROS
```

That produces a faster implementation of the same abstraction.

NROS should instead be:

```text
ROS middleware
       ↓
architectural lessons
       ↓
new semantic kernel
       ↓
agent-native runtime
```

The distinction is fundamental.

ROS primarily organizes:

```text
nodes
topics
services
parameters
packages
```

NROS organizes:

```text
entities
events
beliefs
intentions
goals
plans
commitments
capabilities
resources
authority
work
evidence
time
faults
recovery
```

# 2. The NROS Kernel

Conceptually:

```text
                    NROS
                     │
              ┌──────┴──────┐
              │    KERNEL   │
              └──────┬──────┘
                     │
     ┌───────────────┼────────────────┐
     ↓               ↓                ↓
   State          Authority        Runtime
     │               │                │
     ↓               ↓                ↓
  World          Policy          Scheduling
  Events         Leases          Execution
  Time           Capabilities     Supervision
```

Everything else should be composable around this.

# 3. What Belongs in the Kernel?

A good test is:

> **Can the system remain semantically correct if this subsystem is replaced?**

If yes, it probably belongs outside the kernel.

For example:

```text
DDS
Zenoh
QUIC
TCP
ROS 2 bridge
MQTT
```

should not define NROS semantics.

They are transport implementations.

Likewise:

```text
LLM
planner
SLAM
vision
navigation
```

should not define the kernel.

They are consumers/providers of capabilities.

# 4. Kernel Responsibilities

The kernel should establish invariants for:

```text
Identity
Event semantics
Time
State transitions
Authority
Resources
Work
Execution lifecycle
Fault lifecycle
Evidence
Causality
```

The kernel should **not** decide:

```text
which neural network
which planner
which robot model
which transport
which database
which simulator
```

# 5. Proposed Workspace

A first conceptual Rust workspace:

```text
nros/
├── crates/
│   ├── nros-core/
│   ├── nros-event/
│   ├── nros-time/
│   ├── nros-world/
│   ├── nros-belief/
│   ├── nros-intent/
│   ├── nros-plan/
│   ├── nros-decision/
│   ├── nros-policy/
│   ├── nros-authority/
│   ├── nros-capability/
│   ├── nros-resource/
│   ├── nros-work/
│   ├── nros-scheduler/
│   ├── nros-executor/
│   ├── nros-supervisor/
│   ├── nros-recovery/
│   ├── nros-transport/
│   ├── nros-protocol/
│   ├── nros-observability/
│   └── nros-replay/
```

This is a **semantic decomposition**, not necessarily the final physical crate layout.

# 6. Dependency Direction

The most important rule:

```text
higher-level semantics
        ↓
lower-level mechanisms
```

Never:

```text
transport → goal
```

or:

```text
DDS → world model
```

Instead:

```text
World
  ↓
Event
  ↓
Protocol
  ↓
Transport
```

Transport carries semantics.

It does not define them.

# 7. `nros-core`

This should contain only the smallest universal primitives.

Potential contents:

```rust
EntityId
EventId
ExecutionId
WorkId
GoalId
PlanId
CommitmentId
CapabilityId
ResourceId
FaultId
IncidentId
Epoch
```

Plus common result/error semantics.

The goal:

> Keep `nros-core` boring.

If `nros-core` becomes huge, the architecture is probably collapsing.

# 8. Identity

NROS needs strongly typed identities.

Avoid:

```rust
String
```

everywhere.

Prefer:

```rust
pub struct EntityId(u128);
pub struct EventId(u128);
pub struct GoalId(u128);
pub struct WorkId(u128);
```

This prevents accidental interchange.

For example:

```rust
fn cancel(goal: GoalId)
```

cannot accidentally receive:

```rust
ResourceId
```

# 9. Identity Is More Than UUID

An identity may need:

```text
local identity
origin
generation
epoch
scope
```

For distributed systems:

```text
robot-07/entity-42
```

and:

```text
robot-08/entity-42
```

must not collide semantically merely because the local IDs match.

# 10. `nros-event`

Events are the runtime's fundamental evidence units.

Conceptually:

```rust
pub struct Event<T> {
    pub id: EventId,
    pub kind: EventKind,
    pub time: EventTime,
    pub source: EntityId,
    pub causality: CausalContext,
    pub payload: T,
}
```

The event is not merely a message.

It is:

```text
fact + provenance + temporal context + causality
```

# 11. Event Immutability

A critical principle:

> **Events should be immutable once committed.**

Corrections should create new events.

Bad:

```text
Event 42
value = 5

later mutate:
value = 7
```

Better:

```text
Event 42:
value = 5

Event 57:
correction_of = 42
value = 7
```

This preserves history.

# 12. Event Sourcing vs Event Transport

Do not confuse:

```text
event transport
```

with:

```text
event sourcing
```

NROS can use event semantics without requiring every implementation to store every event forever.

The kernel defines:

```text
event identity
ordering
causality
schema
```

Storage is an adapter.

# 13. `nros-time`

This crate owns temporal primitives:

```text
Instant
Duration
Deadline
Period
TimePoint
Interval
ClockId
ClockDomain
Timestamp
Uncertainty
LeaseDuration
```

It should prevent accidental mixing of:

```text
wall time
monotonic time
logical time
simulation time
```

# 14. Typed Time

A desirable API:

```rust
MonotonicInstant
WallClockTime
LogicalTime
SimulationTime
```

rather than:

```rust
u64
```

everywhere.

The compiler should help enforce temporal correctness.

# 15. `nros-world`

This crate owns the **world model**.

Conceptually:

```text
World
├── entities
├── relationships
├── state
├── observations
├── validity
├── provenance
└── confidence
```

The world model should not itself perform planning.

It describes what the runtime currently believes about the world.

# 16. World State vs Belief

These must remain distinct.

```text
WorldModel:
    representation of relevant world state

Belief:
    runtime's confidence about that state
```

For example:

```text
Entity:
    obstacle-42

Belief:
    position = (4.2, 1.8)
    confidence = 0.87
```

# 17. `nros-belief`

This crate handles:

```text
belief
confidence
evidence
hypothesis
uncertainty
contradiction
belief revision
```

A belief may be:

```text
SUPPORTED
TENTATIVE
CONFLICTED
STALE
REJECTED
UNKNOWN
```

# 18. Evidence Chain

A belief should be traceable:

```text
Sensor Event
     ↓
Observation
     ↓
Inference
     ↓
Belief
```

The chain allows:

```text
"Why does NROS believe X?"
```

to have an answer.

# 19. `nros-intent`

Intent is higher-level than raw goals.

Example:

```text
Intent:
    deliver package
```

It may generate:

```text
Goal:
    package reaches destination
```

The intent layer allows an agent to maintain continuity even when the plan changes.

# 20. `nros-plan`

A plan is a proposed way of achieving goals.

It contains:

```text
steps
dependencies
preconditions
effects
temporal constraints
resource requirements
risk
fallbacks
```

It should remain independent of the execution engine.

# 21. Plan Is Not Execution

This distinction is essential:

```text
Plan:
    "move robot to location X"
```

does not mean:

```text
robot is currently moving to X
```

Execution begins only after:

```text
authorization
commitment
scheduling
```

# 22. `nros-decision`

Decision is where candidate plans/actions become selected actions.

Conceptually:

```text
Observation
   ↓
Candidate Actions
   ↓
Policy
   ↓
Constraints
   ↓
Risk
   ↓
Decision
```

The decision engine may use:

```text
rule-based logic
optimization
classical planning
ML
LLM
human approval
```

NROS should not mandate one.

# 23. `nros-policy`

Policy answers:

```text
what is allowed?
what is forbidden?
what is preferred?
under which conditions?
```

Policy can cover:

```text
safety
security
resource allocation
mission priority
authority
privacy
energy
risk
```

# 24. `nros-authority`

This is one of the major differences from conventional middleware.

Authority should be explicit.

```text
Authority
├── principal
├── capability
├── scope
├── constraints
├── epoch
├── lease
└── validity
```

A component cannot merely "send a command."

It must possess the authority to perform the operation.

# 25. `nros-capability`

A capability describes what an entity can do.

Examples:

```text
navigate
sense
grasp
move
speak
compute
localize
charge
```

Capability ≠ authority.

An entity can have:

```text
capability = navigate
authority = denied
```

# 26. Capability vs Resource

Consider:

```text
Robot-7
```

It may have:

```text
Capability:
    navigation

Resources:
    lidar
    wheels
    CPU
    battery
```

Capability is **what can be done**.

Resource is **what is consumed or controlled**.

# 27. `nros-resource`

Resources need explicit lifecycle:

```text
AVAILABLE
RESERVED
ALLOCATED
IN_USE
DEGRADED
QUARANTINED
RELEASED
FAILED
```

This gives scheduling and recovery a shared semantic foundation.

# 28. `nros-work`

Work is the bridge between intention and execution.

A useful conceptual object:

```text
Work
├── goal
├── plan_step
├── authority
├── resources
├── temporal constraints
├── execution policy
└── verification criteria
```

Work is therefore a **contract for execution**.

# 29. Work State Machine

```text
PROPOSED
   ↓
AUTHORIZED
   ↓
READY
   ↓
SCHEDULED
   ↓
RUNNING
   ↓
VERIFYING
   ↓
COMPLETED
```

Failure paths:

```text
RUNNING
   ↓
FAILED
   ↓
RECOVERING
   ↓
RESUMED
```

or:

```text
FAILED
   ↓
ABORTED
```

# 30. `nros-scheduler`

The scheduler should answer:

> **Which authorized Work should execute now?**

Inputs:

```text
priority
deadline
resource availability
dependencies
temporal constraints
risk
authority
energy
CPU
safety state
```

It should not invent goals.

# 31. Scheduler Architecture

```text
                 Scheduler
                     │
       ┌─────────────┼─────────────┐
       ↓             ↓             ↓
   Ready Work     Resources     Temporal
       │             │          Constraints
       └─────────────┼─────────────┘
                     ↓
               Schedule Decision
```

# 32. `nros-executor`

The executor turns scheduled Work into actual operations.

Possible execution domains:

```text
async tasks
threads
real-time threads
hardware commands
remote execution
GPU jobs
agent tool calls
```

The executor should be replaceable.

# 33. Real-Time Boundary

This is particularly important.

NROS should not force:

```text
LLM reasoning
```

into:

```text
hard real-time actuator loop
```

Instead:

```text
Agentic Layer
     ↓
Decision
     ↓
Real-Time Contract
     ↓
RT Executor
     ↓
Controller
     ↓
Actuator
```

This separation is fundamental.

# 34. Hard Real-Time Kernel Boundary

A useful architecture:

```text
┌──────────────────────────────────┐
│ Agent / Planning / Reasoning     │
│ non-real-time                    │
└───────────────┬──────────────────┘
                │
          bounded contract
                │
┌───────────────▼──────────────────┐
│ NROS Coordination Kernel         │
└───────────────┬──────────────────┘
                │
          RT-safe command
                │
┌───────────────▼──────────────────┐
│ Real-Time Executor               │
└───────────────┬──────────────────┘
                │
             hardware
```

This prevents the agentic layer from becoming a timing-critical dependency.

# 35. `nros-supervisor`

The supervisor owns:

```text
lifecycle
health
failure detection
restart
containment
escalation
```

It should observe executor and component state without becoming the executor itself.

# 36. `nros-recovery`

Recovery strategies should be explicit objects:

```rust
pub enum RecoveryStrategy {
    Retry,
    Restart,
    Failover,
    Replan,
    Compensate,
    Degrade,
    Isolate,
    Abort,
}
```

Policy determines which strategy is applicable.

# 37. `nros-transport`

Transport is deliberately thin.

Possible implementations:

```text
DDS
Zenoh
QUIC
TCP
Unix sockets
shared memory
CAN
serial
custom fieldbus
```

The transport API should carry NROS protocol objects without defining their meaning.

# 38. `nros-protocol`

Protocol defines wire-level representation:

```text
Event
Command
Query
Response
Lease
Capability
Work
Fault
Evidence
```

Possible encodings:

```text
CBOR
MessagePack
Protobuf
FlatBuffers
custom binary
JSON
```

Again:

> Encoding is replaceable.

# 39. `nros-observability`

Observability should expose:

```text
metrics
traces
events
health
causal graphs
latency
resource usage
faults
recovery
```

Potential integrations:

```text
OpenTelemetry
Prometheus
structured logs
custom event stores
```

But the semantic trace model belongs to NROS.

# 40. `nros-replay`

Replay consumes:

```text
events
temporal metadata
causal metadata
state transitions
external inputs
```

and reconstructs:

```text
runtime state
```

This makes replay a first-class capability rather than a debugging afterthought.

# 41. Dependency Graph

The conceptual dependency structure should look approximately like:

```text
                    nros-core
                        │
          ┌─────────────┼─────────────┐
          ↓             ↓             ↓
      nros-time     nros-event    nros-protocol
          │             │             │
          └──────┬──────┴──────┬──────┘
                 ↓             ↓
             nros-world     nros-resource
                 │             │
          ┌──────┴─────┐       │
          ↓            ↓       ↓
    nros-belief    nros-intent
          │            │
          └──────┬─────┘
                 ↓
             nros-plan
                 ↓
           nros-decision
                 ↓
            nros-policy
                 ↓
           nros-authority
                 ↓
             nros-work
                 ↓
          nros-scheduler
                 ↓
           nros-executor
                 ↓
          nros-supervisor
                 ↓
           nros-recovery
```

Transport and observability should remain largely orthogonal.

# 42. The Most Important Dependency Rule

Never allow:

```text
nros-core
    ↓
nros-transport
    ↓
nros-core
```

or:

```text
nros-world
    ↔
nros-planner
```

through circular dependencies.

The semantic kernel must remain acyclic.

# 43. Traits at the Boundaries

Rust traits become especially useful at subsystem boundaries.

For example:

```rust
pub trait Clock {
    type Instant;

    fn now(&self) -> Self::Instant;
}
```

Transport:

```rust
pub trait Transport {
    type Error;

    fn publish(&self, event: EventEnvelope)
        -> Result<(), Self::Error>;
}
```

Scheduler:

```rust
pub trait Scheduler {
    fn schedule(&mut self, work: Work)
        -> Result<ScheduleDecision, ScheduleError>;
}
```

# 44. State Machines as Types

Instead of allowing invalid states:

```rust
Work {
    state: WorkState,
}
```

everywhere, NROS can eventually use typestate where appropriate:

```rust
Work<Authorized>
Work<Running>
Work<Completed>
```

This is particularly valuable for security- and safety-sensitive transitions.

# 45. Runtime State Machine

At the orchestration level:

```text
BOOTSTRAPPING
      ↓
INITIALIZING
      ↓
READY
      ↓
ACTIVE
      ↓
DEGRADED
      ↓
RECOVERING
      ↓
ACTIVE
```

Terminal states:

```text
SAFE
STOPPED
FAILED
```

# 46. Kernel Event Loop

A conceptual kernel loop:

```text
loop {
    let event = ingress.next().await?;

    let state = state.apply(event)?;

    let decisions = policy.evaluate(&state)?;

    let work = planner_or_agent
        .propose(decisions)?;

    let authorized = authority
        .authorize(work)?;

    scheduler.submit(authorized)?;

    executor.dispatch()?;
}
```

But this is only conceptual.

The actual implementation must preserve:

```text
concurrency
real-time boundaries
causal ordering
backpressure
failure isolation
```

# 47. Don't Build One Giant Event Loop

A common mistake:

```text
NROS
└── one global loop
```

That creates:

```text
contention
latency coupling
fault coupling
poor scalability
```

Instead, use multiple execution domains:

```text
Control Domain
Planning Domain
Coordination Domain
I/O Domain
Observation Domain
Supervision Domain
```

connected through explicit contracts.

# 48. Execution Domains

For example:

```text
┌───────────────┐
│ Control       │
│ hard RT       │
└───────┬───────┘
        │
┌───────▼───────┐
│ Coordination  │
│ bounded RT    │
└───────┬───────┘
        │
┌───────▼───────┐
│ Agent         │
│ soft RT       │
└───────┬───────┘
        │
┌───────▼───────┐
│ Intelligence  │
│ best effort   │
└───────────────┘
```

This is much more appropriate for agentic robotics.

# 49. Agent Runtime Boundary

An LLM/agent should interact with NROS through:

```text
Observe
Query
Propose
Request
Commit
Execute
Verify
```

rather than direct access to:

```text
hardware registers
actuator drivers
scheduler internals
authority database
```

The kernel remains the policy enforcement point.

# 50. Agent as a Runtime Participant

The major conceptual shift is:

```text
ROS:
    Node is the primary execution abstraction.

NROS:
    Agent / Work / Capability / Resource participate in a
    semantic runtime.
```

An agent is therefore not necessarily a privileged super-process.

It is another participant subject to:

```text
authority
resource limits
deadlines
policy
supervision
evidence
```

# 51. Hardware Boundary

Hardware drivers should expose capabilities:

```text
MotorDriver
CameraDriver
LidarDriver
IMUDriver
PLCDriver
CANDriver
```

rather than forcing the rest of NROS to understand hardware-specific protocols.

# 52. Industrial Integration

This architecture is particularly useful for industrial systems:

```text
NROS
 ↓
Capability
 ↓
PLC / RT controller
 ↓
fieldbus
 ↓
actuator
```

NROS does not replace the deterministic controller.

It coordinates it.

# 53. NROS as a Control Plane

This suggests an important distinction:

```text
ROS:
    communication middleware

NROS:
    autonomous control + coordination plane
```

while lower-level controllers remain:

```text
data plane / control loop
```

# 54. NROS Kernel Invariants

The kernel should enforce invariants such as:

```text
1. No unauthorized Work executes.

2. Expired leases cannot authorize execution.

3. Stale epochs cannot issue commands.

4. Invalid state transitions are rejected.

5. Verified evidence cannot be silently rewritten.

6. Failed resources cannot be allocated normally.

7. Hard safety constraints dominate mission optimization.

8. Recovery must be verified before restoration.

9. Temporal constraints are explicit.

10. Causal history remains reconstructible.
```

These invariants are more important than any particular API.

# 55. Kernel vs Ecosystem

The resulting architecture becomes:

```text
                  NROS ECOSYSTEM
────────────────────────────────────────────
 Agents       Planners       Perception
 Simulators   Drivers        AI Models
 Navigation   SLAM           GUIs
 Databases    Bridges        Cloud
────────────────────────────────────────────
                  NROS KERNEL
────────────────────────────────────────────
 Identity
 Events
 Time
 World
 Intent
 Goals
 Authority
 Resources
 Work
 Scheduling
 Execution Contracts
 Supervision
 Recovery
 Evidence
 Causality
────────────────────────────────────────────
                  ADAPTERS
────────────────────────────────────────────
 DDS / Zenoh / QUIC / CAN / IPC / RTOS
```

# 56. The Architectural Thesis

The evolution from ROS to NROS can now be stated more precisely:

```text
ROS
=
distributed robotics middleware

NROS
=
semantic autonomous runtime
```

or:

```text
ROS connects computation.

NROS coordinates agency.
```

That is the architectural leap.

# 57. Next Layer

The next problem is the **actual kernel object model**.

We need to define the canonical objects and their relationships:

```text
Entity
Agent
Observation
Belief
Goal
Intent
Plan
Decision
Capability
Resource
Authority
Commitment
Work
Execution
Event
Evidence
Fault
Recovery
Lease
Epoch
```

and answer a deceptively important question:

> **Which of these are persistent domain objects, which are ephemeral runtime objects, and which are immutable evidence?**

That leads directly to:

# Part LX — NROS Canonical Object Model & State Algebra

where we can construct the underlying type system:

```text
Entity
   │
   ├── Agent
   ├── Resource
   ├── Capability
   └── Component

Observation
   ↓
Belief
   ↓
Intent
   ↓
Goal
   ↓
Plan
   ↓
Decision
   ↓
Commitment
   ↓
Work
   ↓
Execution
   ↓
Evidence
   ↓
Belief Update
```

with:

```text
Event
Time
Causality
Authority
Fault
Recovery
```

cross-cutting the entire graph.

That object algebra is where NROS stops being merely an architecture diagram and starts becoming a **formalizable Rust runtime model**.

# NROS — Part LX: Canonical Object Model & State Algebra

We now move from **crate architecture** to the deeper question:

> **What are the fundamental objects of NROS, what state can they occupy, and how are transitions between them constrained?**

This is the point where NROS can begin to acquire a precise mathematical and Rust-level identity.

# 1. The NROS Object Universe

The runtime can be organized around a relatively small set of canonical objects:

```text
ENTITY
AGENT
COMPONENT

OBSERVATION
BELIEF

INTENT
GOAL
PLAN
DECISION

CAPABILITY
RESOURCE
AUTHORITY

COMMITMENT
WORK
EXECUTION

EVENT
EVIDENCE

FAULT
INCIDENT
RECOVERY
```

These should not all be treated equally.

They belong to different semantic categories.

# 2. Four Fundamental Categories

A useful first partition is:

```text
IDENTITY
    Entity
    Agent
    Component
    Resource
    Capability

STATE
    World
    Belief
    Goal
    Commitment
    Execution
    Fault

PROPOSAL
    Intent
    Plan
    Decision
    Recovery

EVIDENCE
    Event
    Observation
    Evidence
```

This distinction prevents the runtime from confusing:

```text
what exists
```

with:

```text
what someone proposes
```

and:

```text
what actually happened
```

# 3. Immutable vs Mutable

A second fundamental distinction:

### Immutable

```text
Event
Observation
Evidence
DecisionRecord
AuditRecord
```

### Mutable State

```text
WorldModel
BeliefState
ResourceState
GoalState
ExecutionState
FaultState
```

### Append-Only History

```text
EventLog
CausalGraph
AuditTrail
```

This gives NROS historical integrity without requiring every runtime object to be immutable.

# 4. Entity

The most primitive semantic object is:

```text
Entity
```

An entity represents something that can be identified by the runtime.

Examples:

```text
robot-7
camera-2
warehouse-3
package-42
agent-alpha
charging-station-4
```

# 5. Entity Identity

Conceptually:

```rust
struct EntityId {
    namespace: NamespaceId,
    local: LocalId,
    generation: Generation,
}
```

The exact representation can evolve.

The important property is:

> Identity must be stable enough to support provenance, authority, and history.

# 6. Entity Lifecycle

Entities may exist through:

```text
DISCOVERED
REGISTERED
ACTIVE
DEGRADED
QUARANTINED
RETIRED
```

But discovery should not automatically imply authority.

# 7. Agent

An `Agent` is an entity capable of autonomous decision-making.

Examples:

```text
human operator
robot controller
autonomous robot
planning agent
fleet coordinator
software agent
```

The crucial definition is:

> **An agent is an entity that can originate or participate in intentional decision processes.**

# 8. Agent ≠ Process

This is a major distinction.

One agent may span:

```text
multiple processes
multiple machines
multiple execution contexts
```

And one process may host:

```text
multiple agents
```

Therefore:

```text
Agent
≠
OS Process
```

# 9. Component

A component is an executable runtime unit:

```text
process
thread
service
driver
plugin
actor
container
```

Components provide mechanisms.

Agents provide or participate in agency.

# 10. Agent–Component Relationship

```text
Agent
  │
  ├── Component A
  ├── Component B
  └── Component C
```

But an agent may also invoke external components:

```text
Agent
   ↓
Capability
   ↓
Component
```

# 11. Capability

A capability is a typed declaration:

```text
Capability:
    what can be done
```

Examples:

```text
MoveArm
CaptureImage
Navigate
ReadTemperature
OpenValve
EstimatePose
GeneratePlan
```

Capabilities should be composable.

# 12. Capability Contract

A capability should describe:

```text
inputs
outputs
preconditions
postconditions
resource requirements
timing constraints
safety constraints
authority requirements
```

For example:

```text
NavigateTo

Input:
    destination

Requires:
    localization
    propulsion
    navigation authority

Produces:
    reached(destination)
```

# 13. Resource

A resource is something whose use is constrained.

Examples:

```text
CPU
GPU
battery
motor
camera
radio channel
physical workspace
robot
charging station
```

A resource may be:

```text
exclusive
shared
partitionable
renewable
consumable
```

# 14. Resource Algebra

Resources can have quantities:

```text
CPU:
    4 cores

Battery:
    82%

Memory:
    2 GB free
```

or discrete ownership:

```text
Arm:
    owned by Work-42
```

The scheduler should understand both.

# 15. Authority

Authority answers:

> **Who is permitted to cause a particular effect?**

This is separate from capability.

```text
Capability:
    can open valve

Authority:
    currently permitted to open valve
```

# 16. Authority Tuple

Conceptually:

```text
Authority =
    Principal
    +
    Capability
    +
    Scope
    +
    Constraints
    +
    Epoch
    +
    Lease
```

This becomes one of the central security/runtime objects.

# 17. Observation

An observation represents information obtained from the environment or another source.

```text
Observation
├── source
├── event_time
├── received_time
├── payload
├── confidence
└── provenance
```

An observation is evidence.

It is not automatically truth.

# 18. Belief

A belief is an interpreted proposition.

Example:

```text
Observation:
    lidar detects object

Belief:
    object-42 exists at position X
```

Thus:

```text
Observation
      ↓ interpretation
Belief
```

# 19. Belief State

Beliefs may conflict.

Example:

```text
Camera:
    object at X

Lidar:
    object at Y
```

NROS should preserve:

```text
hypothesis A
hypothesis B
confidence
sources
```

rather than silently overwriting one.

# 20. Intent

Intent expresses desired direction.

Example:

```text
"Deliver package 42."
```

Intent can survive plan changes.

```text
Plan A fails
    ↓
Plan B generated
    ↓
same Intent
```

This provides continuity.

# 21. Goal

A goal is more operationally precise:

```text
Goal:
    package-42
    at destination-D
    before deadline-T
```

Goals may have:

```text
success criteria
deadline
priority
constraints
utility
risk tolerance
```

# 22. Intent → Goal

One intent may produce several goals:

```text
Intent:
    deliver package

Goals:
    acquire package
    navigate to destination
    deliver package
    confirm delivery
```

This decomposition belongs above execution.

# 23. Plan

A plan is a candidate strategy for satisfying one or more goals.

```text
Plan
├── steps
├── dependencies
├── preconditions
├── effects
├── resources
├── timing
└── fallback
```

A plan is not necessarily executable yet.

# 24. Decision

A decision chooses among alternatives.

```text
Candidate Plan A
Candidate Plan B
Candidate Plan C
        ↓
Policy
Constraints
Risk
Utility
Authority
        ↓
Decision
```

The decision should preserve its rationale/provenance.

# 25. Decision Record

A durable decision record can contain:

```text
DecisionId
Agent
Goal
Candidates
Selected
PolicyVersion
Evidence
Timestamp
CausalContext
```

This supports later questions such as:

> Why did NROS choose Plan B?

# 26. Commitment

A commitment means:

> **The runtime has accepted responsibility for attempting a specific outcome under specified conditions.**

This is stronger than a plan.

```text
Plan:
    proposed

Commitment:
    accepted responsibility
```

# 27. Commitment Contract

```text
Commitment
├── owner
├── goal
├── plan
├── authority
├── resources
├── deadline
├── guarantees
└── cancellation conditions
```

# 28. Work

Work is the executable unit derived from commitment.

```text
Commitment
    ↓
Work
```

One commitment may generate multiple Work items.

# 29. Execution

Execution represents what the runtime is actually doing.

```text
Work
   ↓
Execution
```

Execution has concrete runtime state:

```text
QUEUED
STARTING
RUNNING
PAUSED
COMPLETED
FAILED
CANCELLED
```

# 30. Evidence

Evidence describes what actually happened.

For example:

```text
Execution:
    "close valve"

Evidence:
    valve_position = CLOSED
    measured_at = T
```

The distinction is crucial:

```text
command ≠ effect
```

# 31. Event

An event records a transition or occurrence:

```text
ValveCommandIssued
ValveClosedObserved
ExecutionCompleted
```

Events are the temporal backbone.

# 32. Fault

A fault represents abnormal state:

```text
ValveDidNotClose
```

A fault can reference:

```text
execution
resource
event
evidence
impact
recovery
```

# 33. Incident

An incident groups related faults and consequences:

```text
Incident
├── Fault A
├── Fault B
├── affected Work
├── affected Goals
└── Recovery
```

# 34. Recovery

Recovery is itself a first-class operation:

```text
Recovery
├── incident
├── strategy
├── attempt
├── actions
├── verification
└── result
```

This means recovery can itself fail, be supervised, and produce evidence.

# 35. The Complete Semantic Chain

We can now formulate the core transformation:

```text
OBSERVATION
     ↓
BELIEF
     ↓
INTENT
     ↓
GOAL
     ↓
PLAN
     ↓
DECISION
     ↓
AUTHORITY
     ↓
COMMITMENT
     ↓
WORK
     ↓
EXECUTION
     ↓
EVIDENCE
     ↓
BELIEF UPDATE
```

With failures:

```text
EXECUTION
     ↓
FAULT
     ↓
INCIDENT
     ↓
RECOVERY
     ↓
EVIDENCE
     ↓
REPLAN
```

And all of it is crossed by:

```text
TIME
CAUSALITY
POLICY
IDENTITY
```

# 36. State Algebra

Now we can define a more formal idea.

Every runtime object has:

```text
State
+
Transitions
+
Invariants
+
Evidence
```

For example:

```text
WorkState
```

could be:

```text
Proposed
Authorized
Ready
Scheduled
Running
Verifying
Completed
Failed
Cancelled
Aborted
```

# 37. Valid Transitions

Not every transition is legal.

For example:

```text
PROPOSED → AUTHORIZED
```

may be legal.

But:

```text
PROPOSED → COMPLETED
```

must be rejected.

This is a **state invariant**.

# 38. Transition Authority

Transitions may themselves require authority.

For example:

```text
Running → Cancelled
```

may require:

```text
operator authority
OR
safety authority
OR
owning agent authority
```

Thus lifecycle is not merely an internal implementation detail.

# 39. State Transition as an Event

A transition should generate an event:

```text
WorkStateChanged {
    work_id: W42,
    from: Running,
    to: Failed,
    reason: Fault F9,
    time: T
}
```

This gives us:

```text
state
+
history
```

rather than state alone.

# 40. State Reducer

A conceptual model:

```rust
fn apply(
    state: &mut RuntimeState,
    event: Event,
) -> Result<(), TransitionError>
```

The reducer validates:

```text
identity
authority
ordering
state transition
temporal constraints
invariants
```

before applying the event.

# 41. Event → State

This gives NROS a powerful architecture:

```text
Events
   ↓
State Transition Function
   ↓
Runtime State
```

Then:

```text
Runtime State
   ↓
Decision
   ↓
New Events
```

This forms a closed semantic loop.

# 42. Command vs Event

Another essential distinction:

```text
Command:
    request for an effect

Event:
    record of an occurrence
```

Example:

```text
Command:
    CloseValve(V42)

Event:
    ValveClosed(V42)
```

A command must never be interpreted as proof that the effect occurred.

# 43. Query

A third primitive:

```text
Query:
    request information
```

Thus NROS has:

```text
Command
Event
Query
```

with clearly different semantics.

# 44. Command Lifecycle

```text
Command
   ↓
Authorized?
   ↓
Accepted?
   ↓
Scheduled?
   ↓
Executed?
   ↓
Verified?
```

Each stage can produce evidence.

# 45. Semantic Integrity

This allows NROS to distinguish:

```text
command accepted
```

from:

```text
execution started
```

and:

```text
effect verified
```

This is a major improvement over loosely coupled command/message systems.

# 46. Canonical Runtime Graph

The whole runtime can now be visualized as:

```text
                    ┌──────────────┐
                    │   OBSERVATION│
                    └──────┬───────┘
                           ↓
                    ┌──────────────┐
                    │    BELIEF    │
                    └──────┬───────┘
                           ↓
                    ┌──────────────┐
                    │    INTENT    │
                    └──────┬───────┘
                           ↓
                    ┌──────────────┐
                    │     GOAL     │
                    └──────┬───────┘
                           ↓
                    ┌──────────────┐
                    │     PLAN     │
                    └──────┬───────┘
                           ↓
                    ┌──────────────┐
                    │   DECISION   │
                    └──────┬───────┘
                           ↓
                    ┌──────────────┐
                    │  AUTHORITY   │
                    └──────┬───────┘
                           ↓
                    ┌──────────────┐
                    │ COMMITMENT   │
                    └──────┬───────┘
                           ↓
                    ┌──────────────┐
                    │     WORK     │
                    └──────┬───────┘
                           ↓
                    ┌──────────────┐
                    │  EXECUTION   │
                    └──────┬───────┘
                           ↓
                    ┌──────────────┐
                    │   EVIDENCE   │
                    └──────┬───────┘
                           ↓
                    ┌──────────────┐
                    │ BELIEF UPDATE│
                    └──────────────┘
```

Cross-cutting:

```text
TIME ────────────────┐
CAUSALITY ───────────┤
POLICY ──────────────┤
IDENTITY ────────────┤
SUPERVISION ─────────┤
FAULT/RECOVERY ──────┘
```

# 47. The NROS State Algebra

A useful abstract formulation is:

```text
Sₜ₊₁ = Reduce(Sₜ, Eₜ)
```

where:

```text
Sₜ = runtime state at time t
Eₜ = validated event
```

But not every event is valid.

Therefore:

```text
Reduce:
    State × Event
        →
    State | TransitionError
```

# 48. Decision Algebra

Likewise:

```text
Decision =
    f(
        Beliefs,
        Goals,
        Plans,
        Policy,
        Authority,
        Resources,
        TemporalConstraints,
        Risk
    )
```

This does **not** imply that NROS itself must implement the intelligence.

It defines the contract under which intelligence operates.

# 49. Execution Algebra

Execution becomes:

```text
Execute(
    AuthorizedWork,
    Resources,
    TemporalContract
)
→
Evidence
```

Failure produces:

```text
Fault
```

rather than silently returning an ambiguous error.

# 50. Recovery Algebra

Recovery becomes:

```text
Recover(
    Fault,
    State,
    Policy,
    AvailableResources
)
→
RecoveryPlan
```

which then re-enters:

```text
Authorization
→ Commitment
→ Work
→ Execution
→ Verification
```

Thus recovery is not an architectural side road.

It is another form of planning.

# 51. A Crucial Consequence

This gives NROS a unified principle:

> **Planning and recovery are instances of the same higher-level transformation: selecting actions that transform the current state toward an acceptable target state under constraints.**

Normal planning:

```text
Current State
   ↓
Goal
   ↓
Plan
```

Recovery:

```text
Faulted State
   ↓
Recovery Goal
   ↓
Recovery Plan
```

The same runtime machinery can therefore support both.

# 52. NROS Is Not a Graph of Nodes

This is perhaps the clearest departure from ROS.

ROS's conceptual center:

```text
Node
 ↕
Topic
```

NROS's conceptual center:

```text
State
 ↕
Event
 ↕
Intent
 ↕
Decision
 ↕
Work
 ↕
Evidence
```

Communication remains necessary, but becomes an implementation mechanism rather than the primary ontology.

# 53. From Computation Graph to Agency Graph

ROS:

```text
Computation Graph
```

NROS:

```text
Agency Graph
```

An agency graph contains relationships such as:

```text
Agent
  ├── intends
  ├── believes
  ├── pursues
  ├── commits
  ├── owns
  ├── controls
  ├── uses
  ├── delegates
  └── verifies
```

This is the real conceptual transformation.

# 54. The Next Necessary Layer

We now have the objects and their state algebra.

The next question is:

> **How do multiple NROS agents coordinate without a centralized ROS1-style master?**

That requires defining:

```text
Discovery
Identity
Membership
Naming
Scopes
Namespaces
Federation
Delegation
Distributed coordination
Leader election
Authority transfer
Resource ownership
Conflict resolution
```

This leads to:

# Part LXI — NROS Distributed Coordination & Federation

The central architectural transition will be:

```text
ROS1 Master
      ↓
ROS2 discovery
      ↓
NROS federation
```

where the system is no longer merely discovering **nodes**, but discovering:

```text
agents
capabilities
resources
authority
services
belief domains
execution domains
```

and forming a **dynamic autonomous federation** rather than a static computation graph.
