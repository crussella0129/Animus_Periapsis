# Test Critique — Sprint 1, round 1

Method: I read the locked plans, every test record, ci-sprint0.md, the INT-0003 to INT-0007 acceptance criteria, and the test code. I also cross-checked the four recorded GitHub runs with `gh run view`. Their conclusions, head SHAs and job ids all match ci-sprint0.md.

I re-ran `store` and `sim` at `133127a` in a scratch clone with its own `target/`. The results matched the record: 9,344 cells per mode with 0 violations, the same negative-control counts, the same digest `blake3:4e4d9e8b…99c59`, 243/243 truncations, 40/40 bit flips, and 16,256 recovered stores with 0 reports. The test count of 236 also adds up from the per-target table.

I then ran 16 mutants. 12 were caught:
- recover keeps all pending changes (M2);
- op data not synced (M3);
- init skips the parent `sync_dir` (M4);
- fsck skips object decoding (M5);
- fsck skips the missing-object check (M6);
- policy drops `cfg_attr` or `expect` detection (M7, M7b);
- xtask drops the exe canonicalisation or the workspace comparison (M8, M8b);
- `commit_op` drops the `objects/` re-sync (M10).

The survivors and the policy bypass are below. The clone was restored to a clean state afterwards.

## Concerns

### C-001: The correct-store crash matrix accepts torn files under final names
- **Where:** `unit-tests.md` T-005 row 1, and test-plan traceability rows for INT-0007 AC-3 and INT-0005 AC-2. `crates/store/tests/common/matrix.rs` `present()`/`check()`.
- **Quote:** INT-0007 AC-3: "in-flight ops … each of which is either wholly present and valid or absent". `present()` silently skips any file whose content does not hash to its name.
- **Failure mode:** weak-assertion
- **Why it matters:** Mutant M11 makes `write_file` write straight to the final name: no temp file and no rename. `crash_matrix_finds_no_violations` still passes with 0 violations. Only `fsck_is_clean_on_matrix_stores` fails, and that test is traced to AC-4, not AC-3. It also covers only the states that open. So the matrix cited for INT-0007 AC-1 to AC-3 does not assert the "wholly present or absent" half of AC-3. For the correct store, the heads check (`store.heads() == heads_of(present ops)`) recomputes from the same files, so in practice it only checks that no unattempted op is present.
- **Suggested response:** tighten-assertion. In `check()`, count any parseable, non-`tmp-` file under `objects/` or `ops/` that does not hash to its name or decode as a heads or atomicity violation. Consider adding "write in place" as a fourth negative control, and cite `fsck_is_clean_on_matrix_stores` in the AC-3 trace.

### C-002: `crash_point_boundary_is_exact` does not prove that syscall *k* has no effect
- **Where:** `unit-tests.md` T-002 row 1; `crates/sim/tests/contract.rs` `crash_point_boundary_is_exact`.
- **Quote:** "Syscall k took no effect: the run without it equals the one with a crash at k" — but `before` is `script(0, Strict, Some(k)).snapshot()`, the same crashed run, so the comparison is a tautology.
- **Failure mode:** weak-assertion
- **Why it matters:** The test checks "no effect" only for k = 1 and k = 4. Two mutants survived the whole `sim` contract, `store` and `crash` suites:
  - M1: at its crash point, `rename` takes effect and returns `Ok`;
  - M1c: the same for `sync_dir` and `create_new`.

  The EARS clause "syscall k SHALL NOT take effect and SHALL return the crash error" is therefore unproved for most syscalls.
- **Suggested response:** tighten-assertion. For every k, compare the crash-at-k state with the state just before syscall k (for example, captured with the `SimFs` hook at boundary k). Also assert that syscall k's own result is the crash error.

