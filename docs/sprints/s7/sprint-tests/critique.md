# Test Critique — Sprint 7

Round 3, the final critique, on the records at `2031b13` with the tested head `4068f43`. A separate, read-only critic wrote it from the Sprint Loops test-critic prompt; it ran no test and no command that changes anything, and read GitHub only through read-only `gh` commands. Round 1, whose verdict was `block`, is [critique-round1.md](critique-round1.md), and round 2, also `block`, is [critique-round2.md](critique-round2.md), each with the responses to its concerns. Its concerns and its account of round 2 are given as it wrote them; the responses at the end are the primary agent's.

## Concerns

### C-001: `praps lane DIR` is never run inside a lane
- **Where:**
  - `build-plan.md`, T-007. One clause: "WHEN any command runs in a lane, at its top or below it, THEN it SHALL act on that lane over the shared store". Another: `praps lane DIR` run at a working copy's top and from below it agree as two `new` runs do. T-006's first clause makes the lane "from workspace W".
  - `docs/spec/working-copy.md`, "Finding the working copy", lists `lane` among the commands that act on the lane they run in.
  - `cli.rs`:
    - `commands_act_on_the_lane_they_run_in` (lines 2999–3030) runs eight commands, and `lane` is not one of them.
    - `commands_act_the_same_from_below_in_a_lane` (3156–3306) has the corpus and, since round 2, `new`. It does not have the `lane DIR` section that `commands_act_the_same_from_below` has (1278–1322).
  - Every `praps lane` runs from a repository: in `cli.rs` at lines 1302, 2314, 2346, 2759, 2772, 2845, 2900, 2932, 3139 and 3322, and in `e2e.rs` at 705, 825 and 831.
  - Every `make_lane` in `crates/workspace/tests/` is called on the repository's own workspace.
  - `completed-tasks.md` T-007: "Every command run in a lane, or below it, acts on that lane".
- **Quote:** "WHEN any command runs in a lane, at its top or below it, THEN it SHALL act on that lane over the shared store."
- **Failure mode:** EARS-coverage
- **Why it matters:**
  - **The gap.** `lane` is the only command that opens a working copy and is never run in a lane, neither at its top nor below it.
  - **What could break unseen.** In a lane, `make_lane` writes `self.store.root()` into the new lane's `.praps/workspace` (`crates/workspace/src/lib.rs` 1248–1256), and starts the lane on the parent of a lane's change. That is correct by reading. If it wrote `<root>/.praps/store` instead, every test would still pass, and a lane made from a lane would be refused as `StoreGone` at its first command.
  - **The false trace.** The report would trace T-007's in-lane clause, and INT-0037 AC-3 inside a lane, to tests that never run `lane` there. This is round 2's C-004 one command on, and weaker: `new` was at least run in a lane.
- **Suggested response:** add-test, or else defer-with-rationale.
  - **add-test.** Move the `lane DIR` section of `commands_act_the_same_from_below` into the in-lane test. Run it at the lane's top and from two levels down, with DIR relative to each place. Compare the masked line, exit code, op count, files and `praps lanes`. Then assert that the new lane's `.praps/workspace` names the shared store, and that `praps status` there exits 0.
  - **defer-with-rationale.** The report limits T-007's in-lane clause, and INT-0037 AC-3 in a lane, to every command except `lane`, and a backlog item holds the rest.

### C-002: Record lines that disagree with the logs, the code or themselves
- **Where:**
  - `test-matrix-log.md`: "What the test critique changed in these lines", and the fourth Concurrency control.
  - `sign-offs.md`: "Decisions that are not sign-offs".
  - `tasks.md` T-036.
  - `hint.rs`: `unreadable_above` and the review `table()`.
- **Quote:** "Against `5b65da8`, each system prints four lines more, and three lines of the Concurrency controls changed"
- **Failure mode:** evidence-drift
- **Why it matters:** None of these changes a result, but the report takes its counts and its list of open gaps from these lines.
  - **Changed lines.** Against the lines recorded in `c3ad8b8` (for `5b65da8`), only one of the three control lines changed: `UnsyncedObjects`' one-store line (store 1→2, kept 8→16, view 9→18). The `FirstHeadOnly` and `MergeOnRead` lines are identical byte for byte. So "a control's first cell counts some violations twice" holds only for the two `UnsyncedObjects` lines.
  - **The 58.** The log says "with each lane's next op checked too, the cell counts 58". That cell shows `next: 8`, so no lane's next command made an op. The 58 is the same 29 ops counted at both checks, as `unit-tests.md` and the full-sweep note say.
  - **The sign-offs.** `sign-offs.md` says "The same round of questions settled two choices" above three bullets, and the third was asked during Test.
  - **T-036 and the hint.** These still describe a gap that is closed:
    - T-036 asks to "name the directory that failed when a directory above cannot be searched";
    - `unreadable_above`'s comment says "The message names only where the search started";
    - the table's example message for that kind leaves the path out.

    Since `b951d6d`, that refusal prints `place::Unreadable`, which names `<dir>/.praps/store` and its reason.
