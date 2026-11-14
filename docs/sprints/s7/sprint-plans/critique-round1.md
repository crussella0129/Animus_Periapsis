# Plan Critique — Sprint 7

Round 1. A read-only subagent reviewer critiqued the plans at `9b2734e` against the Book and the code. Its concerns are given as it wrote them; the responses are at the end. The verdict was `block`, so the plans were revised and reviewed again ([critique.md](critique.md)).

## Concerns

### C-001: AC-2 does not hold for the per-owner merge T-003 defines
- **Where:** `build-plan.md` T-003 (AC-2 clause) and Intents (INT-0010 line); `test-plan.md` T-003 unit tests, `two_heads_equal_the_three_way_merge`; INT-0010 Transition history, 2026-11-14.
- **Quote:** "WHEN two heads with one latest common ancestor are merged, THEN each entry SHALL equal the classic three-way merge of the two head views against that ancestor's view"
- **Failure mode:** intent-drift
- **Why it matters:** The T-003 generator can produce a history that breaks this clause. From root r, W commits c1 (pointer {a1}) and, at the same time, wB ({a2}). The operation that makes lane Y is committed over {c1, wB}, so it carries W as {a1, a2}. W then commits wA ({a3}) over c1 only. For heads {wA, y}, the only latest common ancestor is c1. The classic merge against c1 gives W = {a1, a2, a3}. T-003 takes W's writers {wA, wB} against r and gives {a2, a3}, because W's own wA replaced a1. A head that carries a merged copy is a committed partial merge, so AC-2 has the same limit as AC-3, yet AC-2 is claimed with no restriction. R4's compat proof and R5's "conv ∧ compat rejects 6/6" were shown for the prototype that merged against the lowest common ancestor, not for this merge. As written, `two_heads_equal_the_three_way_merge` will fail against the plan's own merge, and INT-0010 already records AC-2 as covered.
- **Suggested response:** fix-in-plan.
  - Restrict AC-2's clause, generator and claim to two heads in each of whose ancestries every entry has a single writer. Equality holds there.
  - Record that limit in INT-0010 next to AC-3's.
  - Re-check AC-6 against the narrowed test.
  - Pin nested-writer cases by hand, with their expected values, in `entries_merge_by_their_rules`. AC-4's reference is "written in the test from the definition alone", so it cannot catch an error in the definition itself.

### C-002: The two new points are registered before any scenario reaches them
- **Where:** `build-plan.md` T-003 and T-006 (point clauses; Touches `crates/workspace/src/points.rs`, no `docs/reach.md`); T-009 (reach clause).
- **Quote:** "it SHALL mark the point `workspace.merge.computed`, registered for unit `workspace::merge` in Sprint 7" … "The point is a scheduling boundary in simulation; T-009's scenario reaches it."
- **Failure mode:** hidden-dep
- **Why it matters:** `cargo policy` (rule `points`) requires every marked point to be registered. The reach table (`crates/xtask/src/reach.rs`, `table`) fails with "point … is reached by no scenario" for any registered point that no scenario reaches. None of the six existing scenarios merges workspace heads or makes a lane, so `cargo ci`'s `reach` step fails at every commit from T-003 to T-008. `docs/reach.md`, whose Points table lists every registered point, is in neither T-003's nor T-006's Touches.
- **Suggested response:** fix-in-plan. Mark and register both points in T-009 together with the Concurrency row, or land a scenario that reaches them in the same task. State which task commits must pass `cargo ci`.

### C-003: T-005's tests need lanes, which T-006 builds after it
- **Where:** `test-plan.md` T-005 unit tests; `build-plan.md` T-005 and T-006 "Depends on".
- **Quote:** "`undo_is_scoped_to_its_workspace`: lanes A and B each commit" … "right after lane B is made, `undo` in B fails" … T-006 "Depends on: T-005"
- **Failure mode:** hidden-dep
- **Why it matters:** Four of T-005's five new tests need a second workspace with its own directory, opened by name. So does its clause about "the other's entry and files". Only T-006 provides that: `Workspace::open` on `.praps/workspace`, and the lane operation that is "the operation that made W". In the current order, T-005 cannot be verified at its own commit.
- **Suggested response:** fix-in-plan. Make T-006 depend on T-004 and T-005 depend on T-006. Then add T-005 to T-009's dependencies, because T-009's script runs `undo`. Alternatively, say how T-005's tests build and open a second workspace without T-006.

