# Domain Model

This is the canonical conceptual spine for Flowstate. New important concepts MUST be defined here before they are depended on in specs, ADRs, or code.

<!--
ENTRY FORMAT (use H3 with anchor):

  ### <a id="DM-NNNN"></a> DM-NNNN - Title
  **Status:** Active  
  **Tags:** tag1, tag2

  **Definition:** A concise definition of this concept...

Section groupings (H2) are optional, for organization only.
-->

## Core Simulation

### <a id="DM-0001"></a> DM-0001 — Tick
**Status:** Active  
**Tags:** determinism, simulation

**Definition:** A single discrete simulation timestep; the atomic unit of game time. All authoritative state transitions occur on Tick boundaries.

### <a id="DM-0002"></a> DM-0002 — World
**Status:** Active  
**Tags:** simulation

**Definition:** The authoritative simulation state container, encompassing both static environment (terrain, obstacles, surfaces, traversal features, hazard volumes) and dynamic simulation state (Entities and their evolving state). The Simulation Core (DM-0014) maintains World state and advances it each Tick (DM-0001).

*Non-normative note: World is the complete simulation context. Static environment defines affordances and constraints; dynamic state includes player Characters, projectiles, timers, triggers, and any other state that evolves during gameplay.*

### <a id="DM-0003"></a> DM-0003 — Character
**Status:** Active  
**Tags:** simulation, entity

**Definition:** The entity a player controls; has position, state, and can perform actions. The primary interactive agent in the simulation.

### <a id="DM-0004"></a> DM-0004 — LocomotionMode
**Status:** Active  
**Tags:** movement

**Definition:** The character's current movement regime (Grounded, Airborne, Gliding, Rail). Defines how movement behaves and what constraints apply.

### <a id="DM-0005"></a> DM-0005 — Entity
**Status:** Active  
**Tags:** simulation, architecture, identity

**Definition:** An object in the simulation with a unique identity and simulation state (often including position). The base unit of dynamic game objects. Each Entity has a unique EntityId (DM-0020) for its lifetime within the Match (DM-0010). A Character (DM-0003) is a kind of Entity. A World (DM-0002) contains Entities.

*Non-normative note: Entities include players, projectiles, obstacles, pickups, timers, triggers, etc. The Entity abstraction provides the common interface for identity, state management, and lifecycle. Not all entities are spatial (e.g., match timers).*

### <a id="DM-0006"></a> DM-0006 — InputCmd
**Status:** Active  
**Tags:** networking, input, protocol

**Definition:** A tick-indexed message containing one player's input intent (movement direction, aim, actions) for a specific Tick (DM-0001). An InputCmd represents a per-player intent sample tagged for application at a specific tick. The Server Edge (DM-0011) may receive multiple InputCmd messages for the same (PlayerId, Tick) and MUST deterministically resolve them (e.g., via InputSeq (DM-0026)) to produce exactly one AppliedInput (DM-0024). The Server Edge then converts AppliedInput into StepInput (DM-0027), and the Simulation Core (DM-0014) consumes StepInput during the Tick step.

*Non-normative note: We chose "InputCmd" over "InputFrame" because it's per-player, not per-tick-aggregate. An InputFrame would mean "all players' inputs for tick T," which is a server-side collection concern, not a protocol primitive. Clients may send multiple InputCmds for the same tick due to network conditions; the Server Edge uses a deterministic selection mechanism (e.g., InputSeq greatest-wins) to produce exactly one AppliedInput per player per tick.*

### <a id="DM-0007"></a> DM-0007 — Snapshot
**Status:** Active  
**Tags:** networking, state-sync, protocol

**Definition:** A tick-indexed serialization of authoritative world state produced at a specific Tick (DM-0001) by the Simulation Core (DM-0014), transmitted to Game Clients (DM-0015) by the Server Edge (DM-0011) for state synchronization.

*Non-normative note: Snapshots are authoritative. Game Clients use them for reconciliation and remote entity interpolation. Snapshot packing strategies (priority-based, budget-aware) are implementation details. The Simulation Core produces snapshot data; the Server Edge handles transmission.*

### <a id="DM-0008"></a> DM-0008 — Session
**Status:** Active  
**Tags:** networking, connection

**Definition:** A client's connection lifecycle from handshake through disconnect, including assigned player identity and synchronization state. Sessions are owned by the Server Edge (DM-0011) and are not part of simulation state.

