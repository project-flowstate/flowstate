---
status: Draft
issue: TBD
title: v0 Two-Client Multiplayer Slice
---

# FS-TBD: v0 Two-Client Multiplayer Slice

> **Status:** Draft  
> **Issue:** [#TBD](https://github.com/project-flowstate/flowstate/issues/TBD)  
> **Author:** @copilot  
> **Date:** 2025-12-21

> **Note:** Replace `TBD` with the GitHub issue number for AC-0001 once created.

---

## Problem

The Flowstate project requires a minimal end-to-end multiplayer implementation to validate the Authoritative Multiplayer Architecture before adding gameplay complexity. Without this slice, the team cannot verify that:

1. The Simulation Core remains deterministic and isolated from I/O
2. The Server Edge correctly mediates between Game Clients and the Simulation Core
3. Replay verification works on the same build/platform
4. The chosen networking stack (ENet + Protobuf) integrates correctly

This spec defines the minimal implementation: two native Game Clients connect to a Game Server Instance, move Characters with WASD, and the system produces replay artifacts that prove determinism.

## Issue

- Issue: [#TBD](https://github.com/project-flowstate/flowstate/issues/TBD)

## Trace Map

| ID | Relationship | Notes |
|----|--------------|-------|
| AC-0001 | Implements | Primary acceptance criterion for this spec |
| INV-0001 | Constrains | Deterministic simulation: identical inputs + seed + state → identical outputs |
| INV-0002 | Constrains | Fixed timestep: simulation advances in fixed-size Ticks only |
| INV-0003 | Constrains | Server authoritative: clients send intent, server decides outcomes |
| INV-0004 | Constrains | Simulation Core isolation: no I/O, networking, rendering, wall-clock |
| INV-0005 | Constrains | Tick-indexed I/O: all boundary messages carry explicit Tick |
| INV-0006 | Constrains | Replay verifiability: reproduce authoritative outcome from artifact |
| KC-0001 | Constrains | Kill criterion: any Simulation Core boundary violation |
| KC-0002 | Constrains | Kill criterion: replay cannot reproduce authoritative outcome |
| DM-0001 | Implements | Tick: atomic unit of game time |
| DM-0002 | Implements | World: playable map space containing entities |
| DM-0003 | Implements | Character: player-controlled entity |
| DM-0005 | Implements | Entity: object with unique identity and simulation state |
| DM-0006 | Implements | InputCmd: tick-indexed player input |
| DM-0007 | Implements | Snapshot: tick-indexed authoritative world state |
| DM-0008 | Implements | Session: client connection lifecycle (Server Edge owned) |
| DM-0016 | Implements | Baseline: pre-step state serialization for join/replay |
| DM-0009 | Implements | Channel: logical communication lane (Realtime, Control) |
| DM-0010 | Implements | Match: discrete game instance lifecycle |
| DM-0011 | Implements | Server Edge: networking component within Game Server Instance |
| ADR-0001 | Implements | Authoritative Multiplayer Architecture |
| ADR-0002 | Implements | Deterministic simulation requirements |
| ADR-0003 | Implements | Fixed timestep simulation |
| ADR-0004 | Implements | Server-authoritative architecture |
| ADR-0005 | Implements | v0 networking architecture (ENet, Protobuf, channels) |

## Domain Concepts

| Concept | ID | Notes |
|---------|-----|-------|
| Tick | DM-0001 | Atomic simulation time unit |
| World | DM-0002 | Contains entities, RNG state, tick counter |
| Character | DM-0003 | Player-controlled entity with position/velocity |
| Entity | DM-0005 | Base object with unique EntityId |
| InputCmd | DM-0006 | Tick-indexed movement intent from Game Client |
| Baseline | DM-0016 | Pre-step world state serialization at tick T (used for join/replay initialization) |
| Snapshot | DM-0007 | Post-step world state serialization at tick T+1 (after inputs applied) |
| Session | DM-0008 | Per-client connection lifecycle (Server Edge) |
| Channel | DM-0009 | Realtime (unreliable+sequenced) or Control (reliable+ordered) |
| Match | DM-0010 | Game instance lifecycle; scopes replay |
| Server Edge | DM-0011 | Owns transport; validates inputs; exchanges tick-indexed messages |
| Simulation Core | DM-0014 | Deterministic game logic; no I/O |
| Game Client | DM-0015 | Player runtime; rendering, input, UI |

> **Note:** "Baseline" (DM-0016) and "Snapshot" (DM-0007) are both serialized world state but differ semantically and temporally: Baseline is pre-step state at tick T used for join and replay initialization; Snapshot is post-step state at tick T+1 used for ongoing synchronization. This distinction eliminates ambiguity in initial state handling.
> 
> **Note:** "Replay artifact" is an implementation artifact under INV-0006, not a separate domain concept. It captures the data required for replay verifiability.

## Interfaces

### Simulation Core Types

The Simulation Core (`crates/sim`) exposes the following public interface:

```rust
/// Ref: DM-0001. Atomic simulation time unit.
pub type Tick = u64;

/// Ref: DM-0005. Unique identifier for entities.
pub type EntityId = u64;

/// Ref: DM-0006. Tick-indexed input from a single player.
/// InputCmd.tick indicates the tick at which this input is applied.
/// Applied at tick T means: affects the state transition T → T+1.
///
/// **player_id boundary contract (NORMATIVE):** Simulation Core MUST treat `player_id`
/// as an indexing/ordering key only (for deterministic association of inputs to entities).
/// Simulation Core MUST NOT perform identity validation or security checks on `player_id`.
/// Server Edge is the authority for binding `player_id` to a session/connection.
pub struct InputCmd {
    pub tick: Tick,
    pub player_id: u8,
    /// Movement direction. Magnitude MUST be <= 1.0 (clamped by Server Edge).
    pub move_dir: [f64; 2],
}

/// Ref: DM-0016. Pre-step world state at tick T (before inputs applied).
/// Used for JoinBaseline and replay initial state.
pub struct Baseline {
    pub tick: Tick,
    pub entities: Vec<EntitySnapshot>,
    pub digest: u64,
}

/// Ref: DM-0007. Post-step world state at tick T+1 (after inputs applied and step executed).
/// After applying inputs at tick T and stepping, Snapshot.tick = T+1.
pub struct Snapshot {
    pub tick: Tick,
    pub entities: Vec<EntitySnapshot>,
    pub digest: u64,
}

/// Per-entity state within a Snapshot.
pub struct EntitySnapshot {
    pub entity_id: EntityId,
    pub position: [f64; 2],
    pub velocity: [f64; 2],
}

/// Ref: DM-0002. Authoritative world state.
pub struct World { /* opaque to Server Edge */ }

impl World {
    /// Create a new world with the given RNG seed and tick rate.
    /// - `seed`: RNG seed for deterministic randomness
    /// - `tick_rate_hz`: Simulation tick rate in Hz. MUST be supplied from tunable v0 parameters by Server Edge/configuration.
    ///   Non-normative example: 60 Hz.
    /// 
    /// The World computes `dt_seconds = 1.0 / tick_rate_hz` internally and uses it for all ticks.
    /// This eliminates the dt_seconds footgun by making it impossible to pass varying values.
    pub fn new(seed: u64, tick_rate_hz: u32) -> Self;
    
    /// Spawn a character for a player. Returns the assigned EntityId.
    pub fn spawn_character(&mut self, player_id: u8) -> EntityId;
    
    /// Capture pre-step state (Baseline) at current tick.
    /// Used for JoinBaseline and replay initial state.
    pub fn baseline(&self) -> Baseline;
    
    /// Advance simulation by one tick.
    /// - `inputs`: Validated InputCmds for current tick (input.tick == self.tick).
    /// 
    /// Uses the tick_rate_hz configured at construction to compute dt_seconds internally.
    /// Returns Snapshot of state after this step (Snapshot.tick = self.tick + 1).
    pub fn advance(&mut self, inputs: &[InputCmd]) -> Snapshot;
    
    /// Current tick of the world.
    pub fn tick(&self) -> Tick;
    
    /// Compute state digest for replay verification.
    pub fn state_digest(&self) -> u64;
    
    /// Get the configured tick rate (Hz).
    pub fn tick_rate_hz(&self) -> u32;
}
```

### Protocol Messages

Per ADR-0005, messages use inline Protobuf (`prost` derive). Channel mappings:

| Message | Channel | Direction | Purpose |
|---------|---------|-----------|---------|
| `ClientHello` | Control | C→S | Handshake initiation |
| `ServerWelcome` | Control | S→C | Handshake response with session info + current server tick + tick_rate_hz |
| `JoinBaseline` | Control | S→C | Pre-step authoritative state (Baseline, DM-0016) |
| `InputCmdProto` | Realtime | C→S | Tick-indexed movement intent |
| `SnapshotProto` | Realtime | S→C | Post-step world state (Snapshot) |
| `TimeSyncPing` | Control | C→S | Client timestamp for latency measurement (v0: basic, every 2s) |
| `TimeSyncPong` | Control | S→C | Server tick + server timestamp + echo of ping timestamp (v0: basic offset tracking only) |

Message field definitions are implementation details. The semantic contract is:
- `ServerWelcome.server_tick`: Current server tick; client uses to initialize last_server_tick_seen for snapshot-driven input tagging
- `ServerWelcome.tick_rate_hz`: Server simulation tick rate (Hz); stored for reference (not used for wall-clock estimation in v0 snapshot-driven approach)
- `ServerWelcome.player_id`: Assigned player identifier (v0: exactly two player_ids: 0 and 1); **assignment rule:** the first connected session is assigned player_id = 0; the second connected session is assigned player_id = 1. These IDs remain stable for the Session/Match. Server binds player identity to session (see player_id handling in validation rules).
- `JoinBaseline.tick`: Pre-step tick of Baseline (state before inputs applied at that tick)
- `JoinBaseline.digest`: Baseline state digest. Clients MAY ignore digest in v0, but server/CI replay verification MUST use digest checks (baseline and final outcome) as defined in the Replay Artifact Contents section.
- `InputCmdProto.tick`: The tick at which this input is applied (affects T → T+1)
- `SnapshotProto.tick`: Post-step tick of Snapshot (state after T → T+1, tick = T+1)
- `TimeSyncPing.client_timestamp`: Client wall-clock timestamp when ping sent
- `TimeSyncPong.server_tick`: Current server tick when pong sent
- `TimeSyncPong.server_timestamp`: Server wall-clock timestamp when pong sent
- `TimeSyncPong.ping_timestamp_echo`: Echo of received ping's client_timestamp for RTT calculation
- **TimeSync purpose (debug/telemetry only):** v0 MAY implement basic client-initiated ping/pong every ~2 seconds for debug visibility and telemetry. TimeSync MUST NOT affect authoritative outcomes. Correctness relies on snapshot-driven input tagging (last_server_tick_seen + INPUT_LEAD_TICKS), not wall-clock estimation.

## Determinism Notes

### Simulation Core Isolation (INV-0004, KC-0001)

The Simulation Core MUST NOT:
- Perform file I/O or network operations
- Read wall-clock time (`Instant::now()`, `SystemTime`)
- Use thread-local or ambient RNG (`thread_rng()`)
- Call OS/platform APIs
- Import crates that perform the above

**Enforcement (v0 Guardrails):**
1. **Crate separation:** `crates/sim` is a library crate with no access to server-edge modules (enforced by Cargo dependency graph)
2. **Dependency policy check (required):** CI MUST run `cargo-deny` or equivalent scripted allowlist/denylist check to prevent disallowed crates/features in sim crate. Only allowed dependencies: `rand_chacha`, `serde`, math/container crates. Dependency changes require code review.
3. **Forbidden-API source scan (required):** CI runs a fast source scan (e.g., `grep -r` or equivalent) over `crates/sim/src/` for forbidden symbols:
   - `std::time::{Instant, SystemTime}`, `Instant::now`, `SystemTime::now`
   - `std::fs`, `std::net`, `std::thread::sleep`
   - `rand::thread_rng`, non-seeded RNG entrypoints
   - Any forbidden imports fail the build
4. **Compile-time feature boundary:** No `#[cfg(feature = "server")]` or similar escape hatches in sim crate

*Note: These are early guardrails, not a perfect proof. Code review remains the primary enforcement mechanism.*

### Determinism Guarantees (INV-0001)

Given identical:
- Initial `World` state (constructed with identical `seed` and `tick_rate_hz` via `World::new(seed, tick_rate_hz)`)
- `InputCmd` sequence (ordered by tick, then player_id)
- Same build (see "Same Build Constraints" below)

The simulation produces identical `Snapshot` sequences and identical `final_digest`.

### Same Build Constraints (v0 Determinism Scope)

**NORMATIVE:** For v0, determinism verification (replay, CI tests) MUST use the same build constraints:

1. **Same binary artifact:** Replay verification in CI MUST run using the exact same produced binary artifact as the server run that generated the replay artifact. Do NOT rebuild between server run and replay verification.
2. **Fixed target triple/profile:** CI MUST use a fixed target triple (e.g., `x86_64-pc-windows-msvc`) and build profile (e.g., `release` or `dev`) for all simulation/replay verification runs.
3. **No CPU-specific flags:** Avoid CPU-specific optimization flags (e.g., `-C target-cpu=native`) that can alter floating-point behavior. Use conservative target settings for reproducibility.

**Rationale:** v0 guarantees determinism for same-build/same-platform only (per ADR-0005). Cross-platform determinism is deferred to post-v0.

### Numeric Representation

- All simulation math uses `f64` to minimize drift.
- Floating-point operations occur in deterministic order (single-threaded, no parallelism in v0).
- `tick_rate_hz` is configured once at `World::new(seed, tick_rate_hz)` and cannot be changed (value MUST come from [../networking/v0-parameters.md](../networking/v0-parameters.md)). World computes `dt_seconds = 1.0 / tick_rate_hz` internally once at construction and reuses it for all ticks. This eliminates the dt_seconds footgun by making it impossible to pass varying dt values.

### RNG Usage

- World uses an explicit, versioned RNG algorithm (e.g., `rand_chacha::ChaCha8Rng`).
- Seed is recorded in replay artifact.
- RNG calls occur in stable order (entity iteration via `BTreeMap`).
- v0 movement does not consume RNG; plumbing exists for future features.

### StateDigest Algorithm

The authoritative outcome checkpoint is verified via a stable 64-bit digest:

1. **Algorithm:** FNV-1a 64-bit (offset basis `0xcbf29ce484222325`, prime `0x100000001b3`)
2. **Purpose:** Determinism regression check, not a cryptographic guarantee; collisions are possible but acceptable risk for v0
3. **Ordering:** Entities iterated in `EntityId` ascending order (`BTreeMap` guarantees this)
4. **Canonicalization (applied before hashing):**
   - Convert `-0.0` to `+0.0` for all f64 values
   - Convert any NaN to the canonical bit pattern `0x7ff8000000000000` (quiet NaN)
5. **Byte encoding:** All integers and floats as little-endian bytes
6. **Included data:** `tick` (u64), then for each entity in order: `entity_id` (u64), `position[0]` (f64), `position[1]` (f64), `velocity[0]` (f64), `velocity[1]` (f64)

### Authoritative Outcome Checkpoint

For v0 replay verification:
- Checkpoint tick = match end tick (fixed tick count for tests)
- `final_digest` = `state_digest()` at checkpoint tick
- Replay is valid if replayed `final_digest` matches recorded `final_digest`

## Boundary Contracts

### Inputs into Simulation Core

| Aspect | Specification |
|--------|---------------|
| **Type** | `Vec<InputCmd>` |
| **Tick Semantics** | `InputCmd.tick = T` means the input is applied during the T → T+1 transition. The input affects the state that becomes `Snapshot.tick = T+1`. |
| **Validation Ownership** | Server Edge validates BEFORE delivering to Simulation Core. |
| **Delivery Contract** | Simulation Core receives zero or more `InputCmd` per tick, pre-validated, for the current tick only. |
| **Ordering** | Inputs sorted by `player_id` before application for determinism. |
| **Absence Semantics (Last-Known Intent)** | Server Edge maintains per-player "current intent". If an InputCmd for player P at tick T arrives, update P's current intent and deliver it. If no InputCmd arrives for P at tick T, deliver P's last-known intent as InputCmd for tick T. Initial intent is zero (`move_dir = [0, 0]`). This provides continuity under packet loss and remains deterministic. **NORMATIVE (Testable):** When InputCmd packets are missing for some ticks, the server MUST apply the last-known intent for those missing ticks, and the filled inputs MUST be recorded in the replay artifact (as if they were received) so replay produces the same digest. |

### Outputs from Simulation Core

| Aspect | Specification |
|--------|---------------|
| **Baseline** | Ref: DM-0016. Pre-step state at tick T (before inputs applied). Used for JoinBaseline and replay initial state. |
| **Snapshot** | Post-step state at tick T+1 (after inputs applied and step executed). |
| **Tick Semantics** | `Baseline.tick = T` is state before inputs at T. `Snapshot.tick = T+1` is state after applying inputs at T and advancing. |
| **Digest** | Both Baseline and Snapshot include `digest` computed via StateDigest algorithm (with canonicalization). |
| **Replay Artifact** | Produced at match end, containing: initial Baseline, seed, RNG algorithm ID, tick_rate_hz, tuning parameters, input stream, final digest, checkpoint tick. |

### Tick Semantics Diagram

```
World.tick = T                              World.tick = T+1
     │                                           │
     │ ── Baseline.tick=T (pre-step state) ─────►│
     │ ── InputCmd.tick=T applied ──────────────►│
     │    (affects T → T+1 step)                 │
     └──────────────────────────────────────────►│ ── Snapshot.tick=T+1 produced
                                                  │    (post-step state)
```

## Component Responsibilities

### Simulation Core

**Location:** `crates/sim` (library crate, no binary)

**Owns:**
- `World`, `Entity`, `Character` state
- `advance()` stepping logic
- `state_digest()` computation
- `Snapshot` production

**Forbidden (KC-0001):**
- File I/O, network I/O
- Wall-clock reads
- Ambient RNG
- OS/platform calls
- Rendering, audio

### Server Edge

**Location:** Server binary (single binary containing both Simulation Core and Server Edge)

**Owns:**
- ENet transport adapter (DM-0011)
- Session management (DM-0008)
- Channel abstraction (DM-0009)
- Match lifecycle (DM-0010)
- Tier-0 input validation and buffering:
  - Validate inputs (tick window, magnitude, NaN/Inf, rate limit)
  - Buffer at most one input per player per tick (latest-wins: later arrivals for same tick overwrite)
  - Reject inputs for ticks < last_applied_tick (already processed)
  - Accept inputs for current_tick and limited future window
  - On each simulation step at current_tick, consume buffered input for current_tick only; future-tick inputs remain buffered
- Last-known intent tracking per player (for missing input handling)
- Baseline and Snapshot broadcasting
- Replay artifact generation (written to `replays/{match_id}.replay` in v0)
- Wall-clock tick scheduling (paces simulation at tick_rate_hz)
- TimeSync pong response (basic v0: clients ping every ~2s, server responds with pong)

**Interface to Simulation Core:**
- Calls `World::advance()` with validated inputs
- Receives `Snapshot` and broadcasts to Game Clients
- Records inputs for replay artifact

### Game Client

**Location:** Godot project (`client/`)

**Owns:**
- Input capture (WASD → InputCmdProto)
- Presentation (render entities from Snapshot)
- Network connection via ENetMultiplayerPeer
- Direct connection to Game Server Instance (no orchestration service in v0)

**Forbidden:**
- Authoritative state mutation
- Game-rule decisions

**v0 Connection Model:** Game Clients connect directly to a known Game Server Instance address (IP:port). No matchmaking, lobbies, or orchestration service. Match starts when two clients connect.

**v0 Handshake Gating (NORMATIVE):** Client MUST tolerate connecting and waiting without receiving ServerWelcome until the server has both clients and starts the match. The first client to connect will wait (no simulation steps, no ServerWelcome) until the second client connects.

## Server Tick Loop

The server binary hosts both components. The Server Edge paces the simulation:

```
initialize:
    // NORMATIVE: Server does NOT begin stepping until two sessions are connected.
    // ServerWelcome is sent only once the match is ready to start (after second client connects).
    // The first client may connect and wait; no simulation steps or ServerWelcome occur until both sessions are present.
    wait for two client connections
    
    tick_rate_hz = load from [../networking/v0-parameters.md](../networking/v0-parameters.md)  // Non-normative example: 60 Hz
    world = World::new(seed, tick_rate_hz)  // Initializes world.tick() = 0; computes dt_seconds = 1.0 / tick_rate_hz internally
    spawn characters for both connected sessions  // Deterministic spawn order: player_id 0 (first connection), then player_id 1 (second connection)
    record: seed, rng_algorithm, tick_rate_hz, tuning params
    
    baseline = world.baseline()  // Pre-step state at tick 0 (both characters spawned, no inputs applied)
    replay_artifact.initial_baseline = baseline
    
    // Initialize per-player state
    for each player:
        current_intent[player_id] = InputCmd{tick: 0, player_id, move_dir: [0, 0]}
        last_applied_tick[player_id] = None  // Option<Tick>; None means no tick applied yet
    
    // Once match is ready: send ServerWelcome and JoinBaseline
    send ServerWelcome (with world.tick(), tick_rate_hz, player_id) on Control channel to each client
    send JoinBaseline (with baseline, DM-0016) to both Game Clients on Control channel

loop (paced by wall-clock at tick_rate_hz from [../networking/v0-parameters.md](../networking/v0-parameters.md)):
    current_tick = world.tick()
    
    // 1. Process incoming inputs: validate and buffer
    //    - Validate: tick window [current_tick, current_tick + MAX_FUTURE_TICKS], magnitude, NaN/Inf, rate limit
    //    - Reject if last_applied_tick[player_id] is Some(tick) and cmd.tick <= tick (already processed)
    //    - Buffer: input_buffer[player_id][tick] = cmd (latest-wins: overwrites duplicates)
    //    - NORMATIVE: Receiving an input for tick T+k MUST NOT change applied intent
    //      for ticks < T+k. Future-tick inputs only become active when the server
    //      reaches that tick and consumes the buffer for current_tick.
    server_edge.receive_and_buffer_inputs()
    
    // 2. Build input list for current_tick using last-known intent
    inputs = []
    for each player:
        if input_buffer[player_id][current_tick] exists:
            cmd = input_buffer[player_id][current_tick]
            // NORMATIVE: When consuming buffered input, ensure cmd.tick == current_tick.
            // Server MUST stamp/overwrite cmd.tick to current_tick if it differs (buffer key is authoritative).
            cmd.tick = current_tick
            current_intent[player_id] = cmd  // Update intent ONLY when consumed at current_tick
            delete input_buffer[player_id][current_tick]
        else:
            // Missing input: reuse last-known intent
            cmd = InputCmd{tick: current_tick, player_id, move_dir: current_intent[player_id].move_dir}
        inputs.append(cmd)
        last_applied_tick[player_id] = Some(current_tick)
    
    // 3. Record inputs in replay artifact (sorted by player_id)
    // NORMATIVE: Replay inputs MUST be the per-tick inputs actually APPLIED by the server
    // after validation, buffering rules, and last-known-intent fill (server truth), not raw client messages.
    replay_artifact.inputs.extend(sorted(inputs, by: player_id))
    
    // 4. Advance simulation (pure, deterministic)
    // World uses tick_rate_hz configured at construction; dt_seconds computed internally
    snapshot = world.advance(sorted(inputs, by: player_id))
    
    // 5. Broadcast snapshot on Realtime channel
    // NORMATIVE: Server broadcasts exactly one Snapshot per simulation tick
    // v0 uses full snapshots at 1/tick cadence for simplicity; bandwidth/serialization optimizations
    // (delta compression, throttling) are explicitly out of scope for Tier-0/AC-0001 and belong to later tiers.
    server_edge.broadcast(snapshot)
    
    // 6. Respond to TimeSync pings (clients initiate; server responds)
    server_edge.process_time_sync_pings()  // Send pongs with server_tick + timestamps

on match end (fixed tick count for v0 tests or player disconnect):
    // NORMATIVE: Server MUST only end the match (and capture checkpoint_tick) immediately AFTER
    // completing a tick's world.advance(); it MUST NOT end mid-tick.
    //
    // Disconnect timing (NORMATIVE): If a player disconnect is detected during tick T (at any point
    // before or during processing), the server MUST still complete tick T's world.advance() and then
    // end the match immediately after that step. Therefore checkpoint_tick MUST equal the post-step
    // tick (the world tick after applying tick T, i.e., T+1).
    replay_artifact.final_digest = world.state_digest()
    replay_artifact.checkpoint_tick = world.tick()
    if match ended due to player disconnect:
        replay_artifact.end_reason = "disconnect"  // Or equivalent marker
    else:
        replay_artifact.end_reason = "completed"  // Or equivalent marker
    serialize and write replay artifact to: replays/{match_id}.replay
    
    // NORMATIVE (Tier-0 disconnect handling summary):
    // If a player disconnects after match start, the server MUST:
    // 1. Complete the current tick's world.advance() (do not abort mid-tick)
    // 2. Persist a replay artifact marked as aborted/disconnect with checkpoint at post-step tick
    // 3. Shut down the instance cleanly (close remaining connections, flush logs)
    // v0 does not support reconnection or match continuation after disconnect.
```

## Tier-0 Input Validation and Buffering

Validation and buffering occur in the Server Edge BEFORE inputs reach Simulation Core.

**Parameter values:** MUST use values from [v0-parameters.md](../networking/v0-parameters.md).

**Constants:**
- `MAX_FUTURE_TICKS`: Maximum number of ticks ahead a client can send inputs. **Definition:** `MAX_FUTURE_TICKS := input_tick_window_ticks` where `input_tick_window_ticks` is the tunable parameter from [../networking/v0-parameters.md](../networking/v0-parameters.md). This is the single authoritative source of truth for the future window size.
- `INPUT_LEAD_TICKS`: Client-side input lead for RTT compensation. **Definition:** Fixed v0 code constant = 1 (not sourced from v0 parameters). This value is hardcoded in v0 for simplicity; post-v0 versions may make it tunable.
- `INPUT_CMDS_PER_SECOND_MAX`: Maximum input commands per second per player. **Definition:** `INPUT_CMDS_PER_SECOND_MAX := input_rate_limit_per_sec` where `input_rate_limit_per_sec` is the tunable parameter from [../networking/v0-parameters.md](../networking/v0-parameters.md). Prevents spam/DoS.
- **Constraint:** `INPUT_LEAD_TICKS MUST be <= MAX_FUTURE_TICKS` (v0 default: 1 <= input_tick_window_ticks)

**Window interpretation (NORMATIVE):** For v0, the tick window is interpreted as **FUTURE-ONLY acceptance**. Late inputs (`cmd.tick < current_tick`) are always dropped. The parameter `input_tick_window_ticks` defines the maximum future horizon, not a symmetric (±) window.

**Rationale for window sizing:** Large future windows increase buffering memory and abuse surface; small windows reduce tolerance to jitter. The tunable parameter allows adjustment based on deployment environment (LAN/WAN) and observed RTT/jitter profiles.

### Validation Rules

| Check | Behavior on Failure |
|-------|---------------------|
| **NaN/Inf in move_dir** | **DROP + LOG**: Drop input silently, log warning for debugging. |
| **Magnitude > 1.0** | **CLAMP + LOG**: Normalize to unit length, log warning, buffer clamped input. |
| **Tick outside window** | **DROP + LOG**: Drop input, log warning. **Acceptance window:** `current_tick <= cmd.tick <= current_tick + MAX_FUTURE_TICKS` where `current_tick = world.tick()` and `MAX_FUTURE_TICKS = input_tick_window_ticks` from [../networking/v0-parameters.md](../networking/v0-parameters.md). Inputs for `cmd.tick < current_tick` (late) are dropped. Inputs for `cmd.tick > current_tick + MAX_FUTURE_TICKS` (too far future) are dropped. |
| **Player identity mismatch** | **OVERRIDE + LOG**: Server MUST bind player identity to the session/connection. Server MUST ignore/overwrite any client-provided `player_id` field and stamp the session's assigned `player_id` before validation/application. If client-provided `player_id` mismatches session, server SHOULD log a warning for debugging but MUST NOT drop input solely due to mismatch. This avoids "client bug => no movement" failure mode. |
| **Already processed tick** | **DROP + LOG**: Drop input, log warning. Reject if `last_applied_tick[player_id]` is `Some(tick)` and `cmd.tick <= tick`. (`last_applied_tick` is `Option<Tick>`; `None` means no tick applied yet.) |
| **Rate limit exceeded** | **DROP + LOG**: Drop excess inputs beyond INPUT_CMDS_PER_SECOND_MAX (where `INPUT_CMDS_PER_SECOND_MAX = input_rate_limit_per_sec` from [../networking/v0-parameters.md](../networking/v0-parameters.md)), log warning. |

**v0 disconnect policy:** Validation failures (DROP + LOG) do not disconnect clients. Server MAY disconnect for egregious abuse (e.g., sustained rate-limit violations) but this is optional for v0. Default posture: log and drop.

### Buffering Semantics ("Latest-Wins")

- **Per-player per-tick buffer:** `input_buffer[player_id][tick] = InputCmd`
- **Latest-wins for duplicates (before consumption):** If multiple InputCmds arrive for the same player and tick BEFORE that tick is consumed/applied, the most recent overwrites previous (consistent with unreliable+sequenced channel semantics). Once a tick is consumed (applied), any InputCmd for that tick is rejected via the "already processed" validation rule.
- **No early application:** Inputs for future ticks (cmd.tick > current_tick) are buffered and NOT applied until current_tick advances to that tick
- **Consumption:** On each simulation step at current_tick, the server consumes ONLY `input_buffer[player_id][current_tick]` (if present) and applies it; future-tick entries remain in buffer
- **Last-known intent fallback:** If no input exists for player P at current_tick, the server reuses P's last-known intent (most recent move_dir) to construct an InputCmd for current_tick

### Last-Known Intent Tracking

- **Initialization:** Each player's `current_intent[player_id]` starts as `move_dir = [0, 0]`
- **Update:** When an InputCmd for player P at current_tick is consumed, update `current_intent[player_id] = cmd.move_dir`
- **Reuse:** If no InputCmd for P at current_tick, create `InputCmd{tick: current_tick, player_id: P, move_dir: current_intent[P]}`
- **Determinism:** Last-known intent reuse is deterministic and replay-stable; the replay artifact records all applied inputs (including those generated from last-known intent)

## Game Client Input Tick Tagging

### Snapshot-Driven Tick Tagging (v0 Primary Approach)

**Goal:** Correctness and simplicity. Clients tag inputs based on the latest authoritative server tick observed from snapshots, not wall-clock estimation.

**State:**
```
last_server_tick_seen = 0  // Updated from SnapshotProto.tick or ServerWelcome.server_tick
```

**Initialization (on ServerWelcome):**
```
last_server_tick_seen = ServerWelcome.server_tick
tick_rate_hz = ServerWelcome.tick_rate_hz  // Stored for reference, not used for estimation
```

**Update (on each SnapshotProto):**
```
last_server_tick_seen = max(last_server_tick_seen, SnapshotProto.tick)
```

**Input Tick Tagging:**

**Constant:** `INPUT_LEAD_TICKS = 1` (fixed v0 code constant; not tunable)

**Rule:** When sending input, set:
```
InputCmdProto.tick = last_server_tick_seen + INPUT_LEAD_TICKS
```

**Clamping (optional but recommended):** To reduce rejections, clients SHOULD clamp the tagged tick to the server's acceptance window:
```
InputCmdProto.tick = clamp(
    last_server_tick_seen + INPUT_LEAD_TICKS,
    last_server_tick_seen,
    last_server_tick_seen + MAX_FUTURE_TICKS
)
```
where `MAX_FUTURE_TICKS = input_tick_window_ticks` from [../networking/v0-parameters.md](../networking/v0-parameters.md).

**Rejection Handling:**
- If the client observes repeated rejections for "already processed" or "too far future" (detected via explicit logs/debug counters if available), it SHOULD re-base to the latest `last_server_tick_seen` from snapshots and reapply `INPUT_LEAD_TICKS`.
- No wall-clock estimation required; snapshots provide authoritative tick information.

**Rationale:** Lead ticks compensate for RTT and jitter, ensuring the input targets a tick the server is likely to process in the future. Snapshot-driven tagging avoids wall-clock drift and simplifies correctness.

**Tick semantics consistency:**
- `Baseline.tick = T` is pre-step state at tick T
- `InputCmd.tick = T` means "apply this intent during step T → T+1"
- `Snapshot.tick = T+1` is post-step state after applying inputs at tick T
- Client uses latest observed server tick S from snapshots, so next input targets tick S + INPUT_LEAD_TICKS

**First inputs:** After receiving ServerWelcome at tick T, client can immediately send inputs tagged for tick T + INPUT_LEAD_TICKS (or later), even before the first snapshot arrives.

**Continuous input and coalescing (NORMATIVE):** Clients send inputs every frame/poll cycle; the server's buffering and latest-wins semantics handle duplicates and out-of-order arrival. **Client MUST cap InputCmd send rate to INPUT_CMDS_PER_SECOND_MAX (where `INPUT_CMDS_PER_SECOND_MAX = input_rate_limit_per_sec` from [../networking/v0-parameters.md](../networking/v0-parameters.md)).** Client SHOULD send at most one InputCmd per tick per player. If multiple input updates occur within the same tick (same tagged tick value), client SHOULD coalesce by latest-wins: the most recent input state replaces earlier queued input for that tick before sending.

v0 does not implement client-side prediction. Game Clients render authoritative Snapshot positions directly.

### Wall-Clock Tick Estimator (Optional, Not Required for v0)

**Note:** The wall-clock-based tick estimator (using `elapsed_seconds * tick_rate_hz`) is optional and not required for v0 correctness. It MAY be used for smoother input UX or telemetry but MUST NOT affect authoritative outcomes. If implemented, clients SHOULD still prefer snapshot-driven tagging for input tick assignment.

**TimeSync (Debug/Telemetry Only):**
- Clients MAY send TimeSyncPing every ~2 seconds (client_timestamp)
- Server responds with TimeSyncPong (server_tick, server_timestamp, ping_timestamp_echo)
- Clients MAY compute RTT and track offset for debug visibility
- **NORMATIVE:** TimeSync MUST NOT affect authoritative outcomes. INPUT_LEAD_TICKS and server-side buffering/validation are the correctness mechanisms.

## Replay Artifact Contents

Per INV-0006, the replay artifact captures all data needed for reproduction:

- **replay_format_version:** Replay artifact schema version (u32); start at 1. Required for forward compatibility.
- **initial_baseline:** Baseline (DM-0016) at tick 0 (pre-step state before any inputs applied)
- **seed:** RNG seed used to initialize World
- **rng_algorithm:** Explicit algorithm identifier. Non-normative example: "ChaCha8Rng".
- **tick_rate_hz:** Simulation tick rate (Hz). MUST match value from v0-parameters.md used at match start. Used to construct World with `World::new(seed, tick_rate_hz)` during replay.
- **tuning_parameters:** Any parameters affecting authoritative outcomes. Non-normative examples: move_speed, acceleration.
- **entity_spawn_order:** Explicit ordered list of entity spawns (type, player_id if applicable) to ensure deterministic EntityId assignment
- **player_entity_mapping:** Map of player_id → EntityId for deterministic character assignment
- **inputs:** Chronologically ordered InputCmd stream (authoritative per-tick applied inputs after validation and last-known fallback; represents server truth, not raw client messages)
- **final_digest:** StateDigest (with canonicalization) at checkpoint tick
- **checkpoint_tick:** The tick at which final_digest was computed

**v0 artifact location:** Replay artifacts are written to an untracked local directory for development/testing. Default path: `replays/{match_id}.replay` (relative to server working directory). Tests running in CI should use a deterministic temp location or ensure the output directory exists and is writable. The `replays/` directory is added to `.gitignore`.

**Replay verification procedure:**
1. **Deterministic initial world construction:** Load artifact; initialize `World` via `World::new(artifact.seed, artifact.tick_rate_hz)` using identical RNG algorithm, tuning parameters, and spawn/setup order. Entity identity assignment (EntityId) MUST be deterministic and match the original match's initial conditions. This produces a pre-step world at tick 0.
2. **Baseline digest verification:** Compute `world.baseline().digest` and verify equality with `artifact.initial_baseline.digest`. If mismatch, fail immediately (initialization not deterministic).
3. **Replay execution:** Iterate ticks `t` in half-open range [0, checkpoint_tick) (exclusive upper bound): collect inputs for tick `t` from artifact, call `world.advance(inputs)`. After the loop, `world.tick()` MUST equal `checkpoint_tick`.
4. **Final digest verification:** Assert `world.tick() == checkpoint_tick`, then compute digest via `world.state_digest()`; compare to `artifact.final_digest`.
5. **Result:** Pass if digests match; fail if mismatch (KC-0002 release blocker).

**checkpoint_tick semantics:** The `checkpoint_tick` field in the replay artifact is the world tick AFTER the last applied step (i.e., the tick value the world holds at match end). Replay execution MUST end with `world.tick() == checkpoint_tick` before final digest verification.

**Canonical digest computation:** StateDigest algorithm includes canonicalization (-0.0 → +0.0, NaN → 0x7ff8000000000000) before hashing.

## Gate Plan

### Tier 0 (Must pass before merge)

- [ ] **T0.1:** Two native clients connect via ENet and complete handshake (ClientHello → ServerWelcome with server_tick + tick_rate_hz + player_id)
- [ ] **T0.2:** JoinBaseline delivers initial Baseline (DM-0016, pre-step state); both clients display Characters
- [ ] **T0.3:** Clients tag inputs using snapshot-driven approach: InputCmdProto.tick = last_server_tick_seen + INPUT_LEAD_TICKS (fixed constant = 1); first inputs use ServerWelcome.server_tick + INPUT_LEAD_TICKS
- [ ] **T0.4:** WASD input produces movement; both clients see own + opponent movement via Snapshots
- [ ] **T0.5:** Simulation Core has no I/O dependencies: crate separation enforced, dependency policy check enforced via CI (cargo-deny or equivalent), forbidden-API source scan enforced via CI (std::time, std::fs, std::net, thread_rng); tick_rate_hz configured at `World::new(seed, tick_rate_hz)` and immutable (dt_seconds computed internally)
- [ ] **T0.6:** Tier-0 input validation and buffering enforced per [v0-parameters.md](../networking/v0-parameters.md): magnitude clamped, NaN/Inf dropped+logged, tick window `current_tick <= cmd.tick <= current_tick + MAX_FUTURE_TICKS` enforced (late/too-far-future dropped+logged), already-processed ticks dropped+logged (cmd.tick <= last_applied_tick), rate limit enforced (drop+log), latest-wins per-player per-tick buffering, last-known intent reuse for missing inputs, future-tick inputs MUST NOT affect current/past ticks, player_id bound to session (client-provided value overridden, mismatch logged)
- [ ] **T0.7:** Malformed inputs do not crash server (negative test)
- [ ] **T0.8:** TimeSync ping/pong implemented (basic v0: clients ping every ~2s, server responds with pong) — debug/telemetry only, not correctness-critical
- [ ] **T0.9:** Replay artifact generated at match end with all required fields (including end_reason), written to `replays/{match_id}.replay` (untracked local directory)
- [ ] **T0.10:** **Replay range correctness test (MUST-have):** Run N ticks (non-normative example: 100), write replay artifact with checkpoint_tick = N. Replay using half-open range [0, checkpoint_tick). Assert world.tick() == checkpoint_tick at end. Verify final_digest matches artifact.final_digest.
- [ ] **T0.11:** **Future input non-interference test (MUST-have):** Enqueue input for tick T+k where k > MAX_FUTURE_TICKS; server rejects. Enqueue input for tick T+1 (within window); verify current-tick applied inputs (tick T) remain unchanged (buffer future input without affecting current step).
- [ ] **T0.12:** **Last-known intent determinism test (MUST-have):** Introduce input gaps (drop packets for ticks T+2, T+5); server fills with last-known intent. Verify filled inputs are recorded in replay artifact. Replay artifact produces same final_digest.
- [ ] **T0.13:** **Validation matrix test (MUST-have, table-driven):** Test all Server Edge validation rules: NaN/Inf rejection, magnitude clamp (>1.0 → normalized), tick window rejection (too early, too far future), already-processed rejection (cmd.tick <= last_applied_tick), rate limit rejection (exceed INPUT_CMDS_PER_SECOND_MAX where `INPUT_CMDS_PER_SECOND_MAX = input_rate_limit_per_sec`), player_id override (client mismatch → session identity used). Assert expected behavior for each case.
- [ ] **T0.14:** **Disconnect handling test (MUST-have):** Disconnect one player during tick 50 processing. Verify server completes tick 50's advance(), then ends match, persists replay artifact with end_reason="disconnect" and checkpoint_tick=51 (post-step tick), shuts down cleanly.
- [ ] **T0.15:** `just ci` passes

### Tier 1 (Tracked follow-up)

- [ ] Extended replay test: 10,000+ tick match
- [ ] Client-side interpolation for smoother visuals
- [ ] Graceful disconnect handling
- [ ] Stricter input validation (Tier-1 security posture)

### Tier 2 (Aspirational)

- [ ] Cross-platform determinism verification
- [ ] Client-side prediction + reconciliation
- [ ] Snapshot delta compression
- [ ] WebTransport adapter

## Acceptance Criteria

These map directly to AC-0001 sub-criteria:

- [ ] **AC-0001.1 (Connectivity & JoinBaseline):** Two native Game Clients connect directly to known server address (no orchestration service), complete handshake (ServerWelcome with server_tick + tick_rate_hz + assigned player_id), receive initial authoritative Baseline (DM-0016, pre-step state), remain synchronized.
- [ ] **AC-0001.2 (Gameplay Slice):** Each Game Client issues WASD movement; server processes using last-known intent for missing inputs; both see own + opponent movement via Snapshots with acceptable consistency. Clients tag inputs using snapshot-driven approach (InputCmdProto.tick = last_server_tick_seen + INPUT_LEAD_TICKS where INPUT_LEAD_TICKS = 1, fixed v0 constant). Movement intent remains continuous under packet loss (no stutter from zero-intent fallbacks); visual smoothness depends on snapshot delivery. Last-known intent filling is testable and produces deterministic replay artifacts.
- [ ] **AC-0001.3 (Boundary Integrity + Replay):** Identical outcomes for identical input+seed+state+tick_rate_hz (same build/platform per "Same Build Constraints"); verified by Tier-0 replay test with deterministic initial world construction via `World::new(seed, tick_rate_hz)`, half-open tick range [0, checkpoint_tick), `world.tick() == checkpoint_tick` assertion, and canonical digest (StateDigest with -0.0/NaN canonicalization); Simulation Core MUST NOT perform I/O, networking, rendering, or wall-clock reads. Tick configuration (`tick_rate_hz`) fixed at `World::new()` to eliminate dt_seconds footgun (dt computed internally).
- [ ] **AC-0001.4 (Tier-0 Validation):** Server enforces validation per [v0-parameters.md](../networking/v0-parameters.md); tick window `[current_tick, current_tick + MAX_FUTURE_TICKS]` enforced; already-processed ticks rejected; latest-wins buffering per-player per-tick; future-tick inputs buffered and MUST NOT affect current/past ticks (only active when server reaches that tick); player_id bound to session (client-provided value overridden); malformed inputs rejected without crashing. Disconnect triggers match end after completing current tick's advance(), replay artifact persistence with end_reason marker and post-step checkpoint_tick, and clean shutdown.
- [ ] **AC-0001.5 (Replay Artifact):** Match produces replay artifact at `replays/{match_id}.replay` (untracked local directory) with all required fields (including end_reason); reproduces authoritative outcome on same build/platform via `World::new(artifact.seed, artifact.tick_rate_hz)` and recorded last-known intent sequence. Replay verification uses same binary artifact (no rebuild) and fixed target triple/profile per "Same Build Constraints".

## Non-Goals

Explicitly out of scope for this spec:

- **Client-side prediction / reconciliation:** v0 Game Clients render authoritative snapshots only.
- **Cross-platform determinism:** v0 guarantees same-build/same-platform only (ADR-0005).
- **Web clients:** v0 is native Game Clients only.
- **Matchmaking / lobbies / orchestration service:** v0 auto-starts match when two Game Clients connect directly to a known server address. No service discovery, matchmaker, or relay.
- **Collision / terrain:** v0 Characters move freely without obstacles.
- **Combat / abilities:** Beyond AC-0001 scope.
- **Snapshot delta compression:** v0 sends full snapshots.
- **`.proto` files:** v0 uses inline prost derive; formal schemas in v0.2.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| ENet Rust crate instability | Low | Medium | Fallback to `enet-sys` if issues arise |
| f64 cross-platform variance | Low | High | v0 scoped to same-build/same-platform |
| Protobuf schema evolution | Medium | Low | v0 uses inline prost; migrate to `.proto` in v0.2 |

## Milestones

| # | Milestone | AC-0001 Sub-Criterion | Deliverable |
|---|-----------|----------------------|-------------|
| 1 | **Simulation Core** | Foundation | `crates/sim`: World with `new(seed, tick_rate_hz)`, Entity, `baseline()`, `advance(inputs)` (no dt parameter), `state_digest()` with canonicalization |
| 2 | **Server Skeleton** | Foundation | Server binary with ENet, sessions, tick loop using `World::new(seed, tick_rate_hz)` |
| 3 | **Connectivity & Baseline** | AC-0001.1 | Two Game Clients connect, receive ServerWelcome (with player_id) + JoinBaseline (Baseline, DM-0016) |
| 4 | **WASD Slice** | AC-0001.2 | Movement works end-to-end; clients implement snapshot-driven tick tagging (last_server_tick_seen + INPUT_LEAD_TICKS where INPUT_LEAD_TICKS = 1, fixed constant) |
| 5 | **Validation & TimeSync** | AC-0001.4 | Tier-0 input validation and buffering (tick window, latest-wins per-player per-tick, last-known intent tracking, player_id binding, disconnect handling) + basic TimeSync ping/pong (debug/telemetry only) |
| 6 | **Replay Verification** | AC-0001.3, AC-0001.5 | Artifact generated at `replays/{match_id}.replay` (with end_reason) + replay test passes with half-open range [0, checkpoint_tick), world.tick() == checkpoint_tick assertion, canonical digest, same-build constraints |

## Assumptions

1. **Godot 4.x** for native Game Clients (ENetMultiplayerPeer compatible).
2. **Single server binary** hosts both Simulation Core (as library) and Server Edge.
3. **Single match auto-start:** When two Game Clients connect, match begins immediately (no lobby).
4. **Character spawn positions:** Fixed positions (implementation detail).
5. **Match end condition (v0):** Fixed tick count for tests (non-normative example: 600 ticks = 10 seconds at tick_rate_hz from v0-parameters.md). Manual stop may exist for local dev.

## Open Questions

| # | Question | Impact | Status |
|---|----------|--------|--------|
| 1 | What is the GitHub issue number for AC-0001? | Needed for spec filename and trace block | **Awaiting maintainer** |
| 2 | Crate organization: single `crates/game` with sim module, or separate `crates/sim` + server binary? | Project structure | Recommend: `crates/sim` library + server binary that depends on it |

