# Workspace

The NROS workspace contains implementation, specifications, documentation, tests, examples, and supporting project material. The exact tree is authoritative in the repository.

## Orientation

```text
README.md       → project entry point
docs/           → user and developer documentation
specs/          → normative or machine-oriented specifications
src/crates/etc. → implementation, where present
tests/           → executable verification
audits/          → historical and evidence records, where present
```

## Working rule

Before changing implementation, identify the relevant specification and verification material. Before changing a specification, identify the implementation and tests it governs.

This creates a traceable path:

`Requirement → Specification → Implementation → Test → Evidence → Documentation`
