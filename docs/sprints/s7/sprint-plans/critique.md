# Plan Critique — Sprint 7

Round 3. The same kind of read-only reviewer checked the plans at `0751de2` against its round-2 concerns ([critique-round2.md](critique-round2.md); round 1 is [critique-round1.md](critique-round1.md)) and screened what the round-2 revision changed. It found every round-2 response carried out but two. Its concerns are given as it wrote them; the responses are at the end.

## Concerns

### C-001: The round-2 C-005 response cites a `praps` test that does not exist
- **Where:** `build-plan.md` T-005 and T-006, the error-kind clauses; `test-plan.md` T-004, T-005 and T-006 sections ("In `crates/praps/`") and the T-005 and T-006 traceability rows; `crates/praps/src/hint.rs` tests; `crates/workspace/tests/workspace.rs:1189`.
- **Quote:** "both `messages_say_only_what_is_wrong` tests, the workspace crate's and `praps`'s, **SHALL** fix its text". The test plan, T-006: "In `crates/praps/`: `every_error_kind_has_a_hint`, `messages_say_only_what_is_wrong`, `hints_take_one_of_three_forms`, `hints_name_registry_commands`, with the five new kinds." T-004 and T-005 have the same line.
- **Failure mode:** plan-test-mismatch
- **Why it matters:**
  - `messages_say_only_what_is_wrong` exists only at `crates/workspace/tests/workspace.rs:1189`. `praps` has no test of that name: its `hint.rs` tests are `every_error_kind_has_a_hint`, `hints_take_one_of_three_forms`, `hints_name_registry_commands` and `quoted_text_is_shown_only_when_it_is_plain`, and none of them fixes an error's text.
  - Round-2 C-005 asked for the location to be corrected. The response kept the `praps` listing and added a second test that does not exist. As a result, no build can meet the T-005 and T-006 clauses.
  - A related point for the same clause: the workspace test rejects any error text that contains "praps". A new kind whose text names `.praps/workspace` (INT-0014 says "an error naming it") fails that check unless the check changes, and the plan lists no such change.
- **Suggested response:** fix-in-plan. Name only the workspace crate's test. Drop it from the "In `crates/praps/`" lines of T-004, T-005 and T-006. Either say that the new kinds' texts name no `.praps/…` path, or list the change to the test's check for "praps".

### C-002: T-007's new from-below clause cannot hold for `praps lane DIR` as written
- **Where:** `build-plan.md` T-007, the AC-3 clause added for round-2 C-010, and its first clause; `test-plan.md` T-007 (`commands_act_the_same_from_below`); `crates/praps/tests/cli.rs:1142-1256`; `crates/praps/src/commands.rs:250` (`os_seed()`).
- **Quote:** "**WHEN** `praps lanes`, or `praps lane DIR` with DIR given relative to where it runs, is run at a working copy's top and from below it, **THEN** each **SHALL** print and leave the same (INT-0037 AC-3)". Also: "It **SHALL** print one line naming NAME, DIR and the new change."
- **Failure mode:** EARS-vague
- **Why it matters:**
  - A correct `lane` mints the lane's change ID from `os_seed()`. Its printed line therefore differs from run to run, and so does the operation it commits, whose bytes depend on that change ID.
  - The printed DIR also differs: given relative to two different places, it is spelt differently, and as a full path it differs between the harness's copies.
  - `commands_act_the_same_from_below` masks only each copy's top path in the output, and it compares every file under the top byte for byte. So `lane DIR` fails it against a correct build.
  - The test already handles `new` separately for exactly this reason: it masks the ID and compares the op count, the files outside `.praps`, and what `status` prints next. The plan states no such comparison for `lane`, so Build would have to define the oracle itself.
- **Suggested response:** fix-in-plan. Say how `lane DIR` is compared, as the test does for `new`:
  - mask the change ID and DIR in the printed line;
  - compare the exit code, the number of operations, the files outside `.praps`, the lane's files, and what `praps lanes` prints next.

  `lanes` itself can stay in the byte-for-byte corpus.

## Confidence
proceed-with-caveats

## Responses

| Concern | Response |
|---|---|
| C-001 | **fix-in-plan.** Only the workspace crate's `messages_say_only_what_is_wrong` is named now, in T-005's and T-006's clauses and in the test plan's T-004, T-005 and T-006 rows and sections. The "In `crates/praps/`" lines list `every_error_kind_has_a_hint`, `hints_take_one_of_three_forms` and `hints_name_registry_commands`. T-006's clause, and its test section, add the change to that test's check: it looks for a command (`praps ` or `` `praps``) instead of the bare word. A text may therefore name `.praps/workspace`, as INT-0014 says the error does. |
| C-002 | **fix-in-plan.** T-007 has two clauses now. `praps lanes` stays in the byte-for-byte corpus. `praps lane DIR`, with DIR given relative to each place, is compared as `new` is: the printed line with the change ID and DIR masked, then the exit code, the number of operations, the files outside `.praps`, the lane's files, and what the following `praps lanes` prints. The test plan's line for `commands_act_the_same_from_below` says the same. |
