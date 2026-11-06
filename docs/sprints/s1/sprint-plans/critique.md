# Plan Critique — Sprint 1

Round 2. Round 1 blocked with 15 concerns ([critique-round1.md](critique-round1.md)). This round checked that each response landed in the plans and intents, re-screened the revised plans, and tried the feasibility questions in a scratch workspace with its own target directory.
- **Self dev-dependency:** `store` with `features = ["variants"]` builds and runs `tests/crash.rs` under plain `cargo test --workspace`, and passes `deps`.
- **Crate-level config:** Clippy reads a crate-level `clippy.toml` in place of the root one.
- **Banned paths:** `std::fs::read`, `std::path::Path::exists` (also through a `PathBuf`), `std::fs::File` and a re-listed `HashMap` are all reported.
- **Module allow:** an allow at the top of `real.rs` exempts that module.

All 15 round-1 responses landed. C-002, C-010 and C-011 had gaps, which are C-001, C-004 and C-003 below.

## Concerns

### C-001: Negative controls 1 and 2 depend on simulator behaviour that T-002 neither specifies nor tests
- **Where:** `build-plan.md` T-005 variants and T-002 EARS and Notes; `test-plan.md` T-002 unit tests
- **Quote:** T-005: "keeps a `HEAD` file holding the head set, rewritten in place (truncate and write, no temporary file, rename or sync)" / "skips `sync_file` on object files but still renames them and syncs `objects/`". T-002 Notes: "`create_new` refuses an existing name; `rename` replaces the target"
- **Failure mode:** hidden-dep
- **Why it matters:**
  - **Pointer file:** T-002's syscall set has no way to open an existing file for writing or to truncate one, so the pointer-file variant cannot be written against it.
  - **Overwrite crash semantics:** No clause says what a crash leaves after an unsynced truncate and rewrite.
  - **Unsynced data after a rename:** Control 2 fails only if unsynced data stays at risk after `rename` and `sync_dir`. That means the pending data must follow the file across the rename. A `SimFs` whose `rename` or `sync_dir` settles pending data passes every T-002 test, and then `unsynced_objects_variant_is_caught` comes out by accident.
- **Suggested response:** fix-in-plan. Add an overwrite/truncate syscall with a crash clause. Add a clause and test that unsynced data stays at risk across a rename and directory sync. Name the test the pointer-file control relies on.

### C-002: Control 3 can only show up as a layout violation unless invariants are checked on the raw recovered state
- **Where:** `build-plan.md` T-005 invariants and the "Unsynced init" variant
- **Quote:** "**Layout:** if `init` was acknowledged, `Store::open` succeeds; otherwise either `open` fails as not-a-store …" / "each **SHALL** produce at least one violation of the closure, durability or heads invariant, counted separately from any layout or stray-file check"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** If closure, durability and heads are checked only through a store that opened, unsynced init loses `ops/` or `objects/` and makes `open` fail. That produces only layout violations, which are excluded from the count. So `unsynced_init_variant_is_caught` fails, or the counting quietly gets bent.
- **Suggested response:** fix-in-plan. Check closure, durability, heads and atomicity on the recovered `SimFs` state directly, with a missing directory meaning an empty set. Count a missing or torn `HEAD` in the pointer variant as a heads violation.

### C-003: The `fs-boundary` allow check can be bypassed with a group-level allow
- **Where:** `build-plan.md` T-003 EARS and the `fs-boundary` rule
- **Quote:** "fail if … such an allow appears anywhere else in those crates"
- **Failure mode:** missing-risk
- **Why it matters:** In scratch, `#![allow(clippy::style)]` in `store/src` silenced `Path::exists`. `clippy::all`, `#[expect(...)]`, `cfg_attr` and the old lint names slip past a text check for two lint names.
- **Suggested response:** fix-in-plan. Reject any `allow`, `expect` or `cfg_attr` naming the two lints, their old names, `style` or `all` outside `real.rs` and `tests/`, with fixtures; or add a Clippy probe.

### C-004: T-008's path comparisons fail on Windows unless both sides are normalised
- **Where:** `build-plan.md` T-008 EARS 1 and 2; `crates/xtask/src/main.rs` `workspace_root()`
- **Quote:** "**WHEN** the running xtask binary (`std::env::current_exe()`, canonicalized) is not inside `<workspace root>/target`, where the workspace root comes from the build-time `CARGO_MANIFEST_DIR`"
- **Failure mode:** EARS-vague
- **Why it matters:**
  - **Verbatim prefix:** on Windows, `canonicalize` yields `\\?\C:\…`, while `CARGO_MANIFEST_DIR` is `C:\…`. `starts_with` would then refuse every run.
  - **Manifest path:** `locate-project` returns a manifest path, not a directory.
  - **Placement:** the plan doesn't say where the guard sits. A guard inside `ci::run` would break `hostile_variables_never_reach_a_step`, and its order against usage errors is unstated.
- **Suggested response:** fix-in-plan. Canonicalize both sides, take the parent of `root`, and put the guard in `main` after argument validation. Add a Windows test with a real `current_exe()`.

