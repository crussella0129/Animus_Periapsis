# Test Critique — Sprint 6

Round 2, the final critique, on the records at `acdbf20` with the tested head `bbffce6`. A separate, read-only critic wrote it; it ran nothing but read-only `git` and one `gh run view`. Round 1, whose verdict was `block`, is [critique-round1.md](critique-round1.md). The responses at the end are the primary agent's.

## Concerns

### C-001: The breakdown of the 32 kinds does not add up, though "31 of 32" is right
- **Where:** `unit-tests.md`, "Where the tests differ from the test plan", second bullet / `crates/praps/tests/cli.rs` `every_refusal_prints_one_hint`.
- **Quote:** "Of the 32 kinds in the hint table: 25 are made on both systems … The table has two rows the corpus counts once each way: "usage error" is made twice"
- **Failure mode:** evidence-drift
- **Why it matters:** 25 + 5 + 2 + 1 is 33, not 32, and the reconciling sentence does not close the gap. The test makes 25 refusals on both systems, but they are 23 table rows: "usage error" is made twice, and so is "a word that is not a command" (`statsu` and `commit`). The true count is 23 rows on both, 5 on Linux only, 2 on Windows only, 1 in `describe_with_unquoted_words_hints_quotes`, and 1 nowhere. So 31 of 32 through the binary holds; "25 kinds on both" does not.
- **Suggested response:** tighten-assertion (wording only): say 25 refusals covering 23 rows, and name both doubled rows. In the last paragraph, "changed five hints and added the row" should read four hints changed and one row added.

### C-002: "31 of 32" counts table rows; three call sites that print those rows' hints are run by no test
- **Where:** `crates/praps/src/commands.rs` lines 149, 561 and 609 / `code-review.md`, last paragraph / `unit-tests.md`, same bullet / INT-0037 AC-4 threshold.
- **Quote:** "Two things it does not reach: `verify`'s own arm for a filesystem failure while reading a store it has chosen"
- **Failure mode:** EARS-coverage
- **Why it matters:** AC-4 names "every way `verify` and finding the working copy can fail". Three ways are covered only because another call site shares their hint function:
  - `must_exist`'s catch-all (line 149): a given directory that exists but cannot be listed.
  - `store_for` (line 561): `verify` where `DIR/.praps/store` cannot be listed. The corpus makes this for `status` only.
  - `verify`'s `(false, _)` arm (line 609).

  Only the third is recorded, and only in `code-review.md`. This is the same class as round 1's C-001 (call site to hint, bound by hand), at lower risk because all three go through the shared `refuse`.
- **Suggested response:** defer-with-rationale: name the three in `unit-tests.md` and the test report beside the one undone kind, and add them to T-036.

### C-003: The one kind made nowhere: the deferral is honest, but its stated reason is narrower than the facts
- **Where:** `unit-tests.md`, same bullet / `code-review.md`, second reader's row "A top that cannot be entered" / `commands.rs` lines 220 to 226 / `docs/work/tasks.md` T-036.
- **Quote:** "one is made on neither: a top that cannot be made the current directory. Nothing in the test can close a directory to entry without closing its store too."
- **Failure mode:** intent-coverage
- **Why it matters:** That is true for file modes. The second reader names two other causes in the same records: a top removed in between, and "on Windows, a very long path". The second may be reachable in a test. So "cannot be made" is really "not made this sprint". The kind's evidence is the unit test of the hint's form, a second reader's reading of the call site, and the shared `refuse` path that 31 other rows exercise. The second reader also judged its hint the least apt.
- **Suggested response:** defer-with-rationale, as done, with two conditions on the test report:
  - State AC-4 as shown through the binary for 31 of 32 kinds and by form and reading for the last. Do not claim the 100% threshold as met without that qualifier.
  - Reword the reason to "not by file modes; a Windows long path was not tried".

### C-004: `test-matrix-log.md` still calls the WSL checkout "a fresh clone"
- **Where:** `test-matrix-log.md`, second paragraph, against `integration-tests.md`, first paragraph.
- **Quote:** "WSL Ubuntu (ext4, a fresh clone) printed the same 39 lines"
- **Failure mode:** evidence-drift
- **Why it matters:** The lines were printed at `4152526`. `integration-tests.md` now says the clone was "made fresh for the first run, at `80a3e7e`, and then checked out at `4152526`". This is the residue of round 1's C-006, corrected in one record and not the other.
- **Suggested response:** tighten-assertion (wording).

