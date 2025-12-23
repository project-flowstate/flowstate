---
status: Approved
issue: 7
title: v0 Two-Client Multiplayer Slice
---

# FS-0007: v0 Two-Client Multiplayer Slice

> **Status:** Approved  
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
| Snapshot | DM-0007 | Post-step world state. After `world.advance(T, inputs)`, returned Snapshot has `snapshot.tick = T+1` |
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
///
/// NORMATIVE CONSTRAINT (ref DM-0019, testable via T0.17): Simulation Core MUST NOT
/// assume PlayerIds are contiguous, zero-based, or start at specific literal values
/// (e.g., {0,1}). Code MUST function correctly for arbitrary assigned PlayerIds.
/// PlayerId MAY be used only as: (a) a stable indexing/ordering key for per-player
/// state, and (b) deterministic initialization wiring (spawn order/entity binding/
/// initial placement) as fully captured in ReplayArtifact initialization data.
///
/// Non-normative intent: This prevents conferring gameplay advantages based on
/// connection order or implementing player-specific rules that would violate fairness.
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
    /// v0 NORMATIVE: World::new() creates World at tick 0.
    pub fn new(seed: u64, tick_rate_hz: u32) -> Self;
    pub fn spawn_character(&mut self, player_id: PlayerId) -> EntityId;
    /// Ref: DM-0016. Postcondition: baseline().tick == world.tick()
    pub fn baseline(&self) -> Baseline;
    /// Advance simulation from tick T to T+1. `tick` parameter is the pre-step tick.
    /// inputs sorted by player_id ascending (INV-0007).
    /// Ref: DM-0007. NORMATIVE: world.advance(T, inputs) advances world.tick() from T to T+1
    /// and returns Snapshot with snapshot.tick = T+1 (the post-step tick).
    /// Precondition: tick MUST == self.tick().
    /// Postconditions: world.tick() == tick + 1; snapshot.tick == tick + 1.
    pub fn advance(&mut self, tick: Tick, step_inputs: &[StepInput]) -> Snapshot;
    pub fn tick(&self) -> Tick;
    /// Ref: ADR-0007
    pub fn state_digest(&self) -> u64;
    pub fn tick_rate_hz(&self) -> u32;
}
```

### v0 Movement Model (Normative)

For v0, Character movement is defined as follows:

```rust
// Constants (NORMATIVE for v0):
const MOVE_SPEED: f64 = 5.0;  // units per second

// Per-tick physics (applied in world.advance()):
let dt: f64 = 1.0 / tick_rate_hz as f64;  // seconds per tick
for each character with input move_dir:
    velocity = move_dir * MOVE_SPEED;  // move_dir is unit-length (clamped during validation)
    position += velocity * dt;
