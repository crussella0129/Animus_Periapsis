# Sprint 3 End-to-End Test Results

- **Status:** possible, and run at tested head `70a2a4d` ([test-matrix-log.md](test-matrix-log.md)).

### praps_history_end_to_end
- **Intents:**
  - [INT-0009](../../../intents/INT-0009-reversible-by-default.md) AC-1 to AC-3
  - [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) AC-1
  - [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-1
- **Run:** the built `praps` binary as a subprocess (`crates/praps/tests/e2e.rs`), in a temporary directory on NTFS (Windows) and on ext4 (WSL).
- **Steps:**
  1. `praps init project`. Write `README.md`, `src/main.rs`, a `.gitignore` of `*.log`, and `build.log`. `praps status` lists `A .gitignore`, `A README.md` and `A src/main.rs`.
  2. `praps new`. Edit `README.md`, delete `src/main.rs`, add `docs/guide.md`. `praps status` lists `M README.md`, `A docs/guide.md` and `D src/main.rs`.
  3. `praps undo` prints `undid the last op…`. `README.md` and `src/main.rs` are back as they were before `new`. `docs/guide.md` is gone, and so is the `docs` directory it emptied. The ignored `build.log` is untouched.
  4. `praps log`, then `praps restore` of the snapshot that `undo` took. The edits are back, and `praps status` gives the step 2 answer again.
  5. `praps log` lists one op more than before, and every earlier line is still there.
  6. `praps fsck` is clean.
- **Result:** passed on Windows and on Linux (sections `e2e praps`).

### real_hazards_are_refused_on_ntfs
- **Intents:**
  - [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-3, AC-4
  - [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) AC-1
- **Run:** a real `Workspace` on NTFS restores a view holding `README` (next to an existing `readme`), `CON` and `dot.`.
- **Result:**
  - The restore fails with `Error::Hazards`, which names all three.
  - The directory still holds only `readme`, with its content, and no working-copy record was left.
  - Passed on Windows (section `int_reversibility_and_hazards_windows`).

### real_hazard_trees_materialise_on_ext4
- **Intents:** [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-3, AC-4
- **Run:** the same names on ext4, plus `readme`, `a:b`, and a file `a` beside a directory `A/`. Linux allows all of these.
- **Result:** every file is written with its exact content. Passed on Linux (section `int_reversibility_and_hazards_linux`).

### Earlier end-to-end tests
`praps_end_to_end`, `praps_status_reports_a_non_utf8_name` and `praps_status_reports_non_utf8_names_and_directories` passed again on both systems. The `praps` fsck tests in `cli.rs` passed too (section `e2e praps_fsck_end_to_end`).

### e2e fresh_clone_ci
- **Intents:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-1, AC-8, and [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-1
- **Run:** a fresh clone of `70a2a4d` into the WSL ext4 home directory, then a cold `cargo ci`.
- **Result:** every step PASS, exit 0 in 993 s, 404 tests, 0 tracked files changed.

## Not possible
- Power loss on real hardware. The simulator's crash model stands in for it, as in Sprints 1 and 2.
- Nighthawk, the Debian machine. It still needs Tailscale approval.
- Several processes or workspaces writing at once. That is backlog T-011.
