# Distributed Architecture

## Purpose

NROS may operate across multiple compute nodes, robots, or network-connected components. The distributed architecture defines the conceptual boundaries for identity, coordination, discovery, communication, and replicated state.

## Responsibilities

- identify participating nodes;
- discover peers and capabilities;
- establish communication relationships;
- coordinate distributed state where required;
- isolate network failures from local execution where possible;
- make distributed behavior observable.

```text
Local Runtime ── Transport ── Local Runtime
      │                         │
      └────── Distributed Control ──────┘
```

## Coordination boundary

A distributed coordination protocol must not be inferred from the existence of node IDs, terms, peer registries, or protocol-shaped data structures. Consensus, election, replication, and failure recovery require protocol-level implementation and verification evidence.

## Current status

This document defines the architectural boundary. Repository evidence must be consulted before describing any distributed feature as a production consensus or replication implementation.
