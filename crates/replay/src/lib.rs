//! Flowstate Replay System
//!
//! This crate provides replay artifact generation and verification.
//!
//! # Architecture
//!
//! The replay system consists of:
//! - `ReplayRecorder`: Collects AppliedInputs during a match
//! - `ReplayVerifier`: Verifies replay artifacts produce identical outcomes
//! - Build fingerprint acquisition for same-build verification scope
//!
//! # References
//!
//! - INV-0006: Replay Verifiability
//! - DM-0017: ReplayArtifact
//! - DM-0024: AppliedInput
//! - ADR-0007: StateDigest Algorithm

#![deny(unsafe_code)]

use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

use flowstate_sim::{
    self, Baseline, MOVE_SPEED, PlayerId, STATE_DIGEST_ALGO_ID, StepInput, Tick, World,
};
use flowstate_wire::{
    AppliedInputProto, BuildFingerprint, EntitySnapshotProto, JoinBaseline, PlayerEntityMapping,
    ReplayArtifact, TuningParameter,
};
use prost::Message;
use sha2::{Digest, Sha256};

// ============================================================================
// Applied Input
// ============================================================================

/// Applied input representing post-normalization input.
/// Ref: DM-0024
///
/// This is the Server Edge's canonical "input truth" for a player at a tick.
#[derive(Debug, Clone, PartialEq)]
pub struct AppliedInput {
    pub tick: Tick,
    pub player_id: PlayerId,
    pub move_dir: [f64; 2],
    pub is_fallback: bool,
}

impl AppliedInput {
    /// Convert to StepInput for simulation.
    pub fn to_step_input(&self) -> StepInput {
        StepInput {
            player_id: self.player_id,
            move_dir: self.move_dir,
        }
    }
}

impl From<AppliedInput> for AppliedInputProto {
    fn from(input: AppliedInput) -> Self {
        Self {
            tick: input.tick,
            player_id: u32::from(input.player_id),
            move_dir: input.move_dir.to_vec(),
            is_fallback: input.is_fallback,
        }
    }
}

impl TryFrom<AppliedInputProto> for AppliedInput {
    type Error = &'static str;

    fn try_from(proto: AppliedInputProto) -> Result<Self, Self::Error> {
        if proto.move_dir.len() != 2 {
            return Err("move_dir must have exactly 2 elements");
        }
        Ok(Self {
            tick: proto.tick,
            player_id: proto.player_id as PlayerId,
            move_dir: [proto.move_dir[0], proto.move_dir[1]],
            is_fallback: proto.is_fallback,
        })
    }
}

// ============================================================================
// Replay Recorder
// ============================================================================

/// Configuration for replay recording.
#[derive(Debug, Clone)]
pub struct ReplayConfig {
    pub seed: u64,
    pub tick_rate_hz: u32,
    pub rng_algorithm: String,
    pub test_mode: bool,
    pub test_player_ids: Vec<PlayerId>,
}

impl Default for ReplayConfig {
    fn default() -> Self {
        Self {
            seed: 0,
            tick_rate_hz: 60,
            rng_algorithm: "none".to_string(), // v0 doesn't use RNG in movement
            test_mode: false,
            test_player_ids: Vec::new(),
        }
    }
}

/// Records match data for replay artifact generation.
/// Ref: DM-0017
pub struct ReplayRecorder {
    config: ReplayConfig,
    entity_spawn_order: Vec<PlayerId>,
    player_entity_mapping: Vec<(PlayerId, flowstate_sim::EntityId)>,
    initial_baseline: Option<Baseline>,
    inputs: Vec<AppliedInput>,
    build_fingerprint: Option<BuildFingerprintData>,
}

/// Build fingerprint data.
#[derive(Debug, Clone)]
pub struct BuildFingerprintData {
    pub binary_sha256: String,
    pub target_triple: String,
    pub profile: String,
    pub git_commit: String,
}

