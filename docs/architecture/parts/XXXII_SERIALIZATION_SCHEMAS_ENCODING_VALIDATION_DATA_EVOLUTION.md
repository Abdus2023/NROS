# Part XXXII — Serialization, Schemas, Encoding, Validation & Data Evolution

> **Series:** NROS Architecture Series  
> **Part:** XXXII  
> **Role:** Data contracts, schemas, canonical encoding, serialization, framing, validation, compatibility, migration, and data evolution  
> **Status:** Architectural design document — not implementation conformance evidence

## 1. Purpose

Part XXXI established trusted protocol sessions and negotiation. Part XXXII defines how NROS represents protocol data consistently across processes, machines, versions, and persistence boundaries.

The central rule is:

> **NROS must distinguish semantic data contracts from their concrete encodings: schemas define meaning and constraints, serialization maps values to representations, framing defines message boundaries, validation establishes admissibility, and migration governs evolution between incompatible representations.**

## 2. Fundamental Distinctions

```text
schema
  ≠
encoding
  ≠
serialization
  ≠
framing
  ≠
validation
  ≠
version
  ≠
migration
```

## 3. Data Contract

A data contract defines the meaning and constraints of exchanged data:

```text
Data Contract
 ├─ type
 ├─ fields
 ├─ constraints
 ├─ semantics
 ├─ version
 ├─ compatibility rules
 └─ lifecycle
```

A wire representation is valid only when it conforms to the applicable contract.

## 4. Schema

A schema describes structure and constraints:

```text
Message
 ├─ field A: type
 ├─ field B: type
 ├─ field C: optional
 └─ invariants
```

Schemas should distinguish structural constraints from higher-level semantic invariants.

## 5. Schema Version

Schema evolution should be explicit:

```text
Schema V1
   ↓ compatible evolution
Schema V2
```

A schema version must not be inferred solely from a transport version unless the protocol explicitly binds them.

## 6. Type System

NROS data contracts may include:

```text
integers
floats
booleans
strings
bytes
arrays
maps
records
enums
unions
optionals
identifiers
timestamps
durations
resource references
```

Each type needs defined encoding and validation semantics.

## 7. Semantic Types

A primitive representation should not erase important semantics:

```text
u64
  ≠
ResourceId
  ≠
Timestamp
  ≠
Duration
```

Semantic types reduce accidental interchange of values that share a machine representation but have different meanings.

## 8. Nullability

The contract must distinguish:

```text
missing
null
empty
zero/default
unknown
```

These values must not silently collapse when their distinctions affect behavior.

## 9. Optional Fields

Optionality should define:

```text
may be absent
may be null
has default
required for version V
```

A default value must not accidentally change security-sensitive semantics.

## 10. Enumerations

Enums require explicit unknown-value behavior:

```text
Known value
Unknown value
   ↓
accept / preserve / reject
```

Forward-compatible consumers may preserve unknown values when the protocol permits it.

## 11. Maps and Ordering

If maps are semantically unordered, their serialized ordering must not affect meaning.

If deterministic encoding is required, canonical ordering rules must be specified.

## 12. Canonical Encoding

Canonical encoding provides a unique or constrained representation for equivalent data:

```text
Semantic Value
      ↓
Canonical Encoding
```

This is useful for hashing, signatures, content addressing, reproducibility, and deterministic tests.

## 13. Canonicalization

Canonicalization may define:

```text
field ordering
integer representation
string normalization
map ordering
omission/default rules
binary representation
```

Canonicalization must not silently alter semantic values.

## 14. Serialization

Serialization converts an in-memory value into a wire or stored representation:

```text
Value
 ↓ serialize
Bytes / representation
```

Deserialization performs the reverse operation only when the representation is valid for the contract.

## 15. Serialization Determinism

Where required:

```text
Serialize(X) = Serialize(X)
```

for the same logical value, schema, and canonicalization policy.

This connects directly to Part XXIV reproducibility.

## 16. Framing

Framing identifies message boundaries:

```text
Length / delimiter / envelope
        ↓
Payload
```

Framing is a transport/protocol concern and must not be confused with serialization.

## 17. Envelope

A protocol envelope may contain:

```text
message type
schema/version
request ID
session ID
flags
length
integrity metadata
payload
```

Envelope fields should have stable compatibility semantics.

## 18. Length Validation

Receivers must validate declared sizes before allocating or processing large payloads:

```text
Declared length
      ↓
local maximum
      ↓
accept / reject
```

This limits memory-exhaustion attacks and malformed-message amplification.

## 19. Structural Validation

Structural validation verifies:

```text
type
field presence
field shape
lengths
nesting depth
enum validity
encoding validity
```

Structural validity does not necessarily establish semantic validity.

## 20. Semantic Validation

Semantic validation verifies domain rules:

```text
start <= end
resource exists
identifier belongs to scope
state transition is legal
```

