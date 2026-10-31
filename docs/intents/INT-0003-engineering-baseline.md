# INT-0003 — Engineering baseline: from-scratch, auditable, locked Rust

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0003
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Periapsis's own code is 100% Rust in one Cargo workspace whose crates use the
`peri-` prefix (the binary is `peri`). Every crate forbids `unsafe` code; the
toolchain is pinned in `rust-toolchain.toml`; `Cargo.lock` is committed and CI
builds `--locked`; dependency licences, advisories and bans are enforced with
cargo-deny; formatting and Clippy are clean with warnings denied. Designs
borrowed from other systems are credited in `docs/prior-art.md`, and no code is
copied from them. Pure-Rust dependencies are preferred wherever one exists.

Boundaries: this intent covers the workspace, its guardrails and its
dependency policy. Reproducible, attested releases are
[INT-0029](INT-0029-reproducible-releases.md).

Non-goal: "100% Rust, verifiable down to the compiler". rustc's LLVM backend
and the independent mrustc bootstrap are C++, so that claim is not achievable
(Biblio §5).

## Acceptance criteria
- **AC-1** `necessary` `formal` — No workspace crate contains `unsafe` code.
  - Obligation: `unstated`.
  - Proxy: the workspace lint `unsafe_code = "forbid"` inherited by every crate, enforced by the compiler on every CI build.
- **AC-2** `necessary` `empirical` — Builds are pinned and locked: `rust-toolchain.toml` names an exact stable release, `Cargo.lock` is committed, and every CI build of the work branch runs with `--locked` on Windows and Linux.
  - Threshold: 100% of CI runs.
- **AC-3** `necessary` `empirical` — `cargo deny check` passes in CI: every dependency licence is on the GPL-3.0-compatible allowlist, no RustSec advisory is unaddressed and the ban policy holds.
- **AC-4** `necessary` `empirical` — `cargo fmt --check` and `cargo clippy --workspace --all-targets -- -D warnings` pass in CI on Windows and Linux.
- **AC-5** `necessary` `asserted` — `docs/prior-art.md` credits every adopted design with its source system and licence, and a reviewer confirms that no code was copied from another VCS project.
  - Signers: project owner or a human reviewer.
- **AC-6** `supporting` `empirical` — Every dependency able to compile C or assembly (each crate that has `cc` as a build-dependency, per `cargo tree -e build -i cc`) is either built with its pure-Rust option (for example `blake3` with `features = ["pure"]`) or listed with a justification in `docs/prior-art.md`.
- **AC-7** `supporting` `asserted` — Crate names under the `peri-` prefix are reserved on crates.io before first publication; bare `periapsis` and `peri` are taken (R1 #22).
  - Signers: project owner (publishing is an owner action).

## Rationale
- "Reproducible and bootstrappable" is the honest target: trusting-trust risk,
  a C++ independent compiler, and rustc reproducibility tracked as a
  never-closing issue all rule out the stronger claim, while Rust *programs*
  are often reproducible given a pinned toolchain and normalized paths
  (Biblio §5).
- Pure-Rust cryptography avoids C libraries (`secp256k1` wraps libsecp256k1;
  `k256` and Ed25519 implementations exist in pure Rust) (Biblio §5).
- Credit from day one: the Scion/Mononoke attribution dispute shows the cost of
  not doing it (II §2.3, corrected by R1 #21).
- Crate names: `periapsis` and `peri` are taken on crates.io; `peri-*` and
  `animus-periapsis` were free when checked (R1 #22).
- Sources: [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md)
  I.3 principles 8–9, II §4 Phase 0; [Biblio report](../sprints/s0/sprint-research/biblio-report.md) §5.

## Alternatives
- **cargo-vet audits alongside cargo-deny.** Deferred: cargo-deny covers
  licences, advisories and bans now; vet attestations can be added once the
  dependency set stabilises.
- **C-backed crates where faster** (for example C zstd or libsecp256k1).
  Allowed only as justified exceptions under AC-6, because each one widens the
  audit surface.

## Consequences
- Some pure-Rust implementations are slower or less mature than their C
  counterparts. The known tension is zstd (INT-0008): the reference library is
  C, so the storage work may need a recorded AC-6 exception.
- The repository LICENSE is the GPL-3.0 text, but `GPL-3.0-only` versus
  `GPL-3.0-or-later` is not stated. Crate metadata uses `GPL-3.0-only`, the
  narrower grant, until the owner decides.

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 I.5 Phase 0 and the Biblio report §5).
