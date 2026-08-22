# Part XIX — Formal Models, Invariants & Proof Boundaries

> **Series:** NROS Architecture Series  
> **Part:** XIX  
> **Role:** Formal state models, invariants, preconditions, postconditions, temporal properties, refinement, model checking, proof assumptions, and assurance boundaries  
> **Status:** Architectural design document — not formal-proof evidence

## 1. Purpose

Part XVIII defined empirical verification and evidence. Part XIX defines the formal-reasoning layer used to state precisely what NROS is intended to guarantee and under which assumptions those guarantees hold.

The central rule is:

> **A formal proof establishes a precisely stated property of a precisely stated model under explicitly stated assumptions; it does not automatically establish the corresponding property of the entire deployed system.**

## 2. Formal Assurance Stack

```text
System requirement
      ↓
Formal property
      ↓
Model
      ↓
Assumptions
      ↓
Proof / model checking
      ↓
Proof result
      ↓
Refinement argument
      ↓
Implementation correspondence
      ↓
System assurance
```

Each transition requires justification.

## 3. State Model

A formal state model describes relevant system state as:

```text
S = state variables + control state + resources + external conditions
```

Transitions define how the system moves between states.

```text
S₀ --event--> S₁ --event--> S₂
```

## 4. State Machines

Lifecycle and protocol behavior should be expressible as explicit state machines where sequencing matters.

```text
INIT
 ↓ start
RUNNING
 ↓ suspend
QUIESCING
 ↓ checkpoint
STOPPED
```

Invalid transitions should be defined rather than left implicit.

## 5. Preconditions

A precondition states what must hold before an operation is valid.

```text
Pre(P): authorization exists
       resource admission succeeds
       entity is in an allowed state
```

A caller cannot assume an operation is valid when its preconditions are false.

## 6. Postconditions

A postcondition states what must hold after successful completion.

```text
Post(P): state transition occurred
         result satisfies contract
         ownership is correct
         required events were emitted
```

## 7. Invariants

An invariant must remain true across all permitted transitions.

Examples:

```text
I1: entity identity is unique within its scope
I2: unauthorized operations cannot execute
I3: released resources cannot remain allocated
I4: invalid lifecycle transitions cannot be committed
I5: stale generations cannot receive current work
```

## 8. Safety Properties

Safety properties state that something bad never happens.

```text
“Nothing outside the permitted state space occurs.”
```

Examples:

```text
no unauthorized execution
no double ownership
no invalid transition
no resource-accounting violation
no stale-generation execution
```

## 9. Liveness Properties

Liveness states that something good eventually happens under specified assumptions.

Examples:

```text
accepted work eventually completes
recoverable failure eventually reaches recovery state
valid request eventually receives a response
```

Liveness always requires assumptions about fairness, resources, failures, and scheduling.

## 10. Safety vs Liveness

```text
Safety:
    bad event never occurs

Liveness:
    good event eventually occurs
```

A system can satisfy one while violating the other.

## 11. Temporal Properties

Temporal claims should explicitly define time semantics.

Examples:

```text
G(not unauthorized_execution)
F(response)
G(request → F(response))
```

Where the notation is used, its temporal logic and interpretation must be defined by the applicable formal method.

## 12. Deadline Properties

A deadline claim requires more than eventual completion:

```text
request
  ↓
completion ≤ deadline
```

The proof or verification model must define the clock, scheduler assumptions, execution bound, and environment assumptions.

## 13. Ordering Properties

Ordering may be stated formally:

```text
if A precedes B
then delivery(A) precedes delivery(B)
```

The exact scope of ordering must be defined:

```text
per producer
per stream
per entity
per connection
system-wide
```

## 14. Atomicity Properties

An atomic operation should have a clearly defined observation boundary:

```text
before
  ↓
atomic transition
  ↓
after
```

Partial intermediate states must either be unobservable or explicitly permitted.

## 15. Resource Invariants

Part VII can define formal conservation-style properties:

```text
allocated + available = capacity
```

subject to explicitly modeled reservations, fragmentation, and external resources.

