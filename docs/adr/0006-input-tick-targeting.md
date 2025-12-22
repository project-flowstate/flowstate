# ADR 0006: Input Tick Targeting & Server Tick Guidance

## Status
Accepted

## Type
Technical

## Context
Clients must tag InputCmd messages with a target Tick. If clients choose target ticks solely based on the most recently observed server tick (from Welcome/Snapshots), that observation is inherently delayed by network latency. This produces late-tagged inputs that arrive after the server has already processed the targeted tick(s), causing dropped inputs and confusing startup behavior.

Additionally, when multiple InputCmd messages target the same (PlayerId, Tick), the Server Edge must select one deterministically. Relying on packet arrival order is non-deterministic and transport-dependent.

We need a deterministic, protocol-level mechanism that:
- makes the input-targeting rule explicit and testable,
- avoids relying on wall-clock timing inside the Simulation Core,
- tolerates reordering/duplication without "arrival order" semantics,
- and preserves replay verification by ensuring the Simulation Core consumes only StepInput (derived from AppliedInput).

## Decision
The Server Edge MUST provide tick guidance to clients, and clients MUST use that guidance when selecting InputCmd target ticks.

### Normative Requirements

**Server-Guided Targeting (TargetTickFloor):**
- The Server Edge MUST emit TargetTickFloor (DM-0025) in ServerWelcome and in each Snapshot (DM-0007).
- TargetTickFloor MUST be computed as: `server.current_tick + INPUT_LEAD_TICKS` (see [docs/networking/v0-parameters.md](../networking/v0-parameters.md) for v0 value).
- TargetTickFloor MUST be monotonic non-decreasing per Session (DM-0008); resets on session re-establishment or new MatchId (DM-0021).
- Game Clients MUST target InputCmd.tick values >= TargetTickFloor (clients MUST clamp upward).
- Game Clients MUST NOT target InputCmd.tick values earlier than TargetTickFloor.

**Deterministic Input Selection (InputSeq):**
- InputCmd MUST carry an InputSeq (DM-0026) that is monotonically increasing per Session.
- When multiple InputCmd messages are received for the same (PlayerId, Tick), the Server Edge MUST deterministically select the winning InputCmd using InputSeq (greatest wins) or an equivalent deterministic mechanism.
- InputSeq selection MUST NOT depend on packet arrival order or other runtime artifacts.

**Input Pipeline:**
- The Server Edge MUST buffer inputs by (PlayerId, Tick) within the InputTickWindow (DM-0022).
- For each Tick T processed, the Server Edge MUST produce exactly one AppliedInput (DM-0024) per participating player, using LastKnownIntent (DM-0023) fallback when no valid input exists for T.
- The Server Edge MUST convert AppliedInput to StepInput (DM-0027) before invoking the Simulation Core.
- The Server Edge MUST invoke `advance(tick, step_inputs)` with the explicit tick T per INV-0005.
- The Simulation Core MUST assert that the provided tick matches its internal state (e.g., `tick == world.tick()`).
- The Simulation Core MUST consume only StepInput and MUST NOT consume raw InputCmd or AppliedInput directly.

### Startup Behavior

In v0, the first simulation step (tick 0 → 1) uses neutral LastKnownIntent for all players because clients cannot provide inputs before receiving ServerWelcome.

**Rationale:** ServerWelcome is sent when the match starts (after all required clients connect). At that moment, `server.current_tick = 0`. ServerWelcome.TargetTickFloor = `0 + INPUT_LEAD_TICKS = 1`. Clients target tick 1 or later. The server's first `advance()` call (processing tick 0) has no client inputs, so LastKnownIntent (zero-intent) is used for all players.

Tier-0 validation focuses on responsiveness starting with the first eligible tick (tick 1 with INPUT_LEAD_TICKS=1).

## Rationale
**Why server-provided tick guidance:**
- Avoids stale client tick observations being treated as authoritative targeting information
- Eliminates first-input ambiguity (clients know exactly which tick to target)
- Reduces late-input drops compared to pure snapshot-driven targeting

**Why sequence-based tie-breaker (InputSeq):**
- Makes "latest wins" deterministic and independent of packet arrival order
- On transports with guaranteed ordering (e.g., ENet sequenced channels), InputSeq is defense-in-depth
- On unordered transports (e.g., WebTransport datagrams), InputSeq becomes required

**Why separate StepInput from AppliedInput:**
- Preserves Simulation Core boundary (INV-0004); simulation code never sees protocol-plane types
- AppliedInput is what the Server Edge selected (protocol truth); StepInput is what the sim consumes
- Prevents implementers from accidentally passing raw client messages into the sim

## Constraints & References (no prose duplication)
- Constitution IDs:
  - INV-0004 (Simulation Core Isolation)
  - INV-0005 (Tick-Indexed I/O Contract)
  - INV-0006 (Replay Verifiability)
  - INV-0007 (Deterministic Ordering & Canonicalization)
  - DM-0001 (Tick)
  - DM-0006 (InputCmd)
  - DM-0022 (InputTickWindow)
  - DM-0023 (LastKnownIntent)
  - DM-0024 (AppliedInput)
  - DM-0025 (TargetTickFloor)
  - DM-0026 (InputSeq)
  - DM-0027 (StepInput)
- Related ADRs:
  - ADR-0003 (Fixed Timestep) — defines tick-driven stepping API
  - ADR-0005 (v0 Networking Architecture) — transport/channel architecture
- Parameters:
  - [docs/networking/v0-parameters.md](../networking/v0-parameters.md) — INPUT_LEAD_TICKS value

## Alternatives Considered
- **Client-only snapshot-driven targeting (no server guidance)** — Rejected: Stale observation leads to dropped inputs at startup and under RTT variation. Requires clients to guess server state.
- **Start delay / first-input barrier** — Acceptable as auxiliary mitigation, but insufficient as the long-term targeting model. Adds latency to match start.
- **RTT / clock-based tick estimation** — Viable later for prediction/reconciliation, but adds complexity. Server guidance achieves robust behavior with less coupling for v0.
- **Arrival-order tie-breaking** — Rejected: Non-deterministic; depends on transport and network conditions. Breaks replay verification.

## Implications
- **Enables:** Deterministic input selection independent of transport; robust startup behavior; replay verification via AppliedInput
- **Constrains:** Protocol messages (Welcome/Snapshot/InputCmd) carry additional semantics (TargetTickFloor, InputSeq); clients must implement clamping
- **Migration costs:** None (greenfield protocol)
- **Contributor impact:** Contributors must understand the input pipeline (InputCmd → AppliedInput → StepInput) and respect plane boundaries

## Follow-ups
- Update v0 multiplayer slice spec to incorporate TargetTickFloor and InputSeq semantics
- Add Tier-0 tests validating deterministic selection under duplication/reordering
- Add Tier-0 tests validating "first movement" occurs without prolonged input drops
- Document input pipeline in contributor handbook
