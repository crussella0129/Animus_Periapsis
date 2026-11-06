# Sprint 1 Research Report

## Intents Reviewed
- [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) — **selected**; relevance: the `sim` crate, the crash matrix and its negative controls (AC-1 to AC-3) arrive with the first storage code, as Sprint 0 planned; AC-4 (no randomly seeded hash collections) already holds workspace-wide; AC-5 (network faults) stays with INT-0024; current state: active
- [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) — **selected**; relevance: the object store, one file per operation, op heads derived from the op files, durable initialisation and `fsck` (AC-1 to AC-4); AC-5 (four writing processes) is deferred; current state: proposed
- [INT-0006](../../../intents/INT-0006-format-v1.md) — **selected**; relevance: the store needs the reserved `operation` kind (`0x03`) specified, and the spec requires the body format to land in `docs/spec/format-v1.md` before the code; current state: active
- [INT-0004](../../../intents/INT-0004-windows-and-linux.md) — **selected**; relevance: AC-5 asks for a documented Windows durability strategy with crash-test evidence a reviewer accepts; current state: active
- [INT-0003](../../../intents/INT-0003-engineering-baseline.md) — **selected**; relevance: backlog task T-010 (refuse a target directory shared with another checkout); new crates must meet the guardrails (no `unsafe`, credited dependencies); current state: active
- [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md) — selected for backlog task T-009 only: record Sprint 0's GitHub Actions run and the owner's sign-offs; current state: proposed and active
- [INT-0009](../../../intents/INT-0009-reversible-by-default.md) — reviewed, not selected; relevance: the op log must stay injective and erasure explicit, so this sprint's store has no delete path; undo and restore need views (Phase 1 later); current state: proposed
- [INT-0013](../../../intents/INT-0013-derived-data.md) — reviewed, not selected; relevance: op heads are derived data, recomputed from the op files and never stored; current state: proposed

## 1. Sprint Goal
Start Phase 1's storage with the deterministic simulation it requires. Build a
`sim` crate with filesystem, clock and randomness traits, real and simulated.
The simulated filesystem loses or tears unsynced writes at a crash injected at
any syscall, under lenient and strict-POSIX directory semantics. Build a
`store` crate on it:
- content-addressed object files;
- one file per operation, as the newly specified `operation` object;
- op heads derived from the op files, with no HEAD file;
- durable initialisation and `fsck`.

Prove the store with a seeded crash matrix and three negative controls, and
document the Windows durability strategy. Carry forward T-009 and T-010.

Out of scope:
- views, changesets and the working copy;
- compression (INT-0008);
- multi-process writers (INT-0007 AC-5);
- network simulation;
- signatures;
- a `praps` command beyond what `fsck` needs.

