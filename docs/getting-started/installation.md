# Installation

## Development prerequisites

Use the toolchain and dependencies declared by the repository itself. Do not infer supported versions from historical documentation when the current repository configuration provides a more authoritative value.

## Recommended sequence

```text
clone repository
    ↓
inspect toolchain configuration
    ↓
install declared prerequisites
    ↓
format/check
    ↓
test
```

## Rust workspace

NROS uses Rust for its core implementation. The exact supported toolchain should be taken from the repository's current toolchain configuration and CI configuration.

## Verification

After installation, run the repository's documented formatting, build, and test commands. Record failures rather than replacing them with claims of successful setup.

## Troubleshooting

When an environment cannot obtain a required dependency, distinguish:

- an environment/network failure;
- a missing prerequisite;
- a repository build failure;
- a test failure.

Do not report one category as another.
