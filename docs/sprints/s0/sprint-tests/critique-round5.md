# Test Critique — Sprint 0, round 5

## Concerns

### C-001: Text form of a HashRef has no known-answer check
- **Where:** `build-plan.md` T-003 EARS (text form) / `unit-tests.md` T-003 text-form row / `crates/hash/tests/vectors.rs` `text_round_trip`, `text_rejects_malformed` / `docs/spec/format-v1.md` Hash algorithms, Text form / `test-plan.md` Intent Traceability, INT-0006 AC-2 row
- **Quote:** "a `HashRef` is rendered as text, **THEN** it **SHALL** read `<name>:<lowercase hex>` and parse back to itself"
- **Failure mode:** weak-assertion
- **Why it matters:** No test compares a rendering with a fixed string. `text_round_trip` checks only the `<name>:` prefix and the round trip, `text_rejects_malformed` builds its cases from the renderer under test, and the golden files pin only the binary form. On a scratch clone of `24d01f5`, a renderer that reverses the digest bytes, paired with a parser that reverses them back, passes all 151 tests and a full `cargo ci`. It renders BLAKE3 of empty input as `blake3:62321fe4…13af` instead of the spec's `blake3:af1349b9…3262`, and a nibble-swapping mutant passes too, so the T-003 row's pass would hold just as well for a text form that contradicts `format-v1.md`.
- **Suggested response:** add-test — assert that `HashAlg::Blake3.hash(b"").to_string()` equals the spec's example and `HashAlg::Sha256.hash(b"abc").to_string()` equals `sha256:ba7816bf…15ad`, and that each parses back (needs a new tested head, 152 tests)

### C-002: InvalidUtf8 offset fixture cannot tell the specified offset from alternatives
- **Where:** `crates/codec/tests/properties.rs` `decode_rejects_invalid_primitives` (line 269) / `build-plan.md` T-004 EARS (invalid bool, option tag, UTF-8) / `unit-tests.md` T-004 row "(UTF-8 at the first content byte)" / `docs/spec/format-v1.md` Decoder requirements, Located / plan `critique-round2.md` C-013
- **Quote:** "`from_slice::<Vec<String>>(&second_element(0x00, &[0x01, 0xff]))`"
- **Failure mode:** weak-assertion
- **Why it matters:** The clause puts `InvalidUtf8` at the string's first content byte, after its length prefix. The only fixture has a one-byte prefix and an invalid first byte, so the first content byte, the first invalid byte and the prefix offset plus 1 are all 3. At `24d01f5`, a decoder that reports the first invalid byte (`at + valid_up_to()`) and one that reports the prefix offset plus 1 both pass all 151 tests, fmt and clippy, and the first also passes a full `cargo ci`. Yet they give 4 for `02 00 02 61 ff` and 3 for a two-byte prefix, where the spec requires 3 and 4. Plan critique round 2's C-013 named exactly this "bad byte" alternative, and the recorded pass would hold for it just as well.
- **Suggested response:** tighten-assertion — add `02 00 02 61 ff` → `InvalidUtf8 { at: 3 }` and `02 00 80 01` followed by 128 × `ff` → `InvalidUtf8 { at: 4 }` to `decode_rejects_invalid_primitives` (needs a new tested head)

### C-003: Workflow test accepts a deny job that skips licences and advisories
- **Where:** `crates/xtask/src/workflow.rs` `runs_deny_on_ubuntu` (line 121), `runs_deny_check`, `workflow_forms_read_alike` / `crates/xtask/src/ci.rs` `workflow_matches_ci_contract` / `build-plan.md` T-007 EARS (workflow clause) and Acceptance criterion ("AC-3 (the deny job in the workflow)") / `test-plan.md` Intent Traceability, INT-0003 AC-3 row / `unit-tests.md` T-007 workflow row / `INT-0003` AC-3 and Consequences
- **Quote:** "`.is_none_or(|command| value_of(command).starts_with("check"))`"
- **Failure mode:** weak-assertion
- **Why it matters:** T-007 claims INT-0003 AC-3 through the deny job, and AC-3 (`necessary`) requires licences, advisories and bans to be checked in CI. `workflow_matches_ci_contract` is Sprint 0's only check of what that job runs. It accepts any action `command` that starts with `check`, ignores `command-arguments`, and accepts any `cargo deny` line that contains `check`. At `24d01f5`, four narrowed forms pass the workflow tests: `command: check bans`, the action's own recommended `command: check bans licenses sources`, `command-arguments: advisories` and `run: cargo deny check sources`. Only `command: fetch` fails. So a narrowed job would give T-009 a green deny job that never checked licences or advisories. Nothing recorded is false, because `ci.yml`'s deny step has no `with:` and runs the full default `check`.
- **Suggested response:** tighten-assertion — accept only a missing or exactly-`check` `command`, reject check names in `command-arguments`/`arguments` and after `check` on a `run:` line, add those narrowed forms as mutants that must fail, and drop `check bans` from `workflow_forms_read_alike` (needs a new tested head)

