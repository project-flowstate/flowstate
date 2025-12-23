//! Flowstate Wire Protocol Types
//!
//! This crate defines the shared Protobuf message types used for communication
//! between Game Client and Server Edge. Both client and server binaries MUST
//! depend on this crate (T0.19: Schema Identity CI Gate).
//!
//! # Message Categories
//!
//! - **Control Channel** (reliable + ordered): Handshake, lifecycle messages
//! - **Realtime Channel** (unreliable + sequenced): Inputs, snapshots
//!
//! # References
//!
//! - ADR-0005: v0 Networking Architecture
//! - ADR-0006: Input Tick Targeting & TargetTickFloor
//! - ADR-0007: StateDigest Algorithm
//! - DM-0006: InputCmd
//! - DM-0007: Snapshot
//! - DM-0016: Baseline
//! - DM-0017: ReplayArtifact

#![deny(unsafe_code)]

use prost::Message;

// ============================================================================
// Type Aliases (matching simulation crate)
// ============================================================================

/// Tick type alias for wire protocol.
pub type Tick = u64;

/// PlayerId type alias for wire protocol.
pub type PlayerId = u8;

/// EntityId type alias for wire protocol.
pub type EntityId = u64;

/// InputSeq type alias for wire protocol.
/// Ref: DM-0026
pub type InputSeq = u64;

// ============================================================================
// Control Channel Messages
// ============================================================================

/// Client initiates handshake.
/// Ref: ADR-0005 (Control Channel)
///
/// v0: No fields required (handshake initiation only).
/// Future versions MAY add fields (e.g., protocol version, client capabilities).
#[derive(Clone, PartialEq, Message)]
pub struct ClientHello {
    // Empty for v0
}

/// Server welcome response with session info and tick guidance.
/// Ref: ADR-0005, ADR-0006 (Control Channel)
#[derive(Clone, PartialEq, Message)]
pub struct ServerWelcome {
    /// Initial TargetTickFloor for client input targeting.
    /// Ref: DM-0025
    #[prost(uint64, tag = "1")]
    pub target_tick_floor: Tick,

    /// Server tick rate in Hz.
    #[prost(uint32, tag = "2")]
    pub tick_rate_hz: u32,

    /// Assigned PlayerId for this session.
    /// Ref: DM-0019
    #[prost(uint32, tag = "3")]
    pub player_id: u32, // Wire as u32 for protobuf compatibility

    /// EntityId of the Character this client controls.
    /// Ref: DM-0020
    #[prost(uint64, tag = "4")]
    pub controlled_entity_id: EntityId,
}

/// Initial baseline state sent to client after welcome.
/// Ref: DM-0016 (Control Channel)
#[derive(Clone, PartialEq, Message)]
pub struct JoinBaseline {
    /// Baseline tick.
    #[prost(uint64, tag = "1")]
    pub tick: Tick,

    /// Entity snapshots, ordered by entity_id ascending per INV-0007.
    #[prost(message, repeated, tag = "2")]
    pub entities: Vec<EntitySnapshotProto>,

    /// StateDigest at this tick (ADR-0007).
    #[prost(uint64, tag = "3")]
    pub digest: u64,
}

// ============================================================================
// Realtime Channel Messages
// ============================================================================

/// Client input command targeting a specific tick.
/// Ref: DM-0006, ADR-0006 (Realtime Channel)
///
/// Note: `player_id` is NOT included - bound by Server Edge from session.
#[derive(Clone, PartialEq, Message)]
pub struct InputCmdProto {
    /// Target tick for this input.
    /// MUST be >= TargetTickFloor.
    #[prost(uint64, tag = "1")]
    pub tick: Tick,

    /// Per-session sequence number for deterministic selection.
    /// Ref: DM-0026
    #[prost(uint64, tag = "2")]
    pub input_seq: InputSeq,

