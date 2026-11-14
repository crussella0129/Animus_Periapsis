# Prior art and credits

Periapsis is written from scratch ([INT-0003](intents/INT-0003-engineering-baseline.md)).
It adopts **designs** from the systems below, with credit, and copies **no
code** from any of them. A design is not code, so none of these licences binds
this repository; they are listed for reference. Licences were read from each
project's repository on 2026-10-31; "not verified" means no licence could be
confirmed from a primary source.

## Designs adopted

| Idea in Periapsis | Source | Licence of the source | Book intent |
|---|---|---|---|
| Lock-free operation log of views; divergent op heads merged on load; stable change IDs; working copy as a change; conflicts stored as data | [Jujutsu](https://github.com/jj-vcs/jj) | Apache-2.0 | INT-0010, INT-0011, INT-0012, INT-0014 |
| Phases (`draft`/`public`); obsolescence markers as a second-order history of rewrites | [Mercurial](https://www.mercurial-scm.org/) (evolve) | GPL-2.0-or-later | INT-0011, INT-0015 |
| `restack`/`absorb`; segmented changelog; per-file history; pushrebase | [Sapling](https://github.com/meta/sapling) | GPL-2.0 | INT-0011, INT-0023, INT-0030 |
| Bonsai-style canonical changesets; derived-data framework | Mononoke (part of [Sapling](https://github.com/meta/sapling)) | GPL-2.0 | INT-0006, INT-0013 |
| Content-addressed workflow at scale; worktree and source-control timings cited as payoff | Scion (Anthropic; described in a public engineering post) | not open source | INT-0003, INT-0030 |
| Signed, typed in-repository records; self-certifying repositories; staged fetch (verify, then download) | [Radicle](https://github.com/radicle-dev/heartwood) | Apache-2.0 | INT-0016, INT-0017, INT-0024 |
| Project records kept inside the repository | [Fossil](https://fossil-scm.org/) | BSD-2-Clause | INT-0017 |
| Content-addressed Merkle snapshots; lessons from the SHA-1 → SHA-256 transition | [Git](https://github.com/git/git) | GPL-2.0 | INT-0006, INT-0025 |
| Histogram diff | [JGit](https://github.com/eclipse-jgit/jgit) | EDL-1.0 (BSD-3-Clause) | INT-0012 |
| Structured merge limited to conflicting regions | [Mergiraf](https://codeberg.org/mergiraf/mergiraf), [Spork](https://github.com/ASSERT-KTH/spork) | GPL-3.0; MIT | INT-0012 |
| Compression against the previous version as a prefix | [zstd](https://github.com/facebook/zstd) | BSD-3-Clause OR GPL-2.0-only | INT-0008 |
| Deterministic simulation testing with crash injection | [TigerBeetle](https://github.com/tigerbeetle/tigerbeetle), [turmoil](https://github.com/tokio-rs/turmoil), [madsim](https://github.com/madsim-rs/madsim) | Apache-2.0; MIT; Apache-2.0 | INT-0005 |
| The simulator's fault model: unsynced data lost or torn, unsynced directory entries lost, writes and renames unordered without a sync (`sim`) | Pillai et al., "All File Systems Are Not Created Equal", OSDI 2014 (paper; the ALICE study) | paper | INT-0005, INT-0007 |
| Op heads kept as one file per operation id rather than a HEAD pointer; Periapsis goes further and derives heads from the op files themselves, with no head files (`store`) | [Jujutsu](https://github.com/jj-vcs/jj) `simple_op_heads_store`; R2-E23b | Apache-2.0 | INT-0007, INT-0013 |
| The seeded generator `sim::Rng` (SplitMix64) | Steele, Lea and Flood, OOPSLA 2014 (paper); Vigna's [`splitmix64.c`](https://prng.di.unimi.it/splitmix64.c) | public domain (reference code); written anew in Rust | INT-0005 |
| The working copy snapshotted into the working-copy change at the start of every command; `new` starting a change without touching files (`workspace`) | [Jujutsu](https://github.com/jj-vcs/jj) working-copy model | Apache-2.0 | INT-0014 |
| `undo` and `restore` as operations in the op log, so undo toggles and nothing is lost; finishing an interrupted working-copy update on the next command, as jj recovers a stale working copy (`workspace`) | [Jujutsu](https://github.com/jj-vcs/jj) operation log (`jj op undo`, `jj op restore`) and stale-working-copy recovery | Apache-2.0 | INT-0009, INT-0014 |
| The Windows names a write refuses: reserved device names with any extension, refused characters, trailing dots and spaces (`workspace::hazard`) | Microsoft, "Naming Files, Paths, and Namespaces" (Win32 documentation; specification only) | documentation | INT-0004 |
| Changesets as a canonical path → put/delete map against the first parent, trees derived (`codec`, `workspace`) | Mononoke Bonsai changesets (part of [Sapling](https://github.com/meta/sapling)) | GPL-2.0 | INT-0006, INT-0013 |
| Linked working copies over one repository: a small file at the top of each names the repository whose objects and history it shares, and each keeps a working copy of its own (`praps lane`, `.praps/workspace`) | Git linked worktrees ([`git worktree`](https://git-scm.com/docs/git-worktree)) | GPL-2.0 (design only) | INT-0014 |
| Merging against the merge of the latest common ancestors when there are several, the virtual merge base, for criss-cross histories of operations (`workspace::merge`) | Git's merge strategies ([`ort`, `recursive`](https://git-scm.com/docs/merge-strategies)) | GPL-2.0 (design only) | INT-0010 |
| Ignore rules with `.gitignore` semantics: per-directory files, last match wins, no re-inclusion under an excluded directory (`workspace`) | [Git](https://git-scm.com/docs/gitignore); matched by the [`ignore`](https://github.com/BurntSushi/ripgrep/tree/master/crates/ignore) crate | GPL-2.0 (specification only); Unlicense OR MIT | INT-0014 |
| Dormant named points in code, switched on by name and kept in a registry; the reach table of scenarios × units by sprint, the triangle of regression runs passing back through earlier units (`sim`, `cargo reach`) | The owner's proposal (Sprint 4: coded breakpoints collected in a table, and the run × unit diagram); fail points in [fail-rs](https://github.com/tikv/fail-rs) | owner; Apache-2.0 | INT-0005 |
| Orderings between named points across threads, kept as a table (`sim::Processes`) | RocksDB `TEST_SYNC_POINT` and `SyncPoint::LoadDependency` ([RocksDB](https://github.com/facebook/rocksdb)) | GPL-2.0 OR Apache-2.0 | INT-0005, INT-0007 |
| Fault sites marked in code that only the simulator activates | FoundationDB `BUGGIFY` ([FoundationDB](https://github.com/apple/foundationdb)) | Apache-2.0 | INT-0005 |
| Running a workspace's test executables in parallel instead of one after another, with the executables Cargo built as the list of what must run (`cargo ci`'s test step, written from scratch; no code taken) | [nextest](https://github.com/nextest-rs/nextest) (cargo-nextest's execution model) | Apache-2.0 OR MIT | INT-0003 |
| One logical thread at a time under a seeded scheduler, replayable from the seed (`sim::Processes`) | [shuttle](https://github.com/awslabs/shuttle); PCT, Burckhardt et al., ASPLOS 2010 (paper) | Apache-2.0; paper | INT-0005 |
| Speculative gated merge queue with an adaptive window; rollup classes; conflict analysis by build target | [Zuul](https://zuul-ci.org/), [bors](https://github.com/rust-lang/bors), Uber SubmitQueue (EuroSys 2019 paper) | Apache-2.0; Apache-2.0; paper | INT-0021 |
| Kernel replay, independent statement and axiom checks, second kernel | [Lean 4](https://github.com/leanprover/lean4), [comparator](https://github.com/leanprover/comparator), [nanoda](https://github.com/ammkrn/nanoda_lib), [lean4export](https://github.com/leanprover/lean4export) | Apache-2.0 (all four) | INT-0018, INT-0019 |
| Rust → Lean extraction for proofs about real code | [Aeneas](https://github.com/AeneasVerif/aeneas) | Apache-2.0 | INT-0018 |
| Proof-carrying code | Necula, POPL 1997 (paper) | — | INT-0018 |
| Attestation statements and signing envelopes, as export formats | [in-toto](https://in-toto.io/), [DSSE](https://github.com/secure-systems-lab/dsse) | not verified; Apache-2.0 | INT-0016 |
| Build provenance trust model (the signer of an evaluation must not be the evaluated party) | [SLSA](https://slsa.dev/) | not verified (specification) | INT-0016 |
| Per-line AI attribution, as an optional export | Agent Trace (provenance format) | not verified | INT-0016 |
| What `praps` says about itself: the command list when run alone, `help` under `-h` and `--help`, the nearest command for a mistyped word, and a refusal that says what to do next (`praps`) | [Command Line Interface Guidelines](https://clig.dev/) (licence read from its repository on 2026-11-10) | CC-BY-SA-4.0 (the guide's text; practice only, no text taken) | INT-0037 |
| The name `verify` for the integrity check | [Mercurial](https://www.mercurial-scm.org/) (`hg verify`) | GPL-2.0-or-later (name only) | INT-0037 |
| Agent interface protocol for `praps mcp` | [Model Context Protocol](https://modelcontextprotocol.io/) | not verified (specification) | INT-0022 |

## Designs evaluated and not adopted in core

| Idea | Source | Licence | Outcome |
|---|---|---|---|
| Commutative patch theory as the storage core | [Pijul](https://pijul.org/), Darcs | GPL-2.0-or-later | Deprioritised: no correctness gap observed (INT-0012) |
| Content-defined chunking for every file | [Xet](https://github.com/huggingface/xet-core) (FastCDC) | Apache-2.0 | Optional extension only (INT-0008) |
| Range-based set reconciliation as the sync protocol | [Negentropy](https://github.com/hoytech/negentropy) | MIT | Optional optimisation over head sets (INT-0024) |
| A map whose stored form depends only on its contents, never on the order of edits (a Prolly tree), so two versions share storage and a comparison skips equal subtrees | [Dolt](https://github.com/dolthub/dolt) (licence read on 2026-11-10) | Apache-2.0 | Evaluated in Sprint 6 research for the view; the view's form is not chosen yet (INT-0006) |
| CRDT record layer | Radicle collaborative objects | Apache-2.0 | Replaced by immutable events (INT-0017) |

## The owner's own projects

| Project | Licence | What Periapsis takes |
|---|---|---|
| [Animus_Sprint_Loops](https://github.com/crussella0129/Animus_Sprint_Loops) | MIT | The Project Book, evidence-derived phases, the sprint loop (INT-0026, INT-0027) |
| [CubiKan](https://github.com/crussella0129/CubiKan) | MIT | Intent units: immutable origins, revisioned lifecycle transitions (INT-0017) |
| [array-test](https://github.com/crussella0129/array-test) | MIT | Confirmation roots as a Merkle DAG of test results (INT-0015) |

## Third-party dependencies

Every entry of `[workspace.dependencies]` that is not a workspace crate has a
row here; `cargo policy` (rule `credits`) checks it. "Can build C" says whether
the crate's build script can compile C or assembly. For a `yes` row, the last
cell names the feature that keeps the build pure Rust, as
`features = ["…"]`, which `cargo policy` checks is enabled; otherwise it
starts with `C allowed:` and gives the reason.

| Crate | Version | Licence | Used by | Can build C | How Periapsis builds it |
|---|---|---|---|---|---|
| `blake3` | 1.8.7 | CC0-1.0 OR Apache-2.0 OR Apache-2.0 WITH LLVM-exception | `hash` | yes (its build script compiles C and assembly SIMD backends through `cc`) | `features = ["pure"]`: Rust implementations only, including Rust SIMD |
| `sha2` | 0.11.1 | MIT OR Apache-2.0 | `hash` | no | default features off; pure Rust |
| `proptest` | 1.12.0 | MIT OR Apache-2.0 | tests only | no | `default-features = false, features = ["std"]` |
| `pulldown-cmark` | 0.13.6 | MIT | `book`, `xtask`; tests (`codec`) | no (its build script only generates tests, behind the `gen-tests` feature) | `default-features = false`: the CommonMark parser without the HTML renderer and command-line options |
| `toml_edit` | 0.25.17 | MIT OR Apache-2.0 | `xtask` | no | `default-features = false, features = ["parse"]`: the parser only |
| `allocation-counter` | 0.8.1 | MIT OR Apache-2.0 | tests only (`codec`) | no | as published; it installs a counting global allocator, so only `crates/codec/tests/alloc.rs` uses it |
| `getrandom` | 0.4.3 | MIT OR Apache-2.0 | `sim` (real half only: `sim::os_seed`, which seeds change IDs outside simulation) | no (on Windows it calls `ProcessPrng` through a raw-dylib import, on Linux `getrandom(2)` through `libc`; no build script compiles C) | `default-features = false`; confined to `crates/sim/src/real.rs` by `sim/clippy.toml` and the `deps` rule |
| `ignore` | 0.4.34 | Unlicense OR MIT | `workspace` (only `gitignore::GitignoreBuilder::add_line` and `Gitignore::matched`, fed lines read through `sim::Fs`) | no (none of it or its dependencies, `globset`, `regex-automata`, `aho-corasick`, `bstr`, `memchr`, `walkdir`, `same-file`, `crossbeam-deque`, `winapi-util`, `log`, has a build script that compiles C) | `default-features = false` (`simd-accel` off); its filesystem readers are banned in `crates/workspace/clippy.toml` (`cargo policy`, rule `fs-boundary`) |

## Test data

| Data | Source | Licence | Where |
|---|---|---|---|
| SplitMix64 reference outputs (seeds 0 and 1234567) | Vigna's [`splitmix64.c`](https://prng.di.unimi.it/splitmix64.c) | public domain | `crates/sim/tests/contract.rs` |
| BLAKE3 test vectors (35 cases) | [`test_vectors.json`](https://github.com/BLAKE3-team/BLAKE3/blob/master/test_vectors/test_vectors.json) | CC0-1.0 OR Apache-2.0 | `crates/hash/tests/vectors.rs` |
| Two Sprint Loops intent chapters and their Book marker (commit `216867f`) | [Animus_Sprint_Loops](https://github.com/crussella0129/Animus_Sprint_Loops) `docs/` | MIT (notice kept in `NOTICE.md` beside the fixtures) | `crates/book/tests/fixtures/sprint-loops/` |
