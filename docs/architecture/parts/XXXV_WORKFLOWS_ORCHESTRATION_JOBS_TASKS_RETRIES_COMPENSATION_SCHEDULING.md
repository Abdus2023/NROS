# Part XXXV — Workflows, Orchestration, Jobs, Tasks, Retries, Compensation & Scheduling

> **Series:** NROS Architecture Series  
> **Part:** XXXV  
> **Role:** Long-running execution, workflow state, task dependencies, scheduling, retries, deadlines, cancellation, compensation, checkpointing, and recovery  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part XXXIV established state machines, transitions, invariants, reconciliation, and convergence. Part XXXV defines the execution-coordination plane that turns those state semantics into durable workflows and orchestrated operations.

The central rule is:

> **NROS treats workflows as durable state machines over execution activities: tasks have explicit identity, dependencies, attempts, deadlines, cancellation semantics, retry policy, compensation behavior, and durable progress; orchestration must never confuse a requested operation with a completed effect.**

## 2. Fundamental Distinctions

```text
workflow
  ≠
job
  ≠
task
  ≠
operation
  ≠
execution attempt
  ≠
retry
  ≠
compensation
```

## 3. Workflow

A workflow is a durable definition and execution context for a multi-step operation:

```text
Workflow
 ├─ workflow ID
 ├─ definition/version
 ├─ execution state
 ├─ tasks
 ├─ dependencies
 ├─ policy
 ├─ deadlines
 └─ checkpoints
```

## 4. Workflow Definition

The definition describes intended structure:

```text
Tasks
 + Dependencies
 + Preconditions
 + Outputs
 + Failure policy
 + Compensation policy
```

A workflow definition is not itself evidence that execution occurred.

## 5. Workflow Execution

A workflow execution is a concrete instance of a definition:

```text
Definition V3
     ↓
Execution E42
```

Definition version and execution identity must remain distinguishable.

## 6. Job

A job is a schedulable unit of work with lifecycle state:

```text
Created
 ↓
Queued
 ↓
Running
 ↓
Succeeded / Failed / Cancelled
```

A job may contain one or more tasks depending on the domain model.

## 7. Task

A task is an independently identifiable execution step:

```text
Task
 ├─ task ID
 ├─ workflow ID
 ├─ dependencies
 ├─ inputs
 ├─ execution policy
 └─ result
```

## 8. Operation

An operation is the semantic action being requested or performed. A single operation may have multiple execution attempts.

```text
Operation O
 ├─ Attempt 1
 ├─ Attempt 2
 └─ Attempt 3
```

Attempts must not be mistaken for separate logical operations.

## 9. Execution Attempt

An attempt represents one concrete execution:

```text
Task T
 ↓
Attempt A1
 ↓ failure
Attempt A2
```

Attempt identity should be unique even when the logical task is unchanged.

## 10. Task State Machine

```text
Pending
 ↓
Ready
 ↓
Running
 ├─→ Succeeded
 ├─→ Failed
 ├─→ TimedOut
 └─→ Cancelled
```

Retry policy may transition a failed attempt back to a new attempt without changing the logical task identity.

## 11. Workflow State Machine

```text
Pending
 ↓
Running
 ├─→ Succeeded
 ├─→ Failed
 ├─→ CancelRequested
 ├─→ Cancelling
 └─→ Cancelled
```

Terminal states must not silently return to active execution.

## 12. Dependencies

Dependencies define when a task becomes eligible:

```text
A ─→ B
```

B cannot become ready until the dependency contract for A is satisfied.

## 13. Dependency Types

Possible dependency semantics include:

```text
completion
success
failure
output availability
condition
resource availability
```

A dependency must declare which condition is required.

## 14. DAG Workflows

A directed acyclic graph can express parallel execution:

```text
      A
     / \
    B   C
     \ /
      D
```

A DAG workflow must reject dependency cycles at validation time.

## 15. Dynamic Workflows

Some workflows create tasks during execution:

```text
Task A
 ↓
Discover work
 ↓
Create B, C, D
```

Dynamic task creation must be bounded and authorized.

## 16. Task Inputs

Inputs should be bound to explicit versions and schemas:

```text
Task
 + Input
 + Schema Version
```

Part XXXII governs serialization and data-contract semantics.

## 17. Task Outputs

Outputs should identify:

```text
producer task
execution attempt
schema/version
result status
content/reference
```

A successful task should not claim an output that was not durably established when durability is required.

## 18. Scheduling

Scheduling maps ready work to execution capacity:

```text
Ready Tasks
    ↓
Scheduler
    ↓
Execution Capacity
    ↓
Workers / Agents
```

Scheduling policy is distinct from workflow semantics.

