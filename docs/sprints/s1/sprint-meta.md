# Sprint 1 Meta

- **Sprint number:** 1
- **Book schema version:** 2
- **Start timestamp:** 2026-11-06T22:33:37Z
- **End timestamp:** 2026-11-07T02:31:52Z
- **Model:** claude-opus-5-5
- **Bundle version:** 0.22.0
- **Exit status:** success
- **Token count:** (filled at Loop Phase if observable)
- **Summary:** Deterministic simulation (`sim`) and a crash-tested store (`store`): per-op files, heads derived from op files, `fsck` and `praps fsck`, the `operation` object kind, a crash matrix with negative controls, the Windows durability strategy; plus the Sprint 0 CI record and the cargo ci target-directory guard
- **Intents:** [INT-0005](../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0007](../../intents/INT-0007-crash-safe-storage.md), [INT-0006](../../intents/INT-0006-format-v1.md), [INT-0004](../../intents/INT-0004-windows-and-linux.md), [INT-0003](../../intents/INT-0003-engineering-baseline.md), [INT-0001](../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../intents/INT-0002-typed-acceptance-criteria.md)
- **Completion evidence:** 9f138f4: 242 tests pass on Windows and WSL; cargo ci and cargo deny pass; crash matrix 0 violations in 18,688 cells with 4 negative controls caught; final test critique (round 2) proceed-with-caveats
- **Confidence:** 0.9 -> 0.8 (`patched`: the Test-phase code review and two test critiques found defects and weak tests, all fixed before the tested head)
