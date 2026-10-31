# INT-0004 — Identical behaviour on Windows and Linux filesystems

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0004
- **State:** active
- **Work evidence:** [T-005 build plan](../sprints/s0/sprint-plans/build-plan.md#t-005-golden-corpus-and-spec-consistency), [T-007 build plan](../sprints/s0/sprint-plans/build-plan.md#t-007-canonical-suite-runner-and-ci-workflow)
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Periapsis produces the same object hashes, the same op log and the same
working-copy results on Windows (NTFS: case-insensitive by default, `\`
separators, reserved device names, different directory-durability semantics)
and on Linux (ext4). Repository paths are stored in one platform-neutral form:
relative, `/`-separated, UTF-8. Platform differences are isolated behind the
I/O traits of [INT-0005](INT-0005-deterministic-simulation-testing.md). Every
sprint that changes code records test evidence from both platforms.

Test matrix: the Windows 11 host; WSL2 Ubuntu 26.04 on the same machine (ext4);
the Debian host "Nighthawk" over Tailscale SSH for end-to-end and multi-machine
runs; GitHub Actions `windows-latest` and `ubuntu-latest`.

Non-goals for now: macOS (no macOS machine in the matrix) and filesystem
watchers.

## Acceptance criteria
- **AC-1** `necessary` `empirical` — The full workspace test suite passes on Windows and on Linux for every sprint that changes code, and the sprint's test evidence names the OS and toolchain of each run.
  - Threshold: 100% of suites on both platforms.
- **AC-2** `necessary` `formal` — Path encoding is host-independent: the canonical encoding of any repository path is the same bytes on Windows and on Linux.
  - Obligation: `unstated`.
  - Proxy: golden-file tests of encoded paths, run on both platforms.
- **AC-3** `necessary` `empirical` — Case-fold collisions (two tracked paths equal under case-insensitive comparison) are detected and reported when materialising onto a case-insensitive filesystem; neither file is silently overwritten.
  - Threshold: 100% detection over a collision fixture corpus.
- **AC-4** `necessary` `empirical` — Paths that Windows cannot represent (reserved names such as `CON` or `NUL`, trailing dots or spaces, `:` and other forbidden characters) produce an error naming the path instead of a silent rename or partial write.
  - Threshold: 100% over a hazard fixture corpus.
- **AC-5** `necessary` `asserted` — The storage layer documents its durability strategy on Windows, where directory fsync is not available as on POSIX, and a reviewer accepts the crash-test evidence for it.
  - Signers: human reviewer.
- **AC-6** `supporting` `empirical` — End-to-end scenarios (multi-peer sync, crash and restart) also run on the Debian host Nighthawk.

## Rationale
- Windows and case-insensitive filesystems are explicit gaps in the research
  ("not researched", II §7 and I.8), yet the owner develops on Windows.
- Crash consistency depends on directory-entry durability: R2-E23 found a real
  bug (missing directory fsync at init) only under strict POSIX semantics, so
  Windows needs its own documented and tested strategy.
- The owner has a Windows host, WSL on the same machine, and the Debian
  machine Nighthawk on Tailscale for runner and E2E testing.
- Sources: [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md)
  II §7, I.8, R2-E23.

## Alternatives
- **Linux-first and port later.** Rejected: path, case and durability
  semantics leak into the format and storage design, so retrofitting is
  expensive.
- **Preserve OS-native path bytes.** Rejected: identical histories would hash
  differently per platform.

## Consequences
- Every storage, working-copy and sync change needs evidence from both
  platforms, which costs CI time.
- Nighthawk runs need Tailscale SSH to authenticate (its policy asks for a
  browser check), so E2E evidence there depends on the owner approving access.

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; research gap II §7 plus the owner's test-machine inventory).
- 2026-10-31: `proposed` → `planned` (Sprint 0 build plan T-005, T-007: golden bytes and the canonical runner on Windows and WSL, including a fresh Windows clone with `core.autocrlf=true`).
- 2026-10-31: `planned` → `active` (Sprint 0 build, T-005 starts: golden bytes kept binary on every platform).