    /// Movement direction [x, y], magnitude <= 1.0.
    #[prost(double, repeated, tag = "3")]
    pub move_dir: Vec<f64>,
}

/// Server snapshot broadcast.
/// Ref: DM-0007, ADR-0006 (Realtime Channel)
#[derive(Clone, PartialEq, Message)]
pub struct SnapshotProto {
    /// Post-step tick.
    #[prost(uint64, tag = "1")]
    pub tick: Tick,

    /// Entity snapshots, ordered by entity_id ascending per INV-0007.
    #[prost(message, repeated, tag = "2")]
    pub entities: Vec<EntitySnapshotProto>,

    /// StateDigest at this tick (ADR-0007).
    #[prost(uint64, tag = "3")]
    pub digest: u64,

    /// TargetTickFloor for client input targeting.
    /// Ref: DM-0025, ADR-0006
    #[prost(uint64, tag = "4")]
    pub target_tick_floor: Tick,
}

/// Entity snapshot embedded in JoinBaseline/SnapshotProto.
#[derive(Clone, PartialEq, Message)]
pub struct EntitySnapshotProto {
    /// EntityId.
    /// Ref: DM-0020
    #[prost(uint64, tag = "1")]
    pub entity_id: EntityId,

    /// Position [x, y].
    #[prost(double, repeated, tag = "2")]
    pub position: Vec<f64>,

    /// Velocity [vx, vy].
    #[prost(double, repeated, tag = "3")]
    pub velocity: Vec<f64>,
}

// ============================================================================
// Time Sync Messages (Tier 1 - Stub for future)
// ============================================================================

/// Time synchronization ping from client.
/// Ref: Tier 1 (debug/telemetry only)
#[derive(Clone, PartialEq, Message)]
pub struct TimeSyncPing {
    /// Client-side timestamp (opaque to server).
    #[prost(uint64, tag = "1")]
    pub client_timestamp: u64,
}

/// Time synchronization pong from server.
/// Ref: Tier 1 (debug/telemetry only)
#[derive(Clone, PartialEq, Message)]
pub struct TimeSyncPong {
    /// Server's current tick at time of response.
    #[prost(uint64, tag = "1")]
    pub server_tick: Tick,

    /// Server-side timestamp.
    #[prost(uint64, tag = "2")]
    pub server_timestamp: u64,

    /// Echo of client's ping timestamp.
    #[prost(uint64, tag = "3")]
    pub ping_timestamp_echo: u64,
}

// ============================================================================
// Replay Artifact Types
// ============================================================================

/// Applied input recorded for replay.
/// Ref: DM-0024
#[derive(Clone, PartialEq, Message)]
pub struct AppliedInputProto {
    /// Tick at which this input was applied.
    #[prost(uint64, tag = "1")]
    pub tick: Tick,

    /// Player this input is for.
    #[prost(uint32, tag = "2")]
    pub player_id: u32,

    /// Normalized movement direction.
    #[prost(double, repeated, tag = "3")]
    pub move_dir: Vec<f64>,

    /// True if generated via LastKnownIntent fallback.
    /// Ref: DM-0023
    #[prost(bool, tag = "4")]
    pub is_fallback: bool,
}

/// Player to Entity mapping for replay initialization.
#[derive(Clone, PartialEq, Message)]
pub struct PlayerEntityMapping {
    #[prost(uint32, tag = "1")]
    pub player_id: u32,

    #[prost(uint64, tag = "2")]
    pub entity_id: EntityId,
}

/// Tuning parameter key-value pair.
#[derive(Clone, PartialEq, Message)]
pub struct TuningParameter {
    #[prost(string, tag = "1")]
    pub key: String,

    #[prost(double, tag = "2")]
    pub value: f64,
}

/// Build fingerprint for replay scope verification.
#[derive(Clone, PartialEq, Message)]
pub struct BuildFingerprint {
    /// SHA-256 of server executable bytes.
    #[prost(string, tag = "1")]
    pub binary_sha256: String,

