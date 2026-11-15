# Sprint 7 Test Report

**Tested head:** `4068f43`. It carries T-001 to T-010 and four commits made during Test:
- `1d5c94b`: the fixes from the [code review](code-review.md), and a digest for each platform for the two matrices whose cells hold a lane file;
- `5b65da8`: T-008's 1,000-process test moved to a binary of its own, after GitHub's Windows runner timed out one of the scheduler tests beside it;
- `b951d6d`: the answers to the test critique's first round ([critique-round1.md](critique-round1.md)): an unreadable `.praps/workspace` is refused as that file, and the tests it found weak are added or tightened;
- `4068f43`: the answers to its second round ([critique-round2.md](critique-round2.md)): a lane file met from below a lane's top gets that file's hint, `verify` in a lane refuses a damaged lane file and a missing store as `status` does, the Concurrency scenario checks every store again after each lane's next command, and `new` is compared from below inside a lane.

The owner merged the sprint's pull request, #10, at `b951d6d` while Test was still running. At the owner's choice the commits after it go to `main` through a second pull request, #11, and the tested head is that pull request's.

Every commit after `4068f43` changes records only.

**Where the results are:**
- [unit-tests.md](unit-tests.md), with the negative controls and the reviews the test plan names;
- [integration-tests.md](integration-tests.md), with the suite on both systems, the Concurrency scenario and the cost before and after;
- [e2e-tests.md](e2e-tests.md), with GitHub's three samples;
- [test-matrix-log.md](test-matrix-log.md);
- [code-review.md](code-review.md);
- [critique-round1.md](critique-round1.md), [critique-round2.md](critique-round2.md) and [critique.md](critique.md).

