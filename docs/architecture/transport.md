# NROS Transport

> **Status:** Active architectural documentation.
>
> Transport describes how communication data is moved between endpoints. It does not, by itself, establish reliability, latency, zero-copy, security, or production readiness.

## 1. Purpose

Transport connects the logical communication model to a concrete delivery mechanism.

```text
Application message
        ↓
Typed communication contract
        ↓
Representation / serialization
        ↓
Transport
        ↓
Framing / delivery
        ↓
Remote or local endpoint
```

Local IPC and network transport share communication concerns but have different failure, addressing, and lifecycle boundaries.

## 2. Transport responsibilities

A transport implementation may be responsible for:

- endpoint creation and teardown;
- addressing;
- discovery or endpoint resolution;
- serialization and framing;
- connection/session management;
- delivery and retry behavior;
- ordering;
- flow control and backpressure;
- timeout and cancellation handling;
- security mechanisms where applicable.

The exact responsibilities depend on the selected transport contract.

## 3. Serialization boundary

Serialization is distinct from transport:

```text
Logical type
    ↓
Wire representation
    ↓
Frame / packet
    ↓
Transport
```

Serialization compatibility must be considered independently from network reachability. A transport can successfully deliver bytes while the receiving endpoint cannot interpret them correctly.

## 4. Discovery and addressing

Distributed communication requires a mechanism to determine where an endpoint is located and how it can be reached.

```text
Logical endpoint
      ↓
Discovery / configuration
      ↓
Address
      ↓
Connection / delivery
```

Static configuration, discovery services, registries, multicast, or other mechanisms are implementation choices. Their existence must be demonstrated separately.

## 5. Delivery semantics

Transport documentation must distinguish:

```text
Reachability
Ordering
Reliability
Duplication behavior
Loss behavior
Latency
Throughput
```

For example, reliable delivery does not imply bounded latency, and ordered delivery does not imply exactly-once processing.

## 6. Backpressure and overload

Transport must define what happens when production exceeds available delivery capacity.

```text
Producer
   ↓
Transport queue / buffer
   ↓
Network capacity
   ↓
Consumer
```

Possible policies include blocking, buffering, dropping, rejection, prioritization, or flow control. The actual policy must be documented and tested for the implemented backend.

## 7. Failure model

Important distributed failure modes include:

- endpoint unavailable;
- connection loss;
- peer restart;
- malformed frame;
- incompatible serialization;
- timeout;
- partial delivery;
- network partition;
- resource exhaustion;
- authentication or authorization failure where security is enabled.

Failure recovery must not be inferred from the existence of a reconnect or retry API.

## 8. Performance claims

Transport performance depends on the complete path:

```text
Serialization
 + allocation
 + copying
 + scheduling
 + transport
 + kernel / driver
 + network
 + deserialization
```

Therefore:

> **A fast transport primitive does not establish an end-to-end latency guarantee.**

Benchmarks must declare target hardware, operating environment, message characteristics, workload, measurement method, and observed results.

## 9. Security boundary

When transport crosses a trust boundary, authentication, authorization, confidentiality, integrity, replay resistance, and key-management concerns may become relevant.

Security architecture must be documented separately from generic transport reachability. An encrypted channel does not automatically establish endpoint authorization or application-level safety.

## 10. Simulation versus production

A simulated transport may reproduce selected communication semantics without reproducing network failures, kernel behavior, physical topology, or deployment conditions.

```text
Simulation evidence
       ≠
Production transport evidence
```

## 11. Verification requirements

| Claim | Evidence |
|---|---|
| Transport API exists | Source/interface inspection |
| Endpoints communicate | Executed integration test |
| Serialization is compatible | Cross-endpoint compatibility test |
| Ordering is preserved | Controlled ordering test |
| Reliability is provided | Failure/recovery tests |
| Latency is bounded | Targeted benchmark with declared bound |
| Zero-copy is provided | End-to-end ownership/path evidence |
| Security property exists | Protocol/configuration inspection + security test |
| Production readiness | Deployment/integration evidence |

## 12. Related documents

- [Architecture Overview](overview.md)
- [System Model](system-model.md)
- [Runtime](runtime.md)
- [IPC](ipc.md)
- [Scheduling](scheduling.md)
- [Distributed](distributed.md)
- [Verification](../verification/README.md)
- [Safety](../safety/README.md)
