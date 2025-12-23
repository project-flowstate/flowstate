//! Input buffering for Server Edge.
//!
//! Ref: FS-0007 Validation Rules
//! - Buffer keyed by (player_id, tick)
//! - InputSeq selection: greatest wins
//! - Rate limiting: per-tick limit = ceil(input_rate_limit_per_sec / tick_rate_hz)
//! - Buffer cap: one selected InputCmd per (player_id, tick)

use std::collections::HashMap;

use flowstate_sim::{PlayerId, Tick};
use flowstate_wire::InputCmdProto;

use crate::validation::{BufferResult, ValidationConfig};

/// Per-(player_id, tick) buffer entry.
#[derive(Debug, Clone)]
struct BufferEntry {
    /// Selected InputCmd (the one with max InputSeq so far).
    selected: InputCmdProto,
    /// Maximum InputSeq observed.
    max_input_seq: u64,
    /// Whether max_input_seq was observed more than once (tie).
    max_seq_tied: bool,
    /// Number of inputs received for this (player_id, tick) in this tick window.
    receive_count: u32,
}

/// Input buffer for Server Edge.
///
/// Buffers inputs by (player_id, tick) within the InputTickWindow.
pub struct InputBuffer {
    config: ValidationConfig,
    /// Buffer keyed by (player_id, tick).
    buffer: HashMap<(PlayerId, Tick), BufferEntry>,
    /// Per-tick rate limit = ceil(input_rate_limit_per_sec / tick_rate_hz).
    per_tick_limit: u32,
}

impl InputBuffer {
    /// Create a new input buffer.
    pub fn new(config: ValidationConfig) -> Self {
        // per_tick_limit = ceil(input_rate_limit_per_sec / tick_rate_hz)
        let per_tick_limit = config
            .input_rate_limit_per_sec
            .div_ceil(config.tick_rate_hz);

        Self {
            config,
            buffer: HashMap::new(),
            per_tick_limit,
        }
    }

    /// Get the configuration.
    pub fn config(&self) -> &ValidationConfig {
        &self.config
    }

    /// Try to buffer an input.
    ///
    /// Returns `BufferResult` indicating whether the input was accepted.
    pub fn try_buffer(&mut self, player_id: PlayerId, input: InputCmdProto) -> BufferResult {
        let key = (player_id, input.tick);
        let input_seq = input.input_seq;

        // Check if we already have an entry for this (player_id, tick)
        if let Some(entry) = self.buffer.get_mut(&key) {
            // Rate limiting: check receive count
            if entry.receive_count >= self.per_tick_limit {
                return BufferResult::RateLimited;
            }
            entry.receive_count += 1;

            // InputSeq tie-breaking per spec:
            // - seq > max: update to new max, clear tie flag
            // - seq == max: set tie flag
            // - seq < max: ignore for selection
            if input_seq > entry.max_input_seq {
                entry.max_input_seq = input_seq;
                entry.max_seq_tied = false;
                entry.selected = input;
            } else if input_seq == entry.max_input_seq {
                entry.max_seq_tied = true;
            }
            // else seq < max: ignore

            // Check for magnitude clamping
            let clamped = needs_magnitude_clamp(&entry.selected.move_dir);
            if clamped {
                clamp_magnitude(&mut entry.selected.move_dir);
            }

            BufferResult::Accepted { clamped }
        } else {
            // First input for this (player_id, tick)
            let clamped = needs_magnitude_clamp(&input.move_dir);
            let mut input = input;
            if clamped {
                clamp_magnitude(&mut input.move_dir);
            }

            let entry = BufferEntry {
                selected: input.clone(),
                max_input_seq: input_seq,
                max_seq_tied: false,
                receive_count: 1,
            };
            self.buffer.insert(key, entry);

            BufferResult::Accepted { clamped }
        }
    }

