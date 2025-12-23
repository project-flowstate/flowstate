//! Flowstate Simulation Core
//!
//! This crate contains the deterministic, fixed-timestep game simulation.
//! It is the authoritative source of truth for all game-outcome-affecting state.
//!
//! # Architecture Constraints (INV-0004, KC-0001)
//!
//! The Simulation Core MUST NOT:
//! - Perform I/O operations (file, network, etc.)
//! - Read wall-clock time
//! - Use ambient/unseeded randomness
//! - Make system calls
//! - Depend on frame rate or variable delta time
//!
//! All external communication occurs through explicit, serializable message
//! boundaries owned by the Server Edge (DM-0011).
//!
//! # References
//!
//! - INV-0001: Deterministic Simulation
//! - INV-0002: Fixed Timestep
//! - INV-0004: Simulation Core Isolation
//! - INV-0007: Deterministic Ordering & Canonicalization
//! - DM-0014: Simulation Core
//! - ADR-0007: StateDigest Algorithm

#![deny(unsafe_code)]

// ============================================================================
// Type Aliases (Ref: DM-0001, DM-0019, DM-0020)
// ============================================================================

/// A single discrete simulation timestep; the atomic unit of game time.
/// Ref: DM-0001
pub type Tick = u64;

/// Per-Match participant identifier used for deterministic ordering.
/// Ref: DM-0019
///
/// NORMATIVE CONSTRAINT: Simulation Core MUST NOT assume PlayerIds are
/// contiguous, zero-based, or start at specific literal values (e.g., {0,1}).
/// PlayerId is used only as a stable indexing/ordering key.
pub type PlayerId = u8;

/// Unique identifier for an Entity within a Match.
/// Ref: DM-0020
pub type EntityId = u64;

// ============================================================================
// Core Types
// ============================================================================

/// Simulation-plane input consumed by advance().
/// Ref: DM-0027
///
/// `player_id` is an association key used to match intent to player's entity;
/// Server Edge owns identity binding (INV-0003).
///
/// StepInput values passed to advance() MUST be sorted by player_id ascending
/// for deterministic iteration (INV-0007).
#[derive(Debug, Clone, PartialEq)]
pub struct StepInput {
    pub player_id: PlayerId,
    /// Movement direction, magnitude <= 1.0
    pub move_dir: [f64; 2],
}

/// Snapshot of a single entity's state.
/// Used in both Baseline and Snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct EntitySnapshot {
    pub entity_id: EntityId,
    pub position: [f64; 2],
    pub velocity: [f64; 2],
}

/// Pre-step world state at tick T.
/// Ref: DM-0016
///
/// Digest computed via World::state_digest() per ADR-0007.
/// entities MUST be sorted by entity_id ascending (INV-0007).
#[derive(Debug, Clone, PartialEq)]
pub struct Baseline {
    pub tick: Tick,
    pub entities: Vec<EntitySnapshot>,
    pub digest: u64,
}

/// Post-step world state at tick T+1.
/// Ref: DM-0007
///
/// After `world.advance(T, inputs)`, returned Snapshot has `snapshot.tick = T+1`.
/// Digest computed via World::state_digest() per ADR-0007.
/// entities MUST be sorted by entity_id ascending (INV-0007).
#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot {
    pub tick: Tick,
    pub entities: Vec<EntitySnapshot>,
    pub digest: u64,
}

// ============================================================================
// v0 Movement Model Constants (Normative)
// ============================================================================

/// Movement speed in units per second.
/// NORMATIVE: This value MUST be recorded in ReplayArtifact tuning_parameters
/// with key "move_speed" per INV-0006.
pub const MOVE_SPEED: f64 = 5.0;

// ============================================================================
// StateDigest Implementation (ADR-0007)
// ============================================================================

/// StateDigest algorithm identifier for v0.
/// Ref: ADR-0007
pub const STATE_DIGEST_ALGO_ID: &str = "statedigest-v0-fnv1a64-le-f64canon-eidasc-posvel";

/// FNV-1a 64-bit offset basis.
const FNV1A_OFFSET_BASIS: u64 = 0xcbf29ce484222325;

