# Plan Critique — Sprint 7

Round 2. The same kind of read-only reviewer checked the plans at `7321a41` against its round-1 concerns ([critique-round1.md](critique-round1.md)) and screened what the revision changed. Its concerns are given as it wrote them; the responses are at the end. The verdict was `block`, so the plans were revised and reviewed a third time ([critique.md](critique.md)).

## Concerns

### C-001: The reachability oracle fails a correct build when an in-flight command's operation lands
- **Where:** `build-plan.md` T-009, second clause; `test-plan.md` `concurrency_finds_no_violations`; INT-0010 AC-5 (claimed from it).
- **Quote:** "every changeset in each lane's last acknowledged entry, pointer and heads, SHALL be in the merged view on both stores, with everything it follows present". The same clause also says "each lane's writer SHALL be its last acknowledged operation, or an operation of the command in flight that follows it", and the script is "`status`, `describe`, `new`, and `undo`".
- **Failure mode:** EARS-vague
- **Why it matters:** The writer oracle lets the in-flight command's operation land. When it does, the last acknowledged changeset can drop out of the view:
  - a snapshot or `describe` replaces the working-copy changeset with a new version on the same parents (`replace_working_copy` removes the old id from `heads`);
  - `undo` takes `new`'s change back out of the entry.

  The old changeset is then neither in the merged view nor followed by anything in it. So in every cell where a kill or crash falls after `commit_op` and before acknowledgement, a correct build fails. AC-5's "no reachable change" would then rest on an oracle that Build has to redefine.
- **Suggested response:** fix-in-plan. Apply the check to lanes whose writer is their last acknowledged operation. Where an in-flight operation landed, require that it follows the last acknowledged one and that the changesets it dropped stay restorable through the op log.

### C-002: T-006's crash clause cannot hold, and "free" leaves out a file a crash leaves (round-1 C-006, partly fixed)
- **Where:** `build-plan.md` T-006, crash clause and "free" clause; `test-plan.md` `a_crash_while_making_a_lane_is_finished_or_refused`; `crates/workspace/src/record.rs` (`RECORD_NEW`, `drop_record`); `crates/praps/src/hint.rs` `no_repository`.
- **Quote:** "and making the lane at DIR again SHALL succeed." The test reads "In every cell … Making the lane again then succeeds". The free clause reads "Such a `.praps` holds no `workspace` file, or one naming a workspace no operation made, and at most its temporary name and a record beside it."
- **Failure mode:** EARS-vague
- **Why it matters:**
  - In cells where the operation landed (a crash in step 7, or after the last syscall), NAME is already a workspace. T-006's own refusal clause then makes a second making refuse, so the matrix as written fails a correct build.
  - A crash inside step 5's `record::write`, after `create_new(.praps/working-copy.new)` and before the rename, leaves a file that is neither allowed leftover. `drop_record` removes only `working-copy`, so DIR is not free and the second making refuses.
  - Before `.praps/workspace` is in place, the next command at DIR finds no repository and prints today's hint, `praps init`. Following it makes DIR a separate repository, which `praps lane DIR` then refuses. This was the first bullet of round-1 C-006, and it is still open.
- **Suggested response:** fix-in-plan.
  - Scope "making again succeeds" to cells where the operation did not land, and assert the name-in-use refusal where it did.
  - Allow `working-copy.new` among the leftovers.
  - Give the window before `.praps/workspace` exists a hint other than `init`, or record why it keeps that hint.

### C-003: Through the binary, `praps lane DIR` refuses the interrupted lane it is meant to make again
- **Where:** `build-plan.md` T-007, refusal list and top rule, against T-006's interrupted-lane clause; INT-0014 Consequences ("`praps lane` makes it again at the same directory (C-006)").
- **Quote:** "DIR lies inside a working copy, this one or another, found by searching up from DIR with the top rule below"; "A top SHALL be a directory whose `.praps/store` lists or whose `.praps/workspace` reads."; T-006: "Its hint SHALL say to make the lane again at DIR."
- **Failure mode:** hidden-dep
- **Why it matters:** An interrupted lane's `.praps/workspace` reads and parses, so by the top rule DIR is itself a top. A search that starts at DIR, as the commands' searches do and as "at a lane's top" implies for `init`, refuses the very command the hint prescribes. The way back out of an interrupted lane is tested only through the library (`an_interrupted_lane_is_refused_and_made_again`). No `cli.rs` or E2E test makes a lane over an interrupted one, though that test is easy to write.
- **Suggested response:** fix-in-plan. Say that the search starts at DIR's parent, or that an interrupted lane's top does not count. Add a binary test that makes a lane over an interrupted one.