```

**v0 Tuning Parameters (Normative):** For v0, `MOVE_SPEED = 5.0` is a compile-time constant AND MUST be recorded in ReplayArtifact `tuning_parameters` field with key `move_speed` and value `5.0` per INV-0006. Any change to `MOVE_SPEED` or other movement constants MUST be guarded by build fingerprint mismatch. Post-v0, movement constants SHOULD be fully parameterized and recorded in `tuning_parameters` for auditability.

### Protocol Messages

Per ADR-0005. Channel mappings (NORMATIVE): Realtime = unreliable + sequenced (ENet channel 0); Control = reliable + ordered (ENet channel 1).

| Message | Channel | Direction | Key Fields |
|---------|---------|-----------|------------|
| `ClientHello` | Control | C→S | Handshake initiation |
| `ServerWelcome` | Control | S→C | `target_tick_floor`, `tick_rate_hz`, `player_id`, `controlled_entity_id` |
| `JoinBaseline` | Control | S→C | Baseline (DM-0016) |
| `InputCmdProto` | Realtime | C→S | `tick`, `input_seq`, `move_dir` (no `player_id` - bound by Server Edge) |
| `SnapshotProto` | Realtime | S→C | Snapshot + `target_tick_floor` |
| `TimeSyncPing` | Control | C→S | `client_timestamp` (Tier 1 only) |
| `TimeSyncPong` | Control | S→C | `server_tick`, `server_timestamp`, `ping_timestamp_echo` (Tier 1 only) |

**Wire Schema Definitions (Normative):**

The following protobuf message schemas define the wire contract. Rust simulation-plane types (Baseline, Snapshot) are distinct from wire types (JoinBaseline, SnapshotProto) but carry equivalent semantic content.

- **ClientHello** (Control channel):
  - No fields required for v0 (handshake initiation only)
  - Future versions MAY add fields (e.g., protocol version, client capabilities)

- **ServerWelcome** (Control channel):
  - `target_tick_floor` (u64): Initial TargetTickFloor (DM-0025) for client input targeting
  - `tick_rate_hz` (u32): Server tick rate
  - `player_id` (u8): Assigned PlayerId (DM-0019) for this session
  - `controlled_entity_id` (u64): EntityId (DM-0020) of the Character this client controls

- **JoinBaseline** (Control channel):
  - `tick` (u64): Baseline tick (DM-0016)
  - `entities` (repeated EntitySnapshot, ordered by `entity_id` ascending per INV-0007)
  - `digest` (u64): StateDigest (ADR-0007) at this tick

- **SnapshotProto** (Realtime channel):
  - `tick` (u64): Post-step tick (DM-0007)
  - `entities` (repeated EntitySnapshot, ordered by `entity_id` ascending per INV-0007)
  - `digest` (u64): StateDigest (ADR-0007) at this tick
  - `target_tick_floor` (u64): TargetTickFloor (DM-0025) for client input targeting

- **EntitySnapshot** (embedded in JoinBaseline/SnapshotProto):
  - `entity_id` (u64): EntityId (DM-0020)
  - `position` (repeated f64, length 2): [x, y]
  - `velocity` (repeated f64, length 2): [vx, vy]

*Non-normative note: Under v0 same-build scope and T0.19 shared crate requirement, protobuf field numbers won't diverge between client and server. Post-v0, when cross-build compatibility is required, field numbers become part of the compatibility contract and MUST remain stable across versions.*

**Normative requirements:**
- ServerWelcome and every SnapshotProto MUST include `target_tick_floor` (DM-0025, ADR-0006).
- `ServerWelcome.target_tick_floor` MUST equal `world.tick() + input_lead_ticks` at the moment the welcome is sent. For v0, `world.tick()` is the initial tick 0 (per World::new() postcondition), so `ServerWelcome.target_tick_floor = 0 + input_lead_ticks`.
- After `world.advance(T, inputs)` completes, the returned Snapshot has `snapshot.tick = T+1` (post-step tick).
- The `target_tick_floor` value in the resulting SnapshotProto MUST be computed as `snapshot.tick + input_lead_ticks` (post-step tick + lead). This preserves the INPUT_LEAD_TICKS headroom specified in ADR-0006: clients target at least `(T+1) + lead` when server is at tick T+1. Equivalently: `world.tick() + input_lead_ticks` after advance completes.
- Server MUST emit target_tick_floor as monotonic non-decreasing per Session (DM-0008); clients MUST take max to ensure their local floor is monotonic.

**Notes:**
- `player_id` assignment: **v0 default assignment** is first session = 0, second session = 1 (bound by Server Edge from session, not from protocol). This is a Server Edge implementation detail; the Simulation Core MUST NOT assume PlayerIds are {0, 1} or contiguous/zero-based (enforced via T0.17 test-mode override with non-contiguous IDs such as {17, 99}).
- **Test-Mode PlayerId Override (Test-Only):** Server MAY support overriding PlayerId assignment via `--test-mode` + `--test-player-ids <id1>,<id2>` CLI flags or `FLOWSTATE_TEST_MODE=1` + `FLOWSTATE_TEST_PLAYER_IDS=<id1>,<id2>` environment variables. When enabled: first accepted session receives first ID, second receives second ID. Override MUST be disabled by default. If enabled, ReplayArtifact MUST record `test_mode=true` and the assigned player IDs for traceability (determinism-relevant provenance). This mechanism exists solely to validate Simulation Core boundary constraints (no literal PlayerId assumptions).
- *Non-normative: PlayerId assignment is connection-order dependent. Tests MUST NOT assume "client A is always player 0" unless the test harness controls connection order.*
- Clients MUST take `max(previous, received)` when updating local TargetTickFloor.
- Clients MUST target `InputCmd.tick >= TargetTickFloor`.
- *Non-normative recommendation: Clients SHOULD target at least `TargetTickFloor + 1` (or similar small margin) to reduce input drops under snapshot packet loss, while still satisfying the `>= TargetTickFloor` requirement.*

## Determinism Notes

This feature is the foundation of the determinism guarantee. Key constraints:

- **Simulation Core isolation (INV-0004, KC-0001):** No I/O, networking, wall-clock, ambient RNG. Enforced via crate separation, CI dependency allowlist, and forbidden-API source scan.
- **Fixed timestep (INV-0002):** `tick_rate_hz` configured at `World::new()` only; `dt_seconds` computed internally.
- **Deterministic ordering (INV-0007):** Inputs sorted by `player_id`; entities iterated by `EntityId` ascending.
- **PlayerId design intent:** PlayerId in StepInput is an indexing/binding key only, not a gameplay authority or identity discriminator. Simulation Core MUST NOT assume PlayerIds are contiguous/zero-based (enforced via T0.17 with non-contiguous test IDs). Future per-player gameplay logic MUST be expressed via entity/component patterns, not PlayerId-based special cases.
- **StateDigest (ADR-0007):** FNV-1a 64-bit with canonicalization (`-0.0` → `+0.0`, NaN → quiet NaN).
- **Same-build scope (ADR-0005):** v0 guarantees determinism for same binary artifact + same target triple only.

Replay verification validates initialization anchor (baseline digest) and final outcome digest per INV-0006.

## Validation Rules

Server Edge validates inputs BEFORE converting to StepInput. Parameters from [v0-parameters.md](../networking/v0-parameters.md).

**Validation Ordering (Normative):** Rate limiting and basic validity checks (NaN/Inf, magnitude, tick window) MUST be applied at receive-time before buffering. InputSeq selection and the storage cap MUST then retain only the chosen cmd (plus detectability metadata per the Detectability Data Contract). Detectability requirements apply only to InputCmds that pass prior validation gates (including rate limiting) and are admitted for (session,tick) selection; the server is NOT required to track metadata for rate-limited or pre-selection-dropped commands.

| Check | Behavior |
|-------|----------|
| NaN/Inf in move_dir | DROP + LOG |
| Magnitude > 1.0 | CLAMP to unit length + LOG |
| Tick target below floor: `cmd.tick < last_emitted_target_tick_floor_for_session` | DROP (protocol violation) + LOG (Note: `last_emitted_target_tick_floor_for_session` is the most recently computed TargetTickFloor policy value for that Session, whether or not the client has observed it yet. v0 tradeoff: this enforcement can cause input drops during snapshot packet loss since floor is transmitted via unreliable snapshots. This is acceptable for v0 correctness-over-smoothness; Tier-1 may introduce mitigation such as periodic reliable floor updates or client targeting slightly ahead.) |
| Tick non-monotonic: `cmd.tick < last_valid_cmd_tick_for_session` | DROP (protocol violation) + LOG (per INV-0005: tick IDs must be monotonic non-decreasing per session). *Clarification: Transport sequencing (unreliable + sequenced ENet channel) is a transport-level ordering/discard behavior; it does not imply tick monotonicity. Tick monotonicity is an application-level constraint per INV-0005.* |
| Tick window: `cmd.tick < current_tick` | DROP (late) |
| Tick window: `cmd.tick > current_tick + max_future_ticks` | DROP (too far future) |
| Rate limit exceeded | DROP + LOG (Server MUST reject inputs exceeding input_rate_limit_per_sec per Session. v0 rate limit semantics: Per-target-tick limit derived from input_rate_limit_per_sec and tick_rate_hz using formula: `per_tick_limit = ceil(input_rate_limit_per_sec / tick_rate_hz)`. Limiter is keyed by (session, cmd.tick): for each unique target tick, accept at most per_tick_limit inputs per session. Test requirement: if a client sends N > per_tick_limit inputs targeting the same tick, at least (N - per_tick_limit) MUST be dropped. Example for v0 parameters (tick_rate_hz=60, input_rate_limit_per_sec=120): per_tick_limit = ceil(120/60) = 2.) |
| Buffer Cap (Normative) | Server MUST bound buffered input storage to at most one selected InputCmd per (session, tick) within the current InputTickWindow `[current_tick, current_tick + max_future_ticks]`. When `current_tick` advances, Server MUST evict buffered entries that fall below the new window floor. This provides O(max_future_ticks) memory bound per session and prevents unbounded growth from clients spamming distinct future ticks. |
| InputSeq validity and selection per (session, tick) | **Client obligation:** Clients MUST send strictly increasing `input_seq` per session (not per tick). **Server behavior:** For each received InputCmd targeting `(session, tick)`, server MUST LOG if `input_seq` is non-increasing relative to the last cmd from that session (protocol violation), but MUST NOT drop the cmd solely for non-increasing seq; instead, proceed with per-(session,tick) selection. *Non-normative note: v0 is intentionally permissive (log only); dropping equality would prevent tie detection. Dropping strictly-less-than would be safe but deferred.* **Selection rule:** Let `max_seq` be the maximum `input_seq` observed for (session, tick). **Detectability Data Contract (Normative):** For each (session_id, tick) entry, Server Edge MUST track `max_input_seq` and whether `max_input_seq` was observed more than once (`max_seq_tied`). Storing all InputCmds is NOT required. **Normative trigger:** If `max_seq_tied` is true for that (session_id, tick), the applied input MUST be dropped and replaced with LastKnownIntent; server MUST log protocol violation. Otherwise, select the unique cmd with `input_seq == max_seq`. **Tie evolution semantics (Normative):** When a new InputCmd with `seq` arrives for (session, tick): (1) If `seq > max_input_seq`: set `max_input_seq = seq`, set `max_seq_tied = false`, update selected reference. (2) If `seq == max_input_seq`: set `max_seq_tied = true`. (3) If `seq < max_input_seq`: ignore for selection (but may LOG). This ensures ties at earlier seq values do not poison the bucket if a later higher seq arrives. *Example: For inputs with seq {7,7,8} targeting same (session,tick), select seq=8. For {8,8}, `max_seq_tied=true`, drop both and use LKI. For {7,9,8} across different ticks from same session, LOG seq=8 as non-increasing (9→8 violation) but still consider seq=8 for its target tick's selection.* |

**Input Buffer Keying (Normative):** In v0, the input buffer is keyed by `(player_id, tick)` derived from the session→player_id binding established at ServerWelcome. Since v0 has a 1:1 session-to-player binding, references to "(session, tick)" in validation rules are equivalent to "(player_id, tick)" for buffering purposes.

**LastKnownIntent (DM-0023):** "Missing input" means no valid buffered input for (player_id, T) at the moment T is processed (i.e., after validation). Server reuses last move_dir; initial = `[0, 0]`. The fallback AppliedInput MUST be recorded in ReplayArtifact.

*Non-normative note: The InputSeq-equal drop rule ensures determinism without depending on packet arrival order, even for malformed clients.*

## Snapshot Transmission

**v0 Rate:** One snapshot per tick. Per [v0-parameters.md](../networking/v0-parameters.md), `snapshot_rate_hz MUST == tick_rate_hz` for v0. Server MUST broadcast SnapshotProto after every `world.advance()`. *Non-normative: For v0 parameters, both are 60 Hz.*

**Server Tick Definition at Emission (Normative):** When the spec or ADR-0006 refers to "server.current_tick" at the time of snapshot/floor emission, this means the post-step world tick (`world.tick()` after `advance()` completes), not the pre-step tick. The TargetTickFloor computation uses this post-step tick value. *Rationale: Eliminates classic off-by-one ambiguity between pre-step tick T (being processed) and post-step tick T+1 (after advance).*

**Floor Coherency (Normative):** For each server tick T, Server Edge MUST compute a single `target_tick_floor(T)` value and broadcast that same value to all connected sessions for that tick.

**v0 Byte-Identical Snapshots (Normative):** For v0, the server MUST serialize `SnapshotProto` exactly once per tick and broadcast the same byte payload to all connected sessions. Therefore `target_tick_floor` and all other fields are byte-identical across sessions for a given tick. *Non-normative rationale: Byte-identical broadcast is simpler for v0 than semantic-only identity; eliminates protobuf encoding ambiguity; makes T0.18 floor coherency test trivially verifiable.*

**v0 Floor Delivery Tradeoff (Explicit Design Choice):** TargetTickFloor updates are transmitted via unreliable Realtime channel (SnapshotProto). Under packet loss, clients may observe stale floor values, causing temporary input drops due to floor enforcement (validation rule: `cmd.tick < last_emitted_target_tick_floor_for_session` → DROP). This is an **explicit v0 design choice prioritizing correctness over smoothness**: floor coherency is maintained server-side; client UX degradation under loss is acceptable for v0. Tier 1 may introduce mitigation (e.g., periodic reliable floor updates, client heuristics). Client targeting guidance (`TargetTickFloor + 1` or `+2`) provides hardening against this scenario.

**Channel:** Realtime (unreliable + sequenced) per ADR-0005. Late snapshots are obsolete; no retransmission.

**Non-goal:** Delta compression and priority-based packing are Tier 2 (deferred).

*Non-normative: Visual jitter from packet loss is acceptable in v0; correctness is the objective. Client-side interpolation and render delay are deferred to Tier 1.*

## Server Tick Loop (Non-Normative Pseudocode)

Parameter values referenced from [v0-parameters.md](../networking/v0-parameters.md). *Non-normative example values for v0: `tick_rate_hz=60`, `max_future_ticks=120`, `input_lead_ticks=1`, `match_duration_ticks=3600`, `connect_timeout_ms=30000`.*

**Test Harness Note (Normative):** In tests, the server tick loop MUST be runnable in 'manual step' mode without wall-clock pacing (e.g., explicit `server.tick()` calls). CI MUST NOT sleep for match duration.

```
// Wait for two sessions with timeout; then send ServerWelcome + JoinBaseline
wait for two client connections (timeout: connect_timeout_ms)
if timeout expires before 2 sessions connect:
    log timeout event
    exit with error status
    // No ReplayArtifact written for pre-match timeout
