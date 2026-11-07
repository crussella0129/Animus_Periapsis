# Sprint 2 Meta

- **Sprint number:** 2
- **Book schema version:** 2
- **Start timestamp:** 2026-11-07T13:06:26Z
- **End timestamp:** 2026-11-07T18:50:50Z
- **Model:** claude-opus-5-5
- **Bundle version:** 0.22.0
- **Exit status:** success
- **Token count:** (filled at Loop Phase if observable)
- **Summary:** The first working-copy snapshot: `praps init`, `praps new` and `praps status`, a snapshot at the start of every command with `.gitignore` rules, the provisional `changeset` and `view` kinds and an `operation` that names its view, closure through references, a `sim` scan syscall that keeps names with no UTF-8 form, a Snapshot crash scenario and edit corpora; plus the Sprint 1 CI record
- **Intents:** [INT-0014](../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0006](../../intents/INT-0006-format-v1.md), [INT-0007](../../intents/INT-0007-crash-safe-storage.md), [INT-0004](../../intents/INT-0004-windows-and-linux.md), [INT-0005](../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0003](../../intents/INT-0003-engineering-baseline.md), [INT-0001](../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../intents/INT-0002-typed-acceptance-criteria.md)
- **Completion evidence:** ea92720: 332 tests pass on Windows and WSL; cargo ci and cargo deny pass; store matrix 0 violations in 32,840 cells and Snapshot matrix 0 in 25,360, 5 + 1 negative controls caught; edit corpus 6,000 snapshots and 0 lost edits; final test critique (round 2) proceed-with-caveats
- **Checkpoint:** https://github.com/crussella0129/Animus_Periapsis/pull/4
- **Confidence:** 0.8 -> 0.7 (`patched`: the Test-phase code review and two test critiques found defects and weak tests, all fixed before the tested head)
