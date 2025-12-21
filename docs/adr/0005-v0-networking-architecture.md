# ADR 0005: v0 Networking Architecture

## Status
Proposed

## Type
Technical

## Context

Flowstate requires client-server networked multiplayer. The goal for v0 is to demonstrate **two native clients moving characters with WASD inputs** over a network, with the server running an authoritative, deterministic simulation.

Key constraints:
- The simulation MUST be deterministic (INV-0001) and advance in fixed Ticks (INV-0002)
- The server is authoritative (INV-0003); client inputs are intent, not commands
- The Simulation Plane MUST NOT perform I/O or networking (INV-0004)
- All inputs and outputs MUST carry explicit Tick identifiers (INV-0005)
- Replay artifacts MUST reproduce authoritative outcomes on the same build/platform (INV-0006)

We must choose: transport adapter, message serialization, channel semantics, snapshot strategy, input handling, baseline synchronization, tick synchronization, and input validation.

For v0, we optimize for:
- **Rapid iteration** — inline schemas, LAN-friendly simplicity
- **Architectural correctness** — plane boundaries intact, determinism provable
- **Future flexibility** — decisions can be upgraded without architectural rewrites

## Decision

We adopt a **two-layer networking architecture**:

### A. Semantic Contract (Transport-Agnostic)

The I/O Boundary (DM-0011) mediates communication between the Control Plane and Simulation Plane via **tick-indexed messages** on **logical Channels** (DM-0009) with defined semantics:

- **Realtime Channel:** Unreliable + sequenced (discard older packets)
  - Uses: Snapshots (DM-0007), InputCmds (DM-0006)
  - Rationale: Prefer fresh data; late arrivals are obsolete
- **Control Channel:** Reliable + ordered
  - Uses: ClientHello, ServerWelcome, JoinBaseline, match lifecycle events
  - Rationale: Critical for correctness; must not be lost or reordered

This semantic model is inspired by QUIC's unreliable datagrams + reliable streams, making future WebTransport adoption straightforward.

### B. Transport Adapters (v0 Concrete Choices)

For v0, we use:

- **ENet** for native clients (Rust server via `enet` crate, Godot client via `ENetMultiplayerPeer`)
  - ENet channels: 0 (unreliable+sequenced) → Realtime, 1 (reliable+ordered) → Control
  - Justification: Battle-tested, Godot-native support, minimal friction for v0
  - Limitation: Native-only (no web browser support)
- **Protobuf** (inline `#[derive(prost::Message)]` in v0; migrate to `.proto` files in v0.2)
  - Justification: Fast iteration now, formal schemas later without protocol changes
- **Explicit JoinBaseline** on Control Channel for initial state transfer
  - Justification: Avoids ambiguity; client knows when it's synchronized
- **60 Hz tick rate**, **unreliable snapshots** @ 60 Hz, **unreliable inputs** @ 60 Hz
  - See [docs/networking/v0-parameters.md](../networking/v0-parameters.md) for tunables
  - Justification: Simple 1:1 tick-to-snapshot-to-input mapping for v0
- **Tier-0 input validation:** magnitude limit, tick window (±120 ticks), rate limit (120/sec)
  - Justification: Permissive for LAN/dev; prevents crashes from malformed inputs
  - Values in [docs/networking/v0-parameters.md](../networking/v0-parameters.md)

### Determinism Scope

v0 guarantees determinism **within the same build and platform**. Cross-platform determinism (e.g., Windows vs Linux, x86 vs ARM) is deferred to post-v0. This allows us to prove architectural correctness without immediately solving floating-point portability.

## Rationale

**Why ENet for v0?** Godot has first-class support; server has mature Rust bindings; avoids WebTransport complexity until web clients are required.

**Why inline Protobuf now, .proto later?** Faster iteration during active development; no breaking protocol changes when we formalize schemas.

**Why unreliable snapshots?** Late snapshots are obsolete. Clients interpolate; missing a frame is acceptable. Reliable delivery would buffer stale state.

**Why unreliable inputs?** Inputs are timestamped by tick. If an input arrives late, it's already obsolete (server has advanced). Reliable delivery would queue stale inputs. Future: client-side prediction and reconciliation will handle packet loss gracefully; for v0, we accept occasional input loss in exchange for simplicity.

