# Plan Critique — Sprint 0, round 2 (superseded)

Provenance only. This was produced by a three-lens critic workflow. Three critics (traceability; intent and risk; dependencies and feasibility) raised 29 concerns. One adversarial verifier per critic tried to refute each, and 25 survived. A synthesis agent merged them into the 20 below. The final verdict is in [`critique.md`](critique.md); the resolution of each concern is listed at the end.


## Concerns (as synthesized)

### C-001: `cargo tree -e build -i cc` never reaches `blake3`, so the AC-6 audit checks nothing
- **Where:** `test-plan.md` Integration Tests `int_cc_build_dependency_audit`; INT-0003 AC-6 (the command written into the criterion); `build-plan.md` T-003 (no EARS clause states the pure-or-justified condition)
- **Quote:** "`int_cc_build_dependency_audit`: `cargo tree -e build -i cc --workspace` → the crates listed match the C-build column of `docs/prior-art.md` (`blake3` yes, built with `pure`)." / INT-0003 AC-6: "each crate that has `cc` as a build-dependency, per `cargo tree -e build -i cc`"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** With `-e build`, cargo follows only build-dependency edges out of the workspace members (rust-lang/cargo `src/ops/cargo_tree/graph.rs`: `if !opts.edge_kinds.contains(&EdgeKind::Dep(dep.kind())) { return false; }`). `blake3` is a normal dependency of `hash`, so `cc` never enters the graph and cargo only warns "nothing to print." and exits 0. The test therefore either fails for no real reason or passes without checking anything, and AC-6's own definition of the audited set is empty. It is also the only check of AC-6's pure-or-justified condition: the `credits` rule checks only that the C-build cell "starts with `yes` or `no`", and `pure` appears only in T-003 Notes.
- **Suggested response:** fix-in-plan: change the command to `cargo tree --workspace --target all -e normal,build -i cc` and require its output to list `blake3`. Add a T-003 EARS clause for the pure-or-justified condition. Amend INT-0003 AC-6's command, with a Transition-history entry, before locking.

### C-002: CI and owner-review evidence is assigned to Loop, but the checkpoint opens only after the sprint closes
- **Where:** `test-plan.md` Intent Traceability rows INT-0003 AC-2, AC-3 and AC-4, and the End-to-End "Not yet possible" note; INT-0003 Consequences; `test-plan.md` INT-0006 AC-3 row
- **Quote:** "The GitHub Actions conclusion becomes available when the checkpoint pushes `dev`; Loop records it as the CI evidence for INT-0003 AC-2 to AC-4." / "the checkpoint's GitHub run (recorded in Loop)"
- **Failure mode:** e2e-drift
- **Why it matters:** The Book's marker sets `substrate-version: 4`. Under that contract, `remote-adapter.sh open-pr` refuses ("refusing to open a checkpoint while the sprint is open") until `close-sprint.sh` writes a terminal Exit status, and `06-loop-phase.md` opens the checkpoint only "After the sprint is closed". That push is the first `dev` push and the first GitHub Actions run (there is no `origin/dev` and no `.github/workflows/`), so no Sprint 0 phase can record it. The plan also contradicts research-report §4 ("this sprint's authoritative test evidence is the local Windows and WSL runs"). Separately, INT-0006 AC-3's "CI fails on Windows and Linux" is verified only by local runs, while INT-0003 Consequences take "in CI" to mean the GitHub run, so the two intents read "CI" differently.
- **Suggested response:** fix-in-plan: replace "recorded in Loop" with a named carry-forward. Loop appends a `(backlog)` task carrying INT-0003 AC-2 to AC-4 and the checkpoint sign-offs (INT-0003 AC-5, INT-0006 AC-6). Sprint 1's Research or Test phase then records the Sprint 0 checkpoint run: head SHA, both matrix legs, the `deny` job, runner OS and `rustc`. Mark these criteria as carried forward rather than proven in Sprint 0. State in INT-0006 whether AC-3's "CI" means `cargo ci` or the GitHub run.

