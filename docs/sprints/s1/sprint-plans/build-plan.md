# Sprint 1 Build Plan

## Intents
- [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) — state: active; acceptance criteria covered: AC-1 (every crash-matrix cell run twice gives identical recovered state and trace), AC-2 (the crash matrix: crash at every syscall, ≥ 40 seeds per scenario, lenient and strict directory semantics, 0 violations of referential integrity, durability of acknowledged ops and atomicity), AC-3 (the three named negative controls each produce violations through the crash model, not through a structural check), AC-4 (re-checked for the new crates, whose own Clippy configs keep the hash-collection bans). AC-5 (network faults) stays with INT-0024. The Consequence "forbids casual `std::fs` use in core crates" is made a checked rule by T-003.
- [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) — state: planned; acceptance criteria covered: AC-1, AC-2, AC-3 as revised after the round-1 plan critique (in-flight ops may survive, wholly valid or absent), and AC-4 (`fsck` corruption corpus, 100% detection and 0 false reports). AC-5 (four writing processes) is deferred to a later sprint; this sprint's two interleaved writers in one process are supporting evidence only.
- [INT-0006](../../../intents/INT-0006-format-v1.md) — state: active; acceptance criteria covered: AC-1, AC-3 and AC-5 for the `operation` body (round trip, golden files, total decoding), and AC-6's first half (the body is specified in `docs/spec/format-v1.md` before its code). The body is *provisional* as INT-0006 now defines it (revised after the round-1 plan critique), so it is excluded from the owner's AC-6 sign-off until the Phase 1 exit gate; the sign-off still requested covers only Sprint 0's frozen sections.
- [INT-0004](../../../intents/INT-0004-windows-and-linux.md) — state: active; acceptance criteria covered: AC-1 for this sprint's code (Windows host and WSL ext4), and AC-5 up to the reviewer's acceptance (the documented Windows durability strategy with simulated and real-filesystem evidence; the owner's acceptance is requested at the checkpoint).
- [INT-0003](../../../intents/INT-0003-engineering-baseline.md) — state: active; acceptance criteria covered: AC-1, AC-8 and AC-9 for the new crates and the `praps` binary (no `unsafe`, one-word names, no flags), AC-5's credits for the designs adopted this sprint (ALICE's fault model, jj's file-per-op-head store, SplitMix64), and the carried-forward backlog T-010 (T-008 here), which hardens the `cargo ci` runner that realizes AC-2 to AC-4.
- [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md) — state: proposed and active; acceptance criteria covered: only through T-009 (backlog T-009), which records Sprint 0's GitHub Actions runs and whichever owner sign-offs (INT-0001 AC-1, INT-0002 AC-1, INT-0003 AC-5, INT-0006 AC-6) have actually been given.
- Context, not advanced: [INT-0009](../../../intents/INT-0009-reversible-by-default.md) (the store offers no delete or overwrite of an object or op, so nothing is erased without a future signed erasure op) and [INT-0013](../../../intents/INT-0013-derived-data.md) (op heads are derived, never stored).

## Schema Tree
- Sprint Goal: deterministic simulation and a crash-tested store on Windows and Linux
  - Format
    - T-001: Specify and implement the provisional `operation` object kind
  - Simulation
    - T-002: Filesystem, clock and randomness traits in the `sim` crate
    - T-003: Filesystem boundary: only `sim`'s real half touches the real filesystem
  - Storage
    - T-004: Object and op-file store with derived heads and `fsck` in the `store` crate
    - T-005: Crash matrix with negative controls
    - T-006: `fsck` corruption corpus and the `praps fsck` command
    - T-007: Windows durability strategy and real-filesystem evidence
  - Carried forward
    - T-008: `cargo ci` refuses a target directory outside the checkout (backlog T-010)
    - T-009: Record Sprint 0's GitHub Actions runs and owner sign-offs (backlog T-009)

## Execution Sequence

### T-001: Specify and implement the provisional operation object kind
- **Intent:** [INT-0006](../../../intents/INT-0006-format-v1.md)
- **Touches:** docs/spec/format-v1.md, crates/codec/src/envelope.rs, crates/codec/src/operation.rs, crates/codec/src/lib.rs, crates/codec/tests/golden.rs, crates/codec/tests/golden/v1/operation-root.bin, crates/codec/tests/golden/v1/operation-merge.bin, crates/codec/tests/spec.rs, crates/codec/tests/properties.rs
- **Depends on:** (none)
- **Acceptance criterion:** INT-0006 AC-6 (specified before its implementation lands; provisional, so not yet signed off), AC-1 (canonical round trip), AC-3 (golden files), AC-5 (total decoding)
- **Success criterion (EARS):**
  - **WHEN** the spec commit is made, **THEN** `docs/spec/format-v1.md` **SHALL**:
    - give `0x03 operation` the registry status `defined`;
    - specify its body as `parents` (a canonical set of `HashRef`) ‖ `objects` (a canonical set of `HashRef`);
    - mark it provisional in a separate note, with Change control gaining a bullet for provisional kinds as INT-0006 defines them;
    - include worked examples for a root operation and a two-parent operation.

    The spec commit lands before or with the code, never after.
  - **WHEN** any `Operation` value is encoded and decoded, **THEN** the codec **SHALL** return the same value, and every byte string it accepts as an operation body **SHALL** re-encode to the same bytes (proptest, 10,000 cases each).
  - **WHEN** an operation body has unsorted or duplicate `parents` or `objects`, or trailing bytes, **THEN** decoding **SHALL** fail with the existing canonical-order, duplicate or trailing-bytes error.
  - **WHEN** `ObjectKind::from_id` reads `0x03`, **THEN** it **SHALL** return the operation kind; `0x02`, `0x04` and `0x05` **SHALL** still fail with `ReservedObjectKind`.
  - **WHEN** the golden and spec suites run, **THEN** `operation-root.bin` and `operation-merge.bin` **SHALL** equal the encodings of their corpus values byte for byte, the spec's worked examples **SHALL** match them, and the registry table **SHALL** match the code.
  - **WHEN** random byte strings are decoded as operation bodies, **THEN** decoding **SHALL** never panic (100,000 cases).
- **Notes:**
  - **Reuse:** `Encode`/`Decode` for `BTreeSet<HashRef>` in `crates/codec/src/wire.rs`, and `encode_object`/`object_id` in `envelope.rs`.
  - **Existing tests:** `properties.rs` (the reserved range `0x02..=0x05`, the class count `("reserved", 4)`, the "object kind 0x03 is reserved" message test) and `spec.rs` (`spec_registries_match_code`) are updated in the same commit; the golden test is `golden_v1_corpus_matches` and the examples test is `spec_examples_match_golden_files`.
  - **Deferred fields:** actor, time, view and signature are left to later sprints, as INT-0006 records.

### T-002: Filesystem, clock and randomness traits in the sim crate
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md)
- **Touches:** crates/sim/Cargo.toml, crates/sim/src/lib.rs, crates/sim/src/fs.rs, crates/sim/src/real.rs, crates/sim/src/simfs.rs, crates/sim/src/rng.rs, crates/sim/src/clock.rs, crates/sim/tests/contract.rs, Cargo.toml, Cargo.lock, docs/prior-art.md
- **Depends on:** (none)
- **Acceptance criterion:** INT-0005 AC-1 (same seed and inputs give byte-identical storage and identical traces) and the simulator half of AC-2 (a crash at any syscall, torn or lost unsynced writes, lenient and strict directory semantics)
- **Success criterion (EARS):**
  - **WHEN** a run is given crash point *k*, **THEN** syscalls 1 to *k* − 1 **SHALL** take effect, syscall *k* **SHALL NOT** take effect and **SHALL** return the crash error, and every later syscall **SHALL** return the crash error without changing state. A crash after syscall *k* completes is the run with crash point *k* + 1.
  - **WHEN** two `SimFs` runs execute the same syscall sequence with the same seed and crash point, **THEN** their recovered durable states **SHALL** be byte-identical and their syscall traces equal.
  - **WHEN** a crash occurs after a file's `sync_file` and its directory's `sync_dir` both returned, and the file was not later renamed over or removed, **THEN** its name and content **SHALL** survive unchanged in both modes.
  - **WHEN** a crash occurs with written but unsynced file data, **THEN** across seeds the recovered content **SHALL** be observed kept, dropped and torn to a prefix, and never containing bytes that were not written.
  - **WHEN** in strict mode a file is created or renamed, and its data synced, but its directory is not synced, **THEN** across seeds the entry **SHALL** sometimes vanish (and a rename **SHALL** sometimes revert) after a crash; **WHEN** the same happens in lenient mode, **THEN** the entry **SHALL** survive.
  - **WHEN** in strict mode a directory is created but its parent is not synced, **THEN** across seeds the directory **SHALL** sometimes vanish after a crash together with everything below it; **WHEN** the same happens in lenient mode, **THEN** it **SHALL** survive.
  - **WHEN** `Rng` (SplitMix64) is seeded with 0 and with 1234567, **THEN** its first outputs **SHALL** match the published reference values.
  - **WHEN** the contract suite runs against `RealFs` in a freshly claimed temporary directory, **THEN** it **SHALL** pass on Windows and Linux, `sync_dir` included.
