# Sprint 0 Research Report

## Intents Reviewed
Sprint 0 created every chapter below by transcribing the owner-supplied deep
research (see Artifacts) into typed intents. Chapters marked **selected** are
the ones this sprint advances.

- [INT-0001](../../../intents/INT-0001-periapsis-charter.md) — created, **selected**; relevance: root charter and intent map; the sprint starts its Phase 0; current state: proposed
- [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md) — created, **selected**; relevance: typed-acceptance grammar used by every chapter; the sprint adds its validator; current state: proposed
- [INT-0003](../../../intents/INT-0003-engineering-baseline.md) — created, **selected**; relevance: workspace guardrails the first crates must satisfy; current state: proposed
- [INT-0004](../../../intents/INT-0004-windows-and-linux.md) — created, **selected**; relevance: every result must hold on Windows and Linux; current state: proposed
- [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) — created; relevance: needed from the first storage code (next sprint), not by pure encoding; current state: proposed
- [INT-0006](../../../intents/INT-0006-format-v1.md) — created, **selected**; relevance: hash agility and canonical encoding are the base of every object; current state: proposed
- [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) — created; relevance: next storage slice; current state: proposed
- [INT-0008](../../../intents/INT-0008-compact-history-storage.md) — created; relevance: storage representation, zstd licence/purity tension; current state: proposed
- [INT-0009](../../../intents/INT-0009-reversible-by-default.md) — created; relevance: op-log design constraint; current state: proposed
- [INT-0010](../../../intents/INT-0010-lock-free-concurrency.md) — created; relevance: first formalisation target; current state: proposed
- [INT-0011](../../../intents/INT-0011-change-identity-and-rewriting.md) — created; relevance: Phase 2; current state: proposed
- [INT-0012](../../../intents/INT-0012-conflicts-and-merge.md) — created; relevance: Phase 2; current state: proposed
- [INT-0013](../../../intents/INT-0013-derived-data.md) — created; relevance: keys derived data by algorithm version, which format v1 must allow; current state: proposed
- [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) — created; relevance: Phase 1; current state: proposed
- [INT-0015](../../../intents/INT-0015-phases-and-confirmation.md) — created; relevance: Phase 3; current state: proposed
- [INT-0016](../../../intents/INT-0016-actors-and-agent-lifecycle.md) — created; relevance: signature payload rules reserved in format v1 (INT-0006 AC-7); current state: proposed
- [INT-0017](../../../intents/INT-0017-event-records-and-book-view.md) — created; relevance: the Book-parsing validator is the seed of its importer; current state: proposed
- [INT-0018](../../../intents/INT-0018-proof-pairing-certificates.md) — created; relevance: Lean toolchain not yet installed; current state: proposed
- [INT-0019](../../../intents/INT-0019-cheat-resistant-verifier.md) — created; relevance: Phase 0 spec work after the Lean toolchain decision; current state: proposed
- [INT-0020](../../../intents/INT-0020-intent-calculus.md) — created; relevance: kernel semantics behind INT-0002; current state: proposed
- [INT-0021](../../../intents/INT-0021-lanes-and-review-requests.md) — created; relevance: Phase 3; current state: proposed
- [INT-0022](../../../intents/INT-0022-agent-and-human-interfaces.md) — created; relevance: Phase 3; current state: proposed
- [INT-0023](../../../intents/INT-0023-query-language.md) — created; relevance: Phase 2; current state: proposed
- [INT-0024](../../../intents/INT-0024-staged-sync.md) — created; relevance: Phase 4; current state: proposed
- [INT-0025](../../../intents/INT-0025-git-bridge.md) — created; relevance: Phase 4, prerequisite of INT-0027; current state: proposed
- [INT-0026](../../../intents/INT-0026-sprint-loops-on-periapsis.md) — created; relevance: self-hosting milestone; current state: proposed
- [INT-0027](../../../intents/INT-0027-port-sprint-loops-projects.md) — created; relevance: owner-requested port path and Sprint Loops change; current state: proposed
- [INT-0028](../../../intents/INT-0028-entropy-layer.md) — created; relevance: records the abandoned entropy layer so it is not re-added; current state: abandoned
- [INT-0029](../../../intents/INT-0029-reproducible-releases.md) — created; relevance: Phase 5; current state: proposed
- [INT-0030](../../../intents/INT-0030-optional-scale-mode.md) — created; relevance: Phase 5, lowest priority; current state: proposed
- [INT-0031](../../../intents/INT-0031-visual-identity.md) — created; relevance: owner's README branding intent; current state: proposed

