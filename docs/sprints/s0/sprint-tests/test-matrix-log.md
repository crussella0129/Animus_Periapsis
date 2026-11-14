# Sprint 0 Test Matrix Log

The raw output of the final test-matrix run at `c342412`, produced by the scripts reproduced at the end of this page: `test_matrix.sh`, its WSL leg `wsl_ci.sh`, and `port_corpus.py`. [integration-tests.md](integration-tests.md) and [e2e-tests.md](e2e-tests.md) summarize it.

```text

==== host (Windows)
Microsoft Windows 11 Pro 10.0.26300
rustc 1.99.1 (58cf8f9c3 2026-10-13)
binary: rustc
commit-hash: 58cf8f9c35b355199727627d07ae34a4e2846d67
commit-date: 2026-10-13
host: x86_64-pc-windows-msvc
release: 1.99.1
LLVM version: 23.1.1
cargo-deny 0.20.2
git version 2.56.0.windows.3
PROPTEST_*, PERI_*, CLIPPY_CONF_DIR, GIT_*, RUSTFLAGS, RUSTDOCFLAGS, RUSTC*, RUSTDOC, RUSTUP_TOOLCHAIN, CARGO_BUILD_*, CARGO_TARGET_*, CARGO_PROFILE_*, CARGO_ENCODED_* or CARGO_ALIAS_* set: GIT_EDITOR 
head c342412a2951c70d7db68608f09e300e55e745be on dev; uncommitted paths: 0
target directory: C:\Users\charl\Animus_Periapsis\target
cargo config files above the working checkout or in CARGO_HOME: none

==== int_ci_windows (working checkout, NTFS)
PASS fmt
PASS clippy
PASS test
PASS policy
PASS book
ci: all 5 steps passed
cargo ci exit 0

==== int_deny_windows
advisories ok, bans ok, licenses ok, sources ok
cargo deny exit 0
advisory db advisory-db-3157b0e258782691: 26fd2d43f7280a1fb925a3743ec463017afe7119 2026-11-05

==== unit listing (Windows working checkout)
     Running unittests src\lib.rs
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src\main.rs
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests\book.rs
test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.83s
     Running tests\cli.rs
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.51s
     Running unittests src\lib.rs
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests\alloc.rs
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests\golden.rs
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running tests\mutants.rs
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests\properties.rs
test result: ok. 41 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.54s
     Running tests\spec.rs
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src\lib.rs
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests\vectors.rs
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.34s
     Running unittests src\main.rs
test result: ok. 59 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 19.77s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
tests passed: 171

==== int_cc_build_dependency_audit
cc v1.10.0
[build-dependencies]
└── blake3 v1.8.7
blake3 v1.8.7
├── blake3 feature "pure"
└── blake3 feature "std"
C-build yes rows:
  `blake3` 

==== int_disallowed_types_probe (two scratch clones, discarded)
-- planted std::collections::HashMap at c342412:  1 file changed, 5 insertions(+)
target directory: C:\Users\charl\AppData\Local\Temp\claude\C--Users-charl--claude-spares-spare-61ekw4\5eb9caae-ac3b-41b6-a91c-2cad206f0660\scratchpad\probe\target
cargo config files above the clone or in CARGO_HOME: none
error: use of a disallowed type `std::collections::HashMap`
clippy exit 101 (non-zero required)
scratch clone removed: yes
-- planted std::collections::HashSet at c342412:  1 file changed, 5 insertions(+)
target directory: C:\Users\charl\AppData\Local\Temp\claude\C--Users-charl--claude-spares-spare-61ekw4\5eb9caae-ac3b-41b6-a91c-2cad206f0660\scratchpad\probe\target
cargo config files above the clone or in CARGO_HOME: none
error: use of a disallowed type `std::collections::HashSet`
clippy exit 101 (non-zero required)
scratch clone removed: yes

==== int_ci_reports_failures (Windows scratch clone: one golden byte changed, one fmt violation)
changed:  M crates/codec/src/lib.rs  M crates/codec/tests/golden/v1/string-ascii.bin 
target directory: C:\Users\charl\AppData\Local\Temp\claude\C--Users-charl--claude-spares-spare-61ekw4\5eb9caae-ac3b-41b6-a91c-2cad206f0660\scratchpad\probe\target
cargo config files above the clone or in CARGO_HOME: none
FAIL fmt: cargo fmt --all --check exited with exit code: 1
PASS clippy
FAIL test: cargo test --workspace --locked exited with exit code: 101
PASS policy
PASS book
ci: failed: fmt, test
cargo ci exit 1 (1 required)
test golden_v1_corpus_matches ... FAILED
string-ascii.bin: encoding changed (file 10 bytes, encoder 10 bytes)
scratch clone removed: yes

==== int_ci_reports_book_violation (Windows scratch clone: one untyped criterion)
changed:  M docs/intents/INT-0031-visual-identity.md 
target directory: C:\Users\charl\AppData\Local\Temp\claude\C--Users-charl--claude-spares-spare-61ekw4\5eb9caae-ac3b-41b6-a91c-2cad206f0660\scratchpad\probe\target
cargo config files above the clone or in CARGO_HOME: none
PASS fmt
PASS clippy
FAIL test: cargo test --workspace --locked exited with exit code: 101
PASS policy
FAIL book: 1 Book violations; run `cargo book`
ci: failed: test, book
cargo ci exit 1 (1 required)
test book_intents_pass_strict ... FAILED
C:\Users\charl\AppData\Local\Temp\claude\C--Users-charl--claude-spares-spare-61ekw4\5eb9caae-ac3b-41b6-a91c-2cad206f0660\scratchpad\probe\crates\book\../../docs\intents\INT-0031-visual-identity.md:25: untyped: criterion lacks `**AC-n**` `polarity` `modality`
scratch clone removed: yes

==== int_ci_ignores_hostile_environment (Windows scratch clone: HashMap planted, a golden file deleted)
changed:  M crates/codec/src/lib.rs  D crates/codec/tests/golden/v1/uvarint-0.bin 
target directory: C:\Users\charl\AppData\Local\Temp\claude\C--Users-charl--claude-spares-spare-61ekw4\5eb9caae-ac3b-41b6-a91c-2cad206f0660\scratchpad\probe\target
cargo config files above the clone or in CARGO_HOME: none
PASS fmt
FAIL clippy: cargo clippy --workspace --all-targets --locked -- -D warnings exited with exit code: 101
FAIL test: cargo test --workspace --locked exited with exit code: 101
PASS policy
PASS book
ci: failed: clippy, test
cargo ci exit 1 (1 required)
error: use of a disallowed type `std::collections::HashMap`
test case_count_is_not_overridden ... ok
test golden_v1_corpus_matches ... FAILED
uvarint-0.bin recreated: no
scratch clone removed: yes

==== int_policy_reports_failures (Windows scratch clone: .gitattributes emptied, so nothing marks *.bin binary)
changed:  M .gitattributes 
target directory: C:\Users\charl\AppData\Local\Temp\claude\C--Users-charl--claude-spares-spare-61ekw4\5eb9caae-ac3b-41b6-a91c-2cad206f0660\scratchpad\probe\target
cargo config files above the clone or in CARGO_HOME: none
-- the caller's environment as is
PASS lints
PASS unsafe
PASS toolchain
PASS lockfile
PASS publish
PASS deps
PASS credits
PASS clippy
PASS names
PASS spec
FAIL golden: 24 *.bin paths are not binary: the .gitattributes files give crates/codec/tests/golden/v1/blob-envelope.bin `text: unspecified`
cargo policy exit 1 (1 required)
-- Git variables that point core.attributesFile at a file marking *.bin binary
control, plain git check-attr: crates/codec/tests/golden/v1/uvarint-0.bin: text: unspecified
control, with the variables:   crates/codec/tests/golden/v1/uvarint-0.bin: text: unset
PASS lints
PASS unsafe
PASS toolchain
PASS lockfile
PASS publish
PASS deps
PASS credits
PASS clippy
PASS names
PASS spec
FAIL golden: 24 *.bin paths are not binary: the .gitattributes files give crates/codec/tests/golden/v1/blob-envelope.bin `text: unspecified`
cargo policy exit 1 (1 required)
scratch clone removed: yes

==== int_aliases_refuse_flags_and_stale_locks (Windows scratch clone: aliases with flags, then a stale Cargo.lock)
target directory: C:\Users\charl\AppData\Local\Temp\claude\C--Users-charl--claude-spares-spare-61ekw4\5eb9caae-ac3b-41b6-a91c-2cad206f0660\scratchpad\probe\target
cargo config files above the clone or in CARGO_HOME: none
cargo ci --fast exit 2 (2 required)
cargo policy --fix exit 2 (2 required)
cargo book --strict exit 2 (2 required)
changed:  M Cargo.toml 
error: cannot update the lock file C:\Users\charl\AppData\Local\Temp\claude\C--Users-charl--claude-spares-spare-61ekw4\5eb9caae-ac3b-41b6-a91c-2cad206f0660\scratchpad\probe\Cargo.lock because --locked was passed to prevent this
cargo ci exit 101 (101 required); Cargo.lock changed: 0
scratch clone removed: yes

==== e2e_fresh_clone_windows (core.autocrlf=true)
clone head c342412a2951c70d7db68608f09e300e55e745be, core.autocrlf=true
target directory: C:\Users\charl\AppData\Local\Temp\claude\C--Users-charl--claude-spares-spare-61ekw4\5eb9caae-ac3b-41b6-a91c-2cad206f0660\scratchpad\e2e-autocrlf\target
cargo config files above the clone or in CARGO_HOME: none
golden: all 23 .bin files byte-identical to the original
tracked files: 132; with CRLF or mixed line ends in the working tree: 0
PASS fmt
PASS clippy
PASS test
PASS policy
PASS book
ci: all 5 steps passed
cargo ci exit 0 (cold build, 59 s)

==== e2e_port_rule_sprint_loops_book
Animus_Sprint_Loops head e8c3a58018e1627a145759fdac1f089c6236ba36, uncommitted paths: 0
INT files: 13
awk INT-0001-project-book.md: 5
awk INT-0002-substrate-and-branch-model.md: 6
awk INT-0003-dependency-updates-on-work.md: 7
awk INT-0004-substrate-contract-versioning.md: 7
awk INT-0005-turn-and-checkpoint-contract.md: 6
awk INT-0006-provider-reach-and-ci-truth.md: 11
awk INT-0007-integrity-sweep.md: 5
awk INT-0008-escalation-and-flow.md: 5
awk INT-0009-sprint-identity.md: 5
awk INT-0010-rust-portability-study.md: 5
awk INT-0011-human-in-the-loop-compatibility.md: 5
awk INT-0012-ci-scaffolding-lifecycle.md: 6
awk INT-0013-verification-integrity.md: 9
independent count (awk): 82
book: C:/Users/charl/Animus_Sprint_Loops/docs (undeclared, port rule)
  INT-0001-project-book.md: 5 criteria
  INT-0002-substrate-and-branch-model.md: 6 criteria
  INT-0003-dependency-updates-on-work.md: 7 criteria
  INT-0004-substrate-contract-versioning.md: 7 criteria
  INT-0005-turn-and-checkpoint-contract.md: 6 criteria
  INT-0006-provider-reach-and-ci-truth.md: 11 criteria
  INT-0007-integrity-sweep.md: 5 criteria
  INT-0008-escalation-and-flow.md: 5 criteria
  INT-0009-sprint-identity.md: 5 criteria
  INT-0010-rust-portability-study.md: 5 criteria
  INT-0011-human-in-the-loop-compatibility.md: 5 criteria
  INT-0012-ci-scaffolding-lifecycle.md: 6 criteria
  INT-0013-verification-integrity.md: 9 criteria
13 chapters, 82 criteria, 0 violations
cargo book exit 0

==== e2e_port_rule_local_books (every Book v2 on this host vs markdown-it-py)
markdown-it-py 4.0.0
| Book | Rule | Chapters | `cargo book` | CommonMark | Violations | Exit | Mismatches |
|---|---|---|---|---|---|---|---|
| Animus_Amalgam | port | 7 | 39 | 39 | 0 | 0 | none |
| Animus_Ferric | port | 14 | 112 | 112 | 0 | 0 | none |
| Animus_Periapsis | strict | 31 | 164 | 164 | 0 | 0 | none |
| Animus_Sprint_Loops | port | 13 | 82 | 82 | 0 | 0 | none |
| Animus_Unitarity | port | 1 | 7 | 7 | 0 | 0 | none |
| CubiKan | port | 15 | 136 | 136 | 0 | 0 | none |
| Feel_of_the_Wheel | port | 13 | 77 | 77 | 0 | 0 | none |
| Gnash_Equilibrium | port | 74 | 627 | 627 | 0 | 0 | none |
| Kinesin | port | 33 | 159 | 159 | 0 | 0 | none |
| Leibniz | port | 1 | 7 | 7 | 0 | 0 | none |
| Lovers_Lagoon | port | 7 | 43 | 43 | 0 | 0 | none |
| MDeezl-hammer | port | 6 | 63 | 63 | 0 | 0 | none |
| MDeezl | port | 6 | 63 | 63 | 0 | 0 | none |
| Project_Lagoon | port | 8 | 64 | 64 | 0 | 0 | none |
| Thread-Signal-Website | port | 2 | 11 | 11 | 0 | 0 | none |
| Tome | port | 25 | 106 | 106 | 0 | 0 | none |
| LEXPLOIT | port | 6 | 41 | 41 | 0 | 0 | none |
| beamsplitter | port | 3 | 16 | 16 | 0 | 0 | none |
| contrapunctus | port | 3 | 15 | 15 | 0 | 0 | none |
| diver | port | 26 | 147 | 147 | 0 | 0 | none |
| oxidizer | port | 5 | 31 | 31 | 0 | 0 | none |
| sdr.rs | port | 17 | 83 | 83 | 0 | 0 | none |
| sprint-loops | port | 13 | 82 | 82 | 0 | 0 | none |
| yarduluv | port | 31 | 478 | 478 | 0 | 0 | none |
port_corpus exit 0
  Animus_Amalgam: 7a6786b3e1, docs/intents uncommitted: 0
  Animus_Ferric: 508edfd, docs/intents uncommitted: 0
  Animus_Periapsis: c342412, docs/intents uncommitted: 0
  Animus_Sprint_Loops: e8c3a58, docs/intents uncommitted: 0
  Animus_Unitarity: 4f82372, docs/intents uncommitted: 0
  CubiKan: 77642b4, docs/intents uncommitted: 0
  Feel_of_the_Wheel: 804a1dd, docs/intents uncommitted: 2
  Gnash_Equilibrium: 10c269c, docs/intents uncommitted: 0
  Kinesin: 6fc2710, docs/intents uncommitted: 0
  Leibniz: 3660b96, docs/intents uncommitted: 0
  Lovers_Lagoon: 850c7cd, docs/intents uncommitted: 0
  MDeezl-hammer: 869ebfe, docs/intents uncommitted: 0
  MDeezl: de66382, docs/intents uncommitted: 0
  Project_Lagoon: 0b617e3, docs/intents uncommitted: 0
  Thread-Signal-Website: 7b4bf53, docs/intents uncommitted: 0
  Tome: df10abd, docs/intents uncommitted: 0
  LEXPLOIT: bd7af52, docs/intents uncommitted: 0
  beamsplitter: d1580d0, docs/intents uncommitted: 0
  contrapunctus: 1a9b00b, docs/intents uncommitted: 0
  diver: 2ae10f9, docs/intents uncommitted: 0
  oxidizer: d6555c3, docs/intents uncommitted: 0
  sdr.rs: ca127fe, docs/intents uncommitted: 0
  sprint-loops: 0bdbe66, docs/intents uncommitted: 0
  yarduluv: 08922c5, docs/intents uncommitted: 0

==== WSL leg (int_ci_wsl, int_tooling_is_idempotent, int_ci_reports_failures, int_ci_reports_book_violation, int_policy_reports_failures, int_disallowed_types_probe, int_aliases_refuse_flags_and_stale_locks)
== host
Linux 6.6.114.1-microsoft-standard-WSL2 x86_64
PRETTY_NAME="Ubuntu 26.04.1 LTS"
git version 2.53.0
PROPTEST_*, PERI_*, CLIPPY_CONF_DIR, GIT_*, RUSTFLAGS, RUSTDOCFLAGS, RUSTC*, RUSTDOC, RUSTUP_TOOLCHAIN, CARGO_BUILD_*, CARGO_TARGET_*, CARGO_PROFILE_*, CARGO_ENCODED_* or CARGO_ALIAS_* set: none
== clone
head c342412a2951c70d7db68608f09e300e55e745be
checkout on ext4
rustc 1.99.1 (58cf8f9c3 2026-10-13)
binary: rustc
commit-hash: 58cf8f9c35b355199727627d07ae34a4e2846d67
commit-date: 2026-10-13
host: x86_64-unknown-linux-gnu
release: 1.99.1
LLVM version: 23.1.1
target directory: /home/charles/periapsis-s0-test/target
cargo config files above the clone or in CARGO_HOME: none
== int_ci_wsl: cargo ci
PASS fmt
PASS clippy
PASS test
PASS policy
PASS book
ci: all 5 steps passed
cargo ci exit 0
tests passed on Linux: 171
== int_tooling_is_idempotent
run 1 cargo ci: exit 0, dirty files 0
run 1 cargo policy: exit 0, dirty files 0
run 1 cargo book: exit 0, dirty files 0, counted chapter lines 31
run 2 cargo ci: exit 0, dirty files 0
run 2 cargo policy: exit 0, dirty files 0
run 2 cargo book: exit 0, dirty files 0, counted chapter lines 31
chapters in docs/intents: 31
== int_ci_reports_failures: one golden byte changed, one fmt violation
changed:  M crates/codec/src/lib.rs  M crates/codec/tests/golden/v1/string-ascii.bin 
target directory: /home/charles/periapsis-s0-probe/target
cargo config files above the clone or in CARGO_HOME: none
FAIL fmt: cargo fmt --all --check exited with exit status: 1
PASS clippy
FAIL test: cargo test --workspace --locked exited with exit status: 101
PASS policy
PASS book
ci: failed: fmt, test
cargo ci exit 1 (1 required)
test golden_v1_corpus_matches ... FAILED
string-ascii.bin: encoding changed (file 10 bytes, encoder 10 bytes)
== int_ci_reports_book_violation: one untyped criterion
changed:  M docs/intents/INT-0031-visual-identity.md 
PASS fmt
PASS clippy
FAIL test: cargo test --workspace --locked exited with exit status: 101
PASS policy
FAIL book: 1 Book violations; run `cargo book`
ci: failed: test, book
cargo ci exit 1 (1 required)
test book_intents_pass_strict ... FAILED
/home/charles/periapsis-s0-probe/crates/book/../../docs/intents/INT-0031-visual-identity.md:25: untyped: criterion lacks `**AC-n**` `polarity` `modality`
== int_policy_reports_failures: .gitattributes emptied, so nothing marks *.bin binary
changed:  M .gitattributes 
-- the caller's environment as is
PASS lints
PASS unsafe
PASS toolchain
PASS lockfile
PASS publish
PASS deps
PASS credits
PASS clippy
PASS names
PASS spec
FAIL golden: 24 *.bin paths are not binary: the .gitattributes files give crates/codec/tests/golden/v1/blob-envelope.bin `text: unspecified`
cargo policy exit 1 (1 required)
-- Git variables that point core.attributesFile at a file marking *.bin binary
control, plain git check-attr: crates/codec/tests/golden/v1/uvarint-0.bin: text: unspecified
control, with the variables:   crates/codec/tests/golden/v1/uvarint-0.bin: text: unset
PASS lints
PASS unsafe
PASS toolchain
PASS lockfile
PASS publish
PASS deps
PASS credits
PASS clippy
PASS names
PASS spec
FAIL golden: 24 *.bin paths are not binary: the .gitattributes files give crates/codec/tests/golden/v1/blob-envelope.bin `text: unspecified`
cargo policy exit 1 (1 required)
== int_disallowed_types_probe: HashMap, then HashSet
-- planted std::collections::HashMap:  1 file changed, 5 insertions(+)
error: use of a disallowed type `std::collections::HashMap`
clippy exit 101 (non-zero required)
-- planted std::collections::HashSet:  1 file changed, 5 insertions(+)
error: use of a disallowed type `std::collections::HashSet`
clippy exit 101 (non-zero required)
== int_aliases_refuse_flags_and_stale_locks: aliases with flags, then a stale Cargo.lock
cargo ci --fast exit 2 (2 required)
cargo policy --fix exit 2 (2 required)
cargo book --strict exit 2 (2 required)
changed:  M Cargo.toml 
error: cannot update the lock file /home/charles/periapsis-s0-probe/Cargo.lock because --locked was passed to prevent this
cargo ci exit 101 (101 required); Cargo.lock changed: 0
== done
wsl exit 0

==== after
working checkout uncommitted paths: 0
tested head c342412a2951c70d7db68608f09e300e55e745be
```

