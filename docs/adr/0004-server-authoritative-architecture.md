# ADR 0004: Server-Authoritative Architecture

## Status
Accepted

## Type
Technical

## Context
Competitive multiplayer games require a trusted arbiter to prevent cheating and ensure consistent outcomes. If clients authoritatively decide game-affecting state (e.g., "I hit the target", "I have 1000 resources"), players can modify their clients to cheat, and different clients may disagree on match outcomes.

The Simulation Plane (ADR-0001) is deterministic (ADR-0002), but determinism alone does not prevent cheating—clients could run modified simulations. A single authoritative instance must enforce the rules.

## Decision
The **server is the single source of truth** for all game-outcome-affecting state. Clients are presentation-only and never authoritatively decide hits, damage, resource changes, or state transitions.

**Server responsibilities:**
- Run the authoritative Simulation Plane (ADR-0001)
- Receive client inputs (movement intent, aim direction, action requests)
- Advance simulation via deterministic stepping (ADR-0002, ADR-0003)
- Broadcast authoritative state snapshots to clients
- Validate inputs (rate limiting, bounds checking, action legality)
- Enforce match rules and win conditions

**Client responsibilities:**
- Capture player inputs and send to server
- Receive authoritative snapshots from server
- Render game state (interpolate/predict for responsiveness)
- Provide immediate feedback (cosmetic effects, predicted movement)
- MUST NOT authoritatively decide: hits, damage, status effects, spawns, despawns, resource changes

**Future ADR: Client Prediction & Reconciliation:**
- Clients MAY predict local player movement for responsiveness
- Clients MUST reconcile predicted state when authoritative snapshot arrives
- Cosmetic effects (muzzle flash, VFX) are allowed but never commit game state
- (Deferred to post-v0; v0 clients render authoritative snapshots only)

## Rationale
**Why server-authoritative:**
- **Security:** Single trusted arbiter prevents client-side cheating
- **Consistency:** All clients see the same match outcome (modulo latency)
- **Correctness:** Deterministic server simulation (ADR-0002) is the ground truth
- **Verifiability:** Match replays reflect actual server state, not client predictions

**Why not peer-to-peer:**
- No trusted arbiter → clients can cheat
- Desyncs are unresolvable conflicts (whose version is correct?)
- NAT traversal and connection quality issues scale poorly

**Why not authoritative client:**
- Player-controlled arbiter invites cheating
- No way to verify client is running unmodified simulation

**Tradeoffs:**
- Server hosting required (cost, infrastructure)
- Clients experience latency (50-100ms input delay)
- Prediction/reconciliation complexity (to mask latency)
- Single point of failure (server down → no matches)

## Constraints & References (no prose duplication)
- Constitution IDs:
  - INV-0001 (Deterministic Simulation) — server runs deterministic simulation
  - INV-0002 (Fixed Timestep) — server advances simulation in fixed ticks
- Canonical Constitution docs:
  - [docs/constitution.md](../constitution.md) — "server-authoritative" is explicit in-scope goal
  - [docs/constitution/scope-non-goals.md](../constitution/scope-non-goals.md)
- Related ADRs:
  - ADR-0001 (Three-Plane Architecture) — server runs Simulation Plane, clients run Client Plane
  - ADR-0002 (Deterministic Simulation) — server simulation is deterministic and replayable
  - ADR-0003 (Fixed Timestep) — server simulation advances in fixed ticks

## Alternatives Considered
- **Peer-to-peer lockstep** — All clients simulate identically, exchange inputs. Rejected: No cheat prevention; desyncs unresolvable; poor scalability.
- **Authoritative client (host)** — One player's client is the server. Rejected: Host can cheat; unfair latency advantage.
- **Blockchain consensus** — Distributed trust via blockchain. Rejected: High latency (consensus time); not suitable for realtime gameplay.

## Implications
- **Enables:** Cheat resistance, consistent match outcomes, replay verification from server perspective, competitive integrity
- **Constrains:** Requires server hosting infrastructure, clients cannot play offline (for competitive modes), latency must be masked via prediction
- **Migration costs:** None (greenfield project)
- **Contributor impact:** All simulation changes must consider server authority; clients cannot "decide" outcomes locally

## Follow-ups
- Define client-server message protocol (inputs client→server, snapshots server→client)
- Implement input validation at simulation boundary (rate limiting, bounds checking)
- Document prediction/reconciliation strategy for clients (Future ADR: Client Prediction & Reconciliation)
- Establish server hosting requirements (headless mode, tick rate targets, scalability)