*Non-normative note: The simulation knows about player inputs and entity state. The Server Edge (DM-0011) manages session lifecycle (connections, authentication, tokens). Don't conflate simulation state with session state.*

### <a id="DM-0009"></a> DM-0009 — Channel
**Status:** Active  
**Tags:** networking, transport, protocol

**Definition:** A logical communication lane with defined delivery semantics (reliability, ordering, sequencing), independent of transport implementation. Examples: Realtime (unreliable + sequenced), Control (reliable + ordered), Bulk (reliable + independent, does not block realtime channels).

*Non-normative note: ENet channels, WebTransport streams/datagrams, and future transports all map to this semantic model. The channel abstraction is transport-agnostic. "Independent" means separate lanes; Bulk traffic does not cause head-of-line blocking for Realtime/Control.*

### <a id="DM-0010"></a> DM-0010 — Match
**Status:** Active  
**Tags:** orchestration, replay, simulation

**Definition:** A discrete game instance with a defined lifecycle (create → active → end), a fixed simulation tick rate, an initial authoritative state, and a set of participating Sessions (DM-0008). Match is the scope boundary for gameplay, replay artifacts, and outcome determination.

*Non-normative note: A Match corresponds to "one game" from a player's perspective. It has a stable tick rate (see [docs/networking/v0-parameters.md](../networking/v0-parameters.md) for v0 values), a known start state, and produces a complete input+output history for replay (INV-0006). Session management is Server Edge logic; the Simulation Core sees player inputs and produces world state. Match is the conceptual glue between them.*

### <a id="DM-0011"></a> DM-0011 — Server Edge
**Status:** Active  
**Tags:** networking, architecture

**Definition:** The networking and session boundary within a Game Server Instance (DM-0013). Owns sockets/transports, validates inputs, manages Sessions (DM-0008), and exchanges tick-indexed messages with the Simulation Core (DM-0014). Performs all I/O operations on behalf of the Game Server Instance.

*Non-normative note: The Server Edge is the runtime interface between the outside world and the deterministic Simulation Core. It handles networking, session lifecycle, snapshot transmission, and (in future) replay recording. It lives in the same process as the Simulation Core but maintains a strict message-passing boundary per INV-0004. The term "I/O boundary" may be used descriptively, but Server Edge is the canonical component name.*

### <a id="DM-0012"></a> DM-0012 — Matchmaker
**Status:** Active  
**Tags:** orchestration, infrastructure, preservability

**Definition:** The sole system a Game Client (DM-0015) contacts to create, find, or join a match. Owns matchmaking, lobbies/queues (if any), match creation, and assignment of players to a specific Game Server Instance (DM-0013). Returns connection information/credentials for the assigned Game Server Instance. Must not contain game rules. Optional for LAN/dev scenarios.

*Non-normative note: The Matchmaker is the single entry point for clients seeking a match. Any other backend concepts (authentication, analytics, provisioning) are dependencies of the Matchmaker, not peer authorities. For local/LAN play, the Matchmaker may not exist—clients connect directly to a known Game Server Instance. The Matchmaker never directly manipulates simulation state.*

### <a id="DM-0013"></a> DM-0013 — Game Server Instance
**Status:** Active  
**Tags:** architecture, networking

**Definition:** One running authoritative match runtime (process/container). Owns the authoritative game simulation for exactly one match lifecycle. Contains exactly two named subcomponents: the Server Edge (DM-0011) and the Simulation Core (DM-0014). The Server Edge performs all I/O; the Simulation Core contains deterministic game logic.

*Non-normative note: In v0, the Game Server Instance may include minimal match lifecycle logic for dev/LAN (e.g., auto-start when two clients connect), but this logic must not bypass the Server Edge or inject authoritative state directly into the Simulation Core.*

### <a id="DM-0014"></a> DM-0014 — Simulation Core
**Status:** Active  
**Tags:** simulation, determinism, architecture

**Definition:** The deterministic, fixed-timestep, replayable game rules engine that defines authoritative state transitions for World (DM-0002). It is engine-agnostic and safe to embed in multiple hosts (e.g., Game Server Instance (DM-0013) and, in future tiers, clients for prediction/rollback), but only the authoritative server instance is permitted to commit game-outcome-affecting state (see INV-0003). The Simulation Core performs NO I/O, networking, rendering, engine calls, or OS/system calls. Replayable purely from recorded inputs.