impl ReplayRecorder {
    /// Create a new replay recorder.
    pub fn new(config: ReplayConfig) -> Self {
        Self {
            config,
            entity_spawn_order: Vec::new(),
            player_entity_mapping: Vec::new(),
            initial_baseline: None,
            inputs: Vec::new(),
            build_fingerprint: None,
        }
    }

    /// Record entity spawn order.
    pub fn record_spawn(&mut self, player_id: PlayerId, entity_id: flowstate_sim::EntityId) {
        self.entity_spawn_order.push(player_id);
        self.player_entity_mapping.push((player_id, entity_id));
    }

    /// Record the initial baseline.
    pub fn record_baseline(&mut self, baseline: Baseline) {
        self.initial_baseline = Some(baseline);
    }

    /// Record an applied input.
    pub fn record_input(&mut self, input: AppliedInput) {
        self.inputs.push(input);
    }

    /// Set the build fingerprint.
    pub fn set_build_fingerprint(&mut self, fingerprint: BuildFingerprintData) {
        self.build_fingerprint = Some(fingerprint);
    }

    /// Finalize the replay artifact.
    pub fn finalize(
        self,
        final_digest: u64,
        checkpoint_tick: Tick,
        end_reason: &str,
    ) -> ReplayArtifact {
        let initial_baseline = self.initial_baseline.map(|b| JoinBaseline {
            tick: b.tick,
            entities: b
                .entities
                .into_iter()
                .map(|e| EntitySnapshotProto {
                    entity_id: e.entity_id,
                    position: e.position.to_vec(),
                    velocity: e.velocity.to_vec(),
                })
                .collect(),
            digest: b.digest,
        });

        let player_entity_mapping: Vec<_> = self
            .player_entity_mapping
            .iter()
            .map(|(pid, eid)| PlayerEntityMapping {
                player_id: u32::from(*pid),
                entity_id: *eid,
            })
            .collect();

        let tuning_parameters = vec![TuningParameter {
            key: "move_speed".to_string(),
            value: MOVE_SPEED,
        }];

        let build_fingerprint = self.build_fingerprint.map(|f| BuildFingerprint {
            binary_sha256: f.binary_sha256,
            target_triple: f.target_triple,
            profile: f.profile,
            git_commit: f.git_commit,
        });

        ReplayArtifact {
            replay_format_version: 1,
            initial_baseline,
            seed: self.config.seed,
            rng_algorithm: self.config.rng_algorithm,
            tick_rate_hz: self.config.tick_rate_hz,
            state_digest_algo_id: STATE_DIGEST_ALGO_ID.to_string(),
            entity_spawn_order: self
                .entity_spawn_order
                .iter()
                .map(|&p| u32::from(p))
                .collect(),
            player_entity_mapping,
            tuning_parameters,
            inputs: self.inputs.into_iter().map(Into::into).collect(),
            build_fingerprint,
            final_digest,
            checkpoint_tick,
            end_reason: end_reason.to_string(),
            test_mode: self.config.test_mode,
            test_player_ids: self
                .config
                .test_player_ids
                .iter()
                .map(|&p| u32::from(p))
                .collect(),
        }
    }
}

// ============================================================================
// Replay Verification
// ============================================================================

/// Replay verification error.
#[derive(Debug, Clone, PartialEq)]
pub enum VerifyError {
    /// Build fingerprint mismatch.
    BuildMismatch { expected: String, actual: String },
    /// Missing initial baseline.
    MissingBaseline,
    /// Initialization anchor (baseline digest) mismatch.
    InitializationAnchorMismatch { expected: u64, actual: u64 },
    /// Spawn reconstruction mismatch.
    SpawnReconstructionMismatch {
        player_id: PlayerId,
        expected_entity_id: flowstate_sim::EntityId,
        actual_entity_id: flowstate_sim::EntityId,
    },
    /// Input stream validation failed.
    InputStreamInvalid { reason: String },
    /// Final digest mismatch.
    FinalDigestMismatch { expected: u64, actual: u64 },
    /// Checkpoint tick mismatch.
    CheckpointTickMismatch { expected: Tick, actual: Tick },
    /// Invalid replay artifact format.
    InvalidFormat { reason: String },
}