/// FNV-1a 64-bit prime.
const FNV1A_PRIME: u64 = 0x100000001b3;

/// FNV-1a 64-bit hasher for StateDigest computation.
/// Ref: ADR-0007
#[derive(Debug, Clone)]
struct Fnv1a64 {
    state: u64,
}

impl Fnv1a64 {
    fn new() -> Self {
        Self {
            state: FNV1A_OFFSET_BASIS,
        }
    }

    fn update(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.state ^= u64::from(byte);
            self.state = self.state.wrapping_mul(FNV1A_PRIME);
        }
    }

    fn finish(self) -> u64 {
        self.state
    }
}

/// Canonicalize an f64 value for deterministic hashing.
/// Ref: ADR-0007
///
/// Rules:
/// - `-0.0` → `+0.0`
/// - Any NaN → quiet NaN bit pattern `0x7ff8000000000000`
fn canonicalize_f64(value: f64) -> u64 {
    const QUIET_NAN_BITS: u64 = 0x7ff8000000000000;

    if value.is_nan() {
        QUIET_NAN_BITS
    } else if value == 0.0 {
        // Both +0.0 and -0.0 compare equal to 0.0
        // Canonicalize to +0.0 bit pattern
        0u64
    } else {
        value.to_bits()
    }
}

// ============================================================================
// Internal Entity Types
// ============================================================================

/// Internal representation of a Character entity.
/// Ref: DM-0003, DM-0005
#[derive(Debug, Clone)]
struct Character {
    entity_id: EntityId,
    player_id: PlayerId,
    position: [f64; 2],
    velocity: [f64; 2],
}

impl Character {
    fn new(entity_id: EntityId, player_id: PlayerId) -> Self {
        Self {
            entity_id,
            player_id,
            position: [0.0, 0.0],
            velocity: [0.0, 0.0],
        }
    }

    fn to_snapshot(&self) -> EntitySnapshot {
        EntitySnapshot {
            entity_id: self.entity_id,
            position: self.position,
            velocity: self.velocity,
        }
    }
}

// ============================================================================
// World Implementation (DM-0002)
// ============================================================================

/// The authoritative simulation state container.
/// Ref: DM-0002
///
/// Contains entities and advances simulation state each Tick.
/// The Simulation Core maintains World state and advances it via `advance()`.
#[derive(Debug, Clone)]
pub struct World {
    /// Current simulation tick
    tick: Tick,
    /// Configured tick rate (Hz)
    tick_rate_hz: u32,
    /// Computed delta time per tick (seconds)
    dt_seconds: f64,
    /// Characters indexed by player_id
    /// Note: We use a Vec and search by player_id to maintain deterministic ordering
    characters: Vec<Character>,
    /// Next entity ID to assign (deterministic allocation)
    next_entity_id: EntityId,
    /// RNG seed (recorded for replay, not currently used in v0 movement)
    #[allow(dead_code)]
    seed: u64,
}

impl World {
    /// Create a new World.
    /// Ref: DM-0002
    ///
    /// v0 NORMATIVE: World::new() creates World at tick 0.
    ///
    /// # Arguments
    /// * `seed` - RNG seed (recorded for replay)
    /// * `tick_rate_hz` - Simulation tick rate in Hz
    pub fn new(seed: u64, tick_rate_hz: u32) -> Self {
        assert!(tick_rate_hz > 0, "tick_rate_hz must be positive");

        Self {
            tick: 0,
            tick_rate_hz,
            dt_seconds: 1.0 / f64::from(tick_rate_hz),
            characters: Vec::new(),
            next_entity_id: 1, // Start at 1 (0 could be reserved)
            seed,
        }
    }

    /// Spawn a character for the given player.
    /// Returns the EntityId of the spawned character.
    /// Ref: DM-0003, DM-0020
    ///
    /// EntityId assignment is deterministic based on spawn order.
    pub fn spawn_character(&mut self, player_id: PlayerId) -> EntityId {
        let entity_id = self.next_entity_id;
        self.next_entity_id += 1;

        let character = Character::new(entity_id, player_id);
        self.characters.push(character);

        // Maintain sorted order by entity_id for deterministic iteration (INV-0007)
        self.characters.sort_by_key(|c| c.entity_id);

        entity_id
    }