### C-004: New error kinds break the `praps` build and have no hint clause
- **Where:** `build-plan.md` T-005 and T-006 (Touches), T-007 hint clause; `crates/praps/src/hint.rs`.
- **Quote:** T-005 "`restore` SHALL refuse"; T-006 "the command SHALL refuse, naming the interrupted lane"; T-007 "WHEN a refusal of either command is printed"
- **Failure mode:** hidden-dep
- **Why it matters:** `hint::for_error`, and the test helper `variant`, match on `workspace::Error` "with no catch-all arm, so an error added there does not compile until it has one". A variant added in T-005 or T-006 therefore breaks the `praps` build at that commit, and neither task lists `hint.rs`. T-007's clause covers only refusals of `lane` and `lanes`. That leaves three refusals with no clause and no test through the binary (INT-0037 AC-4):
  - the `restore` refusal;
  - the interrupted-lane refusal, which any command in DIR prints;
  - a lane whose store is gone. Today's `open` maps that to `NotInitialised`, whose hint is `praps init`, a command T-007 refuses inside a lane.
- **Suggested response:** fix-in-plan. Add `hint.rs` and the hint and message tests to T-005's and T-006's Touches, or put the lane errors in a type of their own. Give every new kind a hint and a case in `every_refusal_prints_one_hint`.

### C-005: `undo` and `restore` can commit a pointer that holds several changesets
- **Where:** `build-plan.md` T-005 against T-004's race clause.
- **Quote:** T-005 "return W's entry to its value just before them: the merge of their parents"; "W's entry SHALL become its value in OP's view"; T-004 "Once it commits, the pointer SHALL hold one changeset."
- **Failure mode:** EARS-vague
- **Why it matters:** Take a race {a, b} that W's next `describe` resolves, as an operation d over both racing operations. `undo` then returns W to the merge of d's parents, which is the pointer {a, b}. `restore` of any operation whose copy of W was merged does the same. Both therefore commit the several-changeset pointer that T-004 says a commit never leaves. The plan does not say which tree is then written to disk (writing the working copy takes one tree), or which operation `undo` reports ("back to op X") when the value is a merge of several. No T-005 test starts from a raced pointer.
- **Suggested response:** fix-in-plan. Either apply the race rule to the target as well (write and keep the smallest, the others among the heads) or refuse with a hint. Align T-004's clause with that choice. Add tests for `undo` after a resolved race and for `restore` of an operation that holds a merged copy.

### C-006: Making a lane has crash windows the clause cannot meet
- **Where:** `build-plan.md` T-006 steps and crash clause; T-004 record and resume clauses; `test-plan.md` `a_crash_while_making_a_lane_is_finished_or_refused`.
- **Quote:** "3. write `DIR/.praps/workspace`, naming NAME and the store, durably;" … "if not, the command SHALL refuse, naming the interrupted lane, with nothing more written." vs T-004 "WHEN the target is absent, THEN resume SHALL drop the record and touch no file."
- **Failure mode:** hidden-dep
- **Why it matters:** Creating DIR and DIR/.praps is not among the seven steps. A crash after they exist but before `.praps/workspace` is in place leaves a directory that is neither outcome the clause allows:
  - it is not a top, so the next command there finds no repository and suggests `praps init`;
  - it is not empty, so `praps lane DIR` refuses it;
  - no command can finish or remove an interrupted lane.

  Two further conflicts:
  - When the record exists but the operation did not land, T-004's resume drops the record before refusing, which contradicts "nothing more written".
  - T-004's record names "the working-copy changeset it starts from", so it cannot express a `from` of "the empty tree", and T-006 does not define that form.
- **Suggested response:** fix-in-plan.
  - Add the directory steps.
  - State what a half-made DIR does at the next command, and how the user gets out, with its hint.
  - Reconcile the resume rule for a lane's record.
  - Define the record's empty-tree form in the spec.