impl std::fmt::Display for VerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BuildMismatch { expected, actual } => {
                write!(
                    f,
                    "Build fingerprint mismatch: expected {expected}, got {actual}"
                )
            }
            Self::MissingBaseline => write!(f, "Missing initial baseline in replay artifact"),
            Self::InitializationAnchorMismatch { expected, actual } => {
                write!(
                    f,
                    "Initialization anchor mismatch: expected {expected:#x}, got {actual:#x}"
                )
            }
            Self::SpawnReconstructionMismatch {
                player_id,
                expected_entity_id,
                actual_entity_id,
            } => {
                write!(
                    f,
                    "Spawn reconstruction mismatch for player {player_id}: expected entity {expected_entity_id}, got {actual_entity_id}"
                )
            }
            Self::InputStreamInvalid { reason } => {
                write!(f, "Input stream invalid: {reason}")
            }
            Self::FinalDigestMismatch { expected, actual } => {
                write!(
                    f,
                    "Final digest mismatch: expected {expected:#x}, got {actual:#x}"
                )
            }
            Self::CheckpointTickMismatch { expected, actual } => {
                write!(
                    f,
                    "Checkpoint tick mismatch: expected {expected}, got {actual}"
                )
            }
            Self::InvalidFormat { reason } => {
                write!(f, "Invalid replay format: {reason}")
            }
        }
    }
}

impl std::error::Error for VerifyError {}

/// Options for replay verification.
#[derive(Debug, Clone)]
pub struct VerifyOptions {
    /// Whether to strictly enforce build fingerprint matching.
    /// - true: fail on mismatch (CI/Tier-0)
    /// - false: warn but continue (dev mode)
    pub strict_build_check: bool,
    /// Current build fingerprint for comparison.
    pub current_build: Option<BuildFingerprintData>,
}

impl Default for VerifyOptions {
    fn default() -> Self {
        Self {
            strict_build_check: true,
            current_build: None,
        }
    }
}