- **Notes:**
  - **Contract:** `create_new` refuses an existing name; `rename` replaces the target, as `std::fs::rename` does on both platforms.
  - **Directory sync:** on Windows, `RealFs::sync_dir` opens the directory with `OpenOptions::read(true).write(true)` and `custom_flags(0x0200_0000)` (`FILE_FLAG_BACKUP_SEMANTICS`), then calls `sync_all`; on Unix it opens it read-only and calls `sync_all`.
  - **Lenient mode:** a file's `sync_file` also makes that file's own directory entry durable, and `create_dir` is durable at once.
  - **Determinism:** ordered maps only (INT-0005 AC-4), and no third-party dependency.
  - **Fresh directories:** temporary directories are claimed fresh (never reused) to avoid the Sprint 0 known limit.

### T-003: Filesystem boundary: only sim's real half touches the real filesystem
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- **Touches:** crates/sim/clippy.toml, crates/sim/src/real.rs, crates/xtask/src/policy.rs
- **Depends on:** T-002
- **Acceptance criterion:** INT-0005's Consequence that I/O in core crates goes through the traits, so the simulator's results speak for the real store; INT-0005 AC-4 must keep holding for every crate
- **Success criterion (EARS):**
  - **WHEN** Clippy lints `sim`, `store` or `praps`, **THEN** a crate-level `clippy.toml` **SHALL** reject:
    - the `std::fs` types `File`, `OpenOptions` and `DirBuilder`;
    - every `std::fs` free function;
    - the `std::path::Path` methods that touch the filesystem (`exists`, `try_exists`, `is_file`, `is_dir`, `metadata`, `symlink_metadata`, `read_dir`, `read_link`, `canonicalize`).

    The config **SHALL** also keep the root's `HashMap`/`HashSet` bans.
  - **WHEN** code needs the real filesystem, **THEN** an `#[allow(clippy::disallowed_methods, clippy::disallowed_types)]` **SHALL** appear only in `crates/sim/src/real.rs` and in files under a crate's `tests/` directory.
  - **WHEN** `cargo policy` runs:
    - the `clippy` rule **SHALL** accept a member's own `clippy.toml` only if it lists every root `disallowed-types` path;
    - a new `fs-boundary` rule **SHALL** fail if `sim`, `store` or `praps` lacks a config listing the filesystem bans above, or if such an allow appears anywhere else in those crates.
  - **WHEN** the rule's fixtures run, **THEN**:
    - a store crate without the config **SHALL** fail;
    - an allow in `store/src` **SHALL** fail;
    - a member config missing a hash-collection ban **SHALL** fail;
    - the real layout **SHALL** pass.
