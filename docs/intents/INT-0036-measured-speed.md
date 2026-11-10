# INT-0036 — Speed: measured, and faster than Git and Jujutsu on the same machine

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0036
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Periapsis is faster than the version control systems people use today. The
claim is measured on fixed workloads, on the same machine, with the same
durability, and the measurement is part of the project. A speed that has not
been measured is not claimed.

The suite. Each workload is run with `praps`, with Git and with Jujutsu, each
at its latest stable release:

| | Workload | What is timed |
|---|---|---|
| W1 | First snapshot | Recording 10,000 new files of 2 KiB as one change |
| W2 | Nothing changed | `status` on an unchanged working copy of 100,000 files |
| W3 | One edit | Recording an edit to one file among 100,000 |
| W4 | Looking back | Listing the latest 100 changes of a history of 100,000 |
| W5 | Undo | Undoing an operation that changed 100 files |
| W6 | Many writers | 100 processes each recording one change of 10 files in one repository at once, until a reader sees all 100 |
| W7 | A new lane | Making a workspace on a project of 100,000 files, ready for an agent to write |
| W8 | Sync | Bringing a peer up to date with 1,000 new changes over a local connection |

Rules:
- **Equal durability.** Each comparison is made twice: with every tool at its
  defaults, and with every tool set to keep what it has acknowledged through
  a crash, as Periapsis always does
  ([INT-0007](INT-0007-crash-safe-storage.md)). The criteria use the second.
  Durability is never given up for speed without saying so.
- **Both platforms.** Every figure is taken on Windows and on Linux
  ([INT-0004](INT-0004-windows-and-linux.md)).
- **Counts as well as clocks.** For each workload the suite also records the
  syscalls, flushes and bytes the simulator counts. Counts are exact and the
  same on every machine, so they catch a regression that a noisy clock hides.
- **Speed comes from doing less**, never from a weaker guarantee:
  - one flush for a command, not two for each file
    ([INT-0008](INT-0008-compact-history-storage.md));
  - no file read that has not changed
    ([INT-0014](INT-0014-working-copy-and-workspaces.md));
  - indexes kept as derived data ([INT-0013](INT-0013-derived-data.md));
  - scanning and hashing on every core;
  - for many agents on one machine, one local service that shares a flush
    among them ([INT-0022](INT-0022-agent-and-human-interfaces.md)).

Boundaries: this intent owns the workloads, the comparison rules and the
budgets. How cost grows with size is
[INT-0035](INT-0035-scale-and-asynchrony.md). Non-goal: timings of single
functions as evidence. They may guide the work, but no criterion rests on
them.

## Acceptance criteria
- **AC-1** `necessary` `empirical` — The suite runs: one project command, with no flags, runs every workload whose commands exist against `praps`, Git and Jujutsu, on Windows and on Linux. It records each tool's version and durability setting, the machine, the median of at least 10 runs, and their spread.
  - Threshold: 100% of the workloads whose commands exist.
- **AC-2** `necessary` `empirical` — Faster at equal durability: on every workload, the median time of `praps` is below the lower of Git's and Jujutsu's, with each tool set to keep acknowledged work through a crash.
  - Threshold: 8 of 8 workloads, on Windows and on Linux.
- **AC-3** `supporting` `empirical` — Far faster where agents work together: `praps` is at least 10 times faster than the faster of Git and Jujutsu on W6 and W7, and at least twice as fast on W1 and W3.
- **AC-4** `supporting` `empirical` — Faster than the others at their fastest: on every workload `praps` is faster than Git and Jujutsu at their defaults, and with every setting their own documentation recommends for large repositories.
  - Threshold: 8 of 8 workloads, on both platforms.
- **AC-5** `supporting` `empirical` — The interactive budget: on a repository of 100,000 files and 10⁶ operations, every command that changes nothing returns within 100 ms.
  - Threshold: the median of 10 runs on each reference machine the suite records.
- **AC-6** `necessary` `empirical` — No silent regression: `cargo ci` fails when a workload's count of syscalls, flushes or bytes rises above its recorded value, unless the same commit changes the recorded value.
- **AC-7** `necessary` `asserted` — The project owner approves the workloads and the thresholds before the first run of the suite that counts as evidence.
  - Signers: project owner.

