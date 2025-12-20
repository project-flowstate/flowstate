# Scope and Non-Goals

This document defines what Flowstate is and is not. It is authoritative.

## Scope

Flowstate is a spiritual successor inspired by SUPERVIVE, focused on extracting and re-expressing mechanical pillars (especially movement + traversal + hazard interaction) using original content and an open-source-first architecture.

The intended foundation is a competitive, server-authoritative multiplayer game with:
- Deterministic simulation and replay verification
- Clear client/server authority boundaries
- Long-term preservability (the core loop must remain runnable/hostable)

## Non-Goals

Flowstate is not a clone.

We will not copy:
- Characters, lore, naming, voice lines, audio, art, branding, or proprietary content
- Map layouts copied from the inspiration title
- Proprietary SDKs, closed platforms, or closed live-service dependencies as a requirement for core gameplay

We will not optimize for:
- Fastest time-to-prototype at the expense of core correctness and boundaries
- Early “content scale” (many heroes, maps, cosmetics) before the simulation contract is proven

## “Inspired by” Constraints

We may replicate genre-typical mechanics and interaction patterns (movement, gliding-style traversal, knockback-to-hazard dynamics) as long as:
- The implementation is original
- The assets and names are original
- The resulting work is not a derivative copy of proprietary expression

## License Posture (High-Level)

- Repository code is MIT licensed.
- Third-party intake is permissive-only by default and must comply with `docs/licensing/*`.
- Assets should be original or CC0 / permissive-compatible with explicit provenance.