- **Notes:**
  - **Why Clippy:** it resolves paths, so brace imports (`use std::{fs, io}`), `::std::fs` and `Path::exists` are all caught, which a text match is not.
  - **Rollout:** the rule applies to whichever of `sim`, `store` and `praps` exist, so each commit stays green. `store` and `praps` get their configs in the same commits that create them (T-004, T-006), and from then on the rule fails if a config is removed.
  - **Test outputs:** the hard-coded `PASS` output in `policy.rs`'s tests gains the new rule's line.

### T-004: Object and op-file store with derived heads and fsck in the store crate
- **Intent:** [INT-0007](../../../intents/INT-0007-crash-safe-storage.md)
- **Touches:** crates/store/Cargo.toml, crates/store/clippy.toml, crates/store/src/lib.rs, crates/store/src/fsck.rs, crates/store/src/names.rs, crates/store/tests/store.rs, Cargo.toml, Cargo.lock, docs/prior-art.md
- **Depends on:** T-001, T-002, T-003
- **Acceptance criterion:** INT-0007 AC-3 (heads recomputed from op files, no HEAD or pointer file) and the checker behind AC-4; its Consequences (on-disk names, syncing an already-present file); INT-0009 as context (no erasure path)
- **Success criterion (EARS):**
  - **WHEN** `Store::init` runs, **THEN** it **SHALL** create whichever of `objects/` and `ops/` is missing, sync both and the repository directory, and return only after the last sync. Running it again on a complete store **SHALL** change nothing and succeed, so `init` is idempotent and completes an interrupted init.
  - **WHEN** `Store::open` runs on a directory that lacks `objects/` or `ops/`, **THEN** it **SHALL** fail with a not-a-store error; after an acknowledged `init` it **SHALL** succeed.
  - **WHEN** `put_object` stores content, **THEN** `get_object` **SHALL** return the same envelope bytes, from a file whose name is `<algorithm id as two hex digits>-<digest hex>`; that name **SHALL** parse back to the same `HashRef` and contain no character Windows forbids.
  - **WHEN** `put_object` or `commit_op` finds its target file already present, **THEN** it **SHALL** still `sync_file` that file and `sync_dir` its directory before returning.
  - **WHEN** a store writes a file, **THEN** it **SHALL** write `tmp-<nonce>` with a nonce drawn from its `Rng`, retry with a new nonce on an existing name, `sync_file`, rename to the final name, and `sync_dir`. A leftover `tmp-*` from a crash **SHALL NOT** block putting the same object again.
  - **WHEN** `commit_op` names a parent op or an object that is not present, **THEN** it **SHALL** fail and leave no new file, temporary files included.
  - **WHEN** ops form a chain, a fork or a merge, **THEN** `heads()` **SHALL** return exactly the valid ops that no other present op names as a parent, computed from the files in `ops/`.
  - **WHEN** any store operation runs, **THEN** no file other than `objects/*`, `ops/*` and transient `tmp-*` files **SHALL** be created, and the API **SHALL** offer no delete or overwrite of an object or op.
  - **WHEN** `fsck` runs, **THEN** it **SHALL** report each file whose name does not parse, whose content does not hash to its name, that does not decode, or that names a missing parent or object. It **SHALL** list `tmp-*` files as leftovers without reporting them as corruption.