## Scripts

### `test_matrix.sh`

````bash
#!/usr/bin/env bash
# Sprint 0 Test phase: every integration and E2E run in the test plan, at the
# committed head of dev, plus the negative controls the test critic asked for.
# Windows-host runs happen here (Git Bash); the WSL leg runs wsl_ci.sh inside
# WSL. Output is the record the sprint-tests files cite.
set -u
REPO=C:/Users/charl/Animus_Periapsis
SL=C:/Users/charl/Animus_Sprint_Loops
S=C:/Users/charl/AppData/Local/Temp/claude/C--Users-charl--claude-spares-spare-61ekw4/5eb9caae-ac3b-41b6-a91c-2cad206f0660/scratchpad
WSL_SCRIPT=/mnt/c/Users/charl/AppData/Local/Temp/claude/C--Users-charl--claude-spares-spare-61ekw4/5eb9caae-ac3b-41b6-a91c-2cad206f0660/scratchpad/tools/wsl_ci.sh

section() { printf '\n==== %s\n' "$1"; }

# The target directory cargo uses in the current directory. Every checkout
# here builds into its own: cargo decides freshness by file times, so a
# shared one could run binaries built for another checkout.
target_dir() {
  cargo metadata --format-version 1 --no-deps 2>/dev/null |
    python -X utf8 -c "import json, sys; print(json.load(sys.stdin)['target_directory'])"
}