## 1. Sprint Goal
Seed the Book with the owner's research as rigorous, typed intents (done in
this phase), then begin Phase 0 in Rust: a Cargo workspace that meets the
engineering baseline (INT-0003) on Windows and Linux (INT-0004), holding the
first two format-v1 crates — `peri-hash` (algorithm-tagged `HashRef`,
BLAKE3-256) and `peri-encoding` (canonical, versioned binary encoding) —
specified in `docs/spec/format-v1.md` and pinned by golden files (INT-0006),
plus a `peri-book` typed-acceptance validator that checks this Book's own
chapters (INT-0002). Out of scope: storage, the op log, Lean, networking and the
Sprint Loops upstream change.

## 2. Existing Code Survey
| File | Relevance | Notes |
|------|-----------|-------|
| `README.md` | high | Product statement (Rust, agent-human hybrid DVCS; Sprint Loops, CubiKan, array-test as concept sources) and the branding intent → INT-0001, INT-0031 |
| `LICENSE` | medium | GPL-3.0 text; `-only` vs `-or-later` not stated → INT-0003 Consequences |
| `.gitignore` | low | GitHub Rust template plus the sprint-loops block; `target/` already ignored |
| `docs/sprints/s0/sprint-research/master-report-v6.md` | high | Primary research input, Parts I–VIII; source of 29 of 31 chapters |
| `docs/sprints/s0/sprint-research/biblio-report.md` | high | Identity, lineage, sunset and Rust auditability findings → INT-0003, INT-0016, INT-0029 |
| `Animus_Periapsis_Round5_AntiCheat_IntentTypes_RR.md` (removed) | low | Diffed against master report Part VIII: no non-heading differences, so removed as a duplicate (recoverable from `da806b1`) |
| `docs/work/remote-profile.md` | medium | provider github, base `main`, work `dev`, merge policy human-approve |
| `.github/dependabot.yml` | low | Generated for the `github-actions` ecosystem only; add `cargo` when the workspace lands |
| installed bundle `schemas/intent.md`, `scripts/check-book.sh` | high | The validator checks header fields and states only and allows extra content, so typed criteria are Book-v2 compatible |
| `~/Animus_Sprint_Loops` (Book v2, sprint 22) | high | Corpus for the INT-0002 AC-4 port-rule check and the INT-0027 importer; home of the upstream schema change |
| `~/Animus_Sprint_Loops/ROADMAP.md` | low | array-test integration is gated on array-test's own engine; confirms confirmation roots are not yet a usable dependency |
| `~/CubiKan/README.md` | medium | Intent units with immutable origins, revisioned transitions and stale-revision rejection → INT-0017 collision policy |