if any session disconnects before 2 sessions connected:
    // Abort on any pre-match disconnect (v0 simplicity)
    log pre-match disconnect
    exit with error status
    // No ReplayArtifact written
world = World::new(seed, tick_rate_hz)  // tick_rate_hz from v0-parameters.md
                                         // seed sourcing: see Seed Sourcing section below

// NORMATIVE: Spawn characters in entity_spawn_order (assigned player IDs)
// Normal mode: entity_spawn_order = [player_id_of_first_session, player_id_of_second_session] (v0 default values: 0, 1; connection order)
// Test-mode: entity_spawn_order = [id1, id2] from --test-player-ids (e.g., [17, 99])
for each player_id in entity_spawn_order:
    entity_id = world.spawn_character(player_id)
    record (player_id, entity_id) in player_entity_mapping
// entity_spawn_order and player_entity_mapping recorded in ReplayArtifact

initial_tick = world.tick()  // v0 NORMATIVE: World::new() creates World at tick 0, so initial_tick = 0
target_tick_floor = initial_tick + input_lead_ticks  // input_lead_ticks from v0-parameters.md

// Initialize floor state BEFORE ServerWelcome (pre-Welcome inputs will be dropped)
for each session: last_emitted_target_tick_floor_for_session = target_tick_floor