- **Notes:**
  - **Acknowledgement:** `commit_op` returns its id only after the final `sync_dir`.
  - **Credits:** `docs/prior-art.md` credits jj's `simple_op_heads_store` (op heads as one file per op id; Periapsis drops head files altogether, after R2-E23b) and ALICE (Pillai et al., OSDI 2014) for the write sequence's fault model.
  - **Filesystem boundary:** `store` gets its `clippy.toml` (T-003) in the same commit.

### T-005: Crash matrix with negative controls
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0007](../../../intents/INT-0007-crash-safe-storage.md)
- **Touches:** crates/store/Cargo.toml, crates/store/src/lib.rs (the `Variant` switch, behind the `variants` feature), crates/store/tests/crash.rs
- **Depends on:** T-004
- **Acceptance criterion:** INT-0005 AC-1, AC-2 and AC-3; INT-0007 AC-1, AC-2 and AC-3 (as revised)
- **Success criterion (EARS):**
  - **WHEN** the matrix runs each scenario with a crash at every syscall index, under 40 seeds and both directory modes, **THEN** for the correct store it **SHALL** report 0 violations of the invariants below. The scenarios are:
    - init alone;
    - init then a five-op chain with objects;
    - a fork and a merge;
    - two interleaved writers (two `Store` values) in one process.

    "Present" means a file whose content hashes to its name and decodes, as `fsck` checks it. The invariants:
    - **Layout:** if `init` was acknowledged, `Store::open` succeeds; otherwise either `open` fails as not-a-store, or it succeeds with no ops. In both cases a following `init` succeeds.
    - **Closure:** every recovered op's parents and objects are present (INT-0007 AC-1).
    - **Durability:** every acknowledged op is present (INT-0007 AC-2).
    - **Heads:** recomputed heads equal the heads of the last acknowledged state extended only by in-flight ops, each wholly present or absent, and no file outside `objects/*`, `ops/*` and `tmp-*` exists (INT-0007 AC-3 as revised).
    - **Atomicity:** in the single-writer chain scenario, the recovered op chain is a prefix of the intended chain (INT-0005 AC-2).
  - **WHEN** the whole matrix is run twice, **THEN** each cell's recovered-state hash and trace **SHALL** be equal between the runs (INT-0005 AC-1 proxy: every scenario twice).
  - **WHEN** the matrix runs the three negative-control variants, **THEN** each **SHALL** produce at least one violation of the closure, durability or heads invariant, counted separately from any layout or stray-file check, in strict mode at least. The variants are:
    - **Pointer file:** keeps a `HEAD` file holding the head set, rewritten in place (truncate and write, no temporary file, rename or sync), and takes heads from it in recovery.
    - **Unsynced objects:** skips `sync_file` on object files but still renames them and syncs `objects/`.
    - **Unsynced init:** `init` performs no `sync_dir`.
  - **WHEN** the matrix finishes, **THEN** it **SHALL** print the number of cells run and violations per variant and mode, and **SHALL** fail if any variant ran zero cells.
