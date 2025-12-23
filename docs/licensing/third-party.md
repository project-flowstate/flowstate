# Third-Party Software & Asset Licensing Policy

This document defines the **licensing policy, intake process, and provenance requirements**
for all third-party software, libraries, tools, assets, and data used by the Flowstate project.

Flowstate itself is licensed under the **MIT License**.  
All third-party dependencies must be compatible with this goal and must not impose
restrictions that conflict with open-source redistribution, modification, or future commercialization.

This policy applies equally to:
- Code (runtime, build-time, tooling)
- Assets (models, textures, audio, animations)
- Data sets
- Documentation excerpts
- Generated content derived from third-party sources

## 1. Guiding Principles

1. **Permissive by default**  
   Prefer licenses that allow unrestricted use, modification, distribution, and sublicensing.

2. **No viral or reciprocal obligations**  
   Dependencies must not impose copyleft, reciprocal disclosure, or downstream licensing requirements.

3. **Clear provenance**  
   Every external component must have a clear, documented source and license.

4. **Future flexibility**  
   The project must remain free to:
   - Change architecture
   - Dual-license in the future if desired
   - Be used in commercial and non-commercial contexts
   - Be forked without legal friction

5. **Agent-operable**  
   The rules must be simple and unambiguous so that human and AI contributors can comply without guesswork.

## 2. Approved Licenses (Default Allowlist)

The following licenses are **explicitly permitted** without additional approval:

### Software / Code
- **MIT**
- **Apache License 2.0**
- **BSD 2-Clause**
- **BSD 3-Clause**
- **ISC**
- **Zlib**
- **Boost Software License 1.0**
- **Unlicense / CC0** (for code only)

### Assets / Content
- **CC0**
- **CC-BY 4.0** (attribution required; must be documented)
- **Public Domain** (verifiable)

These licenses are considered compatible with MIT and impose no unacceptable downstream restrictions.

## 3. Conditionally Allowed Licenses (Require Explicit Review)

The following licenses are **not automatically approved** but may be allowed with justification
and explicit acknowledgment in this document:

- **MPL 2.0** (file-level copyleft; must remain isolated)
- **CC-BY-SA** (generally discouraged; requires careful boundary isolation)
- **Dual-licensed projects** (MIT/Apache side must be clearly used)

If a dependency uses one of these licenses:
- An explicit entry **must** be added to the intake table (see Section 6)
- Architectural isolation **must** be documented if relevant

## 4. Disallowed Licenses (Not Permitted)

The following licenses are **explicitly prohibited** and must not be introduced into the project:

### Strong / Weak Copyleft
- **GPL (any version)**
- **AGPL (any version)**
- **LGPL (any version)**

### Source-Available / Restricted Use
- **BUSL**
- **SSPL**
- **Elastic License**
- **Commons Clause**
- Any license that restricts:
  - Field of use
  - Commercial use
  - Hosting / SaaS use
  - Modification or redistribution

### Ambiguous or Custom Licenses
- Custom licenses without legal review
- Licenses with unclear or unverifiable terms

## 5. Generated Content & AI Outputs

AI-generated code or assets are permitted **only if**:

- The model’s output license allows unrestricted use
- No training data restrictions propagate to outputs
- The contributor asserts (in the PR description) that the output is license-compatible

If provenance cannot be reasonably asserted, the content must not be merged.

## 6. Third-Party Intake Record (Required)

Every third-party dependency or asset must be recorded below.

| Name | Version / Commit | License | Source URL | Usage Scope | Notes |
|----|----|----|----|----|----|
| prost | 0.13 | Apache-2.0 | https://crates.io/crates/prost | Runtime dependency | Protobuf serialization for wire protocol |
| sha2 | 0.10 | MIT OR Apache-2.0 | https://crates.io/crates/sha2 | Runtime dependency | SHA-256 for build fingerprint |

**Usage Scope examples**
- Runtime dependency
- Build tool
- Dev-only tool
- Asset (model / texture / audio)
- Documentation reference

## 7. Attribution Requirements

If a license requires attribution (e.g. CC-BY, BSD-3-Clause):
- Attribution text must be preserved
- Attribution location must be documented (README, NOTICE file, in-game credits, etc.)

## 8. Enforcement

- Pull Requests introducing new dependencies **must** update this document.
- CI or reviewers may block merges if licensing information is missing or ambiguous.
- In case of uncertainty, default to **not merging** until clarified.

## 9. Changes to This Policy

This document is part of the project’s governance surface.
Changes must be made via Pull Request and approved by the project maintainer(s).

## 10. Summary (Non-Normative)

- Flowstate is MIT licensed.
- Permissive licenses are welcome.
- Copyleft and source-available licenses are not.
- Clear provenance is mandatory.
- When in doubt: ask, document, or don’t merge.
