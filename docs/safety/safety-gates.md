# Safety Gates

Safety gates are explicit conditions that prevent a system from advancing when required evidence or preconditions are absent.

## Gate structure

A gate should define:

- entry condition;
- required evidence;
- acceptance criteria;
- failure behavior;
- owner;
- recorded result.

## Documentation rule

A gate marked as passed must correspond to an observable verification record. Documentation alone cannot manufacture a gate result.

## Typical gate categories

- build and static-analysis gates;
- unit and integration-test gates;
- protocol conformance gates;
- deployment readiness gates;
- hardware validation gates;
- release gates.
