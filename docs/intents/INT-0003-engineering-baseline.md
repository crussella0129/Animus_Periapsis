# INT-0003 — Engineering baseline: from-scratch, auditable, locked Rust

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0003
- **State:** active
- **Work evidence:** [T-001 build plan](../sprints/s0/sprint-plans/build-plan.md#t-001-workspace-guardrails-policy-checker-and-credits), [T-007 build plan](../sprints/s0/sprint-plans/build-plan.md#t-007-canonical-suite-runner-and-ci-workflow), [T-008 build plan](../sprints/s0/sprint-plans/build-plan.md#t-008-readme-and-documentation-checks), [Sprint 1 T-009 build plan](../sprints/s1/sprint-plans/build-plan.md#t-009-record-sprint-0s-github-actions-runs-and-owner-sign-offs), [T-010 backlog](../work/tasks.md), [Sprint 1 T-003 build plan](../sprints/s1/sprint-plans/build-plan.md#t-003-filesystem-boundary-only-sims-real-half-touches-the-real-filesystem), [Sprint 1 T-008 build plan](../sprints/s1/sprint-plans/build-plan.md#t-008-cargo-ci-refuses-a-target-directory-outside-the-checkout), [T-010 backlog](../work/tasks.md), [Sprint 2 T-005 build plan](../sprints/s2/sprint-plans/build-plan.md#t-005-the-workspace-crate-init-snapshot-new-and-status), [Sprint 2 T-006 build plan](../sprints/s2/sprint-plans/build-plan.md#t-006-praps-init-praps-new-and-praps-status), [Sprint 2 T-008 build plan](../sprints/s2/sprint-plans/build-plan.md#t-008-working-copy-spec-readme-credits-and-backlog), [Sprint 2 T-001 build plan](../sprints/s2/sprint-plans/build-plan.md#t-001-record-sprint-1s-github-actions-runs-and-owner-sign-offs), [T-013 backlog](../work/tasks.md), [Sprint 3 T-001 build plan](../sprints/s3/sprint-plans/build-plan.md#t-001-record-sprint-2s-github-actions-runs-and-owner-sign-offs), [Sprint 3 T-002 build plan](../sprints/s3/sprint-plans/build-plan.md#t-002-ban-the-remaining-routes-to-operating-system-randomness), [Sprint 3 T-010 build plan](../sprints/s3/sprint-plans/build-plan.md#t-010-readme-credits-and-backlog), [T-017 backlog](../work/tasks.md), [Sprint 4 T-001 build plan](../sprints/s4/sprint-plans/build-plan.md#t-001-record-the-sprint-3-checkpoints-github-actions-runs-and-owner-sign-offs), [Sprint 4 T-004 build plan](../sprints/s4/sprint-plans/build-plan.md#t-004-the-points-policy-rule), [Sprint 4 T-008 build plan](../sprints/s4/sprint-plans/build-plan.md#t-008-reach-recording-cargo-reach-the-reach-step-in-cargo-ci-and-the-simulation-spec), [Sprint 4 T-010 build plan](../sprints/s4/sprint-plans/build-plan.md#t-010-consequences-credits-readme-and-backlog)
- **Completion evidence:** [T-001 completion](../work/completed-tasks.md#t-001-sprint-0), [T-007 completion](../work/completed-tasks.md#t-007-sprint-0), [T-008 completion](../work/completed-tasks.md#t-008-sprint-0), [Sprint 1 T-003 completion](../work/completed-tasks.md#t-003-sprint-1), [Sprint 1 T-008 completion](../work/completed-tasks.md#t-008-sprint-1), [Sprint 1 T-009 completion](../work/completed-tasks.md#t-009-sprint-1), [Sprint 2 T-001 completion](../work/completed-tasks.md#t-001-sprint-2), [Sprint 2 T-008 completion](../work/completed-tasks.md#t-008-sprint-2)
- **Code evidence:** [workspace manifest](../../Cargo.toml), [toolchain pin](../../rust-toolchain.toml), [deny.toml](../../deny.toml), [policy checker](../../crates/xtask/src/policy.rs), [canonical runner](../../crates/xtask/src/ci.rs), [CI workflow](../../.github/workflows/ci.yml)
- **Test evidence:** [Sprint 0 test report](../sprints/s0/sprint-tests/test-report.md), [Sprint 0 GitHub Actions record](../sprints/s1/sprint-tests/ci-sprint0.md), [Sprint 1 test report](../sprints/s1/sprint-tests/test-report.md), [Sprint 1 CI record](../sprints/s2/sprint-tests/ci-sprint1.md), [Sprint 2 test report](../sprints/s2/sprint-tests/test-report.md), [Sprint 2 CI record](../sprints/s3/sprint-tests/ci-sprint2.md), [Sprint 3 test report](../sprints/s3/sprint-tests/test-report.md), [Sprint 3 CI record](../sprints/s4/sprint-tests/ci-sprint3.md), [Sprint 4 test report](../sprints/s4/sprint-tests/test-report.md)
- **Documentation evidence:** [README](../../README.md), [prior art](../prior-art.md)

## Intent
Periapsis's own code is 100% Rust in one Cargo workspace. Crates have the
simplest names that say what they do — single lowercase words such as `hash`,
`codec`, `book` and `xtask`; the user-facing crate and its command are
`praps`, short for periapsis. Project tooling is just as plain: `cargo ci`,
`cargo policy`, `cargo book` and `cargo reach` are idempotent,
so they take no flags. Every crate forbids `unsafe` code; the toolchain is
pinned in `rust-toolchain.toml`; `Cargo.lock` is committed and CI builds
`--locked`; dependency licences, advisories and bans are enforced with
cargo-deny; formatting and Clippy are clean with warnings denied. Designs
borrowed from other systems are credited in `docs/prior-art.md`, and no code is
copied from them. Pure-Rust dependencies are preferred wherever one exists.

Boundaries: this intent covers the workspace, its naming, its guardrails and
its dependency policy. Workspace crates are `publish = false`. The user-facing
crate can be published as `praps`; a library that is ever published goes to
crates.io under a `praps-` prefixed name, because bare words such as `hash`
and `codec` are taken there. Reproducible, attested releases are
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
- **AC-6** `supporting` `empirical` — Every dependency able to compile C or assembly (each crate that depends on `cc`, per `cargo tree --workspace --target all -e normal,build -i cc`) is either built with its pure-Rust option (for example `blake3` with `features = ["pure"]`) or listed with a justification in `docs/prior-art.md`.
- **AC-7** `supporting` `asserted` — Before any crate is published, its package name is reserved on crates.io: `praps` for the user-facing crate and a `praps-` prefixed name for a library; bare `periapsis` and `peri` are taken (R1 #22).
  - Signers: project owner (publishing is an owner action).
- **AC-8** `necessary` `formal` — Every unpublished (`publish = false`) workspace package name is a single lowercase ASCII word matching `[a-z][a-z0-9]*`.
  - Obligation: `unstated`.
  - Proxy: the `names` rule of `cargo policy`.
- **AC-9** `necessary` `empirical` — The tooling commands `cargo ci`, `cargo policy`, `cargo book` and `cargo reach` are idempotent and take no flags: each rejects any argument starting with `-` with exit code 2, and two consecutive runs leave the working tree unchanged.
  - Threshold: 100% of tooling commands.
- **AC-10** `supporting` `empirical` — The README keeps the owner's original statement, links the Book, documents the commands `cargo ci`, `cargo policy`, `cargo book`, `cargo reach` and `cargo deny check`, and states that the four project commands (`cargo ci`, `cargo policy`, `cargo book`, `cargo reach`) take no flags; `cargo deny check` is a third-party tool, run exactly as shown.

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
- Crate names: `periapsis` and `peri` are taken on crates.io (R1 #22). On
  2026-10-31 the owner chose `praps`, short for periapsis, for the command and
  the user-facing crate: no crate had that name, and neither Ubuntu 26.04's
  package lists nor Homebrew had a `praps` package. Inside the workspace the
  owner asked for the simplest names possible and for flags only where a
  command is not idempotent (2026-10-31), so internal names stay bare and only
  a published library would need the `praps-` prefix.
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
- cargo-deny runs as its own command (`cargo deny check`) and its own CI job,
  not as a `cargo ci` step, so a machine without cargo-deny can still run the
  whole suite. A green local `cargo ci` therefore says nothing about AC-3; the
  deny job does.
- AC-2, AC-3 and AC-4 are claims about CI, so they hold only once a GitHub
  Actions run on the work branch is recorded. That run starts at a sprint's
  checkpoint, which opens only after the sprint closes, so each sprint's CI
  evidence is recorded by the next sprint as a carried-forward task.
- The user-facing crate keeps its single-word name when published, but a
  published library would need a `praps-` package name, which AC-8's rule
  rejects. AC-8 therefore covers unpublished packages only; when a library
  first becomes publishable, the `names` rule gains a `praps-[a-z][a-z0-9]*`
  form for it, as a recorded revision of this chapter.
- `praps` stays free only until someone publishes a crate with that name; the
  owner can hold it by publishing a placeholder crate early (AC-7).
- The repository LICENSE is the GPL-3.0 text, but `GPL-3.0-only` versus
  `GPL-3.0-or-later` is not stated. Crate metadata uses `GPL-3.0-only`, the
  narrower grant, until the owner decides.

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 I.5 Phase 0 and the Biblio report §5).
- 2026-10-31: `proposed` → `planned` (Sprint 0 build plan T-001, T-007, T-008). Revised before planning on owner direction: single-word crate names and flagless idempotent tooling (AC-8 and AC-9 added, AC-7 reworded for publication only). After the first plan critique: cargo-deny's place outside `cargo ci`, and the CI-dependence of AC-2 to AC-4, recorded in Consequences. After the second plan critique: AC-6's audit command corrected to `cargo tree --workspace --target all -e normal,build -i cc` (`-e build` alone never reaches a normal dependency such as `blake3`); AC-8 limited to unpublished packages, with the publication path in Consequences; AC-10 added for the README; CI evidence recorded by the next sprint. After the third: AC-10 limits the no-flags statement to the three project commands, since cargo-deny is a third-party tool with its own flags.
- 2026-10-31: `planned` → `active` (Sprint 0 build, T-001 starts).
- 2026-11-01: revised while `active`, on owner direction (no state change): the command and the user-facing crate are `praps`, short for periapsis, replacing `peri`, and a published library takes a `praps-` prefix. `praps` was free on crates.io when checked. The Intent, Boundaries, AC-7, Rationale and Consequences are reworded to match.
- 2026-11-08: revised while `active` (Sprint 4 plan critique, round 1, C-012): `cargo reach`, which writes the reach table of [INT-0005](INT-0005-deterministic-simulation-testing.md) AC-6, joins AC-9 and AC-10 as a fourth flagless, idempotent project command; `cargo ci` checks the table in-process instead of through a flag.
- 2026-11-08: revised while `active` (Sprint 4 plan critique, round 2, C-007): the Intent paragraph names `cargo reach` among the flagless project commands, matching AC-9 and AC-10.
