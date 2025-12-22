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
| max_future_ticks | 120 | Maximum ticks ahead a client can target (InputTickWindow upper bound) |
| input_tick_window | `[current_tick, current_tick + max_future_ticks]` | Future-only acceptance; late inputs dropped |
| input_lead_ticks | 1 | TargetTickFloor = server.current_tick + input_lead_ticks |

## Parameter definitions

- **max_future_ticks:** Defines the InputTickWindow (DM-0022) upper bound. Inputs targeting `cmd.tick > current_tick + max_future_ticks` are rejected.
- **input_tick_window:** Future-only acceptance window. Inputs with `cmd.tick < current_tick` (late) are always dropped. This is not a symmetric ± window.
- **input_lead_ticks:** Used to compute TargetTickFloor (DM-0025) in ServerWelcome and Snapshots. Clients target at least `TargetTickFloor = server.current_tick + input_lead_ticks`.

## Change policy

- These parameters are not architectural law. They can be tuned as needed.
- Changes should be recorded via PR with a brief rationale in the commit message or PR description.