### C-007: Two working-copy writes at once in one workspace
- **Where:** `build-plan.md` owner decisions and T-004 race clause; `crates/workspace/src/record.rs`; `test-plan.md` `a_raced_pointer_continues_from_the_smallest`.
- **Quote:** "after two commands at once in one workspace, the next command continues from one change and keeps the other"; test: "two `new` from one state leave the pointer at two changes"
- **Failure mode:** missing-risk
- **Why it matters:** `record::write` starts by removing `.praps/working-copy.new`. So two `undo` or `restore` commands running at once in one working copy can do damage in three ways:
  - One can delete or rename the other's half-written record, and that command fails with an I/O error. That is a command failing because of another writer.
  - One can drop the other's record, leaving a half-written working copy that the next command snapshots as user edits.
  - When the files on disk are the other command's target, "continue from the smallest id" records them as edits to the chosen change.

  The race is tested only with `new`, which writes no file. Research's note on Jujutsu (a working copy records the operation it was last brought up to, and a stale one is detected) has no task, test or deferral.
- **Suggested response:** fix-in-plan. Use per-process record names, or another scheme two writers cannot overwrite. Add a rule for which change to continue from when the disk holds one side. Add a test of two working-copy writes at once in one directory, both simulated and through the binary. Or defer with a rationale recorded in INT-0014.

### C-008: Two lanes made at once with the same name share one entry
- **Where:** `build-plan.md` T-006 refusal clause, T-007 refusal list, T-003 base rule.
- **Quote:** "WHEN NAME is already a workspace, or DIR exists and is not empty, THEN making the lane SHALL refuse" … "or absent when there is none."
- **Failure mode:** missing-risk
- **Why it matters:** The check reads the merged view before the lane operation commits. Two `praps lane` commands started together with the same last directory name therefore both pass it. Their NAME operations have no common NAME ancestor, so the merge unions their pointers and two directories end up bound to one entry. The next command in either directory continues from the smaller change and snapshots its own files over the other's. Agents name lanes after directories, so collisions are likely, and every planned test makes lanes one at a time.
- **Suggested response:** fix-in-plan. For example, have `.praps/workspace` name the operation that made the lane, and refuse in a directory whose entry was made by another operation. Test two lanes made at once with one name. Or defer with a rationale.

### C-009: Where a lane may be made, and what counts as a top
- **Where:** `build-plan.md` T-007 refusal list, top clause and Notes; INT-0014 Consequences.
- **Quote:** "DIR lies inside this working copy;" … "A top SHALL be a directory whose `.praps/store` lists or whose `.praps/workspace` reads. `init` SHALL be refused inside a lane." … INT-0014: "The top of a working copy is the directory that holds `.praps/store`."
- **Failure mode:** intent-drift
- **Why it matters:**
  - A DIR inside another lane, or inside any other working copy, is accepted, although T-007's own reason ("would have its files snapshotted by that working copy") applies. A nested `.praps` that holds no store is tracked as content, so the outer lane would track the inner lane's `.praps/workspace` and record, and its `undo` could remove them.
  - `init` searches only the directories above it (`place::above` skips the directory itself). So `praps init` at a lane's top would make a store inside the lane's `.praps`, a case "inside a lane" leaves open.
  - T-007 redefines a top, contradicting INT-0014's "the directory that holds `.praps/store`", without revising the chapter.
  - A `.praps/workspace` that exists but cannot be read or parsed is left undefined. Sprint 6's rule stops the search on any failure other than absence.
- **Suggested response:** fix-in-plan.
  - Refuse a DIR that lies inside any working copy, by searching upward from it with the new top rule.
  - Refuse `init` at a lane's top as well as below it.
  - Revise INT-0014's definition of a top.
  - State how an unreadable or malformed `.praps/workspace` is treated.

