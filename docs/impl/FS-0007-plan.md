# FS-0007: v0 Two-Client Multiplayer Slice — Implementation Plan

> **Spec:** [FS-0007-v0-multiplayer-slice.md](../specs/FS-0007-v0-multiplayer-slice.md)  
> **Issue:** [#7](https://github.com/project-flowstate/flowstate/issues/7)  
> **Date:** 2025-12-23

---

## 1. Task Breakdown by Spec Section

Tasks are ordered by dependency. Within each section, tasks are listed in implementation order.

### 1.1 Simulation Core Types (`crates/sim/src/lib.rs`)

| Task ID | Description | Dependencies | Est. Complexity |
|---------|-------------|--------------|-----------------|
| SIM-001 | Define type aliases: `Tick = u64`, `PlayerId = u8`, `EntityId = u64` | None | Trivial |
| SIM-002 | Implement `EntitySnapshot` struct with `entity_id`, `position: [f64; 2]`, `velocity: [f64; 2]` | SIM-001 | Trivial |
| SIM-003 | Implement `StepInput` struct with `player_id`, `move_dir: [f64; 2]` | SIM-001 | Trivial |
| SIM-004 | Implement `Baseline` struct with `tick`, `entities: Vec<EntitySnapshot>`, `digest: u64` | SIM-001, SIM-002 | Trivial |
| SIM-005 | Implement `Snapshot` struct (same fields as Baseline) | SIM-001, SIM-002 | Trivial |
| SIM-006 | Implement internal `Character` struct with `entity_id`, `player_id`, `position`, `velocity` | SIM-001 | Low |
| SIM-007 | Implement `World` struct with internal fields: `tick`, `tick_rate_hz`, `dt_seconds`, `entities` (character storage), `next_entity_id` counter | SIM-006 | Low |
| SIM-008 | Implement `World::new(seed, tick_rate_hz)` — initialize at tick 0, compute `dt_seconds` | SIM-007 | Low |
| SIM-009 | Implement `World::spawn_character(player_id) -> EntityId` — deterministic ID assignment | SIM-007 | Low |
| SIM-010 | Implement `World::tick()`, `World::tick_rate_hz()` accessors | SIM-007 | Trivial |
| SIM-011 | Implement f64 canonicalization: `-0.0 → +0.0`, `NaN → 0x7ff8000000000000` (ADR-0007) | None | Low |
| SIM-012 | Implement FNV-1a 64-bit hasher (offset `0xcbf29ce484222325`, prime `0x100000001b3`) | None | Low |
| SIM-013 | Implement `World::state_digest()` — hash tick + entities by EntityId ascending per ADR-0007 | SIM-011, SIM-012 | Medium |
| SIM-014 | Implement `World::baseline()` — return Baseline with sorted entities + digest | SIM-004, SIM-013 | Low |
| SIM-015 | Implement v0 movement physics in `World::advance()`: `MOVE_SPEED=5.0`, `velocity = move_dir * MOVE_SPEED`, `position += velocity * dt` | SIM-003, SIM-007 | Medium |
| SIM-016 | Implement `World::advance(tick, step_inputs) -> Snapshot` with precondition assert, tick increment, return Snapshot | SIM-015 | Medium |
| SIM-017 | Add `#![deny(unsafe_code)]` and document isolation constraints (INV-0004) | None | Trivial |

### 1.2 v0 Movement Model

| Task ID | Description | Dependencies | Est. Complexity |
|---------|-------------|--------------|-----------------|
| MOV-001 | Define `MOVE_SPEED = 5.0` constant (compile-time, recorded in ReplayArtifact) | SIM-015 | Trivial |
| MOV-002 | Implement `move_dir` magnitude clamping in advance() (defense-in-depth; validation is Server Edge) | SIM-015 | Low |
| MOV-003 | Verify deterministic f64 arithmetic — document fast-math disabled for sim crate | SIM-015 | Low |

### 1.3 Protocol Messages (`crates/wire/`)

| Task ID | Description | Dependencies | Est. Complexity |
|---------|-------------|--------------|-----------------|
| WIRE-001 | Create `crates/wire/` crate with `Cargo.toml`, add `prost` + `prost-types` dependencies | None | Low |
| WIRE-002 | Define `EntitySnapshotProto` message (prost derive) | WIRE-001 | Low |
| WIRE-003 | Define `ClientHello` message (empty for v0) | WIRE-001 | Trivial |
| WIRE-004 | Define `ServerWelcome` message: `target_tick_floor`, `tick_rate_hz`, `player_id`, `controlled_entity_id` | WIRE-001 | Low |
| WIRE-005 | Define `JoinBaseline` message: `tick`, `entities`, `digest` | WIRE-002 | Low |
| WIRE-006 | Define `InputCmdProto` message: `tick`, `input_seq`, `move_dir` (no `player_id`) | WIRE-001 | Low |
| WIRE-007 | Define `SnapshotProto` message: `tick`, `entities`, `digest`, `target_tick_floor` | WIRE-002 | Low |
| WIRE-008 | Define channel constants: `CHANNEL_REALTIME = 0`, `CHANNEL_CONTROL = 1` | WIRE-001 | Trivial |
| WIRE-009 | Add wire crate as dependency of sim crate (for shared types awareness only) | WIRE-001 | Trivial |

### 1.4 Replay Artifact (`crates/replay/`)

| Task ID | Description | Dependencies | Est. Complexity |
|---------|-------------|--------------|-----------------|
| REP-001 | Create `crates/replay/` crate with `Cargo.toml` | None | Low |
| REP-002 | Define `AppliedInput` struct: `tick`, `player_id`, `move_dir`, `is_fallback` | REP-001 | Low |
| REP-003 | Define `BuildFingerprint` struct: `binary_sha256`, `target_triple`, `profile`, `git_commit` | REP-001 | Low |
| REP-004 | Define `TuningParameter` struct: `key: String`, `value: String` | REP-001 | Trivial |
| REP-005 | Define `PlayerEntityMapping` struct: `player_id`, `entity_id` | REP-001 | Trivial |
| REP-006 | Define `ReplayArtifact` protobuf message with all required fields per spec | REP-002, REP-003, REP-004, REP-005 | Medium |
| REP-007 | Implement `ReplayArtifact::serialize()` — deterministic protobuf encoding | REP-006 | Low |
| REP-008 | Implement `ReplayArtifact::deserialize()` | REP-006 | Low |
| REP-009 | Implement BuildFingerprint acquisition at runtime: SHA-256 of executable, target triple, profile | REP-003 | Medium |
| REP-010 | Implement ReplayVerifier — initialization reconstruction, baseline digest check, replay loop, final digest check | REP-006, SIM-016 | High |
| REP-011 | Implement AppliedInput stream integrity validation (no gaps, no duplicates) | REP-010 | Medium |
| REP-012 | Add replay crate as dependency of server crate | REP-001 | Trivial |

### 1.5 Server Edge (`crates/server/`)

| Task ID | Description | Dependencies | Est. Complexity |
|---------|-------------|--------------|-----------------|
| SRV-001 | Create `crates/server/` crate with `Cargo.toml`, add enet-rs dependency | None | Low |
| SRV-002 | Implement ENet host initialization with two channels (Realtime, Control) | SRV-001 | Medium |
| SRV-003 | Implement Session struct: peer handle, `player_id`, `last_emitted_target_tick_floor`, `last_known_intent`, `last_valid_cmd_tick`, `input_seq` state | SRV-001 | Medium |
| SRV-004 | Implement connection accept loop with timeout (`connect_timeout_ms`) | SRV-002, SRV-003 | Medium |
| SRV-005 | Implement PlayerId assignment (v0 default: connection order 0, 1; test-mode override) | SRV-003 | Low |
| SRV-006 | Implement CLI parsing: `--seed`, `--replay-dir`, `--test-mode`, `--test-player-ids` | SRV-001 | Low |
| SRV-007 | Implement environment variable fallbacks: `FLOWSTATE_SEED`, `FLOWSTATE_REPLAY_DIR`, `FLOWSTATE_TEST_MODE`, `FLOWSTATE_TEST_PLAYER_IDS` | SRV-006 | Low |
| SRV-008 | Implement MatchId generation (16-64 chars, `[A-Za-z0-9_-]`) | SRV-001 | Low |
| SRV-009 | Implement handshake: receive ClientHello, send ServerWelcome + JoinBaseline | SRV-003, WIRE-003, WIRE-004, WIRE-005 | Medium |
| SRV-010 | Implement input buffer keyed by `(player_id, tick)` with window bounds | SRV-003 | Medium |
| SRV-011 | Implement InputSeq tracking with tie detection (`max_input_seq`, `max_seq_tied`) | SRV-010 | Medium |
| SRV-012 | Implement validation: NaN/Inf detection → DROP | SRV-010 | Low |
| SRV-013 | Implement validation: magnitude > 1.0 → CLAMP | SRV-010 | Low |
| SRV-014 | Implement validation: tick floor enforcement (`cmd.tick < last_emitted_target_tick_floor`) → DROP | SRV-010 | Low |
| SRV-015 | Implement validation: tick monotonicity (`cmd.tick < last_valid_cmd_tick`) → DROP | SRV-010 | Low |
| SRV-016 | Implement validation: tick window (`cmd.tick < current_tick` or `> current_tick + max_future_ticks`) → DROP | SRV-010 | Low |
| SRV-017 | Implement validation: rate limit per (session, tick) with `per_tick_limit = ceil(120/60) = 2` | SRV-010 | Medium |
| SRV-018 | Implement buffer cap: evict entries below window floor on tick advance | SRV-010 | Low |
| SRV-019 | Implement LastKnownIntent fallback (DM-0023): initial `[0,0]`, update on valid input | SRV-003 | Low |
| SRV-020 | Implement AppliedInput generation: buffer or LKI, record `is_fallback` | SRV-010, SRV-019, REP-002 | Medium |
| SRV-021 | Implement StepInput conversion from AppliedInput (sorted by player_id ascending) | SRV-020, SIM-003 | Low |
| SRV-022 | Implement TargetTickFloor computation: `world.tick() + input_lead_ticks` after advance | SRV-003 | Low |
| SRV-023 | Implement SnapshotProto broadcast (byte-identical to all sessions) | SRV-002, WIRE-007 | Medium |
| SRV-024 | Implement disconnect detection via ENet events | SRV-002 | Medium |
| SRV-025 | Implement match termination: complete tick, set `end_reason`, persist ReplayArtifact | SRV-024, REP-006 | Medium |
| SRV-026 | Implement pre-Welcome input drop (discard immediately without buffering) | SRV-009, SRV-010 | Low |
| SRV-027 | Add logging for all validation drops and protocol violations | SRV-012–SRV-017 | Low |

### 1.6 Server Tick Loop

| Task ID | Description | Dependencies | Est. Complexity |
|---------|-------------|--------------|-----------------|
| LOOP-001 | Implement tick loop pacing at `tick_rate_hz` (wall-clock for production) | SRV-002 | Medium |
| LOOP-002 | Implement manual-step mode for tests (no wall-clock pacing) | LOOP-001 | Low |
| LOOP-003 | Implement match duration check (`current_tick >= initial_tick + match_duration_ticks`) | LOOP-001 | Low |
| LOOP-004 | Implement per-tick receive and buffer cycle | SRV-010, LOOP-001 | Medium |
| LOOP-005 | Implement per-tick AppliedInput collection for all players | SRV-020, LOOP-004 | Medium |
| LOOP-006 | Implement per-tick advance call and snapshot broadcast | SIM-016, SRV-023, LOOP-005 | Medium |
| LOOP-007 | Implement per-tick ReplayArtifact input recording (canonical order) | REP-006, LOOP-005 | Low |
| LOOP-008 | Implement disconnect handling within tick loop (complete tick first) | SRV-024, LOOP-006 | Medium |

### 1.7 Game Client (v0 Test Harness)

| Task ID | Description | Dependencies | Est. Complexity |
|---------|-------------|--------------|-----------------|
| CLI-001 | Create minimal Rust test client binary or test module | WIRE-001 | Medium |
| CLI-002 | Implement ENet client connection and handshake | CLI-001, WIRE-003, WIRE-004 | Medium |
| CLI-003 | Implement JoinBaseline reception and state initialization | CLI-002, WIRE-005 | Medium |
| CLI-004 | Implement TargetTickFloor tracking: max(previous, received) | CLI-003 | Low |
| CLI-005 | Implement InputSeq generation (monotonically increasing) | CLI-001 | Low |
| CLI-006 | Implement InputCmdProto send with tick >= TargetTickFloor | CLI-004, CLI-005, WIRE-006 | Medium |
| CLI-007 | Implement SnapshotProto reception and state update | CLI-003, WIRE-007 | Medium |
| CLI-008 | Implement programmatic WASD input simulation for tests | CLI-006 | Low |
| CLI-009 | Implement test assertions: position matches expected deterministic values | CLI-007, SIM-015 | Medium |

### 1.8 CI and Validation Infrastructure

| Task ID | Description | Dependencies | Est. Complexity |
|---------|-------------|--------------|-----------------|
| CI-001 | Add sim crate dependency allowlist check (no I/O, no networking, no time) | SIM-017 | Medium |
| CI-002 | Add forbidden-API source scan for sim crate (file I/O, sockets, `std::time`, etc.) | SIM-017 | Medium |
| CI-003 | Add T0.19 schema identity gate: verify wire crate is shared dependency | WIRE-001 | Medium |
| CI-004 | Add replay verification integration test | REP-010 | High |
| CI-005 | Update Justfile: `just ci` includes all Tier-0 gates | CI-001–CI-004 | Low |

---

## 2. Trace Matrix

### 2.1 Acceptance Criteria → Implementation

| Criterion | Description | Code Location(s) | Test(s) |
|-----------|-------------|------------------|---------|
| **AC-0001.1** | Two clients connect, handshake, receive Baseline, synchronized | `crates/server/src/session.rs`, `crates/server/src/handshake.rs` | `tests/integration/handshake_test.rs` |
| **AC-0001.2** | WASD movement, LastKnownIntent, TargetTickFloor targeting, snapshots | `crates/sim/src/lib.rs` (advance), `crates/server/src/input.rs`, `crates/server/src/tick_loop.rs` | `tests/integration/movement_test.rs`, `tests/unit/lki_test.rs` |
| **AC-0001.3** | Replay verification, Simulation Core isolation, fixed timestep | `crates/replay/src/verifier.rs`, `crates/sim/src/lib.rs` | `tests/integration/replay_test.rs`, CI dependency scan |
| **AC-0001.4** | Validation per v0-parameters, InputSeq selection, disconnect handling | `crates/server/src/validation.rs`, `crates/server/src/session.rs` | `tests/unit/validation_test.rs`, `tests/integration/disconnect_test.rs` |
| **AC-0001.5** | ReplayArtifact with all fields, reproduces outcome | `crates/replay/src/artifact.rs`, `crates/server/src/replay_writer.rs` | `tests/integration/replay_test.rs` |

### 2.2 Tier-0 Gates → Implementation

| Gate | Description | Code Location(s) | Test(s) |
|------|-------------|------------------|---------|
| **T0.1** | Two clients connect, complete handshake | `crates/server/src/handshake.rs`, `crates/server/src/session.rs` | `tests/t0/t0_01_handshake.rs` |
| **T0.2** | JoinBaseline delivers initial Baseline | `crates/server/src/handshake.rs`, `crates/wire/src/messages.rs` | `tests/t0/t0_02_baseline.rs` |
| **T0.3** | Clients tag inputs per ADR-0006 | `crates/server/src/validation.rs` (floor check), test client | `tests/t0/t0_03_input_tagging.rs` |
| **T0.4** | WASD produces movement with exact f64 equality | `crates/sim/src/lib.rs` (advance, movement) | `tests/t0/t0_04_movement.rs` |
| **T0.5** | Simulation Core isolation enforced | `crates/sim/Cargo.toml` (no I/O deps), CI scan | `tests/t0/t0_05_isolation.rs`, CI job |
| **T0.5a** | Tick/floor relationship assertion | `crates/server/src/tick_loop.rs` | `tests/t0/t0_05a_tick_floor.rs` |
| **T0.6** | Validation per v0-parameters.md | `crates/server/src/validation.rs` | `tests/t0/t0_06_validation.rs` |
| **T0.7** | Malformed inputs do not crash server | `crates/server/src/validation.rs` | `tests/t0/t0_07_malformed.rs` |
| **T0.8** | Replay artifact generated with all fields | `crates/replay/src/artifact.rs`, `crates/server/src/replay_writer.rs` | `tests/t0/t0_08_artifact_fields.rs` |
| **T0.9** | Replay verification passes | `crates/replay/src/verifier.rs` | `tests/t0/t0_09_replay_verify.rs` |
| **T0.10** | Initialization anchor failure test | `crates/replay/src/verifier.rs` | `tests/t0/t0_10_anchor_fail.rs` |
| **T0.11** | Future input non-interference | `crates/server/src/input.rs` | `tests/t0/t0_11_future_input.rs` |
| **T0.12** | LastKnownIntent determinism | `crates/server/src/input.rs`, `crates/replay/src/verifier.rs` | `tests/t0/t0_12_lki_determinism.rs` |
| **T0.12a** | Non-canonical AppliedInput storage order test | `crates/replay/src/verifier.rs` | `tests/t0/t0_12a_noncanonical.rs` |
| **T0.13** | Validation matrix | `crates/server/src/validation.rs` | `tests/t0/t0_13_validation_matrix.rs` |
| **T0.13a** | Floor enforcement drop and recovery | `crates/server/src/validation.rs`, test harness | `tests/t0/t0_13a_floor_recovery.rs` |
| **T0.14** | Disconnect handling | `crates/server/src/session.rs`, `crates/server/src/tick_loop.rs` | `tests/t0/t0_14_disconnect.rs` |
| **T0.15** | Match termination | `crates/server/src/tick_loop.rs` | `tests/t0/t0_15_match_end.rs` |
| **T0.16** | Connection timeout | `crates/server/src/main.rs` | `tests/t0/t0_16_timeout.rs` |
| **T0.17** | PlayerId non-assumption test | `crates/sim/src/lib.rs`, `crates/server/src/session.rs` | `tests/t0/t0_17_playerid.rs` |
| **T0.18** | Floor coherency broadcast | `crates/server/src/tick_loop.rs` | `tests/t0/t0_18_floor_coherency.rs` |
| **T0.19** | Schema identity CI gate | `crates/wire/Cargo.toml`, CI script | CI job `check_wire_shared.rs` |
| **T0.20** | `just ci` passes | All crates | `just ci` |

---

## 3. File/Crate Inventory

### 3.1 Files to Modify

| Path | Purpose | Tasks |
|------|---------|-------|
| `crates/sim/src/lib.rs` | Replace stub with Simulation Core types and logic | SIM-001–SIM-017 |
| `crates/sim/Cargo.toml` | Add `#![deny(unsafe_code)]`, minimal deps only | SIM-017 |
| `Cargo.toml` (workspace) | Add new crate members: wire, server, replay | WIRE-001, SRV-001, REP-001 |
| `Justfile` | Add Tier-0 test commands, update `just ci` | CI-005 |
| `.gitignore` | Add `replays/` directory | REP-006 |

### 3.2 New Crates to Create

#### `crates/wire/` — Protocol Messages (shared)

| File | Purpose |
|------|---------|
| `crates/wire/Cargo.toml` | Crate manifest with prost dependency |
| `crates/wire/src/lib.rs` | Module exports |
| `crates/wire/src/messages.rs` | All protobuf message definitions (prost derive) |
| `crates/wire/src/channels.rs` | Channel constants |

#### `crates/server/` — Server Edge

| File | Purpose |
|------|---------|
| `crates/server/Cargo.toml` | Crate manifest with enet-rs, wire, sim, replay deps |
| `crates/server/src/main.rs` | Entry point, CLI parsing, server bootstrap |
| `crates/server/src/lib.rs` | Module exports for testing |
| `crates/server/src/session.rs` | Session struct and management |
| `crates/server/src/handshake.rs` | Connection accept and handshake logic |
| `crates/server/src/input.rs` | Input buffer, InputSeq tracking, AppliedInput generation |
| `crates/server/src/validation.rs` | All input validation rules |
| `crates/server/src/tick_loop.rs` | Main tick loop, pacing, advance calls |
| `crates/server/src/replay_writer.rs` | ReplayArtifact recording during match |
| `crates/server/src/config.rs` | v0 parameters from v0-parameters.md |

#### `crates/replay/` — Replay Artifact & Verification

| File | Purpose |
|------|---------|
| `crates/replay/Cargo.toml` | Crate manifest with prost, sha2, sim deps |
| `crates/replay/src/lib.rs` | Module exports |
| `crates/replay/src/artifact.rs` | ReplayArtifact struct and serialization |
| `crates/replay/src/applied_input.rs` | AppliedInput struct |
| `crates/replay/src/fingerprint.rs` | BuildFingerprint acquisition |
| `crates/replay/src/verifier.rs` | Replay verification logic |

### 3.3 Test Files to Create

| Path | Purpose |
|------|---------|
| `crates/sim/src/tests/mod.rs` | Sim unit test module |
| `crates/sim/src/tests/digest_test.rs` | StateDigest algorithm tests (ADR-0007) |
| `crates/sim/src/tests/movement_test.rs` | v0 movement model tests |
| `crates/sim/src/tests/advance_test.rs` | World::advance() contract tests |
| `tests/t0/` | Tier-0 integration tests (all T0.* gates) |
| `tests/integration/` | End-to-end integration tests |

### 3.4 CI/Infrastructure Files

| Path | Purpose |
|------|---------|
| `scripts/check_sim_isolation.py` | Forbidden-API scan for sim crate |
| `scripts/check_wire_shared.py` | T0.19 schema identity verification |

---

## 4. Definition of Done Checklist

### 4.1 Acceptance Criteria (AC-0001)

- [ ] **AC-0001.1:** Two native Game Clients connect to Game Server Instance, complete handshake (ServerWelcome with TargetTickFloor + tick_rate_hz + player_id + controlled_entity_id), receive JoinBaseline, remain synchronized (client state = last received authoritative Snapshot)
- [ ] **AC-0001.2:** WASD movement works; LastKnownIntent for missing inputs; TargetTickFloor-based targeting; both clients receive snapshots (one per tick)
- [ ] **AC-0001.3:** Replay verification passes (baseline + final digest); Simulation Core has no I/O; tick_rate_hz fixed at construction; advance() takes explicit tick + StepInput
- [ ] **AC-0001.4:** Validation per v0-parameters.md; InputSeq selection per Validation Rules; future inputs buffered correctly; player_id bound to session; disconnect → complete tick → persist artifact → shutdown; connection timeout aborts cleanly
- [ ] **AC-0001.5:** ReplayArtifact produced with all required fields; reproduces outcome on same build/platform

### 4.2 Tier-0 Gates

- [ ] **T0.1:** Two clients connect, complete handshake (ServerWelcome with TargetTickFloor + tick_rate_hz + player_id + controlled_entity_id)
- [ ] **T0.2:** JoinBaseline delivers initial Baseline; clients display Characters
- [ ] **T0.3:** Clients tag inputs per ADR-0006: InputCmd.tick >= TargetTickFloor, InputSeq monotonic
- [ ] **T0.4:** WASD produces movement with exact f64 equality (deterministic harness, N ticks, position.x increases by expected amount)
- [ ] **T0.5:** Simulation Core isolation enforced: crate separation, dependency allowlist (CI), forbidden-API scan (CI); advance() takes explicit tick per ADR-0003
- [ ] **T0.5a:** Tick/floor relationship assertion: After world.advance(T, inputs), snapshot.tick == T+1, TargetTickFloor in SnapshotProto == snapshot.tick + input_lead_ticks
- [ ] **T0.6:** Validation per v0-parameters.md: magnitude clamp, NaN/Inf drop, tick window, rate limit, InputSeq selection, LastKnownIntent, player_id bound to session
- [ ] **T0.7:** Malformed inputs do not crash server
- [ ] **T0.8:** Replay artifact generated with all required fields
- [ ] **T0.9:** Replay verification: initialization reconstruction, baseline digest check, half-open range, final digest match
- [ ] **T0.10:** Initialization anchor failure: mutated baseline digest fails immediately after spawn reconstruction
- [ ] **T0.11:** Future input non-interference: input for T+k (k > window) rejected; T+1 input buffered without affecting T
- [ ] **T0.12:** LastKnownIntent determinism: input gaps filled, recorded in artifact, replay produces same digest
- [ ] **T0.12a:** Non-canonical AppliedInput storage order test: verifier canonicalizes successfully, dev warning allowed
- [ ] **T0.13:** Validation matrix: NaN, magnitude, tick window, rate limit (N > limit drops at least N-limit), InputSeq selection (tied → LKI fallback), TargetTickFloor enforcement, pre-Welcome input drop
- [ ] **T0.13a:** Floor enforcement drop and recovery test: snapshot loss → inputs dropped → recovery within bounded ticks
- [ ] **T0.14:** Disconnect handling: complete current tick, persist artifact with end_reason="disconnect", clean shutdown
- [ ] **T0.15:** Match termination: complete match reaches match_duration_ticks, artifact persisted with end_reason="complete"
- [ ] **T0.16:** Connection timeout: server aborts if < 2 sessions within connect_timeout_ms, non-zero exit code, no artifact
- [ ] **T0.17:** PlayerId non-assumption: `--test-mode --test-player-ids 17,99` produces correct movement and replay verification
- [ ] **T0.18:** Floor coherency: server broadcasts byte-identical SnapshotProto to all sessions per tick
- [ ] **T0.19:** Schema identity CI gate: wire crate is shared dependency of both server and client
- [ ] **T0.20:** `just ci` passes

---

## 5. Dependency Graph (Critical Path)

```
SIM-001..SIM-017 (Simulation Core)
        │
        ├──────────────────┐
        ▼                  ▼
WIRE-001..WIRE-009     REP-001..REP-012
(Protocol Messages)    (Replay Artifact)
        │                  │
        └────────┬─────────┘
                 ▼
        SRV-001..SRV-027
        (Server Edge)
                 │
                 ▼
        LOOP-001..LOOP-008
        (Tick Loop)
                 │
                 ▼
        CLI-001..CLI-009
        (Test Client)
                 │
                 ▼
        CI-001..CI-005
        (CI Gates)
```

**Critical Path:** SIM → WIRE → SRV → LOOP → CLI → CI

**Parallelizable Work:**
- REP-001..REP-009 can proceed in parallel with WIRE-* after SIM-* completes
- CI-001..CI-003 can be developed in parallel with SRV-*

---

## 6. Risk Areas and Notes

### 6.1 High-Risk Items

| Area | Risk | Mitigation |
|------|------|------------|
| f64 Determinism | Cross-compile or optimization flags may break determinism | Disable fast-math, test on CI target, document constraints |
| ENet Integration | enet-rs crate may have API quirks or platform issues | Prototype connection handling early, have fallback plan |
| StateDigest | FNV-1a implementation correctness | Unit test against known test vectors |
| BuildFingerprint | Executable file locking on Windows | Implement graceful fallback for dev mode |

### 6.2 Implementation Notes

1. **Seed = 0 Default:** Per spec, default seed is 0 for deterministic testing. All CI runs use this.

2. **Test-Mode PlayerIds:** The `--test-mode --test-player-ids 17,99` feature is required for T0.17. Must be implemented early to validate Simulation Core boundary.

3. **Manual Step Mode:** Server tick loop must support non-paced stepping for CI tests. Use a trait or flag pattern.

4. **Byte-Identical Snapshots:** Serialize SnapshotProto once per tick, broadcast same bytes. This simplifies T0.18.

5. **Validation Order:** Rate limiting and basic checks at receive-time, InputSeq selection at apply-time.

---

## 7. Suggested Implementation Order

### Phase 1: Foundation (Week 1)
1. SIM-001 through SIM-017 (Simulation Core complete)
2. Unit tests for StateDigest, movement, advance

### Phase 2: Protocol (Week 1-2)
3. WIRE-001 through WIRE-009 (Protocol messages)
4. REP-001 through REP-009 (Replay artifact structures)

### Phase 3: Server Core (Week 2-3)
5. SRV-001 through SRV-011 (Session, handshake, input buffer)
6. SRV-012 through SRV-021 (Validation, AppliedInput)
7. LOOP-001 through LOOP-008 (Tick loop)

### Phase 4: Integration (Week 3-4)
8. SRV-022 through SRV-027 (Broadcast, disconnect, replay write)
9. REP-010 through REP-012 (Replay verification)
10. CLI-001 through CLI-009 (Test client)

### Phase 5: Validation (Week 4)
11. CI-001 through CI-005 (CI gates)
12. All T0.* tests
13. Final integration testing

---

## References

- [FS-0007 Spec](../specs/FS-0007-v0-multiplayer-slice.md)
- [v0 Parameters](../networking/v0-parameters.md)
- [ADR-0006: Input Tick Targeting](../adr/0006-input-tick-targeting.md)
- [ADR-0007: StateDigest Algorithm](../adr/0007-state-digest-algorithm-canonical-serialization.md)
- [Invariants](../constitution/invariants.md)
- [Domain Model](../constitution/domain-model.md)
- [Acceptance Criteria](../constitution/acceptance-kill.md)