    /// Target triple (e.g., "x86_64-pc-windows-msvc").
    #[prost(string, tag = "2")]
    pub target_triple: String,

    /// Build profile ("release" or "dev").
    #[prost(string, tag = "3")]
    pub profile: String,

    /// Git commit hash (metadata/traceability).
    #[prost(string, tag = "4")]
    pub git_commit: String,
}

/// Complete replay artifact.
/// Ref: DM-0017, INV-0006
#[derive(Clone, PartialEq, Message)]
pub struct ReplayArtifact {
    /// Schema version (v0 starts at 1).
    #[prost(uint32, tag = "1")]
    pub replay_format_version: u32,

    /// Initial baseline at match start.
    /// Ref: DM-0016
    #[prost(message, optional, tag = "2")]
    pub initial_baseline: Option<JoinBaseline>,

    /// RNG seed.
    #[prost(uint64, tag = "3")]
    pub seed: u64,

    /// RNG algorithm identifier (e.g., "ChaCha8Rng").
    #[prost(string, tag = "4")]
    pub rng_algorithm: String,

    /// Simulation tick rate.
    #[prost(uint32, tag = "5")]
    pub tick_rate_hz: u32,

    /// StateDigest algorithm identifier (ADR-0007).
    #[prost(string, tag = "6")]
    pub state_digest_algo_id: String,

    /// Entity spawn order (PlayerIds in spawn sequence).
    #[prost(uint32, repeated, tag = "7")]
    pub entity_spawn_order: Vec<u32>,

    /// Player to Entity mapping.
    #[prost(message, repeated, tag = "8")]
    pub player_entity_mapping: Vec<PlayerEntityMapping>,

    /// Tuning parameters (sorted by key).
    #[prost(message, repeated, tag = "9")]
    pub tuning_parameters: Vec<TuningParameter>,

    /// Applied input stream.
    /// Ref: DM-0024
    #[prost(message, repeated, tag = "10")]
    pub inputs: Vec<AppliedInputProto>,

    /// Build fingerprint for verification scope.
    #[prost(message, optional, tag = "11")]
    pub build_fingerprint: Option<BuildFingerprint>,

    /// StateDigest at checkpoint_tick.
    #[prost(uint64, tag = "12")]
    pub final_digest: u64,

    /// Post-step tick for verification anchor.
    #[prost(uint64, tag = "13")]
    pub checkpoint_tick: Tick,

    /// Match termination reason.
    #[prost(string, tag = "14")]
    pub end_reason: String,

    /// Test mode flag.
    #[prost(bool, tag = "15")]
    pub test_mode: bool,

    /// Test player IDs (when test_mode=true).
    #[prost(uint32, repeated, tag = "16")]
    pub test_player_ids: Vec<u32>,
}

// ============================================================================
// Conversion Traits
// ============================================================================

impl From<flowstate_sim::EntitySnapshot> for EntitySnapshotProto {
    fn from(e: flowstate_sim::EntitySnapshot) -> Self {
        Self {
            entity_id: e.entity_id,
            position: e.position.to_vec(),
            velocity: e.velocity.to_vec(),
        }
    }
}

impl TryFrom<EntitySnapshotProto> for flowstate_sim::EntitySnapshot {
    type Error = &'static str;

    fn try_from(e: EntitySnapshotProto) -> Result<Self, Self::Error> {
        if e.position.len() != 2 {
            return Err("position must have exactly 2 elements");
        }
        if e.velocity.len() != 2 {
            return Err("velocity must have exactly 2 elements");
        }
        Ok(Self {
            entity_id: e.entity_id,
            position: [e.position[0], e.position[1]],
            velocity: [e.velocity[0], e.velocity[1]],
        })
    }
}

impl From<flowstate_sim::Baseline> for JoinBaseline {
    fn from(b: flowstate_sim::Baseline) -> Self {
        Self {
            tick: b.tick,
            entities: b.entities.into_iter().map(Into::into).collect(),
            digest: b.digest,
        }
    }
}