### C-010: INT-0009 AC-6 is built but neither verified nor deferred
- **Where:** `build-plan.md` Intents (INT-0009 line) and Deferred; INT-0009 AC-6 and Consequences.
- **Quote:** "AC-1 to AC-3 kept, with `undo` and `restore` scoped to the workspace that runs them (T-005)"; INT-0009: "The scoped form (AC-6) arrives with the merge of op heads"
- **Failure mode:** intent-drift
- **Why it matters:** T-005 builds exactly what AC-6 measures: with two writers interleaved, over at least 200 seeded interleavings, `undo` and `restore` by one leave every entry the other owns as the merge left it. The only check is the single case `history_in_one_lane_leaves_the_other`, and the Concurrency script runs `undo` with no oracle on other lanes' entries. AC-6 is in neither the claimed criteria nor the Deferred list.
- **Suggested response:** fix-in-plan. Add that oracle to T-009, or a seeded two-writer property of at least 200 interleavings to T-005, and claim AC-6. Or defer it with a reason.

### C-011: T-009 overclaims INT-0014 AC-3, INT-0005 AC-2 and INT-0010 AC-5
- **Where:** `build-plan.md` Intents (INT-0014 and INT-0005 lines), T-009; `test-plan.md` `eight_lanes_share_one_store`.
- **Quote:** "AC-3 (at least 8 workspaces over one repository, in T-006 and T-009, and through the binary in T-010)"; "8 lane processes run at once, four on each of two stores"; "8 lanes made, each running an edit, `status`, `describe` and `new` in turn"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** INT-0014 AC-3 asks for at least 8 workspaces working at once over one repository, "without clones, without duplicated objects", with 0 violations in simulation. None of the planned tests shows that:
  - T-006 runs its 8 lanes in turn.
  - T-009 runs 4 lanes per store, on two stores that each hold every object after the exchange.
  - T-010 is not a simulation, and checks neither objects nor lost operations.

  INT-0005 AC-2 (a crash "injected at every syscall") is claimed for a scenario that crashes at 40 seeded steps. INT-0010 AC-5's "no reachable change" has no oracle among T-009's six.
- **Suggested response:** fix-in-plan.
  - Add a run with 8 lane processes at once on one store, checking one file per object and every acknowledged operation present.
  - Claim INT-0005 AC-2 only if a reduced script crashes at every syscall.
  - Add a reachability oracle, or narrow the AC-5 claim.

### C-012: Matrix literals and test names
- **Where:** `test-plan.md` T-002 and T-004 rows, "The matrices"; `build-plan.md` T-004 Touches.
- **Quote:** "the Materialise matrix, unchanged in what they assert"; "Each matrix's cell count at `855eb67` and at the tested head, recorded; they must be equal."; "`golden_files_match`, `golden_files_match_the_spec`"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** Three mismatches:
  - A Materialise cell's state digest includes `repo/.praps/working-copy` whenever a crash leaves the record. T-004's new `from` therefore moves the digest literal in `materialise_matrix_is_reproducible`, yet that matrix is listed as "unchanged in what they assert" and `materialise_crash.rs` is in no task's Touches.
  - The Test-phase rule that cell counts equal those at `855eb67` also requires T-004 to T-007 to add no syscall (scoped `undo` and `restore`, `open` probing `.praps/workspace`). None of their clauses says so.
  - The tests named for T-002, `golden_files_match` and `golden_files_match_the_spec`, do not exist. The real ones are `golden_v1_corpus_matches`, `every_golden_file_is_a_corpus_case`, `golden_corpus_has_the_spec_types` (whose type list needs a `view-conflict` row) and `spec_examples_match_golden_files`.
- **Suggested response:** fix-in-plan.
  - List which matrix literals each task may move (T-004: the Materialise digest).
  - Either add the no-new-syscall requirement to the clauses of T-004 to T-007, or allow cell counts to move with a recorded reason.
  - Name the real golden tests, with their changed assertions.