## Intent Verification
| Intent | Acceptance criterion | EARS / tests | Result | Intent evidence update |
|--------|----------------------|---------------|--------|------------------------|
| [INT-0010](../../../intents/INT-0010-lock-free-concurrency.md) | AC-1: the merge is the same for every order and repetition of the heads | T-003. It holds by construction: the merge takes a set of heads, walks only from them, and builds only ordered collections. `merge_ignores_order_and_repeats` (2,000 histories) checks that the library is deterministic, and fails the two registered merges that take a list | pass | Test evidence links this report |
| INT-0010 | AC-2: two heads give the classic three-way merge | T-003 / `two_heads_equal_the_three_way_merge` (24,090 pairs); the limit pinned by `merged_copies_and_nested_set_backs_are_pinned` | pass for heads that carry no merged copy, the scope the plan locked; the limit stays recorded in the chapter | as above |
| INT-0010 | AC-3: partial merges equal the flat merge | T-003 / `committed_partial_merges_change_nothing` (1,313 partial merges) | pass for writers that diverge from one base whose pointer holds one changeset, the scope the plan locked | as above |
| INT-0010 | AC-4: at least 4,000 criss-cross histories | T-003 / `criss_cross_histories_match_the_reference` (4,000), `a_resolution_is_not_reopened` | pass | as above |
| INT-0010 | AC-5: 8 writers on 2 replicas lose no committed op and no reachable change | T-009 / `concurrency_finds_no_violations`: 160 cells, a kill and a crash in each, 0 violations, on both systems. Each lane loads its last acknowledged entry. Each lane's next op is acknowledged too, and after it, and after a second exchange, every store is checked again. `concurrency_catches_lost_ops` shows the lost-op check firing, on ops a failed exchange left on one store | pass in simulation, for two stores in one simulated filesystem that exchange every op twice, after the run and not during it. Real replicas wait for INT-0024 | as above |
| INT-0010 | AC-6: AC-1 and AC-2 reject every registered wrong merge | T-003 / `wrong_merges_are_rejected`, `an_arbitrary_ancestor_fails_criss_cross` | pass | as above |
| INT-0010 | AC-7: readers add nothing | T-004, T-009 / `readers_write_nothing` (2 and 8), `readers_add_nothing_at_scale` (2, 8 and 1,000), `readers_catch_merge_on_read`; the Concurrency scenario's reader oracle | pass | as above |
| INT-0010 | AC-8, AC-9: 1,000 writers on 100 replicas; merge cost | not planned for this sprint | not tested | backlog T-039, T-040 |
| [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) | AC-3: 8 workspaces at once over one repository, without clones, duplicated objects or lost ops | T-006, T-009, T-010 / `eight_lanes_share_one_store` (no object outside the store, `fsck` clean); `concurrency_on_one_store_finds_no_violations` (160 cells, 0 violations; each lane's `.praps` holds only its lane file, and no object lies outside the store); E2E `lanes_at_once_end_to_end` on NTFS and ext4, whose commands overlapped (35 ops, 16 of them over several heads in every run, and 5 to 8 heads left) | pass | Test evidence links this report |
| INT-0014 | AC-1 kept: no command loses an edit | the edit corpora and the five matrices unchanged; `several_heads_are_merged_for_every_command`, `an_operation_written_after_open_is_merged`; `resume_leaves_files_when_its_workspace_raced` | pass | as above |
| [INT-0009](../../../intents/INT-0009-reversible-by-default.md) | AC-6: `undo` and `restore` by one writer leave the other's entries as the merge left them | T-005 / `undo_is_scoped_over_200_interleavings` (2,373 commands, 2,002 of which succeeded, 1,023 of those beside the other lane's op), `undo_is_scoped_to_its_workspace`, `restore_is_scoped_to_its_workspace`, `history_in_one_lane_leaves_the_other`; E2E `a_lane_session_end_to_end` | pass | Test evidence links this report |
| INT-0009 | AC-1 to AC-3 kept | the history tests, with view-id assertions for one workspace unchanged | pass | as above |
| [INT-0016](../../../intents/INT-0016-actors-and-agent-lifecycle.md) | AC-8: the owner approves the classes of owner | T-001 / [sign-offs.md](../sign-offs.md): the workspace class approved on 2026-11-14, word for word | signed in part, one class of the several the criterion asks for | Test evidence links this report |
| INT-0016 | the ownership rule applied at merge | T-003 / `stale_copies_do_not_override_the_owner` | pass; enforcement against forged operations waits for signed operations (AC-7) | as above |
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) | AC-1 to AC-3: determinism, crash matrices, controls | T-002, T-004, T-006, T-009 / seven matrices with cell counts and digests asserted; 0 violations; every control caught, and every Concurrency control over all its cells | pass. The five earlier matrices kept their counts. The two new ones assert a digest for each platform, and a portable digest that is one literal on every system | Test evidence links this report |
| INT-0005 | AC-6: reach | T-009 / `merging_marks_its_point`, `making_a_lane_marks_its_point`, `reach_table_shows_the_triangle`, `expectations_are_real`; the `reach` step | pass | as above |
| INT-0005 | AC-8 in part: 1,000 simulated processes | T-008, T-009 / `a_thousand_processes_run`, which also bounds the scheduler's needless wakes, and `readers_add_nothing_at_scale` | the 1,000-process half; the criterion's other half is not attempted. The bound is asserted on GitHub too, but the count is printed only in local runs, 0 on both systems | as above |
| [INT-0006](../../../intents/INT-0006-format-v1.md) | AC-1 to AC-3, AC-5 for the new bodies | T-002 / the codec tests and golden files for a view, a view in conflict, a root op and a merge op | pass | Test evidence links this report |
| INT-0006 | AC-6: the owner signs the specification change | review `format_sign_off_is_the_owners_answer` / [sign-offs.md](../sign-offs.md) | **signed for this change**: the owner answered "Sign off (Recommended)" on 2026-11-14, after merging #10, and the answer is recorded word for word. T-002 is complete with its last clause | Test evidence links this report |
| [INT-0037](../../../intents/INT-0037-command-set.md) | AC-2, AC-3, AC-4 for `lane` and `lanes` | T-007, T-010 / `praps_lane_refusals` (eight refusals, each one hint), `every_refusal_prints_one_hint` (44 kinds on Linux and 41 on Windows, a lane file that cannot be read or does not parse among them, at a lane's top, from below it and through `verify`), `commands_act_on_the_lane_they_run_in`, `commands_act_the_same_from_below_in_a_lane`, `praps_lane_finishes_an_interrupted_lane`, E2E `a_lane_session_end_to_end` | pass for the two commands and every refusal this sprint added. Inside a lane, AC-3 and T-007's clause that every command acts on the lane are shown for every command but `lane` itself, which no test runs in a lane (backlog T-042). AC-4's 100% over every kind is still not claimed: Sprint 6's one kind made by no test, and three places that print a hint, stay in backlog T-036 | Test evidence links this report |
| INT-0037 | AC-8: the owner approves the names | T-001 / [sign-offs.md](../sign-offs.md): `lane` and `lanes` approved | signed for these two names; the unbuilt names stay open | as above |
| [INT-0022](../../../intents/INT-0022-agent-and-human-interfaces.md) | AC-8, AC-9, AC-11 for `lanes` and `lane` | T-007 / the registry tests over every command, `idempotent_commands_repeat_exactly` (`lanes` among them), `praps_lanes_lists_every_workspace` (a repeat prints the same and writes nothing) | pass | Test evidence links this report |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-2 to AC-4 | T-001 / [ci-sprint6.md](ci-sprint6.md), checked again against GitHub during Test; this sprint's runs, and `main`'s run for the merge of #10, which was neither cancelled nor replaced | pass | Test evidence links this report |
| INT-0003 | AC-5 (credits), AC-10 (README) | T-010 / `prior_art_credits_every_listed_system`, the README checks | pass; the AC-5 sign-off is still pending | as above |
| INT-0003 | AC-11: three samples, every job under 20 minutes | [e2e-tests.md](e2e-tests.md) / run 43652905935, attempts 1 to 3 | **pass**: the Windows suite took 17 min 10 s, 16 min 57 s and 9 min 51 s, the first two over the aim of 15; every job of every sample waited 3 to 4 s for a runner; in `cargo ci` each Concurrency control stops at its first caught cell | as above |
| [INT-0004](../../../intents/INT-0004-windows-and-linux.md) | AC-1: the suite on both systems | every task on Windows; the head on Windows 11 (663) and WSL Ubuntu (666), and on GitHub's runners; the portable digests equal on both | pass | Test evidence links this report |

## Summary
- **Unit tests:** 663 passed on Windows and 666 on Linux at `4068f43`, with 0 failed and 0 ignored. 76 test functions are new and 3 renamed; none was removed.
- **The merge (T-003):**
  - order and repetition independence, which holds by construction, checked over 2,000 histories;
  - the three-way merge over 24,090 pairs of heads;
  - 1,313 partial merges;
  - 4,000 criss-cross histories against a slow recursive reference.

  Six registered wrong merges are rejected as the plan says, and an older workspace's stale copy cannot undo its owner's change.
- **Several agents in one repository:**
  - 8 `praps new` at once in one working copy all succeed; before, 3 did and the repository stopped. In three more runs on each system they left 3 to 7 op heads, and `status` named every change of the raced pointer.
  - 8 real processes in 8 lanes run at once on NTFS and ext4, and every command exits 0. Their op logs show the commands overlapped: 5 to 8 op heads left, and 16 ops made over several heads in every run.
  - In simulation, 8 lanes over one store and over two stores that then exchange lose nothing in 160 + 160 cells, with a kill and a crash in each.
- **Readers add nothing** at 2, 8 and 1,000 processes. `undo` and `restore` in one lane leave the other's entry and files alone: 2,373 commands, 2,002 of which succeeded, 1,023 of those beside the other lane's op in 200 interleavings.
- **Integration:**
  - `cargo ci` passed on Windows and in a fresh WSL clone at the tested head, and on Windows at every task's commit.
  - The five earlier matrices keep their cell counts, and the two new ones find no violation. Every control is caught, and every Concurrency control was also run over all its cells on both systems.
  - `cargo book` reports 38 chapters, 267 criteria and 0 violations; `cargo deny check` passes; `cargo reach` rewrites `docs/reach.md` unchanged. All three hold on both systems.
- **Cost, as a record and not a criterion:**
  - `status` on a repository with 2, 8 or 32 op heads failed at `855eb67` and now takes 4.5 to 5.6 ms in WSL, writing nothing.
  - At 800 operations, the new binary is 17 to 24 percent slower in WSL, and no slower on Windows, whose times vary widely.
- **E2E:** the lane tests and every earlier end-to-end test pass on both systems. On GitHub, all five checks passed in three samples of the tested head.

Not yet possible: INT-0010 AC-8 and AC-9; ownership against forged operations; real replicas over a network; INT-0037 AC-6; Nighthawk. [e2e-tests.md](e2e-tests.md) names what unlocks each.

## CI Confirmation
- **Sprint 6's checkpoint** ([ci-sprint6.md](ci-sprint6.md)), checked again against GitHub during Test. Its pull request's last runs, the merge's run on `main` and the dependency update's checks all concluded `success`.
- **This sprint's pull request,** [crussella0129/Animus_Periapsis#10](https://github.com/crussella0129/Animus_Periapsis/pull/10), opened during Test with the owner's approval. The owner merged it at `b951d6d` at 2026-11-14T22:36:56Z, while Test was still running; `main`'s run for the merge, 43649024121, concluded `success`. The commits after `b951d6d` go through [crussella0129/Animus_Periapsis#11](https://github.com/crussella0129/Animus_Periapsis/pull/11), opened during Test at the owner's choice. The agent merged nothing.
  - Run 43624973832 at `38a6b96`: `failure`. The Linux suite failed the two digest literals that were Windows-only; fixed in `1d5c94b`.
  - Run 43628379803 at `1d5c94b`: `failure`. The Windows suite's scheduler test ran out its 60-second guard; handled in `5b65da8`.
  - Run 43632738184 at `5b65da8`, attempts 1 to 3: all five jobs `success` each time. The test critique's first round then led to `b951d6d`.
  - Run 43642111524 at `b951d6d`, attempts 1 to 3: all five jobs `success` each time, with 663 tests on Windows and 666 on Linux. The Windows suite took 12 min 42 s, 16 min 27 s and 12 min 43 s. The test critique's second round then led to `4068f43`.
  - Run 43652905935 on #11 at `4068f43`, the tested head, attempts 1 to 3: all five jobs `success` each time, with 663 tests on Windows and 666 on Linux. The Windows suite took 17 min 10 s, 16 min 57 s and 9 min 51 s, and every job waited 3 to 4 s for a runner.

  The authoritative conclusion is **success**, from run 43652905935, attempt 3, finished 2026-11-15T00:11:27Z. Commits after `4068f43` change records only.

## Failures
None at the tested head. Problems that were found and fixed:
- **During Build:**
  - T-009's first `cargo ci` failed twice. A test of `cargo ci`'s own runner saw 2 units at once under the new load, where it asserts 3; it now holds each unit until three are inside. A `clippy` lint failed in the new test.
  - The Concurrency controls first ran all 320 cells each, and the unit took 374 s beside the others. Each now stops at its first caught cell: 63 s. `PRAPS_FULL_CONTROLS=1` runs every cell, and that sweep is recorded at the tested head.
- **During Test:**
  - The first WSL run, and GitHub's Linux job, failed the lane matrix's and the Concurrency scenario's digests. These had been taken on Windows only, and a lane file names its store's path as the system spells it. Each now asserts a digest per platform, and a portable digest that is one literal everywhere.
  - GitHub's Windows job failed a scheduler test whose 60-second deadlock guard ran out while the new 1,000-process test shared its binary. Locally, 25 runs under load found no deadlock. That test moved to a binary of its own, and the guards allow five minutes.
- **Code review:** 6 findings. Four were fixed, one was fixed in part, and one was deferred to backlog T-042, which held its cause already ([code-review.md](code-review.md)). Each fix has a test that fails without it. Finding 6's fix took two rounds of the test critique to finish.
- **Test critique:** three rounds. The first (`block`, 12 concerns) led to `b951d6d`. The second (`block`, 4 concerns) found that the first round's fix for an unreadable lane file covered a lane's top only, and that `verify` offered `praps init` in a lane; that led to `4068f43`. The third, on the final records, is `proceed-with-caveats`: one gap is deferred with its rationale, `praps lane DIR` run inside a lane (backlog T-042), and four record lines were corrected ([critique.md](critique.md)).
- **A record corrected:** the job logs of the samples at `5b65da8` had been read with `gh run view --job <job> --log`, which returns the run's latest attempt's log whatever job is named, so attempt 3's Linux runner version was given for all three attempts. Each job's log is now read by its own id, and [e2e-tests.md](e2e-tests.md) is corrected.

## Technical Debt Identified
- **A lane's directory is tied to its entry by name only** (code review, finding 2). A lane killed while it was being made, and then made again at another directory with the same last name, leaves the first directory driving the same entry. Nothing is lost. Backlog T-042, with two lanes made at once with one name and the lane file's encoding of a store path that is not UTF-8.
- **Reading cost grows with the history.** At 800 operations the new binary is up to 24 percent slower in WSL, because operations and views are larger. Backlog T-030 and T-038.
- **The simulator's thread handoffs are slow on GitHub's Windows runners.** In the nine Windows samples, the scheduler tests' binary took 20.4 to 52.8 s and the 1,000-process test 7.8 to 22.9 s, against 0.4 to 0.9 s and 2.2 to 2.8 s on Linux. The mechanism is now bounded by a test; resumable steps in place of one thread per process are backlog T-032.
- **The Windows suite on GitHub** took 17 min 10 s, 16 min 57 s and 9 min 51 s in the three samples of the tested head, against the aim of 15 and the limit of 20. `materialise_crash` remains the longest unit. Backlog T-027.
- **What T-029 leaves:** signed ownership, a view whose size follows the change, merge cost, 1,000 writers on 100 replicas, the merge's known limit, and the scope of local operations. Backlog T-037 to T-043.
- **Agents that learn `praps` from its help alone** (INT-0037 AC-6) need fresh agents and a hidden task set. Backlog T-044.
- **The lost-op check is shown on ops a failed exchange left behind,** not on an op file that a crash itself loses: no store variant leaves op files unsynced. Backlog T-034.
- **The owner's sign-offs:** INT-0006 AC-6 is signed for this sprint's format change; the rest stay in backlog T-035.
