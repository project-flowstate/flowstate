# Networking Parameters (v0)

This document contains tunable, non-constitutional networking parameters for v0.
These values may change as we iterate, without requiring changes to invariants or domain model entries.

## v0 defaults

| Parameter | Value | Notes |
|---|---:|---|
| tick_rate_hz | 60 | Authoritative simulation tick rate for v0 matches |
| snapshot_rate_hz | 60 | One snapshot per tick (v0 simplicity) |
| input_send_rate_hz | 60 | Target send rate; may be clamped to tick rate |
| input_rate_limit_per_sec | 120 | Tier-0 spam control |
| input_tick_window_ticks | ±120 | Tier-0 sanity window (dev/LAN posture) |

## Change policy

- These parameters are not architectural law. They can be tuned as needed.
- Changes should be recorded via PR with a brief rationale in the commit message or PR description.