### C-013: T-007 needs workspace-crate code it does not list
- **Where:** `build-plan.md` T-007 Touches and the `ops` and `lanes` clauses; `crates/workspace/src/lib.rs` `Workspace::ops`.
- **Quote:** "WHEN `praps ops` lists operations, THEN each line SHALL name the workspace that made the operation." … "It SHALL write nothing and repeat exactly."
- **Failure mode:** hidden-dep
- **Why it matters:** `Workspace::ops` builds every line from the `default` workspace's pointer, and `OpEntry` names no workspace. Naming each operation's workspace, showing that workspace's change rather than `default`'s, and listing every entry for `lanes` all need workspace-crate code. T-006's clauses do not define it, and T-007's Touches list only `praps` files. A `lanes` that "SHALL write nothing" cannot snapshot, which makes it a third exception to INT-0014's "Every command that opens the working copy snapshots it first". Its path count for the current lane then leaves out unrecorded edits, and the chapter is not revised.
- **Suggested response:** fix-in-plan. Add `crates/workspace/src/lib.rs` (or `lane.rs`) to T-007, with clauses for the listing and for showing the operation's own workspace's change. Record `lanes` beside `verify` and `init` in INT-0014.

### C-014: A lane's base makes a divergent change, and lanes are not the cheap workspace
- **Where:** `build-plan.md` T-006 step 2; INT-0014 Intent ("A workspace is cheap") and Consequences (Lanes).
- **Quote:** "2. start a new empty change on top of W's working-copy change;"; INT-0014: "Making one writes one view entry and no working-copy file"
- **Failure mode:** intent-drift
- **Why it matters:** Every snapshot and `describe` in W replaces W's working-copy change with a new changeset under the same change ID. After any later edit in W, the lane's change therefore sits on a superseded version, and two visible changesets share one change ID. That is a divergence (INT-0011) made by every lane, not only by races. Basing the lane on the parent of W's change, which W does not rewrite, is not discussed, and the choice and its consequence exist only in the plan.

  `praps lane` also writes every file into DIR, against INT-0014's rule for a cheap workspace. The new Lanes Consequence does not say that lanes are the checkout form and that the cheap form is still to come.
- **Suggested response:** fix-in-plan. Record the base choice and its consequence in INT-0014, or base the lane on the parent of W's change. Record that `lane` makes a checkout workspace, not the cheap one.

### C-015: The simulated replicas exchange local operations
- **Where:** `build-plan.md` T-009 Notes and Deferred ("Real replicas"); INT-0009 Intent ("Operations are local or shared"); INT-0010 Intent ("Scope").
- **Quote:** "Their exchange copies what each lacks through the store's own `put_object` and `commit_op`"; INT-0010: "never another replica's local operations"
- **Failure mode:** intent-drift
- **Why it matters:** By INT-0009's definition, every operation Sprint 7 makes (snapshot, `describe`, `new`, `undo`, `restore`) is local and "stays on its replica". T-009's oracle that every lane on either store computes the same view holds only because all of them are exchanged. Under the per-owner rule, an entry whose owner's operations a replica lacks is "absent, with no writer". So the Scope rule and this merge interact in a way no chapter records.
- **Suggested response:** fix-in-plan. Record in INT-0010 and INT-0009 that Sprint 7's replicas exchange every operation because operations carry no scope yet. Note in the backlog what the per-owner rule implies once local operations stay on their replica.

### C-016: "The same lines from every lane" contradicts the `*` marker
- **Where:** `build-plan.md` T-010 first clause against the T-007 `lanes` clause; `test-plan.md` E2E `lanes_at_once_end_to_end`.
- **Quote:** "`praps lanes` SHALL then print the same lines from every lane" vs "`*` for this one"
- **Failure mode:** EARS-vague
- **Why it matters:** T-007 marks the running lane's own line with `*`, so `praps lanes` cannot print the same lines in eight different lanes. As written, `lanes_at_once_end_to_end` either fails or compares something the clause does not name.
- **Suggested response:** fix-in-plan. Say the lines are the same apart from the `*`, which marks each lane's own line.

## Confidence
block

## Responses

While the reviewer worked, drafting T-003's tests found the same gap as C-001 from another side. Those findings are folded into the C-001 response below, together with two more:
- **Writers nested under a set-back value.** Take an entry with three writers whose ancestry nests, one of which sets the value back. With two heads and one latest common ancestor, the merge reads that writer as unchanged against the older base, where the classic merge reads a change against the newer one.
- **An equivalent mutant.** The registry's "keep-duplicates" wrong merge cannot differ from the merge: writers are a set, a pointer is a union, and the heads rule asks "every writer" and "some writer". A repeated head changes none of these.

