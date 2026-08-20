# Claims

A documentation claim is a statement about NROS behavior, implementation, performance, safety, compatibility, or maturity.

## Claim structure

Where practical, claims should be traceable to:

1. the requirement or specification;
2. the implementation artifact;
3. the verification artifact;
4. the resulting status.

```text
Requirement
    ↓
Specification
    ↓
Implementation
    ↓
Verification
    ↓
Claim
```

## Claim discipline

Avoid absolute statements such as "fully implemented", "zero-copy", "real-time", "production-ready", or "hardware validated" unless the repository contains evidence that supports the exact statement.

A narrower claim is preferable when evidence is partial.

## Review rule

When documentation and repository evidence disagree, the documentation must be corrected or explicitly marked as historical/proposed. Evidence does not become true merely because it appears in a README.