### C-004: Unit record still describes the superseded three-failure probe
- **Where:** `unit-tests.md` T-007 row "two failing steps, one a missing tool" / `integration-tests.md` `int_ci_reports_failures` and `int_ci_reports_book_violation` / `test-matrix-log.md` both legs / `critique-round3.md` closing paragraph
- **Quote:** "a real `cargo ci` with three failing steps, `book` among them, prints each and exits 1"
- **Failure mode:** evidence-drift
- **Why it matters:** At `24d01f5`, both legs log `PASS book` and `ci: failed: fmt, test` for `int_ci_reports_failures`. Round-4 C-001 moved the Book violation to `int_ci_reports_book_violation` (`ci: failed: test, book`), which no `unit-tests.md` row cites. Commit `a6352e7` left this row as it was written for `6a85e66`, and the round-3 closing paragraph still says the probe "must print `FAIL book`". A report built from the row would describe a run that the log contradicts. No clause loses proof: the probe still shows two named failing steps and exit 1, and `book_step_fails_on_a_violation` covers the Book path.
- **Suggested response:** tighten-assertion — describe `int_ci_reports_failures` as fmt and test failing with book passing (`ci: failed: fmt, test`, exit 1), cite `int_ci_reports_book_violation` (`ci: failed: test, book`, exit 1) in the same row, and mark the round-3 closing paragraph as superseded by round-4 C-001

### C-005: Deny test's negatives cannot tell an exact match from a prefix match
- **Where:** `crates/xtask/src/deny.rs` `incompatible` (line 69) and `incompatible_licences_are_named` / `critique-round4.md` Responses, C-002 / `unit-tests.md` T-001 deny row
- **Quote:** "`incompatible_licences_are_named` shows that `SSPL-1.0`, `BUSL-1.1` and `OpenSSL` are named"
- **Failure mode:** weak-assertion
- **Why it matters:** No negative case starts with or contains an allowed identifier, so replacing the exact `GPL3_COMPATIBLE.contains(&licence)` with a `starts_with` or `contains` match still passes both deny tests. Under the prefix mutant, a `deny.toml` that allows `MPL-2.0-no-copyleft-exception`, which the FSF treats as GPL-incompatible, passes both tests and `cargo deny check licenses`. Nothing recorded is false: the match is exact at `24d01f5`, and all nine allowed identifiers appear verbatim on the list.
- **Suggested response:** tighten-assertion — add a near-miss negative such as `MPL-2.0-no-copyleft-exception` to `incompatible_licences_are_named` in the next tested head

### C-006: T-010 as scoped leaves the test-binary half of the shared-target limit open
- **Where:** `e2e-tests.md` Carried forward, T-010 bullet / `code-review.md` known-limit row on xtask's `workspace_root()` and tests that use `env!("CARGO_MANIFEST_DIR")` / `critique-round3.md` Responses, C-005
- **Quote:** "so that a target directory shared between checkouts cannot make `cargo ci` or `cargo policy` check the wrong one"
- **Failure mode:** flake-risk
- **Why it matters:** The known limit has two halves: xtask's compiled-in root, and test binaries whose compiled-in `CARGO_MANIFEST_DIR` makes them read the other checkout's files. T-010 only makes xtask refuse to run. With two scratch clones of `24d01f5` sharing one target directory, touching xtask in the second clone rebuilt only xtask, with that clone's path, so a T-010 check would pass. Its `cargo ci` then reused the first clone's golden test binary and printed `ci: all 5 steps passed` despite a flipped golden byte. No recorded run is affected, since each one used its own target directory.
- **Suggested response:** defer-with-rationale — either widen T-010 so tests that read workspace files use or check the runtime `CARGO_MANIFEST_DIR`, or narrow its purpose clause to what xtask controls and name reused test binaries as an open limit, then align the code-review row and the round-3 C-005 response with that choice

