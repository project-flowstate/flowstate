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

**Definition:** The playable map space—terrain, obstacles, surfaces, and traversal features—that shapes movement and combat.

### <a id="DM-0003"></a> DM-0003 — Character
**Status:** Active  
**Tags:** simulation, entity

**Definition:** The entity a player controls; has position, state, and can perform actions. The primary interactive agent in the simulation.

### <a id="DM-0004"></a> DM-0004 — Locomotion Mode
**Status:** Active  
**Tags:** movement

**Definition:** The character's current movement regime (Grounded, Airborne, Gliding, Rail). Defines how movement behaves and what constraints apply.

### <a id="DM-0005"></a> DM-0005 — Entity
**Status:** Active  
**Tags:** simulation, architecture, identity

**Definition:** An object in the simulation with a unique identity and simulation state (often including position). The base unit of dynamic game objects. A Character (DM-0003) is a kind of Entity. A World (DM-0002) contains Entities.

*Non-normative note: Entities include players, projectiles, obstacles, pickups, timers, triggers, etc. The Entity abstraction provides the common interface for identity, state management, and lifecycle. Not all entities are spatial (e.g., match timers).*

### <a id="DM-0006"></a> DM-0006 — InputCmd
**Status:** Active  
**Tags:** networking, input, protocol

**Definition:** A tick-indexed message containing one player's inputs (movement direction, aim, actions) for a specific Tick (DM-0001). One InputCmd per player per tick.

*Non-normative note: We chose "InputCmd" over "InputFrame" because it's per-player, not per-tick-aggregate. An InputFrame would mean "all players' inputs for tick T," which is a server-side collection concern, not a protocol primitive.*

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

**Definition:** A logical communication lane with defined delivery semantics (reliability, ordering, sequencing), independent of transport implementation. Examples: Realtime (unreliable + sequenced), Control (reliable + ordered), Bulk (reliable + non-blocking).

*Non-normative note: ENet channels, WebTransport streams/datagrams, and future transports all map to this semantic model. The channel abstraction is transport-agnostic.*

### <a id="DM-0010"></a> DM-0010 — Match
**Status:** Proposed  
**Tags:** orchestration, replay

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

*Non-normative note: The Simulation Core advances in discrete Ticks (DM-0001). It consumes validated Tick-indexed InputCmds (DM-0006) via the Server Edge (DM-0011), produces Baselines (DM-0016) and Snapshots (DM-0007), and maintains World state. If clients implement prediction/rollback, they MUST invoke the same Simulation Core logic (same rules/version) rather than duplicating gameplay math; client results remain non-authoritative and are reconciled to server snapshots. Isolation is enforced by INV-0004.*

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


## Domain Model Change Policy

- Adding a domain concept is straightforward; define clearly and concisely.
- Changing or removing a domain concept may require updating dependent specs and code.