send ServerWelcome (target_tick_floor, tick_rate_hz, player_id, controlled_entity_id) to each client
send JoinBaseline (world.baseline()) to both clients

// Pre-Welcome Input Handling (Normative): Server Edge MUST discard immediately
// without buffering any InputCmdProto received before ServerWelcome is sent to that
// session. Rationale: PlayerId binding occurs at Welcome; inputs cannot be validated
// or associated with a player before that point. Immediate discard avoids edge-case
// state (queued-but-not-validated) and matches typical connection protocol patterns
// (nothing accepted until handshake complete).

loop (paced at tick_rate_hz):
    current_tick = world.tick()  // NORMATIVE: current_tick is the pre-step tick being processed (tick T).
                                  // After world.advance(T, inputs), world.tick() returns T+1.
    
    // v0 Match Invariant: Authoritative player roster is fixed at match start (two players).
    // AppliedInput MUST be produced for both players on every processed tick,
    // even if one session is silent (LastKnownIntent fallback).
    
    // Check match termination (see v0-parameters.md)
    // For match_duration_ticks=N starting at initial_tick, server processes ticks
    // [initial_tick, initial_tick+N) and exits with checkpoint_tick = initial_tick+N.
    if current_tick >= initial_tick + match_duration_ticks:
        trigger match end (end_reason="complete")
        break
    
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
    replay_artifact.inputs.extend(sort applied_inputs by player_id ascending)
    step_inputs = convert applied_inputs to StepInput (sorted by player_id ascending)
    snapshot = world.advance(current_tick, step_inputs)
    
    // NORMATIVE: Compute TargetTickFloor AFTER advance using post-step tick (ADR-0006)
    target_tick_floor = world.tick() + input_lead_ticks  // = (current_tick + 1) + input_lead_ticks
    for each session: last_emitted_target_tick_floor_for_session = target_tick_floor
    
    // Check for disconnects (after advance, before termination decision)
    // NORMATIVE (v0 ENet): 'disconnect detected' means the server received an ENet
    // disconnect event for the peer (ENET_EVENT_TYPE_DISCONNECT) or the library
    // reported a timeout/disconnect condition as a disconnect event. If using a
    // wrapper abstraction, the wrapper's disconnect event enum is authoritative.
    disconnect_detected = any session disconnected
    
    // Broadcast to all currently connected sessions; send failure to disconnected peer is ignored
    broadcast(snapshot, target_tick_floor)
    
    // Terminate on disconnect after completing this tick
    if disconnect_detected:
        trigger match end (end_reason="disconnect")
        break