### C-003: No EARS clause requires `cargo ci` to pass, so the platform rows cite clauses that do not exist
- **Where:** `build-plan.md` T-007 EARS; `test-plan.md` Intent Traceability rows INT-0003 AC-4 and INT-0004 AC-1; `int_ci_windows`, `int_ci_wsl`, `e2e_fresh_clone_windows`, `ci_failures_do_not_stop_later_steps`
- **Quote:** "print one `PASS <step>` or `FAIL <step>: <reason>` line per step." / "T-007 / runner on both platforms" / "`ci_failures_do_not_stop_later_steps`: a failing step (including a missing tool) between passing steps → every step runs; the failed step is reported."
- **Failure mode:** plan-test-mismatch
- **Why it matters:** A runner that prints `FAIL` for every step satisfies all six T-007 clauses. No clause anywhere requires fmt and clippy to be clean (the whole of INT-0003 AC-4) or names a platform on which the suite must pass (INT-0004 AC-1), so the "every step PASS" assertions in the three platform tests trace to no clause. The continue-on-failure test uses a single failing step and asserts no exit code, so T-007's "exit non-zero, naming every failed step" is never tested.
- **Suggested response:** fix-in-plan: add a T-007 clause such as "WHEN `cargo ci` runs on the Windows 11 host checkout, in a WSL ext4 clone and in a fresh `core.autocrlf=true` Windows clone, THEN every step SHALL print `PASS` and the runner SHALL exit 0, with the OS release and `rustc -Vv` recorded". Point the INT-0003 AC-4 and INT-0004 AC-1 rows at it. Make the continue-on-failure test use two failing steps and assert a non-zero exit that names both.