- **Notes:**
  - **Feature wiring:** `Variant` exists only under the non-default `variants` feature of `store`. `store` lists itself as a dev-dependency with `features = ["variants"]`, so `cargo test --workspace` (which `cargo ci` runs without `--features`) compiles the matrix and the controls; no test uses `required-features`.
  - **Feature unification:** in test builds this unifies the feature into every `store` build, including the one `praps`'s tests link. Release and non-test builds never enable it, and `cargo policy` fails if any non-dev dependency on `store` enables `variants`.
  - **Time budget:** the matrix must finish within the `cargo ci` budget (target: under 60 s per run in a debug build). If it does not, scenario sizes shrink before the seed count does, and the measured count goes into the evidence.

### T-006: fsck corruption corpus and the praps fsck command
- **Intent:** [INT-0007](../../../intents/INT-0007-crash-safe-storage.md), [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- **Touches:** crates/store/tests/fsck.rs, crates/praps/Cargo.toml, crates/praps/clippy.toml, crates/praps/src/main.rs, crates/praps/tests/cli.rs, Cargo.toml, Cargo.lock, README.md
- **Depends on:** T-004, T-005
- **Acceptance criterion:** INT-0007 AC-4 (100% detection, 0 false reports over a corruption corpus); INT-0003 AC-8 and AC-9 for the new binary
- **Success criterion (EARS):**
  - **WHEN** a corrupt store is checked, **THEN** `fsck` **SHALL** report the affected file in every case, with 100% detection over the corpus. The corpus covers:
    - an op file truncated to every shorter length;
    - a small object with each single bit flipped;
    - a referenced object missing;
    - a parent op missing;
    - a file renamed to a name that does not parse.
  - **WHEN** `fsck` runs on every recovered store from the correct store's crash matrix, **THEN** it **SHALL** report no corruption (temporary files only as leftovers).
  - **WHEN** `praps fsck` runs:
    - with `DIR` on a healthy store, **THEN** it **SHALL** exit 0;
    - on a corrupt store, **THEN** it **SHALL** list each corrupt file and exit 1;
    - with an argument starting with `-`, an unknown subcommand, an extra argument, or a directory that is not a store, **THEN** it **SHALL** exit 2.
- **Notes:** `praps` is a one-word binary package (`publish = false`); reserving the name on crates.io stays an owner action (INT-0003 AC-7). `fsck` with no directory checks the current directory. The CLI tests corrupt files with `std::fs` under a test-file allow (T-003).

### T-007: Windows durability strategy and real-filesystem evidence
- **Intent:** [INT-0004](../../../intents/INT-0004-windows-and-linux.md), [INT-0007](../../../intents/INT-0007-crash-safe-storage.md)
- **Touches:** docs/spec/storage-durability.md, crates/sim/tests/real_durability.rs, crates/store/tests/real_store.rs, docs/SUMMARY.md
- **Depends on:** T-002, T-004
- **Acceptance criterion:** INT-0004 AC-5 (the durability strategy on Windows is documented and a reviewer accepts its crash-test evidence)
- **Success criterion (EARS):**
  - **WHEN** a reader opens `docs/spec/storage-durability.md`, **THEN** it **SHALL** state:
    - the write sequence and the on-disk names;
    - which steps the simulator models, and how lenient and strict modes differ;
    - how `rename` and `sync_dir` are realized on Windows and on Linux;
    - the evidence for the Windows directory flush, and that it is empirical rather than documented by Microsoft;
    - what remains asserted.
  - **WHEN** the real-filesystem test runs on Windows, **THEN** a directory handle opened with write access and backup semantics **SHALL** flush with `Ok`, and one opened read-only **SHALL** fail with a permission error.
  - **WHEN** a store is initialized and written through `RealFs` on Windows and on Linux, **THEN** reopening it **SHALL** give the same heads and objects, and `fsck` **SHALL** report it clean.
- **Notes:** A power-loss test is impossible here. The note says so plainly and leaves acceptance to the reviewer.

### T-008: cargo ci refuses a target directory outside the checkout
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- **Touches:** crates/xtask/src/ci.rs, crates/xtask/src/main.rs
- **Depends on:** (none)
- **Acceptance criterion:** INT-0003 AC-2 to AC-4, as realized by `cargo ci`: the steps it runs must be this checkout's own (Sprint 0 test critiques round 3, C-005; round 5, C-006; round 6, C-008; round 7, C-007)
- **Success criterion (EARS):**
  - **WHEN** the running xtask binary (`std::env::current_exe()`, canonicalized) is not inside `<workspace root>/target`, where the workspace root comes from the build-time `CARGO_MANIFEST_DIR`, **THEN** `cargo ci` and `cargo policy` **SHALL** refuse before doing anything, naming both paths, and exit 1. This is the case when the `cargo ci` alias built xtask into a `CARGO_TARGET_DIR` or `build.target-dir` outside the checkout.
  - **WHEN** the current directory's workspace (`cargo locate-project --workspace`, run with the inherited environment) is not the build-time workspace root, **THEN** xtask **SHALL** refuse and exit 1, because it was built for another checkout.
  - **WHEN** xtask runs from the default in-tree `target/` in its own checkout, **THEN** `cargo ci` **SHALL** run its steps as before.
- **Notes:**
  - **Exit code:** exit 1, not 2, because this is an environment failure rather than a usage error (INT-0003 AC-9 reserves 2 for usage).
  - **Child steps:** they keep scrubbing `CARGO_TARGET_*` and `CARGO_BUILD_*`, as before.
  - **Backlog:** this absorbs backlog entry T-010, whose entry is removed when this task completes. Another checkout's cargo command building into this checkout's in-tree `target/` stays a recorded open limit, as the backlog entry states.

### T-009: Record Sprint 0's GitHub Actions runs and owner sign-offs
- **Intent:** [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md), [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0006](../../../intents/INT-0006-format-v1.md)
- **Touches:** docs/sprints/s1/sprint-tests/ci-sprint0.md, docs/intents/INT-0001-periapsis-charter.md, docs/intents/INT-0002-typed-acceptance-criteria.md, docs/intents/INT-0003-engineering-baseline.md, docs/intents/INT-0006-format-v1.md
- **Depends on:** (none)
- **Acceptance criterion:** the GitHub-matrix halves of INT-0003 AC-2 to AC-4 and INT-0006 AC-3; the sign-off criteria INT-0001 AC-1, INT-0002 AC-1, INT-0003 AC-5, INT-0006 AC-6, as far as the owner has given them
- **Success criterion (EARS):**
  - **WHEN** the record is written, **THEN** it **SHALL** give, for runs 42227550987 (push) and 42227551468 (pull request) at `5062157`:
    - each job's conclusion;
    - the runner OS image and `rustc` version, quoted from the job logs;
    - the golden-suite and `cargo ci` step results of both matrix legs.
  - **WHEN** the record is written, **THEN** it **SHALL** also give the result of PR #2's run with the pinned `checkout@v7` workflow (run 42303659961).
  - **WHEN** an owner sign-off has not been given in the conversation or on a PR, **THEN** the chapter **SHALL** keep that criterion unsigned, and the Loop **SHALL** carry it forward again; no sign-off is recorded on the owner's behalf.
- **Notes:** This absorbs backlog entry T-009. The owner merging PR #1 is recorded as acceptance of the checkpoint, not as any of the four named sign-offs.