### C-004: INT-0005 AC-2 is claimed for a 10-seed lane matrix with no negative control
- **Where:** `build-plan.md` Intents (INT-0005 line), T-006 and T-009 Notes; `test-plan.md` T-006 `lane_crash.rs` and "The matrices"; INT-0005 AC-2 and Consequences.
- **Quote:** "AC-1 and AC-2 for making a lane, with a crash at every syscall in both modes (T-006)"; the test: "a crash at every syscall of making one lane, and after the last, for 10 seeds"; INT-0005 AC-2: "Sample: at least 40 seeds per scenario."
- **Failure mode:** intent-drift
- **Why it matters:** The C-011 response moved the AC-2 claim onto this matrix, but it runs a quarter of AC-2's sample; every existing matrix uses `SEEDS = 40`. It also has no negative control, though INT-0005's Consequences require one for "each new durable structure" (`.praps/workspace`). Its oracle accepts any of three outcomes in every cell, and nothing checks that an acknowledged lane operation survives. Finally, neither INT-0005's Sprint 7 Consequence nor the Test-phase matrices record mentions this matrix.
- **Suggested response:** fix-in-plan. Run 40 seeds and add a control, for example `.praps/workspace` written in place, or `DIR/.praps` left unsynced. Check durability in the cell after the last syscall, and list the matrix in INT-0005 and in "The matrices". Otherwise claim AC-1 only.

### C-005: `messages_say_only_what_is_wrong` is in the workspace crate, which T-005 and T-006 do not touch (round-1 C-004, partly fixed)
- **Where:** `build-plan.md` T-005 and T-006 Touches and error-kind clauses; `test-plan.md` T-006 and T-005 "In `crates/praps/`"; `crates/workspace/tests/workspace.rs:1189`.
- **Quote:** "WHEN this task adds an error kind, THEN the kind SHALL have a hint in the table, and `messages_say_only_what_is_wrong` SHALL fix its text." The test plan reads "In `crates/praps/`: `every_error_kind_has_a_hint`, `messages_say_only_what_is_wrong`, `hints_take_one_of_three_forms`, with the three new kinds."
- **Failure mode:** hidden-dep
- **Why it matters:** That test lives in `crates/workspace/tests/workspace.rs`. It names every `workspace::Error` variant in a `match` with no catch-all arm and asserts `names.len() == 10`, so the workspace tests stop compiling at the T-005 and T-006 commits. That breaks "Every task's commit passes `cargo ci`". The same file fixes `NothingToUndo`'s text, "the last op is the root op", which T-005 reuses for a lane's first operation, where it is false.
- **Suggested response:** fix-in-plan.
  - Add the file to both tasks' Touches and correct the location in the test plan.
  - List each task's kinds, including T-006's library refusals for a name in use and a DIR that is not free. Say whether "an error of its own for each" means two kinds or three.
  - Say whether `NothingToUndo`'s text changes.

### C-006: T-006's hint names `praps lane` before `lane` is in the registry
- **Where:** `build-plan.md` T-006, interrupted-lane clause; `crates/praps/src/hint.rs` tests `hints_take_one_of_three_forms` and `hints_name_registry_commands`.
- **Quote:** "Its hint SHALL say to make the lane again at DIR." The round-1 response reads "Its hint is to run `praps lane DIR` again from the working copy".
- **Failure mode:** hidden-dep
- **Why it matters:** Both tests require every `Hint::Run`, and every `` `praps X` `` in a hint, to name a command that `registry::find` holds. `lane` (and `lanes`) enter the registry only in T-007, so the hint T-006 adds fails `cargo ci` at T-006's commit.
- **Suggested response:** fix-in-plan. Add that hint in T-007, or word T-006's as a `Fix` that names no command until then.