- **Suggested response:** tighten-assertion of the records:
  - say one control line changed and one was added;
  - say the 58 is "the same 29 ops, counted at both checks";
  - say three choices: two in Plan, one during Test;
  - drop T-036's item, or keep only the narrower hint, and correct the comment and the table example.

## Status of round 2 and the open items, at `4068f43`
- **Round 2's C-001: resolved.**
  - **`top_above`.** It chooses its hint by `place::Unreadable` (`commands.rs` 197–200).
  - **`store_of`** (685–708):
    - a lane file that does not parse is refused with `fail(… BadWorkspaceFile)`;
    - one that cannot be read gets "make that file readable".
  - **`verify`.** A lane store's `NotFound` becomes `StoreGone` (834–837).
  - **Tests.** `every_refusal_prints_one_hint` (2422–2443) asserts the right hint for each of the three cases. `verify` from below a lane goes through the same `store_of`.
  - **Counts.** The test checks 39 refusal kinds on both systems, plus 5 on Unix (44) and 2 on Windows (41). The hint table has 15 + 28 = 43 rows, four of them new (429–440 and 483–497).
  - **`init`.** No refusal in a lane offers `init` any more, except the case INT-0014 records: before `.praps/workspace` is in place.
- **Round 2's C-002: resolved, with an honest deferral.**
  - **Comment and records.** The comment (1128–1135) and the records say what the control shows.
  - **Order.** `control_in([true, false], …)` runs the two-store run first and stops at its first caught cell, so `cargo ci` no longer runs the 160 one-store cells.
  - **Why the T-034 deferral hides no criterion:**
    - `lost` counts every acknowledged id missing from `store.ops()`, whatever removed it;
    - an op file torn by a crash also fails `fsck`;
    - INT-0005 AC-3's list of known-bad variants does not name unsynced op files.
  - **Left over.** One phrase in the matrix log (C-002 above).
- **Round 2's C-003: resolved.**
  - **What is checked.** Each lane's next op is acknowledged as its last, with `done` set (612–628). Every store is then checked again for lost, writer, kept and view (659–689).
  - **The check can fire.**
    - In the full sweep, `FirstHeadOnly`'s two-store `kept` rose from 180 to 820. That is 4 lanes per cell, the lanes whose next op sits on the head `FirstHeadOnly` does not load.
    - With the second exchange taken out, the scenario failed, as reading the code predicts.
  - **Digests.** They are taken before `check` runs (`checked()` 834–838), so the digests rightly do not move.
- **Round 2's C-004: resolved.** `new` runs in three places, each on its own copy (3245–3304). The test masks the change ID and compares:
  - the op count;
  - the lane's files;
  - what `status` prints next.

  The newest op must name `lane`.
- **Open items.** None of them hides an unproved criterion, but each needs the report to say it.
  - **Finding 2 (T-042).**
    - The deferral is defensible: T-006 promises a remake at DIR only, the hint names DIR, and no op is lost.
    - T-042's sentence is corrected.
    - INT-0014 still lists "Two cases are left open". The report owes the kill route as a known defect.
  - **Two exchanges (INT-0024).**
    - The plan scopes AC-5 to two stores that exchange every operation.
    - `integration-tests.md` and `e2e-tests.md` both say the stores exchange twice, after the run.
    - The report must claim AC-5 within that scope only.
  - **The lost-op control (T-034).** See round 2's C-002 above.
  - **The needless-wake count on GitHub.**
    - All six job logs print only `test a_thousand_processes_run ... ok`, and the bound of 30 was asserted and passed on every run.
    - The records' "saw 0 on both systems" is a local count. That GitHub's count is unknown appears only in `critique-round2.md`'s response, so the report must say it.
  - **AC-11.** `cargo ci` stops each Concurrency control at its first caught cell. The AC-11 "pass" in `e2e-tests.md` does not say so, and the report must carry it there, as promised.
  - **Format sign-off.** The owner has signed it, so T-002's last clause and INT-0006 AC-6 hold for this change. AC-6 does not require the answer before the merge.