**Why explicit JoinBaseline?** "Assume synchronized after handshake" is ambiguous and error-prone. Explicit baseline message makes synchronization observable and testable.

**Why permissive Tier-0 validation?** We're building on LAN for v0. Strict validation (tick windows, rate limits, payload sizes) comes in v0.1+. For now: prevent crashes, not cheating.

**Why QUIC-inspired semantic model?** WebTransport (our future web transport) is built on QUIC. By designing our channels to match QUIC's unreliable datagrams + reliable streams model now, we avoid architectural churn later. ENet is a temporary adapter; the channel semantics are durable.

## Constraints & References

- Constitution IDs:
  - INV-0001 (Deterministic Simulation), INV-0002 (Fixed Timestep), INV-0003 (Authoritative Simulation), INV-0004 (Simulation Plane Isolation), INV-0005 (Tick-Indexed I/O Contract), INV-0006 (Replay Verifiability)
  - DM-0001 (Tick), DM-0006 (InputCmd), DM-0007 (Snapshot), DM-0008 (Session), DM-0009 (Channel), DM-0010 (Match)
  - AC-0001 (v0 Two-Client Multiplayer Slice)
  - KC-0001 (Plane Boundary Violation), KC-0002 (Replay Verification Blocker)
- Related ADRs:
  - ADR-0001 (Three-Plane Architecture)
  - ADR-0002 (Deterministic Simulation)
  - ADR-0003 (Fixed Timestep Simulation)
  - ADR-0004 (Server Authoritative Architecture)
- Parameters:
  - [docs/networking/v0-parameters.md](../networking/v0-parameters.md) — Tunable numeric values (tick rates, limits, windows)

## Alternatives Considered

- **Reliable inputs via TCP-like channel** — Rejected: Queues stale inputs; breaks real-time responsiveness. Input timestamps make reliable delivery unnecessary.
- **Delta-compressed snapshots** — Deferred to v0.1+: Adds complexity; full snapshots are simpler for v0.
- **`.proto` files from day one** — Deferred to v0.2: Slows iteration; inline structs are faster to change during active development.
- **Strict input validation (cryptographic signatures, replay attack prevention)** — Deferred to v0.1+: Overkill for LAN-only v0; Tier-0 validation prevents crashes without security theater.
- **WebTransport from day one** — Deferred: No Godot support yet; ENet unblocks v0 without compromising future WebTransport adoption (semantic model is already QUIC-aligned).

## Implications

**Enables:**
- Rapid v0 implementation with battle-tested tools (ENet, Godot, Protobuf)
- Architectural correctness from day one (plane boundaries, determinism, replay)
- Future transport upgrades (WebTransport) without protocol redesign

**Constrains:**
- v0 is native-only (no web clients until WebTransport adapter exists)
- Cross-platform determinism deferred (limits replay portability until addressed)
- Permissive Tier-0 validation means security hardening is future work

**Migration / Churn Costs:**
- ENet → WebTransport migration requires new transport adapter, but channel semantics and message types remain unchanged
- Inline Protobuf → `.proto` files migration is straightforward (codegen produces same Rust types)
- Tier-0 → production validation is additive (no breaking changes)

**Operational / Contributor Impact:**
- Contributors must understand tick-indexing and channel semantics
- Replay verification becomes a standard CI gate (AC-0001 requirement)
- Numeric parameters live in [docs/networking/v0-parameters.md](../networking/v0-parameters.md); tune there, not in code constants

## Follow-ups

- Implement ENet server adapter with two-channel mapping (Realtime/Control)
- Define initial message types (ClientHello, ServerWelcome, JoinBaseline, InputCmd, Snapshot) with inline Protobuf
- Implement Tier-0 replay verification test (same build/platform)
- Document operational parameters in [docs/networking/v0-parameters.md](../networking/v0-parameters.md)
- Plan v0.1 features: client-side prediction, delta compression, stricter validation
- Plan v0.2+ migration: `.proto` files, WebTransport adapter, cross-platform determinism