Semantic validation may require authoritative state or policy.

## 21. Validation Pipeline

A robust pipeline can be:

```text
bytes
 ↓ framing validation
 ↓ encoding validation
 ↓ schema validation
 ↓ structural validation
 ↓ semantic validation
 ↓ authorization/policy
 ↓ application operation
```

Cheap rejection should occur before expensive processing where practical.

## 22. Validation Ordering

Security-sensitive validation must occur before using untrusted values as authoritative inputs.

For example:

```text
parse
 ↓ bounds check
 ↓ validate
 ↓ authorize
 ↓ execute
```

not:

```text
parse
 ↓ execute
 ↓ validate
```

## 23. Resource Limits

Schemas should define or cooperate with limits for:

```text
message size
field size
array length
map entries
string length
nesting depth
number of objects
```

Limits are part of safe parsing, not merely performance tuning.

## 24. Numeric Semantics

Numeric fields must define:

```text
width
signedness
range
overflow behavior
special values
precision
```

Implicit narrowing or wrapping should not silently occur for security-sensitive fields.

## 25. Strings

String contracts should specify:

```text
encoding
normalization expectations
maximum length
allowed characters
comparison semantics
```

A byte sequence that is not valid according to the declared string contract must be rejected or handled according to explicit policy.

## 26. Binary Data

Binary fields should have explicit size and interpretation rules:

```text
bytes
 ↓
content type / semantic contract
```

Opaque bytes must not accidentally be interpreted as trusted structured data.

## 27. Timestamps and Durations

Time values should distinguish:

```text
instant
local time
calendar date
duration
timeout
deadline
```

An instant and a duration are not interchangeable even if both use integer machine representations.

## 28. Identifiers

Identifiers should specify:

```text
scope
format
uniqueness
case sensitivity
lifetime
reusability
```

An identifier's textual form must not imply stronger uniqueness than its authority guarantees.

## 29. Resource References

References to Part XXVIII resources should include enough context where stale-reference detection is required:

```text
resource ID
 + generation
 + authority/scope
```

This prevents an old serialized reference from silently targeting a new resource incarnation.

## 30. Identity References

References to Part XXX identities may similarly include:

```text
identity ID
incarnation/generation
trust scope
```

The encoding must preserve distinctions required for authorization.

## 31. Capability References

Part XXIX capabilities should not be serialized as unrestricted opaque authority unless their security contract explicitly permits it.

Capability-bearing messages require:

```text
scope
issuer
subject
operations
generation
validity
```

or an equivalent secure representation.

## 32. Serialization and Sessions

Part XXXI determines which schema/version is valid for a negotiated session:

```text
Session Contract
      ↓
Schema Contract
      ↓
Encoding
      ↓
Message
```

A message must not silently switch schema semantics mid-session.

## 33. Content Negotiation

If multiple representations are supported:

```text
Representation A
Representation B
Representation C
        ↓
Negotiated / selected representation
```

The selection must remain within security and resource policy.

## 34. Compression

Compression is a representation optimization, not a semantic change:

```text
Schema
 ↓ serialize
Encoding
 ↓ compress
Transport representation
```

Decompression limits must prevent resource-exhaustion attacks.

## 35. Integrity

Integrity metadata may cover:

```text
payload
schema/version
message type
envelope
session context
```

The covered fields must be explicit.

## 36. Authentication and Signatures

When data is signed, the signature must bind the semantic contract sufficiently to prevent reinterpretation under a different schema or context.

Conceptually:

```text
Identity
 + schema/version
 + canonical representation
 + context
      ↓
Signature
```

## 37. Hashing

Hashes should be computed over a defined canonical representation when equality across implementations is required.

```text
Semantic Value
 ↓ canonicalize
Canonical Bytes
 ↓ hash
Digest
```

## 38. Unknown Fields

The contract must specify whether unknown fields are:

```text
ignored
preserved
rejected
forwarded
```

Security-sensitive fields should not be silently ignored if doing so changes authorization or integrity semantics.

## 39. Backward Compatibility

A newer reader may consume older data when the schema contract permits it:

```text
Writer V1 → Reader V2
```

Compatibility should be tested rather than assumed.

## 40. Forward Compatibility

An older reader may encounter newer data:

```text
Writer V2 → Reader V1
```

Unknown-field and unknown-enum behavior determines whether safe degradation is possible.

## 41. Compatibility Classes

Compatibility should be classified as:

```text
wire compatible
schema compatible
semantic compatible
security compatible
migration compatible
```

One form of compatibility does not guarantee the others.

## 42. Schema Migration

Persistent data may require migration:

```text
Stored V1
  ↓ migration
Stored V2
```

Migration should be:

```text
deterministic
idempotent or transactionally protected
versioned
observable
recoverable
```

## 43. Dual-Read / Dual-Write

During controlled transitions:

```text
write V1 + V2
read V1 or V2
```