## 19. Scheduling Constraints

A scheduler may consider:

```text
priority
capacity
resource requirements
affinity
anti-affinity
deadline
fairness
locality
policy
```

Constraints must not override authorization or resource ownership.

## 20. Priority

Priority influences scheduling order but does not automatically change semantic dependency ordering.

```text
Priority
 ≠
Causality
```

## 21. Fairness

A scheduler should define whether fairness is:

```text
per workflow
per tenant
per identity
per resource
per queue
```

“Fair” without a scope is insufficient.

## 22. Capacity

Execution capacity must be bounded:

```text
max concurrent tasks
CPU
memory
I/O
network
agent slots
resource quotas
```

The scheduler must avoid creating more work than downstream systems can safely process.

## 23. Backpressure

When capacity is exhausted:

```text
Ready
 ↓
Queue / Backpressure
 ↓
Run when capacity exists
```

Unbounded task creation is prohibited as a safe default.

## 24. Admission Control

Before scheduling:

```text
Task
 ↓ validate
 ↓ authorize
 ↓ resource check
 ↓ quota check
 ↓ schedule
```

Rejected work must have an explicit failure state.

## 25. Resource Binding

A task may require a resource:

```text
Task
 ↓
Resource Generation N
```

If the resource generation changes before execution, the task must revalidate or fail according to policy.

## 26. Agent Scheduling

NROS agents may be scheduled as execution units:

```text
Event / Task
    ↓
Observe
    ↓
Plan
    ↓
Execute
    ↓
Reflect
    ↓
Checkpoint
```

Agent autonomy does not remove workflow authorization or resource constraints.

## 27. Retry

Retry creates a new execution attempt for the same logical task:

```text
Task T
 ├─ Attempt 1 → failed
 ├─ Attempt 2 → failed
 └─ Attempt 3 → succeeded
```

## 28. Retryable vs Non-Retryable Errors

Errors should be classified:

```text
transient
capacity-related
network-related
conflict
validation
authorization
permanent application failure
```

Authorization and invalid-input failures should not normally be retried blindly.

## 29. Retry Policy

A retry policy may define:

```text
max attempts
backoff
jitter
retryable classes
retry deadline
per-attempt timeout
```

## 30. Exponential Backoff

A common policy is:

```text
Delay(n) = min(MaxDelay, Base × 2^n)
```

Jitter should be considered to avoid synchronized retry storms.

## 31. Retry Storm Prevention

Retries must consume bounded resources:

```text
retry budget
concurrency limit
backoff
circuit breaker / suppression
```

A failed dependency must not trigger unlimited recursive retries.

## 32. Idempotency

Retries are safe only when the logical operation is idempotent or protected by an idempotency mechanism:

```text
Idempotency Key
 + Operation Identity
      ↓
Safe Retry Semantics
```

Part XXXIV transition semantics govern state effects.

## 33. Exactly-Once Effects

A retry loop cannot establish exactly-once effects merely by counting attempts.

Exactly-once effects require coordination between:

```text
execution
 + effect commitment
 + durable progress
```

## 34. Timeout

A timeout limits execution duration:

```text
Start
 ↓
Deadline
 ↓
Timeout
```

Timeout detection does not prove that the underlying external operation stopped.

## 35. Deadline

A deadline is an absolute completion boundary:

```text
Now < Deadline → eligible
Now ≥ Deadline → expired
```

Deadlines should propagate to downstream operations where supported.

## 36. Timeout vs Deadline

```text
Timeout → duration constraint
Deadline → absolute time constraint
```

They must not be conflated.

## 37. Cancellation

Cancellation requests termination:

```text
Running
 ↓ cancel request
Cancelling
 ↓
Cancelled
```

Cancellation is a state transition, not merely a transport message.

## 38. Cooperative Cancellation

Tasks should periodically observe cancellation:

```text
Execution
 ↓ cancellation checkpoint
Stop safely
 ↓
Cancelled
```

## 39. Forced Cancellation

If safe cooperative cancellation fails, a stronger termination mechanism may be required.

Forced termination must define cleanup and resource reclamation semantics.

## 40. Cancellation Races

Cancellation can race with completion:

```text
Complete ─┐
          ├─ race
Cancel ───┘
```

The authority must define which committed outcome wins and make the result observable.

## 41. Compensation

Compensation reverses or counteracts a previously committed effect when direct rollback is impossible:

```text
T1 → T2 → T3
          ↓ failure
      Compensate T2
      Compensate T1
```

Compensation is not automatically equivalent to rollback.

## 42. Compensation Task

A compensation action is itself a governed task:

```text
Compensation
 + identity
 + authorization
 + retry policy
 + timeout
 + result
```

