# API Reference

This page is the entry point for concrete NROS APIs.

## API sources

The authoritative API surface is the Rust source code in the workspace crates. Documentation should link to concrete modules, types, functions, traits, and commands rather than reproducing large source listings.

## CLI surface

The current CLI entry point exposes commands including `init`, `build`, `run`, `topic`, `record`, `replay`, `analyze`, `profile`, `fleet`, `migrate`, and `check`. The implementation explicitly reports several operations as simulated and unverified; therefore command availability must not be interpreted as proof of backend capability.

## API maturity

For each public API, documentation should distinguish:

- declared interface;
- implemented behavior;
- tested behavior;
- simulated behavior;
- unsupported behavior.

The implementation remains the source of truth for signatures and semantics.
