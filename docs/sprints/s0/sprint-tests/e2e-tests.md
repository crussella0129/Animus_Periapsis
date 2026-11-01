# Sprint 0 End-to-End Test Results

- **Status:** possible. Both planned scenarios ran, plus one added scenario, `e2e_port_rule_local_books`.
- **Tested head:** `ac0e010a76d45fcce7a619889d94e4dfb8048597` on `dev`, committed. The head includes the fixes from the Test-phase [code review](code-review.md) and from the test critiques. Later commits change only the Book.
- **Host:** Microsoft Windows 11 Pro 10.0.26300; rustc 1.99.1 (58cf8f9c3 2026-10-13), `x86_64-pc-windows-msvc`, LLVM 23.1.1.
- **Raw output:** [test-matrix-log.md](test-matrix-log.md).

## `e2e_fresh_clone_windows`
- **Intents:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-4 (local half), [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-1, [INT-0006](../../../intents/INT-0006-format-v1.md) AC-3 (local half)
- **Input:** `git clone -c core.autocrlf=true --branch dev` of the committed head into a new Windows temp directory. This is the line-ending setting most likely to corrupt files on Windows.
- **Observed:**
  - the clone's head is `ac0e010`, with `core.autocrlf=true`;
  - `sha256sum` over the 23 `crates/codec/tests/golden/v1/*.bin` files matches the original checkout's;
  - no tracked text file has CRLF line endings, because `.gitattributes` (`* text=auto eol=lf`) overrides `core.autocrlf`;
  - `cargo ci` from a cold build printed `PASS` for fmt, clippy, test, policy and book, then `ci: all 5 steps passed`; exit 0, in 40 s.
  - its `policy` step ran the golden rule in the clone: the clone's `core.autocrlf` is not among the settings the rule reads, and the workspace's own `.gitattributes` kept every `.bin` file binary.
- **Result:** pass

## `e2e_port_rule_sprint_loops_book`
- **Intent:** [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md) AC-4
- **Input:** `cargo book C:/Users/charl/Animus_Sprint_Loops/docs`. The Animus_Sprint_Loops checkout is at its head, `e8c3a58018e1627a145759fdac1f089c6236ba36` (2026-09-04), with a clean tree. Its marker has `schema-version: 2` and `substrate-version: 4` but no `typed-acceptance` line, so the validator selects the port rule.
- **Observed:** `book: C:/Users/charl/Animus_Sprint_Loops/docs (undeclared, port rule)`, one line per chapter, then `13 chapters, 82 criteria, 0 violations`; exit 0.
- **Independent count:** an `awk` count of the top-level `- ` items under each `## Acceptance criteria` heading, skipping fenced code, gives the same number for every chapter:

| Chapter | `cargo book` | `awk` |
|---------|--------------|-------|
| INT-0001-project-book.md | 5 | 5 |
| INT-0002-substrate-and-branch-model.md | 6 | 6 |
| INT-0003-dependency-updates-on-work.md | 7 | 7 |
| INT-0004-substrate-contract-versioning.md | 7 | 7 |
| INT-0005-turn-and-checkpoint-contract.md | 6 | 6 |
| INT-0006-provider-reach-and-ci-truth.md | 11 | 11 |
| INT-0007-integrity-sweep.md | 5 | 5 |
| INT-0008-escalation-and-flow.md | 5 | 5 |
| INT-0009-sprint-identity.md | 5 | 5 |
| INT-0010-rust-portability-study.md | 5 | 5 |
| INT-0011-human-in-the-loop-compatibility.md | 5 | 5 |
| INT-0012-ci-scaffolding-lifecycle.md | 6 | 6 |
| INT-0013-verification-integrity.md | 9 | 9 |
| **Total** | **82** | **82** |

- **Pass conditions:** 13 chapters = 13 `INT-*.md` files; every chapter has at least one criterion; 82 equals the independent count; 0 violations; exit 0.
- **Result:** pass

## `e2e_port_rule_local_books` (added in the Test phase)
- **Intent:** [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md) AC-4, in the sense of its port rule: every Book v2 is already a valid typed Book.
- **Why added:** the code review showed that the first, line-based validator misread other Books on this host. It merged their ordered lists, for example, reading 696 criteria as 245. INT-0002 AC-4 names only the Animus_Sprint_Loops Book, so this broader scenario is supporting evidence.
- **Input:** `cargo book <dir>` for every directory `C:/Users/charl/*/docs` with a `.sprint-loop-book` marker: 24 Books, 23 undeclared and this repository's declared Book.
- **Oracle:** markdown-it-py 4.0.0 (CommonMark preset), an independent parser. For each chapter it counts the top-level list items of the top-level level-2 `Acceptance criteria` section, up to the next top-level heading of level 1 or 2. A code-review verifier also compared every criterion's line, claim and details against a markdown-it-py model, with no differences.
- **Observed:** every chapter's `cargo book` count equals the CommonMark count:

| Book | Rule | Chapters | `cargo book` | CommonMark | Violations | Exit |
|------|------|----------|--------------|------------|------------|------|
| Animus_Amalgam | port | 7 | 39 | 39 | 0 | 0 |
| Animus_Ferric | port | 14 | 112 | 112 | 0 | 0 |
| Animus_Periapsis | strict | 31 | 164 | 164 | 0 | 0 |
| Animus_Sprint_Loops | port | 13 | 82 | 82 | 0 | 0 |
| Animus_Unitarity | port | 1 | 7 | 7 | 0 | 0 |
| CubiKan | port | 15 | 136 | 136 | 0 | 0 |
| Feel_of_the_Wheel | port | 13 | 77 | 77 | 0 | 0 |
| Gnash_Equilibrium | port | 74 | 627 | 627 | 0 | 0 |
| Kinesin | port | 33 | 159 | 159 | 0 | 0 |
| Leibniz | port | 1 | 7 | 7 | 0 | 0 |
| Lovers_Lagoon | port | 7 | 43 | 43 | 0 | 0 |
| MDeezl-hammer | port | 6 | 63 | 63 | 0 | 0 |
| MDeezl | port | 6 | 63 | 63 | 0 | 0 |
| Project_Lagoon | port | 8 | 64 | 64 | 0 | 0 |
| Thread-Signal-Website | port | 2 | 11 | 11 | 0 | 0 |
| Tome | port | 25 | 106 | 106 | 0 | 0 |
| WordCrack | port | 6 | 41 | 41 | 0 | 0 |
| beamsplitter | port | 3 | 16 | 16 | 0 | 0 |
| contrapunctus | port | 3 | 15 | 15 | 0 | 0 |
| diver | port | 26 | 147 | 147 | 0 | 0 |
| oxidizer | port | 5 | 31 | 31 | 0 | 0 |
| sdr.rs | port | 17 | 83 | 83 | 0 | 0 |
| sprint-loops | port | 13 | 82 | 82 | 0 | 0 |
| yarduluv | port | 31 | 478 | 478 | 0 | 0 |
| **Total** | | **360** | **2,653** | **2,653** | **0** | |

- **Inputs pinned:** each Book's commit is listed in [test-matrix-log.md](test-matrix-log.md). Both counts read the same working files. Only Feel_of_the_Wheel had uncommitted changes under `docs/intents` (2 paths), and its counts agree as well.
- **Notes:** in three chapters, both counts are 0: Gnash_Equilibrium's INT-0006, whose section is a single paragraph, and Feel_of_the_Wheel's INT-0011 and INT-0012, whose heading reads "Acceptance criteria (for the future build sprint)". Under the port rule, criteria are list items in a section with exactly that title, so these read as having no criteria. That is consistent with CommonMark.
- **Result:** pass

## Carried forward
- **To Sprint 1, as `T-009 (backlog) [intent: INT-0001, INT-0002, INT-0003, INT-0006]`:**
  - the GitHub Actions run of the checkpoint: head SHA, both matrix legs with their golden and `cargo ci` results, the `deny` job, and the runner OS and `rustc`. This is the GitHub half of INT-0003 AC-2 to AC-4 and of INT-0006 AC-3;
  - the owner's checkpoint sign-offs: INT-0001 AC-1, INT-0002 AC-1, INT-0003 AC-5 and INT-0006 AC-6.
  - Neither can happen inside Sprint 0, because the checkpoint PR opens only after `close-sprint.sh`.
- **Optional, not run:** `cargo ci` on the Debian host Nighthawk, whose Tailscale SSH check needs the owner's approval in a browser. Sprint 0 loses no coverage, because the WSL Ubuntu leg covers Linux.
- **Not yet possible:** the INT-0004 AC-6 scenarios on Nighthawk (multi-peer sync, crash and restart) need sync and storage code. The first sprint that plans INT-0007 or INT-0024 scenarios unlocks them.