    /// Take the selected input for a (player_id, tick), removing it from the buffer.
    ///
    /// Returns `None` if:
    /// - No input exists for this (player_id, tick)
    /// - InputSeq was tied (per spec: use LastKnownIntent instead)
    pub fn take_input(&mut self, player_id: PlayerId, tick: Tick) -> Option<InputCmdProto> {
        let key = (player_id, tick);
        let entry = self.buffer.remove(&key)?;

        if entry.max_seq_tied {
            // Tied InputSeq → drop and use LKI
            None
        } else {
            Some(entry.selected)
        }
    }

    /// Evict all buffered entries for ticks before the given tick.
    pub fn evict_before(&mut self, tick: Tick) {
        self.buffer.retain(|&(_, t), _| t >= tick);
    }

    /// Check if an entry exists (for testing).
    #[cfg(test)]
    pub fn has_entry(&self, player_id: PlayerId, tick: Tick) -> bool {
        self.buffer.contains_key(&(player_id, tick))
    }
}

/// Check if magnitude exceeds 1.0.
fn needs_magnitude_clamp(move_dir: &[f64]) -> bool {
    if move_dir.len() != 2 {
        return false;
    }
    let mag_sq = move_dir[0] * move_dir[0] + move_dir[1] * move_dir[1];
    mag_sq > 1.0
}