**Normative constraints:**
- The Simulation Core MUST consume only simulation-plane input types (StepInput, DM-0027), not protocol-plane types (InputCmd, DM-0006).
- The Server Edge (DM-0011) MUST convert AppliedInput (DM-0024) to StepInput before invoking the Simulation Core.

*Non-normative note: The Simulation Core advances in discrete Ticks (DM-0001). It consumes validated, tick-indexed StepInput (DM-0027) values supplied by the Server Edge (DM-0011), produces Baselines (DM-0016) and Snapshots (DM-0007), and maintains World state. If clients implement prediction/rollback, they MUST invoke the same Simulation Core logic (same rules/version) rather than duplicating gameplay math; client results remain non-authoritative and are reconciled to server snapshots. Isolation is enforced by INV-0004.*

### <a id="DM-0015"></a> DM-0015 — Game Client
**Status:** Active  
**Tags:** presentation, networking

**Definition:** The player runtime. Owns rendering, input capture, UI, interpolation, and (future) prediction/reconciliation. Connects to a Game Server Instance (DM-0013) via its Server Edge (DM-0011). Receives Snapshots (DM-0007) and sends InputCmds (DM-0006). MUST NOT authoritatively decide game-outcome-affecting state.

*Non-normative note: Game Clients are untrusted. They capture player intent (inputs) and send it to the server. The server decides outcomes. Clients render authoritative state received via Snapshots. Client-side prediction is a presentation optimization that does not affect authoritative outcomes.*

### <a id="DM-0016"></a> DM-0016 — Baseline
**Status:** Active  
**Tags:** networking, state-sync, protocol, replay

**Definition:** A tick-indexed serialization of authoritative world state at a specific Tick (DM-0001) before inputs are applied at that tick. Used for join synchronization (JoinBaseline) and replay initialization. Distinguished from Snapshot (DM-0007) by timing: Baseline at tick T is pre-step state; Snapshot at tick T+1 is post-step state after inputs at tick T are applied and the step executes. Baseline.tick uses the canonical simulation tick numbering.

*Non-normative note: Baseline eliminates ambiguity in initial state handling. When a client joins mid-match or a replay starts, Baseline provides the deterministic starting point. The Simulation Core emits Baseline as a serializable artifact; the Server Edge owns all I/O and transmission. Baseline.tick = T means “world state before any inputs are applied at tick T.”*

### <a id="DM-0017"></a> DM-0017 — ReplayArtifact
**Status:** Active  
**Tags:** replay, schema, traceability

**Definition:** A versioned, self-describing record of an authoritative Match that is sufficient to **reproduce and verify** the authoritative outcome under the replay scope defined by INV-0006. A ReplayArtifact is produced by the Server Edge and treated as the canonical input to the replay verifier.

A ReplayArtifact MUST include enough information to deterministically re-simulate the same authoritative timeline, including at minimum:
- **format_version** (and, if applicable, a compatibility/profile identifier for the replay scope),
- **initialization data** sufficient to reconstruct the authoritative starting state used for replay (e.g., an initial Baseline (DM-0016) or equivalent canonical state seed + deterministic initialization parameters). This MUST include enough information to reproduce any determinism-critical identity allocation performed during initialization (e.g., deterministic spawn/setup order and any required participant→entity association needed to reproduce EntityId (DM-0020) assignment).
- **determinism-relevant configuration** required to interpret and reproduce the run (e.g., tick rate; any tuning parameters that affect simulation outcomes; and identifiers for any algorithms whose choice affects outcomes),
- a chronologically ordered stream of **AppliedInput (DM-0024)** values: the per-tick inputs that were actually applied by the authoritative simulation (server truth), not raw client messages,
- one or more verification anchors: **checkpoint_tick** and associated **StateDigest**(s) (DM-0018) as required by the replay procedure (e.g., checkpoint digest and/or final digest),
- **end_reason** (the authoritative termination cause for the recorded segment).

A ReplayArtifact is distinct from Baseline (DM-0016) and Snapshot (DM-0007): Baseline/Snapshot are world-state serializations used for join sync or post-step observation, while ReplayArtifact is an authoritative "reproduction + verification" package for determinism auditing.

