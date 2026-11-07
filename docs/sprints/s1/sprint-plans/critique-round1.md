# Plan Critique — Sprint 1, round 1

## Concerns

### C-001: Negative control 3 (no directory sync at init) cannot fail unless the simulator models durability of directory entries
- **Where:** `build-plan.md` T-002 EARS and T-004 EARS 3; `test-plan.md` T-002 unit tests
- **Quote:** T-002: "WHEN in strict mode a file is created or renamed and synced but its directory is not, THEN across seeds the entry SHALL sometimes vanish". T-004: "the variant that does not sync directories at init … SHALL produce at least one violation"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** The init bug only shows if the `objects/` or `ops/` entry made by `create_dir` can vanish when the root is not synced, and everything below it vanishes too. T-002 specifies and tests vanishing only for file entries. A `SimFs` that treats `create_dir` as durable at once meets every T-002 clause, and then `unsynced_init_variant_is_caught` fails or passes by accident. The check list also has no "acknowledged init leaves an openable layout" invariant, and it does not say what recovery does when `ops/` is missing (error, empty heads, or violation). An init-only scenario with no ops can therefore never show this control.
- **Suggested response:** fix-in-plan. Add a T-002 clause and test `strict_mode_loses_unsynced_directories`: a directory created without a parent sync may vanish with its contents. Add a T-004 invariant that after an acknowledged `init` the layout exists and `Store::open` succeeds. Define what an interrupted, unacknowledged init recovers to, and whether `init` on that leftover is refused for good, which would leave the repository unusable.

### C-002: Negative control 1 (pointer file) is caught by a structural check, not by the crash model
- **Where:** `build-plan.md` T-004 EARS (check "absence of any pointer file") and the pointer-file variant
- **Quote:** "absence of any pointer file" … "the variant that writes a pointer file in place … SHALL produce at least one violation"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** A variant that writes `HEAD` breaks "no pointer file" on every run, crash or not. `pointer_file_variant_is_caught` would pass against a simulator that never crashes, which is the "harness that cannot fail" INT-0005 AC-3 warns about. In R2-E23 the 84 violations came from a torn or stale pointer giving wrong heads after recovery. The plan never says that the variant's `heads()` reads the pointer.
- **Suggested response:** fix-in-plan. Make the variant read heads from the pointer file, rewritten in place without a temporary file and rename. Require that it violates the heads or durability invariant, counted apart from the pointer-file-presence check, in strict mode at least.

### C-003: Negative control 2 (objects not synced) may pass a check that only looks at object names
- **Where:** `build-plan.md` T-004 EARS ("ancestry closure and object presence"); T-002 "kept, dropped and torn-prefix"
- **Quote:** "ancestry closure and object presence of the recovered ops"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** Suppose the variant skips only `sync_file` for objects but still renames and syncs `objects/`. In the model the name is then durable while the content is dropped or torn, so an existence check finds the object present and the control yields 0 violations. The plan does not define the variant precisely (skip `sync_file`, `sync_dir`, or both). It also does not say whether "present" means present with content that hashes to its name.
- **Suggested response:** fix-in-plan. Define "present" as a file whose content hashes to its name and decodes, for example by running `fsck` as part of the matrix check. Define exactly which syncs the variant drops.

### C-004: The correct store as designed can produce real violations: deduplication without a sync, and temporary-name collisions
- **Where:** `build-plan.md` T-003 Notes
- **Quote:** "A file that already exists with the right content is not rewritten." / "Write sequence for both kinds: `create_new(tmp-<id>)` …" / "Temporary names are unique per call (a store-local counter)."
- **Failure mode:** hidden-dep
- **Why it matters:**
  - **Deduplication:** In the interleaved two-writer scenario, writer A renames object X but has not yet run `sync_dir`. Writer B puts X, sees the file, skips the write and never syncs `objects/`. B's op is then acknowledged, and a crash before A's `sync_dir` loses X in strict mode. That is a real AC-1 violation by the correct store, or a matrix that has to be weakened to pass.
  - **Temporary names:** The plan names temporary files both `tmp-<id>` and with a counter, and the two conflict. A counter local to each store collides between two writers (two `Store` values) and after a restart, when a leftover `tmp-0` exists. `create_new` refuses an existing name, so writes can fail permanently after a crash, and with `tmp-<id>` the same object can never be re-put.