/// Verify a replay artifact produces the recorded outcome.
/// Ref: INV-0006, T0.9
///
/// # Verification Steps (per spec):
/// 1. Verify build fingerprint matches (strict mode: fail; dev mode: warn)
/// 2. Validate AppliedInput stream integrity
/// 3. Initialize World with recorded seed and tick_rate_hz
/// 4. Reconstruct initialization (spawn order, verify entity IDs)
/// 5. Verify baseline digest (initialization anchor)
/// 6. Replay ticks [initial_baseline.tick, checkpoint_tick)
/// 7. Assert world.tick() == checkpoint_tick
/// 8. Assert world.state_digest() == final_digest
pub fn verify_replay(
    artifact: &ReplayArtifact,
    options: &VerifyOptions,
) -> Result<(), VerifyError> {
    // Step 1: Verify build fingerprint
    if let (Some(recorded), Some(current)) = (&artifact.build_fingerprint, &options.current_build) {
        let mismatch = recorded.binary_sha256 != current.binary_sha256
            || recorded.target_triple != current.target_triple
            || recorded.profile != current.profile;
        if mismatch && options.strict_build_check {
            return Err(VerifyError::BuildMismatch {
                expected: recorded.binary_sha256.clone(),
                actual: current.binary_sha256.clone(),
            });
        }
        // In non-strict mode, we'd log a warning here (not implemented for v0)
    }

    // Step 2: Validate input stream integrity
    validate_input_stream(artifact)?;

    // Get initial baseline
    let baseline_proto = artifact
        .initial_baseline
        .as_ref()
        .ok_or(VerifyError::MissingBaseline)?;

    let initial_tick = baseline_proto.tick;
    let checkpoint_tick = artifact.checkpoint_tick;

    // Step 3: Initialize World
    let mut world = World::new(artifact.seed, artifact.tick_rate_hz);

    // Step 4: Reconstruct initialization (spawn order)
    let player_entity_map: HashMap<u32, flowstate_sim::EntityId> = artifact
        .player_entity_mapping
        .iter()
        .map(|m| (m.player_id, m.entity_id))
        .collect();

    for &player_id_u32 in &artifact.entity_spawn_order {
        let player_id = player_id_u32 as PlayerId;
        let actual_entity_id = world.spawn_character(player_id);

        if let Some(&expected_entity_id) = player_entity_map.get(&player_id_u32)
            && actual_entity_id != expected_entity_id
        {
            return Err(VerifyError::SpawnReconstructionMismatch {
                player_id,
                expected_entity_id,
                actual_entity_id,
            });
        }
    }

    // Step 5: Verify initialization anchor (baseline digest)
    let baseline = world.baseline();
    if baseline.digest != baseline_proto.digest {
        return Err(VerifyError::InitializationAnchorMismatch {
            expected: baseline_proto.digest,
            actual: baseline.digest,
        });
    }

    // Convert inputs to lookup map: tick -> Vec<AppliedInput>
    let mut inputs_by_tick: HashMap<Tick, Vec<AppliedInput>> = HashMap::new();
    for input_proto in &artifact.inputs {
        let input: AppliedInput =
            input_proto
                .clone()
                .try_into()
                .map_err(|e: &str| VerifyError::InvalidFormat {
                    reason: e.to_string(),
                })?;
        inputs_by_tick.entry(input.tick).or_default().push(input);
    }

    // Step 6: Replay ticks [initial_tick, checkpoint_tick)
    for tick in initial_tick..checkpoint_tick {
        let mut step_inputs: Vec<StepInput> = inputs_by_tick
            .get(&tick)
            .map(|inputs| inputs.iter().map(AppliedInput::to_step_input).collect())
            .unwrap_or_default();

        // Sort by player_id (INV-0007) - defense in depth, verifier canonicalizes
        step_inputs.sort_by_key(|i| i.player_id);

        let _ = world.advance(tick, &step_inputs);
    }

    // Step 7: Verify checkpoint tick
    if world.tick() != checkpoint_tick {
        return Err(VerifyError::CheckpointTickMismatch {
            expected: checkpoint_tick,
            actual: world.tick(),
        });
    }

    // Step 8: Verify final digest
    let actual_digest = world.state_digest();
    if actual_digest != artifact.final_digest {
        return Err(VerifyError::FinalDigestMismatch {
            expected: artifact.final_digest,
            actual: actual_digest,
        });
    }

    Ok(())
}

/// Validate the input stream integrity.
/// Ref: INV-0006 AppliedInput stream validation
fn validate_input_stream(artifact: &ReplayArtifact) -> Result<(), VerifyError> {
    let baseline = artifact
        .initial_baseline
        .as_ref()
        .ok_or(VerifyError::MissingBaseline)?;

    let initial_tick = baseline.tick;
    let checkpoint_tick = artifact.checkpoint_tick;

    // Get player IDs from mapping
    let player_ids: Vec<u32> = artifact
        .player_entity_mapping
        .iter()
        .map(|m| m.player_id)
        .collect();

    // Build a set of (player_id, tick) pairs from inputs
    let mut input_pairs: HashMap<(u32, Tick), usize> = HashMap::new();
    for input in &artifact.inputs {
        let key = (input.player_id, input.tick);
        *input_pairs.entry(key).or_insert(0) += 1;
    }

    // Verify: for each player, for each tick in range, exactly one input
    for &player_id in &player_ids {
        for tick in initial_tick..checkpoint_tick {
            let key = (player_id, tick);
            match input_pairs.get(&key) {
                None => {
                    return Err(VerifyError::InputStreamInvalid {
                        reason: format!("Missing input for player {player_id} at tick {tick}"),
                    });
                }
                Some(&count) if count > 1 => {
                    return Err(VerifyError::InputStreamInvalid {
                        reason: format!("Duplicate input for player {player_id} at tick {tick}"),
                    });
                }
                Some(_) => {}
            }
        }
    }

    // Verify: no inputs outside the range
    for input in &artifact.inputs {
        if input.tick < initial_tick || input.tick >= checkpoint_tick {
            return Err(VerifyError::InputStreamInvalid {
                reason: format!(
                    "Input for player {} at tick {} is outside valid range [{}, {})",
                    input.player_id, input.tick, initial_tick, checkpoint_tick
                ),
            });
        }
        if !player_ids.contains(&input.player_id) {
            return Err(VerifyError::InputStreamInvalid {
                reason: format!("Input references unknown player_id {}", input.player_id),
            });
        }
    }

    Ok(())
}