### C-003: The fs-boundary exemption for `tests/` can be pulled into `src/` with `#[path]`
- **Where:** `unit-tests.md` T-003 row 2; `crates/xtask/src/policy.rs` `fs_boundary` (the exemption for `name.starts_with(&tests)`).
- **Quote:** "Opt-outs only in `sim/src/real.rs` and `tests/`".
- **Failure mode:** negative-path
- **Why it matters:** I added `crates/store/tests/escape.rs` with `#![allow(clippy::disallowed_methods)]` and `std::fs::read(..)`, plus `#[path = "../tests/escape.rs"] pub mod escape;` in `store/src/lib.rs`. Both `cargo policy` and `cargo clippy --workspace --all-targets -- -D warnings` exited 0. The store library then calls the real filesystem directly, which defeats the INT-0005 Consequence the rule exists for. `include!(..)`, or `#[path]` pointing at `sim/src/real.rs`, opens the same hole. A `[build] rustflags` bypass in `.cargo/config.toml` also passes policy and Clippy, but the `pins` test catches it, so it is not raised.
- **Suggested response:** add-test. The rule should fail on `#[path` and `include!` (or any reference into `tests/` or `real.rs`) in the boundary crates' non-test sources. Add a fixture for each.

### C-004: The matrix log drops lines, including every Lenient negative-control result
- **Where:** `test-matrix-log.md` sections `int_crash_matrix_windows` and `int_crash_matrix_linux`; `integration-tests.md`.
- **Quote:** "The output of the two matrix scripts at the tested head". Under `cargo test -q`, the script's `grep -E '^matrix'` drops every line that begins with a progress dot.
- **Failure mode:** evidence-drift
- **Why it matters:** The log shows 5 of the 6 correct-store lines and none of the three controls' Lenient lines. The hidden counts are:
  - pointer-file: 5,966 heads violations;
  - unsynced-objects: 6,633 closure and 9,402 durability;
  - unsynced-init: 0 violations of any kind.

  INT-0005 AC-3 says "every known-bad storage variant produces at least one violation". The plan scopes that to strict mode, but the record should show the Lenient zero rather than lose it to a filter.
- **Suggested response:** tighten-assertion (of the record). Re-run without `-q`, or strip the dots before grepping. Record both modes per variant, and state that unsynced-init cannot be detected in lenient mode because `create_dir` is durable at once.

### C-005: The two-writer race can become a vacuous pass, and its coverage is overstated
- **Where:** `e2e-tests.md` Not yet possible; `code-review.md` Known limits; `matrix.rs` `Scenario::TwoWriters`.
- **Quote:** "Two writers in one process are covered, interleaved at every syscall." The code says: "A's put of X takes six syscalls … B runs before one of those eight, chosen by the seed."
- **Failure mode:** flake-risk (vacuous pass) and evidence-drift
- **Why it matters:**
  - B runs at one of 8 hard-coded boundaries inside A's put of X, not "at every syscall". The plan said "inside writer A's `commit_op`".
  - B's errors are discarded (`let _ = commit(...)`).
  - Nothing asserts that any cell actually acknowledges B's op in the race window (after A's rename, before A's `sync_dir`). If `put_object`'s syscall count changes, the window could miss the race and the matrix would still report 0 violations. P1 proved the window only at this head.
- **Suggested response:** tighten-assertion. Assert that the matrix has at least one cell where B's commit was acknowledged while X's directory entry was still pending. Correct the wording in e2e-tests.md and code-review.md.

### C-006: The unsynced-init control's `open` still syncs the layout
- **Where:** `crates/store/src/lib.rs` `open_variant` and `open`; `unit-tests.md` T-005 row 3.
- **Quote:** `let mut store = Store::open(fs, root, alg, seed)?; store.variant = variant;`
- **Failure mode:** stub-leak
- **Why it matters:** `open` runs as `Variant::Correct`, so its `if store.syncs_at_init()` guard never applies under variants. In TwoWriters, writer B's `open` syncs the layout, so the unsynced-init variant is not the "`init` performs no `sync_dir`" store it claims to be for that scenario. TwoWriters has 468 semantic cells, against 2,790 for Chain.
- **Suggested response:** tighten-assertion. Set the variant before the layout sync in `open_variant`, or document the deviation.

### C-007: T-001 rejection tests cover half of the clause
- **Where:** `unit-tests.md` T-001 row 3; `crates/codec/tests/properties.rs`.
- **Quote:** EARS: "unsorted or duplicate `parents` or `objects`". The tests are `operation_rejects_unsorted_parents` and `operation_rejects_duplicate_objects` only.
- **Failure mode:** EARS-coverage
- **Why it matters:** Unsorted `objects` and duplicate `parents` are never exercised. The set decoder is shared, so the risk is low, but the record claims the whole clause.
- **Suggested response:** add-test (two hand-built bodies), or reword the row.