- **Suggested response:** fix-in-plan. When a file already exists, still `sync_file` it and `sync_dir` its directory before acknowledging, or only treat as present files known to be durable. Name temporary files with a per-writer nonce (from `Rng`), retry on `AlreadyExists`, and add a test that puts the same object again over a leftover temporary file.

### C-005: The plan's heads invariant differs from INT-0007 AC-3, and its own wording can flag the correct store
- **Where:** `build-plan.md` T-004 EARS; `INT-0007` AC-3
- **Quote:** Plan: "recomputed heads equal to the last acknowledged heads, with every pending op either a head or absent". AC-3: "op heads recomputed from the op files equal the heads of the last acknowledged state"
- **Failure mode:** intent-drift
- **Why it matters:**
  - **Wording:** An op caught in the window between its rename and its acknowledgement can survive, so AC-3 as written cannot hold. The plan relaxes the criterion in sprint prose only, which the critic screen forbids.
  - **False violations:** With two writers, B can take A's renamed but unacknowledged op as a parent. That op then survives as a non-head, which "either a head or absent" counts as a violation of the correct store.
  - **Atomicity:** INT-0005 AC-2's atomicity clause ("the recovered op chain is a prefix of the intended one") has no named check.
- **Suggested response:** fix-in-plan. Revise INT-0007 AC-3 (and its Transition history) to say that recovered heads equal the last acknowledged heads extended only by in-flight ops and their descendants. Add an explicit prefix or atomicity check traced to INT-0005 AC-2.

### C-006: The negative controls depend on a `variants` feature that `cargo test --workspace` does not enable
- **Where:** `build-plan.md` T-004 Notes; `crates/xtask/src/ci.rs` STEPS (`["test", "--workspace", "--locked"]`)
- **Quote:** "compiled only under `cfg(any(test, feature = \"variants\"))`, with a non-default `variants` feature used by the integration test"
- **Failure mode:** hidden-dep
- **Why it matters:**
  - **`cfg(test)` does not reach `tests/crash.rs`:** The library is compiled without `cfg(test)` when an integration test links it, so `crash.rs` sees `Variant` only through the feature.
  - **`required-features` skips silently:** `cargo ci` passes no `--features`. With `required-features = ["variants"]`, the matrix and all three controls would be skipped in CI with no failure.
  - **A self dev-dependency spreads:** The working alternative is `store = { workspace = true, features = ["variants"] }` under `[dev-dependencies]`, and it passes the `deps` rule. Under `cargo test --workspace` that feature is unified into every build of `store`, including the one linked into `praps`'s tests, so "production builds cannot select a broken variant" holds only for non-test builds.
- **Suggested response:** fix-in-plan. Choose the self dev-dependency (or a separate `#[cfg(feature)]` test crate) and state it. Add a guard that the matrix actually ran: the test evidence quotes nonzero scenario counts per variant from `cargo ci` output.

### C-007: Object and op file names: the `HashRef` text form contains `:`, which on NTFS names an alternate data stream
- **Where:** `build-plan.md` T-003 EARS 2; `research-report.md` §2 (`crates/hash/src/lib.rs`: "its text form, usable for file names")
- **Quote:** "under the file name that is its object id's hex text"
- **Failure mode:** hidden-dep
- **Why it matters:** `HashRef`'s `Display` is `<name>:<hex>` (`crates/hash/src/lib.rs` line 204ff). On NTFS, `blake3:af13…` silently writes the alternate data stream `af13…` of a file named `blake3`, so every object would collide in one file. That is exactly the hazard INT-0004 forbids. The bare digest hex avoids the colon, but then equal digests under `blake3` and `sha256` share one name, which INT-0006 says is a migration case. `fsck` "hashes to its name" also needs the algorithm encoded in the name.
- **Suggested response:** fix-in-plan. Specify a name form without a colon, for example `<alg-id-hex>-<digest-hex>` or one directory per algorithm. Record it in INT-0007 or the durability note (on-disk layout belongs to INT-0007). Add a test that the name round-trips and is legal on Windows.