- **Numbers checked against the code, the logs and GitHub.**
  - **Totals.**
    - 663 and 666 tests, 52 test executables and 43 units appear in all six `cargo ci` logs of run 43652905935, and in `main`'s run.
    - 636 = 560 + 76. A raw count of `#[test]` lines gives 639 and 563, three more at both commits, two of them in `properties.rs`'s macro. So the difference of 76 holds.
  - **Full sweep.**
    - 148 + 160 = 308, and 88 + 92 = 180.
    - 7,264 = 2 × 3,632 fits `next: 1280`: every lane's next command failed in every cell, so the second check counts the same ops again.
    - In the first cells, as the code counts them: 58 = 2 × 29, store 6 = 3 + 3, and the one-store store 2 = 1 + 1.
  - **Result lines.** 98 = 94 + 3 portable digests + 1 line for the lost-op control.
  - **Run 43652905935.**
    - It is the only run of `4068f43`: attempts 1 to 3, every job `success`.
    - The 15 job times and the waits match: 3 s each, and 4 s for `cargo deny check` in attempt 3.
    - All 18 unit times match.
    - The runners are 20261103.298.1 and 20261111.196.1, and every `cargo ci` job restored its cache.
  - **#10 and `main`.** #10 was merged at 22:36:56Z at `b951d6d`, as `5e56dd7`. `main`'s run 43649024121 succeeded in attempt 1: Windows took 10 min 14 s with 663 tests, Linux 5 min 44 s with 666. No later push to `main` followed it.
  - **Overlap counts.**
    - 35 = 3 + 8 + 24 ops.
    - 16 = 24 − 8: every lane's first op was made over the one head the setup left, and every later op over several.
    - Each head and merge pair given for the eight `new` runs is possible.
  - **PR #11.** Its head on GitHub is `4068f43`. `2031b13` is on local `dev` only, one commit ahead of `origin/dev`.
- **What the verdict rests on.**
  - Round 2's four concerns are fixed in the code and the tests.
  - Each open item is a stated deferral, inside the plan's own scope.
  - C-001 is the one EARS gap left. It is narrow and cheap to close.
  - If the report claimed T-007's in-lane clause for `lane` without either a test or a scope note, that would be a false pass, and a block.

## Confidence
proceed-with-caveats

## Responses

No code or test changed after this critique, so the tested head is still `4068f43`.

- **C-001: defer-with-rationale.**
  - No test runs `praps lane DIR` inside a lane.
  - Adding one now would make a new tested head, and every run of Test would be repeated, GitHub's three samples among them, for a path that reads correct: `make_lane` writes its store's own root into the new lane's file.
  - The test report therefore limits T-007's in-lane clause, and INT-0037 AC-3 inside a lane, to every command but `lane`. `unit-tests.md` says so on T-007's row.
  - Backlog T-042 now holds the test: `lane DIR` run at a lane's top and from below it, compared as `new` is, with the new lane's file checked to name the shared store and `status` there checked to work.
- **C-002: tighten-assertion of the records.**
  - The matrix log now says that, against `5b65da8`, four lines were added (three portable digests and the fourth control's line) and one control line changed, `UnsyncedObjects`' one-store first cell. The `FirstHeadOnly` and `MergeOnRead` lines are as they were.
  - It gives the 58 as the same 29 ops counted at both of the cell's checks: there every lane's next command failed (`next: 8`), and made no op.
  - `sign-offs.md` says three choices, two in Plan and one during Test.
  - T-036 no longer asks to name the directory that failed when a directory above cannot be searched, since that refusal now names the store and its reason. Instead it holds the stale comment of `hint::unreadable_above` and the review table's example for that kind. Correcting those changes source, which would make a new tested head.
- **The open items** are stated in the test report as the critique asks:
  - the kill route of finding 2, among the known defects; INT-0014's chapter names it at the sprint's close;
  - INT-0010 AC-5, claimed only for two stores that exchange twice, after the run;
  - the lost-op control, with backlog T-034;
  - GitHub's runs, which assert the needless-wake bound but do not print the count, so the counts recorded are local;
  - `cargo ci`, which stops each Concurrency control at its first caught cell. `e2e-tests.md` now says this beside AC-11 too.
