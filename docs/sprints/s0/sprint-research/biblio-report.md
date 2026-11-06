# Research Report: Rewriting Animus_Sprint_Loops as "Biblio", an agent-native version-management system written in Rust

**Question:** Should Animus_Sprint_Loops be rewritten as a standalone, Git-independent version-management product built around agent-driven flows? The proposal includes cryptographically verifiable agent identities (for example via Nostr), verified sunsetting of agents, an agent life-story and lineage record alongside a "book of intents", agents returning as performance-selected "descendants" or blends, and a codebase that is 100% Rust and auditable "down to the compiler".

**Method:** I split the question into five sub-questions and answered each from primary sources where possible: specs, official READMEs and docs, maintainer issue threads, and the crates.io API. Secondary sources are marked **[secondary]**. Statements that are my own analysis are marked **[analysis]**. Sources were checked on 2026-10-26.

---

## Executive summary

| Idea | Verdict | Why |
|---|---|---|
| Book of intents plus agent life stories and lineage | **Build it.** This is the new part. | The existing repo already stores state as evidence; adding agents as signed actors is a natural extension. |
| A VCS engine completely apart from Git | **Don't write one from scratch.** Build on jj's library or Pijul, and keep Git interop. | jj's backends are pluggable Rust traits with an operation log built for concurrency; Pijul keeps patch identity permanently. Both are Rust. |
| Nostr identity per agent | **Use it to publish records, not as the source of truth.** | NIP-26 delegation is deprecated, key revocation (NIP-41) is best-effort only, and `created_at` can be backdated. |
| Verified sunsetting | **Possible only as "nothing signed after this log entry counts".** | You can't prove a key was destroyed. Ordering has to come from your own hash-chained log. |
| Descendants / blends | **Possible as a lineage graph of configurations** | Record in-toto attestations; scoring must come from an independent verifier, not the agent itself. |
| 100% Rust, verifiable down to the compiler | **Not achievable as stated.** Aim for "reproducible and bootstrappable". | rustc's backend (LLVM) is C++, the independent bootstrap path (mrustc) is C++, and even the Rust `nostr` crate uses a C cryptography library. |

---

## Sub-question 1: What does the existing repo do, and what would a rewrite change?

