---
status: Draft
issue: 7
title: v0 Two-Client Multiplayer Slice
---

# FS-0007: v0 Two-Client Multiplayer Slice

> **Status:** Draft  
> **Issue:** [#7](https://github.com/project-flowstate/flowstate/issues/7)  
> **Owner:** @danieldilly
> **Date:** 2025-12-21

---

## Problem

The Flowstate project requires a minimal end-to-end multiplayer implementation to validate the Authoritative Multiplayer Architecture before adding gameplay complexity. Without this slice, the team cannot verify that:

1. The Simulation Core remains deterministic and isolated from I/O
2. The Server Edge correctly mediates between Game Clients and the Simulation Core
3. Replay verification works on the same build/platform
4. The chosen networking stack (ENet + Protobuf) integrates correctly

This spec defines the minimal implementation: two native Game Clients connect to a Game Server Instance, move Characters with WASD, and the system produces replay artifacts that prove determinism.

## Trace Map

| ID | Relationship | Notes |
|----|--------------|-------|
| AC-0001 | Implements | Primary acceptance criterion |
| INV-0001 | Constrains | Deterministic simulation |
| INV-0002 | Constrains | Fixed timestep |
| INV-0003 | Constrains | Server authoritative |
| INV-0004 | Constrains | Simulation Core isolation |
| INV-0005 | Constrains | Tick-indexed I/O |
| INV-0006 | Constrains | Replay verifiability |
| INV-0007 | Constrains | Deterministic ordering/canonicalization |
| KC-0001 | Constrains | Kill: Simulation Core boundary violation |
| KC-0002 | Constrains | Kill: Replay cannot reproduce outcome |
| DM-0001..DM-0027 | Implements | See Domain Concepts table |
| ADR-0001 | Implements | Authoritative Multiplayer Architecture |
| ADR-0002 | Implements | Deterministic simulation requirements |
| ADR-0003 | Implements | Fixed timestep (tick-driven stepping API) |
| ADR-0004 | Implements | Server-authoritative architecture |
| ADR-0005 | Implements | v0 networking (ENet, Protobuf, same-build scope) |
| ADR-0006 | Implements | Input tick targeting & TargetTickFloor |
| ADR-0007 | Implements | StateDigest algorithm (FNV-1a 64-bit, canonicalization) |

## Domain Concepts

| Concept | ID | v0 Notes |
|---------|-----|----------|
| Tick | DM-0001 | Atomic simulation time unit |
| World | DM-0002 | Contains entities, RNG state, tick counter |
| Character | DM-0003 | Player-controlled entity with position/velocity |
| Entity | DM-0005 | Base object with EntityId |
| InputCmd | DM-0006 | Tick-indexed movement intent (logical concept; wire: InputCmdProto) |
| Snapshot | DM-0007 | Post-step world state at tick T+1 |
| Session | DM-0008 | Per-client connection lifecycle |
| Channel | DM-0009 | Realtime (unreliable+sequenced) or Control (reliable+ordered) |
| Match | DM-0010 | Game instance lifecycle; scopes replay |
| Server Edge | DM-0011 | Transport, validation, tick scheduling |
| Simulation Core | DM-0014 | Deterministic game logic; no I/O |
| Game Client | DM-0015 | Player runtime; rendering, input |
| Baseline | DM-0016 | Pre-step state at tick T |
| ReplayArtifact | DM-0017 | Versioned record for replay verification |
| StateDigest | DM-0018 | Deterministic digest (see ADR-0007) |
| PlayerId | DM-0019 | Per-Match participant identifier |
| EntityId | DM-0020 | Unique identifier for Entity |
| MatchId | DM-0021 | Stable Match correlation key |
| InputTickWindow | DM-0022 | Server tick-indexed acceptance window |
| LastKnownIntent | DM-0023 | Input continuity fallback |
| AppliedInput | DM-0024 | Post-normalization input (Server Edge truth) |
| TargetTickFloor | DM-0025 | Server-emitted tick floor for input targeting |
| InputSeq | DM-0026 | Per-session sequence for deterministic selection |
| StepInput | DM-0027 | Simulation-plane input consumed by advance() |

**Pipeline:** InputCmd (protocol) → AppliedInput (Server Edge selection) → StepInput (Simulation Core).

## Interfaces

### Simulation Core Types

```rust
/// Ref: DM-0001
pub type Tick = u64;
/// Ref: DM-0019 (v0 representation)
pub type PlayerId = u8;
/// Ref: DM-0020
pub type EntityId = u64;

/// Ref: DM-0027. Simulation-plane input consumed by advance().
/// player_id is an association key used to match intent to player's entity;
/// Server Edge owns identity binding (INV-0003).
/// StepInput values passed to advance() MUST be sorted by player_id ascending
/// for deterministic iteration (INV-0007), not for semantic discrimination.
pub struct StepInput {
    pub player_id: PlayerId,
    pub move_dir: [f64; 2],  // Magnitude <= 1.0
}

/// Ref: DM-0016. Pre-step world state at tick T.
/// Digest computed via World::state_digest() per ADR-0007.
/// entities MUST be sorted by entity_id ascending (INV-0007).
pub struct Baseline {
    pub tick: Tick,
    pub entities: Vec<EntitySnapshot>,
    pub digest: u64,  // state_digest() at this tick
}

/// Ref: DM-0007. Post-step world state at tick T+1.
/// Digest computed via World::state_digest() per ADR-0007.
/// entities MUST be sorted by entity_id ascending (INV-0007).
pub struct Snapshot {
    pub tick: Tick,
    pub entities: Vec<EntitySnapshot>,
    pub digest: u64,  // state_digest() at this tick
}

pub struct EntitySnapshot {
    pub entity_id: EntityId,
    pub position: [f64; 2],
    pub velocity: [f64; 2],
}

/// Ref: DM-0002
pub struct World { /* opaque to Server Edge */ }

impl World {
    /// Create world. dt_seconds = 1.0 / tick_rate_hz computed internally.
    pub fn new(seed: u64, tick_rate_hz: u32) -> Self;
    pub fn spawn_character(&mut self, player_id: PlayerId) -> EntityId;
    /// Ref: DM-0016. Postcondition: baseline().tick == world.tick()
    pub fn baseline(&self) -> Baseline;
    /// Ref: ADR-0003, ADR-0006.
    /// Precondition: tick MUST == self.tick().
    /// Postconditions: world.tick() == tick + 1; snapshot.tick == tick + 1.
    /// step_inputs MUST be sorted by player_id ascending (INV-0007).
    pub fn advance(&mut self, tick: Tick, step_inputs: &[StepInput]) -> Snapshot;
    pub fn tick(&self) -> Tick;
    /// Ref: ADR-0007
    pub fn state_digest(&self) -> u64;
    pub fn tick_rate_hz(&self) -> u32;
}
```

### Protocol Messages

Per ADR-0005. Channel mappings:

| Message | Channel | Direction | Key Fields |
|---------|---------|-----------|------------|
| `ClientHello` | Control | C→S | Handshake initiation |
| `ServerWelcome` | Control | S→C | `target_tick_floor`, `tick_rate_hz`, `player_id` |
| `JoinBaseline` | Control | S→C | Baseline (DM-0016) |
| `InputCmdProto` | Realtime | C→S | `tick`, `input_seq`, `move_dir` (no `player_id` - bound by Server Edge) |
| `SnapshotProto` | Realtime | S→C | Snapshot + `target_tick_floor` |
| `TimeSyncPing` | Control | C→S | `client_timestamp` |
| `TimeSyncPong` | Control | S→C | `server_tick`, `server_timestamp`, `ping_timestamp_echo` |

**Normative requirements:**
- ServerWelcome and every SnapshotProto MUST include `target_tick_floor` (DM-0025, ADR-0006).
- `target_tick_floor` MUST be computed as `server.current_tick + input_lead_ticks`.
- Server MUST emit target_tick_floor as monotonic non-decreasing for the match; clients MUST take max to ensure their local floor is monotonic.

**Notes:**
- `player_id` assignment: first session = 0, second session = 1 (bound by Server Edge from session, not from protocol).
- Clients MUST take `max(previous, received)` when updating local TargetTickFloor.
- Clients MUST target `InputCmd.tick >= TargetTickFloor`.
- TimeSync is debug/telemetry only; MUST NOT affect authoritative outcomes.

## Determinism Notes

This feature is the foundation of the determinism guarantee. Key constraints:

- **Simulation Core isolation (INV-0004, KC-0001):** No I/O, networking, wall-clock, ambient RNG. Enforced via crate separation, CI dependency allowlist, and forbidden-API source scan.
- **Fixed timestep (INV-0002):** `tick_rate_hz` configured at `World::new()` only; `dt_seconds` computed internally.
- **Deterministic ordering (INV-0007):** Inputs sorted by `player_id`; entities iterated by `EntityId` ascending.
- **StateDigest (ADR-0007):** FNV-1a 64-bit with canonicalization (`-0.0` → `+0.0`, NaN → quiet NaN).
- **Same-build scope (ADR-0005):** v0 guarantees determinism for same binary artifact + same target triple only.

Replay verification validates initialization anchor (baseline digest) and final outcome digest per INV-0006.

## Validation Rules

Server Edge validates inputs BEFORE converting to StepInput. Parameters from [v0-parameters.md](../networking/v0-parameters.md).

| Check | Behavior |
|-------|----------|
| NaN/Inf in move_dir | DROP + LOG |
| Magnitude > 1.0 | CLAMP to unit length + LOG |
| Tick window: `cmd.tick < current_tick` | DROP (late) |
| Tick window: `cmd.tick > current_tick + max_future_ticks` | DROP (too far future) |
| Rate limit exceeded | DROP + LOG |
| InputSeq non-increasing | DROP non-increasing cmd + LOG protocol violation (clients MUST send strictly increasing input_seq per session) |
| Multiple InputCmdProto for same (session, tick) | Keep greatest `input_seq`; if equal, keep first-seen and LOG protocol violation |

**LastKnownIntent (DM-0023):** Missing input at current_tick → reuse last move_dir. Initial = `[0, 0]`.

## Server Tick Loop (Non-Normative Pseudocode)

Constants from [v0-parameters.md](../networking/v0-parameters.md): `tick_rate_hz=60`, `max_future_ticks=120`, `input_lead_ticks=1`.

```
// Wait for two sessions; only then send ServerWelcome + JoinBaseline
wait for two client connections
world = World::new(seed, tick_rate_hz)  // tick_rate_hz from v0-parameters.md
spawn characters (player_id 0 first, then 1)
target_tick_floor = world.tick() + input_lead_ticks  // input_lead_ticks from v0-parameters.md

send ServerWelcome (target_tick_floor, tick_rate_hz, player_id) to each client
send JoinBaseline (world.baseline()) to both clients

loop (paced at tick_rate_hz):
    current_tick = world.tick()
    target_tick_floor = current_tick + input_lead_ticks
    
    // Buffer incoming inputs with validation + InputSeq selection
    server_edge.receive_and_buffer_inputs()
    
    // Produce AppliedInput per player (from buffer or LastKnownIntent)
    applied_inputs = []
    for each player:
        if input_buffer[player][current_tick] exists:
            applied_inputs.append(buffer entry)
            update current_intent[player]
        else:
            applied_inputs.append(LastKnownIntent fallback)
    
    // Record for replay, convert to StepInput, advance
    replay_artifact.inputs.extend(sorted(applied_inputs))
    step_inputs = convert applied_inputs to StepInput
    snapshot = world.advance(current_tick, step_inputs)
    
    broadcast(snapshot, target_tick_floor)

on match end:
    // Complete current tick before ending (no mid-tick termination)
    replay_artifact.final_digest = world.state_digest()
    replay_artifact.checkpoint_tick = world.tick()
    write replay artifact to replays/{match_id}.replay
```

## Client Input Targeting (Non-Normative Pseudocode)

Per ADR-0006:

```
// On ServerWelcome:
target_tick_floor = ServerWelcome.target_tick_floor
input_seq = 0

// On each SnapshotProto:
target_tick_floor = max(target_tick_floor, SnapshotProto.target_tick_floor)

// When sending input:
InputCmdProto.tick = target_tick_floor  // Or higher
InputCmdProto.input_seq = ++input_seq
```

## Replay Artifact (DM-0017)

Required fields per INV-0006:

| Field | Purpose |
|-------|---------|
| `replay_format_version` | Schema version (start at 1) |
| `initial_baseline` | Baseline at match start tick (DM-0016); v0 starts at tick 0 |
| `seed` | RNG seed |
| `rng_algorithm` | e.g., "ChaCha8Rng" |
| `tick_rate_hz` | Simulation tick rate |
| `state_digest_algo_id` | Per ADR-0007 |
| `entity_spawn_order` | Deterministic EntityId assignment |
| `player_entity_mapping` | player_id → EntityId |
| `tuning_parameters` | Any sim-affecting parameters (e.g., move_speed); v0 may be empty |
| `inputs` | AppliedInput stream (DM-0024) sorted by: (1) tick ascending, (2) player_id ascending. Gaps filled by LastKnownIntent (DM-0023) and recorded. |
| `build_fingerprint` | Binary identity (e.g., git commit + target triple); enables same-build verification |
| `final_digest` | StateDigest at checkpoint_tick (ADR-0007) |
| `checkpoint_tick` | Post-step tick for verification |
| `end_reason` | "complete" or "disconnect" |

**Verification (ref INV-0006):**
1. Verify `artifact.build_fingerprint` matches current binary: CI/Tier-0 MUST fail on mismatch; dev MAY warn and proceed
2. Validate AppliedInput stream integrity: MUST contain exactly one entry per (player_id, tick) for each tick in range [initial_baseline.tick, checkpoint_tick); fail immediately if missing or extra entries
3. Initialize World with `World::new(artifact.seed, artifact.tick_rate_hz)`
4. Verify `world.baseline().digest == artifact.initial_baseline.digest` (fail immediately if mismatch)
5. Replay ticks [initial_baseline.tick, checkpoint_tick): convert AppliedInput → StepInput, call `world.advance(t, step_inputs)`
6. Assert `world.tick() == checkpoint_tick`
7. Assert `world.state_digest() == artifact.final_digest`

**Location:** `replays/{match_id}.replay` (untracked, gitignored)

## Gate Plan

### Tier 0 (Must pass before merge)

- [ ] **T0.1:** Two clients connect, complete handshake (ServerWelcome with TargetTickFloor + tick_rate_hz + player_id)
- [ ] **T0.2:** JoinBaseline delivers initial Baseline; clients display Characters
- [ ] **T0.3:** Clients tag inputs per ADR-0006: InputCmd.tick >= TargetTickFloor, InputSeq monotonic
- [ ] **T0.4:** WASD produces movement; both clients see own + opponent via Snapshots
- [ ] **T0.5:** Simulation Core isolation enforced: crate separation, dependency allowlist (CI), forbidden-API scan (CI); advance() takes explicit tick per ADR-0003
- [ ] **T0.6:** Validation per v0-parameters.md: magnitude clamp, NaN/Inf drop, tick window, rate limit, InputSeq selection (DM-0026), LastKnownIntent (DM-0023), player_id bound to session (INV-0003)
- [ ] **T0.7:** Malformed inputs do not crash server
- [ ] **T0.8:** TimeSync ping/pong implemented (debug/telemetry only)
- [ ] **T0.9:** Replay artifact generated with all required fields
- [ ] **T0.10:** Replay verification: baseline digest check, half-open [0, checkpoint_tick), final digest match
- [ ] **T0.10a:** Initialization anchor failure: mutated baseline digest fails immediately
- [ ] **T0.11:** Future input non-interference: input for T+k (k > window) rejected; T+1 input buffered without affecting T
- [ ] **T0.12:** LastKnownIntent determinism: input gaps filled, recorded in artifact, replay produces same digest
- [ ] **T0.13:** Validation matrix: NaN, magnitude, tick window, rate limit, InputSeq selection with tie-breaking
- [ ] **T0.14:** Disconnect handling: complete current tick, persist artifact with end_reason="disconnect", clean shutdown
- [ ] **T0.15:** `just ci` passes

### Tier 1 (Tracked follow-up)

- [ ] Extended replay test: 10,000+ ticks
- [ ] Client-side interpolation
- [ ] Graceful disconnect handling
- [ ] Stricter validation (Tier-1 security posture)

### Tier 2 (Aspirational)

- [ ] Cross-platform determinism
- [ ] Client-side prediction + reconciliation
- [ ] Snapshot delta compression
- [ ] WebTransport adapter

## Acceptance Criteria

Maps to AC-0001 sub-criteria:

- [ ] **AC-0001.1:** Two clients connect, handshake, receive Baseline, remain synchronized
- [ ] **AC-0001.2:** WASD movement works; LastKnownIntent for missing inputs; TargetTickFloor-based targeting
- [ ] **AC-0001.3:** Replay verification passes (baseline + final digest); Simulation Core has no I/O; tick_rate_hz fixed at construction; advance() takes explicit tick + StepInput
- [ ] **AC-0001.4:** Validation per v0-parameters.md; InputSeq selection with deterministic tie-breaking; future inputs buffered correctly; player_id bound to session (INV-0003); disconnect → complete tick → persist artifact → shutdown
- [ ] **AC-0001.5:** ReplayArtifact produced with all fields; reproduces outcome on same build/platform

## Non-Goals

Explicitly out of scope:

- Client-side prediction / reconciliation
- Cross-platform determinism (v0 = same-build/same-platform per ADR-0005)
- Web clients
- Matchmaking / lobbies / orchestration
- Collision / terrain
- Combat / abilities
- Snapshot delta compression
- `.proto` files (v0 uses inline prost derive)