### C-007: The nested-writer values exist only in the test plan, not in INT-0010 (round-1 C-001, partly fixed)
- **Where:** `build-plan.md` T-003, known-limit clause; `test-plan.md` `merged_copies_and_nested_set_backs_are_pinned`; INT-0010 Consequences and the Transition entry of 2026-11-14.
- **Quote:** "the merge SHALL give the values INT-0010 records for them, pinned by hand: a head that carries a merged copy, and nested writers with a value set back." INT-0010 says only: "The merge can keep a value the owner moved away from, or gain a spurious member."
- **Failure mode:** intent-drift
- **Why it matters:** The chapter gives a history and values only for the merged-copy case. The nested case's history is in neither document, and its values ({1} against {0}, {1, 2} against {2}) appear only in the test plan. Yet the response and the transition entry say the chapter gives "both cases with their values". The clause's oracle points at values the chapter does not hold.
- **Suggested response:** fix-in-plan. Record the nested history and both pairs of values in INT-0010.

### C-008: T-004 allows the Materialise cell count to move for a reason that cannot move it
- **Where:** `build-plan.md` T-004, last clause; `test-plan.md` "The matrices" and review `t004_moves_only_the_materialise_literals`; `crates/workspace/tests/materialise_crash.rs` `matrix_on` and `run`.
- **Quote:** "Its cell count may move too, because resume reads the `from` change without an operation's view, and the matrix crashes recovery at every syscall."
- **Failure mode:** plan-test-mismatch
- **Why it matters:** The asserted 34,400 cells come from `for crash_at in 1..=syscalls + 1` over the uncrashed `run`, which never opens or resumes. Resume affects only the second pass, whose count has no literal. So the allowance would wave through an unexpected syscall change in `undo`, `restore` or `snapshot`. Any count change would also move the Materialise row of `docs/reach.md`, which T-004 does not touch.
- **Suggested response:** fix-in-plan. Keep the Materialise cell count fixed at T-004; only its digest moves.

### C-009: T-005's one-workspace clause contradicts its race clause, and the report for several parents is undefined
- **Where:** `build-plan.md` T-005; `test-plan.md` T-005 rows.
- **Quote:** "WHEN a repository has one workspace, THEN every `undo` and `restore` SHALL leave the same view object as before this task, naming the target's view rather than writing it again." Also: "WHEN `undo` succeeds, THEN it SHALL report the operations it went back to, the parents of the writers it undid."
- **Failure mode:** EARS-vague
- **Why it matters:** In one workspace, two `new` at once (as `eight_new_at_once_all_succeed` does) and then a `describe` leave a writer with two parents. `undo` then sets a merge that is no stored view, and the race clause makes it write a new one. "As before this task" is also unspecified there, since today's `undo` takes the first parent's view. Separately, the report for several parents has no form, and no test checks it.
- **Suggested response:** fix-in-plan. Limit the one-workspace clause to targets that are one operation's own view. Specify the report line for several parents and test it.

### C-010: INT-0037 AC-3 is claimed for `lane` and `lanes`, but neither is run from below
- **Where:** `build-plan.md` Intents (INT-0037) and T-007 Acceptance criterion; `test-plan.md` T-007; `crates/praps/tests/cli.rs:1153` (the corpus of `commands_act_the_same_from_below`).
- **Quote:** "AC-1 to AC-5 for `lane` and `lanes`"; `commands_act_on_the_lane_they_run_in` covers "`status`, `describe`, `new`, `log`, `ops`, `undo`, `restore` and `verify`, at a lane's top and two levels below".
- **Failure mode:** plan-test-mismatch
- **Why it matters:** AC-3 asks every command that opens a working copy to print and leave the same output and state from below. `lanes` opens the working copy and snapshots it, yet neither of these tests runs `lanes` or `lane DIR`. The fixed corpus does not pick new commands up.
- **Suggested response:** fix-in-plan. Add `lanes`, and `lane` with DIR given from each place, to one of the two corpora.

## Confidence
block

## Responses