    /// Get the current simulation tick.
    /// Ref: DM-0001
    pub fn tick(&self) -> Tick {
        self.tick
    }

    /// Get the configured tick rate in Hz.
    pub fn tick_rate_hz(&self) -> u32 {
        self.tick_rate_hz
    }

    /// Get the pre-step world state (Baseline) at the current tick.
    /// Ref: DM-0016
    ///
    /// Postcondition: baseline().tick == world.tick()
    pub fn baseline(&self) -> Baseline {
        let entities = self.sorted_entity_snapshots();
        let digest = self.state_digest();

        Baseline {
            tick: self.tick,
            entities,
            digest,
        }
    }

    /// Advance simulation from tick T to T+1.
    /// Ref: DM-0007, INV-0002, ADR-0003
    ///
    /// # Arguments
    /// * `tick` - The pre-step tick (MUST equal self.tick())
    /// * `step_inputs` - Inputs sorted by player_id ascending (INV-0007)
    ///
    /// # Returns
    /// Snapshot with snapshot.tick = tick + 1 (post-step tick)
    ///
    /// # Panics
    /// If `tick != self.tick()` (precondition violation)
    pub fn advance(&mut self, tick: Tick, step_inputs: &[StepInput]) -> Snapshot {
        // Precondition: tick MUST == self.tick() (ADR-0003)
        assert_eq!(
            tick, self.tick,
            "advance() tick mismatch: expected {}, got {}",
            self.tick, tick
        );

        // Debug assert: inputs must be sorted by player_id (INV-0007)
        debug_assert!(
            step_inputs
                .windows(2)
                .all(|w| w[0].player_id <= w[1].player_id),
            "step_inputs must be sorted by player_id ascending"
        );

        // Apply movement physics for each input
        for input in step_inputs {
            self.apply_movement(input);
        }

        // Advance tick
        self.tick += 1;

        // Build and return snapshot
        let entities = self.sorted_entity_snapshots();
        let digest = self.state_digest();

        Snapshot {
            tick: self.tick,
            entities,
            digest,
        }
    }

    /// Compute the StateDigest for the current world state.
    /// Ref: ADR-0007
    ///
    /// Algorithm: FNV-1a 64-bit with canonicalization
    /// - `-0.0` → `+0.0`
    /// - NaN → quiet NaN `0x7ff8000000000000`
    /// - Entities iterated by EntityId ascending
    pub fn state_digest(&self) -> u64 {
        let mut hasher = Fnv1a64::new();

        // Hash tick (u64, little-endian)
        hasher.update(&self.tick.to_le_bytes());

        // Hash entities in EntityId ascending order (INV-0007)
        // Characters are maintained sorted by entity_id
        for character in &self.characters {
            // entity_id (u64, little-endian)
            hasher.update(&character.entity_id.to_le_bytes());

            // position[0] (f64, canonicalized, little-endian)
            hasher.update(&canonicalize_f64(character.position[0]).to_le_bytes());
            // position[1] (f64, canonicalized, little-endian)
            hasher.update(&canonicalize_f64(character.position[1]).to_le_bytes());

            // velocity[0] (f64, canonicalized, little-endian)
            hasher.update(&canonicalize_f64(character.velocity[0]).to_le_bytes());
            // velocity[1] (f64, canonicalized, little-endian)
            hasher.update(&canonicalize_f64(character.velocity[1]).to_le_bytes());
        }

        hasher.finish()
    }

    // ========================================================================
    // Internal Methods
    // ========================================================================

    /// Apply movement physics for a single input.
    /// Ref: v0 Movement Model in spec
    fn apply_movement(&mut self, input: &StepInput) {
        // Find character by player_id
        let Some(character) = self
            .characters
            .iter_mut()
            .find(|c| c.player_id == input.player_id)
        else {
            // No character for this player_id; skip (defensive)
            return;
        };

        // Clamp move_dir magnitude to 1.0 (defense-in-depth; validation is Server Edge)
        let move_dir = clamp_magnitude(input.move_dir, 1.0);

        // v0 Movement Model:
        // velocity = move_dir * MOVE_SPEED
        // position += velocity * dt
        character.velocity[0] = move_dir[0] * MOVE_SPEED;
        character.velocity[1] = move_dir[1] * MOVE_SPEED;

        character.position[0] += character.velocity[0] * self.dt_seconds;
        character.position[1] += character.velocity[1] * self.dt_seconds;
    }