/// Clamp magnitude to 1.0 in place.
fn clamp_magnitude(move_dir: &mut [f64]) {
    if move_dir.len() != 2 {
        return;
    }
    let mag_sq = move_dir[0] * move_dir[0] + move_dir[1] * move_dir[1];
    if mag_sq > 1.0 {
        let mag = mag_sq.sqrt();
        move_dir[0] /= mag;
        move_dir[1] /= mag;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_input(tick: Tick, seq: u64, x: f64, y: f64) -> InputCmdProto {
        InputCmdProto {
            tick,
            input_seq: seq,
            move_dir: vec![x, y],
        }
    }

    #[test]
    fn test_first_input_accepted() {
        let mut buffer = InputBuffer::new(ValidationConfig::default());
        let input = make_input(5, 1, 1.0, 0.0);

        let result = buffer.try_buffer(0, input);
        assert_eq!(result, BufferResult::Accepted { clamped: false });
        assert!(buffer.has_entry(0, 5));
    }

    #[test]
    fn test_higher_seq_replaces() {
        let mut buffer = InputBuffer::new(ValidationConfig::default());

        // First input with seq 1
        buffer.try_buffer(0, make_input(5, 1, 1.0, 0.0));

        // Second input with seq 2 (higher)
        buffer.try_buffer(0, make_input(5, 2, 0.0, 1.0));

        // Should have the second input
        let taken = buffer.take_input(0, 5).unwrap();
        assert_eq!(taken.input_seq, 2);
        assert_eq!(taken.move_dir, vec![0.0, 1.0]);
    }

    #[test]
    fn test_lower_seq_ignored() {
        let mut buffer = InputBuffer::new(ValidationConfig::default());

        // First input with seq 5
        buffer.try_buffer(0, make_input(5, 5, 1.0, 0.0));

        // Second input with seq 3 (lower)
        buffer.try_buffer(0, make_input(5, 3, 0.0, 1.0));

        // Should still have first input
        let taken = buffer.take_input(0, 5).unwrap();
        assert_eq!(taken.input_seq, 5);
        assert_eq!(taken.move_dir, vec![1.0, 0.0]);
    }

    #[test]
    fn test_equal_seq_causes_tie() {
        let mut buffer = InputBuffer::new(ValidationConfig::default());

        // First input with seq 5
        buffer.try_buffer(0, make_input(5, 5, 1.0, 0.0));

        // Second input with seq 5 (same - tie!)
        buffer.try_buffer(0, make_input(5, 5, 0.0, 1.0));

        // Should return None (tie → use LKI)
        let taken = buffer.take_input(0, 5);
        assert!(taken.is_none());
    }

    #[test]
    fn test_tie_cleared_by_higher_seq() {
        // Use a higher rate limit so we can send 3 inputs
        let config = ValidationConfig {
            max_future_ticks: 120,
            input_rate_limit_per_sec: 180, // 3 per tick at 60hz
            tick_rate_hz: 60,
        };
        let mut buffer = InputBuffer::new(config);

        // Create a tie
        buffer.try_buffer(0, make_input(5, 5, 1.0, 0.0));
        buffer.try_buffer(0, make_input(5, 5, 0.0, 1.0));

        // Now send a higher seq
        buffer.try_buffer(0, make_input(5, 8, 0.5, 0.5));

        // Should have the seq 8 input (tie cleared)
        let taken = buffer.take_input(0, 5).unwrap();
        assert_eq!(taken.input_seq, 8);
    }

    /// T0.6, T0.13: Rate limiting - N > limit drops at least N-limit.
    #[test]
    fn test_rate_limiting() {
        let config = ValidationConfig {
            max_future_ticks: 120,
            input_rate_limit_per_sec: 120,
            tick_rate_hz: 60,
        };
        let mut buffer = InputBuffer::new(config);

        // per_tick_limit = ceil(120/60) = 2
        // Send 5 inputs for the same (player, tick)
        let mut accepted = 0;
        let mut dropped = 0;

        for seq in 1..=5 {
            let result = buffer.try_buffer(0, make_input(5, seq, 1.0, 0.0));
            if result == BufferResult::RateLimited {
                dropped += 1;
            } else {
                accepted += 1;
            }
        }

        // Should accept 2, drop 3
        assert_eq!(accepted, 2);
        assert_eq!(dropped, 3);
    }

    #[test]
    fn test_magnitude_clamping() {
        let mut buffer = InputBuffer::new(ValidationConfig::default());

        // Input with magnitude > 1
        let input = make_input(5, 1, 2.0, 0.0);
        let result = buffer.try_buffer(0, input);

        assert_eq!(result, BufferResult::Accepted { clamped: true });

        let taken = buffer.take_input(0, 5).unwrap();
        // Should be clamped to unit length
        let mag = (taken.move_dir[0].powi(2) + taken.move_dir[1].powi(2)).sqrt();
        assert!((mag - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_eviction() {
        let mut buffer = InputBuffer::new(ValidationConfig::default());

        buffer.try_buffer(0, make_input(5, 1, 1.0, 0.0));
        buffer.try_buffer(0, make_input(10, 1, 1.0, 0.0));
        buffer.try_buffer(0, make_input(15, 1, 1.0, 0.0));

        // Evict before tick 10
        buffer.evict_before(10);

        assert!(!buffer.has_entry(0, 5));
        assert!(buffer.has_entry(0, 10));
        assert!(buffer.has_entry(0, 15));
    }

    /// T0.11: Future input non-interference.
    #[test]
    fn test_t0_11_future_input_buffered() {
        let mut buffer = InputBuffer::new(ValidationConfig::default());

        // Buffer input for tick 5 (future)
        buffer.try_buffer(0, make_input(5, 1, 1.0, 0.0));

        // Should still be there
        assert!(buffer.has_entry(0, 5));

        // Taking input for tick 0 should return None (not 5)
        assert!(buffer.take_input(0, 0).is_none());

        // Tick 5 should still be available
        assert!(buffer.take_input(0, 5).is_some());
    }

    /// T0.13: InputSeq selection (tied → LKI fallback).
    #[test]
    fn test_t0_13_inputseq_tie_uses_lki() {
        let mut buffer = InputBuffer::new(ValidationConfig::default());

        // Send two inputs with same seq
        buffer.try_buffer(0, make_input(5, 10, 1.0, 0.0));
        buffer.try_buffer(0, make_input(5, 10, 0.0, 1.0));

        // take_input should return None (use LKI)
        let result = buffer.take_input(0, 5);
        assert!(result.is_none());
    }
}