## Round-1 concerns, status at `bbffce6`

- **C-001: resolved with an honest deferral.** `every_refusal_prints_one_hint` now makes `Merge` on both systems and, under `#[cfg(unix)]`, the four mode-closed kinds; each was traced to its call site. `assert_eq!(kinds, expected)` with 30 and 27 replaces `>= 25`, and `assert!(!root)` fails the test rather than skipping it. One kind is deferred; see new C-003. The locked T-011 clause 1 does not list that kind, so the EARS list is fully executed. Only AC-4's wider threshold has the gap.
- **C-002: resolved, on Linux only, and stated as such.** A regression to searching upward would exit 0 with `clean` and fail the test.
- **C-003: resolved with an honest deferral.** `code-review.md` holds a second reader's verdicts on the five rows; each is accepted with a caveat. The serious caveat is described plainly, attributed to `init`'s behaviour from before this sprint, and is the first item of T-036. AC-4 asks for the form and one of three remedies, and T-011 clause 5 asks that the review read the table. Both are met. No data is destroyed.
- **C-004: resolved.** With the arm removed, the note arm would run and every assertion of `a_crash_is_not_taken_for_a_refusal` would fail. The "run once by hand" claim cannot be checked from the repository, but the reading supports it.
- **C-005: resolved.** The rows sum to 79, and 79 + 4 + 5 = 88. `git grep '#[test]'` under `crates` gives 475 at `b0675f6` and 563 at `bbffce6`: 3 above the record's 472 and 560 on both sides, so the difference of 88 is unaffected.
- **C-006: resolved, with one residue (new C-004).**
- **C-007: resolved.** Eight command lines, each for a missing directory and for a file.
- **C-008: resolved.** A committing `snapshot` is traced.
- **C-009: resolved with an honest deferral.** The column is headed as an average.

## Numbers checked and found to agree
- **Totals:** 588 and 590 in every record; 587 and 589 at the two earlier heads; 501 + 87 and 502 + 88.
- **Hint table:** 32 rows, 10 + 22 as `hint.rs` asserts, split 17/12/3.
- **GitHub run 42867476706:** head `bbffce6`, event `pull_request`, attempt 1, `success`, and the five job ids and times as `e2e-tests.md` gives them.
- **`bbffce6` changes only files under `tests/`.**

## Confidence
proceed-with-caveats

No intent criterion or EARS promise rests on a false pass. The locked T-011 list of kinds is executed through the binary in full, on Linux for the mode-closed kinds. The AC-4 gap is one kind plus three sibling call sites, each covered by form, by reading and by the shared `refuse` path. The `init` caveat is behaviour from before this sprint, destroys nothing, and is first in T-036. This holds only if the test report carries the AC-4 qualifier of C-003 and the items of C-002, and corrects the counts of C-001. A report that claims AC-4's 100% without the qualifier would warrant a block.

## Responses

No code or test changed after this critique, so the tested head is still `bbffce6`.

- **C-001: tighten-assertion.** `unit-tests.md` now says 25 refusals on both systems covering 23 rows, names both doubled rows, and gives the sum: 23 + 5 + 2 + 1 + 1 = 32. "Four hints changed and one row added" replaces "five hints".
- **C-002: defer-with-rationale.** The three call sites are named in `unit-tests.md` and in the test report beside the one kind made nowhere, and backlog T-036 carries them. They share their hint functions and the one function that prints a refusal with call sites that are run.
- **C-003: defer-with-rationale, with both conditions met.** The test report states INT-0037 AC-4 as shown through the binary for 31 of 32 kinds, and by its hint's form and a reader's reading for the last, and does not claim the 100% threshold as met. The reason now reads: not by file modes, and a Windows long path was not tried.
- **C-004: tighten-assertion.** `test-matrix-log.md` describes the clone as `integration-tests.md` does.
- **The count of test functions.** The record's 472 and 560 come from a script that takes the function on the lines after each `#[test]`; `git grep` finds three more attributes at both commits. `unit-tests.md` gives both figures. The difference, 88, is the same.
