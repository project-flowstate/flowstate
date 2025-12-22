# Acceptance and Kill Criteria

This document defines what "good" means (acceptance criteria) and when to stop/re-scope (kill criteria).

Acceptance criteria MUST be testable and minimal.

<!--
ENTRY FORMAT (use H3 with anchor):

  ### <a id="AC-NNNN"></a> AC-NNNN - Title
  **Status:** Active  
  **Tags:** tag1, tag2

  **Criterion:** Observable condition that defines "done"...

  ### <a id="KC-NNNN"></a> KC-NNNN - Title
  **Status:** Active  
  **Tags:** tag1, tag2

  **Trigger:** Condition that triggers stop/re-scope...
-->

## Acceptance Criteria

### <a id="AC-0001"></a> AC-0001 — v0 Two-Client Multiplayer Slice
**Status:** Proposed  
**Tags:** phase0, networking, replay

**Pass Condition:** The system MUST demonstrate functional end-to-end multiplayer with two connected Game Clients where:

1. **Connectivity & initial authoritative state transfer (JoinBaseline):** Two native Game Clients can connect to a Game Server Instance, complete handshake, receive initial authoritative state, and remain synchronized.
2. **Gameplay slice (WASD control):** Each Game Client can issue WASD movement inputs; the authoritative simulation processes them; both Game Clients see their own and the opponent's movement via snapshots with acceptable consistency.
3. **Simulation Core boundary integrity:** The authoritative simulation produces identical outcomes for identical input+seed+state across multiple runs (same build/platform), verified by Tier-0 replay test. The Simulation Core MUST NOT perform I/O, networking, rendering, or wall-clock reads (INV-0001, INV-0002, INV-0004).
4. **Tier-0 input validation:** Server enforces magnitude limit, tick window sanity check, and rate limit (values in [docs/networking/v0-parameters.md](../networking/v0-parameters.md)); input handling per ADR-0006; malformed or out-of-policy inputs are rejected without crashing.
5. **Replay artifact generation:** A completed match produces a replay artifact (initial state, seed, input stream, final state hash) that can reproduce the authoritative outcome on the same build/platform (INV-0006). "Input stream" means the AppliedInput (DM-0024) stream—the per-tick inputs that were actually applied by the authoritative simulation, not raw InputCmd (DM-0006) traffic.

*Non-normative note: "Acceptable consistency" means visual correctness for WASD movement in a LAN/dev environment without requiring tick-perfect lockstep rendering (client-side prediction is future work). Specific validation thresholds documented in [docs/networking/v0-parameters.md](../networking/v0-parameters.md). This is a composite criterion because all five sub-criteria are interdependent and must ship together to constitute a viable v0 milestone.*

## Kill Criteria

### <a id="KC-0001"></a> KC-0001 — Simulation Core Boundary Violation
**Status:** Proposed  
**Tags:** architecture, determinism, networking

**Kill Condition:** Reject any change that introduces networking, file I/O, rendering dependencies, wall-clock time reads, OS/system calls, game engine API calls, or other external side effects into the Simulation Core (as defined in DM-0014, enforced by INV-0004). This is a **hard stop**—no exceptions for expediency.

*Non-normative note: This guards the architectural foundation. Once the boundary is compromised, determinism (INV-0001), replay (INV-0006), and testability collapse. If a feature "requires" Simulation Core I/O, the correct solution is to refactor the feature's design, not weaken the boundary.*

### <a id="KC-0002"></a> KC-0002 — Replay Verification Blocker
**Status:** Proposed  
**Tags:** replay, verification, operations

**Kill Condition:** If replay verification (INV-0006) cannot reproduce the defined authoritative outcome checkpoint for a Match (DM-0010) on the same build/platform, treat the issue as a release blocker. Do not ship until determinism is restored or the fault is definitively isolated to test infrastructure (not simulation logic).

*Non-normative note: Cross-platform determinism is deferred to post-v0, but same-build/same-platform replay MUST work. If replay breaks, it signals either non-determinism (violates INV-0001) or incomplete input recording. Both are foundational failures. This criterion does not apply to "replay desync due to intentional simulation change"—in that case, regenerate golden replays. It applies when replay fails unexpectedly for an unchanged simulation.*

## Criteria Change Policy

- Acceptance criteria may evolve as understanding improves, but should remain minimal and testable.
- Kill criteria should be changed only with maintainer approval and an ADR explaining why the guardrail is no longer valid.
