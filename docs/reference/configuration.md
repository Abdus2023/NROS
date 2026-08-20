# Configuration Reference

NROS configuration is documented here only when a configuration surface is present in the repository.

## Current workspace configuration

The workspace manifest defines the Rust workspace members, resolver, package metadata, and shared dependency section. See the repository `Cargo.toml` for the authoritative values.

## Configuration categories

Future concrete configuration references may include:

- runtime configuration;
- transport configuration;
- node configuration;
- simulation configuration;
- build profiles;
- deployment configuration.

## Evidence rule

A configuration option belongs in this reference only after its parser, schema, or consuming implementation exists. Design-only configuration belongs in specifications.