# The Cargo configuration files that a build in checkout "$1" would read
# besides the checkout's own: `.cargo/config` and `.cargo/config.toml` in
# every directory above it, and in CARGO_HOME.
configs_above() {
  local dir found=""
  dir=$(dirname "$(cygpath -u "$1")")
  while :; do
    for file in "$dir/.cargo/config" "$dir/.cargo/config.toml"; do
      [ -f "$file" ] && found="$found $file"
    done
    [ "$(dirname "$dir")" = "$dir" ] && break
    dir=$(dirname "$dir")
  done
  for file in "${CARGO_HOME:-$HOME/.cargo}/config" "${CARGO_HOME:-$HOME/.cargo}/config.toml"; do
    [ -f "$file" ] && found="$found $file"
  done
  echo "${found:- none}"
}

# A fresh clone of the committed head in a new scratch directory.
fresh_clone() {
  rm -rf "$1"
  git clone --quiet --branch dev "$REPO" "$1"
}

# Appends a formatted function using `$1` to codec's lib.rs.
plant() {
  printf '\n/// Probe only: a randomly seeded collection Clippy must reject.\npub fn probe() -> usize {\n    %s::new().len()\n}\n' "$1" >> crates/codec/src/lib.rs
}

# Removes a scratch clone, retrying while Windows still holds it.
discard() {
  for _ in 1 2 3 4 5; do rm -rf "$1" 2>/dev/null && break; sleep 1; done
  echo "scratch clone removed: $([ -e "$1" ] && echo no || echo yes)"
}