## 16. Ownership Invariants

Ownership transitions should preserve exclusivity or explicitly model sharing.

```text
resource
  ↓ acquire
owner A
  ↓ release
unowned
  ↓ acquire
owner B
```

A proof must define whether concurrent ownership is permitted.

## 17. Identity Invariants

Part X identity semantics can be formalized:

```text
within scope:
identity → at most one current owner
```

If generations are used:

```text
generation(old) < generation(current)
⇒ old work is rejected
```

## 18. Authorization Invariants

Part XI can express security safety properties:

```text
execute(op, principal)
⇒ authorized(principal, op, context)
```

The proof boundary must identify the trusted authorization mechanism and the assumptions surrounding identity authenticity.

## 19. Persistence Invariants

Part XII can express durability properties such as:

```text
committed state
⇒ recoverable state
```

The meaning of “recoverable” must include the storage and corruption assumptions of the model.

## 20. Protocol Invariants

Part XVI protocol state machines can define:

```text
message accepted
⇒ message valid for current state
```

and:

```text
negotiated version
∈ supported intersection
```

## 21. Configuration Invariants

Part XVII can formalize policy resolution:

```text
effective_config
= resolve(defaults, inheritance, overrides, precedence)
```

The resolver should be deterministic where deterministic resolution is required.

## 22. Deployment Invariants

Part XV can define placement constraints:

```text
placement(entity)
∈ permitted_nodes
```

and anti-affinity constraints:

```text
replica(A) and replica(B)
∉ same_failure_domain
```

when such independence is required.

## 23. Refinement

A high-level model can be refined into lower-level models:

```text
Architectural model
       ↓ refinement
Runtime model
       ↓ refinement
Implementation model
```

A refinement argument should demonstrate preservation of the properties relevant to the contract.

## 24. Abstraction

Formal models intentionally omit irrelevant implementation details.

The abstraction must preserve every detail necessary to prove the target property.

```text
implementation
     ↓ abstraction
model
```

An abstraction that removes the failure mode being studied cannot prove its absence in the implementation.

## 25. Assumptions

Every formal claim should record assumptions such as:

```text
clock correctness
memory safety
scheduler fairness
trusted hardware
network behavior
storage guarantees
cryptographic assumptions
compiler correctness
```

Assumptions are part of the proof boundary.

## 26. Trusted Computing Base

The trusted computing base (TCB) consists of mechanisms whose correctness the assurance argument depends upon.

Examples may include:

```text
runtime kernel
compiler
cryptographic primitives
hardware isolation
storage layer
formal verifier
```

Reducing the TCB can strengthen assurance but does not automatically prove correctness.

## 27. Model Checking

Model checking can exhaustively explore a finite or appropriately bounded state space.

```text
model
 ↓
state exploration
 ↓
property check
 ↓
PASS / counterexample
```

A counterexample is valuable evidence of a violated property or inadequate assumptions.

## 28. Counterexamples

A model-checking counterexample should identify a trace such as:

```text
S0
 ↓ event A
S1
 ↓ event B
S2
 ↓ event C
S3  ← property violated
```

The trace becomes a diagnostic artifact and can inform Part XVIII testing.

## 29. Proof vs Model Checking

Formal proof and model checking are related but distinct methods.

```text
proof
    → mathematical derivation

model checking
    → systematic state-space exploration
```

The selected method must match the property and model.

## 30. Formal Specification

A formal specification should define:

```text
states
inputs
outputs
transitions
invariants
assumptions
properties
failure semantics
```

Ambiguous natural-language claims should be refined before formal verification.

## 31. Proof Obligations

A refinement or implementation may generate obligations such as:

```text
preconditions preserved
invariants preserved
postconditions satisfied
state mapping total
error behavior preserved
resource bounds preserved
```

Each obligation should have an explicit status.

## 32. Compositional Proof

Large systems may be verified compositionally:

```text
Component A → property PA
Component B → property PB
        ↓
composition assumptions
        ↓
System property P
```

The composition assumptions are essential. Independent proofs do not automatically compose.