### C-005: T-005 adds a policy check that has no EARS clause and isn't in Touches; other Touches gaps
- **Where:** `build-plan.md` T-005 Notes and Touches; T-006; T-008 and T-009
- **Quote:** "`cargo policy` fails if any non-dev dependency on `store` enables `variants`" (Notes only)
- **Failure mode:** hidden-dep
- **Why it matters:**
  - **Policy check:** it has no clause, and `policy.rs` isn't touched.
  - **Workspace entry:** the root `store` workspace entry has no owner task.
  - **Shared matrix:** `fsck_is_clean_on_matrix_stores` needs the matrix from another integration-test crate.
  - **Backlog:** `docs/work/tasks.md` is untouched by the tasks that remove backlog entries.
- **Suggested response:** fix-in-plan. Add the clause and Touches, assign the workspace entry, add a shared matrix module, and touch `tasks.md`.

### C-006: The two-writer scenario can't exercise the race its traceability row claims
- **Where:** `build-plan.md` T-005 scenarios; `test-plan.md` traceability row for the INT-0007 Consequences
- **Quote:** "two interleaved writers (two `Store` values) in one process"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** With synchronous calls alternating, writer A's rename is always followed by its `sync_dir` before writer B runs. The race behind the re-sync Consequence can then never occur in the matrix.
- **Suggested response:** fix-in-plan. Define syscall-level interleaving (a seeded scheduler or `SimFs` yield points), or narrow the row.

### C-007: Clock has no clause or test; `praps fsck`'s exit codes and its no-argument case go beyond INT-0003
- **Where:** `build-plan.md` T-002 (`clock.rs`); T-006 EARS and Notes; INT-0003 AC-9
- **Quote:** T-006 "a directory that is not a store, **THEN** it **SHALL** exit 2" / "`fsck` with no directory checks the current directory"
- **Failure mode:** plan-test-mismatch
- **Why it matters:**
  - **Clock:** it is built untested.
  - **No-argument case:** untested.
  - **Non-store exit 2:** contradicts T-008's own reason for exit 1.
  - **AC-9:** it names only the three tooling commands.
- **Suggested response:** fix-in-plan. Test or drop the clock, test the no-argument case, use exit 1 for a non-store, and stop citing AC-9 for `praps`.

### C-008: A golden-corpus test that round 1 named is still missing from the test plan
- **Where:** `test-plan.md` T-001; `crates/codec/tests/golden.rs` `golden_corpus_has_the_spec_types`
- **Quote:** "Extended in `spec.rs` and `golden.rs`: `spec_registries_match_code` …, `spec_examples_match_golden_files`, `golden_v1_corpus_matches` …"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** The test asserts the exact corpus list and fails once operations are added. The plan also doesn't say whether the `.bin` files hold envelopes or bare bodies.
- **Suggested response:** fix-in-plan. List the test as updated, and state that the files hold full envelopes.

## Confidence
proceed-with-caveats

## Responses

Every concern is fixed in the plans before finalizing.

| Concern | Response |
|---------|----------|
| C-001 | fix-in-plan. T-002 lists its syscalls, adding `overwrite` (truncate and write, used only by the pointer-file control). It adds clauses and tests `unsynced_data_stays_at_risk_across_rename` (pending data follows a file across a rename; neither `rename` nor `sync_dir` makes data durable) and `unsynced_overwrite_recovers_old_empty_or_torn`. The pointer-file variant now writes `HEAD` with `overwrite`. |
| C-002 | fix-in-plan. T-005 checks every invariant on the recovered `SimFs` state directly, whether or not `open` succeeds, with a missing directory as an empty set. A missing, empty, torn or undecodable `HEAD` in the pointer variant counts as a heads violation. |
| C-003 | fix-in-plan. An opt-out is any `allow`, `expect` or `cfg_attr` naming `disallowed_methods`/`disallowed_types`, their old names, `clippy::style` or `clippy::all`. `policy_enforces_the_fs_boundary` has a fixture for each form. |
| C-004 | fix-in-plan. Both sides are canonicalized (exe and `<root>/target`; the located and build-time roots). The located root is the parent of `locate-project`'s `root`. The checks run in `main` only, after argument validation, so `ci::run` and its tests are unaffected. New tests: `usage_errors_come_before_the_checkout_checks`, and `xtask_in_its_own_target_dir_runs` with the real canonicalized paths on both systems. |
| C-005 | fix-in-plan. The variants guard is an EARS clause extending `deps`, so the rule count and `PASS` output are unchanged, and `policy.rs` is in T-005's Touches. T-004 adds the workspace `store` entry. `tests/common/matrix.rs` is shared by `crash.rs` and `fsck.rs`. T-008 and T-009 touch `docs/work/tasks.md`. |
| C-006 | fix-in-plan. The two-writer scenario interleaves at the syscall level. A seeded `SimFs` hook runs writer B's whole put and commit at a seed-chosen syscall inside writer A's `commit_op`, and B's op names A's object, so B can find it renamed but unsynced. `crates/sim/src/simfs.rs` joins T-005's Touches. |
| C-007 | fix-in-plan. The clock is deferred to the first timestamped record. `clock.rs` is dropped, and T-002 is retitled "Filesystem and randomness traits", with Work evidence links updated. `praps fsck` exits 1 for a non-store or unreadable directory, and 2 for usage only. The no-argument case has a clause and `fsck_without_dir_checks_the_current_directory`. The exit codes are stated as a design choice, and the traceability row now cites only AC-8. |
| C-008 | fix-in-plan. `golden_corpus_has_the_spec_types` is listed as updated, and `operation-*.bin` hold full envelopes, like `blob-envelope.bin`, matching the spec's worked examples. |