## 3. External Sources
- [BLAKE3 official test vectors](https://github.com/BLAKE3-team/BLAKE3/blob/master/test_vectors/test_vectors.json) — fetched 2026-10-31; inputs are a repeating 0…250 byte pattern; the first 32 bytes of each `hash` are the standard digest (empty input: `af1349b9…41f3262`). Used for INT-0006 AC-4.
- [blake3 1.8.7 on crates.io](https://crates.io/crates/blake3) — the `pure` feature disables the C and assembly backends; licence CC0-1.0 OR Apache-2.0 OR Apache-2.0 WITH LLVM-exception (GPL-3.0 compatible).
- [proptest 1.12.0 on crates.io](https://crates.io/crates/proptest) — property testing for the round-trip and totality criteria; MIT OR Apache-2.0; MSRV 1.88.
- [ed25519-dalek 3.0.0 on crates.io](https://crates.io/crates/ed25519-dalek) — pure-Rust Ed25519 (BSD-3-Clause) for INT-0016 in a later sprint; recorded now because format v1 reserves signature algorithm ids.

## 4. Risks, Unknowns, Dependencies
- **Risk:** the prototypes and experiment scripts the reports cite (`prototypes/peri-merge`, `peri-store`, `peri-content`, `peri-proof`, `peri-verify`, `experiments/`) are not in this repository. Their numbers become thresholds in the intents, but they cannot be re-run until re-implemented.
- **Risk:** golden files are byte-exact, while Git on Windows may convert line endings. Mitigation: store golden vectors as binary files marked `-text` in `.gitattributes`.
- **Risk:** the reference zstd library is C, which conflicts with INT-0003 AC-6 for INT-0008. Not this sprint's problem, but it must be decided before Phase 1 storage.
- **Risk:** Windows has no POSIX directory fsync, so crash-safety evidence differs by platform (INT-0004 AC-5, INT-0007).
- **Unknown:** the SPDX licence expression. Crate metadata uses `GPL-3.0-only` until the owner decides.
- **Unknown:** Nighthawk's toolchain. Tailscale SSH there needs the owner to approve a browser check first (attempted 2026-10-31).
- **Dependency:** Lean 4 is not installed on the Windows host or in WSL. INT-0018, INT-0019 and INT-0020 wait on the owner's approval to install elan and a pinned Lean (≥ 4.33.1).
- **Dependency:** Rust 1.99.1 stable is present on Windows and in WSL (Ubuntu 26.04.1); cargo-deny 0.20.2 is installed on Windows; cargo-vet and cargo-nextest are not.
- **Dependency:** GitHub Actions runs only once `dev` is pushed at the sprint checkpoint, so this sprint's authoritative test evidence is the local Windows and WSL runs.
- **Dependency:** Sprint Loops bundle 0.22.0 (Book schema v2, substrate contract 4).

## 5. Recommended Approach
Primary: build the Phase-0 Rust foundation in dependency order. (1) Workspace
and guardrails: edition 2024, resolver 3, workspace lints forbidding `unsafe`,
a pinned `rust-toolchain.toml`, a committed lockfile, `deny.toml` and
`.gitattributes`. (2) `peri-hash`: `HashAlg` with one-byte ids, a
length-checked `HashRef`, and BLAKE3-256 via `blake3` with `pure`, tested
against the official vectors. (3) `peri-encoding`: canonical primitives —
minimal LEB128 integers, length-prefixed bytes, UTF-8 strings, sorted unique
maps, `HashRef`, a version byte and rejection of trailing bytes — with
round-trip and totality property tests. (4) `docs/spec/format-v1.md` plus a
binary golden corpus checked on both platforms. (5) `peri-book`: a line-based
parser for the `## Acceptance criteria` section that skips fenced code,
validates the INT-0002 grammar, applies the port rule to untyped criteria and
computes certified/refuted/undetermined status. (6) CI for `windows-latest`
and `ubuntu-latest`: fmt, clippy, `test --locked`, deny and the Book check.
(7) `docs/prior-art.md` and a README that points at the Book.

Alternative considered: a Lean-first sprint (intent-calculus library and
verifier skeleton). Not chosen: the toolchain needs owner approval, and every
later layer depends on the format foundation anyway. A storage-first sprint
(`peri-sim` plus a crash-tested store) is the natural Sprint 1, because it
needs encoding and hashing first. The Sprint Loops typed-acceptance change
belongs to the Animus_Sprint_Loops repository's own loop; this sprint supplies
its grammar (INT-0002) and a reference validator to port.

Rationale: this follows the plan's Phase 0 order (spec, hash agility, then the
store), is small enough to verify completely on two platforms in one sprint,
and turns the Book's own typed criteria into something a machine checks from
the first sprint on.

## Artifacts
- `master-report-v6.md` — the owner-supplied combined deep-research report (Parts I–VIII), moved here from the repository root with its history.
- `biblio-report.md` — the owner-supplied Biblio research report, moved here from the repository root.
- Removed duplicate: `Animus_Periapsis_Round5_AntiCheat_IntentTypes_RR.md` matched master report Part VIII apart from heading levels (a diff ignoring headings and blank lines was empty); it remains in Git history at `da806b1`.
- BLAKE3 test vectors: not stored as a file; the subset used is embedded in the `peri-hash` tests and credited in `docs/prior-art.md`.