## 2. Existing Code Survey
| File | Relevance | Notes |
|------|-----------|-------|
| `docs/intents/INT-0005-deterministic-simulation-testing.md` | high | AC-1 determinism, AC-2 crash matrix (≥ 40 seeds per scenario, 0 violations), AC-3 three named negative controls, AC-4 already enforced by `clippy.toml` |
| `docs/intents/INT-0007-crash-safe-storage.md` | high | Per-op files, objects durable before the op, no HEAD file, durable init, `fsck` corruption corpus |
| `docs/intents/INT-0004-windows-and-linux.md` | high | AC-5: Windows durability strategy accepted by a reviewer |
| `docs/intents/INT-0009-reversible-by-default.md` | medium | No erasure path without a signed op, so the store has no delete or overwrite of objects or ops |
| `docs/sprints/s0/sprint-research/master-report-v6.md` (I.5 step 8, R2-E23, R2-E23b, R2-E27) | high | Prototype design and thresholds: 0 violations / 2,600 scenarios; controls 84, 1,457 and 48; heads derived from op files with 37% fewer syscalls. The prototype is not in this repository (Sprint 0 risk), so the harness is rebuilt from this description |
| `docs/spec/format-v1.md` | high | `0x03 operation` is reserved; specifying it adds rows and worked examples and "lands before the code that implements it"; v1 freezes at the Phase 1 exit gate |
| `crates/codec/src/envelope.rs` | high | `ObjectKind`, `encode_object`, `decode_envelope`, `object_id`; reserved kinds are rejected with `ReservedObjectKind` until specified |
| `crates/codec/src/wire.rs` | high | `Encoder`/`Decoder` primitives (sorted sets, `HashRef`) the operation body reuses |
| `crates/hash/src/lib.rs` | medium | `HashAlg`, `HashRef` and its text form, usable for file names |
| `crates/codec/tests/golden.rs` | medium | Golden-corpus pattern to extend with operation examples |
| `crates/xtask/src/ci.rs`, `crates/xtask/src/main.rs` | high | T-010 touches both |
| `crates/xtask/src/policy.rs` | medium | New crates must pass `lints`, `deps` and `credits`; no new third-party dependency is needed |
| `clippy.toml` | medium | Bans `HashMap`/`HashSet` workspace-wide (INT-0005 AC-4) |
| `Cargo.toml` | medium | Workspace members are `crates/*`; `unsafe_code = "forbid"` rules out raw Win32 calls |
| `docs/work/tasks.md` | high | Backlog: T-009 (record Sprint 0 CI and sign-offs), T-010 |
| `docs/sprints/s0/sprint-tests/code-review.md` | medium | Known limit: test fixtures reuse a per-process scratch directory; the store's tests should claim fresh directories instead |