// ============================================================================
// Build Fingerprint Acquisition
// ============================================================================

/// Acquire the current build fingerprint.
/// Ref: Spec "Build Fingerprint Acquisition"
///
/// # Returns
/// - `Ok(fingerprint)` on success
/// - `Err(io::Error)` if executable cannot be read
///
/// # Tier-0/CI Behavior
/// If this fails, Tier-0/CI MUST fail. Dev MAY proceed with "unknown".
pub fn acquire_build_fingerprint() -> io::Result<BuildFingerprintData> {
    // Get current executable path
    let exe_path = std::env::current_exe()?;

    // Read executable bytes and compute SHA-256
    let mut file = fs::File::open(&exe_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    let binary_sha256 = format!("{:x}", hasher.finalize());

    // Get target triple
    let target_triple = get_target_triple();

    // Get profile
    let profile = if cfg!(debug_assertions) {
        "dev"
    } else {
        "release"
    };

    // Get git commit (best effort)
    let git_commit = get_git_commit().unwrap_or_else(|| "unknown".to_string());

    Ok(BuildFingerprintData {
        binary_sha256,
        target_triple,
        profile: profile.to_string(),
        git_commit,
    })
}

/// Get the target triple for the current build.
fn get_target_triple() -> String {
    // Use compile-time constant
    #[cfg(target_os = "windows")]
    {
        #[cfg(target_arch = "x86_64")]
        return "x86_64-pc-windows-msvc".to_string();
        #[cfg(target_arch = "aarch64")]
        return "aarch64-pc-windows-msvc".to_string();
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        return "unknown-pc-windows-msvc".to_string();
    }
    #[cfg(target_os = "linux")]
    {
        #[cfg(target_arch = "x86_64")]
        return "x86_64-unknown-linux-gnu".to_string();
        #[cfg(target_arch = "aarch64")]
        return "aarch64-unknown-linux-gnu".to_string();
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        return "unknown-unknown-linux-gnu".to_string();
    }
    #[cfg(target_os = "macos")]
    {
        #[cfg(target_arch = "x86_64")]
        return "x86_64-apple-darwin".to_string();
        #[cfg(target_arch = "aarch64")]
        return "aarch64-apple-darwin".to_string();
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        return "unknown-apple-darwin".to_string();
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        "unknown-unknown-unknown".to_string()
    }
}

/// Get the git commit hash (best effort).
fn get_git_commit() -> Option<String> {
    // Try to read from environment (set by build script or CI)
    if let Ok(commit) = std::env::var("FLOWSTATE_GIT_COMMIT") {
        return Some(commit);
    }

    // Could shell out to git, but for v0 we just return None if not set
    None
}

// ============================================================================
// Replay I/O
// ============================================================================

/// Write a replay artifact to a file.
pub fn write_replay(artifact: &ReplayArtifact, path: &Path) -> io::Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Check for existing file (collision handling per spec)
    if path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("Replay artifact already exists at {}", path.display()),
        ));
    }

    // Encode and write
    let encoded = artifact.encode_to_vec();
    let mut file = fs::File::create(path)?;
    file.write_all(&encoded)?;

    Ok(())
}

