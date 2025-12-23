//! Flowstate Server Edge
//!
//! The Server Edge mediates communication between Game Clients and the
//! Simulation Core. It owns:
//! - Session management (DM-0008)
//! - Input validation and buffering
//! - TargetTickFloor computation (ADR-0006)
//! - AppliedInput → StepInput conversion
//! - Replay recording
//!
//! # Architecture (INV-0003, INV-0004)
//!
//! The Server Edge performs all I/O on behalf of the Game Server Instance.
//! The Simulation Core is invoked only with StepInput and produces Snapshots.
//!
//! # References
//!
//! - INV-0003: Authoritative Simulation
//! - INV-0004: Simulation Core Isolation
//! - INV-0005: Tick-Indexed I/O Contract
//! - ADR-0005: v0 Networking Architecture
//! - ADR-0006: Input Tick Targeting
//! - DM-0011: Server Edge

#![deny(unsafe_code)]

pub mod input_buffer;
pub mod session;
pub mod validation;

use std::collections::HashMap;

use flowstate_replay::{AppliedInput, BuildFingerprintData, ReplayConfig, ReplayRecorder};
use flowstate_sim::{Baseline, PlayerId, Snapshot, StepInput, Tick, World};
use flowstate_wire::{InputCmdProto, JoinBaseline, ReplayArtifact, ServerWelcome, SnapshotProto};
use input_buffer::InputBuffer;
use session::{Session, SessionId};
use validation::{ValidationConfig, ValidationResult, validate_input};

// ============================================================================
// v0 Parameters (from docs/networking/v0-parameters.md)
// ============================================================================

/// v0 tick rate in Hz.
pub const TICK_RATE_HZ: u32 = 60;

/// Maximum ticks ahead a client can target.
pub const MAX_FUTURE_TICKS: u64 = 120;

/// TargetTickFloor lead.
pub const INPUT_LEAD_TICKS: u64 = 1;

/// Input rate limit per second.
pub const INPUT_RATE_LIMIT_PER_SEC: u32 = 120;

/// Match duration in ticks.
pub const MATCH_DURATION_TICKS: u64 = 3600;

/// Connection timeout in milliseconds.
pub const CONNECT_TIMEOUT_MS: u64 = 30000;

// ============================================================================
// Match End Reason
// ============================================================================

/// Reason for match termination.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndReason {
    Complete,
    Disconnect,
}

impl EndReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Disconnect => "disconnect",
        }
    }
}

// ============================================================================
// Server State
// ============================================================================

/// Server configuration.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub seed: u64,
    pub tick_rate_hz: u32,
    pub max_future_ticks: u64,
    pub input_lead_ticks: u64,
    pub input_rate_limit_per_sec: u32,
    pub match_duration_ticks: u64,
    pub connect_timeout_ms: u64,
    pub test_mode: bool,
    pub test_player_ids: Option<(PlayerId, PlayerId)>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            seed: 0,
            tick_rate_hz: TICK_RATE_HZ,
            max_future_ticks: MAX_FUTURE_TICKS,
            input_lead_ticks: INPUT_LEAD_TICKS,
            input_rate_limit_per_sec: INPUT_RATE_LIMIT_PER_SEC,
            match_duration_ticks: MATCH_DURATION_TICKS,
            connect_timeout_ms: CONNECT_TIMEOUT_MS,
            test_mode: false,
            test_player_ids: None,
        }
    }
}

/// Server state for running a match.
pub struct Server {
    config: ServerConfig,
    world: World,
    sessions: HashMap<SessionId, Session>,
    next_session_id: SessionId,
    /// PlayerId → SessionId mapping
    player_sessions: HashMap<PlayerId, SessionId>,
    /// SessionId → PlayerId mapping (for convenience)
    session_players: HashMap<SessionId, PlayerId>,
    /// Input buffer per (player_id, tick)
    input_buffer: InputBuffer,
    /// Last known intent per player
    last_known_intent: HashMap<PlayerId, [f64; 2]>,
    /// Last emitted target tick floor per session
    last_emitted_floor: HashMap<SessionId, Tick>,
    /// Replay recorder
    replay_recorder: ReplayRecorder,
    /// Entity spawn order (player_ids in order)
    entity_spawn_order: Vec<PlayerId>,
    /// Player → Entity mapping
    player_entity_mapping: HashMap<PlayerId, flowstate_sim::EntityId>,
    /// Initial tick (set after match starts)
    initial_tick: Tick,
    /// Match started flag
    match_started: bool,
    /// Build fingerprint
    build_fingerprint: Option<BuildFingerprintData>,
}