### C-008: The `praps fsck` "cannot be read" path is only tested with a nonexistent path
- **Where:** `unit-tests.md` T-006 row 3; `crates/praps/tests/cli.rs` `fsck_on_a_non_store_exits_one` and `fsck_exits_zero_on_a_healthy_store`.
- **Quote:** EARS: "on a directory that is not a store or cannot be read … exit 1". The test also says "Checking changes nothing: a second run says the same", but it compares stdout only.
- **Failure mode:** negative-path
- **Why it matters:** No existing-but-unreadable case is run, such as a path that is a file. `praps fsck` goes through `Store::open`, which syncs the layout and the root's parent, so it writes. The no-change claim is not asserted on the tree. On Windows, a healthy read-only store would be reported as unusable.
- **Suggested response:** add-test (a file given as `DIR`; snapshot the tree before and after). Defer-with-rationale for read-only media.

## Confidence
block

Block is driven by C-001 and C-002. In C-001, the matrix cited for INT-0007 AC-3 accepts a store that writes in place. In C-002, the EARS crash-boundary promise has surviving mutants. C-003 is an unproved negative path for the INT-0005 boundary. All three fixes are small and local.

## Responses

The round blocked. Every concern is addressed in `a62632a` and `499fb8c` (tested head), or in the records rewritten at that head.

| Concern | Response |
|---------|----------|
| C-001 | tighten-assertion in `a62632a`. `check()` adds a `torn` invariant, counted with the semantic violations: any file under an object or op name whose content does not hash to that name or does not decode. A fourth negative control, `WriteInPlace`, writes each file directly under its final name. The critic's mutant M11 is exactly that control, and it is caught with 563 torn files per mode. The correct store has 0 torn files. The INT-0007 AC-3 trace now cites the torn check and `fsck_is_clean_on_matrix_stores`. |
| C-002 | tighten-assertion in `a62632a`. New test `every_syscall_has_no_effect_at_its_crash_point`. It runs a script with every syscall kind, a crash at each one, 20 seeds and both modes. At each crash point *k* it compares the visible state and the recovered state with those the hook captured just before syscall *k*, and requires *k*'s own result to be the crash error. The tautological comment in `crash_point_boundary_is_exact` is corrected. Probe: making `rename`, `sync_dir` and `create_new` take effect at their crash point (the critic's M1 and M1c) fails the new test. |
| C-003 | add-test in `a62632a`. `fs-boundary` fails on a `#[path]` module or an `include!` in a boundary crate's sources outside `tests/` and the real half, whitespace included. `policy_enforces_the_fs_boundary` adds four fixtures: `#[path]` into `tests/`, `#[path]` into `sim/src/real.rs`, `include!`, and spaced `include ! (concat!(…))`. |
| C-004 | tighten-assertion (of the record). The matrix scripts strip the progress dots before filtering, so every line is kept. The re-run log at `499fb8c` shows both modes for every variant. The records state that unsynced-init produces no violation in lenient mode, because lenient mode makes `create_dir` durable at once; INT-0005 AC-3 is judged in strict mode, as the plan says. |
| C-005 | tighten-assertion in `a62632a`. Each cell records whether B's op was acknowledged while X's name, renamed into place by A, was not yet durable. That uses `SimFs::is_durable`, which is not a syscall, so the crash points stay where they were. `crash_matrix_finds_no_violations` fails if no cell in either mode ran the race; 70 cells per mode do. The wording in `e2e-tests.md` and `code-review.md` now says B runs at one of eight boundaries within A's put of X. |
| C-006 | tighten-assertion in `a62632a`. `open_variant` sets the variant before the layout sync, so the unsynced-init control's `open` syncs nothing either. |
| C-007 | add-test in `a62632a`: `operation_rejects_unsorted_objects` (at 35) and `operation_rejects_duplicate_parents` (at 34). |
| C-008 | add-test in `a62632a`. `praps fsck` now uses `Store::inspect`, which opens without the layout syncs: a check makes no writing or flushing syscall, so it also works on a read-only store. `fsck_on_a_non_store_exits_one` adds a file given as `DIR`. `fsck_exits_zero_on_a_healthy_store` compares the whole tree, names and contents, before and after a check. Read-only media are not tested; `inspect` opens nothing for writing. |