/// Read a replay artifact from a file.
pub fn read_replay(path: &Path) -> io::Result<ReplayArtifact> {
    let data = fs::read(path)?;
    ReplayArtifact::decode(data.as_slice()).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to decode replay: {e}"),
        )
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_artifact() -> ReplayArtifact {
        let mut recorder = ReplayRecorder::new(ReplayConfig {
            seed: 42,
            tick_rate_hz: 60,
            rng_algorithm: "none".to_string(),
            test_mode: false,
            test_player_ids: Vec::new(),
        });

        // Create a world and record spawns
        let mut world = World::new(42, 60);
        let entity1 = world.spawn_character(0);
        let entity2 = world.spawn_character(1);
        recorder.record_spawn(0, entity1);
        recorder.record_spawn(1, entity2);

        // Record baseline
        recorder.record_baseline(world.baseline());

        // Record inputs for 10 ticks
        for tick in 0..10 {
            recorder.record_input(AppliedInput {
                tick,
                player_id: 0,
                move_dir: [1.0, 0.0],
                is_fallback: false,
            });
            recorder.record_input(AppliedInput {
                tick,
                player_id: 1,
                move_dir: [0.0, 1.0],
                is_fallback: false,
            });

            // Advance world
            let inputs = [
                StepInput {
                    player_id: 0,
                    move_dir: [1.0, 0.0],
                },
                StepInput {
                    player_id: 1,
                    move_dir: [0.0, 1.0],
                },
            ];
            world.advance(tick, &inputs);
        }

        // Finalize
        recorder.finalize(world.state_digest(), world.tick(), "complete")
    }

    /// T0.8: Replay artifact generated with all required fields.
    #[test]
    fn test_t0_08_replay_artifact_has_required_fields() {
        let artifact = create_test_artifact();

        assert_eq!(artifact.replay_format_version, 1);
        assert!(artifact.initial_baseline.is_some());
        assert_eq!(artifact.seed, 42);
        assert!(!artifact.rng_algorithm.is_empty());
        assert_eq!(artifact.tick_rate_hz, 60);
        assert_eq!(
            artifact.state_digest_algo_id,
            "statedigest-v0-fnv1a64-le-f64canon-eidasc-posvel"
        );
        assert_eq!(artifact.entity_spawn_order.len(), 2);
        assert_eq!(artifact.player_entity_mapping.len(), 2);
        assert!(!artifact.tuning_parameters.is_empty());
        assert_eq!(artifact.inputs.len(), 20); // 10 ticks * 2 players
        assert_eq!(artifact.checkpoint_tick, 10);
        assert_eq!(artifact.end_reason, "complete");
    }

    /// T0.9: Replay verification passes.
    #[test]
    fn test_t0_09_replay_verification_passes() {
        let artifact = create_test_artifact();
        let options = VerifyOptions {
            strict_build_check: false, // Don't check build in unit tests
            current_build: None,
        };

        let result = verify_replay(&artifact, &options);
        assert!(result.is_ok(), "Replay verification failed: {result:?}");
    }

    /// T0.10: Initialization anchor failure.
    #[test]
    fn test_t0_10_initialization_anchor_failure() {
        let mut artifact = create_test_artifact();

        // Mutate the baseline digest
        if let Some(ref mut baseline) = artifact.initial_baseline {
            baseline.digest ^= 0xDEADBEEF;
        }

        let options = VerifyOptions {
            strict_build_check: false,
            current_build: None,
        };

        let result = verify_replay(&artifact, &options);
        assert!(matches!(
            result,
            Err(VerifyError::InitializationAnchorMismatch { .. })
        ));
    }

    /// T0.12: LastKnownIntent determinism.
    #[test]
    fn test_t0_12_lki_determinism() {
        let mut recorder = ReplayRecorder::new(ReplayConfig::default());

        let mut world = World::new(0, 60);
        let entity1 = world.spawn_character(0);
        recorder.record_spawn(0, entity1);
        recorder.record_baseline(world.baseline());

        // Record inputs with some fallbacks
        for tick in 0..10 {
            let is_fallback = tick % 3 == 0; // Every 3rd tick is LKI
            recorder.record_input(AppliedInput {
                tick,
                player_id: 0,
                move_dir: if is_fallback { [0.0, 0.0] } else { [1.0, 0.0] },
                is_fallback,
            });

            let inputs = [StepInput {
                player_id: 0,
                move_dir: if is_fallback { [0.0, 0.0] } else { [1.0, 0.0] },
            }];
            world.advance(tick, &inputs);
        }

        let artifact = recorder.finalize(world.state_digest(), world.tick(), "complete");

        // Verify replay
        let options = VerifyOptions {
            strict_build_check: false,
            current_build: None,
        };
        let result = verify_replay(&artifact, &options);
        assert!(result.is_ok(), "Replay with LKI inputs failed: {result:?}");
    }

    /// T0.12a: Non-canonical AppliedInput storage order.
    #[test]
    fn test_t0_12a_noncanonical_input_order() {
        let mut recorder = ReplayRecorder::new(ReplayConfig::default());

        let mut world = World::new(0, 60);
        let entity1 = world.spawn_character(0);
        let entity2 = world.spawn_character(1);
        recorder.record_spawn(0, entity1);
        recorder.record_spawn(1, entity2);
        recorder.record_baseline(world.baseline());

        // Intentionally record inputs in non-canonical order (player 1 before player 0)
        for tick in 0..5 {
            // Wrong order: player 1 first
            recorder.record_input(AppliedInput {
                tick,
                player_id: 1,
                move_dir: [0.0, 1.0],
                is_fallback: false,
            });
            recorder.record_input(AppliedInput {
                tick,
                player_id: 0,
                move_dir: [1.0, 0.0],
                is_fallback: false,
            });

            // Advance world with correct order
            let inputs = [
                StepInput {
                    player_id: 0,
                    move_dir: [1.0, 0.0],
                },
                StepInput {
                    player_id: 1,
                    move_dir: [0.0, 1.0],
                },
            ];
            world.advance(tick, &inputs);
        }

        let artifact = recorder.finalize(world.state_digest(), world.tick(), "complete");

        // Verifier should canonicalize and succeed
        let options = VerifyOptions {
            strict_build_check: false,
            current_build: None,
        };
        let result = verify_replay(&artifact, &options);
        assert!(
            result.is_ok(),
            "Verifier should handle non-canonical order: {result:?}"
        );
    }

    #[test]
    fn test_applied_input_conversion() {
        let input = AppliedInput {
            tick: 100,
            player_id: 5,
            move_dir: [0.5, -0.5],
            is_fallback: true,
        };

        let proto: AppliedInputProto = input.clone().into();
        let back: AppliedInput = proto.try_into().unwrap();

        assert_eq!(input, back);
    }

    #[test]
    fn test_input_stream_validation_missing() {
        let mut artifact = create_test_artifact();

        // Remove an input
        artifact
            .inputs
            .retain(|i| !(i.tick == 5 && i.player_id == 0));

        let options = VerifyOptions::default();
        let result = verify_replay(&artifact, &options);
        assert!(matches!(
            result,
            Err(VerifyError::InputStreamInvalid { .. })
        ));
    }

    #[test]
    fn test_input_stream_validation_duplicate() {
        let mut artifact = create_test_artifact();

        // Add a duplicate
        artifact.inputs.push(AppliedInputProto {
            tick: 5,
            player_id: 0,
            move_dir: vec![1.0, 0.0],
            is_fallback: false,
        });

        let options = VerifyOptions::default();
        let result = verify_replay(&artifact, &options);
        assert!(matches!(
            result,
            Err(VerifyError::InputStreamInvalid { .. })
        ));
    }
}
