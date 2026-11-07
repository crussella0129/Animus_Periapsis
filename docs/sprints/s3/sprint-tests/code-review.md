# Sprint 3 code review

A read-only subagent reviewed the Sprint 3 code (`git diff 1452100..HEAD -- crates/`, T-002 to T-009) against [the working-copy spec](../../../spec/working-copy.md) and the locked [build plan](../sprint-plans/build-plan.md). It verified each finding against the code. It found six, condensed here. All six are fixed in the commit that adds this file, each with a regression test. The spec is amended to match, and every crash matrix and corpus is re-run afterwards.

## Findings and responses

| ID | Severity | Finding | Response |
|---|---|---|---|
| CR-01 | high | Resume removes the temporary name of every path in the diff, but the plan checked only the names of paths to write. An ignored file at the temporary name of a path to remove passed the plan. After any interrupted write (a crash, or a locked file on Windows), resume deleted it, and nothing held a copy. A tracked, unchanged file at such a name went the same way. | **Fixed.** The plan checks the temporary name of every diff path, whether or not the path is written, on disk and against every F and T path. A taken name blocks its path. `remove_temporaries` removes only files, and never an F or T path. The spec now says "the temporary name of any path of the diff". Tests: `removal_temporary_names_are_checked`, `resume_keeps_a_tracked_file_at_a_temporary_name`. |
| CR-02 | medium | Directory listings were keyed by the escaped display form of names, so a name with a control character, which a repository path may hold, was never found on disk. Such a file was silently not removed or not restored. Worse, an ignored file with such a name could be overwritten by `rename`. | **Fixed.** Listings are keyed by the UTF-8 name. A name with no UTF-8 form gets a NUL-prefixed key, which no repository path can equal. Test: `control_characters_in_names_are_seen`, covering a tab-named file removed, and a tab-named ignored file blocking. |
| CR-03 | medium | Each path was classified once, at plan time, then removed or renamed over after several fsyncs. An edit saved in that window, an editor's autosave for example, was lost. | **Fixed, with a stated residual race.** Just before step 1 removes a path and step 4 renames over one, the path is read again. If it changed, it is kept, the temporary file is removed, and a note names it. Notes are printed after every command, not only after resume. The spec states the remaining window: between that read and the next syscall, with the mode not re-read. Test: `edits_during_the_write_are_kept`. A simulator hook saves edits mid-write, to one path to remove and one path to write. |
| CR-04 | medium | During resume, an entry at a temporary name that is not a file stopped the resume: on Linux the write failed after steps 1 to 3, and on Windows the removal itself failed. Every command then failed, against the spec's "does not fail the command". | **Fixed.** The temporary-name check also runs during resume. A name still taken makes its path a noted skip, and the rest is finished. `remove_temporaries` checks the entry's kind first. Test: `resume_skips_a_directory_at_a_temporary_name`. A broader rule, that resume skips any path that hits an I/O error, is not adopted. It would hide real faults, and the specific cases found are covered. |
| CR-05 | low | Paths to remove did not check their ancestors. After a crash, a directory replaced by a symbolic link was followed by `RealFs::scan`. Resume could then delete a file outside the working copy, or fail on the link. | **Fixed.** A path to remove, and an emptied directory, must be reached through real directories only. Test: `real_symlinked_directories_are_not_followed`, a real symlink on ext4. The simulator does not follow links, so only a real filesystem can show this. |
| CR-06 | low (test gap) | The crash matrix never had an ignored file, or a user edit made between the crash and the resuming `open`. That is why CR-01 and CR-04 went unseen. | **Fixed.** The scenario now keeps an ignored file beside the working copy throughout. After every crash inside `undo` or `restore`, it edits a file that the write changes, before the recovering `open`. Both must come through byte for byte. The liveness oracle reads `.gitignore` as Git does, including the prefixes a torn write can leave. Temporary names held by user files are not in the matrix: such a name blocks every write, so the unit tests above cover it. |

## Results after the fixes

- `materialise_matrix_finds_no_violations`: 40,960 cells (20,480 per mode), 0 violations of closure, durability, working copy or liveness. 7,998 cells resumed a record.
- `resume_survives_a_second_crash`: 118,306 second-pass cells, 0 violations.
- `materialise_catches_no_record`: 2,800 working-copy violations in Lenient mode, 2,820 in Strict mode.
- `materialise_matrix_is_reproducible`: the digest over 40,960 cells is `blake3:6274344b358b15cb24d182a681c592ccad66a82564fd40c5cf45e4fcb666a1ac`. The scenario changed (the ignored file, the user edit), so the digest differs from the T-009 one.

The reviewer also checked these and found them sound:
- the order of record, op and apply;
- the three resume branches;
- file↔directory swaps across crashes;
- the executable bit;
- `undo`'s toggle, prefix resolution and `log`'s order;
- the Windows and case rules;
- the policy rules.
