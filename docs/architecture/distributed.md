# NROS Distributed Architecture

> **Status:** Active architectural documentation.
>
> This document defines the conceptual distributed-system boundary for NROS. It does not imply that discovery, clustering, federation, or fleet features are currently implemented.

## 1. Distributed model

A distributed NROS deployment extends the local runtime model across processes, machines, and networks:

```text
Node
  ↓
Process
  ↓
Machine
  ↓
Network
  ↓
Remote Machine
  ↓
Remote Process
  ↓
Remote Node
```

The logical graph may span multiple execution contexts while preserving application-level communication contracts.

## 2. Identity

Distributed operation requires stable identities for participating entities. Identity should be distinguished from:

```text
Logical name
    ≠
Process identity
    ≠
Machine identity
    ≠
Network address
```

An address may change while logical identity remains stable, depending on the discovery and lifecycle model.

## 3. Discovery

Discovery resolves logical participants to reachable endpoints:

```text
Logical participant
        ↓
Discovery / registry / configuration
        ↓
Endpoint information
        ↓
Connection
```

Static configuration, centralized registries, multicast, peer-to-peer discovery, or other mechanisms are implementation choices. Their availability must be verified independently.

## 4. Topology

A distributed deployment may contain multiple processes on one machine, multiple machines on one network, routed network segments, edge devices, simulation nodes, or remote components. Topology affects latency, failure modes, security boundaries, and resource capacity.

## 5. Remote communication

Remote communication inherits the transport concerns described in [Transport](transport.md) and adds distributed-system concerns:

```text
Serialization
 + addressing
 + discovery
 + network failure
 + peer lifecycle
 + clock differences
 + security boundary
```

Successful local IPC does not establish successful distributed communication.

## 6. Failure model

Distributed systems must assume partial failure. Relevant conditions include peer unavailability, process crash, machine failure, network partition, packet loss or delay, stale discovery information, duplicate or reordered messages, incompatible peer versions, and resource exhaustion.

The system should distinguish local failure from remote failure and expose sufficient state for recovery or safe degradation.

## 7. Consistency and coordination

Distributed coordination may require explicit treatment of configuration consistency, service availability, state replication, ordering, retries, idempotency, timeouts, and authority selection where applicable.

A retry mechanism does not automatically provide exactly-once semantics or consistency.

## 8. Time

Distributed clocks cannot be assumed to be identical:

```text
Local monotonic time
    ≠
Wall-clock time
    ≠
Remote machine time
```

Cross-machine timing guarantees require an explicit synchronization model and measured error bounds.

## 9. Security boundary

A network connection may cross a trust boundary. Distributed deployments may therefore require authentication, authorization, confidentiality, integrity, replay protection, credential lifecycle management, and endpoint isolation.

Network reachability must never be treated as authorization.

## 10. Fleet and multi-node operation

A fleet-level architecture can be represented as:

```text
                 Fleet / Control Plane
                         │
        ┌────────────────┼────────────────┐
        ↓                ↓                ↓
      Robot A          Robot B          Robot C
        │                │                │
      Nodes            Nodes            Nodes
        │                │                │
      Runtime          Runtime          Runtime
```

Fleet orchestration introduces rollout, version compatibility, health reporting, configuration distribution, and recovery policy. These should not be inferred from a local runtime implementation.

## 11. Simulation boundary

A distributed simulation can reproduce selected topology and communication behavior, but it cannot automatically reproduce physical network conditions, hardware failures, or deployment-specific security constraints.

```text
Distributed simulation
          ≠
Physical fleet validation
```

## 12. Verification requirements

| Claim | Evidence |
|---|---|
| Remote node communication works | Multi-machine integration test |
| Discovery works | Executed discovery test |
| Peer restart recovery works | Failure/recovery test |
| Ordering is preserved | Controlled distributed test |
| Cross-machine timing bound | Synchronized-target measurement |
| Version compatibility | Compatibility matrix + integration tests |
| Security property works | Security test on deployed topology |
| Fleet operation works | Multi-node operational validation |

## 13. Related documents

- [Architecture Overview](overview.md)
- [System Model](system-model.md)
- [IPC](ipc.md)
- [Transport](transport.md)
- [Simulation](simulation.md)
- [Verification](../verification/README.md)
- [Safety](../safety/README.md)