- Animus Sprint Loops "turns a project's documentation into a living **Project Book**: one durable, linkable record of intended outcomes, active work, realization evidence, and the sprint history that connects them", working through Research → Plan → Build → Test → Loop. (README, https://github.com/crussella0129/Animus_Sprint_Loops)
- State is derived from evidence: "Each phase writes its exit artifact into `docs/`; the current phase is *derived* from that evidence, never a mutable pointer." (README)
- The Book is split into `docs/intents/` (the authority on desired outcomes), `docs/work/` and `docs/sprints/`. (README)
- The Git dependency sits in the branch and checkpoint model: "Each sprint opens **exactly one `dev → main`** pull/merge request as a reversible checkpoint". The logic lives in shell helpers such as `deploy-substrate.sh`, `close-sprint.sh` and `remote-adapter.sh`. (README)
- The repo already checks whether its own tests can fail (suite-sensitivity checks), and it is candid that this is "a floor, not a proof". (README)

**[analysis]** The "book of intents" already exists. What a rewrite adds:
1. Typed, signed records in Rust replacing shell scripts plus Markdown conventions.
2. **Agents as recorded actors**, with birth, work, sunset and descent.
3. A storage and transport layer not tied to GitHub/GitLab pull requests.

Point 3 doesn't require a new VCS (see sub-question 2).

---

## Sub-question 2: Should the version management be a new VCS, independent of Git?

### Prior art in Rust

**Jujutsu (jj)**: https://github.com/jj-vcs/jj, docs at https://docs.jj-vcs.dev
- Operation log: "Jujutsu records each operation that modifies the repo in the 'operation log'… The operation object also… contains pointers to the operation(s) immediately before it, as well as metadata about the operation, such as timestamps, username, hostname, description." (docs/operation-log.md)
- Concurrency: the op log "allows lock-free concurrency -- you can run concurrent jj commands without corrupting the repo, even if you run the commands on different machines". (docs/operation-log.md) This suits many agents writing at once.
- Pluggable storage: commits, operations, op-heads, index and working copy each have their own backend. "The interfaces are defined in terms of plain Rust data types, not tied to a specific format." (https://docs.jj-vcs.dev/latest/technical/architecture/)
- Limitation: "Today, only the Git backend is production-ready. The Git backend uses the gitoxide Rust library." (README)
- Conflicts are recorded in commits instead of blocking the operation, so "there's no need to continue interrupted operations". (README) For autonomous agents this matters: an agent never gets stuck mid-rebase.

**Pijul**: https://pijul.org
- Commutation: "Independent changes can be applied in any order without changing the result or the version identifier." (pijul.org)
- Conflicts are resolved once: "Once resolved, conflict resolutions applies to the same two changes everywhere. Conflicts never come back." (pijul.org)
- Patch identity is permanent: "All patches retain their identities permanently regardless of their context, order, operations performed, or team workflow" **[secondary: initialcommit.com]**, consistent with pijul.org's statement above.
- Theory: Mimram & Di Giusto, "A Categorical Theory of Patches", arXiv:1311.3903 (cited from Pijul's own theory notes).
- Pijul's author argues that jj-style tools "are UIs written on top of Git… they inherit all the issues of not having proper theoretical foundations" (pmeunier on Lobsters) **[secondary; an interested party]**. jj's docs claim it handles conflicts as first-class objects and "lets Jujutsu rebase merge commits correctly (unlike both Git and Mercurial)". These are competing claims from the two projects. Neither is independently benchmarked here.

**ngit / NIP-34** (Git collaboration over Nostr): https://codeberg.org/DanConwayDev/ngit-cli
- Already written in Rust (the repo shows "Rust 99.6%"). It publishes repo announcements, branch state, proposals and issues as signed Nostr events, but "a git server is still required for data storage and syncing state." (ngit-cli README)

### Recommendation [analysis]
- **Don't reimplement storage, diffing and merging.** Signed agent provenance needs change identities that stay stable. Pijul gives that natively. jj gives it through stable change IDs (stored as non-standard Git commit headers that "[are] not preserved through a rebase operation" by plain Git; https://docs.jj-vcs.dev/latest/git-compatibility/).
- **Practical path:** write Biblio as a Rust application that uses `jj-lib` (or `libpijul`) for history, plus its own **Biblio ledger** for intents and agent lifecycle records. Keep Git interop through jj's Git backend so users can still push to GitHub or Codeberg. Write a custom storage backend later only if a real limit shows up. jj's trait design means that wouldn't require a fork.
- **Risk of going fully standalone:** you'd have to rebuild merging, conflict handling, network transport and hosting, and adoption would suffer. Pijul has had years of the same uphill battle as a non-Git VCS.

---

## Sub-question 3: Can Nostr provide verifiable agent identity, including verified sunsetting?

### What exists
- NIP-34 defines repository announcements (kind 30617), repository state (kind 30618), patches, PRs and issues (kinds 1617, 1618, 1621). (https://nips.nostr.com/34)
- The Rust `nostr` crate (v0.45.5) implements the protocol (https://crates.io/crates/nostr).

### What doesn't work, and why it matters for this design
1. **Delegation (operator key → agent key) is deprecated.** The official NIPs index lists "NIP-26: Delegated Event Signing --- unrecommended: adds unnecessary burden for little gain" (https://github.com/nostr-protocol/nips). The Rust crate marks it deprecated too: "NIP-26 is deprecated" (https://docs.rs/nostr/latest/nostr/nips/nip26/).
2. **Expiry can't be enforced through timestamps.** From the deprecation PR: "The expiration dates don't work because as soon as you delegate the key, the sub key can always post in the approved period forever (people can post in the past)." (https://github.com/nostr-protocol/nips/pull/1051) The same applies to a "sunset" event: a leaked or retained agent key can sign backdated events afterwards.
3. **Revocation is unsolved.** The NIP-41 key-migration draft calls itself "a simple, best-effort, not guaranteed, way in which a user can notify their followers that they have migrated to a new public key" (https://github.com/nostr-protocol/nips/blob/pf7z-nip41/41.md, draft branch, not merged). An earlier NIP-41 draft aimed only to "reduce the damage from compromised keys from catastrophic to just very bad" (https://github.com/nostr-protocol/nips/pull/158).
4. **Timestamp anchoring is itself flagged.** NIP-03 (OpenTimestamps) is listed as "unrecommended: vulnerable to one specific attack, needs update" (NIPs index).

### Recommended design [analysis]
- **The order of events comes from a Biblio ledger**: an append-only, hash-chained log in which each entry commits to the previous one, like jj's operation chain.
- **Birth entry:** the parent or operator signs `{agent_pubkey, config_hash, mandate/intents, parent_ids}`. The agent counter-signs to prove it holds the key.
- **Sunset entry:** the agent and its operator co-sign `{final_state_hash, outcome evidence}`. Any event signed by that key that isn't chained before the sunset entry is **invalid by rule**, whatever its `created_at` says.
- **Publish the ledger to Nostr relays (and optionally a public transparency log)** so a single host can't quietly rewrite history. Nostr is used for distribution and discovery, not for ordering.
- **Be honest about what "verified sunset" means.** It means "verified that nothing signed later counts", not "verified the key no longer exists". Short-lived keys that exist only in memory reduce the risk but can't be proven deleted. Hardware or TEE attestation could strengthen this; I didn't research it.

---

## Sub-question 4: How should agent life stories, lineage and "descendants" be represented and verified?

### Format: in-toto attestations rather than an invented format
- "The in-toto Attestation Framework defines a fixed, lightweight 'Statement' that communicates some information about the execution of a software supply chain… The information is communicated as a 'Predicate' using a context-specific schema identified by a 'Predicate Type'." Custom predicates are explicitly supported. (https://slsa.dev/blog/2023/05/in-toto-and-slsa)
- Proposed predicate types **[analysis]**: `biblio.dev/agent-birth/v1`, `agent-task/v1` (subject = digest of the change set or intent realised), `agent-evaluation/v1`, `agent-sunset/v1`, `agent-descent/v1` (parents plus the evidence used to select them).
- Signatures: sign exact bytes in a DSSE-style envelope, as in-toto does, rather than re-serialised JSON.

### What attestations can and can't prove
- SLSA's own trust model says the signer has to be trusted: "The builder is trusted to have faithfully recorded the provenance; there is no option but to trust the builder." (https://slsa.dev/spec/v0.1/provenance)
- SLSA Build L3 requires the platform to "prevent secret material used to sign the provenance from being accessible to the user-defined build steps" (https://slsa.dev/spec/v1.2/build-provenance; level wording quoted via a secondary source). Applied to agents: **the agent must not hold the key that signs its own evaluation.** One vendor makes the same point: "In an agent run, the untrusted part is the agent itself… indirect prompt injection means that content can steer it." **[secondary, vendor blog: traceseal.io]**

### Descendants and blends [analysis; no direct source]
- An agent's "qualities" in practice are: model ID and version, system prompt, tool and skill set, memory or retrieval corpus, and settings. Hash this as a **configuration manifest**.
- A **descendant** is a new manifest whose descent attestation lists one or more parent manifest hashes and the evaluation attestations that justified choosing them. Lineage becomes a DAG that can be queried like commit history.
- **Blending** is only real for parts that can be combined (prompts, skills, memory). It isn't real for model weights unless you actually fine-tune or merge them, and that should be recorded as its own build step with provenance.
- **Goodhart risk:** selecting for performance optimises for whatever the metric measures. Scores should come from independent verifiers, such as the existing suite-sensitivity and mutation checks or held-out tests, signed by a key the agent can't reach.
- Include **negative lineage**: failed or revoked agents stay in the record so later descendants don't inherit known-bad traits.

---

## Sub-question 5: Is "100% Rust, auditable down to the compiler" achievable?

### Evidence
- **Trusting-trust risk:** "the Rust compiler requires the previous Rust compiler binary, making it vulnerable to persistent malware… diverse double compilation is available. Unfortunately, it cannot be easily applied in this case. For one, the Rust compiler is not deterministic." (https://github.com/rust-lang/rust/issues/48707, a 2018 report; reproducibility has improved since)
- **The independent compiler is C++ and outputs C:** mrustc emits "a high-level assembly (currently very ugly C…)" compiled by gcc, and is "currently tested to fully bootstrap (with a binary-equal 1.91.1) version 1.90.0". (https://github.com/thepowersgang/mrustc)
- **The full bootstrap is expensive and hasn't yet converged with official releases:** the goal is that "once the reproducibility of the Rust compiler is in better shape (rust-lang/rust#129080) we can get this bootstrap chain to converge with the chain that underlies the official rustc releases"; "Bootstrapping up through rustc 1.93 requires 310 GB." (https://github.com/dtolnay/bootstrap)
- **Reproducibility is tracked as ongoing work:** rust-lang/rust#129080 is the "Tracking Issue for Reproducible Build bugs and challenges", labelled "S-tracking-forever: Never to be closed". (https://github.com/rust-lang/rust/issues/129080) Reproducibility of rustc on Debian has gone back and forth between releases (https://github.com/rust-lang/rust/issues/75362).
- **For applications (not the compiler), things are better:** "Programs written in Rust are often already reproducible by default, given the original build toolchain is version-matched and build paths are normalized." (https://reproducible-builds.org/docs/rust/)
- **Even the Nostr stack isn't pure Rust:** crates.io API, checked 2026-10-26:
  - `nostr` 0.45.5 depends on `secp256k1`.
  - `secp256k1` is a "Rust wrapper library for Pieter Wuille's `libsecp256k1`" (C).
  - `k256` 0.14.0 is "written in pure Rust with support for… Taproot Schnorr signatures (BIP340)", the scheme Nostr uses.

### Recommendation [analysis]
Aim for **"reproducible and bootstrappable"**:
1. 100% Rust in Biblio's own code, with `#![forbid(unsafe_code)]` where possible.
2. Pure-Rust cryptography (`k256` for BIP-340), with a thin Nostr event layer written on it or a `nostr` fork, to avoid the C library.
3. Pinned `rust-toolchain.toml`, committed `Cargo.lock`, `--locked` builds, path remapping, and reproducible release builds checked by two independent builders.
4. `cargo vet` / `cargo deny` for dependency auditing.
5. Optionally, a release built by an mrustc-bootstrapped toolchain for people who want the strongest assurance.
6. Have Biblio verify itself: each Biblio release ships an in-toto build attestation recorded in its own ledger.

---

## Implementation steps [analysis]
1. Write the **Biblio data model spec**: intent, work, sprint, agent manifest and the five attestation predicates, all in canonical serialisation.
2. Write the **ledger crate**: hash-chained entries, pure-Rust BIP-340 signing (`k256`), and validation rules (including invalidating anything signed after sunset).
3. **History layer:** wrap `jj-lib` with the Git backend. Map Biblio change records to jj change IDs.
4. **Port the sprint loop** (Research → Plan → Build → Test → Loop) from shell to Rust. Phase is derived from ledger evidence, as it is today.
5. **Verifier service:** runs tests and sensitivity checks and signs evaluation attestations with a key the agent can't access.
6. **Nostr publication:** mirror ledger entries as events; optionally interoperate with NIP-34 / ngit for repo announcements.
7. **Lineage tooling:** query and visualise the agent DAG; spawn descendants from selected manifests.
8. **Reproducible release pipeline** as in sub-question 5.

---

## Confidence & gaps

**High confidence (primary sources):**
- NIP-26 deprecation and the backdating problem.
- NIP-41 being best-effort and unmerged.
- NIP-03 being flagged.
- jj's architecture, op log and Git-backend status.
- mrustc and the dtolnay bootstrap facts.
- The rustc reproducibility tracking issue.
- The `nostr` → `secp256k1` (C) dependency and `k256` being pure Rust.

**Conflicting or contested sources:**
- **Pijul vs jj on theoretical soundness.** Pijul's author says Git-based tools lack proper foundations; jj claims correct conflict and merge-commit rebasing. Both are interested parties. I lean towards "Pijul has the cleaner model, jj has far more adoption and Git interop" (jj-vcs/jj has about 32k GitHub stars per its org page), but no neutral benchmark was found.
- **Rust reproducibility.** The 2018 issue says rustc isn't deterministic; reproducible-builds.org says Rust *programs* are often reproducible by default. These don't contradict each other (compiler vs applications), but the current state of rustc itself is a moving target. #129080 is a never-to-close tracker, not a status report.

**Weaker sources, no stronger source found:**
- The Pijul patch-identity wording (initialcommit.com; consistent with pijul.org).
- The agents-are-the-untrusted-step argument (traceseal.io vendor blog).
- The SLSA Build L3 wording, taken from secondary summaries (traceseal.io / dev.to) rather than read in full in the SLSA v1.2 spec.
- The ~32k-star figure for jj (GitHub org page snapshot) and the "uphill adoption battle" remark about Pijul (my own judgement, no source).
- Read these as informed opinion.

**Not covered:**
- W3C DIDs and Verifiable Credentials, Sigstore keyless signing, and hardware/TEE attestation as alternatives or additions to Nostr identity.
- Any existing standards for AI-agent identity.

**Naming:**
- The crate name `biblio` is **already taken** on crates.io: a bibliographic-format parser, v0.1.0, created 2026-03-22 (crates.io API). Pick a different crate name or a namespaced form (e.g. `biblio-vcs`, `animus-biblio`).

**Pure analysis (no source):** the ledger-based sunset rule, the configuration-manifest model of descendants, and the recommendation to build on jj/Pijul rather than a new VCS.