impl Server {
    /// Create a new server with the given configuration.
    pub fn new(config: ServerConfig) -> Self {
        let validation_config = ValidationConfig {
            max_future_ticks: config.max_future_ticks,
            input_rate_limit_per_sec: config.input_rate_limit_per_sec,
            tick_rate_hz: config.tick_rate_hz,
        };

        let replay_config = ReplayConfig {
            seed: config.seed,
            tick_rate_hz: config.tick_rate_hz,
            rng_algorithm: "none".to_string(),
            test_mode: config.test_mode,
            test_player_ids: config
                .test_player_ids
                .map(|(a, b)| vec![a, b])
                .unwrap_or_default(),
        };

        Self {
            world: World::new(config.seed, config.tick_rate_hz),
            sessions: HashMap::new(),
            next_session_id: 1,
            player_sessions: HashMap::new(),
            session_players: HashMap::new(),
            input_buffer: InputBuffer::new(validation_config),
            last_known_intent: HashMap::new(),
            last_emitted_floor: HashMap::new(),
            replay_recorder: ReplayRecorder::new(replay_config),
            entity_spawn_order: Vec::new(),
            player_entity_mapping: HashMap::new(),
            initial_tick: 0,
            match_started: false,
            build_fingerprint: None,
            config,
        }
    }

    /// Set the build fingerprint.
    pub fn set_build_fingerprint(&mut self, fingerprint: BuildFingerprintData) {
        self.build_fingerprint = Some(fingerprint.clone());
        self.replay_recorder.set_build_fingerprint(fingerprint);
    }

    /// Get current tick.
    pub fn current_tick(&self) -> Tick {
        self.world.tick()
    }

    /// Get number of connected sessions.
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Check if server is ready to start (enough sessions connected).
    /// Used for external timeout enforcement (T0.16).
    pub fn is_ready_to_start(&self) -> bool {
        self.sessions.len() >= 2
    }

    /// Accept a new session (client connected).
    /// Returns (session_id, assigned_player_id, controlled_entity_id).
    ///
    /// # Panics
    /// If more than 2 sessions try to connect (v0 limit).
    pub fn accept_session(&mut self) -> (SessionId, PlayerId, flowstate_sim::EntityId) {
        assert!(self.sessions.len() < 2, "v0: Only 2 sessions allowed");
        assert!(
            !self.match_started,
            "Cannot accept sessions after match start"
        );

        let session_id = self.next_session_id;
        self.next_session_id += 1;

        // Assign player ID
        let player_id = if let Some((id1, id2)) = self.config.test_player_ids {
            // Test mode: use configured IDs
            if self.sessions.is_empty() { id1 } else { id2 }
        } else {
            // Normal mode: 0 for first, 1 for second
            self.sessions.len() as PlayerId
        };

        // Spawn character
        let entity_id = self.world.spawn_character(player_id);

        // Create session
        let session = Session::new(session_id, player_id, entity_id);
        self.sessions.insert(session_id, session);
        self.player_sessions.insert(player_id, session_id);
        self.session_players.insert(session_id, player_id);

        // Record spawn order
        self.entity_spawn_order.push(player_id);
        self.player_entity_mapping.insert(player_id, entity_id);
        self.replay_recorder.record_spawn(player_id, entity_id);

        // Initialize last known intent
        self.last_known_intent.insert(player_id, [0.0, 0.0]);

        (session_id, player_id, entity_id)
    }

