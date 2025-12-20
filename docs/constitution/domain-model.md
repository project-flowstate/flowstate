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

## Domain Model Change Policy

- Adding a domain concept is straightforward; define clearly and concisely.
- Changing or removing a domain concept may require updating dependent specs and code.