| Concern | Response |
|---|---|
| C-001 | **fix-in-plan.** AC-2's clause, test and claim are narrowed as suggested: to two heads, neither of which carries a merged copy. In each head's own history, every workspace has one latest operation. The merge then equals the classic three-way merge, because the ancestor's copy of an entry is exactly the writers' base. INT-0010's known-limit bullet now gives both cases with their values. The first is a head that carries a merged copy: in the reviewer's example the merge gives {a2, a3} and the classic merge {a1, a2, a3}, because W's own wA replaced a1. The second is nested writers with a value set back. It also gives the reason the merge does not group by heads to match the classic merge: then a commit by another workspace over the same heads would change W's merged value, which the ownership rule forbids. AC-3 is narrowed the same way, to writers that diverge from one base whose pointer holds one changeset. With a conflicted base resolved both ways, a partial merge can equal the base and read as unchanged. **AC-6 re-checked:** AC-1 still runs over every history and AC-2 over the narrowed pairs. "keep-duplicates" is replaced by **union of heads**, in which a head one writer removed stays because another kept it. The five then split as R5's table did: AC-2 alone misses pairwise fold and the fast path; AC-1 alone misses always-base and union of heads; first-side-wins fails both. **Hand-pinned:** a new test, `merged_copies_and_nested_set_backs_are_pinned`, holds the reviewer's example and the two nested cases with their expected values. |
| C-002 | **fix-in-plan.** Both points are marked and registered in T-009, with the Concurrency row and `docs/reach.md`, so `cargo ci`'s `reach` step passes at every commit. T-003 and T-006 lose their point clauses. `merging_marks_its_point` and a new `making_a_lane_marks_its_point` move to T-009. The build plan now says that every task's commit passes `cargo ci` on Windows. |
| C-003 | **fix-in-plan.** T-006 now depends on T-004, T-005 on T-006, and T-007 on T-005. T-009 depends on T-005 and T-008. The build order is T-001, T-002, T-003, T-004, T-006, T-005, T-007, T-008, T-009, T-010. The task numbers keep their anchors in the chapters. |
| C-004 | **fix-in-plan.** T-005 and T-006 now touch `crates/praps/src/hint.rs` and `crates/praps/tests/cli.rs`. Each gains a clause: every error kind it adds has a hint in the table, its text is fixed by `messages_say_only_what_is_wrong`, and `every_refusal_prints_one_hint` has a case for it through the binary. T-005 adds `restore` of an operation that holds no entry for this workspace. T-006 adds three: the interrupted lane; the lane whose store is gone, whose hint no longer offers `praps init`; and an unreadable or malformed `.praps/workspace`. |
| C-005 | **fix-in-plan.** The race rule applies to the target as well. When `undo` or `restore` would set a pointer that holds several changesets, it writes the smallest and commits that one. The others stay among the heads, and the command prints the race note. T-004's clause now reads: every operation a command commits holds one changeset in its own pointer. `undo` reports the operations it went back to, the parents of the writers it undid; with one parent that is today's line. New tests: `undo_after_a_resolved_race_keeps_one_change` and `restore_of_a_merged_copy_keeps_one_change`. |
| C-006 | **fix-in-plan.** T-006's steps now include making DIR and `DIR/.praps` durable, then writing `.praps/workspace` through a temporary name and a rename. A DIR that holds nothing but such a `.praps` counts as empty, so `praps lane DIR` makes the lane there again. A DIR whose `.praps/workspace` names a workspace no operation made is an interrupted lane. Every command there drops the record, if one is left (T-004's rule for an absent target), and refuses with nothing else written. Its hint is to run `praps lane DIR` again from the working copy, which makes the lane over the interrupted one. The record gains its second form, `from empty`, in T-004 (code, spec and test), and T-006 is its first user. |
| C-007 | **deferred, with the rationale in INT-0014.** Two commands that write one working copy's files at the same moment (`undo`, `restore`) can still interfere through the record and the files. One working copy is one agent's, and agents that work at once each take a lane. The race rule covers commands that move the pointer without writing files (`new`, `describe`, a snapshot), and those are what the tests run at once in one directory. Making two file writers safe needs per-process records or a lock, and research's note on Jujutsu (a working copy that knows the operation it was last brought up to) is the starting point. It goes to the backlog with T-029's remainder. |
| C-008 | **deferred, with the rationale in INT-0014.** Two `praps lane` commands started at the same moment with one name both pass the check, and both directories then share one entry. Lanes are made by whoever sets them up, one at a time, and every test does so. Detecting the collision needs each lane to know the operation that made it, and it goes to the backlog. |
| C-009 | **fix-in-plan.** `praps lane` refuses a DIR inside any working copy, found by searching upward from DIR with the new top rule. `init` is refused at a lane's top as well as below it. INT-0014's definition of a top is revised: a directory that holds `.praps/store` or a readable `.praps/workspace`. A `.praps/workspace` that cannot be read or parsed stops the search with an error naming it, as Sprint 6 stops it on any failure other than absence, and its hint says what to put right. |
| C-010 | **fix-in-plan.** T-005 gains `undo_is_scoped_over_200_interleavings`. Two lanes run 200 seeded interleavings of edits, `describe`, `new`, `undo` and `restore`. After each `undo` or `restore` by one lane, every entry the other owns must equal what the merge gave just before. INT-0009 AC-6 is claimed from it. |
| C-011 | **fix-in-plan.** T-009 gains a one-store run: 8 lane processes at once over one store, with two oracles, one file per object and every acknowledged operation present. INT-0014 AC-3 is claimed from that run. INT-0005 AC-2 is claimed only for T-006's matrix, which crashes at every syscall of making a lane in both modes, not for the Concurrency scenario's seeded crashes. T-009 gains a reachability oracle for INT-0010 AC-5: every changeset in each lane's last acknowledged entry, pointer and heads, is in the merged view on every store, with all it follows present. |
| C-012 | **fix-in-plan.** T-004 now touches `materialise_crash.rs`, and its clause lets the Materialise matrix's digest move, because the record's `from` changed. Its cell count may move too, because resume no longer reads the `from` operation's view, and the new count is recorded with that reason. The second pass crashes recovery at every syscall, so a read fewer is cells fewer. T-005 to T-007 each gain the clause that the five matrices keep their cell counts and digests. For T-006, `open` probes `.praps/workspace` only where `.praps/store` is absent, so opening a main working copy makes no new syscall. The Test-phase rule now reads: a cell count equals its value at `855eb67` unless a task recorded the reason it moved. T-002's tests are named as they are, and `golden_corpus_has_the_spec_types` gains the `view-conflict` row. |
| C-013 | **fix-in-plan.** T-007 now touches `crates/workspace/src/lib.rs`. `OpEntry` names the operation's workspace, and its change is that workspace's pointer in the operation's view, the smallest if it holds several. `Workspace::lanes` lists every entry. `lanes` snapshots like every other command, so INT-0014 gains no exception; with no edit it writes nothing and repeats exactly. |
| C-014 | **fix-in-plan.** A lane's first change is made on the parent of W's working-copy change, which W's commands do not rewrite. When that change has no parent, the lane's change has none either, and the lane starts from the empty tree. W's edits in progress are not in the lane. INT-0014 records this choice and its reason, and that `lane` makes a checkout workspace, with every file written. The cheap, API-only form (AC-4) is still to come. |
| C-015 | **fix-in-plan.** INT-0010 and INT-0009 record that Sprint 7's simulated replicas exchange every operation, because operations carry no scope yet. They also record what follows once local operations stay on their replica: an entry whose owner's operations a replica lacks is absent there. A shared view therefore needs each owner's shared operations. The backlog carries it with backlog T-012's scope fields. |
| C-016 | **fix-in-plan.** T-010's clause now says that `praps lanes` prints the same lines in every lane apart from the `*`, which marks each lane's own line. |