    /// Start the match (after 2 clients connected).
    /// Returns the initial baseline and ServerWelcome data for each session.
    pub fn start_match(&mut self) -> (Baseline, Vec<(SessionId, ServerWelcome)>) {
        assert_eq!(
            self.sessions.len(),
            2,
            "Need exactly 2 sessions to start match"
        );
        assert!(!self.match_started, "Match already started");

        self.match_started = true;
        self.initial_tick = self.world.tick();

        // Record baseline
        let baseline = self.world.baseline();
        self.replay_recorder.record_baseline(baseline.clone());

        // Compute initial target tick floor
        let target_tick_floor = self.initial_tick + self.config.input_lead_ticks;

        // Initialize floor state for all sessions
        for &session_id in self.sessions.keys() {
            self.last_emitted_floor
                .insert(session_id, target_tick_floor);
        }

        // Create ServerWelcome for each session
        let welcomes: Vec<_> = self
            .sessions
            .values()
            .map(|session| {
                let welcome = ServerWelcome {
                    target_tick_floor,
                    tick_rate_hz: self.config.tick_rate_hz,
                    player_id: u32::from(session.player_id),
                    controlled_entity_id: session.controlled_entity_id,
                };
                (session.id, welcome)
            })
            .collect();

        (baseline, welcomes)
    }

    /// Check if match should end.
    pub fn should_end_match(&self) -> Option<EndReason> {
        if !self.match_started {
            return None;
        }

        // Check duration
        if self.world.tick() >= self.initial_tick + self.config.match_duration_ticks {
            return Some(EndReason::Complete);
        }

        None
    }

    /// Handle session disconnect.
    pub fn disconnect_session(&mut self, session_id: SessionId) {
        if let Some(session) = self.sessions.remove(&session_id) {
            self.player_sessions.remove(&session.player_id);
            self.session_players.remove(&session_id);
        }
    }

    /// Check if any session has disconnected.
    pub fn has_disconnect(&self) -> bool {
        // In v0, we check if we started with 2 and now have fewer
        self.match_started && self.sessions.len() < 2
    }

    /// Receive and buffer an input from a client.
    /// Returns validation result.
    pub fn receive_input(
        &mut self,
        session_id: SessionId,
        input: InputCmdProto,
    ) -> ValidationResult {
        // Pre-Welcome input drop
        if !self.match_started {
            return ValidationResult::DroppedPreWelcome;
        }

        // Get player_id for this session
        let Some(&player_id) = self.session_players.get(&session_id) else {
            return ValidationResult::DroppedUnknownSession;
        };

        // Get last emitted floor for this session
        let floor = self
            .last_emitted_floor
            .get(&session_id)
            .copied()
            .unwrap_or(0);

        // Validate input
        validate_input(
            &input,
            self.world.tick(),
            floor,
            &mut self.input_buffer,
            player_id,
        )
    }