## Rationale
- Owner direction, 2026-11-09: Periapsis "should be far faster and more
  optimized than any version control that currently exists".
- Where Periapsis stood
  ([Sprint 6 research](../sprints/s6/sprint-research/research-report.md),
  Measurements; one machine, a throwaway probe):

  | 10,000 files of 2 KiB | `praps` | Git, flushing | Git, defaults |
  |---|---:|---:|---:|
  | Record them all, Linux | 147 s | 67 s | 1.9 s |
  | Record them all, Windows | 140 s | 24 s | 36 s |
  | `status`, nothing changed, Linux | 148 ms | 10 ms | 11 ms |
  | `status`, nothing changed, Windows | 1,190 ms | 78 ms | 85 ms |
  | Record one edit, Linux | 287 ms | 38 ms | 16 ms |
  | Record one edit, Windows | 1,610 ms | 186 ms | 127 ms |

  On Windows, Git's two columns came out in the opposite order to Linux's.
  The probe ran each once, on a machine with virus scanning on, so that
  difference is probably noise and not an effect of the setting.
- Why it was slow is known from the code
  ([code audit](../sprints/s6/sprint-research/code-audit.md)): two flushes
  for every file stored, every file read and hashed by every command, and the
  whole op log read three to six times by every command. None of that is
  needed for the guarantees.
- The Linux row shows what one flush for a command would give. Git flushing
  each object took 67 s and Periapsis, with two flushes for each, 147 s. Git
  not flushing at all took 1.9 s. A store that writes a command's objects
  together and flushes once would keep every guarantee at close to the third
  figure.
- Git is fast for reasons Periapsis can use. Its index is a cache of file
  metadata, its packs put many objects in one file, and its commit graph is
  an index. All three are derived data in the sense of
  [INT-0013](INT-0013-derived-data.md).
- Git's default trades durability for speed: it does not flush loose objects,
  and its documentation says recent work can be lost if the system stops
  uncleanly. A comparison at the defaults alone would be unfair to Periapsis
  and would hide that difference; a comparison at equal durability alone
  would hide what people feel. So both are reported.
- Agents working together are where existing systems are slow by
  construction: one checkout for each agent, and a shared branch to queue at.
  That is where AC-3 asks for the largest margin.
- Thresholds are fixed here, before the suite exists, as
  [INT-0002](INT-0002-typed-acceptance-criteria.md) requires of an
  `empirical` criterion.

## Alternatives
- **"Fast enough", unmeasured.** The owner's direction is a comparison, and a
  comparison needs the other side measured too.
- **Timings of single functions only.** They miss process start, flushes and
  directory listing, which is where the time was.
- **Comparing at default settings only.** Git's default keeps less through a
  crash than Periapsis does.
- **Matching Git by not flushing.** That gives up
  [INT-0007](INT-0007-crash-safe-storage.md).

## Consequences
- Running the suite needs Git and Jujutsu installed. `cargo ci` does not: it
  checks the counts (AC-6), not the clocks.
- Clocks are noisy, above all on a machine with virus scanning. The suite
  reports medians and spread, and the criteria that gate use counts.
- [INT-0003](INT-0003-engineering-baseline.md) forbids `unsafe` and prefers
  pure Rust, which rules out memory-mapped files and hand-written assembly
  for hashing. The suite shows what that costs; an exception goes through
  that chapter's AC-6.
- Beating a Git that uses a file-system monitor on W2 (AC-4) needs a watcher
  of our own, which [INT-0004](INT-0004-windows-and-linux.md) lists as a
  non-goal for now. That chapter is revised when AC-4 is taken up.
- W4 to W8 wait for their commands. The suite grows as they are built, and
  AC-2's count of workloads is checked against those that exist.
- The time budget of [INT-0014](INT-0014-working-copy-and-workspaces.md) AC-5
  is W2 under AC-2 and AC-5 of this chapter.

## Transition history
- 2026-11-10: created as `proposed` (Sprint 6 research; owner direction of 2026-11-09 that Periapsis be far faster than any existing version control).
