//! Input validation for Server Edge.
//!
//! Ref: FS-0007 Validation Rules
//! - NaN/Inf in move_dir: DROP + LOG
//! - Magnitude > 1.0: CLAMP + LOG
//! - Tick below floor: DROP
//! - Tick non-monotonic: DROP
//! - Tick window violation: DROP
//! - Rate limit exceeded: DROP

use flowstate_sim::{PlayerId, Tick};
use flowstate_wire::InputCmdProto;

use crate::input_buffer::InputBuffer;

/// Validation configuration.
#[derive(Debug, Clone, Copy)]
pub struct ValidationConfig {
    pub max_future_ticks: u64,
    pub input_rate_limit_per_sec: u32,
    pub tick_rate_hz: u32,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_future_ticks: 120,
            input_rate_limit_per_sec: 120,
            tick_rate_hz: 60,
        }
    }
}

/// Result of input validation.
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    /// Input accepted and buffered.
    Accepted,
    /// Input accepted with magnitude clamped.
    AcceptedWithClamp,
    /// Dropped: NaN or Inf in move_dir.
    DroppedNanInf,
    /// Dropped: Tick below target tick floor.
    DroppedBelowFloor { tick: Tick, floor: Tick },
    /// Dropped: Tick is late (below current tick).
    DroppedLate { tick: Tick, current: Tick },
    /// Dropped: Tick is too far in future.
    DroppedTooFuture { tick: Tick, max: Tick },
    /// Dropped: Rate limit exceeded.
    DroppedRateLimit,
    /// Dropped: InputSeq tied for this (player, tick).
    DroppedInputSeqTie,
    /// Dropped: Received before ServerWelcome.
    DroppedPreWelcome,
    /// Dropped: Unknown session.
    DroppedUnknownSession,
}

impl ValidationResult {
    pub fn is_accepted(&self) -> bool {
        matches!(self, Self::Accepted | Self::AcceptedWithClamp)
    }
}

/// Validate an input command.
///
/// # Arguments
/// * `input` - The input command to validate
/// * `current_tick` - Current server tick
/// * `target_tick_floor` - Last emitted target tick floor for this session
/// * `buffer` - Input buffer for rate limiting and InputSeq selection
/// * `player_id` - Player ID for this session (bound by Server Edge, not from input)
pub fn validate_input(
    input: &InputCmdProto,
    current_tick: Tick,
    target_tick_floor: Tick,
    buffer: &mut InputBuffer,
    player_id: PlayerId,
) -> ValidationResult {
    // Check for NaN/Inf
    if input.move_dir.len() != 2 {
        return ValidationResult::DroppedNanInf;
    }
    let (x, y) = (input.move_dir[0], input.move_dir[1]);
    if x.is_nan() || x.is_infinite() || y.is_nan() || y.is_infinite() {
        return ValidationResult::DroppedNanInf;
    }

    // Check tick below floor
    if input.tick < target_tick_floor {
        return ValidationResult::DroppedBelowFloor {
            tick: input.tick,
            floor: target_tick_floor,
        };
    }

    // Check tick is late
    if input.tick < current_tick {
        return ValidationResult::DroppedLate {
            tick: input.tick,
            current: current_tick,
        };
    }

    // Check tick is too far in future
    let max_tick = current_tick + buffer.config().max_future_ticks;
    if input.tick > max_tick {
        return ValidationResult::DroppedTooFuture {
            tick: input.tick,
            max: max_tick,
        };
    }

    // Check rate limit and buffer
    match buffer.try_buffer(player_id, input.clone()) {
        BufferResult::Accepted { clamped } => {
            if clamped {
                ValidationResult::AcceptedWithClamp
            } else {
                ValidationResult::Accepted
            }
        }
        BufferResult::RateLimited => ValidationResult::DroppedRateLimit,
        BufferResult::InputSeqTie => ValidationResult::DroppedInputSeqTie,
    }
}