## 3. External Sources
- [Pillai et al., "All File Systems Are Not Created Equal", OSDI 2014](https://research.cs.wisc.edu/adsl/Publications/alice-osdi14.html) — application crash-consistency depends on persistence properties that differ by file system: writes and a later rename are not ordered without an fsync, and a new file's directory entry needs an fsync of its parent. This is the fault model the simulator implements: unsynced data may be lost or torn, and in strict mode unsynced directory entries may vanish.
- [jj-lib `simple_op_heads_store.rs`](https://docs.rs/jj-lib/0.42.0/src/jj_lib/simple_op_heads_store.rs.html) — Jujutsu keeps op heads as empty files named by operation id under `heads/`, and takes a lock only to avoid duplicate resolution of divergent heads, not for correctness. Periapsis goes one step further (R2-E23b): no head files at all; heads are the op files no other op names as a parent.
- [rust-lang/rust#137528](https://github.com/rust-lang/rust/pull/137528/files) and [#134616](https://github.com/rust-lang/rust/pull/134616) — `std::fs::rename` on Windows replaces an existing target and uses POSIX rename semantics (`FileRenameInfoEx`) where the OS supports them, so write-temporary-then-rename works there too. A rename is not durable until the directory is flushed.
- [Microsoft Q&A, NTFS namespace durability without administrator rights](https://learn.microsoft.com/en-us/answers/questions/6057938/supported-non-administrative-ntfs-namespace-durabi) — `FlushFileBuffers` is not documented for directory handles, and a volume flush needs administrator rights. Other storage projects open the directory with `FILE_FLAG_BACKUP_SEMANTICS` and write access and flush it ([apache/maka#4520](https://github.com/apache/maka/pull/4520); [dgraph-io/badger#699](https://github.com/dgraph-io/badger/issues/699) discusses the read-only misuse). Probed on this host (Windows 11, NTFS, safe `std` only: `OpenOptions::write(true)` plus `custom_flags(0x0200_0000)`): the flush returns `Ok`, while a read-only directory handle is refused with "Access is denied". It works, but is an empirical guarantee, not a documented one; INT-0004 AC-5 is `asserted` for this reason.

## 4. Risks, Unknowns, Dependencies
- **Risk:** the simulator is a model. A store proved against it is only as safe as the model is faithful. Mitigations: the model follows the OSDI 2014 fault classes; three negative controls must each fail it (INT-0005 AC-3); and a real-filesystem test on both platforms checks that every simulated syscall maps to the real call the store makes.
- **Risk:** Windows directory durability is undocumented (see External Sources). The strategy is documented with its evidence, and the owner's acceptance is requested at the checkpoint (INT-0004 AC-5).
- **Risk:** specifying the `operation` body now fixes an encoding that later work extends: signatures (INT-0016), the view an op produces (INT-0014) and erasure records (INT-0009). Any field added later changes existing encodings, which the spec forbids. So the operation body carries only what the store needs now, `parents` and the object ids the op makes reachable, and is marked *provisional until the Phase 1 exit gate* in the spec. Format v1 does not freeze before then.
- **Risk:** sprint size. The core is the simulator, the store and the crash matrix. If time runs short, T-010 is the one to defer again.
- **Unknown:** how faithfully lenient mode matches NTFS and ext4 ordering. The crash matrix runs both modes, and the store must pass strict mode, which assumes the least.
- **Dependency:** no new third-party crates. A seeded PRNG (SplitMix64) is small enough to write and test against its published reference outputs, which keeps `deny` and `credits` unchanged.
- **Dependency:** Dependabot PR #2 (`actions/checkout` v4 → v7, with the reviewed copy in `pins.rs` updated on its head) was green and merged into `dev` at `7e84b43` by the owner before this sprint's first commit, so this sprint starts from it.
- **Dependency:** Sprint 0's CI evidence for T-009 is in GitHub Actions runs 42227550987 (push) and 42227551468 (pull request) at `5062157`, all jobs passing.

## 5. Recommended Approach
Primary, in dependency order:
1. **Specify the `operation` kind** in `docs/spec/format-v1.md`: body = sorted, unique `parents` (operation ids) ‖ sorted, unique `objects` (ids the op makes reachable). Add worked examples and golden files, and mark it provisional until the Phase 1 exit gate.
2. **`sim` crate**: a `Fs` trait over the syscalls the store uses (create directory, create-new file, write, sync file, rename, sync directory, read, list), plus `Clock` and a seeded `Rng`. `RealFs` uses `std::fs`; on Windows it flushes directories through a write-access backup-semantics handle. `SimFs` is in memory and tracks durable and pending state per file and per directory entry. A crash at syscall *k*, chosen by seed, drops or tears pending writes. Lenient mode makes an entry durable with its file's sync; strict mode needs the directory sync.
3. **`store` crate**: `init` creates `objects/` and `ops/` and syncs every directory. `put_object` writes a temporary file, syncs it, renames it to the hex id, and syncs the directory. `commit_op` checks that its parents and objects are present and writes the op the same way; it returns only after the directory sync, which is the acknowledgement. `heads()` recomputes the heads from the op files. `fsck` checks every file's hash against its name, decodes it, checks references, and ignores leftover temporary files.
4. **Crash matrix**: scenarios (init; an op chain with objects; two interleaved writers in one process) × every syscall × at least 40 seeds × both modes. It checks INT-0007 AC-1 to AC-3 and INT-0005 AC-1 (each scenario run twice and compared by hash). There are three negative controls, each a store variant behind a test-only switch and each required to produce violations: a pointer file written in place, objects not synced before the op, and no directory sync at init.
5. **Corruption corpus for `fsck`**: truncated op files, flipped bits in objects, missing referenced objects, plus healthy repositories.
6. **Windows durability note** and real-filesystem tests on both platforms.
7. **T-010** (`cargo ci` refuses an outside target directory) and **T-009** (record Sprint 0's CI run and sign-offs).

Alternative considered: adopting `turmoil` or `madsim` for simulation. Not chosen: they target network and async runtimes, while this sprint needs synchronous filesystem fault injection at every syscall with directory-entry semantics, which neither provides. Each would also add a dependency tree to credit and audit.

Alternative considered: a Lean-first sprint. Still blocked on the owner's toolchain decision.

Rationale: INT-0005 says simulation arrives with the first storage code, and Sprint 0's research named this storage-first sprint as the next one. Every later Phase 1 intent (views, working copy, reversibility) writes through this store, so its crash safety must be proved first.

## Artifacts
No files are saved. The Windows directory-flush probe was a throwaway program in the session scratchpad; its result is recorded under External Sources and will be re-proved by the real-filesystem test on Windows.
