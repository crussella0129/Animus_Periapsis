# Sprint 5 Code Review

A read-only subagent reviewer read the code changes between `b62d587`, the finalized plans, and `3289008`, the build's last commit. That covers:
- `Cargo.toml`;
- `crates/xtask/src/{units,ci,workflow,pins,docs}.rs`;
- `crates/codec/tests/alloc.rs`;
- `.github/workflows/ci.yml`;
- the README and credits.

**What it found clean.** It found no high-severity defect: no deadlock, no panic path, no exit-status bug and no collision of executable names (names carry Cargo's hash). `slots == 1` behaves correctly, and Windows paths parse correctly.

**What it found.** Nine findings, all fixed except the per-unit timeout half of CR-05, which is deferred to the backlog. The fixes are in the commit after `3289008`. Every test named below passed in that commit's `cargo ci` on Windows and in WSL ([unit-tests.md](unit-tests.md)). They therefore came after the first tested head, and the local runs and the three GitHub samples were repeated at the new head.

| Finding | Severity | What was wrong | Response | Test |
|---|---|---|---|---|
| CR-01 | medium | `workflow.rs` never required the lint job to be apart from the `cargo ci` job, which AC-4 now requires: fmt and Clippy moved into the `ci` job passed. | A lint job must run both commands and no `cargo ci` step. The problem text now reads "no lint job of its own". | mutant: the `lint` job deleted and its steps put into `ci` |
| CR-02 | medium | The concurrency group was checked by substring, so a group that put every push to `main` in one group passed. Job-level `concurrency` was never read. | The group must equal `ci-${{ github.ref }}-${{ github.event_name == 'pull_request' \|\| github.sha }}` exactly, and any job-level `concurrency` is a problem. | mutants: `github.event_name == 'push'` in the group; a cancelling `concurrency` block in the `ci` job |
| CR-03 | medium | The `cargo ci` and lint steps were checked only for `if:`. A `shell: cat {0}` step, or job `defaults`, would print the script and pass. | `runs_on_both_platforms` requires plain steps (no `if:`, `working-directory` or `shell`) and no job `defaults`, as the deny job already did. | mutants: `shell: cat {0}` on `cargo ci`; `defaults: run: shell` on the `ci` job; `working-directory` on Clippy |
| CR-04 | medium | The unrun-executable check could pass without checking anything. If the pre-build's `Executable` lines failed to parse, `built` was empty and the check vacuous. Nothing checked that the executables run were ones the build reported. | The step fails when the build reports no executable while there are test units, and when a unit runs an executable the build did not report. It prints `test: <n> test executables built, <m> run`. `run_passes_a_clean_workspace` asserts `test: 2 test executables built, 2 run` for the real probe workspace, so the real parse is exercised. | `a_build_that_reports_nothing_fails_the_step`, `an_executable_the_build_did_not_report_fails_the_step`, `run_passes_a_clean_workspace` |
| CR-05 | medium | Unit output is printed only when a unit finishes, and units have no timeout. A hung unit killed by the job's `timeout-minutes` would leave no trace of what was running. | **Fixed in part.** Each unit's start is printed (`---- unit <name>: started`) from the main thread, so a killed job's log shows which units were in flight. A per-unit timeout below the job's 30 minutes is **deferred** to backlog T-026. | `each_unit_is_announced_when_it_starts` |
| CR-06 | low | stdout was printed whole, then stderr. Cargo's `Running` lines, on stderr, ended up apart from the results they head. | `run_command` gives one `std::io::pipe` to both streams, so the output keeps the order the child wrote it. | `run_command_keeps_the_streams_in_order` (a passing and a failing `cargo` command, and a missing program) |
| CR-07 | low | The checker accepted `checksum: false` on the install step, and any tool name starting `cargo-deny`. | `checksum` must be absent or `true`. The tool must be `cargo-deny` or `cargo-deny@<version>`. | mutants: `checksum: false`; `tool: cargo-deny-unrelated` |
| CR-08 | low | `TERMINAL` did not fix progress bars. A user's `term.progress.when = "always"` could put progress text before a status line and hide it from the parser. | `CARGO_TERM_PROGRESS_WHEN=never` is added to `TERMINAL`. This strengthens the T-003 clause, which named the other three settings. | `the_step_sets_cargo_terminal_settings` (all four) |
| CR-09 | low | `unit_output_is_never_interleaved` cannot fail under the current design: only the main thread writes, and the writer is not `Send`. | The test now says so in its comment. It stays, as a guard against a change that would let workers write. | — |
