# NROS Protocol Specification

> **Status:** Normative specification.
>
> A protocol defines the legal observable interactions between participants. Concrete protocol instances MUST define their own state machines and compatibility rules.

## 1. Protocol model

A protocol is more than a collection of messages. It defines:

```text
Participants
     ↓
Messages / Events
     ↓
State
     ↓
Legal transitions
     ↓
Outcomes / Errors
```

A message that is syntactically valid may still be invalid in the current protocol state.

## 2. Participants and roles

Every protocol SHOULD identify:

- participants;
- roles;
- authority or ownership where relevant;
- connection/session identity;
- responsibilities at each state.

Roles MUST NOT be inferred solely from endpoint names when the distinction affects safety, authorization, or state transitions.

## 3. State machines

Protocol behavior SHOULD be represented as an explicit state machine:

```text
             +----------+
             |  Initial |
             +----+-----+
                  |
                  v
             +----+-----+
             |  Active |
             +--+---+---+
                |   |
          fail  |   | complete
                v   v
             Failed Done
```

For each transition, the specification should define:

- triggering event/message;
- preconditions;
- state change;
- emitted result/event;
- postconditions;
- failure behavior.

## 4. Preconditions and postconditions

Protocol operations MUST distinguish what must be true before an operation from what becomes true after successful completion.

```text
Preconditions
      ↓
Operation
      ↓
Postconditions
```

A response indicating receipt does not automatically mean that the requested state transition completed.

## 5. Ordering

Where ordering matters, the protocol MUST define the relevant scope:

- per sender;
- per endpoint;
- per stream/topic;
- per session;
- global ordering, if actually required.

Ordering guarantees MUST NOT be inferred from the implementation's current queue behavior unless that behavior is part of the normative contract.

## 6. Timeouts and retries

Protocols involving waiting or remote participants SHOULD define timeout semantics explicitly.

Retry behavior must define, where applicable:

- retry trigger;
- retry limit;
- backoff;
- cancellation;
- duplicate handling;
- idempotency requirements.

```text
Timeout
  ↓
Retry?
 ├── no → fail / recover
 └── yes → retry policy
```

Retries do not automatically imply exactly-once semantics.

## 7. Cancellation

Cancellation should define whether an operation:

- stops immediately;
- completes the current atomic step;
- remains cancellable after dispatch;
- produces a cancellation result;
- may still emit late events.

A cancellation request is not necessarily proof that the underlying operation has stopped.

## 8. Errors and recovery

Protocol errors SHOULD distinguish:

```text
Invalid request
    ≠
Invalid state
    ≠
Timeout
    ≠
Transport failure
    ≠
Peer failure
    ≠
Internal failure
```

Recovery behavior should specify which states remain valid after each failure.

## 9. Compatibility and versioning

Protocol versions must define compatibility rules for:

- messages;
- states;
- required fields;
- optional extensions;
- error codes;
- capability negotiation.

Matching protocol names or version strings alone do not establish interoperability.

## 10. Observability

Protocol implementations SHOULD expose sufficient information to reconstruct relevant state transitions during verification and diagnosis, including where appropriate:

- participant identity;
- protocol/session identity;
- state;
- transition/event;
- timestamp or sequence information;
- error outcome.

Observability data is evidence about protocol execution; it does not replace the protocol contract.

## 11. Conformance

A protocol implementation conforms only when its externally observable behavior satisfies the normative state and transition rules.

The following are insufficient on their own:

- protocol-shaped structs;
- enums representing states;
- endpoint definitions;
- partial branch tests;
- documentation claims.

## 12. Verification requirements

| Claim | Evidence |
|---|---|
| State machine implemented | Transition coverage tests |
| Preconditions enforced | Negative-path tests |
| Postconditions hold | State/result assertions |
| Ordering guaranteed | Controlled ordering tests |
| Timeout semantics work | Deterministic timeout tests |
| Retry semantics work | Failure/retry integration tests |
| Cancellation is effective | Cancellation race tests |
| Recovery is correct | Fault/recovery scenarios |
| Cross-version compatibility | Compatibility matrix + integration tests |
| Protocol is fully conformant | Complete state/transition verification |

## 13. Related specifications

- [Specifications Index](README.md)
- [Types](types.md)
- [IPC](ipc.md)
- [Transport](transport.md)
- [Safety](safety.md)