### C-008: T-001 misses existing tests and checks that treat `0x03` as reserved or require "defined" status
- **Where:** `build-plan.md` T-001 Touches; `crates/codec/tests/properties.rs` (`expected_header` `0x02..=0x05`, the class count `("reserved", 4)`, the message test "object kind 0x03 is reserved", `kind_registry_is_complete`); `crates/codec/tests/spec.rs` `spec_registries_match_code`; `crates/codec/tests/golden.rs` `golden_corpus_has_the_spec_types`
- **Quote:** Touches lists `golden.rs` and `spec.rs` but not `properties.rs`; EARS: "list `0x03 operation` as specified … mark it provisional"
- **Failure mode:** hidden-dep
- **Why it matters:**
  - **`properties.rs`:** Four assertions there break when `0x03` becomes defined, and the file is not in Touches.
  - **Status cell:** `spec_registries_match_code` requires the status cell of every code-defined kind to be exactly `defined`. A "specified" or "provisional" status cell fails it, so the provisional marker has to live elsewhere.
  - **Test names:** The test plan names `golden_corpus_matches`, but the existing test is `golden_v1_corpus_matches`. The new `operation_examples_match_the_spec` duplicates the existing `spec_examples_match_golden_files`.
  - **No-op Touches entry:** The Touches entry "crates/xtask/src/policy.rs (spec table row counts)" changes nothing: `SPEC_TABLES` only requires at least one row.
  - **Changeset kind:** The EARS checks that `0x04` and `0x05` stay reserved but leaves out `0x02 changeset`.
- **Suggested response:** fix-in-plan. Add `properties.rs` to Touches, use status `defined` with a separate provisional note, include `0x02` in `view_and_event_kinds_stay_reserved`, and drop or justify the `policy.rs` touch.

### C-009: The operation body departs from INT-0006's defined operation, and sign-off conflicts with "provisional"
- **Where:** `build-plan.md` T-001 and the Intents line for INT-0006 ("the owner's sign-off is requested at the checkpoint"); INT-0006 Intent; `format-v1.md` Change control
- **Quote:** INT-0006: "`Operation` (parent ops, view hash, actor, advisory wall clock, signature; …)". Plan: "`parents` … ‖ `objects` …, mark it provisional until the Phase 1 exit gate". Spec: "Specifying a reserved kind's body … never changes an existing encoding."
- **Failure mode:** intent-drift
- **Why it matters:**
  - **Undefined state:** INT-0006 says an encoding freezes at the owner's AC-6 sign-off. Asking for that sign-off on a body the plan calls provisional is self-contradictory. Neither INT-0006 nor Change control defines a "provisional" state, so a later field addition (view hash, actor, signature) either breaks the "never changes an existing encoding" rule or quietly redefines it.
  - **Field outside the object set:** The `objects` field is not in INT-0006's object set at all. Its rationale (no views yet) exists only in sprint prose.
- **Suggested response:** fix-in-plan. Revise INT-0006 Intent and Consequences, and add a Change control bullet, to define a provisional kind that is excluded from AC-6 sign-off until Phase 1 exit. Record why `objects` stands in for the view hash. Ask the owner to sign off only the frozen sections.

### C-010: T-007's two clauses are underspecified against how `cargo ci` already scrubs the environment
- **Where:** `build-plan.md` T-007 EARS; `test-plan.md` Integration probe; `crates/xtask/src/ci.rs` `SCRUBBED_PREFIXES`
- **Quote:** "`cargo metadata` reports a target directory that is not inside the workspace root" / "for a workspace root other than the one it was built for (`CARGO_MANIFEST_DIR` at build time) … exit 2" / probe: "`CARGO_TARGET_DIR` set outside the checkout → `cargo ci` exits 1"
- **Failure mode:** EARS-vague
- **Why it matters:**
  - **Probe may never fire:** The `CARGO_TARGET_` and `CARGO_BUILD_` prefixes are already scrubbed from child steps, so `cargo metadata` run the way `cargo_command` runs steps reports the in-tree `target/`. The probe expects exit 1 and would then see 0. The real exposure is the `xtask` binary that the `cargo ci` alias builds with the caller's environment, and the EARS does not say which environment `cargo metadata` sees.
  - **No comparison basis:** `workspace_root()` is itself derived from the build-time `CARGO_MANIFEST_DIR`, so "another workspace root" has nothing to compare against (current directory? runtime `CARGO_MANIFEST_DIR`? `current_exe()`?).
  - **Exit code:** Exit 2 is the usage-error code under INT-0003 AC-9, while this is an environment failure.
