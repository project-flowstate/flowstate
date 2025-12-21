# ADR 0001: Three-Plane Architecture

## Status
Accepted

## Type
Technical

## Context
Flowstate is a competitive multiplayer game that prioritizes correctness, testability, and long-term preservability. To achieve these goals, the system must separate concerns cleanly: game logic must be isolated from orchestration (matchmaking, lobbies) and presentation (rendering, input, UI). Without this separation, the simulation becomes entangled with I/O, making determinism impossible to guarantee and testing prohibitively expensive.

The architecture must support multiple client types (native, web browser) and enable replay verification, which requires the simulation to be pure and side-effect-free.

## Decision
Flowstate adopts a **Three-Plane Architecture** with strict separation of concerns:

1. **Control Plane** — Orchestration: matchmaking, lobbies, session management, authentication
2. **Simulation Plane** — Authoritative deterministic game logic: physics, movement, abilities, combat resolution
3. **Client Plane** — Presentation: rendering, input capture, UI, interpolation, prediction (non-authoritative)

Planes are logical boundaries and may be implemented within a single binary or as separate processes. The Control Plane includes the in-process I/O Boundary (network/session owner) and may also include optional external Orchestration Services (see DM-0011, DM-0012).

**Hard boundaries:**
- The Simulation Plane MUST NOT perform I/O, networking, rendering, or system calls
- The Simulation Plane MUST be deterministic and replayable from recorded inputs
- The Client Plane MUST NOT authoritatively decide game-outcome-affecting state
- Communication between planes is via explicit, typed message passing only

## Rationale
This architecture enables:
- **Determinism:** Simulation has no hidden state from I/O or timing
- **Testability:** Simulation can be tested without graphics, network, or OS dependencies
- **Replayability:** Record inputs, replay simulation, verify identical outcomes
- **Multiple clients:** Native, web, headless bots all consume the same simulation
- **Preservability:** Core game logic remains runnable decades from now (no platform lock-in)

**Tradeoffs:**
- Higher upfront design cost (explicit boundaries require discipline)
- Cannot use "engine-native" patterns that blur simulation/presentation (e.g., Godot signals from physics to rendering)
- Requires careful message protocol design between planes

## Constraints & References (no prose duplication)
- Constitution IDs:
  - INV-0001 (Deterministic Simulation)
  - INV-0002 (Fixed Timestep)
- Canonical Constitution docs:
  - [docs/constitution.md](../constitution.md) — Product Thesis
  - [docs/constitution/invariants.md](../constitution/invariants.md)
- Related ADRs:
  - ADR-0002 (Deterministic Simulation) — defines properties of Simulation Plane
  - ADR-0003 (Fixed Timestep) — defines stepping model of Simulation Plane
  - ADR-0004 (Server-Authoritative) — defines distribution of planes

## Alternatives Considered
- **Monolithic engine-integrated architecture** — Use Godot's scene tree and signals for all logic. Rejected: Makes determinism and replay impossible; locks simulation to Godot's lifecycle.
- **Two-plane (Client/Server only)** — Merge Control Plane into Server. Rejected: Loses separation between orchestration and simulation; makes headless testing harder.
- **Four-plane (separate Data Plane)** — Add explicit persistence layer. Deferred: Can be added later; v0 doesn't require it.

## Implications
- **Enables:** Deterministic testing, replay verification, multiple client types, long-term preservability
- **Constrains:** Simulation code cannot directly call Godot APIs, network libraries, or OS functions
- **Migration costs:** None (greenfield project)
- **Contributor impact:** Developers must understand plane boundaries and message-passing contracts

## Follow-ups
- Define Simulation I/O interface (message types for inputs/outputs)
- Establish testing patterns for simulation-plane-only tests
- Document plane boundaries in repository map and handbook
