# Part LIX — Data Models, Schemas, Serialization & Evolution

> **Series:** NROS Architecture Series  
> **Part:** LIX  
> **Role:** Data models, schemas, canonical representations, serialization, validation, versioning, migration, evolution, compatibility, and data governance  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part LVIII established API and RPC contracts. Part LIX defines the data-contract plane underneath those APIs and above durable state.

The central rule is:

> **NROS treats data shape, data meaning, serialization, persistence representation, and compatibility as separate concerns; schema compatibility does not automatically imply semantic compatibility.**

## 2. Data Contract Stack

```text
Domain Meaning
 ↓
Data Model
 ↓
Schema
 ↓
Canonical Representation
 ↓
Serialization
 ↓
Validation
 ↓
Storage / Transport
```

## 3. Domain Model

A domain model defines concepts and relationships independently of their wire or storage encoding.

```text
Domain Entity
    ≠
Wire Object
    ≠
Storage Record
```

## 4. Data Model

A data model identifies:

```text
entities
attributes
relationships
invariants
identity
lifecycle
ownership
```

## 5. Schema

A schema describes an accepted representation and its constraints.

A schema is not the complete domain model.

## 6. Semantic Invariants

Constraints that cannot be expressed by syntax alone must remain explicit:

```text
field relationship
state relationship
cross-object invariant
authorization condition
resource invariant
```

## 7. Identity

Every durable or externally referenced entity should have an explicit identity model.

Possible identities include:

```text
stable_id
namespace + id
composite key
content identity
revision identity
```

## 8. Identity vs Address

```text
Entity Identity
    ≠
Network Address
```

An entity may move while retaining identity.

## 9. Identity vs Version

```text
entity_id
    ≠
entity_revision
```

The revision identifies a particular state/version, not a different entity.

## 10. Required Fields

Requiredness is semantic contract, not merely parser behavior.

A field may be syntactically optional but semantically required under a particular state.

## 11. Optionality

The schema should distinguish:

```text
required
optional
nullable
conditionally required
computed
deprecated
```

## 12. Nullability

```text
missing
 ≠
null
 ≠
empty
```

These representations must not be conflated when they have different meanings.

## 13. Defaults

Defaults belong to the contract and should be deterministic.

A server-side default must not silently conflict with client assumptions.

## 14. Enumerations

Enumerated values should define:

```text
identifier
meaning
allowed transitions
unknown-value behavior
```

## 15. Unknown Enum Values

Forward-compatible consumers may preserve unknown values rather than failing, where the contract permits.

Authoritative safety decisions must not silently accept unknown semantics.

## 16. Tagged Variants

Variants should carry explicit discriminators where ambiguity would affect interpretation.

```text
kind = "X"
```

is preferable to inference from incidental fields when stable evolution matters.

## 17. Numeric Semantics

Schemas should define:

```text
signedness
width
range
units
precision
overflow behavior
```

## 18. Units

Physical quantities must define units explicitly.

```text
1000
```

is not meaningful without knowing whether it represents milliseconds, bytes, meters, or another unit.

## 19. Time

Time values should define:

```text
time scale
precision
zone / offset semantics
monotonic vs wall-clock meaning
```

## 20. Duration

Durations should be represented independently from timestamps where appropriate.

```text
Timestamp
    ≠
Duration
```

## 21. Ordering

Schema ordering must not be confused with semantic ordering.

Object field order is generally not an ordering guarantee unless explicitly specified.

## 22. Collections

Collection semantics should specify:

```text
ordered / unordered
unique / duplicate-allowed
maximum size
nullability
identity semantics
```

## 23. Maps

Map keys should have deterministic encoding and uniqueness rules.

## 24. Canonical Representation

Where hashing, signing, comparison, or reproducible evidence depends on serialized data, NROS should define a canonical representation.

## 25. Canonicalization

Canonicalization may define:

```text
field ordering
number formatting
string encoding
escaping
omission rules
normalization
```

## 26. Canonicalization vs Pretty Printing

Human-readable formatting is not necessarily canonical serialization.

```text
Pretty JSON
    ≠
Canonical JSON
```

## 27. Serialization

Serialization maps structured data into a transportable representation.

```text
Model
 ↓
Serializer
 ↓
Bytes
```

## 28. Deserialization

Deserialization must reject representations that violate declared encoding constraints.

## 29. Serialization Formats

NROS may support multiple formats where needed:

```text
JSON
CBOR
MessagePack
Protobuf
binary domain formats
```

The selected format must be contract-governed rather than accidental.

## 30. Format vs Semantics

Changing serialization format does not necessarily change domain semantics.

Conversely, preserving a format does not guarantee semantic compatibility.

## 31. Wire Encoding

Wire encoding should specify:

```text
encoding
framing
compression
limits
character set
canonicalization
```

## 32. Compression

Compression is an encoding concern, not a semantic transformation.

Compressed payloads remain subject to decompression resource limits.

## 33. Resource Limits

Schema processing should bound:

```text
payload size
field size
collection length
nesting depth
string length
allocation budget
```

## 34. Parser Safety

Malformed input must fail safely without unbounded memory, CPU, or recursion consumption.

## 35. Validation Layers

Validation should distinguish:

```text
encoding validation
schema validation
type validation
semantic validation
policy validation
state validation
```

## 36. Validation Timing

Validation may occur at:

```text
ingress
API boundary
service boundary
storage boundary
execution boundary
```

Critical invariants should be enforced at the authoritative boundary.

## 37. Validation vs Authorization

```text
Valid Data
    ≠
Authorized Operation
```

A perfectly valid request may still be forbidden.

## 38. Schema Registry

A schema registry may track:

```text
schema_id
version
owner
status
format
compatibility policy
security classification
```

## 39. Schema Ownership

Each authoritative schema should have an owner responsible for semantic evolution.

## 40. Schema Status

Possible states include:

```text
draft
active
deprecated
retired
```

## 41. Version Identity

Schema versions should be explicit and machine-readable.

## 42. Versioning Strategy

Versioning may be represented through:

```text
major/minor version
schema identifier
media type
namespace
feature negotiation
```

## 43. Backward Compatibility

A newer producer should remain consumable by an older consumer only when the declared compatibility contract permits it.

## 44. Forward Compatibility

An older producer may be consumed by a newer consumer when missing information has defined safe semantics.

## 45. Reader / Writer Compatibility

Compatibility should be evaluated as:

```text
Writer Schema
      ×
Reader Schema
```

rather than judging schemas independently.

## 46. Additive Fields

Adding optional fields can be backward-compatible when unknown fields are safely handled.

But semantic effects still require review.

## 47. Removing Fields

Removing a field is safe only when no supported consumer depends on it or an explicit migration exists.

## 48. Changing Field Meaning

Changing the meaning of an existing field is generally a semantic breaking change even when its type remains unchanged.

## 49. Changing Types

Type changes require explicit compatibility analysis.

```text
int32 → int64
```

may be representationally compatible in one direction while still affecting semantics or generated clients.

## 50. Renaming Fields

A rename may be wire-breaking even when the underlying concept is unchanged.

Aliases or migration periods may be required.

## 51. Enum Evolution

Adding enum values requires consumers to define unknown-value behavior.

## 52. Variant Evolution

Adding a new variant can break exhaustive consumers.

Compatibility therefore depends on consumer handling rules.

## 53. Constraint Tightening

Making a previously accepted value invalid is a breaking change for affected producers.

## 54. Constraint Relaxation

Accepting more values is often backward-compatible for writers but may alter downstream assumptions.

Semantic effects must still be evaluated.

## 55. Precision Changes

Changing numeric or timestamp precision may alter equality, ordering, or replay behavior.