It can fail and therefore requires explicit recovery semantics.

## 43. Saga

Long-running workflows may use saga semantics:

```text
Step A
 ↓
Step B
 ↓
Step C fails
 ↓
Compensate B
 ↓
Compensate A
```

The architecture must define which effects are compensatable.

## 44. Partial Completion

A workflow may have durable partial progress:

```text
A ✓
B ✓
C ✗
D pending
```

The system must preserve enough state to resume, compensate, or terminate safely.

## 45. Checkpointing

Long-running workflows should checkpoint durable progress:

```text
Workflow
 ↓
Checkpoint @ step N
 ↓
Continue
```

Checkpoint contents must identify workflow definition version and execution position.

## 46. Recovery

After failure:

```text
Persisted Workflow State
        ↓
Validate
        ↓
Recover Runnable Tasks
        ↓
Resume / Compensate / Fail
```

Recovery must not duplicate effects that were already committed.

## 47. Orchestrator Failure

The orchestrator itself may fail:

```text
Workflow
 ↓
Orchestrator crash
 ↓
Recovery
```

Durable workflow state and fencing are required if multiple orchestrators may resume the same execution.

## 48. Orchestrator Leadership

Multiple orchestrators require authority coordination:

```text
Epoch 7 → active
Epoch 6 → fenced
```

An old orchestrator must not continue issuing authoritative actions after losing ownership.

## 49. Task Leasing

A scheduler may lease a task:

```text
Task T
 ↓ lease
Worker W
 ↓
execute
```

Lease expiry must define whether another worker may safely retry the task.

## 50. Duplicate Workers

A lease race can produce:

```text
Worker A → execute
Worker B → retry
```

Side effects therefore require idempotency, fencing, or another coordination mechanism.

## 51. Workflow Events

Workflow state transitions should emit events where observability or downstream reaction requires them:

```text
TaskStarted
TaskCompleted
TaskFailed
TaskCancelled
WorkflowCompleted
```

Part XXXIII governs event identity and delivery semantics.

## 52. Workflow State vs Event History

The durable workflow state is a current execution representation; events may provide historical evidence:

```text
Current Workflow State
        ≠
Historical Event Stream
```

They may be correlated through stable IDs and positions.

## 53. Workflow Versioning

A running workflow must retain its definition version:

```text
Workflow execution E
 → Definition V3
```

Changing the definition must not silently mutate an already-running execution unless explicitly supported.

## 54. Migration of Running Workflows

If migration is supported:

```text
Execution V3
 ↓ validate migration
Execution V4
```

Migration requires explicit compatibility and recovery rules.

## 55. Human Intervention

Some workflows may require operator decisions:

```text
Paused
 ↓
Awaiting approval
 ↓
Resume / Reject / Cancel
```

Human actions remain authenticated, authorized, and auditable.

## 56. Approval Gates

Approval may be a task dependency:

```text
Task A
 ↓
Approval Gate
 ↓ approved
Task B
```

Approval must not be represented as an implicit bypass around normal policy checks.

## 57. Workflow Security

Workflow execution requires:

```text
Actor identity
 + capabilities
 + resource scope
 + policy
 + task authorization
```

A workflow's authority must not automatically exceed that of its initiating actor unless explicit delegation exists.

## 58. Secret Handling

Workflow state must avoid exposing secrets in:

```text
logs
checkpoint metadata
event payloads
failure messages
retry diagnostics
```

Secret references should be preferred over copying secret material into durable orchestration state.

## 59. Deterministic Scheduling

Where reproducibility matters, scheduling decisions should be derived from explicit deterministic inputs:

```text
Ready Set
 + Priority
 + Resource State
 + Policy
 + Tie-breaker
      ↓
Deterministic choice
```

## 60. Non-Deterministic Scheduling

Where scheduling is intentionally adaptive, the architecture should record sufficient decision metadata for diagnosis and replay where required.

## 61. Queue Semantics

A queue should define:

```text
ordering
visibility timeout
retention
capacity
acknowledgement
redelivery
priority
```

“Queue” alone is not a complete semantic contract.

## 62. Fair Queuing

If multiple workflows share capacity:

```text
Workflow A ─┐
Workflow B ─┼→ Scheduler
Workflow C ─┘
```

The fairness policy should prevent starvation while respecting priorities and quotas.

## 63. Deadlines and Admission

A task whose remaining deadline is shorter than its minimum viable execution budget may be rejected before scheduling.

This avoids consuming capacity on work that cannot satisfy its own contract.

## 64. Workflow Cancellation Propagation

Cancellation may propagate:

```text
Workflow Cancel
      ↓
Task A cancel
Task B cancel
Task C cancel
```

