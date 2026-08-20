# NROS Safety Specification

> **Status:** Normative safety specification.
>
> This document defines safety-related documentation boundaries and invariants for NROS. It does not constitute a safety certification, hazard analysis, or claim of suitability for a particular safety standard or controlled system.

## 1. Safety boundary

Safety requirements apply to the complete controlled system, not only to the NROS process:

```text
Application
    ↓
NROS Runtime
    ↓
Safety-relevant interface
    ↓
Hardware / actuator
    ↓
Controlled system
```

Software behavior must be evaluated together with hardware, operating environment, external protections, and human/operator interaction where applicable.

## 2. Core safety invariants

The following principles are normative for safety-relevant behavior:

1. Invalid, unavailable, stale, or ambiguous safety-critical data MUST NOT be silently treated as valid.
2. Safety-relevant state transitions MUST have explicit preconditions.
3. Safety-relevant failures MUST produce defined and observable outcomes.
4. Hardware-specific safety behavior MUST remain explicit at the hardware boundary.
5. Simulation MUST NOT be presented as evidence of physical safety.
6. Protective or emergency behavior MUST have an independently reviewable contract.
7. Safety mechanisms MUST have a defined failure response where loss of the mechanism can create additional risk.

## 3. Safe states

A safety-relevant component SHOULD define a safe or controlled state for applicable failure classes.

```text
Normal operation
      ↓
Fault detected
      ↓
Protective transition
      ↓
Defined safe / degraded state
```

The meaning of "safe" is system-specific and MUST be defined by the applicable hazard and system requirements rather than assumed from a software default.

## 4. Preconditions and interlocks

Operations capable of producing safety-relevant effects MUST define applicable preconditions and interlocks.

Examples may include:

- valid sensor state;
- actuator availability;
- communication health;
- operating mode;
- limit conditions;
- authorization state;
- watchdog health.

An API that exposes a control operation does not establish that its safety preconditions are enforced.

## 5. Fault handling

Safety-relevant failures SHOULD be classified according to their effect on the controlled system.

```text
Detection
   ↓
Classification
   ↓
Protective response
   ↓
State stabilization
   ↓
Recovery / maintenance decision
```

Relevant failures can include invalid input, stale data, lost communication, process failure, hardware failure, timing violations, resource exhaustion, and watchdog expiration.

## 6. Watchdogs and timeouts

Watchdogs and timeouts MUST have explicit semantics where they are used as safety mechanisms.

The contract should define:

- what is monitored;
- timeout threshold;
- detection behavior;
- protective response;
- reset/recovery conditions;
- behavior if the watchdog itself fails.

A timeout value alone does not establish that the resulting response is safe.

## 7. Degraded operation

If degraded operation is permitted, the specification MUST define:

- which capabilities remain available;
- which capabilities are disabled;
- transition conditions;
- operator notification requirements;
- recovery conditions;
- whether automatic recovery is permitted.

```text
Full capability
      ↓
Degraded capability
      ↓
Safe / stopped state
```

Degraded mode must not silently expand beyond its specified operating envelope.

## 8. Emergency and protective behavior

Emergency or protective functions require an independently reviewable contract covering trigger conditions, authority, response, and recovery.

Where external hardware safety mechanisms exist, their behavior MUST remain explicitly separated from software-level emergency logic.

```text
Software protection
       ≠
Hardware protection
       ≠
System-level safety function
```

## 9. Human and operator boundary

Operator interfaces, dashboards, and commands are part of the safety context when human action affects the controlled system.

The specification should define relevant authority, feedback, failure indication, and prevention of ambiguous operator state.

A UI state MUST NOT be treated as proof that a physical safety state exists.

## 10. Safety evidence

Safety claims require evidence appropriate to the claim. Evidence may include:

- requirements traceability;
- static analysis;
- unit and integration tests;
- negative-path testing;
- fault injection;
- timing analysis;
- hardware-in-the-loop testing;
- target hardware testing;
- independent review;
- applicable certification assessment.

Evidence must identify the tested revision, environment, assumptions, and scope.

## 11. Certification boundary

The following claims are distinct:

```text
Safety requirement specified
        ≠
Safety mechanism implemented
        ≠
Safety requirement verified
        ≠
System safety validated
        ≠
Safety certified
```

NROS documentation MUST NOT imply certification merely from passing software tests or implementing safety-oriented APIs.

## 12. Historical evidence

Historical audits, previous test results, and old verification records remain historical evidence unless a current process establishes their continuing validity.

Current safety status should therefore identify the applicable revision and verification date.

## 13. Verification requirements

| Claim | Evidence |
|---|---|
| Safety invariant is specified | Normative requirement |
| Invalid safety data is rejected | Negative-path tests |
| Preconditions are enforced | Boundary/authorization tests |
| Protective response works | Fault-injection or integration tests |
| Watchdog response works | Deterministic timeout/failure tests |
| Degraded mode is bounded | Mode-transition tests |
| Emergency behavior works | Independent system-level validation |
| Hardware protection works | Target hardware evidence |
| Safety claim remains current | Revision-specific verification record |
| Certification exists | Applicable independent certification evidence |

## 14. Related specifications

- [Specifications Index](README.md)
- [Types](types.md)
- [Protocols](protocols.md)
- [IPC](ipc.md)
- [Transport](transport.md)
