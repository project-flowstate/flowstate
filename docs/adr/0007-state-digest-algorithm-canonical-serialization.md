# ADR 0007: StateDigest Algorithm (v0)

## Status
Accepted

## Type
Technical

## Context
Replay verification (INV-0006) requires at least one deterministic verification anchor that can be recomputed during replay and compared against the recorded value. The Domain Model defines StateDigest (DM-0018) as a digest over a canonical serialization of authoritative World (DM-0002) state at a specific Tick (DM-0001), and requires a digest algorithm identifier so the procedure is unambiguous and versionable.

Today, the v0 spec contains a concrete digest algorithm definition. Keeping the concrete algorithm in a spec creates drift risk: future specs may restate (or subtly change) the algorithm, breaking replay verification and eroding determinism guarantees. This ADR centralizes the decision, provides an explicit algorithm identifier, and defines the versioning/change policy.

StateDigest is a regression/verification mechanism, not a security primitive; cryptographic collision resistance is not required for v0.

## Decision
Define **StateDigest v0** as a deterministic 64-bit digest computed by the Simulation Core over a canonical byte representation of World state at a specific Tick.

### Algorithm Identifier
The ReplayArtifact (DM-0017) MUST record a `state_digest_algo_id` identifying the exact algorithm/canonicalization used.

For v0, the required value is:

- `state_digest_algo_id = "statedigest-v0-fnv1a64-le-f64canon-eidasc-posvel"`

Any change to the procedure that could alter outputs MUST mint a new identifier (see “Change Policy”).

### Digest Algorithm (v0)
- **Hash:** FNV-1a 64-bit
  - offset basis: `0xcbf29ce484222325`
  - prime: `0x100000001b3`
- **Purpose:** determinism regression check only; collisions are an accepted risk for v0.

### Canonicalization Rules (v0)
Applied prior to hashing (ref: INV-0007):
- For all `f64` values:
  - `-0.0` MUST be canonicalized to `+0.0`
  - Any NaN MUST be canonicalized to the quiet NaN bit pattern `0x7ff8000000000000`

### Byte Encoding (v0)
- All integers and floats MUST be encoded as **little-endian** bytes.
- `f64` values MUST be encoded by their canonicalized IEEE-754 bit pattern.

### Included Data & Ordering (v0)
StateDigest(v0) hashes the following sequence of bytes:

1) `tick` as `u64` (little-endian)

2) For each entity, iterated in **EntityId (DM-0020) ascending order** (ref: INV-0007):
   - `entity_id` as `u64` (little-endian)
   - `position[0]` as `f64` (canonicalized, little-endian)
   - `position[1]` as `f64` (canonicalized, little-endian)
   - `velocity[0]` as `f64` (canonicalized, little-endian)
   - `velocity[1]` as `f64` (canonicalized, little-endian)

### Ownership
- The Simulation Core (DM-0014) MUST provide the canonical StateDigest computation.
- The Server Edge (DM-0011) MUST treat StateDigest as an opaque value and record it as part of ReplayArtifact verification anchors (DM-0017, INV-0006).

## Rationale
- **Single source of truth:** A concrete algorithm belongs in an ADR so specs can reference it without duplication.
- **Determinism and traceability:** Explicit canonicalization and deterministic ordering are required to make digest comparison meaningful (INV-0007).
- **Versionability:** An explicit `state_digest_algo_id` prevents ambiguity and enables controlled evolution without breaking old replays.
- **Simplicity for v0:** FNV-1a 64-bit is easy to implement, fast, and adequate as a regression anchor under v0 scope.

## Constraints & References (no prose duplication)
- Constitution IDs:
  - INV-0006 (Replay Verifiability)
  - INV-0007 (Deterministic Ordering & Canonicalization)
  - DM-0017 (ReplayArtifact)
  - DM-0018 (StateDigest)
  - DM-0020 (EntityId)
  - DM-0001 (Tick)
  - DM-0002 (World)
- Related ADRs:
  - ADR-0002 (Deterministic Simulation)
  - ADR-0003 (Fixed Timestep Simulation Model)

## Alternatives Considered
- **Cryptographic hash (e.g., SHA-256, BLAKE3):** stronger collision resistance than needed; higher implementation/CPU cost; not required for v0 regression anchoring.
- **Non-cryptographic hashes (xxHash, Murmur, etc.):** viable; FNV-1a chosen for simplicity and ease of auditing. If upgraded later, it MUST be done via a new `state_digest_algo_id`.
- **Digest computed outside Simulation Core:** rejected; increases drift risk and violates the principle that the Simulation Core owns canonicalization/verification semantics for authoritative state.

## Implications
- Replay artifacts MUST include `state_digest_algo_id` and the digest value(s) at declared checkpoint ticks.
- Any change to:
  - included fields,
  - ordering,
  - float canonicalization,
  - byte layout/endian,
  - or hash function parameters
  MUST mint a new `state_digest_algo_id`.
- v0 digest scope remains “same build/same platform” per INV-0006. Cross-platform determinism is deferred.

## Change Policy
- Changing the StateDigest procedure is a **compatibility event**.
- A change MUST be accompanied by:
  - a new `state_digest_algo_id`,
  - an ADR update (this ADR) documenting the new algorithm (or a superseding ADR),
  - and a replay/versioning strategy (e.g., replay format version bump if required).
- Replay verification MUST select the digest procedure based on `state_digest_algo_id` recorded in the ReplayArtifact.
