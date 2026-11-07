# Sprint 1 Meta

- **Sprint number:** 1
- **Book schema version:** 2
- **Start timestamp:** 2026-11-06T22:33:37Z
- **End timestamp:** (filled at Loop Phase)
- **Model:** claude-opus-5-5
- **Bundle version:** 0.22.0
- **Exit status:** in-progress
- **Token count:** (filled at Loop Phase if observable)
- **Summary:** Deterministic simulation (`sim`) and a crash-tested store (`store`): per-op files, heads derived from op files, `fsck` and `praps fsck`, the `operation` object kind, a crash matrix with negative controls, the Windows durability strategy; plus the Sprint 0 CI record and the cargo ci target-directory guard
- **Intents:** [INT-0005](../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0007](../../intents/INT-0007-crash-safe-storage.md), [INT-0006](../../intents/INT-0006-format-v1.md), [INT-0004](../../intents/INT-0004-windows-and-linux.md), [INT-0003](../../intents/INT-0003-engineering-baseline.md), [INT-0001](../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../intents/INT-0002-typed-acceptance-criteria.md)
- **Completion evidence:** (filled at Loop Phase)
- **Confidence:** 0.9 -> 0.8 (`patched`: the Test-phase code review and two test critiques found defects and weak tests, all fixed before the tested head)