## 56. Representation Changes

A representation change should preserve semantic meaning where compatibility is claimed.

## 57. Migration

Migrations transform existing data between schema versions.

```text
Schema V1
 ↓ migration
Schema V2
```

## 58. Online Migration

Online migration may require dual-read or dual-write periods.

```text
Old Writer
 ↓
Compatibility Layer
 ↓
New Representation
```

## 59. Dual Write

Dual writing can create divergence.

Reconciliation and failure semantics must therefore be explicit.

## 60. Dual Read

During migration, readers may support both old and new representations with deterministic precedence.

## 61. Backfill

Backfills should be resumable, observable, bounded, and idempotent where possible.

## 62. Migration Checkpoint

Long-running migrations should persist progress so interrupted execution can resume safely.

## 63. Migration Failure

A failed migration must not leave the system claiming a schema state that has not actually been established.

## 64. Rollback

Rollback must distinguish:

```text
code rollback
schema rollback
data rollback
traffic rollback
```

They are not automatically equivalent.

## 65. Irreversible Migration

Destructive migrations should explicitly declare irreversibility and require stronger evidence before execution.

## 66. Data Loss

Schema evolution must identify whether any migration can:

```text
truncate
round
remove
merge
reinterpret
```

data.

## 67. Data Lineage

Important derived data should retain lineage where auditability or reproducibility requires it.

```text
Source
 ↓
Transformation
 ↓
Derived Data
```

## 68. Provenance

Provenance may identify:

```text
producer
schema version
source revision
transformation
creation time
```

## 69. Content Identity

Content-addressed objects may use canonical serialized content as the basis for identity.

```text
Canonical Data
 ↓
Hash
 ↓
Content Identity
```

## 70. Hashing

Hashes are meaningful only when the input representation and canonicalization rules are stable.

## 71. Signing

Digital signatures should sign a well-defined canonical representation rather than an ambiguous human-readable form.

## 72. Encryption

Encryption protects representation according to the cryptographic boundary; it does not replace schema validation.

## 73. Redaction

Sensitive fields may require redaction before logging, tracing, or evidence export.

## 74. Data Classification

Schemas should support classification such as:

```text
public
internal
confidential
restricted
secret
```

where required by governance.

## 75. Retention

Data contracts may define retention constraints independently from storage implementation.

## 76. Deletion Semantics

Deletion should distinguish:

```text
logical deletion
physical deletion
redaction
expiration
cryptographic erasure
```

## 77. Tombstones

Tombstones can preserve deletion semantics across eventually consistent replicas.

## 78. Snapshot Semantics

A snapshot should identify the state boundary it represents:

```text
revision
timestamp
epoch
dataset version
```

## 79. Incremental Updates

Patch or delta formats should define whether operations are:

```text
ordered
idempotent
commutative
version-checked
```

## 80. Merge Semantics

Concurrent updates require explicit conflict semantics.

Possible approaches:

```text
reject
last-writer-wins
field merge
CRDT-style merge
application-defined merge
```

## 81. Conflict Detection

Conflicts should be explicit rather than silently overwritten when correctness requires detection.

## 82. Determinism

Serialization and canonicalization should be deterministic wherever reproducibility, hashing, signing, or evidence depends on them.

## 83. Reproducibility

The same semantic input under the same schema and canonicalization rules should produce equivalent canonical output.

## 84. Floating Point

Floating-point values require explicit semantics where exact reproducibility or financial/safety correctness matters.

## 85. Locale

Data representations must not depend implicitly on locale-specific formatting.

## 86. Text Encoding

Text should define its encoding, normalization expectations, and invalid-sequence behavior.

## 87. Unicode Normalization

Identifiers and human text may require explicit Unicode normalization rules when equality matters.

## 88. Ordering & Locale

Lexicographic ordering must define whether comparison is byte-based, Unicode-code-point-based, locale-aware, or another declared scheme.

