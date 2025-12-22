# ADR 0001: Authoritative Multiplayer Architecture

## Status
Accepted

## Type
Technical

## Context
Flowstate is a competitive multiplayer game that prioritizes correctness, testability, and long-term preservability. To achieve these goals, the system must separate concerns cleanly: game logic must be isolated from networking/session management and from presentation (rendering, input, UI). Without this separation, the simulation becomes entangled with I/O, making determinism impossible to guarantee and testing prohibitively expensive.

The architecture must support multiple client types (native, web browser) and enable replay verification, which requires the simulation to be pure and side-effect-free.

## Decision
Flowstate adopts an **Authoritative Multiplayer Architecture** with strict separation of concerns and clear authority boundaries:

### Runtime Components

**1. Game Client** — The player runtime.
- Owns: rendering, input capture, UI, interpolation, and (future) prediction/reconciliation.
- MUST NOT authoritatively decide game-outcome-affecting state.

**2. Matchmaker** — The sole system a Game Client contacts to create/find/join a match.
- Owns: matchmaking, lobbies/queues (if any), match creation, and assignment of players to a specific Game Server Instance.
- Returns connection information/credentials for a Game Server Instance.
- Optional for LAN/dev scenarios; may not exist in local play.

**3. Game Server Instance** — One running authoritative match runtime (process/container).
- Owns the authoritative game simulation for exactly one match lifecycle.
- Contains exactly two named subcomponents:

  **3a. Server Edge** — The networking and session boundary.
  - Owns: all networking, transports, session lifecycle, input validation.
  - Performs all I/O.
  - Exchanges explicit, typed, tick-indexed messages with the Simulation Core.

  **3b. Simulation Core** — The deterministic game logic.
  - Deterministic, fixed-timestep, replayable game rules: physics, movement, abilities, combat resolution.
  - Performs NO I/O, networking, rendering, engine calls, or OS/system calls.
  - Replayable purely from recorded inputs.

### Hard Boundaries

- The Simulation Core MUST NOT perform I/O, networking, rendering, or system calls.
- The Simulation Core MUST be deterministic and replayable from recorded inputs.
- The Game Client MUST NOT authoritatively decide game-outcome-affecting state.
- Communication across boundaries is via explicit, typed, tick-indexed message passing only.

## Rationale
This architecture enables:
- **Determinism:** Simulation Core has no hidden state from I/O or timing.
- **Testability:** Simulation Core can be tested without graphics, network, or OS dependencies.
- **Replayability:** Record inputs, replay simulation, verify identical outcomes.
- **Multiple clients:** Native, web, headless bots all consume the same simulation.
- **Preservability:** Core game logic remains runnable decades from now (no platform lock-in).
- **Clear authority:** Each component has explicit ownership; no ambiguity about who decides what.

**Tradeoffs:**
- Higher upfront design cost (explicit boundaries require discipline).
- Cannot use "engine-native" patterns that blur simulation/presentation (e.g., Godot signals from physics to rendering).
- Requires careful message protocol design between Server Edge and Simulation Core.

## Constraints & References (no prose duplication)
- Constitution IDs:
  - INV-0001 (Deterministic Simulation)
  - INV-0002 (Fixed Timestep)
  - INV-0004 (Simulation Core Isolation)
- Canonical Constitution docs:
  - [docs/constitution.md](../constitution.md) — Product Thesis
  - [docs/constitution/invariants.md](../constitution/invariants.md)
- Related ADRs:
  - ADR-0002 (Deterministic Simulation) — defines properties of Simulation Core
  - ADR-0003 (Fixed Timestep) — defines stepping model of Simulation Core
  - ADR-0004 (Server-Authoritative) — defines authority model

## Alternatives Considered
- **Monolithic engine-integrated architecture** — Use Godot's scene tree and signals for all logic. Rejected: Makes determinism and replay impossible; locks simulation to Godot's lifecycle.
- **Merge orchestration into server** — Combine matchmaking and simulation into one component. Rejected: Loses separation between orchestration and simulation; makes headless testing harder.
- **Four-layer (separate Data layer)** — Add explicit persistence layer. Deferred: Can be added later; v0 doesn't require it.

## Implications
- **Enables:** Deterministic testing, replay verification, multiple client types, long-term preservability.
- **Constrains:** Simulation Core code cannot directly call Godot APIs, network libraries, or OS functions.
- **Migration costs:** None (greenfield project).
- **Contributor impact:** Developers must understand component boundaries and message-passing contracts.

## Follow-ups
- Define Simulation Core I/O interface (message types for inputs/outputs)
- Establish testing patterns for Simulation Core-only tests
- Document component boundaries in repository map and handbook
