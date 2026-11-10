# Sprint 6 code review

A separate reviewer read `git diff b0675f6..80a3e7e -- crates/` against the build plan, without editing anything and without running the suite. It was asked for correctness bugs, a verdict on each row of the hint table, two reviews of diffs that the test plan names, and tests that pass without checking what they claim. This file gives its findings and what was done about each. The fixes are commit `4152526`, which is the tested head.

**Verdict: no blocker.** Two findings were marked "should fix" and both are fixed. Seven are minor or notes: four are fixed, and three are recorded for the backlog.

The reviewer checked these and found them right: that `replace_working_copy` leaves a command's state as a fresh read would build it; that `open` refuses a damaged op log before it resumes; `needed_for_a_file`; `Store::op_log`; the forwarder macro; the edit distance and the suggestions; that `undo` takes the head before its snapshot; and that `describe` compares the text after its snapshot.

## Findings

| # | Severity | Finding | Outcome |
|---|---|---|---|
| 1 | should fix | `verify DIR` treated every failure to read DIR as "DIR is not a store" and went on to the working copy above. Inside a working copy, a store whose `objects` could not be read was passed over, and `verify` printed `clean` for a store it was not asked about. | **Fixed.** Only absence lets the search go on. Any other failure stops `verify` with that failure and a hint. The spec says so. |
| 2 | should fix | The hint for a corrupt repository, "run `praps verify` to list what is damaged", leads nowhere for the three `Corrupt` errors that are not about store files: a record that does not decode, a record that names a missing op, and an op committed under an unexpected id. `verify` prints `clean` for those. | **Fixed.** The hint now says that `verify` lists what is damaged in the store, and that this build cannot repair what it finds or what the message names outside the store. |
| 3 | minor | `verify` on a working copy whose store lacks `objects` or `ops` printed the hint "put back the files named above", though no file is named, and for an interrupted `init` the remedy is `praps init`. The refusal was missing from the hint table. | **Fixed.** It has a hint of its own (run `praps init` there if its `init` was interrupted; otherwise put the store back from a copy) and a row in the table, now 32 rows. |
| 4 | minor | `a_crash_during_a_removal_still_stops_the_write` cannot fail if the code stops telling a crash from a refusal: after a crash the simulator fails every later syscall. | **Recorded.** The test's comment says what it does not show. The arm stays, for what it says to a reader. |
| 5 | minor | The comment on `every_refusal_prints_one_hint` claimed every refusal a command line can reach. Six kinds are never made through the binary (those that need a directory the user may not read or enter, and a change with several parents), and three run on one system only. | **Fixed** in the comment, which now lists what the corpus does not reach. Making those refusals through the binary is added to backlog T-036. |
| 6 | minor | An argument that begins with `-` is a flag wherever it stands, so `praps describe "-1 for the off-by-one"` is a usage error, and `praps describe "-h"` prints help and describes nothing. | **As planned.** [INT-0037](../../../intents/INT-0037-command-set.md) says the text given to `describe` cannot begin with `-`. The hint does not say why; that is added to backlog T-036. |
| 7 | minor | The note for a directory left said it was empty. An entry that appears between the listing and the removal makes that false. | **Fixed.** The note says the directory could not be removed and was left. |
| 8 | note | A directory given through a symbolic link followed by `..` is checked where the system resolves it, but the directories above it are taken from the path as written. The two can differ. | **Recorded** in backlog T-036. The plan records that links are not resolved, not this mismatch. |
| 9 | note | `parent_dir` of a path that ends in `..` gives a child of it, so `praps init a/..` syncs the wrong directory. The logic is Sprint 2's; T-002 only moved it. | **Recorded** in backlog T-036. |

## The hint table, row by row

"Could a person or an agent act on it?" The reviewer accepted 29 of the 31 rows it read, some with a caveat, and rejected two.

| Row | Verdict | Reason, and what was done |
|---|---|---|
| `Corrupt` | rejected in part | Finding 2. Fixed. |
| A directory above that cannot be searched | rejected | The hint said "make that directory readable", but the message names only where the search started. **Changed:** the hint now says to make the store of each repository above readable. Naming the directory that failed is added to backlog T-036. |
| `Io` | accepted, weak | The message often holds no path. After a clean `verify` the hint leaves little to do. Backlog T-036. |
| `NotInitialised` | accepted, with a caveat | It is also printed when a once-whole repository has lost `ops/`. `praps init` then starts a new first operation and leaves the old objects unused. Nothing is deleted, but "finish making it" does not describe that. Backlog T-036. |
| `Hazards` | accepted, with a caveat | For a name that differs only in case from an existing entry, moving that entry is a remedy the hint does not offer. Backlog T-036. |
| A top that cannot be entered | accepted, small fault | The failure needs search permission, not read. **Changed:** the hint says "one you may enter". |
| `verify` stopped by the filesystem | accepted, small fault | Only a failed directory listing gets there. **Changed:** the hint says "the store's directories". |
| `verify` finds problems | accepted, with a caveat | The remedy is wrong when the problems are permission failures. Backlog T-036. |
| `verify` on a directory that is no store | accepted, with a caveat | Followed on a bare store that has lost `ops/`, `praps init` makes a repository inside the store's directory: litter, nothing lost. Backlog T-036. |
| A current directory that cannot be read | accepted, small fault | Giving the working copy's directory works only with a full path. Backlog T-036. |
| The other 21 rows | accepted | |

No hint sends anyone to destroy data. `praps init`, which three rows name, writes nothing where a valid operation exists and refuses where an op file is damaged.

## The two diff reviews

- **`t003_changes_no_assertion`: yes.** In T-003's commit `0ca3cea`, `store/tests/store.rs` gained 67 lines and `workspace/tests/workspace.rs` 294, each in one hunk at the end. `crash.rs` and `materialise_crash.rs` each changed two lines, the cell count and the digest. Those four are the only lines removed under any `tests/` directory.
- **`store_tests_only_gain`: yes.** `git diff b0675f6..80a3e7e -- crates/store/tests` is one file, 67 insertions and no deletion.

## What the review did not cover

- It ran nothing, so finding 4 and the matrix digests rest on reading the code. The suite ran the matrices on both systems ([integration-tests.md](integration-tests.md)).
- It did not read line by line: `describe_crash.rs`, most of `history.rs`, the first 640 lines of the diff of `cli.rs`, the first-session tests in `e2e.rs`, or the tests of `sim` and `hash`. It searched them for tests that skip a platform or return early.
- It skimmed `crates/xtask` and did not read the specs.
- It did not analyse Windows path forms (`C:foo`, verbatim paths) in the search for the working copy.
- It read `80a3e7e`. The fixes in `4152526` were not reviewed by a second reader; the suite ran on them on both systems.