*Non-normative note: ReplayArtifact proves determinism in practice by capturing "what the server actually applied." This includes any server-side fill rules (e.g., LastKnownIntent (DM-0023)) and validation effects; replay should reproduce the same StateDigest at the declared anchors under the stated replay scope (v0: same build/platform per INV-0006).*

### <a id="DM-0018"></a> DM-0018 — StateDigest
**Status:** Active  
**Tags:** verification, determinism, replay

**Definition:** A deterministic digest value computed from a **canonical serialization** of authoritative World (DM-0002) state at a specific Tick (DM-0001). StateDigest is used as a verification anchor for replay (INV-0006): when re-simulating from the same starting state and AppliedInput (DM-0024) stream, the computed StateDigest at the declared tick(s) MUST match the digest recorded in the ReplayArtifact (DM-0017) under the intended replay scope.

A StateDigest is defined by:
- the **tick** at which it is computed,
- a **canonicalization rule** for state-to-bytes (so equivalent states serialize identically),
- and a **digest algorithm identifier** (so the digest computation is unambiguous and versionable).

The Simulation Core (DM-0014) provides the canonical StateDigest computation for the project.

*Non-normative note: StateDigest is a regression/verification mechanism, not a security primitive; it need not be cryptographically collision-resistant. Specific algorithm choices (e.g., bit-width, hash function, float canonicalization, iteration ordering) are owned by specs/ADRs so they can evolve intentionally while preserving the concept of "StateDigest."*

### <a id="DM-0019"></a> DM-0019 — PlayerId
**Status:** Active  
**Tags:** identity, determinism, authority

**Definition:** A stable, per-Match participant identifier used to **deterministically order and attribute inputs**, and to associate a participant with their controlled entity(ies). PlayerId is assigned and owned by the Server Edge (DM-0011) as part of Session (DM-0008) management and is unique within a Match (DM-0010).

PlayerId is simulation-facing identity, not authentication:
- The Simulation Core (DM-0014) MUST treat PlayerId as an indexing/ordering key only (e.g., input attribution, deterministic input ordering, deterministic player↔entity association).
- The Simulation Core MUST NOT perform identity validation, authentication, or security checks based on PlayerId.
- Binding PlayerId ↔ Session/connection identity is exclusively the Server Edge’s responsibility.

Client-provided identity is not trusted:
- If a client supplies a PlayerId (explicitly or implicitly), the Server Edge MUST ignore/overwrite it with the server-assigned PlayerId before the input reaches the Simulation Core.

*Non-normative note: PlayerId is intentionally separate from Session identity, which is transport/security-facing and may change due to reconnects or connection churn.*

### <a id="DM-0020"></a> DM-0020 — EntityId
**Status:** Active  
**Tags:** identity, entity, determinism

**Definition:** A unique identifier for an Entity (DM-0005) within a Match (DM-0010), assigned by the Simulation Core (DM-0014). EntityId uniquely identifies an entity for its lifetime within the match and is used for stable reference in state serialization (e.g., Snapshots (DM-0007), Baselines (DM-0016), ReplayArtifact (DM-0017)).

EntityId is determinism-relevant:
- Any allocation/assignment strategy for EntityId MUST be deterministic under identical initial state, inputs, and parameters.
- When canonical ordering of entities is required (e.g., StateDigest (DM-0018) computation), EntityId provides a stable ordering key.

*Non-normative note: EntityId is scoped to a single match. It may be reused across different matches, but within a ReplayArtifact (DM-0017) the match scope is established, so EntityId references are unambiguous for reproduction and verification.*
*Non-normative note: Common deterministic allocation strategies include sequential allocation (counter-based) or derivation from deterministic spawn events. The strategy is a spec/implementation detail; the requirement is deterministic reproducibility under replay.*

### <a id="DM-0021"></a> DM-0021 — MatchId
**Status:** Active  
**Tags:** identity, traceability, orchestration

**Definition:** A stable identifier assigned by the Server Edge (DM-0011) at Match (DM-0010) creation that uniquely identifies the Match within the Server Edge's operational scope for the retention period of match-scoped artifacts. MatchId is used as the primary correlation key to associate and retrieve match-scoped records (e.g., replay artifact storage paths, logs, telemetry) and to prevent cross-match identifier collisions in storage and diagnostics.