## 33. Assume-Guarantee Reasoning

A component can be specified as:

```text
Assume A
Guarantee G
```

The composition must establish that the environment satisfies A before relying on G.

## 34. Refinement Boundaries

A formal property should identify where it stops:

```text
proved:
protocol state machine

not proved:
network driver
hardware
physical link
```

This prevents formal results from being overstated.

## 35. Implementation Correspondence

Formal verification of a model does not establish implementation correctness unless correspondence is established:

```text
Model state
   ↕ mapping
Implementation state
```

The mapping itself may require proof or independent verification.

## 36. Compiler and Tool Assumptions

If generated machine code is relied upon, the assurance argument must state its assumptions about:

```text
compiler correctness
compiler configuration
optimization behavior
linker
runtime libraries
hardware architecture
```

A verified source-level model does not automatically prove arbitrary compiled binaries correct.

## 37. Formal Proof Evidence

Formal evidence may include:

```text
proof scripts
proof terms
model files
model-checker reports
counterexample traces
verified invariants
solver certificates
reproducibility manifests
```

These artifacts should be versioned and attributable where assurance requires it.

## 38. Proof Reproducibility

Formal results should be reproducible against a declared baseline:

```text
model revision
specification revision
tool version
solver version
parameters
proof configuration
```

## 39. Formal Verification and Testing

Formal and empirical methods are complementary:

```text
Formal methods
     ↓
prove modeled properties

Testing
     ↓
observe implementation behavior
```

Formal verification should inform test generation, while empirical failures can expose incorrect models or assumptions.

## 40. Verification Status

Formal claims should distinguish:

```text
FORMALLY_SPECIFIED
MODEL_CHECKED
PROVED
PARTIALLY_PROVED
COUNTEREXAMPLE_FOUND
ASSUMPTION_UNSATISFIED
CORRESPONDENCE_UNVERIFIED
```

No stronger status should be inferred automatically.

## 41. Assurance Levels

Different claims may require different assurance levels:

```text
informational
functional
strong empirical
formal model
formal implementation correspondence
high-assurance certification
```

The required level should be determined by risk and contract.

## 42. Verification Matrix

| Property | Formal question |
|---|---|
| State | Is the relevant state space explicitly modeled? |
| Invariant | Is the property preserved across transitions? |
| Safety | Can forbidden states be proven unreachable? |
| Liveness | Under what assumptions does progress follow? |
| Temporal | Are clock and fairness assumptions explicit? |
| Identity | Is uniqueness/generation safety formalized? |
| Security | Does execution imply authorization? |
| Resources | Are accounting invariants preserved? |
| Protocol | Are invalid transitions/messages rejected? |
| Configuration | Is policy resolution deterministic and safe? |
| Deployment | Are placement constraints preserved? |
| Refinement | Is implementation correspondence established? |
| Assumptions | Are proof assumptions explicit and validated? |
| TCB | Are trusted mechanisms identified? |
| Evidence | Can the formal result be reproduced? |
| Scope | Is the boundary of the proof explicit? |

## 43. What Part XIX Does Not Claim

This Part does not claim that the current NROS implementation already has:

- a complete formal specification;
- machine-checked proofs of the architecture;
- verified implementation correspondence;
- exhaustive model checking of all distributed states;
- a formally verified compiler/runtime/OS boundary;
- proof of end-to-end system correctness.

Those are separate engineering and assurance milestones.

## 44. Transition to Part XX

Part XIX defines formal reasoning and proof boundaries.

Part XX should define **fault tolerance, resilience, failure domains, degradation, recovery objectives, and end-to-end availability**, connecting formal safety/liveness properties to real-world failure behavior.

```text
Part XVIII
Testing + conformance + verification
        ↓
Part XIX
Formal models + invariants + proof boundaries
        ↓
Part XX
Resilience + fault tolerance + availability
```

## Canonical rule

> **NROS treats every formal result as scoped to an explicit model, specification, assumptions, verification method, and correspondence boundary; a proven model property is not promoted into an end-to-end implementation guarantee without evidence that the implementation satisfies the model.**
