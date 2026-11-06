# Agent Tasks (Persistent Backlog)

- [ ] T-002 (sprint 1) [intent: INT-0005, INT-0004]: Filesystem and randomness traits in the sim crate — touches: crates/sim/, Cargo.toml, Cargo.lock, docs/prior-art.md
- [ ] T-003 (sprint 1) [intent: INT-0005, INT-0003]: Filesystem boundary: only sim's real half touches the real filesystem — touches: crates/sim/clippy.toml, crates/sim/src/real.rs, crates/xtask/src/policy.rs
- [ ] T-004 (sprint 1) [intent: INT-0007]: Object and op-file store with derived heads and fsck in the store crate — touches: crates/store/, Cargo.toml, Cargo.lock, docs/prior-art.md
- [ ] T-005 (sprint 1) [intent: INT-0005, INT-0007]: Crash matrix with negative controls — touches: crates/store/, crates/sim/src/simfs.rs, crates/xtask/src/policy.rs
- [ ] T-006 (sprint 1) [intent: INT-0007, INT-0003]: fsck corruption corpus and the praps fsck command — touches: crates/store/tests/, crates/praps/, Cargo.toml, Cargo.lock, README.md
- [ ] T-007 (sprint 1) [intent: INT-0004, INT-0007]: Windows durability strategy and real-filesystem evidence — touches: docs/spec/storage-durability.md, crates/sim/tests/real_durability.rs, crates/store/tests/real_store.rs, docs/SUMMARY.md
- [ ] T-008 (sprint 1) [intent: INT-0003]: cargo ci refuses a target directory outside the checkout (absorbs former backlog T-010) — touches: crates/xtask/src/ci.rs, crates/xtask/src/main.rs
- [ ] T-009 (sprint 1) [intent: INT-0001, INT-0002, INT-0003, INT-0006]: Record Sprint 0's GitHub Actions runs and owner sign-offs (absorbs former backlog T-009) — touches: docs/sprints/s1/sprint-tests/ci-sprint0.md, docs/intents/