section "host (Windows)"
powershell.exe -NoProfile -Command "(Get-CimInstance Win32_OperatingSystem) | ForEach-Object { \$_.Caption + ' ' + \$_.Version }" 2>&1 | tr -d '\r'
rustc -Vv
cargo deny --version
git --version
set_vars=$(env | grep -E '^(PROPTEST_|PERI_|CLIPPY_CONF_DIR|GIT_|RUSTFLAGS|RUSTDOCFLAGS|RUSTC|RUSTDOC|RUSTUP_TOOLCHAIN|CARGO_BUILD_|CARGO_TARGET_|CARGO_PROFILE_|CARGO_ENCODED_|CARGO_ALIAS_)' | cut -d= -f1 | tr '\n' ' ')
echo "PROPTEST_*, PERI_*, CLIPPY_CONF_DIR, GIT_*, RUSTFLAGS, RUSTDOCFLAGS, RUSTC*, RUSTDOC, RUSTUP_TOOLCHAIN, CARGO_BUILD_*, CARGO_TARGET_*, CARGO_PROFILE_*, CARGO_ENCODED_* or CARGO_ALIAS_* set: ${set_vars:-none}"
cd "$REPO" || exit 10
HEAD_SHA=$(git rev-parse HEAD)
echo "head $HEAD_SHA on $(git branch --show-current); uncommitted paths: $(git status --porcelain | wc -l)"
echo "target directory: $(target_dir)"
echo "cargo config files above the working checkout or in CARGO_HOME:$(configs_above "$REPO")"

