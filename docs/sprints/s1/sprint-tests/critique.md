# Test Critique — Sprint 1

Round 2. Round 1 blocked with 8 concerns ([critique-round1.md](critique-round1.md)).

Method: I checked each round-1 response against the code and the records at `499fb8c`, and found that each one landed:
- **C-001:** the `torn` count in `check()`, and the `WriteInPlace` variant with its test.
- **C-002:** the new test `every_syscall_has_no_effect_at_its_crash_point`, which covers all 10 `Fs` methods.
- **C-003:** the `#[path` / `include!` text rule, with four fixtures.
- **C-005:** the `raced` flag, and the race assertion in `crash_matrix_finds_no_violations`.
- **C-006:** `open_variant` now goes through `finish_open`.
- **C-007:** the two new codec tests.
- **C-008:** `Store::inspect` and the two strengthened CLI tests.

`000b70e` changes docs only.

I re-ran the suite in a scratch clone at `499fb8c`, with its own `target/`:
- 240 tests passed. The per-target counts match the table in `unit-tests.md`.
- Every matrix line matches `test-matrix-log.md`: 9,344 cells per mode with 70 races; the negative-control counts, including torn 563, 9,527 and 9,535; digest `blake3:4e4d9e8b…99c59`.
- 243/243 truncations, 40/40 bit flips, and 16,256 recovered stores with 0 reports.

I re-ran the round-1 survivors. All are now caught:
- **M1** (`rename` takes effect at its crash point) fails `every_syscall_has_no_effect_at_its_crash_point`.
- **M1c**, split into `sync_dir` alone and `create_new` alone, fails the same test. So does each of `write`, `sync_file`, `overwrite`, `remove_file` and `create_dir`.
- **M11** (write in place in the correct store) fails `crash_matrix_finds_no_violations`.
- **M10/P1** (`commit_op`'s `objects/` re-sync removed) still fails it, with closure violations in TwoWriters.
- The **literal `#[path]` escape** is caught by the new fixtures.

Three fresh findings follow.

## Concerns

### C-001: The fs-boundary escape rule can still be bypassed with `cfg_attr(…, path = …)` or a renamed `include!`
- **Where:** `unit-tests.md` T-003 row 2; `crates/xtask/src/policy.rs` `fs_boundary`, the compact-text check for `"#[path"` and `"include!"`. Round-1 Responses, C-003.
- **Quote:** the record says "four `#[path]`/`include!` escapes (test critique C-003)". The rule is `for (needle, what) in [("#[path", "a #[path] module"), ("include!", "include!")]`.
- **Failure mode:** negative-path
- **Why it matters:** I created `crates/store/tests/escape.rs` and `escape2.rs`, both calling `std::fs::read` with a `disallowed_methods` allow. I pulled them into `store/src/lib.rs` in two ways:
  - `#[cfg_attr(all(), path = "../tests/escape.rs")] pub mod escape;`
  - `use std::include as inc; inc!("../tests/escape2.rs");`

  The store compiled. `cargo policy` passed every rule, including `PASS fs-boundary`. `cargo clippy --workspace --all-targets -- -D warnings` also exited 0. The store library again reaches the real filesystem, which is the INT-0005 Consequence that T-003 protects. The record presents the C-003 hole as closed when only its literal spelling is.
- **Suggested response:** add-test. Widen the rule to reject any `path =` key inside any attribute, `cfg_attr` included. Also reject the identifier `include` used as anything other than `include_str` or `include_bytes`, so `use … include as …` and `include!` both fail. Add both fixtures above.

### C-002: The C-008 fix's "makes no writing or flushing syscall" claim has no test, and reverting it passes
- **Where:** `unit-tests.md` T-006 row "`praps fsck` reads only"; `crates/praps/tests/cli.rs` `fsck_exits_zero_on_a_healthy_store`; `Store::inspect` in `crates/store/src/lib.rs`.
- **Quote:** "`fsck_exits_zero_on_a_healthy_store` compares the whole tree before and after; `praps` uses `Store::inspect`, which makes no writing or flushing syscall … | pass".
- **Failure mode:** weak-assertion
- **Why it matters:** Neither of these mutants fails any `store`, `praps` or `sim` test:
  - F2: `praps fsck` changed back to `Store::open(RealFs, dir, HashAlg::DEFAULT, 0)`, the code round 1 flagged;
  - F3: `Store::inspect` changed to run `finish_open`, which makes the layout `sync_dir`s.

  The tree comparison cannot see a directory sync, because it changes no name or content. It would have passed before the fix too. The row's "pass" rests on reading the code, not on a test.
- **Suggested response:** add-test. On `SimFs`, run `Store::inspect(...)` and then `fsck()`, and assert that the trace has only `read` and `list` entries. Or reword the row to say the property is reviewed, not tested.

### C-003: Replacing a damaged copy under a final name is specified but never tested
- **Where:** `docs/spec/storage-durability.md` "The write sequence"; the `write_file` doc comment in `crates/store/src/lib.rs`; the T-004 rows of `unit-tests.md`.
- **Quote:** "If it holds other bytes (a copy a crash damaged), it is replaced through steps 1 to 5."
- **Failure mode:** negative-path
- **Why it matters:** Mutant F1 changes `write_file`'s `Ok(_) => {}` to `Ok(_) => return self.fs.sync_dir(dir)`. `put_object` then acknowledges an object whose file is damaged and leaves it damaged. F1 passes every `store`, `praps` and `sim` test, including the matrix:
  - the correct store never leaves a damaged file under a final name, so the matrix never reaches this branch;
  - no unit test plants one.

  The Sprint 1 spec states this behaviour as part of the durability argument, but nothing proves it. No EARS clause names it, so it does not block.
- **Suggested response:** add-test. On `SimFs`, plant a truncated file under an object's final name, then call `put_object`. Assert that the file then holds the full envelope, and that the trace goes through `create_new` of a `tmp-` name, `sync_file`, `rename` and `sync_dir`.

## Confidence
proceed-with-caveats

## Responses

Every concern is addressed in `9f138f4`, the final tested head.

| Concern | Response |
|---------|----------|
| C-001 | add-test in `9f138f4`. `fs-boundary` now rejects any attribute with a `path =` key, `cfg_attr` included. It also rejects the identifier `include` as a whole word, called or imported under any name; `include_str` and `include_bytes`, and line comments, are allowed. `policy_enforces_the_fs_boundary` adds both of the critic's escapes (`cfg_attr(all(), path = …)` and `use std::include as inc; inc!(…)`) and a brace-import form, and checks that data includes, comments and a `#[doc]` attribute still pass. |
| C-002 | add-test in `9f138f4`. `inspect_and_fsck_only_read` runs `Store::inspect` and `fsck` on `SimFs` and requires every syscall in the trace to be a `read` or `list`. Mutant F3 (`inspect` through `finish_open`) fails it. F2 (`praps` calling `open`) is guarded by review: `RealFs` keeps no trace, so the CLI test cannot see a sync. The record says so. |
| C-003 | add-test in `9f138f4`. `a_damaged_copy_is_replaced` plants a truncated file under an object's final name and calls `put_object`. The file must then hold the full envelope, and the trace must be exactly `read`, `create_new tmp-…`, `write`, `sync_file`, `rename tmp-…`, `sync_dir`. Mutant F1 fails it. |