### C-007: `[graph] exclude` lets any licence past cargo-deny and the deny test
- **Where:** `crates/xtask/src/deny.rs` module doc and `incompatible` / `critique-round4.md` Responses, C-002 / `integration-tests.md` `int_deny_windows` / `INT-0003` AC-3
- **Quote:** "proves that every dependency's licence is on the policy's `allow` lists"
- **Failure mode:** weak-assertion
- **Why it matters:** cargo-deny 0.20.2 honours `[graph] exclude`, which removes a crate from every check. With winnow's licence clarified to `SSPL-1.0`, adding `exclude = ["winnow"]` turned `licenses FAILED` into `licenses ok`, and both deny tests still passed, so the AC-3 evidence the records cite could vouch for an incompatible dependency. Nothing is false at `24d01f5`, whose `deny.toml` has no `exclude`.
- **Suggested response:** reject (the critique is wrong because no Sprint 0 AC or EARS clause asks for a guard against a `deny.toml` edit that narrows cargo-deny's graph, and failing on `[graph] exclude` would not close the hole, since a `[[licenses.clarify]]` entry passes the same way) — add no test; optionally reword the module doc so it claims only non-dev dependencies, under a `deny.toml` that excludes and clarifies no crate

## Confidence
block

## Responses

Round 5 reviewed the records at `a6352e7`, which describe the tested head `24d01f5`. Each concern is addressed in the head that the final critique (`critique.md`) reviews. The code fixes are in `94f7363`; mutating each fix makes a new or tightened test fail, including the critique's own reversed-digest and nibble-swap mutants.

| Concern | Response |
|---------|----------|
| C-001 | add-test in `94f7363`. `text_form_matches_the_spec` compares the text form with fixed strings both ways, rendering and parsing: the spec's BLAKE3 example and SHA-256 of "abc" (FIPS 180-2). A renderer and parser that agree on reversed digests, or on swapped nibbles, fail it. |
| C-002 | tighten-assertion in `94f7363`. `decode_rejects_invalid_primitives` adds `02 00 02 61 ff`, which must give `InvalidUtf8 { at: 3 }` although the first invalid byte is at 4, and a second string with a two-byte length prefix, which must give `at: 4` although one past the prefix's start is 3. Decoders that report either alternative fail it. |
| C-003 | tighten-assertion in `94f7363`. The deny job must now run every check on this workspace: the action with `command` exactly `check`, `arguments` exactly `--all-features` and no other input but `log-level`, or a plain `cargo deny check` with at most `--all-features` and `--locked`. `workflow_matches_ci_contract` adds eleven narrowed deny steps that must not count (named checks, `command-arguments`, `arguments`, `manifest-path`, `--manifest-path`, `--config`, `--offline`, a repeated `check`) and three full forms that must, and `workflow_forms_read_alike` no longer presents `check bans` as a valid form. Mutants that accept any command starting with `check`, any other action input or any word after `cargo deny` fail it. |
| C-004 | tighten-assertion. The T-007 row now describes `int_ci_reports_failures` as fmt and test failing with book passing (`ci: failed: fmt, test`), and cites `int_ci_reports_book_violation` (`ci: failed: test, book`) beside it. The closing paragraph of `critique-round3.md` is marked as superseded by round 4's C-001. |
| C-005 | tighten-assertion in `94f7363`. `incompatible_licences_are_named` adds `MPL-2.0-no-copyleft-exception`, which starts with an allowed identifier, so a prefix or substring match fails it. |
| C-006 | defer-with-rationale, with T-010 widened to both halves of the limit: xtask, and the tests that read workspace files through `CARGO_MANIFEST_DIR`, refuse to run for a checkout other than the one they were built for. The code-review row, the Carried forward section and the round-3 C-005 response now say the same. (Redefined after the round-6 critique, C-008, and narrowed after round 7, C-007: a checkout's own `cargo ci` refuses a target directory outside its workspace root, and xtask refuses to run for a checkout it was not built for. The check for tests was dropped; only one target directory per checkout keeps another checkout's builds out of this one's `target/`.) |
| C-007 | rejected, as the critique proposes, but closed anyway in `94f7363`. The `deny` module's doc now claims only what holds, and `deny_allows_only_gpl3_compatible_licences` also fails when `deny.toml` hides crates from cargo-deny through `[graph] exclude` or `[licenses] clarify`; `settings_that_hide_licences_are_named` is its negative test. Adding either setting therefore means revisiting the check. |
