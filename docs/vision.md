# Project Flowstate — Game Definition Specification (GDS)

This document is descriptive and non-authoritative. The Constitution overrides all interpretations.

## High-level vision

Flowstate is a competitive, skill-expressive, top-down multiplayer action game where **movement mastery** and **precise aim** are inseparable.

## Player experience

### Player interface

* Primary control scheme: **WASD movement + mouse cursor aiming**.
* Manual aim is the default and the priority; controller support is intended, with conservative assist where appropriate.

### Camera and aim intent (presentation and input)

Key terms:
- **Aim Intent**: the player’s aiming meaning (e.g., “toward that direction”), independent of camera/UI; typically represented as Aim Direction.
- **Aim Direction**: the direction the character is aiming, expressed in gameplay terms (not screen terms).
- **Target Intent**: a chosen target entity (only for abilities that require one), separate from aim direction.

Description:

* The world is **3D** with an **overhead, tilted top-down** camera.
* Primary case (player’s character alive and under control):
  * The camera generally centers on the **character**, but may offset toward the player’s aim indicator (cursor on mouse/keyboard; aim-stick direction on controller) to increase forward visibility.
  * This offset is a presentation aid, not a gameplay rule.
  * The camera supports edge panning (mouse), configurable offset, and constrained zoom.
  * The camera follows vertical character motion, with vertical movement **bounded** to preserve readable framing.

* Aiming is intent-driven:
  * Mouse/keyboard: the cursor expresses Aim Intent.
  * Controller: the aim-stick expresses Aim Intent (already close to Aim Direction form).
  * **Aim Direction is the default representation for aiming.** Some actions may additionally use an Aim Point as a helper reference.
  * The authoritative simulation consumes **player intent** and applies game rules; it does not depend on cameras, cursors, screen space, or UI.

* Aim Point (optional helper reference, when needed):
  * Aim Point is a **client-side reference** used to express certain actions; it is not itself authoritative.
  * Aim Point is never required to express basic aim intent; it exists only for actions that explicitly benefit from a point reference.
  * Aim Point is derived from **Aim Surfaces** (designated ground-like surfaces used for stable aiming references) so it remains predictable in a 3D world with verticality.
  * Occluders (walls/trees/props/characters) affect **outcomes and validity** (blocking, line-of-sight, collision), not the player’s aim meaning.
  * If no Aim Surface is available (e.g., aiming over a gap/abyss), the client derives a predictable fallback reference to keep Aim Point stable.
  * The authoritative simulation applies the ability’s explicit rules (range, line-of-sight, blocking, collision, etc.) to the player’s intent and any point reference the action provides.

* Free-aim and target intent:
  * Free-aim actions use Aim Direction directly (no target required).
  * Target-required actions use **Target Intent**; the simulation validates whether the target is legal under the ability’s rules.
  * Target Intent is conceptually separate from Aim Point (a surface reference).

- Different clients may capture aim differently (cursor vs aim-stick), but gameplay remains consistent because the simulation consumes intent—not camera or UI artifacts.

## Gameplay rules

### Movement kernel

#### Locomotion modes

Characters operate in an explicit **Locomotion Mode** that defines how movement behaves and what constraints apply. Locomotion mode may also constrain which abilities are usable and how they behave.

Core locomotion modes:
* **Grounded**
* **Airborne** (jumping/falling)
* **Gliding** (controlled aerial traversal)
* **Rail** (movement is guided by the rail; the player does not directly steer locomotion while on-rail)

#### Locomotion transitions (current model)
* **Grounded → Airborne (Jump):** Jump applies an upward impulse; gravity pulls the character back down.
* **Airborne → Gliding:** Gliding can only be entered from airborne.
* **Gliding → Airborne (Cancel):** Gliding can be cancelled at any time by releasing the glide input.
* **Airborne/Gliding → Grounded (Land):** Contact with the ground immediately enters grounded.
* **Grounded → Rail (Enter rail):** The character enters rail mode by making contact with a rail while grounded.
* **Rail → Grounded (Rail end):** Reaching the end of a rail path returns the character to grounded.
* **Rail → Airborne (Rail jump):** The character can jump from a rail into airborne.

#### Movement mastery

Movement mastery is expressed through **route choice, angle choice, timing, and geometry use** under pressure—not just raw speed.

### World (primitives)

* **World**: the playable map space—terrain, obstacles, surfaces, and traversal features—that shapes movement and combat.
* The world includes **solid geometry** that characters and many ability effects can collide with or be blocked by, creating meaningful **angles, routes, and line-of-sight breaks**.
* The world supports **blocking and line-of-sight breaks** as a core interaction, shaping positioning and approach.
* The world includes recognizable positional structures such as **obstacles**, **open lanes**, **chokepoints**, and **traversal features**.
* **Verticality** matters insofar as it changes traversal options and moment-to-moment readability (e.g., airborne/gliding arcs, drops, and height-based approach lines).
* **Rails** are world traversal features that enable the **Rail** locomotion mode, trading direct locomotion control for guided movement along a path.

### Character state (foundational)

A character’s available actions and interactions depend on their current state, including locomotion mode and temporary conditions.

This state model is used to express common competitive situations such as being **constrained**, **committed**, or **temporarily modified**, without requiring bespoke special-case rules.