    /// Process a single tick.
    /// Returns (snapshot, target_tick_floor, serialized_snapshot_bytes).
    ///
    /// The serialized bytes are identical for all sessions (T0.18).
    pub fn step(&mut self) -> (Snapshot, Tick, Vec<u8>) {
        let current_tick = self.world.tick();

        // Produce AppliedInput per player
        let mut applied_inputs: Vec<AppliedInput> = Vec::new();

        for &player_id in self.entity_spawn_order.iter() {
            let (move_dir, is_fallback) = self
                .input_buffer
                .take_input(player_id, current_tick)
                .map(|cmd| {
                    // Validate and normalize move_dir
                    let move_dir = if cmd.move_dir.len() == 2 {
                        [cmd.move_dir[0], cmd.move_dir[1]]
                    } else {
                        [0.0, 0.0]
                    };
                    (move_dir, false)
                })
                .unwrap_or_else(|| {
                    // LastKnownIntent fallback
                    let lki = self
                        .last_known_intent
                        .get(&player_id)
                        .copied()
                        .unwrap_or([0.0, 0.0]);
                    (lki, true)
                });

            // Update last known intent
            self.last_known_intent.insert(player_id, move_dir);

            applied_inputs.push(AppliedInput {
                tick: current_tick,
                player_id,
                move_dir,
                is_fallback,
            });
        }

        // Record for replay
        for input in &applied_inputs {
            self.replay_recorder.record_input(input.clone());
        }

        // Convert to StepInput (sorted by player_id)
        let mut step_inputs: Vec<StepInput> = applied_inputs
            .iter()
            .map(AppliedInput::to_step_input)
            .collect();
        step_inputs.sort_by_key(|i| i.player_id);

        // Advance world
        let snapshot = self.world.advance(current_tick, &step_inputs);

        // Compute new target tick floor (post-step tick + lead)
        let target_tick_floor = self.world.tick() + self.config.input_lead_ticks;

        // Update floor for all sessions
        for session_id in self.sessions.keys() {
            self.last_emitted_floor
                .insert(*session_id, target_tick_floor);
        }

        // Evict old buffered inputs
        self.input_buffer.evict_before(self.world.tick());

        // Serialize snapshot (identical for all sessions - T0.18)
        let snapshot_proto = SnapshotProto {
            tick: snapshot.tick,
            entities: snapshot
                .entities
                .iter()
                .map(|e| flowstate_wire::EntitySnapshotProto {
                    entity_id: e.entity_id,
                    position: e.position.to_vec(),
                    velocity: e.velocity.to_vec(),
                })
                .collect(),
            digest: snapshot.digest,
            target_tick_floor,
        };
        let snapshot_bytes = prost::Message::encode_to_vec(&snapshot_proto);

        (snapshot, target_tick_floor, snapshot_bytes)
    }

    /// Finalize the match and produce a replay artifact.
    pub fn finalize(self, end_reason: EndReason) -> ReplayArtifact {
        let final_digest = self.world.state_digest();
        let checkpoint_tick = self.world.tick();

        self.replay_recorder
            .finalize(final_digest, checkpoint_tick, end_reason.as_str())
    }

    /// Get the baseline for JoinBaseline message.
    pub fn baseline_proto(&self) -> JoinBaseline {
        let baseline = self.world.baseline();
        baseline.into()
    }