/// Result of attempting to buffer an input.
#[derive(Debug, Clone, PartialEq)]
pub enum BufferResult {
    Accepted { clamped: bool },
    RateLimited,
    InputSeqTie,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_valid_input(tick: Tick, seq: u64) -> InputCmdProto {
        InputCmdProto {
            tick,
            input_seq: seq,
            move_dir: vec![1.0, 0.0],
        }
    }

    #[test]
    fn test_nan_rejection() {
        let mut buffer = InputBuffer::new(ValidationConfig::default());
        let input = InputCmdProto {
            tick: 5,
            input_seq: 1,
            move_dir: vec![f64::NAN, 0.0],
        };

        let result = validate_input(&input, 0, 0, &mut buffer, 0);
        assert_eq!(result, ValidationResult::DroppedNanInf);
    }

    #[test]
    fn test_inf_rejection() {
        let mut buffer = InputBuffer::new(ValidationConfig::default());
        let input = InputCmdProto {
            tick: 5,
            input_seq: 1,
            move_dir: vec![0.0, f64::INFINITY],
        };

        let result = validate_input(&input, 0, 0, &mut buffer, 0);
        assert_eq!(result, ValidationResult::DroppedNanInf);
    }

    #[test]
    fn test_below_floor_rejection() {
        let mut buffer = InputBuffer::new(ValidationConfig::default());
        let input = make_valid_input(5, 1);

        // Floor is 10, input targets 5
        let result = validate_input(&input, 0, 10, &mut buffer, 0);
        assert!(matches!(result, ValidationResult::DroppedBelowFloor { .. }));
    }

    #[test]
    fn test_late_rejection() {
        let mut buffer = InputBuffer::new(ValidationConfig::default());
        let input = make_valid_input(5, 1);

        // Current tick is 10, input targets 5
        let result = validate_input(&input, 10, 0, &mut buffer, 0);
        assert!(matches!(result, ValidationResult::DroppedLate { .. }));
    }

    #[test]
    fn test_too_future_rejection() {
        let config = ValidationConfig {
            max_future_ticks: 10,
            ..Default::default()
        };
        let mut buffer = InputBuffer::new(config);
        let input = make_valid_input(100, 1);

        // Current tick is 0, max is 0+10=10, input targets 100
        let result = validate_input(&input, 0, 0, &mut buffer, 0);
        assert!(matches!(result, ValidationResult::DroppedTooFuture { .. }));
    }

    #[test]
    fn test_valid_input_accepted() {
        let mut buffer = InputBuffer::new(ValidationConfig::default());
        let input = make_valid_input(5, 1);

        let result = validate_input(&input, 0, 0, &mut buffer, 0);
        assert!(result.is_accepted());
    }

    /// T0.7: Malformed inputs do not crash server.
    #[test]
    fn test_t0_07_malformed_inputs_no_crash() {
        let mut buffer = InputBuffer::new(ValidationConfig::default());

        // Empty move_dir
        let input1 = InputCmdProto {
            tick: 5,
            input_seq: 1,
            move_dir: vec![],
        };
        let _ = validate_input(&input1, 0, 0, &mut buffer, 0);

        // Single element move_dir
        let input2 = InputCmdProto {
            tick: 5,
            input_seq: 2,
            move_dir: vec![1.0],
        };
        let _ = validate_input(&input2, 0, 0, &mut buffer, 0);

        // NaN
        let input3 = InputCmdProto {
            tick: 5,
            input_seq: 3,
            move_dir: vec![f64::NAN, f64::NAN],
        };
        let _ = validate_input(&input3, 0, 0, &mut buffer, 0);

        // Negative infinity
        let input4 = InputCmdProto {
            tick: 5,
            input_seq: 4,
            move_dir: vec![f64::NEG_INFINITY, f64::NEG_INFINITY],
        };
        let _ = validate_input(&input4, 0, 0, &mut buffer, 0);

        // Huge magnitude
        let input5 = InputCmdProto {
            tick: 5,
            input_seq: 5,
            move_dir: vec![1e308, 1e308],
        };
        let _ = validate_input(&input5, 0, 0, &mut buffer, 0);

        // All handled without panic
    }
}