**Normative constraints:**
- MatchId MUST be assigned and owned by the Server Edge.
- MatchId MUST remain stable for the full Match lifecycle.
- MatchId MUST be collision-resistant for concurrently active matches within the operational scope.
- MatchId MUST NOT be treated as a Simulation Core (DM-0014) input and MUST NOT influence authoritative outcomes.

*Non-normative note: In v0, replay artifacts are persisted under a MatchId-keyed path (e.g., `replays/{match_id}.replay`). MatchId exists to provide a durable match-scope handle; it is distinct from Session (DM-0008) and PlayerId (DM-0019). "Operational scope" refers to the deployment/infrastructure context in which the Server Edge operates (e.g., a single server process, a server pool, or a datacenter region).*

### <a id="DM-0022"></a> DM-0022 — InputTickWindow
**Status:** Active  
**Tags:** security, input

**Definition:** A server-defined, tick-indexed acceptance window that determines which InputCmd (DM-0006) ticks are eligible to be accepted (buffered/coalesced) by the Server Edge (DM-0011) at a given authoritative Tick (DM-0001). Inputs outside this window are rejected or clamped per the governing spec.

**Normative constraints:**
- The Server Edge MUST define InputTickWindow relative to its authoritative current tick (e.g., `[current_tick, current_tick + MAX_FUTURE_TICKS]`), and MUST apply it consistently for validation.
- InputTickWindow MUST be expressed purely in terms of tick indices (not wall-clock time) to preserve determinism and replay verifiability.
- The Simulation Core MUST NOT depend on InputTickWindow; it only consumes the applied, per-tick inputs selected by the Server Edge.

*Non-normative note: InputTickWindow provides a stable term for policies like “max future ticks,” client tick clamping guidance, and input acceptance tests without freezing specific constants in the Constitution.*

### <a id="DM-0023"></a> DM-0023 — LastKnownIntent (LKI)
**Status:** Active  
**Tags:** input, determinism, networking

**Definition:** A deterministic Server Edge (DM-0011) fallback concept representing the most recently accepted per-player intent (or intent components) available as of a given Tick (DM-0001). When an InputCmd (DM-0006) for player P at tick T is not available to be applied at the tick boundary, the Server Edge MAY derive the applied per-tick input for P at T using LastKnownIntent, according to the governing spec-level policy.

**Normative constraints:**
- Any use of LastKnownIntent MUST be deterministic and tick-indexed (no wall-clock dependence).
- LastKnownIntent derivation and application MUST be owned by the Server Edge; the Simulation Core (DM-0014) consumes only the applied per-tick inputs it is given.
- Replay verifiability MUST be preserved: the AppliedInput (DM-0024) stream (including any inputs derived via LastKnownIntent) MUST be reproducible from the ReplayArtifact (DM-0017), either by recording the AppliedInput values directly or by recording sufficient rule/version information and state to reconstruct them exactly.

*Non-normative note: LastKnownIntent names the stable concept “server-side deterministic fallback for missing per-tick input.” The specific behavior (e.g., hold-last vs. neutral intent, decay rules, per-component handling, interaction with prediction) is spec-level policy and may evolve post-v0.*

### <a id="DM-0024"></a> DM-0024 — AppliedInput
**Status:** Active  
**Tags:** protocol, determinism, replay

**Definition:** A tick-indexed, per-player input value selected and/or derived by the Server Edge (DM-0011) for application to the Simulation Core (DM-0014) during the Tick (DM-0001) transition T → T+1. AppliedInput is the canonical "input truth" recorded for replay/verification purposes and converted to StepInput (DM-0027) for Simulation Core consumption.

**Normative constraints:**
- For each participating PlayerId (DM-0019) and each Tick T processed by the Server Edge, exactly one AppliedInput value MUST be produced for application at Tick T (including the "no input received" case, which uses LastKnownIntent (DM-0023) fallback).
- AppliedInput MUST be derived deterministically from:
  - the set of received InputCmd (DM-0006) values targeting Tick T (if any),
  - the server's validation/normalization rules, and
  - LastKnownIntent (DM-0023) fallback when no valid input is available for Tick T.
- If multiple InputCmd values exist for the same (PlayerId, Tick T), the selection of the winning InputCmd MUST be deterministic and MUST NOT depend on runtime artifacts such as packet arrival order. Acceptable deterministic tie-breakers include InputSeq (DM-0026) or equivalent mechanisms.
- AppliedInput MUST NOT be raw client messages; it is the post-normalization result produced by the Server Edge.
- AppliedInput MUST be the only input stream recorded into ReplayArtifact (DM-0017) for verification (INV-0006).
- The Server Edge MUST convert AppliedInput to StepInput (DM-0027) before invoking the Simulation Core.

