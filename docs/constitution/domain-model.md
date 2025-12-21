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

**Definition:** A tick-indexed serialization of authoritative world state produced at a specific Tick (DM-0001) by the authoritative simulation, transmitted to clients by the I/O Boundary (DM-0011) for state synchronization.

*Non-normative note: Snapshots are authoritative. Clients use them for reconciliation and remote entity interpolation. Snapshot packing strategies (priority-based, budget-aware) are implementation details. The simulation produces snapshot data; the I/O Boundary handles transmission.*

### <a id="DM-0008"></a> DM-0008 — Session
**Status:** Active  
**Tags:** networking, control-plane, connection

**Definition:** A client's connection lifecycle from handshake through disconnect, including assigned player identity and synchronization state. Sessions are control-plane owned and not part of simulation state.

*Non-normative note: The simulation knows about player inputs and entity state. The I/O Boundary (DM-0011) manages session lifecycle (connections, authentication, tokens). Don't conflate simulation state with session state.*

### <a id="DM-0009"></a> DM-0009 — Channel
**Status:** Active  
**Tags:** networking, transport, protocol

**Definition:** A logical communication lane with defined delivery semantics (reliability, ordering, sequencing), independent of transport implementation. Examples: Realtime (unreliable + sequenced), Control (reliable + ordered), Bulk (reliable + non-blocking).

*Non-normative note: ENet channels, WebTransport streams/datagrams, and future transports all map to this semantic model. The channel abstraction is transport-agnostic.*

### <a id="DM-0010"></a> DM-0010 — Match
**Status:** Proposed  
**Tags:** orchestration, control-plane, replay

**Definition:** A discrete game instance with a defined lifecycle (create → active → end), a fixed simulation tick rate, an initial authoritative state, and a set of participating Sessions (DM-0008). Match is the scope boundary for gameplay, replay artifacts, and outcome determination.

*Non-normative note: A Match corresponds to "one game" from a player's perspective. It has a stable tick rate (see [docs/networking/v0-parameters.md](../networking/v0-parameters.md) for v0 values), a known start state, and produces a complete input+output history for replay (INV-0006). Session management is control-plane logic; the simulation sees player inputs and produces world state. Match is the conceptual glue between them.*

### <a id="DM-0011"></a> DM-0011 — I/O Boundary
**Status:** Active  
**Tags:** networking, control-plane, architecture

**Definition:** The in-process Control Plane component that owns sockets/transports, validates inputs, and exchanges tick-indexed messages with the Simulation Plane. Responsible for all external I/O and converting it into serializable, tick-indexed messages.

*Non-normative note: The I/O Boundary is the runtime interface between the outside world and the deterministic simulation core. It handles networking, session lifecycle, snapshot transmission, and (in future) replay recording. It lives in the same process as the simulation but maintains a strict message-passing boundary per INV-0004.*

### <a id="DM-0012"></a> DM-0012 — Orchestration Service
**Status:** Active  
**Tags:** orchestration, control-plane, preservability

**Definition:** An optional Control Plane component, typically external, providing matchmaking, lobbies, authentication, or server provisioning. Must not contain game rules. May not exist in LAN/dev scenarios. Orchestration Services are outside the game server's I/O Boundary and must not inject authoritative simulation inputs except through the same tick-indexed message interface used for Sessions.

*Non-normative note: Examples include a matchmaking service that pairs players, a lobby server that manages pre-game chat, or a provisioning system that spins up game server instances. These services may be separate processes, separate binaries, or entirely absent for local/LAN play. They communicate with game servers via standard session/network pathways, never by directly manipulating simulation state.*

## Domain Model Change Policy

- Adding a domain concept is straightforward; define clearly and concisely.
- Changing or removing a domain concept may require updating dependent specs and code.