Propagation rules must account for already-completed tasks and compensation requirements.

## 65. Failure Domains

Failures should be classified by scope:

```text
task
worker
resource
network
orchestrator
workflow
cluster
external dependency
```

Recovery policy should match the failure domain rather than retrying everything identically.

## 66. Circuit Breaking

Repeated dependency failures may temporarily prevent new work:

```text
Healthy
 ↓ failures
Open
 ↓ cooldown
Half-open
 ↓ success
Healthy
```

Circuit state itself is durable or coordinated when global behavior depends on it.

## 67. Rate Limits and Quotas

Workflow scheduling must respect:

```text
per actor
per workflow
per resource
per tenant
per dependency
```

Quota exhaustion should produce an explicit scheduling state rather than an ambiguous timeout.

## 68. Workflow Garbage Collection

Terminal workflows may eventually be archived or deleted according to retention policy.

Deletion must not occur while required audit, replay, or recovery evidence remains within its mandated retention window.

## 69. Observability

Useful workflow telemetry includes:

```text
workflow ID
execution ID
task ID
attempt ID
state/version
scheduler decision
worker identity
deadline
retry count
failure class
compensation state
```

Telemetry must remain subject to security and privacy constraints.

## 70. Formal Workflow Invariant

```text
Terminal(Task)
    ⇒
NoNewAttempt(Task)
```

unless an explicitly defined administrative reset/reopen transition exists.

## 71. Formal Retry Invariant

```text
AttemptCount(Task) ≤ MaxAttempts(Task)
```

and retries must obey the applicable deadline and resource budget.

## 72. Formal Dependency Invariant

```text
TaskReady(T)
    ⇒
AllRequiredDependencies(T) = Satisfied
```

## 73. Formal Recovery Invariant

```text
Recover(WorkflowState)
    ⇒
RecoveredState satisfies workflow invariants
```

Recovery is not successful if it creates duplicate committed effects or violates dependency semantics.

## 74. Formal Cancellation Invariant

```text
Cancelled(Task)
    ⇒
NoFutureExecution(Task)
```

except for explicitly authorized cleanup or compensation actions.

## 75. Verification Matrix

| Property | Verification question |
|---|---|
| Workflow identity | Is each execution uniquely identifiable? |
| Definition | Is workflow version explicit? |
| Tasks | Are task and attempt identities distinct? |
| Dependencies | Are readiness conditions explicit? |
| Scheduling | Are capacity and fairness rules defined? |
| Admission | Are authorization/quota checks before execution? |
| Retry | Are retryable errors and budgets explicit? |
| Idempotency | Are retried effects safe? |
| Timeout | Is duration distinct from deadline? |
| Cancellation | Is cancellation a governed state transition? |
| Compensation | Are compensating effects explicit and retryable? |
| Checkpoint | Is durable progress recoverable? |
| Recovery | Can orchestrator failure resume safely? |
| Fencing | Can stale orchestrators/workers be prevented from writing? |
| Versioning | Is running workflow definition version stable? |
| Security | Is authority scoped and auditable? |
| Secrets | Are sensitive values excluded from durable diagnostics? |
| Observability | Can execution history be reconstructed? |
| Formal assurance | Are dependency, retry, recovery, and cancellation invariants explicit? |

## 76. What Part XXXV Does Not Claim

This Part does not claim that the current NROS implementation already has:

- a universal workflow engine;
- production-grade durable orchestration;
- distributed task scheduling;
- exactly-once effects;
- automatic saga compensation;
- global scheduler fairness;
- complete cancellation propagation;
- universal workflow migration;
- production-grade task leasing;
- formally verified orchestration recovery.

Those require implementation-specific evidence.

## 77. Transition to Part XXXVI

Part XXXV establishes durable workflow and execution coordination semantics.

Part XXXVI should define **time, clocks, timers, deadlines, leases, temporal ordering, clock uncertainty, monotonic time, scheduling time, and temporal correctness**, providing the temporal foundation required by workflows, retries, distributed coordination, and agent execution.

```text
Part XXXIV
State machines + transitions + invariants + reconciliation + convergence
        ↓
Part XXXV
Workflows + orchestration + jobs + tasks + retries + compensation + scheduling
        ↓
Part XXXVI
Time + clocks + timers + deadlines + leases + temporal correctness
```

## Canonical rule

> **NROS treats orchestration as durable execution state: workflows, tasks, attempts, schedules, retries, cancellations, deadlines, and compensations are explicit state transitions with bounded resources, stable identity, authorization, checkpointing, and recovery semantics; no scheduler or retry mechanism may claim completion without evidence of the corresponding committed effect.**