section "int_ci_windows (working checkout, NTFS)"
cargo ci 2>&1 | grep -E '^(PASS|FAIL|ci:)'
echo "cargo ci exit ${PIPESTATUS[0]}"

section "int_deny_windows"
cargo deny check 2>&1 | tail -3
echo "cargo deny exit ${PIPESTATUS[0]}"
for db in "$HOME"/.cargo/advisory-dbs/*/; do
  echo "advisory db $(basename "$db"): $(git -C "$db" log -1 --format='%H %cs')"
done

section "unit listing (Windows working checkout)"
cargo test --workspace --locked 2>&1 | grep -E '^\s+Running|^test result' | sed -E 's/ \(target[^)]*\)//'
echo "tests passed: $(cargo test --workspace --locked 2>&1 | grep -cE '^test .* ok$')"

section "int_cc_build_dependency_audit"
cargo tree --workspace --target all -e normal,build -i cc --depth 1 2>&1
cargo tree --workspace -e features -i blake3 2>&1 | grep -E '^blake3 v|feature "pure"|feature "std"'
echo "C-build yes rows:"
grep -E '^\| `[a-z0-9_-]+` \|' docs/prior-art.md | awk -F'|' '$6 ~ /^ *yes/ {print " " $2}'

section "int_disallowed_types_probe (two scratch clones, discarded)"
for type in "std::collections::HashMap::<u8, u8>" "std::collections::HashSet::<u8>"; do
  P="$S/probe"
  fresh_clone "$P"
  (
    cd "$P" || exit 11
    plant "$type"
    name=${type%%::<*}
    echo "-- planted $name at $(git rev-parse --short HEAD): $(git diff --stat | tail -1)"
    echo "target directory: $(target_dir)"
    echo "cargo config files above the clone or in CARGO_HOME:$(configs_above "$PWD")"
    cargo clippy --workspace --all-targets -- -D warnings > "$S/probe.out" 2>&1
    code=$?
    grep -E "disallowed type \`$name\`" "$S/probe.out" | head -1
    echo "clippy exit $code (non-zero required)"
  )
  discard "$P"
done

section "int_ci_reports_failures (Windows scratch clone: one golden byte changed, one fmt violation)"
P="$S/probe"
fresh_clone "$P"
(
  cd "$P" || exit 12
  printf 's' | dd of=crates/codec/tests/golden/v1/string-ascii.bin bs=1 seek=3 count=1 conv=notrunc 2>/dev/null
  printf '\npub fn  badly_formatted( ) {}\n' >> crates/codec/src/lib.rs
  echo "changed: $(git status --porcelain | tr '\n' ' ')"
  echo "target directory: $(target_dir)"
  echo "cargo config files above the clone or in CARGO_HOME:$(configs_above "$PWD")"
  cargo ci > "$S/probe.out" 2>&1
  code=$?
  grep -E '^(PASS|FAIL|ci:)' "$S/probe.out"
  echo "cargo ci exit $code (1 required)"
  # The test step's own report: the golden test failed on the changed file.
  tr -d '\r' < "$S/probe.out" | grep -E '^test golden_v1_corpus_matches \.\.\. |string-ascii\.bin: encoding changed'
)
discard "$P"

section "int_ci_reports_book_violation (Windows scratch clone: one untyped criterion)"
P="$S/probe"
fresh_clone "$P"
(
  cd "$P" || exit 16
  sed -i '/^## Acceptance criteria$/a - An untyped probe criterion.' docs/intents/INT-0031-visual-identity.md
  echo "changed: $(git status --porcelain | tr '\n' ' ')"
  echo "target directory: $(target_dir)"
  echo "cargo config files above the clone or in CARGO_HOME:$(configs_above "$PWD")"
  cargo ci > "$S/probe.out" 2>&1
  code=$?
  grep -E '^(PASS|FAIL|ci:)' "$S/probe.out"
  echo "cargo ci exit $code (1 required)"
  tr -d '\r' < "$S/probe.out" | grep -E '^test book_intents_pass_strict \.\.\. |INT-0031-visual-identity\.md:[0-9]+: untyped: ' | head -3
)
discard "$P"

section "int_ci_ignores_hostile_environment (Windows scratch clone: HashMap planted, a golden file deleted)"
P="$S/probe"
fresh_clone "$P"
mkdir -p "$S/empty-clippy-conf"
(
  cd "$P" || exit 13
  plant "std::collections::HashMap::<u8, u8>"
  rm crates/codec/tests/golden/v1/uvarint-0.bin
  echo "changed: $(git status --porcelain | tr '\n' ' ')"
  echo "target directory: $(target_dir)"
  echo "cargo config files above the clone or in CARGO_HOME:$(configs_above "$PWD")"
  PROPTEST_CASES=1 PERI_GOLDEN_CREATE=1 CLIPPY_CONF_DIR="$S/empty-clippy-conf" \
    cargo ci > "$S/probe.out" 2>&1
  code=$?
  grep -E '^(PASS|FAIL|ci:)' "$S/probe.out"
  echo "cargo ci exit $code (1 required)"
  grep -E 'disallowed type `std::collections::HashMap`' "$S/probe.out" | head -1
  grep -E '^test (case_count_is_not_overridden|case_counts_are_not_overridden|golden_v1_corpus_matches) \.\.\. ' "$S/probe.out"
  echo "uvarint-0.bin recreated: $([ -e crates/codec/tests/golden/v1/uvarint-0.bin ] && echo yes || echo no)"
)
discard "$P"

section "int_policy_reports_failures (Windows scratch clone: .gitattributes emptied, so nothing marks *.bin binary)"
P="$S/probe"
fresh_clone "$P"
printf '*.bin binary\n' > "$S/hostile-attributes"
(
  cd "$P" || exit 15
  # Empty, so that a global attributes file could decide: a workspace line
  # such as `* text=auto` would outrank it.
  : > .gitattributes
  echo "changed: $(git status --porcelain | tr '\n' ' ')"
  echo "target directory: $(target_dir)"
  echo "cargo config files above the clone or in CARGO_HOME:$(configs_above "$PWD")"
  echo "-- the caller's environment as is"
  cargo policy > "$S/probe.out" 2>&1
  code=$?
  grep -E '^(PASS|FAIL)' "$S/probe.out"
  echo "cargo policy exit $code (1 required)"
  echo "-- Git variables that point core.attributesFile at a file marking *.bin binary"
  hostile() {
    GIT_CONFIG_PARAMETERS="'core.attributesfile'='$S/hostile-attributes'" \
      GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.attributesFile GIT_CONFIG_VALUE_0="$S/hostile-attributes" \
      "$@"
  }
  echo "control, plain git check-attr: $(git check-attr text -- crates/codec/tests/golden/v1/uvarint-0.bin)"
  echo "control, with the variables:   $(hostile git check-attr text -- crates/codec/tests/golden/v1/uvarint-0.bin)"
  hostile cargo policy > "$S/probe.out" 2>&1
  code=$?
  grep -E '^(PASS|FAIL)' "$S/probe.out"
  echo "cargo policy exit $code (1 required)"
)
discard "$P"

section "int_aliases_refuse_flags_and_stale_locks (Windows scratch clone: aliases with flags, then a stale Cargo.lock)"
P="$S/probe"
fresh_clone "$P"
(
  cd "$P" || exit 17
  echo "target directory: $(target_dir)"
  echo "cargo config files above the clone or in CARGO_HOME:$(configs_above "$PWD")"
  for args in "ci --fast" "policy --fix" "book --strict"; do
    cargo $args > "$S/probe.out" 2>&1
    echo "cargo $args exit $? (2 required)"
  done
  sed -i 's/^version = "0.0.0"$/version = "0.0.1"/' Cargo.toml
  echo "changed: $(git status --porcelain | tr '\n' ' ')"
  cargo ci > "$S/probe.out" 2>&1
  code=$?
  grep -m1 -E 'cannot update the lock file' "$S/probe.out"
  echo "cargo ci exit $code (101 required); Cargo.lock changed: $(git status --porcelain -- Cargo.lock | wc -l)"
)
discard "$P"

section "e2e_fresh_clone_windows (core.autocrlf=true)"
W="$S/e2e-autocrlf"
rm -rf "$W"
git clone --quiet -c core.autocrlf=true --branch dev "$REPO" "$W"
(
  cd "$W" || exit 14
  echo "clone head $(git rev-parse HEAD), core.autocrlf=$(git config core.autocrlf)"
  echo "target directory: $(target_dir)"
  echo "cargo config files above the clone or in CARGO_HOME:$(configs_above "$PWD")"
  (cd crates/codec/tests/golden/v1 && sha256sum ./*.bin) > "$S/clone.sha"
  (cd "$REPO/crates/codec/tests/golden/v1" && sha256sum ./*.bin) > "$S/orig.sha"
  if diff -q "$S/clone.sha" "$S/orig.sha" >/dev/null; then
    echo "golden: all $(wc -l < "$S/clone.sha") .bin files byte-identical to the original"
  else
    echo "golden: MISMATCH"; diff "$S/clone.sha" "$S/orig.sha"
  fi
  eol=$(git ls-files --eol)
  echo "tracked files: $(printf '%s\n' "$eol" | wc -l); with CRLF or mixed line ends in the working tree: $(printf '%s\n' "$eol" | grep -cE 'w/(crlf|mixed)')"
  start=$(date +%s)
  cargo ci 2>&1 | grep -E '^(PASS|FAIL|ci:)'
  echo "cargo ci exit ${PIPESTATUS[0]} (cold build, $(( $(date +%s) - start )) s)"
)

section "e2e_port_rule_sprint_loops_book"
echo "Animus_Sprint_Loops head $(git -C "$SL" rev-parse HEAD), uncommitted paths: $(git -C "$SL" status --porcelain | wc -l)"
echo "INT files: $(ls "$SL"/docs/intents/INT-*.md | wc -l)"
total=0
for f in "$SL"/docs/intents/INT-*.md; do
  n=$(awk '/^(```|~~~)/{fence=!fence; next} fence{next} /^## /{inside=($0 ~ /^## Acceptance criteria[[:space:]]*$/); next} inside && /^- /{c++} END{print c+0}' "$f")
  echo "awk $(basename "$f"): $n"
  total=$((total + n))
done
echo "independent count (awk): $total"
cargo book "$SL/docs" 2>&1
echo "cargo book exit $?"

section "e2e_port_rule_local_books (every Book v2 on this host vs markdown-it-py)"
python -X utf8 -c "import markdown_it; print('markdown-it-py', markdown_it.__version__)"
python -X utf8 "$S/tools/port_corpus.py" "$REPO" $(ls -d C:/Users/charl/*/docs/.sprint-loop-book | sed 's#/.sprint-loop-book##')
echo "port_corpus exit $?"
for book in $(ls -d C:/Users/charl/*/docs/.sprint-loop-book | sed 's#/docs/.sprint-loop-book##'); do
  if git -C "$book" rev-parse --git-dir > /dev/null 2>&1; then
    echo "  $(basename "$book"): $(git -C "$book" rev-parse --short HEAD), docs/intents uncommitted: $(git -C "$book" status --porcelain -- docs/intents | wc -l)"
  else
    echo "  $(basename "$book"): not a git checkout"
  fi
done

section "WSL leg (int_ci_wsl, int_tooling_is_idempotent, int_ci_reports_failures, int_ci_reports_book_violation, int_policy_reports_failures, int_disallowed_types_probe, int_aliases_refuse_flags_and_stale_locks)"
# A login shell puts ~/.cargo/bin on PATH; the copy drops any CR line ends.
wsl.exe -d Ubuntu -- bash -lc "tr -d '\r' < '$WSL_SCRIPT' > /tmp/wsl_ci.sh && bash /tmp/wsl_ci.sh" 2>&1 | tr -d '\r'
echo "wsl exit ${PIPESTATUS[0]}"

section "after"
echo "working checkout uncommitted paths: $(git -C "$REPO" status --porcelain | wc -l)"
echo "tested head $HEAD_SHA"
````

### `wsl_ci.sh`

````bash
#!/usr/bin/env bash
# Sprint 0 Test phase, Linux leg: int_ci_wsl, int_tooling_is_idempotent,
# int_ci_reports_failures, int_ci_reports_book_violation,
# int_policy_reports_failures, int_disallowed_types_probe and
# int_aliases_refuse_flags_and_stale_locks. Clones the committed
# dev head into the WSL ext4 home directory (not /mnt/c) and runs the project
# commands there. Every checkout builds into its own target directory.
set -u
SRC=/mnt/c/Users/charl/Animus_Periapsis
DEST="$HOME/periapsis-s0-test"
PROBE="$HOME/periapsis-s0-probe"

target_dir() {
  cargo metadata --format-version 1 --no-deps 2>/dev/null |
    python3 -c "import json, sys; print(json.load(sys.stdin)['target_directory'])"
}

# The Cargo configuration files that a build in checkout "$1" would read
# besides the checkout's own: `.cargo/config` and `.cargo/config.toml` in
# every directory above it, and in CARGO_HOME.
configs_above() {
  local dir found=""
  dir=$(dirname "$1")
  while :; do
    for file in "$dir/.cargo/config" "$dir/.cargo/config.toml"; do
      [ -f "$file" ] && found="$found $file"
    done
    [ "$(dirname "$dir")" = "$dir" ] && break
    dir=$(dirname "$dir")
  done
  for file in "${CARGO_HOME:-$HOME/.cargo}/config" "${CARGO_HOME:-$HOME/.cargo}/config.toml"; do
    [ -f "$file" ] && found="$found $file"
  done
  echo "${found:- none}"
}

echo "== host"
uname -srm
grep PRETTY_NAME /etc/os-release
git --version
set_vars=$(env | grep -E '^(PROPTEST_|PERI_|CLIPPY_CONF_DIR|GIT_|RUSTFLAGS|RUSTDOCFLAGS|RUSTC|RUSTDOC|RUSTUP_TOOLCHAIN|CARGO_BUILD_|CARGO_TARGET_|CARGO_PROFILE_|CARGO_ENCODED_|CARGO_ALIAS_)' | cut -d= -f1 | tr '\n' ' ')
echo "PROPTEST_*, PERI_*, CLIPPY_CONF_DIR, GIT_*, RUSTFLAGS, RUSTDOCFLAGS, RUSTC*, RUSTDOC, RUSTUP_TOOLCHAIN, CARGO_BUILD_*, CARGO_TARGET_*, CARGO_PROFILE_*, CARGO_ENCODED_* or CARGO_ALIAS_* set: ${set_vars:-none}"
echo "== clone"
rm -rf "$DEST"
git clone --quiet --branch dev "$SRC" "$DEST" || exit 10
cd "$DEST" || exit 11
echo "head $(git rev-parse HEAD)"
echo "checkout on $(df --output=fstype . | tail -1)"
rustup toolchain install >/dev/null 2>&1
rustc -Vv
echo "target directory: $(target_dir)"
echo "cargo config files above the clone or in CARGO_HOME:$(configs_above "$PWD")"
echo "== int_ci_wsl: cargo ci"
cargo ci 2>&1 | grep -E '^(PASS|FAIL|ci:)'
echo "cargo ci exit ${PIPESTATUS[0]}"
echo "tests passed on Linux: $(cargo test --workspace --locked 2>&1 | grep -cE '^test .* ok$')"
echo "== int_tooling_is_idempotent"
for run in 1 2; do
  for command in ci policy book; do
    out=$(cargo "$command" 2>&1)
    code=$?
    dirty=$(git status --porcelain | wc -l)
    counted=$(printf '%s\n' "$out" | grep -c ' criteria$')
    printf 'run %s cargo %s: exit %s, dirty files %s' "$run" "$command" "$code" "$dirty"
    [ "$command" = book ] && printf ', counted chapter lines %s' "$counted"
    printf '\n'
  done
done
echo "chapters in docs/intents: $(ls docs/intents/INT-*.md | wc -l)"
echo "== int_ci_reports_failures: one golden byte changed, one fmt violation"
rm -rf "$PROBE"
git clone --quiet "$DEST" "$PROBE" || exit 12
cd "$PROBE" || exit 13
printf 's' | dd of=crates/codec/tests/golden/v1/string-ascii.bin bs=1 seek=3 count=1 conv=notrunc 2>/dev/null
printf '\npub fn  badly_formatted( ) {}\n' >> crates/codec/src/lib.rs
echo "changed: $(git status --porcelain | tr '\n' ' ')"
echo "target directory: $(target_dir)"
echo "cargo config files above the clone or in CARGO_HOME:$(configs_above "$PWD")"
out=$(cargo ci 2>&1)
code=$?
printf '%s\n' "$out" | grep -E '^(PASS|FAIL|ci:)'
echo "cargo ci exit $code (1 required)"
# The test step's own report: the golden test failed on the changed file.
printf '%s\n' "$out" | grep -E '^test golden_v1_corpus_matches \.\.\. |string-ascii\.bin: encoding changed'
echo "== int_ci_reports_book_violation: one untyped criterion"
git checkout --quiet -- .
sed -i '/^## Acceptance criteria$/a - An untyped probe criterion.' docs/intents/INT-0031-visual-identity.md
echo "changed: $(git status --porcelain | tr '\n' ' ')"
out=$(cargo ci 2>&1)
code=$?
printf '%s\n' "$out" | grep -E '^(PASS|FAIL|ci:)'
echo "cargo ci exit $code (1 required)"
printf '%s\n' "$out" | grep -E '^test book_intents_pass_strict \.\.\. |INT-0031-visual-identity\.md:[0-9]+: untyped: ' | head -3
echo "== int_policy_reports_failures: .gitattributes emptied, so nothing marks *.bin binary"
git checkout --quiet -- .
: > .gitattributes
printf '*.bin binary\n' > "$HOME/hostile-attributes"
echo "changed: $(git status --porcelain | tr '\n' ' ')"
echo "-- the caller's environment as is"
out=$(cargo policy 2>&1)
code=$?
printf '%s\n' "$out" | grep -E '^(PASS|FAIL)'
echo "cargo policy exit $code (1 required)"
echo "-- Git variables that point core.attributesFile at a file marking *.bin binary"
hostile() {
  GIT_CONFIG_PARAMETERS="'core.attributesfile'='$HOME/hostile-attributes'" \
    GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.attributesFile GIT_CONFIG_VALUE_0="$HOME/hostile-attributes" \
    "$@"
}
echo "control, plain git check-attr: $(git check-attr text -- crates/codec/tests/golden/v1/uvarint-0.bin)"
echo "control, with the variables:   $(hostile git check-attr text -- crates/codec/tests/golden/v1/uvarint-0.bin)"
out=$(hostile cargo policy 2>&1)
code=$?
printf '%s\n' "$out" | grep -E '^(PASS|FAIL)'
echo "cargo policy exit $code (1 required)"
echo "== int_disallowed_types_probe: HashMap, then HashSet"
for type in "std::collections::HashMap::<u8, u8>" "std::collections::HashSet::<u8>"; do
  git checkout --quiet -- .
  printf '\n/// Probe only: a randomly seeded collection Clippy must reject.\npub fn probe() -> usize {\n    %s::new().len()\n}\n' "$type" >> crates/codec/src/lib.rs
  name=${type%%::<*}
  echo "-- planted $name: $(git diff --stat | tail -1)"
  out=$(cargo clippy --workspace --all-targets -- -D warnings 2>&1)
  code=$?
  printf '%s\n' "$out" | grep -E "disallowed type \`$name\`" | head -1
  echo "clippy exit $code (non-zero required)"
done
echo "== int_aliases_refuse_flags_and_stale_locks: aliases with flags, then a stale Cargo.lock"
git checkout --quiet -- .
for args in "ci --fast" "policy --fix" "book --strict"; do
  cargo $args > /dev/null 2>&1
  echo "cargo $args exit $? (2 required)"
done
sed -i 's/^version = "0.0.0"$/version = "0.0.1"/' Cargo.toml
echo "changed: $(git status --porcelain | tr '\n' ' ')"
out=$(cargo ci 2>&1)
code=$?
printf '%s\n' "$out" | grep -m1 -E 'cannot update the lock file'
echo "cargo ci exit $code (101 required); Cargo.lock changed: $(git status --porcelain -- Cargo.lock | wc -l)"
cd "$HOME" && rm -rf "$PROBE" "$HOME/hostile-attributes"
echo "== done"
````

### `port_corpus.py`

````python
"""Compare `cargo book` with an independent CommonMark count, Book by Book.

Usage: python -I port_corpus.py <periapsis-repo> <docs-dir> [<docs-dir> ...]

For each Book, markdown-it-py (CommonMark preset) counts the top-level list
items of every chapter's top-level level-2 "Acceptance criteria" section, up
to the next top-level level-1 or level-2 heading. The Periapsis `book` binary
is run on the same directory and its per-chapter counts are compared. One row
per Book; per-chapter mismatches are listed.
"""

import re
import subprocess
import sys
from pathlib import Path

from markdown_it import MarkdownIt

MD = MarkdownIt("commonmark")
CHAPTER_LINE = re.compile(r"^\s+(\S+\.md): (\d+) criteria$")
SUMMARY = re.compile(r"^(\d+) chapters, (\d+) criteria, (\d+) violations$")


def commonmark_count(text: str) -> int:
    tokens = MD.parse(text)
    count = 0
    inside = False
    for index, token in enumerate(tokens):
        if token.type == "heading_open" and token.level == 0:
            if token.tag in ("h1", "h2"):
                title = tokens[index + 1].content.strip()
                inside = token.tag == "h2" and title == "Acceptance criteria"
            continue
        if inside and token.type == "list_item_open" and token.level == 1:
            count += 1
    return count


def book_counts(repo: Path, docs: Path) -> tuple[dict[str, int], str, int]:
    run = subprocess.run(
        ["cargo", "run", "--locked", "--quiet", "--package", "book", "--", str(docs)],
        cwd=repo,
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    counts = {}
    summary = ""
    for line in run.stdout.splitlines():
        if match := CHAPTER_LINE.match(line):
            counts[match.group(1)] = int(match.group(2))
        elif SUMMARY.match(line):
            summary = line
    return counts, summary, run.returncode


def main() -> int:
    repo = Path(sys.argv[1])
    print("| Book | Rule | Chapters | `cargo book` | CommonMark | Violations | Exit | Mismatches |")
    print("|---|---|---|---|---|---|---|---|")
    bad = 0
    for name in sys.argv[2:]:
        docs = Path(name)
        marker = (docs / ".sprint-loop-book").read_text(encoding="utf-8")
        rule = "strict" if "typed-acceptance:" in marker else "port"
        chapters = sorted(
            p for p in (docs / "intents").glob("*.md") if p.name != "README.md"
        )
        expected = {p.name: commonmark_count(p.read_text(encoding="utf-8")) for p in chapters}
        counts, summary, code = book_counts(repo, docs)
        mismatches = [
            f"{chapter} {counts.get(chapter)} vs {want}"
            for chapter, want in expected.items()
            if counts.get(chapter) != want
        ]
        match = SUMMARY.match(summary)
        violations = match.group(3) if match else "?"
        bad += bool(mismatches) or code != 0
        book = docs.parent.name
        print(
            f"| {book} | {rule} | {len(chapters)} | {sum(counts.values())} | "
            f"{sum(expected.values())} | {violations} | {code} | "
            f"{'; '.join(mismatches) or 'none'} |"
        )
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
````

Note (2026-11-14): one project in this log was renamed after the run. Lines 253 and 278 show its current name, LEXPLOIT; the run read that Book from the project's folder under its earlier name. Nothing else in the log changed.