- **Suggested response:** fix-in-plan. State that `cargo metadata` runs with the inherited, unscrubbed environment, or check `current_exe()` lies under `<root>/target`. Name the runtime reference for the checkout check, and justify exit 1 or exit 2.

### C-011: The `fs-boundary` rule matches text and is easy to evade
- **Where:** `build-plan.md` T-003 EARS (last clause); `test-plan.md` `policy_enforces_the_fs_boundary`
- **Quote:** "fail if any `crates/store/src` file, or any `crates/sim/src` file other than `real.rs`, names `std::fs`"
- **Failure mode:** missing-risk
- **Why it matters:**
  - **Missed I/O:** `use std::{fs, io};`, `::std::fs` and `Path::exists`/`metadata`/`read_dir` (which bypass `std::fs` by name) all reach the real filesystem undetected.
  - **False positive:** A doc comment naming `std::fs` would be flagged.
  - **Uncovered crate:** `praps` is not covered, although INT-0005 says all filesystem I/O goes through the traits.

  This rule is the evidence for "I/O only through the traits", and real I/O in `store` would make the simulator results meaningless.
- **Suggested response:** fix-in-plan. Parse `use` trees (or add Clippy `disallowed-methods`/`disallowed-types` for `std::fs::*` and the `Path` I/O methods in `store`/`sim`). Add fixtures for the brace-import and `Path::exists` cases, and state whether `praps` is in scope.

### C-012: The build plan and test plan disagree on the determinism check
- **Where:** `build-plan.md` T-004 EARS 2 vs `test-plan.md` T-004 `matrix_runs_are_reproducible`; INT-0005 AC-1 proxy
- **Quote:** Build: "WHEN each scenario is run twice with the same seed and crash point". Test: "a sample of (scenario, crash point, seed, mode) cells run twice". AC-1 proxy: "CI runs every scenario twice and compares content hashes"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** The test plan's sampling is weaker than both the EARS and the intent's proxy, and the sample is unspecified (size, how it is chosen).
- **Suggested response:** fix-in-plan. Run every scenario twice for at least one seed and crash point per mode, or state the sampling rule in the EARS and reconcile it with AC-1.

### C-013: T-002's crash-boundary semantics are ambiguous
- **Where:** `build-plan.md` T-002 EARS 2 and 5
- **Quote:** "WHEN the crash point is syscall *k*, THEN every syscall after the *k*-th SHALL fail" / "after a file's `sync_file` and its directory's `sync_dir` returned … survive unchanged"
- **Failure mode:** EARS-vague
- **Why it matters:** The plan does not say whether syscall *k* takes effect before the crash. That decides whether a crash "at" the final `sync_dir` can leave the acknowledgement unreturned while the op is durable, the case behind C-005. "Survive unchanged" also needs "and not written or renamed over afterwards".
- **Suggested response:** fix-in-plan. Define the boundary: syscall *k* either completes then crashes, or crashes before taking effect. Qualify the survival clause.

### C-014: T-003 mixes the store with an xtask policy rule
- **Where:** `build-plan.md` T-003 Touches (`crates/xtask/src/policy.rs`) and EARS (last clause)
- **Quote:** "WHEN `cargo policy` runs, THEN a new `fs-boundary` rule SHALL fail …"
- **Failure mode:** granularity
- **Why it matters:** The policy rule is a separate concern with its own diff. It also changes the hard-coded `PASS … spec\n{GOLDEN}` output in the `policy.rs` tests (around lines 1478–1495) and `RULES`' length. It does not depend on the store, so it could land with T-002.
- **Suggested response:** fix-in-plan (split into its own task, or move it to T-002), or defer-with-rationale.

### C-015: The credit for the derived-heads design is not attached to the task that adopts it
- **Where:** `build-plan.md` T-003 Touches; `test-plan.md` "AC-5 credits | T-002, T-006"
- **Quote:** "T-002, T-006 / new designs credited"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** T-003 adopts heads derived from op files after jj's `simple_op_heads_store` (research External Sources). `docs/prior-art.md` is not in T-003's Touches, and the `credits` rule checks only third-party dependencies, never designs. So INT-0003 AC-5 coverage for this design rests on an unassigned review.
- **Suggested response:** fix-in-plan. Add `docs/prior-art.md` to T-003 with the jj and ALICE (OSDI 2014) credits.

## Confidence
block

## Responses

The round blocked. Every concern was addressed in the plans and intents before round 2; tasks were renumbered (the filesystem boundary is now its own task, T-003), so task numbers below are the new ones.

