//! Session management for Server Edge.
//!
//! Ref: DM-0008 (Session)

use flowstate_sim::{EntityId, PlayerId};

/// Session identifier (server-internal).
pub type SessionId = u64;

/// Client session state.
#[derive(Debug, Clone)]
pub struct Session {
    pub id: SessionId,
    pub player_id: PlayerId,
    pub controlled_entity_id: EntityId,
    /// Last valid input tick received from this session (for monotonicity check).
    pub last_valid_tick: Option<u64>,
    /// Last input_seq received from this session.
    pub last_input_seq: Option<u64>,
}

impl Session {
    /// Create a new session.
    pub fn new(id: SessionId, player_id: PlayerId, controlled_entity_id: EntityId) -> Self {
        Self {
            id,
            player_id,
            controlled_entity_id,
            last_valid_tick: None,
            last_input_seq: None,
        }
    }
}
