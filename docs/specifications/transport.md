# Transport Specification

## Purpose

Transport is the abstraction through which NROS communication moves between endpoints. It must remain separate from application semantics.

## Contract

A transport implementation should define:

- endpoint addressing;
- connection or session lifecycle;
- message boundaries;
- delivery semantics;
- ordering guarantees;
- backpressure behavior;
- timeout and retry behavior;
- error reporting;
- shutdown behavior.

## Performance claims

Transport properties such as zero-copy, latency, throughput, bounded allocation, or deterministic delivery are verification claims, not consequences of an API design alone.

## Verification boundary

A transport adapter is conformant only to the extent that its documented guarantees are demonstrated by implementation and test evidence.