## 89. Generated Code

Generated models and serializers are derived artifacts.

The schema remains the semantic source of truth.

## 90. Schema-to-Code Drift

Generated artifacts should be reproducibly derivable from the authoritative schema.

## 91. Contract Testing

Schema compatibility tests should evaluate representative reader/writer pairs.

## 92. Golden Data

Canonical fixtures can verify:

```text
serialization
parsing
canonicalization
migration
compatibility
```

## 93. Fuzzing

Parsers should be fuzz-tested against malformed and adversarial representations.

## 94. Differential Testing

Multiple implementations may be compared against the same canonical fixtures to detect semantic divergence.

## 95. Migration Testing

Migration tests should cover:

```text
oldest supported version
current version
future-compatible fixtures
partial migration
interrupted migration
rollback conditions
```

## 96. Evidence

Schema claims should connect to:

```text
schema source
implementation
generated artifacts
tests
CI evidence
migration evidence
runtime observations
```

## 97. Formal Representation Invariant

```text
Canonicalize(X) = C
 ∧
Canonicalize(X) = C
```

under identical canonicalization rules, repeated canonicalization must be deterministic.

## 98. Formal Compatibility Invariant

```text
Read(NewWrite(X), OldReader)
```

is valid only when the declared compatibility policy proves that the older reader can safely interpret the representation.

## 99. Formal Migration Invariant

```text
Migrate(V1 → V2)
    ⇒
SchemaV2Valid
 ∧
DeclaredSemanticPreservation
```

unless the migration explicitly declares a semantic transformation.

## 100. Formal Identity Invariant

```text
SameEntity(E)
 ∧
NewRevision(R2)
    ⇒
Identity(E) Stable
```

while revision identity changes.

## 101. Verification Matrix

| Property | Verification question |
|---|---|
| Domain model | Are concepts independent of encoding? |
| Schema | Are representation constraints explicit? |
| Identity | Are identity and revision distinct? |
| Optionality | Are missing/null/empty semantics defined? |
| Units | Are physical units explicit? |
| Time | Are time and duration semantics explicit? |
| Canonicalization | Is canonical output deterministic? |
| Serialization | Is the wire format contract explicit? |
| Limits | Are parser/resource bounds enforced? |
| Validation | Are semantic invariants validated? |
| Compatibility | Are reader/writer pairs evaluated? |
| Evolution | Are breaking changes identified? |
| Migration | Are migrations resumable and verifiable? |
| Rollback | Are code/schema/data rollbacks distinguished? |
| Provenance | Is lineage retained where required? |
| Security | Are classification and redaction rules explicit? |
| Deletion | Are deletion semantics defined? |
| Concurrency | Are merge/conflict rules explicit? |
| Determinism | Is reproducibility testable? |
| Evidence | Can schema claims be independently verified? |

## 102. What Part LIX Does Not Claim

This Part does not claim that the current NROS implementation already has:

- a complete production schema registry;
- universal canonical serialization;
- automated compatibility enforcement for every schema;
- zero-downtime migrations for every datastore;
- universal dual-read/dual-write migration infrastructure;
- complete data lineage across every subsystem;
- universal schema-to-code generation;
- complete parser fuzzing coverage.

Those require implementation-specific evidence.

## 103. Transition to Part LX

Part LIX establishes the data-contract plane.

Part LX should define **security-sensitive data handling, secrets, cryptographic material, key lifecycle, trust stores, secure persistence, and confidential-computing boundaries**.

```text
Part LVIII
APIs + RPC + service contracts
        ↓
Part LIX
Data models + schemas + serialization + evolution
        ↓
Part LX
Secrets + cryptographic material + secure data handling
```

## Canonical rule

> **NROS treats schemas as contracts for representation, not substitutes for domain semantics; every compatibility or migration claim must preserve explicitly declared meaning or declare the transformation that changes it.**
