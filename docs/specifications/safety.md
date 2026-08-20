# Safety Specification

## Purpose

Safety specifications define constraints that protect controlled systems from unsafe software behavior and from incorrect assumptions about system state.

## Core invariants

1. Invalid or unavailable safety-critical data must not be silently treated as valid.
2. Safety-relevant state transitions must have explicit preconditions.
3. Failures must produce defined and observable outcomes.
4. Hardware-specific safety behavior must remain explicit at the hardware boundary.
5. Simulation must not be presented as evidence of physical safety.
6. Emergency or protective behavior must have an independently reviewable contract.

## Evidence requirement

A safety requirement is not satisfied merely because a comment, assertion, type, or API exists. Evidence should demonstrate the relevant invariant under normal, invalid, boundary, and failure conditions.

## Authority

Safety-related claims should be traceable to their originating requirement, implementation, and verification evidence. Historical audit records remain historical unless a current verification process establishes their present validity.
