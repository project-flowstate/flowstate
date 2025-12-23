# FS-0007: v0 Two-Client Multiplayer Slice — Test Gate Specifications

> **Spec:** [FS-0007-v0-multiplayer-slice.md](../specs/FS-0007-v0-multiplayer-slice.md)  
> **Plan:** [FS-0007-plan.md](FS-0007-plan.md)  
> **Issue:** [#7](https://github.com/project-flowstate/flowstate/issues/7)  
> **Date:** 2025-12-23

---

## Overview

This document converts each Tier-0 gate from FS-0007 into executable test specifications. Each gate includes test name, location, setup, assertions, constitution IDs enforced, and required fixtures.

### v0 Parameters Reference

From [v0-parameters.md](../networking/v0-parameters.md):

| Parameter | Value |
|-----------|-------|
| `tick_rate_hz` | 60 |
| `max_future_ticks` | 120 |
| `input_lead_ticks` | 1 |
| `input_rate_limit_per_sec` | 120 |
| `match_duration_ticks` | 3600 |
| `connect_timeout_ms` | 30000 |
| `MOVE_SPEED` | 5.0 |
| `per_tick_limit` | ceil(120/60) = 2 |

---

## T0.1: Two Clients Connect, Complete Handshake

### Test Name
`test_t0_01_two_client_handshake`

### Test Location
`crates/server/tests/integration/t0_01_handshake.rs`

### Setup
1. Start server in manual-step mode with fixed seed (0)
2. Create two mock ENet clients (Client A, Client B)
3. Configure clients to connect to server within `connect_timeout_ms`

### Assertions
1. **Both clients connect successfully** — server accepts exactly two peer connections
2. **ServerWelcome received by each client** with all required fields:
   - `target_tick_floor == 0 + input_lead_ticks == 1` (initial tick 0 + lead 1)
   - `tick_rate_hz == 60`
   - `player_id` — first client receives 0, second receives 1 (connection order)
   - `controlled_entity_id` — valid EntityId corresponding to spawned Character
3. **JoinBaseline received by each client** with:
   - `tick == 0` (initial tick per World::new())
   - `entities.len() == 2` (both Characters spawned)
   - `entities` sorted by `entity_id` ascending
   - `digest` — valid StateDigest matching World::state_digest()
4. **ServerWelcome.controlled_entity_id matches** the entity with position corresponding to that player's spawn

### Constitution IDs Enforced
- **DM-0008** (Session): Client connection lifecycle
- **DM-0019** (PlayerId): Per-match participant identifier assigned at handshake
- **DM-0025** (TargetTickFloor): Initial floor value emitted in ServerWelcome
- **DM-0016** (Baseline): Initial state serialization

### Fixtures Needed
- None (deterministic setup with seed=0)

---

## T0.2: JoinBaseline Delivers Initial Baseline

### Test Name
`test_t0_02_join_baseline_delivery`

### Test Location
`crates/server/tests/integration/t0_02_baseline.rs`

### Setup
1. Start server in manual-step mode with fixed seed (0)
2. Connect two mock clients, complete handshake
3. Parse received JoinBaseline messages

### Assertions
1. **Both clients receive JoinBaseline** on Control channel (reliable + ordered)
2. **Baseline.tick == 0** (World::new() postcondition)
3. **Baseline.entities contains exactly 2 EntitySnapshot entries**:
   - Each with valid `entity_id` (non-zero, unique)
   - Each with `position == [0.0, 0.0]` (initial spawn position)
   - Each with `velocity == [0.0, 0.0]` (initial velocity)
4. **entities sorted by entity_id ascending** (INV-0007)
5. **Baseline.digest matches recomputed digest**:
   - Create local World with same seed/tick_rate_hz
   - Spawn characters in same order
   - Assert `world.baseline().digest == received_baseline.digest`
6. **Client can map controlled_entity_id to an entity in Baseline**

### Constitution IDs Enforced
- **DM-0016** (Baseline): Pre-step state at tick T
- **DM-0007** (Snapshot): Snapshot structure (shared with Baseline)
- **INV-0007** (Deterministic Ordering): Entity ordering by EntityId ascending

### Fixtures Needed
- None (deterministic seed=0 produces predictable entity state)

---

## T0.3: Clients Tag Inputs Per ADR-0006

### Test Name
`test_t0_03_input_tick_targeting`

### Test Location
`crates/client/tests/integration/t0_03_input_targeting.rs`  
`crates/server/tests/integration/t0_03_input_validation.rs`

### Setup
1. Start server in manual-step mode
2. Connect two clients, complete handshake (receive ServerWelcome with `target_tick_floor=1`)
3. Client generates InputCmdProto messages for movement

### Assertions
**Client-side:**
1. **InputCmd.tick >= TargetTickFloor** — client must target tick 1 or higher after receiving `target_tick_floor=1`
2. **InputSeq is strictly monotonic increasing** per session:
   - First input: `input_seq=1`
   - Second input: `input_seq=2`
   - etc.
3. **Client updates target_tick_floor on SnapshotProto receipt** using `max(local_floor, received_floor)`

**Server-side:**
1. **Inputs with cmd.tick < target_tick_floor are dropped** (floor enforcement)
2. **Inputs with valid tick are buffered** for processing
3. **InputSeq is logged if non-increasing** (protocol violation, but not dropped solely for this)

### Constitution IDs Enforced
- **ADR-0006** (Input Tick Targeting): Server-guided targeting, InputSeq semantics
- **DM-0025** (TargetTickFloor): Client clamping to floor
- **DM-0026** (InputSeq): Per-session monotonic sequence number

### Fixtures Needed
- None

---

## T0.4: WASD Produces Movement

### Test Name
`test_t0_04_deterministic_movement`

### Test Location
`crates/sim/tests/unit/t0_04_movement.rs`

### Setup
1. Create World with seed=0, tick_rate_hz=60
2. Spawn one Character (player_id=0) → returns entity_id
3. Define `N = 60` consecutive ticks (1 second of movement)
4. Define `move_dir = [1.0, 0.0]` (full right)

### Assertions
1. **Initial position is [0.0, 0.0]**
2. **After N ticks with move_dir=[1.0, 0.0]:**
   - `expected_delta_x = MOVE_SPEED * dt * N = 5.0 * (1.0/60.0) * 60 = 5.0`
   - `final_position.x == 5.0` (exact f64 equality, no epsilon)
   - `final_position.y == 0.0` (exact f64 equality)
3. **Velocity per tick:**
   - `velocity = move_dir * MOVE_SPEED = [5.0, 0.0]`
4. **Determinism across runs:**
   - Run the same sequence twice with same seed
   - Assert identical position, velocity, and state_digest after each tick
5. **No fast-math contamination:**
   - Verify Cargo.toml for sim crate does NOT enable fast-math or unsafe float opts

### Constitution IDs Enforced
- **INV-0001** (Deterministic Simulation): Identical outcomes from identical inputs
- **INV-0002** (Fixed Timestep): dt = 1.0/tick_rate_hz, no frame-rate dependence

### Fixtures Needed
- **Golden file:** `fixtures/movement/t0_04_60ticks_right.json`
  ```json
  {
    "seed": 0,
    "tick_rate_hz": 60,
    "ticks": 60,
    "move_dir": [1.0, 0.0],
    "expected_final_position": [5.0, 0.0],
    "expected_final_velocity": [5.0, 0.0],
    "expected_final_digest": "<computed_value>"
  }
  ```

---

## T0.5: Simulation Core Isolation Enforced

### Test Name
`test_t0_05_simulation_core_isolation`

### Test Location
`crates/sim/tests/isolation/t0_05_isolation.rs`  
CI script: `scripts/check_sim_isolation.py` or inline `just` target

### Setup
1. Parse `crates/sim/Cargo.toml` for dependencies
2. Scan `crates/sim/src/**/*.rs` for forbidden API patterns

### Assertions
**Crate Separation:**
1. **sim crate exists** at `crates/sim/`
2. **sim crate has `#![deny(unsafe_code)]`** at crate root

**Dependency Allowlist (CI):**
3. **Only allowlisted dependencies** permitted for sim crate:
   - `std` (subset: no `std::fs`, `std::net`, `std::time`, `std::env`, `std::thread::sleep`)
   - (future: explicit allowlist in `scripts/allowed_sim_deps.txt`)
4. **No I/O crates:** `tokio`, `async-std`, `reqwest`, `hyper`, `mio`, etc.

**Forbidden-API Scan (CI):**
5. **No file I/O:** patterns `std::fs::`, `File::`, `OpenOptions`
6. **No network I/O:** patterns `std::net::`, `TcpStream`, `UdpSocket`
7. **No wall-clock time:** patterns `std::time::Instant`, `std::time::SystemTime`, `Instant::now()`, `SystemTime::now()`
8. **No thread sleep:** patterns `std::thread::sleep`, `thread::sleep`
9. **No environment access:** patterns `std::env::var`, `env::vars`
10. **No unseeded RNG:** patterns `rand::thread_rng`, `OsRng`, `getrandom`

**API Contract:**
11. **advance() takes explicit tick parameter:**
    ```rust
    fn advance(&mut self, tick: Tick, step_inputs: &[StepInput]) -> Snapshot
    ```
    - Parse function signature in `crates/sim/src/lib.rs`
    - Assert `tick` parameter exists (per ADR-0003)

### Constitution IDs Enforced
- **INV-0004** (Simulation Core Isolation): No I/O, networking, wall-clock, ambient RNG
- **KC-0001** (Kill: Boundary Violation): Boundary violation = project kill

### Fixtures Needed
- **Allowlist file:** `scripts/allowed_sim_deps.txt`
- **Forbidden patterns:** `scripts/forbidden_sim_patterns.txt`

---

## T0.5a: Tick/Floor Relationship Assertion

### Test Name
`test_t0_05a_tick_floor_relationship`

### Test Location
`crates/server/tests/integration/t0_05a_tick_floor.rs`

### Setup
1. Create World with seed=0, tick_rate_hz=60
2. Spawn characters
3. Call `world.advance(T, inputs)` for T = 0, 1, 2, ...

### Assertions
1. **Pre-advance:** `world.tick() == T`
2. **Post-advance:** `world.tick() == T + 1`
3. **Snapshot.tick == T + 1** (post-step tick)
4. **TargetTickFloor computation:**
   - `target_tick_floor = snapshot.tick + input_lead_ticks`
   - For `input_lead_ticks = 1`: `target_tick_floor = (T + 1) + 1 = T + 2`
5. **SnapshotProto.target_tick_floor matches** computed value
6. **Advance precondition enforced:**
   - Calling `world.advance(T+5, inputs)` when `world.tick() == T` must panic/error

### Constitution IDs Enforced
- **ADR-0006** (Input Tick Targeting): TargetTickFloor = server.current_tick + input_lead_ticks
- **ADR-0003** (Fixed Timestep): Explicit tick parameter contract

### Fixtures Needed
- None

---

## T0.6: Validation Per v0-parameters.md

### Test Name
`test_t0_06_input_validation_matrix`

### Test Location
`crates/server/tests/integration/t0_06_validation.rs`

### Setup
1. Start server in manual-step mode
2. Connect two clients, complete handshake
3. Send various InputCmdProto messages with invalid values

### Assertions

**Magnitude Clamping:**
1. `move_dir = [2.0, 0.0]` → clamped to `[1.0, 0.0]` + logged
2. `move_dir = [0.6, 0.8]` (magnitude=1.0) → accepted as-is
3. `move_dir = [0.7, 0.8]` (magnitude>1.0) → clamped to unit length + logged

**NaN/Inf Drop:**
4. `move_dir = [NaN, 0.0]` → dropped + logged
5. `move_dir = [Inf, 0.0]` → dropped + logged
6. `move_dir = [-Inf, 0.0]` → dropped + logged
7. `move_dir = [0.0, NaN]` → dropped + logged

**Tick Window:**
8. `cmd.tick < current_tick` (late) → dropped
9. `cmd.tick > current_tick + max_future_ticks` (too far future) → dropped
10. `cmd.tick == current_tick + max_future_ticks` → accepted (boundary)

**Rate Limit:**
11. Send 3 inputs for same (session, tick): 3rd dropped (per_tick_limit=2)
12. Send 2 inputs for same tick: both considered for InputSeq selection

**InputSeq Selection (DM-0026):**
13. Inputs with seq {5, 7, 6} for same (session, tick): seq=7 wins
14. Inputs with seq {8, 8} for same (session, tick): tie → LKI fallback + logged

**LastKnownIntent (DM-0023):**
15. No input for (player_id, tick): LKI used (initially [0,0])
16. After valid input with `move_dir=[1,0]`: subsequent LKI is [1,0]

**Player Identity Binding (INV-0003):**
17. InputCmdProto has no player_id field → server binds from session
18. Server ignores any client-provided player_id (if somehow present)

### Constitution IDs Enforced
- **INV-0003** (Authoritative Simulation): player_id bound by Server Edge
- **DM-0022** (InputTickWindow): Tick window validation
- **DM-0023** (LastKnownIntent): Input continuity fallback
- **DM-0026** (InputSeq): Deterministic selection

### Fixtures Needed
- None (programmatic invalid input generation)

---

## T0.7: Malformed Inputs Do Not Crash Server

### Test Name
`test_t0_07_malformed_input_robustness`

### Test Location
`crates/server/tests/integration/t0_07_robustness.rs`

### Setup
1. Start server in manual-step mode
2. Connect two clients
3. Send a variety of malformed/adversarial InputCmdProto messages

### Assertions
**Server does not crash/panic for any of:**
1. `move_dir = [NaN, NaN]`
2. `move_dir = [Inf, -Inf]`
3. `move_dir = [1e308, 1e308]` (huge magnitude)
4. `move_dir = [-1e308, -1e308]`
5. `tick = 0` (below floor, if floor > 0)
6. `tick = u64::MAX`
7. `input_seq = 0`
8. `input_seq = u64::MAX`
9. Truncated protobuf message (incomplete bytes)
10. Empty protobuf message
11. Protobuf with unknown fields
12. Rapid-fire messages (100+ in one network tick)

**Server continues operating:**
13. After all malformed inputs, server still processes valid inputs
14. Match can still complete normally
15. Replay artifact is still generated

### Constitution IDs Enforced
- **KC-0001** (Kill: Boundary Violation): Must not violate Simulation Core boundary under any input

### Fixtures Needed
- **Fuzz corpus (optional):** `fixtures/fuzz/malformed_inputs/`

---

## T0.8: Replay Artifact Generated with All Required Fields

### Test Name
`test_t0_08_replay_artifact_fields`

### Test Location
`crates/server/tests/integration/t0_08_replay_artifact.rs`

### Setup
1. Run complete match (match_duration_ticks=3600 or shortened for test)
2. Or trigger disconnect to end match early
3. Read generated replay artifact from `replays/{match_id}.replay`

### Assertions
**All required fields present and valid:**

| Field | Assertion |
|-------|-----------|
| `replay_format_version` | `>= 1` (starts at 1) |
| `initial_baseline.tick` | `== 0` |
| `initial_baseline.entities` | `len() == 2`, sorted by entity_id |
| `initial_baseline.digest` | Non-zero, matches recomputed |
| `seed` | `== 0` (default seed) |
| `rng_algorithm` | Non-empty string (e.g., "ChaCha8Rng") |
| `tick_rate_hz` | `== 60` |
| `state_digest_algo_id` | `== "statedigest-v0-fnv1a64-le-f64canon-eidasc-posvel"` |
| `entity_spawn_order` | `== [0, 1]` or test-mode override |
| `player_entity_mapping` | 2 entries, sorted by player_id |
| `tuning_parameters` | Contains `{key: "move_speed", value: "5.0"}` |
| `inputs` | Non-empty, sorted by (tick, player_id) |
| `build_fingerprint.binary_sha256` | 64 hex chars or "unknown" (dev) |
| `build_fingerprint.target_triple` | Non-empty (e.g., "x86_64-pc-windows-msvc") |
| `build_fingerprint.profile` | "release" or "dev" |
| `build_fingerprint.git_commit` | Non-empty string |
| `final_digest` | Non-zero |
| `checkpoint_tick` | `== initial_baseline.tick + match_duration_ticks` or disconnect tick |
| `end_reason` | "complete" or "disconnect" |

**AppliedInput stream integrity:**
1. For each player_id in player_entity_mapping
2. For each tick in [initial_baseline.tick, checkpoint_tick)
3. Exactly one AppliedInput entry exists

### Constitution IDs Enforced
- **DM-0017** (ReplayArtifact): Versioned record for replay verification
- **INV-0006** (Replay Verifiability): All required fields for reproduction

### Fixtures Needed
- **Expected artifact schema:** `fixtures/replay/expected_artifact_schema.json`

---

## T0.9: Replay Verification Passes

### Test Name
`test_t0_09_replay_verification`

### Test Location
`crates/replay/tests/integration/t0_09_verification.rs`

### Setup
1. Generate replay artifact from completed match
2. Load artifact into ReplayVerifier
3. Execute verification procedure

### Assertions
**Verification procedure:**
1. **Build fingerprint check:** SHA-256 + target_triple + profile match current binary
2. **AppliedInput stream integrity:** No gaps, no duplicates per (player_id, tick)
3. **World initialization:** `World::new(artifact.seed, artifact.tick_rate_hz)`
4. **Spawn reconstruction:** For each player_id in `entity_spawn_order`:
   - `entity_id = world.spawn_character(player_id)`
   - Assert `entity_id == player_entity_mapping[player_id].entity_id`
5. **Initialization anchor:** `world.baseline().digest == artifact.initial_baseline.digest`
6. **Replay loop:** For each tick T in [initial_baseline.tick, checkpoint_tick):
   - Extract AppliedInputs where tick == T
   - Sort by player_id ascending
   - Convert to StepInput
   - `world.advance(T, step_inputs)`
7. **Final tick check:** `world.tick() == artifact.checkpoint_tick`
8. **Final digest check:** `world.state_digest() == artifact.final_digest`

### Constitution IDs Enforced
- **INV-0006** (Replay Verifiability): Replay reproduction from artifact
- **ADR-0007** (StateDigest): FNV-1a 64-bit, canonicalization

### Fixtures Needed
- **Golden replay artifact:** `fixtures/replay/t0_09_golden.replay`
  - Known match with deterministic seed=0
  - Pre-verified final_digest

---

## T0.10: Initialization Anchor Failure

### Test Name
`test_t0_10_initialization_anchor_failure`

### Test Location
`crates/replay/tests/integration/t0_10_anchor_failure.rs`

### Setup
1. Generate valid replay artifact
2. Mutate `initial_baseline.digest` to incorrect value (e.g., XOR with 0x1)
3. Run ReplayVerifier

### Assertions
1. **Verification fails immediately** after spawn reconstruction
2. **Failure occurs BEFORE any advance() calls**
3. **Error message indicates** "initialization anchor mismatch" or equivalent
4. **No partial replay** — verifier does not continue past anchor check
5. **Mutating other baseline fields** (entities, tick) also triggers anchor failure

### Constitution IDs Enforced
- **INV-0006** (Replay Verifiability): Initialization anchor verification requirement

### Fixtures Needed
- **Mutated artifact:** Programmatically generated from valid artifact

---

## T0.11: Future Input Non-Interference

### Test Name
`test_t0_11_future_input_buffering`

### Test Location
`crates/server/tests/integration/t0_11_future_input.rs`

### Setup
1. Start server, world at tick T
2. Client sends input for tick T+k where k > max_future_ticks
3. Client sends input for tick T+1 (within window)

### Assertions
**Far-future rejection:**
1. Input for T + (max_future_ticks + 1) = T + 121 is **dropped** + logged
2. Input for T + max_future_ticks = T + 120 is **accepted** (boundary)

**Near-future buffering:**
3. Input for T+1 is **buffered without affecting T**
4. When processing tick T:
   - Only inputs where cmd.tick == T are considered
   - T+1 input remains in buffer untouched
5. When processing tick T+1:
   - Buffered T+1 input is used
   - Position reflects intended movement

**Buffer independence:**
6. Sending many future inputs does not alter current tick outcome
7. State digest at tick T is identical regardless of buffered future inputs

### Constitution IDs Enforced
- **DM-0022** (InputTickWindow): Future-only acceptance window

### Fixtures Needed
- None

---

## T0.12: LastKnownIntent Determinism

### Test Name
`test_t0_12_last_known_intent_determinism`

### Test Location
`crates/server/tests/integration/t0_12_lki_determinism.rs`

### Setup
1. Start server, connect clients
2. Client 0: sends inputs for ticks 0, 1, 2, then stops (gap at ticks 3-5)
3. Client 1: sends inputs for all ticks
4. Complete match
5. Run replay verification

### Assertions
**Gap filling:**
1. For ticks 3-5, player 0 uses LKI (last move_dir from tick 2)
2. AppliedInput entries for player 0, ticks 3-5 have `is_fallback=true`
3. AppliedInput entries for player 0, ticks 0-2 have `is_fallback=false`

**Recording in artifact:**
4. All LKI-filled entries are present in `artifact.inputs`
5. `is_fallback` flag correctly set for each entry

**Replay produces same digest:**
6. Replay verification passes
7. `computed_final_digest == artifact.final_digest`

**LKI initial value:**
8. Before any valid input, LKI is [0, 0] (neutral)
9. After input with move_dir=[1,0], LKI becomes [1,0]

### Constitution IDs Enforced
- **DM-0023** (LastKnownIntent): Input continuity fallback
- **INV-0001** (Deterministic Simulation): Determinism with gaps

### Fixtures Needed
- **Expected LKI sequence:** `fixtures/lki/t0_12_expected.json`

---

## T0.12a: Non-Canonical AppliedInput Storage Order Test

### Test Name
`test_t0_12a_noncanonical_input_order_robustness`

### Test Location
`crates/replay/tests/integration/t0_12a_order_robustness.rs`

### Setup
1. Generate valid replay artifact
2. **Fault injection:** Shuffle `inputs` array to violate canonical order
   - Mix tick ordering (non-ascending)
   - Mix player_id ordering within same tick
3. Run ReplayVerifier

### Assertions
1. **Verifier canonicalizes successfully:**
   - Extracts inputs by tick
   - Sorts by player_id ascending
   - Produces correct StepInput sequence
2. **Verification passes** (same final digest)
3. **Dev warning emitted** (optional): "Non-canonical AppliedInput order detected"
4. **No panic or error** from order violation alone

### Constitution IDs Enforced
- **INV-0007** (Deterministic Ordering): Verifier must canonicalize

### Fixtures Needed
- **Shuffled artifact:** Programmatically shuffled from valid artifact

---

## T0.13: Validation Matrix

### Test Name
`test_t0_13_validation_matrix_comprehensive`

### Test Location
`crates/server/tests/integration/t0_13_validation_matrix.rs`

### Setup
1. Start server in manual-step mode
2. Connect clients
3. Systematically test each validation rule

### Assertions

**NaN detection:**
| Input | Expected |
|-------|----------|
| `[NaN, 0.0]` | DROP |
| `[0.0, NaN]` | DROP |
| `[NaN, NaN]` | DROP |
| `[f64::NAN, 1.0]` | DROP |

**Magnitude clamping:**
| Input | Expected |
|-------|----------|
| `[2.0, 0.0]` | CLAMP to `[1.0, 0.0]` |
| `[0.0, 2.0]` | CLAMP to `[0.0, 1.0]` |
| `[0.7071, 0.7071]` | ACCEPT (≈1.0) |
| `[0.8, 0.8]` | CLAMP (magnitude ≈1.13) |

**Tick window:**
| Scenario | Expected |
|----------|----------|
| `cmd.tick == current_tick - 1` | DROP (late) |
| `cmd.tick == current_tick` | ACCEPT |
| `cmd.tick == current_tick + 120` | ACCEPT (boundary) |
| `cmd.tick == current_tick + 121` | DROP (too far) |

**Rate limit:**
| Scenario | Expected |
|----------|----------|
| 2 inputs for same tick | Both considered for InputSeq |
| 3 inputs for same tick | 3rd dropped |
| N > 2 inputs | At least N-2 dropped |

**InputSeq selection:**
| Scenario | Expected |
|----------|----------|
| seq {1, 2, 3} | Select seq=3 |
| seq {5, 5} | Tie → LKI fallback |
| seq {3, 5, 4} | Select seq=5 |

**TargetTickFloor enforcement:**
| Scenario | Expected |
|----------|----------|
| `cmd.tick < last_emitted_floor` | DROP |
| `cmd.tick == last_emitted_floor` | ACCEPT |

**Pre-Welcome input drop:**
| Scenario | Expected |
|----------|----------|
| Input received before ServerWelcome sent | DROP (no buffer, no log) |

### Constitution IDs Enforced
- **INV-0003** (Authoritative Simulation)
- **DM-0022** (InputTickWindow)
- **DM-0025** (TargetTickFloor)
- **DM-0026** (InputSeq)

### Fixtures Needed
- None (programmatic test matrix)

---

## T0.13a: Floor Enforcement Drop and Recovery Test

### Test Name
`test_t0_13a_floor_enforcement_recovery`

### Test Location
`crates/server/tests/integration/t0_13a_floor_recovery.rs`

### Setup
1. Start server, connect clients
2. Process N ticks normally
3. **Simulate snapshot packet loss:** Client does not receive SnapshotProto for ticks T through T+M
4. Client continues sending inputs targeting stale floor

### Assertions
**Drop behavior:**
1. Inputs targeting tick < `last_emitted_target_tick_floor` are dropped
2. Server logs floor enforcement drops
3. Client movement stalls (LKI fallback on server side)

**Recovery behavior:**
4. Deliver SnapshotProto containing new floor (tick T+M+1)
5. Client updates local floor to new value
6. Client sends inputs targeting >= new floor
7. Movement resumes within bounded ticks (≤ 2-3 ticks after floor update received)

**System stability:**
8. No crash or hang during floor staleness period
9. Match continues to completion
10. Replay verification still passes

### Constitution IDs Enforced
- **ADR-0006** (Input Tick Targeting): Floor enforcement and recovery
- **DM-0025** (TargetTickFloor): Monotonic floor updates

### Fixtures Needed
- None (network simulation in test)

---

## T0.14: Disconnect Handling

### Test Name
`test_t0_14_disconnect_handling`

### Test Location
`crates/server/tests/integration/t0_14_disconnect.rs`

### Setup
1. Start server, connect two clients
2. Process several ticks
3. Disconnect one client mid-match

### Assertions
**Tick completion:**
1. Current tick completes before shutdown begins
2. No mid-tick termination (world.tick() is post-step)

**Artifact persistence:**
3. ReplayArtifact is written to disk
4. `artifact.end_reason == "disconnect"`
5. `artifact.checkpoint_tick == world.tick()` at disconnect detection
6. All AppliedInputs up to checkpoint_tick are recorded

**Clean shutdown:**
7. Server exits cleanly (exit code 0 or defined non-zero for disconnect)
8. No panic, no crash
9. Remaining connected client receives final snapshot (best-effort)

### Constitution IDs Enforced
- **DM-0017** (ReplayArtifact): Artifact with end_reason="disconnect"

### Fixtures Needed
- None

---

## T0.15: Match Termination

### Test Name
`test_t0_15_match_complete_termination`

### Test Location
`crates/server/tests/integration/t0_15_match_complete.rs`

### Setup
1. Start server with `match_duration_ticks=100` (shortened for test)
2. Connect two clients
3. Let match run to completion

### Assertions
**Duration enforcement:**
1. Server processes exactly ticks [0, match_duration_ticks) = [0, 100)
2. Final `world.tick() == 100` (post-last-step)
3. Server does not process tick 100 (match ends at checkpoint)

**Artifact content:**
4. `artifact.end_reason == "complete"`
5. `artifact.checkpoint_tick == initial_tick + match_duration_ticks == 100`
6. `artifact.final_digest == world.state_digest()` at tick 100

**Clean shutdown:**
7. Server exits with exit code 0
8. Final snapshot broadcast before exit

### Constitution IDs Enforced
- **DM-0010** (Match): Match lifecycle with defined end
- **DM-0017** (ReplayArtifact): Artifact with end_reason="complete"

### Fixtures Needed
- None

---

## T0.16: Connection Timeout

### Test Name
`test_t0_16_connection_timeout`

### Test Location
`crates/server/tests/integration/t0_16_connection_timeout.rs`

### Setup
1. Start server with `connect_timeout_ms=1000` (shortened for test)
2. Connect only one client (or zero clients)
3. Wait for timeout to expire

### Assertions
**Timeout behavior:**
1. Server waits up to connect_timeout_ms
2. If < 2 sessions connect, server aborts

**Exit behavior:**
3. Server exits with **non-zero exit code**
4. Specific exit code is documented/consistent

**No artifact:**
5. **No ReplayArtifact is written** (pre-match timeout)
6. Replay directory does not contain new files

**Log verification:**
7. Log contains timeout event token (e.g., "CONNECT_TIMEOUT" or similar)
8. CI test asserts on both exit code AND log token

### Constitution IDs Enforced
- **docs/networking/v0-parameters.md**: connect_timeout_ms parameter

### Fixtures Needed
- None

---

## T0.17: Simulation Core PlayerId Non-Assumption

### Test Name
`test_t0_17_noncontiguous_player_ids`

### Test Location
`crates/server/tests/integration/t0_17_playerid_nonassumption.rs`

### Setup
1. Start server with:
   - `--test-mode`
   - `--test-player-ids 17,99`
2. Connect two clients
3. Run complete match with movement inputs

### Assertions
**Assignment correctness:**
1. First session receives `player_id=17`
2. Second session receives `player_id=99`
3. `ServerWelcome.player_id` reflects assigned ID

**Movement correctness:**
4. Both players move correctly with their respective inputs
5. Entity positions update according to MOVE_SPEED formula
6. No special behavior for specific PlayerIds

**Artifact correctness:**
7. `artifact.test_mode == true`
8. `artifact.test_player_ids == [17, 99]`
9. `artifact.entity_spawn_order == [17, 99]`
10. `artifact.player_entity_mapping` contains both (17, eid1) and (99, eid2)

**Replay verification:**
11. Replay verification passes with non-contiguous IDs
12. Final digest matches

**Negative check:**
13. Simulation Core code does NOT contain literals `0` or `1` as PlayerId assumptions
    (verified via code scan or runtime assertion)

### Constitution IDs Enforced
- **DM-0019** (PlayerId): No assumption of contiguous/zero-based IDs
- **INV-0007** (Deterministic Ordering): Ordering by player_id, not by assumed values

### Fixtures Needed
- None

---

## T0.18: Floor Coherency Server-Side Broadcast

### Test Name
`test_t0_18_floor_coherency_broadcast`

### Test Location
`crates/server/tests/integration/t0_18_floor_coherency.rs`

### Setup
1. Start server in manual-step mode
2. Connect two clients
3. Advance world through several ticks
4. Capture SnapshotProto bytes before sending to each client

### Assertions
**Byte-identical broadcast:**
1. For each tick T, capture serialized SnapshotProto bytes
2. Assert: `bytes_to_client_0 == bytes_to_client_1` (byte-for-byte)
3. This includes:
   - `tick`
   - `entities` (same order, same values)
   - `digest`
   - `target_tick_floor`

**Server-side assertion:**
4. Server internally asserts payload identity before send
5. Alternatively: single serialization path (serialize once, send twice)

**Floor value coherency:**
6. `SnapshotProto.target_tick_floor` is identical for both clients at each tick
7. Value matches expected: `world.tick() + input_lead_ticks`

### Constitution IDs Enforced
- **ADR-0006** (Input Tick Targeting): v0 Floor Coherency normative constraint

### Fixtures Needed
- None

---

## T0.19: Schema Identity CI Gate

### Test Name
`test_t0_19_schema_identity_ci`

### Test Location
CI script: `scripts/verify_schema_identity.py` or `just` target  
Cargo metadata check: `crates/*/Cargo.toml`

### Setup
1. Build both server and client binaries
2. Extract dependency metadata using `cargo metadata`

### Assertions
**Shared crate existence:**
1. Crate `flowstate_wire` exists at `crates/wire/`
2. Contains protobuf message definitions (prost derives)

**Dependency verification:**
3. Server binary (`crates/server/`) depends on `flowstate_wire`
4. Client binary (`crates/client/`) depends on `flowstate_wire`
5. Both depend on **same package ID**:
   - Same crate name
   - Same version
   - Same source (workspace member, not external)

**CI gate:**
6. `just ci` includes schema identity check
7. Fail if either binary does not depend on shared wire crate
8. Fail if versions diverge (should not be possible with workspace)

**Compile-time enforcement:**
9. Message types are imported from `flowstate_wire`, not redefined
10. No duplicate protobuf message definitions in client/server crates

### Constitution IDs Enforced
- **ADR-0005** (v0 Networking Architecture): Same-build scope, shared schema

### Fixtures Needed
- None

---

## T0.20: `just ci` Passes

### Test Name
`test_t0_20_just_ci_passes`

### Test Location
CI pipeline / local execution

### Setup
1. Clean workspace (optional: `cargo clean`)
2. Run `just ci`

### Assertions
**All sub-commands pass:**
1. `just fmt` — formatting check passes (no changes needed)
2. `just lint` — clippy with `-D warnings` passes
3. `just test` — all unit and integration tests pass
4. `just ids` — Constitution ID validation passes
5. `just spec-lint` — spec structure validation passes

**Exit code:**
6. `just ci` exits with code 0

**No warnings treated as errors:**
7. Clippy warnings fail the build (`-D warnings`)
8. No test failures or panics

### Constitution IDs Enforced
- **AGENTS.md**: PR checklist requirement

### Fixtures Needed
- None

---

## Fixture Summary

| Fixture | Path | Purpose |
|---------|------|---------|
| Movement golden file | `fixtures/movement/t0_04_60ticks_right.json` | Deterministic movement verification |
| Dependency allowlist | `scripts/allowed_sim_deps.txt` | Simulation Core isolation |
| Forbidden patterns | `scripts/forbidden_sim_patterns.txt` | API scan patterns |
| Expected artifact schema | `fixtures/replay/expected_artifact_schema.json` | Artifact field validation |
| Golden replay artifact | `fixtures/replay/t0_09_golden.replay` | Pre-verified replay for verification test |
| LKI expected sequence | `fixtures/lki/t0_12_expected.json` | LastKnownIntent verification |

---

## Test Dependency Graph

```
T0.1 (handshake)
  ├── T0.2 (baseline delivery)
  └── T0.3 (input targeting)
        ├── T0.4 (movement) ← T0.5 (isolation)
        │     └── T0.5a (tick/floor relationship)
        ├── T0.6 (validation) → T0.7 (robustness)
        │     ├── T0.11 (future input)
        │     ├── T0.12 (LKI determinism)
        │     │     └── T0.12a (order robustness)
        │     ├── T0.13 (validation matrix)
        │     │     └── T0.13a (floor recovery)
        │     └── T0.17 (PlayerId non-assumption)
        └── T0.8 (artifact fields)
              ├── T0.9 (replay verification)
              │     └── T0.10 (anchor failure)
              ├── T0.14 (disconnect)
              └── T0.15 (match complete)

T0.16 (timeout) ← independent
T0.18 (floor coherency) ← depends on T0.1
T0.19 (schema identity) ← CI-level check
T0.20 (just ci) ← aggregates all
```

---

## Implementation Priority

**Phase 1 (Foundation):**
- T0.5: Simulation Core isolation (blocks all sim work)
- T0.1: Handshake (blocks all client-server tests)
- T0.2: Baseline delivery

**Phase 2 (Core Loop):**
- T0.4: Movement determinism
- T0.5a: Tick/floor relationship
- T0.3: Input targeting
- T0.6: Input validation

**Phase 3 (Robustness):**
- T0.7: Malformed input handling
- T0.11: Future input buffering
- T0.12: LastKnownIntent
- T0.13: Validation matrix

**Phase 4 (Replay):**
- T0.8: Artifact generation
- T0.9: Replay verification
- T0.10: Anchor failure
- T0.12a: Order robustness

**Phase 5 (Lifecycle):**
- T0.14: Disconnect handling
- T0.15: Match termination
- T0.16: Connection timeout

**Phase 6 (Polish):**
- T0.13a: Floor recovery
- T0.17: PlayerId non-assumption
- T0.18: Floor coherency
- T0.19: Schema identity
- T0.20: Full CI validation