on match end:
    // Complete current tick before ending (no mid-tick termination)
    if end_reason == "disconnect":
        // Best-effort final snapshot already broadcast in loop above
        pass  // No additional broadcast
    replay_artifact.final_digest = world.state_digest()
    replay_artifact.checkpoint_tick = world.tick()
    write replay artifact to replays/{match_id}.replay  // MUST
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
InputCmdProto.tick = target_tick_floor  // (or slightly higher)
InputCmdProto.input_seq = ++input_seq
```

*Non-normative: Clients SHOULD target at least `TargetTickFloor + 1` (or +2) to reduce input drops under snapshot packet loss (v0 transmits floor updates via unreliable snapshots; targeting slightly ahead provides hardening against loss-induced floor observation lag). Targeting far future (approaching max_future_ticks) may result in inputs being dropped if the server's tick window advances before the input is processed.*

## Replay Artifact (DM-0017)

Required fields per INV-0006:

| Field | Purpose |
|-------|---------|
| `replay_format_version` | Schema version (start at 1) |
| `initial_baseline` | Baseline at match start tick (DM-0016); v0 starts at tick 0 |
| `seed` | RNG seed |
| `rng_algorithm` | e.g., "ChaCha8Rng" |
| `tick_rate_hz` | Simulation tick rate |
| `state_digest_algo_id` | Per ADR-0007. v0 MUST use: `"statedigest-v0-fnv1a64-le-f64canon-eidasc-posvel"`. *Non-normative note: v0 accepts the (non-zero) collision risk of 64-bit FNV-1a as negligible for engineering purposes in short 2-player matches with controlled canonicalization; post-v0 may upgrade to a stronger digest (e.g., 128/256-bit) or dual-digest for additional assurance.* |
| `entity_spawn_order` | Array of PlayerId in spawn sequence for deterministic EntityId assignment. Normal mode: connection order (e.g., `[0, 1]`); test-mode: MUST reflect overridden IDs in spawn order (e.g., `[17, 99]`). |
| `player_entity_mapping` | Array of (player_id, entity_id) pairs sorted by player_id ascending (verifies spawn_character() results). v0: use repeated field `{player_id, entity_id}` in protobuf, not `map<>`, to ensure deterministic serialization |
| `tuning_parameters` | Sim-affecting parameters. v0 MUST include key `move_speed` with value `5.0` (per INV-0006: all determinism-relevant parameters must be recorded). Post-v0, additional parameters SHOULD be added as needed. Protobuf schema: use repeated `{key, value}` pairs sorted by key ascending, not `map<>`, to ensure deterministic wire-order serialization. |
| `inputs` | AppliedInput stream (DM-0024). **AppliedInput Schema (Normative):** Each AppliedInput entry MUST include: `tick` (u64, the tick at which this input was applied), `player_id` (u8, the player this input is for), `move_dir` (repeated f64, length 2, normalized movement direction), `is_fallback` (bool, true if this was generated via LastKnownIntent (DM-0023), false if derived from a received InputCmdProto). Producers MUST write inputs in canonical order (spec-level requirement that satisfies INV-0006 chronological ordering): (1) tick ascending ("chronological" ordering), (2) player_id ascending (deterministic tie-break for same-tick inputs; not part of "chronological" per se). Verifier MUST canonicalize (extract by tick, sort by player_id) before replay regardless of storage order (defense-in-depth). Verifier MAY emit a warning if storage is non-canonical (dev-only). Gaps filled by LastKnownIntent (DM-0023) and recorded. |
| `build_fingerprint` | Binary identity: `binary_sha256` (SHA-256 of server executable bytes, computed at server startup via current_exe() or equivalent and hashing file bytes), `target_triple` (e.g., `x86_64-pc-windows-msvc`), `profile` (`release`/`dev`), `git_commit` (metadata/traceability). NORMATIVE: Fingerprint is computed at runtime, not compile-time embedded. If executable cannot be read (platform constraint/file-locking), v0 behavior per existing rule: Tier-0/CI MUST fail; dev MAY warn and proceed with "unknown" fingerprint. |
| `final_digest` | StateDigest at checkpoint_tick (ADR-0007) |
| `checkpoint_tick` | Post-step tick for verification: `initial_tick + match_duration_ticks` for `end_reason="complete"`, or `world.tick()` when disconnect detected |
| `end_reason` | "complete" or "disconnect" (timeout before match start does not produce ReplayArtifact) |
| `test_mode` | Boolean. MUST be `true` when test-mode override is active; MUST be `false` (or absent) otherwise. |
| `test_player_ids` | Array of assigned PlayerIds (e.g., `[17, 99]`). MUST be present and match `entity_spawn_order` when `test_mode=true`; MUST be absent when `test_mode=false`. Used for traceability and verification of test-mode runs. |

**Verification (ref INV-0006):**

**Initialization Anchor Requirement (INV-0006):** The verifier MUST compute and compare the baseline digest (initialization anchor) BEFORE applying any AppliedInputs or advancing any ticks, and MUST fail fast on mismatch. This ensures the replay starts from a verified-correct initial state.

1. Verify `artifact.build_fingerprint.binary_sha256` + `target_triple` + `profile` match current binary: CI/Tier-0 MUST fail on mismatch; dev MAY warn and proceed
2. Validate AppliedInput stream integrity: Let `player_ids` be the authoritative set recorded in the ReplayArtifact (e.g., from `player_entity_mapping` / match roster). For each `player_id ∈ player_ids`, the replay MUST contain exactly one AppliedInput entry for every tick `T` in the range `[initial_baseline.tick, checkpoint_tick)`. No gaps, no duplicates. The verifier MUST fail immediately if any `(player_id, T)` is missing or duplicated, if any entry references a `tick` outside the range, or if any entry references a `player_id ∉ player_ids`.
3. Initialize World with `World::new(artifact.seed, artifact.tick_rate_hz)`
4. Reconstruct initialization (normative): For each `player_id` in `artifact.entity_spawn_order` (array of PlayerId in spawn sequence), call `entity_id = world.spawn_character(player_id)`. The returned `entity_id` MUST equal the `entity_id` value for the corresponding `player_id` in `artifact.player_entity_mapping` (lookup the pair matching `player_id` in the sorted array). If any mismatch occurs, fail immediately with reason "spawn reconstruction mismatch".
5. Verify `world.baseline().digest == artifact.initial_baseline.digest` (fail immediately if mismatch - initialization anchor). Note: This baseline digest is computed after all spawn_character() calls complete, capturing the initial post-spawn state at tick 0.
6. Replay ticks [initial_baseline.tick, checkpoint_tick): For each tick T in range, extract all AppliedInput entries where `tick == T`, sort by `player_id` ascending, convert to StepInput array, call `world.advance(T, step_inputs)`
7. Assert `world.tick() == checkpoint_tick`
8. Assert `world.state_digest() == artifact.final_digest`

**Location:** `replays/{match_id}.replay` (untracked, gitignored)

**Serialization Format (Normative):** ReplayArtifact MUST be serialized as Protobuf (prost), versioned by `replay_format_version`. The schema is deterministic for same-build/same-platform verification (v0 replay scope per ADR-0005).

**Build Fingerprint Acquisition (Normative):**
- **Tier-0/CI:** If `binary_sha256` cannot be computed at server startup (locked file, permissions, packaging), server MUST fail startup or verifier MUST fail.
- **Dev:** MAY allow `binary_sha256 = "unknown"` but MUST emit a prominent warning and mark the artifact as non-verifiable.

**MatchId Generation (Normative Constraints):**
- **Generator:** Server Edge MUST generate MatchId at match creation
- **Uniqueness scope:** MatchId MUST be unique among concurrently active matches within the server process; SHOULD be unique across server restarts if artifacts can coexist in the same storage root
- **Length bounds:** MatchId MUST be 16-64 characters
- **Allowed alphabet:** MatchId MUST be filesystem-path-safe and URL-safe: allow only `[A-Za-z0-9_-]` (no spaces, slashes, shell metacharacters, or percent-encoding)
- **Collision handling:** If a ReplayArtifact with the same MatchId already exists on disk, server MUST fail (preserve existing artifacts; do not overwrite)
- **Determinism:** MatchId does NOT affect replay determinism (not part of simulation state); reproducibility is not required
- **Privacy:** MatchId SHOULD NOT contain PII or raw timestamps if privacy is a concern (optional for v0)

*Non-normative recommendations: UUIDv4 (36 chars with hyphens, or 32 hex), ULID (26 chars Base32), or timestamp + random suffix. Any algorithm satisfying the above constraints is acceptable for v0.*

**ReplayArtifact Output Path (Normative):**
- **Default path:** `replays/{match_id}.replay` (relative to server working directory)
- **Configurability:** Server MUST support overriding the output directory via CLI flag (e.g., `--replay-dir <path>`) or environment variable (e.g., `FLOWSTATE_REPLAY_DIR`)
- **CI/Test usage:** Tier-0 tests SHOULD use temporary directories to avoid collision and filesystem constraints

## Seed Sourcing (Normative)

For v0, RNG seed acquisition is defined as follows:

- **CLI override:** Server MUST accept an optional `--seed <u64>` CLI argument or `FLOWSTATE_SEED` environment variable
- **Default seed:** If no seed is provided via CLI/env, server MUST use seed = 0 (deterministic by default)
- **Tier-0/CI requirement:** Tier-0 and CI tests MUST run with a fixed, known seed (default seed = 0 satisfies this requirement)
- **Recording:** The chosen seed MUST be recorded in ReplayArtifact regardless of source (per INV-0001)
- **Logging:** Server SHOULD log the seed value at match start for operational traceability

## Gate Plan

### Tier 0 (Must pass before merge)

- [ ] **T0.1:** Two clients connect, complete handshake (ServerWelcome with TargetTickFloor + tick_rate_hz + player_id + controlled_entity_id)
- [ ] **T0.2:** JoinBaseline delivers initial Baseline; clients display Characters
- [ ] **T0.3:** Clients tag inputs per ADR-0006: InputCmd.tick >= TargetTickFloor, InputSeq monotonic
- [ ] **T0.4:** WASD produces movement; both clients see own + opponent via Snapshots. Concrete measurable: in deterministic harness with move_dir=[1.0, 0.0] for N consecutive ticks, Character position.x increases by expected deterministic amount (exact f64 equality; server-authoritative; no rendering required). *Non-normative note: This test assumes deterministic floating-point behavior under the build; fast-math optimizations are expected to be disabled for the Simulation Core crate.*
- [ ] **T0.5:** Simulation Core isolation enforced: crate separation, dependency allowlist (CI), forbidden-API scan (CI); advance() takes explicit tick per ADR-0003
- [ ] **T0.5a:** Tick/floor relationship assertion: After world.advance(T, inputs), assert snapshot.tick == T+1, and TargetTickFloor in SnapshotProto == snapshot.tick + input_lead_ticks (i.e., post-step tick + lead)
- [ ] **T0.6:** Validation per v0-parameters.md: magnitude clamp, NaN/Inf drop, tick window, rate limit (tick-based per-tick limit), InputSeq selection (DM-0026), LastKnownIntent (DM-0023), player_id bound to session (INV-0003)
- [ ] **T0.7:** Malformed inputs do not crash server
- [ ] **T0.8:** Replay artifact generated with all required fields
- [ ] **T0.9:** Replay verification: initialization reconstruction (spawn order), baseline digest check (initialization anchor), half-open [initial_baseline.tick, checkpoint_tick), final digest match
- [ ] **T0.10:** Initialization anchor failure: mutated baseline digest fails immediately after spawn reconstruction
- [ ] **T0.11:** Future input non-interference: input for T+k (k > window) rejected; T+1 input buffered without affecting T
- [ ] **T0.12:** LastKnownIntent determinism: input gaps filled, recorded in artifact, replay produces same digest
- [ ] **T0.12a:** Non-canonical AppliedInput storage order test (fault injection): artificially violate producer canonical-order requirement, verify verifier robustness (canonicalizes successfully; dev warning allowed)
- [ ] **T0.13:** Validation matrix: NaN, magnitude, tick window, rate limit (testability: N > limit drops at least N-limit), InputSeq selection (drop tied InputSeq → LKI fallback), TargetTickFloor enforcement, pre-Welcome input drop
- [ ] **T0.13a:** Floor enforcement drop and recovery test: Simulate snapshot packet loss for N ticks; verify inputs below last_emitted_target_tick_floor are dropped (correctness-over-smoothness v0 tradeoff). Then deliver one SnapshotProto containing the new floor; verify client targets >= new floor and movement resumes within bounded ticks. *Rationale: Proves system recovers from floor staleness, not stuck indefinitely.*
- [ ] **T0.14:** Disconnect handling: complete current tick, persist artifact with end_reason="disconnect", clean shutdown
- [ ] **T0.15:** Match termination: complete match reaches match_duration_ticks, artifact persisted with end_reason="complete"
- [ ] **T0.16:** Connection timeout: server aborts if < 2 sessions connect within connect_timeout_ms, exits with non-zero exit code (no artifact); CI MUST assert exit code and log token for deterministic test verification
- [ ] **T0.17:** Simulation Core PlayerId Non-assumption: In `--test-mode` with `--test-player-ids 17,99`, match MUST produce correct movement behavior and replay verification for both players; Simulation Core MUST NOT assume PlayerIds are {0,1}
- [ ] **T0.18:** Floor coherency server-side broadcast: For any given server tick, the server MUST broadcast byte-identical `SnapshotProto` payload to all connected sessions (assert server-side before send). *Rationale: Directly tests normative floor coherency and v0 byte-identical snapshot requirement; aligned with unreliable transport (cannot guarantee client receipt).*
- [ ] **T0.19:** Schema identity CI gate: Client and server protobuf message types MUST be defined in a single shared crate/workspace package (e.g., `flowstate_wire`) that is a direct dependency of both binaries. Tier-0/CI MUST verify this by building both binaries and failing if either does not depend on the same package ID for the wire crate (same name + version + source). *Rationale: Prevents accidental divergence of client/server message definitions within same-repo v0 scope by enforcing single canonical definition.*
- [ ] **T0.20:** `just ci` passes

### Tier 1 (Tracked follow-up)

- [ ] TimeSync ping/pong (debug/telemetry only; isolated from authoritative outcomes)
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

- [ ] **AC-0001.1:** Two clients connect, handshake, receive Baseline, remain synchronized (v0 definition: clients' rendered state equals last received authoritative snapshot for that tick; no client-side prediction required)
- [ ] **AC-0001.2:** WASD movement works; LastKnownIntent for missing inputs; TargetTickFloor-based targeting; both clients receive snapshots (v0: one per tick)
- [ ] **AC-0001.3:** Replay verification passes (baseline + final digest); Simulation Core has no I/O; tick_rate_hz fixed at construction; advance() takes explicit tick + StepInput
- [ ] **AC-0001.4:** Validation per v0-parameters.md; InputSeq selection per Validation Rules (greatest-wins; equal is protocol violation); future inputs buffered correctly; player_id bound to session (INV-0003); disconnect → complete tick → persist artifact → shutdown; connection timeout aborts cleanly
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