* **Control state:** whether the player has direct control of movement/actions, or the character is temporarily constrained.
* **Action state:** whether the character is free to act, or committed to an action phase (e.g., casting/commitment windows).
* **Condition state:** temporary modifiers that change what the character can do or how they interact with others.

### Combat and abilities (interaction language)

Abilities are a primary vehicle for skill expression. They vary by how they are aimed, how they reach their destination, and how they commit.

#### Targeting and delivery taxonomy

Ability behavior can be described along two axes:

* **Target acquisition** (how a cast selects a target, if any)

  * **Free-aim**: no target; the action uses an aim direction/point.
  * **Target-required**: activation requires a valid target under the cursor.
  * **Controller assist (policy)**: on controller, some actions may use conservative assistance for selecting a valid target, scoped per action.
* **Effect delivery** (how the effect reaches space/target)

  * **Instant**: resolves immediately on commit.
  * **Traveling projectile**: resolves via collision during flight.
  * **Guided projectile**: travels toward a chosen target.

Common aiming and delivery patterns:

* **Instant free-aim** (immediate resolution at the aimed direction/point)
* **Traveling free-aim projectiles** (opponents can dodge during travel)
* **Target-required instant abilities** (must select a valid target under cursor)
* **Target-required guided projectiles** (lock onto a valid target and travel toward them)

#### Commitment and disruption

* Some abilities execute immediately; others have a **cast time**.
* Cast-time abilities may be **cancelled** by the caster before they complete.
* Cast-time abilities may be **interrupted** by defined interactions, preventing completion.

#### Ability authoring model (data-driven)

* Abilities are intended to be authored primarily as **structured definitions** that describe behavior clearly and consistently, rather than requiring bespoke logic for every new ability.
* These definitions capture the key behavioral dimensions needed to create new abilities reliably (aiming/targeting, delivery style, timing/commitment, effects, and constraints).
* New abilities should be creatable by composing supported behavior primitives, so the game can grow in breadth without redefining the core combat language.

#### Ability composition vocabulary (initial)

At a minimum, abilities are expected to be expressible in terms of:

* **Targeting and delivery** (free-aim vs target-required; instant vs traveling vs guided)
* **Spatial forms** (single-target, directional casts, and area-based effects)
* **Effects** (damage/healing, defensive effects, and temporary modifiers)
* **Movement and displacement** (self-mobility and moving/repositioning targets)
* **Persistent interactions** (briefly persistent fields, pulses, delayed triggers)
* **Spawned gameplay objects** (temporary objects that meaningfully interact with characters and space)

#### Constraints and gating

* Ability availability and behavior may depend on **character state**, including the current **Locomotion Mode**.
* Abilities may impose temporary movement constraints or change locomotion as part of their behavior.

#### Resources and costs (ability economy)

Ability usage is governed by a character-specific **Resource Profile**. Different characters may use different resource models without changing the core combat language.

Baseline:
- **Cooldown-only**: abilities are limited by cooldowns and state gating (no additional resource pool).

Optional resources (per character):
- **Mana**: a spendable pool that **regenerates passively over time** and may also be replenished by other explicit means (sources are character / mode dependent and not assumed by default).
- **Energy**: a spendable pool that **does not passively regenerate** and must be **earned through play** (typically by landing basic attacks; exact earn rules are character-specific).

Costs and timing semantics:
- **Eligibility check (start)**: an action cannot begin unless its requirements are satisfied at activation start (e.g., you cannot start a 50-mana cast with only 40 mana).
- **Reservation (start)**: when an action begins, its resource cost is reserved so it cannot be spent by other actions during windup.
- **Commit (later point)**: on commit, the reserved cost is consumed and any associated cooldowns begin.
- If a cast is cancelled or interrupted **before commit**, reserved costs are released (no consumption). If it commits, it pays.

Design intent:
- Resource rules must be predictable to the player (no hidden availability math), and consistent across the roster even when Resource Profiles differ.

#### Readability and counterplay

* Abilities should communicate intent and commitment clearly enough for opponents to respond through positioning, timing, and use of their own tools.
* High-impact actions should have legible tradeoffs (risk, windows of vulnerability, or other costs) appropriate to competitive play.

## Domain terms (minimal)

* **Character**: the entity the player controls.
* **Locomotion Mode**: the character’s current movement regime (grounded/airborne/gliding/rail).
* **Character state**: the current conditions that shape what a character can do (including control/action/condition state).
* **Control state**: whether the player has direct control of movement/actions, or the character is temporarily constrained.
* **Condition**: a temporary modifier that changes what a character can do or how they interact with others.
* **Ability**: a combat action a character can perform.
* **Cast**: a player’s attempt to use an ability (which may commit immediately or after a cast time).
* **Aim**: the player’s aim intent derived from cursor (or controller), represented as a stable direction/point relative to the character (not dependent on the camera).
* **World**: the playable map space (terrain, obstacles, traversal features) that shapes movement and combat.

## Reference games (for shared vocabulary)

### SUPERVIVE (reference only)

* Aerial traversal and “edge/abyss” risk economy
* Gliding as a mobility state with vulnerability windows
* High-speed fights where displacement and positioning matter

### Battlerite (reference only)

* Precise WASD + cursor-aim play
* Short, intense arena fights
* Readable combat language (projectiles, dodges, cooldown mindgames)