This pattern must have explicit retirement conditions to prevent permanent complexity.

## 44. Migration Failure

Migration must define behavior for:

```text
partial migration
invalid old data
resource exhaustion
crash during migration
version mismatch
rollback
```

Part XXVII crash-consistency rules apply to persistent migrations.

## 45. Schema Registry

A governed registry may track:

```text
schema ID
version
owner
status
compatibility class
deprecation state
migration path
```

The registry itself requires authority and lifecycle management.

## 46. Deprecation

A schema or field may transition through:

```text
active
 ↓
deprecated
 ↓
restricted
 ↓
retired
```

Deprecation must not silently invalidate existing durable data without a migration strategy.

## 47. Data Provenance

Important serialized data may carry provenance:

```text
producer identity
schema/version
time
source
transformation history
```

Provenance is metadata, not automatically truth or authorization.

## 48. Error Representation

Validation failures should expose stable machine-readable categories:

```text
invalid encoding
schema mismatch
field constraint violation
size limit exceeded
unsupported version
semantic violation
policy rejection
```

Detailed diagnostics must respect Part XXIX security boundaries.

## 49. Observability

Part XIV may expose:

```text
schema ID/version
message type
encoding
validation result
payload size
migration status
compatibility decision
```

Raw sensitive payloads should not be logged by default.

## 50. Performance Boundary

Serialization should make cost visible:

```text
parse cost
allocation cost
copy cost
compression cost
validation cost
migration cost
```

Large or adversarial inputs must not cause unbounded work.

## 51. Deterministic Data Contract

Where deterministic representation is required:

```text
Same Value
+ Same Schema
+ Same Canonicalization Policy
        ↓
Same Canonical Bytes
```

This enables reproducible hashes, signatures, snapshots, and test fixtures.

## 52. Formal Validation Invariant

A conceptual safety property is:

```text
Accepted(M)
    ⇒
FramingValid(M)
 ∧ EncodingValid(M)
 ∧ SchemaValid(M)
 ∧ SemanticValid(M)
 ∧ PolicyAllows(M)
```

## 53. Canonicalization Invariant

```text
Equivalent(X, Y)
    ⇒
CanonicalEncode(X) = CanonicalEncode(Y)
```

when the schema defines X and Y as semantically equivalent.

## 54. Migration Invariant

For a valid migration:

```text
Semantics(ReadV2(Migrate(V1)))
    =
Semantics(ReadV1(M))
```

except where the migration specification explicitly defines a semantic change.

## 55. Verification Matrix

| Property | Verification question |
|---|---|
| Schema | Is structure and meaning explicitly defined? |
| Types | Are primitive and semantic types distinguished? |
| Encoding | Is representation unambiguous? |
| Canonicalization | Can equivalent values receive deterministic representations? |
| Framing | Are message boundaries explicit? |
| Validation | Are structural and semantic checks separated? |
| Limits | Are size/depth/count limits enforced before expensive work? |
| Numbers | Are overflow and range semantics explicit? |
| Strings | Are encoding and normalization rules defined? |
| Identity | Are identity references generation-aware where required? |
| Resources | Can stale resource references be detected? |
| Capabilities | Is authority preserved without accidental expansion? |
| Integrity | Is the correct semantic context covered? |
| Compatibility | Are forward/backward rules explicit? |
| Migration | Is transformation deterministic and recoverable? |
| Persistence | Are migrations crash-consistent? |
| Observability | Can representation/validation state be diagnosed safely? |
| Performance | Are parsing and migration costs bounded? |
| Formal assurance | Are acceptance/canonicalization invariants explicit? |

## 56. What Part XXXII Does Not Claim

This Part does not claim that the current NROS implementation already has:

- a universal schema language;
- one mandatory serialization format;
- formally canonical encodings for every message;
- complete schema registry infrastructure;
- automatic migration for all persisted data;
- universal forward/backward compatibility;
- formally verified parsers;
- complete cryptographic binding for every representation.

Those require implementation-specific evidence.

## 57. Transition to Part XXXIII

Part XXXII defines stable data contracts and representation semantics.

Part XXXIII should define **event and message semantics: event identity, causality, ordering, delivery guarantees, replay, deduplication, subscriptions, event logs, and event-driven state reconstruction**, connecting serialized data with NROS's temporal and reactive architecture.

```text
Part XXXI
Sessions + negotiation + compatibility + evolution
        ↓
Part XXXII
Serialization + schemas + encoding + validation + data evolution
        ↓
Part XXXIII
Events + causality + ordering + delivery + replay + subscriptions
```

## Canonical rule

> **NROS treats data representation as a governed contract: schemas define semantics, encodings define representation, framing defines boundaries, validation establishes admissibility, canonicalization enables deterministic identity, and migration preserves or explicitly transforms meaning across versions without allowing stale, malformed, oversized, or ambiguously interpreted data to cross architectural boundaries.**
