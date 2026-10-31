# INT-0019 — A cheat-resistant verifier and a human-signed Book

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0019
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
No agent can get a change accepted as discharging an obligation it has not
met. Trusted: the human-signed Book (intent statements, obligations, negative
witnesses), the verifier, the pinned Lean toolchain and the verifier-written
evidence ledger. Untrusted: every agent-produced `.lean` or `.olean` file and
every agent-produced record. The verifier runs seven layers:

1. no agent code runs (modules imported without loading extensions);
2. import policy — only trusted Book modules and `Init.*`;
3. raw extension-entry allowlist, fail-closed;
4. attribute policy — no `unsafe`, `implemented_by`, `extern` or `csimp`;
5. kernel replay into an environment built only from trusted Book modules;
6. an independent axiom audit of the replayed environment;
7. statement pinning — the certificate's type is syntactically the Book
   obligation, and every constant in it comes from a trusted module.

Book integrity: `book.lock` holds hashes of the trusted Book modules, the
verifier and the toolchain and build configuration, signed (Ed25519) by a
principal listed under `book` in `allowed_signers`; agents hold keys but no
Book-signing rights. The two single-point layers (axiom audit, statement
pinning) are implemented twice, independently (Lean's comparator; a second
kernel, nanoda, via exported terms). Every layer has a planted-regression probe
that proves it can fail, as a required CI job. Every formal intent carries a
registry of mutants it must reject, and every Book edit shows its
mutation-score diff; a drop needs a signed justification. In production, agent
builds run only in a sandbox, cross the boundary as exported terms and are
replayed by at least two kernels on toolchain 4.33.1 or later.

## Acceptance criteria
- **AC-1** `necessary` `empirical` — Every cheat in the red-team corpus is rejected and every honest control is accepted; the corpus only grows.
  - Note: R5 rejected 13 of 13 cheats (A01–A08, K1, K2, B1, B1b, B1c) and accepted 4 of 4 controls (A09, A10, K0 ×2).
- **AC-2** `necessary` `empirical` — Ablation shows every attack class caught by at least two independent layers.
  - Note: R5 found the axiom audit and statement pinning were single points of failure, hence the N-version requirement.
- **AC-3** `necessary` `empirical` — Disabling any single layer turns its planted-regression probe red in CI.
- **AC-4** `necessary` `empirical` — A Book edit that weakens an obligation is rejected unless signed by a `book` principal (B1, B1b, B1c), and the reviewer sees the mutation-score diff.
  - Note: R5 showed the weakened root intent's score falling from 6/6 to 0/6.
- **AC-5** `necessary` `formal` — Adequacy: every formal Book intent is kernel-proved to reject each of its registered mutants.
  - Obligation: one `rejects_<mutant>` theorem per registry entry.
- **AC-6** `supporting` `empirical` — The rejection rate of honest certificates is tracked in production, and the allowlist grows only through signed Book changes.

## Rationale
- The common gate ("it builds and `#print axioms` is clean") accepted 3 of 8
  proof-level cheats: a weakened statement, a shadowed obligation name and a
  kernel-bypassing metaprogram (R5 §A.3).
- Kernel and checker exploits are routine in 2026: Lean issue #14576 /
  CVE-2026-72844 passed two kernels whose bugs aligned; refcount-overflow and
  GMP exploits were fixed in Lean 4.33.1; `@[csimp]` can hide axioms from
  `#print axioms` (lean4#7463) (R5 §A.1).
- Agents game evaluations (RL reward hacking by axiom injection; ImpossibleBench
  76%), and the most realistic cheat needs no Lean trickery: weaken the Book,
  delete the failing witnesses, then prove the weaker claim honestly (R5 §A.6).
- The ablation found a silent bug in the prototype verifier — `@[csimp]` became
  invisible with extensions unloaded — which is why every layer needs a probe
  that proves it can fail (R5 §A.5).
- Sources: [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md)
  Part VIII §A, with its external sources in §F.

## Alternatives
- **The naive gate** (build + axiom listing). Accepts statement-level and
  kernel-bypass cheats.
- **LLM proof reviewers.** Caught same-file attacks 35 of 36 times but missed
  attacks hidden in imported modules without tool access (R5 §A.1).
- **Trusting agent-built `.olean` files.** Replaced by sandboxed builds and
  exported terms.

## Consequences
- The verifier is security-critical code with its own red-team corpus,
  ablation runs and probes, all of which must be maintained.
- Kernel diversity lowers but does not remove the risk of aligned kernel bugs.
- Agent-defined inductive types are rejected until replay supports them, which
  can cause false positives (R5 §A.8).

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 Part VIII §A).