While the reviewer worked, T-004 was drafted in a scratch copy. That found two tests the plan did not list, and both are added to T-004's test section:
- `a_failed_snapshot_reports_nothing_stale` used a load stopped by two heads to make a command fail before its scan. It now uses a damaged op file instead.
- The workspace crate's `messages_say_only_what_is_wrong` loses its several-heads case.

| Concern | Response |
|---|---|
| C-001 | **fix-in-plan.** The reachability oracle applies to each lane whose writer is its last acknowledged operation. Where the command in flight landed an operation, the oracle requires instead that the operation follows the last acknowledged one. Every changeset the last acknowledged entry held must then still be in the store and named in that entry's view, which the op log keeps, so `restore` can return to it. |
| C-002 | **fix-in-plan.** The crash clause splits on whether the lane's operation landed. If it landed, the lane's files are finished and making the lane again refuses because NAME is in use. If not, the command refuses as an interrupted lane, or finds no repository at DIR. Making the lane again then succeeds. The "free" rule now allows both temporary names, `workspace`'s and the record's (`working-copy.new`), beside the record itself. **The hint before `.praps/workspace` exists is kept, with the reason recorded in INT-0014.** In that window DIR holds at most an empty `.praps` or a temporary file, and nothing in the store names it, so it is no repository. A command there says so, as in any directory. Making the lane again at DIR finishes it, because DIR counts as free. Following `praps init` there instead makes DIR a repository of its own, which is what that command says it does. |
| C-003 | **fix-in-plan.** `praps lane` searches for an enclosing working copy from DIR's parent, not from DIR. DIR itself is judged only by the "free" rule, which accepts an interrupted lane. A new binary test, `praps_lane_finishes_an_interrupted_lane`, writes an interrupted lane's `.praps/workspace` and then makes the lane over it. |
| C-004 | **fix-in-plan.** The lane matrix runs 40 seeds in both modes. In the cell after the last syscall it checks durability: the lane's operation is present, its files are written and no record is left. It gains a negative control, the variant `LaneFileInPlace`, which writes `.praps/workspace` in place without a temporary name or a sync, and must produce at least one violation. INT-0005's Sprint 7 Consequence and the test plan's "The matrices" now list the lane matrix. |
| C-005 | **fix-in-plan.** `crates/workspace/tests/workspace.rs` is now in T-005's and T-006's Touches, and the test plan names both `messages_say_only_what_is_wrong` tests, the workspace crate's and `praps`'s. The kinds are listed per task. T-005 adds one, `restore` of an operation whose view holds no entry for this workspace. T-006 adds five: a name in use; a DIR that is not free; an interrupted lane; a store that does not open; and a `.praps/workspace` that cannot be read or does not parse, one kind with its reason. `NothingToUndo`'s text changes in T-005 to "nothing to undo: this workspace's last op is the one that made it". That is true of the root op, which made `default`, and of a lane's first operation. |
| C-006 | **fix-in-plan.** T-006 gives the interrupted lane a `Fix` hint that names no command: make the lane again at that directory from the working copy. T-007 turns it into a `Run` of `praps lane DIR` once `lane` is in the registry. |
| C-007 | **fix-in-plan.** INT-0010's known-limit bullet now gives the nested history in full, with both pairs of values: {1} where the classic merge gives {0}, and with the third writer at 2, {1, 2} where it gives {2}. |
| C-008 | **fix-in-plan.** At T-004 the Materialise cell count stays at 34,400 and only its digest moves. The T-004 clause, the review and "The matrices" now say so, and no task may move a cell count. |
| C-009 | **fix-in-plan.** The one-workspace clause is limited to targets that are one operation's own view: there `undo` and `restore` name that view and leave the same view object as before. When the target is a merge of several operations' views, the command writes it as a new view. With several parents, `undo` reports `undid the last op: the working copy is back to ops X, Y`, every parent in id order. With one parent it prints today's line. `undo_after_a_resolved_race_keeps_one_change` checks the several-parents line through the library's return value, and `praps_undo_reports_every_op_it_went_back_to` checks it through the binary. |
| C-010 | **fix-in-plan.** `commands_act_the_same_from_below` gains `lanes`, and `lane` with DIR given relative to each place it is run from. INT-0037 AC-3 is then claimed for both. |