    /// Get sorted entity snapshots.
    /// Entities are sorted by entity_id ascending (INV-0007).
    fn sorted_entity_snapshots(&self) -> Vec<EntitySnapshot> {
        // Characters are already maintained sorted by entity_id
        self.characters.iter().map(Character::to_snapshot).collect()
    }
}

/// Clamp a 2D vector's magnitude to a maximum value.
fn clamp_magnitude(v: [f64; 2], max_magnitude: f64) -> [f64; 2] {
    let magnitude_sq = v[0] * v[0] + v[1] * v[1];
    let max_sq = max_magnitude * max_magnitude;
    if magnitude_sq <= max_sq {
        v
    } else {
        let magnitude = magnitude_sq.sqrt();
        let scale = max_magnitude / magnitude;
        [v[0] * scale, v[1] * scale]
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Tier 0 Gate: T0.4 — WASD produces deterministic movement
    // ========================================================================

    /// T0.4: WASD produces movement with exact f64 equality.
    /// Ref: INV-0001, INV-0002
    #[test]
    fn test_t0_04_wasd_deterministic_movement() {
        const TICK_RATE_HZ: u32 = 60;
        const SEED: u64 = 0;
        const NUM_TICKS: u64 = 10;

        let mut world = World::new(SEED, TICK_RATE_HZ);
        let player_id: PlayerId = 0;
        world.spawn_character(player_id);

        // Move right (x+) for NUM_TICKS ticks
        let move_dir = [1.0, 0.0];
        let input = StepInput {
            player_id,
            move_dir,
        };

        for tick in 0..NUM_TICKS {
            let _ = world.advance(tick, std::slice::from_ref(&input));
        }

        // Expected position:
        // velocity = move_dir * MOVE_SPEED = [5.0, 0.0]
        // position += velocity * dt per tick
        // dt = 1/60
        // After 10 ticks: x = 10 * 5.0 * (1/60) = 50/60 = 5/6
        let dt = 1.0 / f64::from(TICK_RATE_HZ);
        let expected_x = f64::from(NUM_TICKS as u32) * MOVE_SPEED * dt;
        let expected_y = 0.0;

        let snapshot = world.baseline();
        assert_eq!(snapshot.entities.len(), 1);
        let entity = &snapshot.entities[0];

        // Exact f64 equality (no epsilon tolerance - determinism requirement)
        assert_eq!(
            entity.position[0], expected_x,
            "Position X mismatch: got {}, expected {}",
            entity.position[0], expected_x
        );
        assert_eq!(
            entity.position[1], expected_y,
            "Position Y mismatch: got {}, expected {}",
            entity.position[1], expected_y
        );
    }

    /// T0.4: Multiple runs produce identical results (determinism).
    #[test]
    fn test_t0_04_determinism_multiple_runs() {
        const TICK_RATE_HZ: u32 = 60;
        const SEED: u64 = 42;
        const NUM_TICKS: u64 = 100;

        fn run_simulation() -> (Vec<EntitySnapshot>, u64) {
            let mut world = World::new(SEED, TICK_RATE_HZ);
            world.spawn_character(0);
            world.spawn_character(1);

            let inputs = vec![
                StepInput {
                    player_id: 0,
                    move_dir: [1.0, 0.0],
                },
                StepInput {
                    player_id: 1,
                    move_dir: [0.0, 1.0],
                },
            ];

            for tick in 0..NUM_TICKS {
                let _ = world.advance(tick, &inputs);
            }

            let baseline = world.baseline();
            (baseline.entities, baseline.digest)
        }

        // Run twice
        let (entities1, digest1) = run_simulation();
        let (entities2, digest2) = run_simulation();

        // Must be identical
        assert_eq!(entities1, entities2, "Entity snapshots differ between runs");
        assert_eq!(digest1, digest2, "State digests differ between runs");
    }

    // ========================================================================
    // Tier 0 Gate: T0.17 — PlayerId Non-assumption
    // ========================================================================

    /// T0.17: Simulation Core works with non-contiguous PlayerIds.
    /// Ref: DM-0019
    #[test]
    fn test_t0_17_playerid_non_assumption() {
        const TICK_RATE_HZ: u32 = 60;
        const SEED: u64 = 0;

        let mut world = World::new(SEED, TICK_RATE_HZ);

        // Use non-contiguous, non-zero-based PlayerIds as per spec
        let player_a: PlayerId = 17;
        let player_b: PlayerId = 99;

        let entity_a = world.spawn_character(player_a);
        let entity_b = world.spawn_character(player_b);

        // Verify entities were created
        assert!(entity_a > 0);
        assert!(entity_b > 0);
        assert_ne!(entity_a, entity_b);

        // Inputs must be sorted by player_id
        let inputs = vec![
            StepInput {
                player_id: player_a, // 17
                move_dir: [1.0, 0.0],
            },
            StepInput {
                player_id: player_b, // 99
                move_dir: [0.0, 1.0],
            },
        ];

        // Advance simulation
        let snapshot = world.advance(0, &inputs);
        assert_eq!(snapshot.tick, 1);
        assert_eq!(snapshot.entities.len(), 2);

        // Verify both characters moved correctly
        let dt = 1.0 / f64::from(TICK_RATE_HZ);
        let expected_movement = MOVE_SPEED * dt;

        // Find entity A (player 17 moves right)
        let entity_a_snapshot = snapshot
            .entities
            .iter()
            .find(|e| e.entity_id == entity_a)
            .unwrap();
        assert_eq!(entity_a_snapshot.position[0], expected_movement);
        assert_eq!(entity_a_snapshot.position[1], 0.0);

        // Find entity B (player 99 moves up)
        let entity_b_snapshot = snapshot
            .entities
            .iter()
            .find(|e| e.entity_id == entity_b)
            .unwrap();
        assert_eq!(entity_b_snapshot.position[0], 0.0);
        assert_eq!(entity_b_snapshot.position[1], expected_movement);
    }

    // ========================================================================
    // StateDigest Tests (ADR-0007)
    // ========================================================================

    #[test]
    fn test_state_digest_deterministic() {
        let mut world1 = World::new(0, 60);
        let mut world2 = World::new(0, 60);

        world1.spawn_character(0);
        world2.spawn_character(0);

        assert_eq!(world1.state_digest(), world2.state_digest());

        let input = StepInput {
            player_id: 0,
            move_dir: [1.0, 0.0],
        };

        world1.advance(0, std::slice::from_ref(&input));
        world2.advance(0, std::slice::from_ref(&input));

        assert_eq!(world1.state_digest(), world2.state_digest());
    }

    #[test]
    fn test_state_digest_changes_with_state() {
        let mut world = World::new(0, 60);
        world.spawn_character(0);

        let digest_before = world.state_digest();

        let input = StepInput {
            player_id: 0,
            move_dir: [1.0, 0.0],
        };
        world.advance(0, &[input]);

        let digest_after = world.state_digest();

        assert_ne!(
            digest_before, digest_after,
            "Digest should change after state change"
        );
    }

    #[test]
    fn test_f64_canonicalization() {
        // Test -0.0 canonicalization
        assert_eq!(canonicalize_f64(-0.0), canonicalize_f64(0.0));
        assert_eq!(canonicalize_f64(-0.0), 0u64);

        // Test NaN canonicalization
        let nan1 = f64::NAN;
        let nan2 = f64::from_bits(0x7ff0000000000001); // Another NaN
        assert_eq!(canonicalize_f64(nan1), canonicalize_f64(nan2));
        assert_eq!(canonicalize_f64(nan1), 0x7ff8000000000000);

        // Test normal values are unchanged
        assert_eq!(canonicalize_f64(1.0), 1.0f64.to_bits());
        assert_eq!(canonicalize_f64(-1.0), (-1.0f64).to_bits());
    }

    // ========================================================================
    // World API Tests
    // ========================================================================

    #[test]
    fn test_world_new_starts_at_tick_zero() {
        let world = World::new(0, 60);
        assert_eq!(world.tick(), 0, "World should start at tick 0");
    }

    #[test]
    fn test_world_tick_rate() {
        let world = World::new(0, 60);
        assert_eq!(world.tick_rate_hz(), 60);

        let world2 = World::new(0, 30);
        assert_eq!(world2.tick_rate_hz(), 30);
    }

    #[test]
    fn test_spawn_character_returns_unique_ids() {
        let mut world = World::new(0, 60);

        let id1 = world.spawn_character(0);
        let id2 = world.spawn_character(1);
        let id3 = world.spawn_character(2);

        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_baseline_matches_tick() {
        let world = World::new(0, 60);
        let baseline = world.baseline();
        assert_eq!(baseline.tick, world.tick());
    }

    #[test]
    fn test_advance_increments_tick() {
        let mut world = World::new(0, 60);
        world.spawn_character(0);

        assert_eq!(world.tick(), 0);

        let snapshot = world.advance(0, &[]);
        assert_eq!(world.tick(), 1);
        assert_eq!(snapshot.tick, 1);

        let snapshot2 = world.advance(1, &[]);
        assert_eq!(world.tick(), 2);
        assert_eq!(snapshot2.tick, 2);
    }

    #[test]
    #[should_panic(expected = "advance() tick mismatch")]
    fn test_advance_panics_on_tick_mismatch() {
        let mut world = World::new(0, 60);
        world.spawn_character(0);

        // Try to advance with wrong tick
        world.advance(5, &[]);
    }

    #[test]
    fn test_entities_sorted_by_entity_id() {
        let mut world = World::new(0, 60);

        // Spawn in reverse order of what entity IDs will be
        world.spawn_character(99);
        world.spawn_character(50);
        world.spawn_character(1);

        let baseline = world.baseline();

        // Entities should be sorted by entity_id, not player_id
        for i in 1..baseline.entities.len() {
            assert!(
                baseline.entities[i - 1].entity_id < baseline.entities[i].entity_id,
                "Entities not sorted by entity_id"
            );
        }
    }

    #[test]
    fn test_movement_clamp_magnitude() {
        // Test that oversized move_dir is clamped
        let v = clamp_magnitude([2.0, 0.0], 1.0);
        assert!((v[0] - 1.0).abs() < 1e-10);
        assert!((v[1] - 0.0).abs() < 1e-10);

        // Test that normal magnitude is unchanged
        let v2 = clamp_magnitude([0.5, 0.5], 1.0);
        assert_eq!(v2, [0.5, 0.5]);

        // Test zero vector
        let v3 = clamp_magnitude([0.0, 0.0], 1.0);
        assert_eq!(v3, [0.0, 0.0]);
    }

    // ========================================================================
    // Tier 0 Gate: T0.5 — Simulation Core Isolation
    // ========================================================================

    /// T0.5: Verify advance() takes explicit tick parameter (ADR-0003).
    #[test]
    fn test_t0_05_advance_takes_explicit_tick() {
        let mut world = World::new(0, 60);
        world.spawn_character(0);

        // This test verifies the API signature matches the spec
        // advance() takes tick as first parameter
        let snapshot = world.advance(0, &[]);
        assert_eq!(snapshot.tick, 1);
    }

    // ========================================================================
    // Tier 0 Gate: T0.12 — LastKnownIntent Determinism
    // ========================================================================

    /// T0.12: Verify simulation is deterministic even with empty inputs.
    #[test]
    fn test_t0_12_empty_inputs_deterministic() {
        fn run_with_gaps() -> u64 {
            let mut world = World::new(0, 60);
            world.spawn_character(0);

            // Advance with no inputs (simulating LKI scenario)
            for tick in 0..10 {
                world.advance(tick, &[]);
            }

            world.state_digest()
        }

        let digest1 = run_with_gaps();
        let digest2 = run_with_gaps();

        assert_eq!(digest1, digest2);
    }
}
