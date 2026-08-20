# NROS Transport Specification

> **Status:** Normative specification.
>
> Transport defines how communication is moved between endpoints. It MUST remain separate from application semantics and MUST NOT imply performance or deployment properties that are not explicitly specified and verified.

## 1. Endpoint addressing

A transport MUST define how an endpoint is identified and addressed.

```text
Logical endpoint
      ↓
Address resolution
      ↓
Transport endpoint
```

Addressing semantics should define scope, validity, reuse, and behavior when an address becomes unreachable.

## 2. Connection and session lifecycle

Where a transport uses connections or sessions, its lifecycle MUST be defined:

```text
Create
  ↓
Connect / Accept
  ↓
Active
  ↓
Close / Failure
```

Reconnect behavior, session identity, and stale-session handling should be explicit where applicable.

## 3. Message boundaries and framing

Transport MUST define message or frame boundaries.

The wire representation, framing rules, maximum frame size, malformed-frame behavior, and compatibility expectations should be specified where relevant.

## 4. Delivery semantics

The transport contract MUST state applicable delivery properties:

- loss behavior;
- duplication behavior;
- ordering scope;
- acknowledgement behavior;
- delivery completion semantics.

These properties are independent:

```text
Delivery
  ≠ Ordering
  ≠ Reliability
  ≠ Exactly-once
```

## 5. Backpressure

Transport MUST define behavior when producers exceed available capacity where overload can affect correctness.

Possible policies include blocking, bounded buffering, rejection, dropping, prioritization, or explicit flow control.

The selected policy must be observable at the API boundary where callers need to react to overload.

## 6. Timeouts and retries

Transport operations involving remote endpoints SHOULD define timeout semantics.

Retry behavior, when supported, should specify:

- retryable failures;
- retry count or budget;
- backoff;
- cancellation;
- duplicate handling;
- interaction with idempotency.

A retry API does not establish exactly-once delivery.

## 7. Error semantics

Transport errors MUST remain distinguishable from successful application results.

Relevant classes may include:

```text
Address failure
    ≠
Connection failure
    ≠
Timeout
    ≠
Protocol violation
    ≠
Peer failure
    ≠
Resource exhaustion
```

## 8. Shutdown

Transport shutdown MUST define the treatment of:

- queued frames;
- active operations;
- blocked senders/receivers;
- outstanding acknowledgements;
- reconnect attempts;
- peer notifications.

A shutdown request does not automatically establish delivery completion or cancellation of all in-flight work.

## 9. Compatibility

Transport implementations exchanging structured data MUST define compatibility expectations for:

- framing;
- encoding;
- protocol version;
- required/optional fields;
- extension behavior;
- error representation.

Wire compatibility is distinct from source compatibility and ABI compatibility.

## 10. Performance and implementation properties

Claims such as the following require separate evidence:

- zero-copy;
- bounded allocation;
- low latency;
- bounded latency;
- high throughput;
- deterministic delivery;
- real-time suitability.

```text
API contract
     ↓
Implementation
     ↓
Measurement
     ↓
Verification claim
```

An efficient-looking API does not establish an end-to-end performance property.

## 11. Conformance

A transport adapter is conformant only to the extent that its externally observable behavior satisfies the applicable transport contract.

The following are not sufficient by themselves:

- an adapter type;
- a socket/channel wrapper;
- a transport trait;
- benchmark numbers without a declared environment;
- documentation claims.

## 12. Verification requirements

| Claim | Evidence |
|---|---|
| Addressing works | Endpoint integration tests |
| Lifecycle is correct | Connect/disconnect/reconnect tests |
| Framing is correct | Boundary and malformed-frame tests |
| Ordering is guaranteed | Controlled ordering tests |
| Delivery semantics hold | Loss/duplication/failure tests |
| Backpressure works | Capacity/exhaustion tests |
| Timeout semantics work | Deterministic timeout tests |
| Retry semantics work | Failure/retry tests |
| Shutdown is correct | In-flight shutdown tests |
| Compatibility works | Cross-version integration matrix |
| Performance property holds | Declared benchmark + repeatable evidence |

## 13. Related specifications

- [Specifications Index](README.md)
- [Types](types.md)
- [Protocols](protocols.md)
- [IPC](ipc.md)
- [Safety](safety.md)