    /// Get all connected session IDs.
    pub fn session_ids(&self) -> Vec<SessionId> {
        self.sessions.keys().copied().collect()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// T0.1: Two clients connect, complete handshake.
    #[test]
    fn test_t0_01_two_client_handshake() {
        let mut server = Server::new(ServerConfig::default());

        // Accept first session
        let (session1, player1, entity1) = server.accept_session();
        assert_eq!(player1, 0);
        assert!(entity1 > 0);
        assert_eq!(server.session_count(), 1);

        // Accept second session
        let (_session2, player2, entity2) = server.accept_session();
        assert_eq!(player2, 1);
        assert!(entity2 > 0);
        assert_ne!(entity1, entity2);
        assert_eq!(server.session_count(), 2);

        // Start match
        let (baseline, welcomes) = server.start_match();
        assert_eq!(baseline.tick, 0);
        assert_eq!(welcomes.len(), 2);

        // Verify ServerWelcome contents
        for (sid, welcome) in &welcomes {
            assert_eq!(welcome.target_tick_floor, INPUT_LEAD_TICKS);
            assert_eq!(welcome.tick_rate_hz, TICK_RATE_HZ);
            if *sid == session1 {
                assert_eq!(welcome.player_id, 0);
                assert_eq!(welcome.controlled_entity_id, entity1);
            } else {
                assert_eq!(welcome.player_id, 1);
                assert_eq!(welcome.controlled_entity_id, entity2);
            }
        }
    }

    /// T0.2: JoinBaseline delivers initial Baseline.
    #[test]
    fn test_t0_02_join_baseline() {
        let mut server = Server::new(ServerConfig::default());
        server.accept_session();
        server.accept_session();

        let (baseline, _) = server.start_match();

        // Baseline should have 2 entities at tick 0
        assert_eq!(baseline.tick, 0);
        assert_eq!(baseline.entities.len(), 2);
        assert!(baseline.digest != 0);
    }

    /// T0.5a: Tick/floor relationship assertion.
    #[test]
    fn test_t0_05a_tick_floor_relationship() {
        let mut server = Server::new(ServerConfig::default());
        server.accept_session();
        server.accept_session();
        server.start_match();

        // Step once
        let (snapshot, floor, _) = server.step();

        // After advance(0, inputs), snapshot.tick should be 1
        assert_eq!(snapshot.tick, 1);
        // Floor should be post-step tick + lead = 1 + 1 = 2
        assert_eq!(floor, 1 + INPUT_LEAD_TICKS);

        // Step again
        let (snapshot2, floor2, _) = server.step();
        assert_eq!(snapshot2.tick, 2);
        assert_eq!(floor2, 2 + INPUT_LEAD_TICKS);
    }

    /// T0.14: Disconnect handling.
    #[test]
    fn test_t0_14_disconnect_handling() {
        let mut server = Server::new(ServerConfig::default());
        let (session1, _, _) = server.accept_session();
        server.accept_session();
        server.start_match();

        // Simulate disconnect
        server.disconnect_session(session1);

        assert!(server.has_disconnect());
        assert_eq!(server.session_count(), 1);
    }

    /// T0.15: Match termination.
    #[test]
    fn test_t0_15_match_termination() {
        let config = ServerConfig {
            match_duration_ticks: 10, // Short match for testing
            ..Default::default()
        };
        let mut server = Server::new(config);
        server.accept_session();
        server.accept_session();
        server.start_match();

        // Run until match should end
        for _ in 0..10 {
            assert!(server.should_end_match().is_none());
            server.step();
        }

        assert_eq!(server.should_end_match(), Some(EndReason::Complete));
    }

    /// T0.17: PlayerId non-assumption (test mode).
    #[test]
    fn test_t0_17_playerid_test_mode() {
        let config = ServerConfig {
            test_mode: true,
            test_player_ids: Some((17, 99)),
            match_duration_ticks: 10,
            ..Default::default()
        };
        let mut server = Server::new(config);

        let (_, player1, _) = server.accept_session();
        let (_, player2, _) = server.accept_session();

        assert_eq!(player1, 17);
        assert_eq!(player2, 99);

        server.start_match();

        // Run a few ticks
        for _ in 0..5 {
            server.step();
        }

        // Finalize and check artifact
        let artifact = server.finalize(EndReason::Complete);
        assert!(artifact.test_mode);
        assert_eq!(artifact.test_player_ids, vec![17, 99]);
        assert_eq!(artifact.entity_spawn_order, vec![17, 99]);
    }

    /// T0.18: Floor coherency - byte-identical broadcasts.
    #[test]
    fn test_t0_18_floor_coherency_broadcast() {
        let mut server = Server::new(ServerConfig::default());
        server.accept_session();
        server.accept_session();
        server.start_match();

        // Step and get serialized snapshot
        let (_, floor1, bytes1) = server.step();

        // The bytes would be sent to all sessions identically
        // Decode to verify floor is consistent
        let decoded: SnapshotProto = prost::Message::decode(bytes1.as_slice()).unwrap();
        assert_eq!(decoded.target_tick_floor, floor1);

        // Step again
        let (_, floor2, bytes2) = server.step();
        let decoded2: SnapshotProto = prost::Message::decode(bytes2.as_slice()).unwrap();
        assert_eq!(decoded2.target_tick_floor, floor2);
        assert!(floor2 > floor1, "Floor should be monotonic increasing");
    }

    /// T0.12: LastKnownIntent determinism - empty inputs use LKI.
    #[test]
    fn test_t0_12_lki_fallback() {
        let config = ServerConfig {
            match_duration_ticks: 10,
            ..Default::default()
        };
        let mut server = Server::new(config);
        server.accept_session();
        server.accept_session();
        server.start_match();

        // Step without any inputs - should use LKI (zero)
        let (snapshot1, _, _) = server.step();

        // All entities should be at origin (no movement with zero LKI)
        for entity in &snapshot1.entities {
            assert_eq!(entity.position, [0.0, 0.0]);
        }

        // Now finalize and verify artifact has fallback inputs
        let artifact = server.finalize(EndReason::Complete);

        // All inputs should be fallback since we didn't send any
        assert!(artifact.inputs.iter().all(|i| i.is_fallback));
    }

    /// Test replay artifact generation.
    #[test]
    fn test_replay_artifact_generation() {
        let config = ServerConfig {
            match_duration_ticks: 5,
            ..Default::default()
        };
        let mut server = Server::new(config);
        server.accept_session();
        server.accept_session();
        server.start_match();

        // Run the match
        while server.should_end_match().is_none() {
            server.step();
        }

        let artifact = server.finalize(EndReason::Complete);

        assert_eq!(artifact.replay_format_version, 1);
        assert!(artifact.initial_baseline.is_some());
        assert_eq!(artifact.tick_rate_hz, 60);
        assert_eq!(artifact.checkpoint_tick, 5);
        assert_eq!(artifact.end_reason, "complete");
        // 5 ticks * 2 players = 10 inputs
        assert_eq!(artifact.inputs.len(), 10);
    }

    /// T0.13a: Floor enforcement and recovery.
    ///
    /// Simulates a scenario where inputs are submitted below floor (as if
    /// snapshot packets were lost). Verifies these are dropped, then
    /// "recovery" occurs when inputs target future ticks again.
    #[test]
    fn test_t0_13a_floor_enforcement_recovery() {
        let config = ServerConfig {
            match_duration_ticks: 20,
            ..Default::default()
        };
        let mut server = Server::new(config);
        let (session1, _, _) = server.accept_session();
        server.accept_session();
        let (_, welcomes) = server.start_match();

        // Get initial floor (verified for sanity)
        let initial_floor = welcomes[0].1.target_tick_floor;
        assert_eq!(initial_floor, INPUT_LEAD_TICKS);

        // Step a few times to advance the floor
        for _ in 0..5 {
            server.step();
        }

        // Floor should now be higher
        let current_tick = 5;
        let current_floor = current_tick + INPUT_LEAD_TICKS;

        // Try to submit an input targeting OLD tick (below floor) - should be dropped
        let stale_input = InputCmdProto {
            tick: 2, // Way below current floor
            input_seq: 1,
            move_dir: vec![1.0, 0.0],
        };
        let result = server.receive_input(session1, stale_input);
        assert!(
            matches!(result, ValidationResult::DroppedBelowFloor { .. }),
            "Input below floor should be dropped: {:?}",
            result
        );

        // Now submit a valid input targeting current floor - should be accepted
        let valid_input = InputCmdProto {
            tick: current_floor,
            input_seq: 2,
            move_dir: vec![1.0, 0.0],
        };
        let result = server.receive_input(session1, valid_input);
        assert!(
            result.is_accepted(),
            "Input at floor should be accepted: {:?}",
            result
        );
    }

    /// T0.16: Connection timeout.
    ///
    /// Server should detect when connection phase exceeds timeout.
    /// Note: In v0, actual timeout is external (e.g., orchestrator checks).
    /// This test verifies the timeout constant exists and server exposes
    /// connection state for external timeout enforcement.
    #[test]
    fn test_t0_16_connection_timeout() {
        // Verify timeout constant is set per v0-parameters
        assert_eq!(CONNECT_TIMEOUT_MS, 30000);

        // Create server and verify session tracking
        let mut server = Server::new(ServerConfig::default());
        assert_eq!(server.session_count(), 0);
        assert!(!server.is_ready_to_start());

        // Add one session - not ready
        server.accept_session();
        assert_eq!(server.session_count(), 1);
        assert!(!server.is_ready_to_start());

        // Add second session - now ready
        server.accept_session();
        assert_eq!(server.session_count(), 2);
        assert!(server.is_ready_to_start());

        // The timeout itself would be enforced externally by checking:
        // - start_time (when server was created)
        // - current_time - start_time > CONNECT_TIMEOUT_MS
        // - server.is_ready_to_start() == false
        // If that condition is true, orchestrator would exit with non-zero.
        // The server exposes enough state for this check.
    }
}