impl TryFrom<JoinBaseline> for flowstate_sim::Baseline {
    type Error = &'static str;

    fn try_from(b: JoinBaseline) -> Result<Self, Self::Error> {
        let entities: Result<Vec<_>, _> = b.entities.into_iter().map(TryInto::try_into).collect();
        Ok(Self {
            tick: b.tick,
            entities: entities?,
            digest: b.digest,
        })
    }
}

impl From<flowstate_sim::Snapshot> for SnapshotProto {
    fn from(s: flowstate_sim::Snapshot) -> Self {
        Self {
            tick: s.tick,
            entities: s.entities.into_iter().map(Into::into).collect(),
            digest: s.digest,
            target_tick_floor: 0, // Must be set by caller
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_hello_roundtrip() {
        let msg = ClientHello {};
        let encoded = msg.encode_to_vec();
        let decoded = ClientHello::decode(encoded.as_slice()).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_server_welcome_roundtrip() {
        let msg = ServerWelcome {
            target_tick_floor: 5,
            tick_rate_hz: 60,
            player_id: 1,
            controlled_entity_id: 42,
        };
        let encoded = msg.encode_to_vec();
        let decoded = ServerWelcome::decode(encoded.as_slice()).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_input_cmd_roundtrip() {
        let msg = InputCmdProto {
            tick: 100,
            input_seq: 50,
            move_dir: vec![0.707, 0.707],
        };
        let encoded = msg.encode_to_vec();
        let decoded = InputCmdProto::decode(encoded.as_slice()).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_snapshot_roundtrip() {
        let msg = SnapshotProto {
            tick: 100,
            entities: vec![EntitySnapshotProto {
                entity_id: 1,
                position: vec![10.5, 20.5],
                velocity: vec![1.0, 0.0],
            }],
            digest: 0xdeadbeef,
            target_tick_floor: 101,
        };
        let encoded = msg.encode_to_vec();
        let decoded = SnapshotProto::decode(encoded.as_slice()).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_replay_artifact_roundtrip() {
        let msg = ReplayArtifact {
            replay_format_version: 1,
            initial_baseline: Some(JoinBaseline {
                tick: 0,
                entities: vec![],
                digest: 0,
            }),
            seed: 42,
            rng_algorithm: "ChaCha8Rng".to_string(),
            tick_rate_hz: 60,
            state_digest_algo_id: "statedigest-v0-fnv1a64-le-f64canon-eidasc-posvel".to_string(),
            entity_spawn_order: vec![0, 1],
            player_entity_mapping: vec![
                PlayerEntityMapping {
                    player_id: 0,
                    entity_id: 1,
                },
                PlayerEntityMapping {
                    player_id: 1,
                    entity_id: 2,
                },
            ],
            tuning_parameters: vec![TuningParameter {
                key: "move_speed".to_string(),
                value: 5.0,
            }],
            inputs: vec![],
            build_fingerprint: Some(BuildFingerprint {
                binary_sha256: "abc123".to_string(),
                target_triple: "x86_64-pc-windows-msvc".to_string(),
                profile: "release".to_string(),
                git_commit: "deadbeef".to_string(),
            }),
            final_digest: 0xfeedface,
            checkpoint_tick: 3600,
            end_reason: "complete".to_string(),
            test_mode: false,
            test_player_ids: vec![],
        };
        let encoded = msg.encode_to_vec();
        let decoded = ReplayArtifact::decode(encoded.as_slice()).unwrap();
        assert_eq!(msg, decoded);
    }

    /// T0.19: Verify this crate exists and can be depended upon.
    #[test]
    fn test_t0_19_wire_crate_exists() {
        // This test's existence in the shared crate proves the crate exists.
        // CI will verify both server and client depend on this crate.
        // The test body is empty - the existence of this test is the assertion.
    }
}
