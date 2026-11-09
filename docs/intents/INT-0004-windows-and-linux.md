# INT-0004 — Identical behaviour on Windows and Linux filesystems

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0004
- **State:** active
- **Work evidence:** [T-015 backlog](../work/tasks.md), [T-016 backlog](../work/tasks.md), [T-005 build plan](../sprints/s0/sprint-plans/build-plan.md#t-005-golden-corpus-and-spec-consistency), [T-007 build plan](../sprints/s0/sprint-plans/build-plan.md#t-007-canonical-suite-runner-and-ci-workflow), [Sprint 1 T-002 build plan](../sprints/s1/sprint-plans/build-plan.md#t-002-filesystem-and-randomness-traits-in-the-sim-crate), [Sprint 1 T-007 build plan](../sprints/s1/sprint-plans/build-plan.md#t-007-windows-durability-strategy-and-real-filesystem-evidence), [T-010 backlog](../work/tasks.md), [Sprint 2 T-002 build plan](../sprints/s2/sprint-plans/build-plan.md#t-002-specify-and-implement-the-provisional-changeset-and-view-kinds-and-repository-paths), [Sprint 2 T-004 build plan](../sprints/s2/sprint-plans/build-plan.md#t-004-a-scan-syscall-an-executable-bit-capability-and-operating-system-randomness-in-sim), [Sprint 2 T-005 build plan](../sprints/s2/sprint-plans/build-plan.md#t-005-the-workspace-crate-init-snapshot-new-and-status), [Sprint 2 T-001 build plan](../sprints/s2/sprint-plans/build-plan.md#t-001-record-sprint-1s-github-actions-runs-and-owner-sign-offs), [T-013 backlog](../work/tasks.md), [Sprint 3 T-001 build plan](../sprints/s3/sprint-plans/build-plan.md#t-001-record-sprint-2s-github-actions-runs-and-owner-sign-offs), [Sprint 3 T-003 build plan](../sprints/s3/sprint-plans/build-plan.md#t-003-remove_dir-set_executable-and-name_rules-in-sim), [Sprint 3 T-004 build plan](../sprints/s3/sprint-plans/build-plan.md#t-004-specify-writing-the-working-copy-undo-restore-and-log), [Sprint 3 T-005 build plan](../sprints/s3/sprint-plans/build-plan.md#t-005-the-hazard-check), [Sprint 3 T-009 build plan](../sprints/s3/sprint-plans/build-plan.md#t-009-reversibility-and-hazard-corpora-the-materialise-crash-scenario-and-the-extended-edit-corpus), [T-017 backlog](../work/tasks.md), [Sprint 4 T-001 build plan](../sprints/s4/sprint-plans/build-plan.md#t-001-record-the-sprint-3-checkpoints-github-actions-runs-and-owner-sign-offs), [Sprint 4 T-009 build plan](../sprints/s4/sprint-plans/build-plan.md#t-009-report-skipped-names-from-every-command-that-snapshots-backlog-t-019), [Sprint 5 T-001 build plan](../sprints/s5/sprint-plans/build-plan.md#t-001-record-the-sprint-4-checkpoints-github-actions-runs-and-owner-sign-offs), [Sprint 5 T-004 build plan](../sprints/s5/sprint-plans/build-plan.md#t-004-the-ci-workflow-its-pin-and-its-clause)
- **Completion evidence:** [T-005 completion](../work/completed-tasks.md#t-005-sprint-0), [T-007 completion](../work/completed-tasks.md#t-007-sprint-0), [Sprint 1 T-002 completion](../work/completed-tasks.md#t-002-sprint-1), [Sprint 1 T-007 completion](../work/completed-tasks.md#t-007-sprint-1), [Sprint 2 T-004 completion](../work/completed-tasks.md#t-004-sprint-2), [Sprint 2 T-005 completion](../work/completed-tasks.md#t-005-sprint-2)
- **Code evidence:** [.gitattributes](../../.gitattributes), [golden rule](../../crates/xtask/src/attributes.rs), [CI workflow](../../.github/workflows/ci.yml), [real.rs](../../crates/sim/src/real.rs), [name.rs](../../crates/sim/src/name.rs)
- **Test evidence:** [Sprint 0 test report](../sprints/s0/sprint-tests/test-report.md), [Sprint 1 test report](../sprints/s1/sprint-tests/test-report.md), [Sprint 2 test report](../sprints/s2/sprint-tests/test-report.md), [Sprint 2 CI record](../sprints/s3/sprint-tests/ci-sprint2.md), [Sprint 3 test report](../sprints/s3/sprint-tests/test-report.md), [Sprint 3 CI record](../sprints/s4/sprint-tests/ci-sprint3.md), [Sprint 4 test report](../sprints/s4/sprint-tests/test-report.md)
- **Documentation evidence:** [Storage durability](../spec/storage-durability.md), [The working copy](../spec/working-copy.md)

## Intent
Periapsis produces the same object hashes, the same op log and the same
working-copy results on Windows (NTFS: case-insensitive by default, `\`
separators, reserved device names, different directory-durability semantics)
and on Linux (ext4). Repository paths are stored in one platform-neutral form:
relative, `/`-separated, UTF-8. A file or directory name in a working copy
that has no UTF-8 form is reported and left alone, never tracked under another
name. Platform differences are isolated behind the
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
- **AC-7** `necessary` `empirical` — A working-copy file or directory whose name has no UTF-8 form (bytes that are not UTF-8 on ext4, an unpaired surrogate on NTFS) is reported by an escaped form of its name and otherwise left alone, like an ignored path: it is never tracked under a changed name, overwritten, deleted or skipped without a report.
  - Threshold: 100% over a fixture corpus on each platform.

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

- On Windows, every real-filesystem path is passed verbatim (`\\?\`): Win32
  normalisation otherwise strips a trailing `.` or space from a name and turns
  `CON` or `aux.txt` into a device, so a file whose NTFS name is `foo.` would be
  read as `foo` and its content silently replaced (Sprint 2 code review CR-02).
  Paths stay relative or absolute at the `Fs` boundary; `RealFs` converts them,
  taking `..` lexically.

- The case and name rules a write follows come from the platform, not from a
  probe of each directory: Windows is taken as case-insensitive with Windows
  names, Unix as neither. This is conservative on case-sensitive NTFS
  directories (a collision may be reported that would not happen) and wrong
  on Windows directories mounted into Linux (WSL drvfs), where a collision
  could overwrite; probe-based detection is backlog T-015. Hazards are
  checked only for the paths a write creates or rewrites, so a tracked file
  with a Windows-hazardous name (possible through verbatim paths) does not
  block `undo` while it is unchanged. On a case-insensitive filesystem a
  case-only rename (`README` to `readme`) is refused as a collision: a known
  limit until backlog T-016, so that no write ever depends on case-insensitive
  lookups (Sprint 3 plan critique, rounds 1 and 2).
- AC-7 holds for writes as well: a working-copy write keys directory listings
  by UTF-8 name, and gives a name with no UTF-8 form a key no repository path
  can equal, so such an entry is never overwritten or removed, and a
  directory holding it is kept (Sprint 3 code review, CR-02). It is
  reported only by `status`, though: `undo`, `restore` and `new` drop their
  snapshot's report, a gap that backlog T-019 closes (Sprint 3 test
  critique, round 2, C-003).

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; research gap II §7 plus the owner's test-machine inventory).
- 2026-10-31: `proposed` → `planned` (Sprint 0 build plan T-005, T-007: golden bytes and the canonical runner on Windows and WSL, including a fresh Windows clone with `core.autocrlf=true`).
- 2026-10-31: `planned` → `active` (Sprint 0 build, T-005 starts: golden bytes kept binary on every platform).
- 2026-11-01: revised while `active`, on owner direction: Periapsis must work with any toolchain, and some write file names that have no UTF-8 form. The Intent and the new AC-7 say such a name is reported by an escaped form and left alone, never tracked under another name, overwritten or deleted; stored paths stay UTF-8.
- 2026-11-07: revised while `active` (Sprint 2 Loop): Consequences record that `RealFs` passes verbatim paths on Windows (code review CR-02). AC-2 and AC-7 are met by the Sprint 2 test report for snapshotting; AC-3, AC-4 (writing a working copy), AC-5 (sign-off) and AC-6 (Nighthawk) remain, so the chapter stays `active`.
- 2026-11-07: revised while `active` (Sprint 3 plan critique, round 1): Consequences record that write rules come from the platform with their limits (T-015), that hazards apply only to the paths written, and that a case-only rename is not a collision.
- 2026-11-07: revised while `active` (Sprint 3 plan critique, round 2): a case-only rename on a case-insensitive filesystem is refused as a known limit (T-016), replacing the round-1 exemption.
- 2026-11-09: evidence while `active` (Sprint 4 Loop): AC-7's report is complete. `undo`, `restore` and `new` report names with no UTF-8 form as `status` does, on NTFS and ext4 ([Sprint 4 test report](../sprints/s4/sprint-tests/test-report.md); backlog T-019 closed). AC-5 and AC-6 are unchanged: sign-off pending, Nighthawk not reachable.