| Concern | Response |
|---------|----------|
| C-001 | fix-in-plan. T-002 adds the clause and test `strict_mode_loses_unsynced_directories` (an unsynced directory vanishes with everything under it). `init` is now idempotent: it creates whatever is missing, syncs, and completes an interrupted init (`init_is_idempotent`). T-005's layout invariant says what an unacknowledged init recovers to: `open` fails as not-a-store, or succeeds with no ops, and a following `init` succeeds; after an acknowledged init, `open` succeeds. |
| C-002 | fix-in-plan. The pointer-file variant keeps `HEAD` rewritten in place (truncate and write, no temporary file, rename or sync) and recovery takes heads from it. It must produce closure, durability or heads violations, counted apart from the stray-file check. |
| C-003 | fix-in-plan. "Present" now means a file whose content hashes to its name and decodes, as `fsck` checks it. The unsynced-objects variant is defined exactly: it skips `sync_file` on object files but still renames them and syncs `objects/`, so a durable name with lost or torn content is a closure violation. |
| C-004 | fix-in-plan. A present target is still synced with its directory before returning (new INT-0007 Consequence; test `present_files_are_synced_again`). Temporary names are `tmp-<nonce>` from the store's `Rng`, retried on an existing name; `leftover_tmp_does_not_block_a_put` covers a leftover. |
| C-005 | fix-in-plan. INT-0007 AC-3 is revised (Transition history updated): heads equal the last acknowledged heads extended only by in-flight ops, each wholly present and valid or absent. T-005's heads invariant uses that wording, and a separate atomicity (prefix) invariant traces to INT-0005 AC-2. |
| C-006 | fix-in-plan. `store` lists itself as a dev-dependency with `features = ["variants"]`; no `required-features`. The matrix fails if any variant ran zero cells, and the evidence quotes the counts. Unification into test builds is stated; `policy_rejects_variants_outside_dev_dependencies` keeps the feature out of non-dev dependencies. |
| C-007 | fix-in-plan. Names are `<algorithm id as two hex digits>-<digest hex>`, recorded as an INT-0007 Consequence with the NTFS reason; `file_names_round_trip_and_are_portable` and `fsck_detects_unparsable_names` test it. |
| C-008 | fix-in-plan. `properties.rs` joins T-001's Touches, with its four affected assertions named. The registry status stays `defined`; provisional status is a separate note plus a Change-control bullet. `0x02` joins the still-reserved check (`other_kinds_stay_reserved`). Test names now match the existing ones; the `policy.rs` touch is dropped. |
| C-009 | fix-in-plan. INT-0006's Intent now defines provisional kinds (pinned like any other, excluded from AC-6 sign-off and freezing until the Phase 1 exit gate, required to carry the object set's fields before freezing) and records that `objects` stands in for the view hash; Consequences add that provisional-format stores are development stores. The plan asks the owner to sign off only Sprint 0's frozen sections. |
| C-010 | fix-in-plan. T-008 now checks `current_exe()` lies inside `<build-time root>/target`, which is what an outside `CARGO_TARGET_DIR` or `build.target-dir` changes for the alias-built binary, and compares `cargo locate-project --workspace` (inherited environment) with the build-time root. Both exit 1, as environment failures; exit 2 stays for usage errors. |
| C-011 | fix-in-plan. The boundary is enforced by Clippy, which resolves paths: crate-level `clippy.toml` files for `sim`, `store` and `praps` ban the `std::fs` types and functions and the `Path` methods that touch the filesystem, and keep the hash-collection bans. `praps` is in scope. `cargo policy`'s `fs-boundary` rule requires those configs and allows the opt-out only in `sim/src/real.rs` and `tests/` files. |
| C-012 | fix-in-plan. The whole matrix runs twice and every cell is compared, matching INT-0005 AC-1's proxy. |
| C-013 | fix-in-plan. Crash point *k*: syscalls before *k* take effect, *k* does not and fails, later ones fail without effect (`crash_point_boundary_is_exact`). The survival clause excludes a later rename-over or removal. |
| C-014 | fix-in-plan. The policy and Clippy boundary is its own task, T-003, depending on T-002. |
| C-015 | fix-in-plan. T-004 touches `docs/prior-art.md` with the jj and ALICE credits; the AC-5 row names T-002 and T-004. |