### C-004: The golden harness has no negative control for a changed encoding
- **Where:** `build-plan.md` T-005 EARS (create-mode clause); `test-plan.md` T-005 unit tests and INT-0006 AC-3 row; T-007 `test` step
- **Quote:** "**WHEN** it runs with create mode on, **THEN** it **SHALL** write only missing files and **SHALL NOT** overwrite an existing file, even one that differs, which it still reports."
- **Failure mode:** plan-test-mismatch
- **Why it matters:** INT-0006 AC-3 promises that CI "fails ... whenever an encoding changes", yet no test runs the harness with create mode off against an existing file whose bytes differ. `golden_v1_corpus_matches` sees only matching files, `golden_create_mode_never_overwrites` runs with create mode on, and `golden_missing_file_fails_without_create_mode` uses an empty directory, so a harness whose comparison never fails passes all three. This is distinct from round-1 C-013, which covered only the missing-file branch. "Which it still reports" also leaves open whether a create-mode run fails on a differing file (T-005's first clause implies it must), and nothing stops `cargo ci`'s `test` step from inheriting a `PERI_GOLDEN_CREATE=1` left set in the shell.
- **Suggested response:** fix-in-plan: add `golden_changed_file_fails`: create mode off, a temp copy of the corpus with one byte flipped in one file → a failure that names that file. Restate the create-mode clause as "SHALL leave it unchanged and fail the run, naming it". Have the runner's `test` step remove `PERI_GOLDEN_CREATE` with `Command::env_remove`, which is safe in edition 2024.

### C-005: Port-rule checks pass even if the parser reads no criteria
- **Where:** `build-plan.md` T-006 EARS (`missing` clause and fixture-Book clause); `test-plan.md` `sprint_loops_fixtures_pass_port_rule`, `e2e_port_rule_sprint_loops_book`, `crlf_parses_like_lf`; INT-0002 AC-4
- **Quote:** "`e2e_port_rule_sprint_loops_book`: `cargo book ~/Animus_Sprint_Loops/docs` (an undeclared Book) → chapter count equal to that Book's `INT-*.md` files, 0 violations, exit 0."
- **Failure mode:** plan-test-mismatch
- **Why it matters:** The `missing` rule applies only to "a chapter of a declared Book", and neither the fixture test ("0 violations, all `necessary` `asserted`") nor the E2E asserts a criterion count. A parse that silently drops the corpus's criteria therefore passes, "all `necessary` `asserted`" holds over an empty set, and INT-0002 AC-4's "reads each untyped criterion as `necessary` `asserted`" is never checked. The risk is concrete. All 13 chapters at Animus_Sprint_Loops HEAD `e8c3a58` (82 top-level items) wrap untyped items onto two-space continuation lines. Five chapters have an item that starts with a backticked token (INT-0001 at `216867f`: "- `docs/` validates under `check-book.sh`"), and three contain " — " inside a criterion. A synthetic fixture may lack all of these features, and no clause says how a Book directory with no marker file is read.
- **Suggested response:** fix-in-plan: name the two vendored chapters, choosing ones with wrapped and backtick-led items, and assert their exact criterion counts and one joined claim. Have `cargo book` report criterion counts, and require the E2E total to equal an independent count of top-level `- ` items under the criteria heading (at minimum one per chapter). Give the CRLF fixture a wrapped criterion with a detail. State whether a Book with no marker is undeclared or an exit-2 error.

### C-006: T-005 relabels INT-0006 AC-6 as spec-code agreement, checks only registry rows, and freezes v1 before the owner's sign-off
- **Where:** `build-plan.md` T-005 Acceptance criterion, spec EARS clause and "Freeze rule" note; INT-0006 Intent (staged freezing) and AC-6; `test-plan.md` INT-0006 AC-6 row
- **Quote:** "AC-6 (code agrees with the spec)" / "**WHEN** the spec test parses the Hash algorithms and Object kinds tables in `docs/spec/format-v1.md`, **THEN** every row" / INT-0006: "An encoding is frozen once a golden file pins it."
- **Failure mode:** intent-drift
- **Why it matters:** The `.bin` files come only from the harness's create mode, so the codec under test produces them. Apart from T-004's varint and `u32` vectors and its rejection tests, nothing compares them with T-002's spec. Which of `00`/`01` means `None`/`false`, the pair and map-entry layout, `HashRef` inside `codec`, and the blob envelope `01 ‖ 01 ‖ content` are pinned only by the code that produced them. AC-6's owner sign-off happens at the checkpoint, after the sprint closes, but the freeze rule ("files under `v1/` never change") already applies, so any spec change the reviewer requests would force a format v2 for a v1 that was never approved.
- **Suggested response:** fix-in-plan: put the hex encoding of every golden corpus value into T-002's spec as worked examples, and have the T-005 spec test assert that each `.bin` equals the spec's hex. Otherwise, narrow T-005's claim to "registries agree with the spec". Record in INT-0006 that the Sprint 0 freeze takes effect at the owner's AC-6 sign-off, not at a golden file's first commit.

### C-007: T-006's exit-2 and stderr promises are untested, and round-1 C-003's "exit 2" fix is incomplete
- **Where:** `build-plan.md` T-006 EARS (declaration, fenced-code and binary clauses) and the T-001 and T-007 argument clauses; `test-plan.md` `cli_exit_codes`, `unsupported_declaration_is_an_error`, `fenced_code_is_ignored`, `policy_rejects_extra_arguments`, `ci_rejects_extra_arguments`, Intent Traceability "AC-2 (declaration)" row; `critique-round1.md` C-003 resolution
- **Quote:** "`cli_exit_codes`: clean Book → 0; violating Book → 1; missing directory, `--strict`, `-v`, two directories → 2 with nothing on stdout." / "`unsupported_declaration_is_an_error`: `typed-acceptance: 2` → error."
- **Failure mode:** plan-test-mismatch
- **Why it matters:** T-006 says "the binary **SHALL** exit 2" for any declaration other than `1`, and C-003's resolution claims "clauses and tests for wrapped typed claims and unknown declarations (exit 2)". Only the library error is tested, though, so the row's "unknown `typed-acceptance` value → error, exit 2" has no binary-level test. The stderr message promised by the exit-2 clauses of `cargo policy`, `cargo book` and `cargo ci` is asserted nowhere. The fence clause covers both fence styles anywhere in a chapter, but the test uses one fence inside the criteria section, so tilde fences and a fenced criteria heading outside the section are untested.
- **Suggested response:** fix-in-plan: add a `typed-acceptance: 2` Book to `cli_exit_codes`, expecting exit 2 and non-empty stderr. Assert non-empty stderr in every exit-2 case for `book`, `policy` and `ci`. Add a tilde fence, and a fenced criteria heading outside the section, to `fenced_code_is_ignored`.

### C-008: A test asserts "strength computed" with no clause, and the Book defines strength in two ways
- **Where:** `test-plan.md` `typed_criteria_parse_with_details`; `build-plan.md` T-006 (no strength clause); INT-0002 Intent (Grammar, Semantics); `docs/intents/README.md`
- **Quote:** "a wrapped typed claim with two details → joined claim, details kept, strength computed" / INT-0002: "A `formal` criterion supported only by its proxy counts as `empirical` until its obligation is kernel-checked" / README: "An intent's strength is its weakest satisfied criterion"
- **Failure mode:** intent-drift
- **Why it matters:** No EARS clause defines strength or gives an expected value, so the assertion has nothing to check against. No `formal` criterion in this Book is kernel-checked (Lean is not installed), so computing strength from modality alone would report `formal` where INT-0002 requires `empirical`, and README's "weakest satisfied criterion" is a third definition. INT-0002's grammar also says "`<n>` is a positive integer unique within the chapter and never reused". T-006's `duplicate` rule cannot detect reuse, and no plan text assigns it to review.
- **Suggested response:** fix-in-plan: either drop strength from Sprint 0's tests, or add a T-006 clause with expected values (a `formal` criterion without a kernel-checked obligation counts as `empirical`) and align README with INT-0002. Record in INT-0002 Consequences that "never reused" is enforced by review and Transition history until INT-0017's event records make it mechanical.

### C-009: INT-0020 still makes a criterion-less intent Certified, which contradicts revised INT-0002 and T-006
- **Where:** INT-0020 Intent (calculus block) and Transition history; INT-0002 Semantics; `build-plan.md` T-006 status clause; `critique-round1.md` C-003 resolution
- **Quote:** INT-0020: "Certified L c I := (∀ s ∈ I.necessary, ⟦s⟧) ∧ (∀ s ∈ I.supporting, ⟦s⟧)" / INT-0002: "An intent with no criteria is Undetermined, never vacuously Certified"
- **Failure mode:** intent-drift
- **Why it matters:** INT-0020 gives "the kernel semantics behind the Markdown grammar of" INT-0002, and its definition is vacuously true when both lists are empty. The C-003 revision changed only INT-0002 (INT-0020's Transition history has only its "created" entry), so two stable chapters now disagree on the status function T-006 implements. Phase 03 requires a changed outcome to be recorded in the stable intent before a plan reflects it.
- **Suggested response:** fix-in-plan: add a non-emptiness conjunct to INT-0020's Certified, for example `I.necessary ++ I.supporting ≠ []`. Append a "revised while `proposed`, after the Sprint 0 plan critique" Transition-history entry, as was done for INT-0008.

### C-010: `proptest` enters the workspace in T-004, but T-003's property tests need it first
- **Where:** `build-plan.md` T-003 Notes and last EARS clause, T-004 Notes and Acceptance criterion, T-006 "Depends on"; `test-plan.md` T-003 and T-006 unit tests
- **Quote:** T-004 Notes: "`proptest = { version = "1.12.0", default-features = false, features = ["std"] }` is a dev-dependency." / "`incremental_matches_one_shot` (property): random data and cut points → equal for both algorithms."
- **Failure mode:** hidden-dep
- **Why it matters:** T-003 adds only `blake3` and `sha2`, and its policy/deny clause names only those two. Yet `binary_round_trip`, `text_round_trip` and `incremental_matches_one_shot` are property tests, and T-004, which adds proptest to the workspace, `deny.toml` and prior-art, does not touch `crates/hash/Cargo.toml`. T-006's `status_is_consistent` (10,000 cases) also needs a generator, while T-006 declares only "Depends on: T-001"; running tasks in plan order hides this, but the declared dependencies are wrong.
- **Suggested response:** fix-in-plan: introduce proptest in T-003 (its notes, its policy/deny clause, its INT-0003 AC-3/AC-6 mapping and the dev-dependencies in `crates/hash/Cargo.toml`) and let T-004 reuse it. Either declare T-006 dependent on T-003, or state that its property test uses a std-only generator.

### C-011: T-005 relies on a kind registry that T-004 never creates
- **Where:** `build-plan.md` T-005 spec EARS clause and Touches; T-004 EARS and Notes
- **Quote:** "`defined` rows equal `ObjectKind::ALL`, and every `reserved` id is rejected with `ReservedObjectKind`" / "**Touches:** `crates/codec/tests/golden.rs`, `crates/codec/tests/golden/v1/*.bin`, `crates/codec/tests/spec.rs`"
- **Failure mode:** hidden-dep
- **Why it matters:** Every (id, kind, status) row, including the reserved `changeset`, `operation`, `view` and `event`, must equal "the code registry". That needs `ObjectKind::ALL` plus kind names and statuses in code, and no T-004 clause, note or test creates them. T-005 would therefore either edit `crates/codec/src/envelope.rs`, which is outside its Touches, or compare reserved rows by id only, which falls short of round-1 C-011's "full (id, name, status) rows including reserved ones".
- **Suggested response:** fix-in-plan: have T-004 define and test a public kind registry (`ObjectKind::ALL`, plus id, name and status for both defined and reserved kinds), or add `crates/codec/src/envelope.rs` to T-005 Touches.

### C-012: T-007's Touches imply an xtask dependency change, but not the root manifest it needs
- **Where:** `build-plan.md` T-007 Touches and Notes; T-001 `deps` clause; T-006 Notes
- **Quote:** "**Touches:** `crates/xtask/Cargo.toml`, `crates/xtask/src/main.rs`, `crates/xtask/src/ci.rs`, `.github/workflows/ci.yml`, `.github/dependabot.yml`, `Cargo.lock`" / "**WHEN** a member manifest declares a normal, dev or build dependency without `workspace = true`, **THEN** policy **SHALL** fail with rule `deps`"
- **Failure mode:** hidden-dep
- **Why it matters:** T-007's new checks are "std-only text checks in `xtask`", so the only reason to change `crates/xtask/Cargo.toml` and `Cargo.lock` is an xtask → `book` dependency for an in-process `book` step. The `deps` rule requires that as `book = { workspace = true }`, backed by a root `[workspace.dependencies]` entry that neither T-006 nor T-007 adds. The plan never says whether the `book` step runs in-process or as `cargo book`.
- **Suggested response:** fix-in-plan: state how the `book` step runs. If it runs in-process, add the root `Cargo.toml` to T-007 Touches, or have T-006 add the `book` path entry. If it shells out to `cargo book`, drop `crates/xtask/Cargo.toml` and `Cargo.lock` from T-007 Touches.

### C-013: T-004's tests are weaker than its clauses on per-type case counts and error offsets
- **Where:** `build-plan.md` T-004 EARS (varint, primitive, round-trip and mutation clauses); `test-plan.md` T-004 unit tests; INT-0006 AC-1 Proxy
- **Quote:** "`round_trip_all_types` (property, 10,000 cases): every supported type, including blob envelopes."
- **Failure mode:** plan-test-mismatch
- **Why it matters:** INT-0006 AC-1's proxy requires "at least 10,000 cases per encoded type", and T-004 promises "10,000 values of each supported type". The test is one property of 10,000 cases over 13 types, so a single union proptest gives each type about 770. The mutation property checks only mutants that still decode and sets no minimum, so types whose mutants rarely decode (bool, option tag, envelope header) are barely exercised. T-004 promises errors "at the varint's offset" and "carrying the byte offset", but `varint_rejects_non_canonical` asserts no offset and every invalid input starts at offset 0. An implementation reporting offsets relative to the value would pass, and the offset `InvalidUtf8` carries (value start or bad byte) is undefined.
- **Suggested response:** fix-in-plan: use one property (or `proptest_config`) per type at 10,000 or more cases each, with a minimum number of decoding mutants per type. Embed each invalid value at a non-zero position, for example as the second element of a `Vec<bool>`, and define which offset `InvalidUtf8` carries.

### C-014: The `PERI_GOLDEN_CREATE` clause describes code structure, and its mapped test cannot fail
- **Where:** `build-plan.md` T-005 EARS (third clause) and Notes; `test-plan.md` `golden_v1_corpus_matches`
- **Quote:** "**THEN** it **SHALL** read `PERI_GOLDEN_CREATE` once and pass create mode (`1` means on) to the harness as a parameter."
- **Failure mode:** EARS-vague
- **Why it matters:** "Read once" and "pass as a parameter" are not observable behaviour, and the only mapped test runs with the variable unset (T-005 Notes: "tests never set it"). It passes whether or not `1` is parsed correctly, whether any non-empty value enables create mode, and whether the harness reads the variable itself. Round-1 C-013's fix is present; this new clause, which that fix introduced, cannot be tested as written.
- **Suggested response:** fix-in-plan: restate it as a pure function, for example "`create_mode(value: Option<&str>)` SHALL be true exactly for `Some("1")`", and unit-test it with `1`, `0`, `true` and unset, which needs no `set_var`. Alternatively, exercise the variable through a child process with `Command::env`, or move the detail to Notes.

### C-015: The design-credit clause promises "every system", but its list omits systems the intents adopt
- **Where:** `build-plan.md` T-008 first EARS clause; `test-plan.md` `prior_art_credits_every_listed_system`; INT-0003 AC-5
- **Quote:** "**THEN** the file **SHALL** name every system the Book's intents adopt or evaluate"
- **Failure mode:** EARS-vague
- **Why it matters:** The test checks only the enumerated list. That list omits systems the intents adopt or cite as precedent: zstd (INT-0008), Agent Trace, in-toto/DSSE and SLSA (INT-0016), the Model Context Protocol (INT-0022), Scion (INT-0003, INT-0024, INT-0030) and Fossil (INT-0017). It also omits alternatives they evaluate: FastCDC, Deterministic CBOR, Protocol Buffers, Multihash, Loom, TLA+, cargo-vet, SQLite, LMDB, gitoxide, libgit2 and Sigstore. Meanwhile it includes JGit, Spork, Darcs and lean4export, which no intent names. The test can pass while the clause, and INT-0003 AC-5's "credits every adopted design", are false, and the owner's AC-5 review could take the passing test as proof the credits are complete.
- **Suggested response:** fix-in-plan: either reword the clause to "names at least the following systems" and leave completeness to the owner's AC-5 review, or extend the list with at least the adopted systems: zstd, Agent Trace, in-toto/DSSE, the Model Context Protocol, Scion, SLSA and Fossil.

### C-016: T-008 attributes a README duty to INT-0003 AC-9 that AC-9 does not contain, and checks only `cargo ci`
- **Where:** `build-plan.md` T-008 Acceptance criterion and README clause; `test-plan.md` INT-0003 AC-9 row and `readme_keeps_statement_and_links_book`
- **Quote:** "AC-9 (the README documents the flagless commands)" / "the owner's two original paragraphs **SHALL** be present verbatim, along with a link to `docs/SUMMARY.md` and the command `cargo ci`."
- **Failure mode:** plan-test-mismatch
- **Why it matters:** INT-0003 AC-9 covers exit-2 rejection of `-` arguments and idempotence across two runs, and places no documentation duty. The README clause checks only `cargo ci`, not `cargo policy`, `cargo book` or the no-flags rule, so listing `readme_keeps_statement_and_links_book` under AC-9 verifies nothing in AC-9. The verbatim reference text is not pinned: if the test reads it from the README that T-008 edits, it cannot fail. Also, README.md at `620f99c` ends its second paragraph with a trailing space ("emblazoned in the stone. "), so whitespace handling alone decides the result.
- **Suggested response:** fix-in-plan: drop AC-9 from T-008 and the README test from the AC-9 row. Alternatively, add the README duty to INT-0003 with a Transition-history entry and have the clause name all three commands. Embed the two paragraphs of README.md at `620f99c` in `docs.rs` as the reference.

### C-017: Orphan clauses and tests, and traceability rows that misattribute or overclaim
- **Where:** `test-plan.md` Intent Traceability; `build-plan.md` T-001 (`golden` clause and Intent links), T-006 (Intent and Acceptance criterion), T-007 (Dependabot clause)
- **Quote:** "**WHEN** `.gitattributes` does not mark `*.bin` files as binary, **THEN** policy **SHALL** fail with rule `golden`." / "**WHEN** the Dependabot test reads `.github/dependabot.yml`, **THEN** it **SHALL** find a `cargo` and a `github-actions` ecosystem, each with `target-branch: "dev"`."
- **Failure mode:** plan-test-mismatch
- **Why it matters:** Four tests appear in no traceability row:
  - `policy_requires_binary_golden_attr` is the line-ending mitigation for INT-0006 AC-3 and INT-0004 AC-1, yet T-001 links only INT-0003 and INT-0005.
  - `dependabot_targets_work_branch` has no matching acceptance criterion; no intent's criteria mention Dependabot.
  - `ci_failures_do_not_stop_later_steps` has no row.
  - `fixture_passes_every_rule` has no row and also traces to no clause, and neither does `int_disallowed_types_probe` (Clippy rejecting `HashMap`).

  Several rows misattribute or overclaim:
  - `policy_flags_direct_dependency` sits under AC-2 "pinned and locked", although the `deps` rule exists to make AC-6's `credits` rule complete (round-1 C-012).
  - The envelope tests sit only under INT-0006 AC-6, but they test decoder behaviour, not spec-before-code.
  - The AC-3 row claims deny "passes after each" of T-001, T-003 and T-004, but its only test is the end-of-sprint `int_deny_windows`.
  - The AC-9 row cites T-006, whose Acceptance criterion omits INT-0003 AC-9 even though its binary clause implements AC-9 for `cargo book`.
- **Suggested response:** fix-in-plan:
  - Link INT-0006 AC-3 and INT-0004 AC-1 from T-001 (or move the `golden` rule to T-005), and list its test in those rows.
  - Trace Dependabot to an acceptance criterion recorded in an intent, or move it to Notes.
  - Add a T-001 clause for the Clippy rejection, and table entries for the runner-continues and fixture tests.
  - Move `policy_flags_direct_dependency` to the AC-6 row and the envelope tests to INT-0006 AC-1.
  - Record `cargo deny check` output at each commit that adds a dependency, or drop "after each".
  - Add INT-0003 AC-9 to T-006.

### C-018: The `names` rule forbids the `peri-` package names INT-0003 requires for any published crate
- **Where:** INT-0003 Intent (Boundaries), AC-7 and AC-8; `build-plan.md` T-001 `names` clause; `test-plan.md` `policy_requires_single_word_names`
- **Quote:** INT-0003: "a crate that is ever published goes to crates.io under a `peri-` prefixed package name" / "packages `peri-hash`, `Hash`, `hash_ref`, `9hash` → FAIL `names`"
- **Failure mode:** intent-drift
- **Why it matters:** Cargo publishes a crate under its `[package]` name. A published workspace crate therefore needs a name that AC-8 ("Every workspace package name is a single lowercase ASCII word matching `[a-z][a-z0-9]*`") and T-001's rule reject, and INT-0003 does not say whether published crates leave the workspace or are exempt. This Plan phase's owner-directed revision introduced the contradiction, and the plan makes it a check that runs on every `cargo ci`, although nothing is published in Sprint 0.
- **Suggested response:** defer-with-rationale: record the publication path in INT-0003 with a Transition-history entry. Either AC-8 covers only `publish = false` packages and `names` accepts `peri-[a-z][a-z0-9]*` for publishable ones, or published crates move out of the workspace. T-001's rule can stay as written for Sprint 0.

### C-019: The research risk on prototype figures that cannot be re-run has no task, test or deferral
- **Where:** `research-report.md` §4 Risks (first bullet); `build-plan.md` Intents; no intent records it
- **Quote:** "Their numbers become thresholds in the intents, but they cannot be re-run until re-implemented."
- **Failure mode:** missing-risk
- **Why it matters:** Neither plan mentions this risk (the word "prototype" appears in neither) and no intent records the limitation, so it drops out of the record in the very first sprint. Intents still carry figures from the prototypes, for example INT-0005 AC-2 ("R2-E23 measured 0 over 2,600 scenarios") and AC-3's 84, 1,457 and 48. Every other research risk is covered: line endings by `.gitattributes` and the `golden` rule, zstd and Windows fsync by INT-0003, INT-0008, INT-0004 and INT-0007.
- **Suggested response:** defer-with-rationale: add one line to the build plan's Intents section, and a stable record such as INT-0001 Consequences, stating that figures quoted from the R1 to R5 prototypes are targets until the owning intent's sprint re-implements them and measures again.

### C-020: The Nighthawk "not yet possible" item names an unlock that cannot satisfy the criterion it cites
- **Where:** `test-plan.md` End-to-End Tests "Not yet possible"; INT-0004 AC-6; `research-report.md` §4 Unknown (Nighthawk's toolchain)
- **Quote:** "Not yet possible: an E2E run on the Debian host Nighthawk. Unlocked when the owner approves the Tailscale SSH check (INT-0004 AC-6)."
- **Failure mode:** e2e-drift
- **Why it matters:** INT-0004 AC-6 requires "End-to-end scenarios (multi-peer sync, crash and restart)" on Nighthawk. Those scenarios arrive only with INT-0007 and INT-0024, which research-report §1 puts out of scope, so owner approval alone unlocks nothing more than a third Linux `cargo ci` run. The item names neither an intent nor a sprint, as the schema requires, and the research report's open unknown (Nighthawk's toolchain) has no owner.
- **Suggested response:** fix-in-plan: split the item in two:
  - An optional Debian `cargo ci` run with its toolchain recorded, unlocked by the owner's Tailscale approval. Sprint 0 loses nothing without it, since WSL covers Linux.
  - INT-0004 AC-6 evidence, unlocked by the first sprint that plans INT-0007 or INT-0024 scenarios.

## Confidence
proceed-with-caveats

## Resolution in the revised plans

| Concern | Resolution |
|---|---|
| C-001 | INT-0003 AC-6 and the test use `cargo tree --workspace --target all -e normal,build -i cc`, which lists `blake3`. A T-003 clause requires the `cc` reverse tree to equal the `yes` rows, each naming its pure option. The `credits` rule requires a non-empty build cell for `yes` rows. |
| C-002 | GitHub Actions evidence and the owner's checkpoint sign-offs are carried forward explicitly. Loop queues a `(backlog)` task for Sprint 1, and INT-0003 records why. INT-0006 AC-3's "CI" means the canonical suite, locally and in the GitHub matrix. |
| C-003 | New T-007 clause: every step PASS and exit 0 on the Windows checkout, the WSL ext4 clone and the `core.autocrlf=true` clone, with OS and `rustc -Vv` recorded. The continue-on-failure test uses two failing steps and checks the exit status and summary. |
| C-004 | New `golden_changed_file_fails`. The create-mode clause now fails on a differing file. The runner's `test` step removes `PERI_GOLDEN_CREATE` (`test_step_removes_golden_create`). |
| C-005 | The vendored chapters are named, with exact counts (5 and 9) and a joined claim. The E2E total must equal an independent count (82 at `e8c3a58`). The CRLF fixture holds a wrapped criterion with a detail. A Book without a marker exits 2. |
| C-006 | The spec carries hand-derived worked-example hex for every golden value; `spec_examples_match_golden_files` checks every file against it. INT-0006 records that encodings freeze at the owner's AC-6 sign-off. |
| C-007 | `cli_exit_codes` covers `typed-acceptance: 2` and a missing marker, with non-empty stderr on every exit-2 case for `book`, `policy` and `ci`. The fence test covers tilde fences and a fenced criteria heading outside the section. |
| C-008 | Strength removed from Sprint 0; README aligned with INT-0002; INT-0002 Consequences record how "never reused" is enforced. |
| C-009 | INT-0020's `Certified` requires a non-empty spec list, with a Transition-history entry. |
| C-010 | `proptest` enters in T-003 (hash), and T-004 and T-006 reuse it; T-006 depends on T-003. |
| C-011 | T-004 defines and tests the public kind registry (`ObjectKind::ALL`, `RESERVED_KINDS` with names) in `kind_registry_is_complete`. |
| C-012 | The `book` step runs in-process; T-006 adds the root `book` path entry, and T-007 adds `book.workspace = true` to xtask. |
| C-013 | One round-trip property and one mutation property per type, 10,000 cases each. New exhaustive single-byte mutant test with at least one decoding mutant per type. Invalid values are embedded at offset 2, and `InvalidUtf8`'s offset is defined. |
| C-014 | `create_mode` is a pure function tested on `1`, `0`, `true`, empty and unset. |
| C-015 | The clause now says "names at least", the list adds zstd, Agent Trace, in-toto, DSSE, SLSA, Model Context Protocol, Scion and Fossil, and completeness is left to the owner's AC-5 review. |
| C-016 | AC-9 dropped from T-008. New INT-0003 AC-10 covers the README: all four commands and the no-flags statement, with the owner's paragraphs embedded from `620f99c`. |
| C-017 | The `golden` rule moves to T-005 and traces to INT-0006 AC-3 and INT-0004 AC-1. Dependabot moves to Notes. New Clippy-rejection clause for the probe. The runner-continues and fixture tests get rows. `deps` maps to AC-6, the envelope tests to AC-1, deny is recorded at T-001 and T-003, and AC-9 is added to T-006. |
| C-018 | Deferred with rationale: AC-8 covers unpublished packages, and INT-0003 Consequences record the `peri-` path for a future publishable crate. |
| C-019 | INT-0001 Consequences and the build plan's Intents section record that quoted prototype figures are targets until re-measured. |
| C-020 | Split into an optional Debian `cargo ci` run (Tailscale approval) and INT-0004 AC-6, unlocked by the first sprint that plans INT-0007 or INT-0024 scenarios. |