### <a id="DM-0025"></a> DM-0025 — TargetTickFloor
**Status:** Active  
**Tags:** protocol, networking, state-sync

**Definition:** A tick index emitted by the Server Edge (DM-0011) that indicates the earliest Tick (DM-0001) a Game Client (DM-0015) MUST target when tagging new InputCmd (DM-0006) messages.

TargetTickFloor is a protocol-level floor that reduces late-input drops and eliminates startup ambiguity. Clients MUST clamp InputCmd.tick upward to at least TargetTickFloor.

**Normative constraints:**
- TargetTickFloor MUST be derived solely from server-observable state and configuration (e.g., current server tick, InputTickWindow (DM-0022), configured lead/buffer policy).
- TargetTickFloor MUST be included in ServerWelcome and in each Snapshot (DM-0007).
- TargetTickFloor MUST be monotonic non-decreasing per Session (DM-0008); resets on session re-establishment or new MatchId (DM-0021).
- Game Clients MUST NOT target InputCmd.tick values earlier than TargetTickFloor.

*Non-normative note: TargetTickFloor is server-guided input targeting. It avoids stale client tick observations being treated as authoritative targeting information. The floor advances with the server tick; clients always target at or above it.*

### <a id="DM-0026"></a> DM-0026 — InputSeq
**Status:** Active  
**Tags:** protocol, determinism, networking

**Definition:** A per-session, strictly monotonically increasing `u64` sequence number attached to InputCmd (DM-0006) that allows the Server Edge (DM-0011) to deterministically select among multiple InputCmd values that target the same (PlayerId (DM-0019), Tick (DM-0001)).

**Normative constraints:**
- InputSeq MUST be a `u64` value.
- InputSeq MUST be scoped to a Session (DM-0008). It resets when a session is re-established.
- InputSeq MUST be strictly increasing (no duplicates) for each player session across transmitted InputCmd messages.
- When multiple InputCmd messages are received for the same (PlayerId, Tick), the Server Edge MUST select the InputCmd with the greatest InputSeq value. InputSeq ties are impossible if clients behave correctly; if a malformed client sends duplicate InputSeq values, the Server Edge MAY drop duplicates or use any deterministic tie-breaker (e.g., first-processed wins).
- InputSeq MUST NOT be interpreted by the Simulation Core (DM-0014); it is used only for Server Edge selection/derivation of AppliedInput (DM-0024).

*Non-normative note: InputSeq makes "latest wins" deterministic and independent of packet arrival order. On transports with guaranteed ordering (e.g., ENet sequenced channels), InputSeq is defense-in-depth. On unordered transports (e.g., WebTransport datagrams), InputSeq becomes required for deterministic selection. The u64 range (2^64 values) is effectively unlimited for practical session durations.*

### <a id="DM-0027"></a> DM-0027 — StepInput
**Status:** Active  
**Tags:** simulation, determinism

**Definition:** A simulation-plane, per-player input value consumed by the Simulation Core (DM-0014) during a single tick step. StepInput is the Simulation Core's interface type for inputs; it is structurally equivalent to the input-relevant fields of AppliedInput (DM-0024) but exists in the simulation plane, not the protocol plane.

**Normative constraints:**
- StepInput MUST be the only input type consumed by Simulation Core stepping functions.
- StepInput MUST NOT carry protocol-plane metadata (session info, sequence numbers, network timestamps).
- StepInput MUST carry: PlayerId (DM-0019) for deterministic ordering/attribution, and the input payload (e.g., movement direction).
- The Server Edge (DM-0011) MUST convert AppliedInput to StepInput before invoking the Simulation Core.

*Non-normative note: The separation between AppliedInput (protocol plane) and StepInput (simulation plane) preserves the Simulation Core boundary (INV-0004). AppliedInput is what the Server Edge selected; StepInput is what the Simulation Core consumes. They are structurally similar but semantically distinct.*

## Domain Model Change Policy

- Adding a domain concept is straightforward; define clearly and concisely.
- Changing or removing a domain concept may require updating dependent specs and code.
