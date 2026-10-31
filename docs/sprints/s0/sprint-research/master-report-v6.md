# Animus_Periapsis: Master Report (v6: + Round-5 cheat-resistant proofs, intent type theory, RR pipeline)

*A from-scratch, agent-human hybrid distributed version control system in Rust, with an information-theoretic (entropy) layer.*
*Date: 2026-10-31. **v6: Part I updated with Round-5 findings (Part VIII, tagged R5).** **v3: Part I updated with Round-2 experimental findings (Part V, tagged R2-E#). v2: corrected after experimental validation (Part IV).** In-text "(E#)" tags refer to Part IV experiments. Parts II and III are kept as originally written, with inline correction notes where a claim was refuted or changed. This report combines the Deep Research Report (v3) and the Microstates and Entropy Report into one document. Both appear in full as Parts II and III, with section numbers prefixed II. and III.; Part I is the combined synthesis and the single plan to work from.*

---

# PART I: Combined Synthesis

> **v6 status (Round 5, Part VIII):** Proof pairing is now **cheat-resistant**.
> - **Red team.** A 7-layer verifier plus a human-signed `book.lock` rejected **all 13 red-team cheats** and accepted all 4 honest controls. The cheats covered weakened statements, shadowed names, a kernel-bypassing metaprogram, `@[csimp]`/`implemented_by` + native evaluation, forged evidence ledgers and change records, and **weakening the Book itself**. The common gate ("builds + `#print axioms` clean") accepted 3 of 8 proof-level cheats.
> - **Ablation.** It found a silent bug in our own verifier: `@[csimp]` was invisible under `loadExts := false`. Now fixed, fail-closed. It also showed that the axiom audit and statement pinning are single points of failure, so they get independent second implementations (comparator, nanoda).
> - **Intents have a type theory.** There are 5 modalities: formal / empirical / preference / judged / asserted. Non-formal evidence is a decidable proposition about a **verifier-owned, signed ledger**: pre-registration, blinding, independence and completeness are enforced, with exact binomial statistics checked by the kernel. An aesthetic intent was certified (strength *judged*, risk 5.1%), a regression was kernel-refuted, and 9 research-misconduct patterns are proved uncertifiable.
> - **Lanes → RRs → dev → main.** Lanes are workspaces, an RR is a signed event, dev is a derived speculative view, and main is the public phase. In simulation on real change shapes, module-disjoint batching with a Zuul-style adaptive window lands RRs **2.0–2.2× faster than serial with 3.2–3.3× lower latency and 0 main-branch breaks**. Evidence tiers no longer drive batching, only which checks are required and how review is routed.

> **v5 status (Round 4, Part VII):** **Proof pairing is kept, in tiered form, as the formal backbone of the Book.** Every change carries a certificate with (a) local claims and (b) intent claims. Intents are Lean propositions (or opaque prose propositions) whose support is recorded in an *evidence calculus* (`Status`: proved / tested / asserted / reviewed decomposition). The verifier recomputes each intent's evidence profile from the Lean kernel and rejects `sorry` and unregistered axioms. The Lean 4 prototype (`prototypes/peri-proof`) proves change c1 (canonical op-head merge) discharges the convergence intent, proves the old code violated it, catches a buggy later change c2 and localizes it to the intent it breaks, and differential-tests production code against the proved spec (0 vs 3,946 disagreements in 20,000 inputs). Local proofs are required for Periapsis's core algorithms and optional, as the top tier, for user code.

> **v4 status (Round 3, Part VI):** **Specification entropy is removed.** In 127 real bugs across 9 projects it added no predictive value beyond free size/complexity metrics (held-out accuracy 0.738 → 0.736 when added). Also removed as unnecessary: hunk surprisal, Shannon H_heads, decision bits, selection/throughput accounting, the per-op Landauer `bits_destroyed` field (the reversibility *principle* stays), separate evolution-marker objects (derived from rewrite ops), the HEAD file, the CRDT framework for records (immutable event records instead), RBSR as a protocol (DAG negotiation is the single sync mechanism), content-defined chunking in core (predecessor-prefix zstd instead), the patch-theory core, Nostr, and the "entropy layer" as a layer. What survives of the entropy work: information-gain test ordering (validated) and reversible-by-default storage. The tables in I.2–I.4 are kept as historical rationale; **I.5 (the plan) is current.** The slimmed-down architecture is in Part VI §5.

## I.1 What we're building, in one paragraph
Animus_Periapsis is a snapshot-based, content-addressed DVCS, written from scratch in Rust. It takes proven ideas from Jujutsu, Mercurial/Sapling, Mononoke, Radicle, Git, Pijul, Xet and Negentropy, and implements them without depending on any of those projects' code. On top of that sits an **agent-native layer**: signed actors, intents (CubiKan), confirmation roots (array-test), and Sprint Loops' rule that state is *derived from evidence*. Under that sits an **entropy layer**. Each repository state is treated as a microstate, and every operation, conflict, test and agent selection is measured in bits. The two research reports describe one system from two angles. The first covers *what to build and from which prior art*. The second covers *how to measure, verify and govern it precisely*.

## I.2 How the two reports fit together
The systems research and the entropy research arrive at the same design decisions independently. That agreement is the strongest argument for the architecture.

| Design element (Part II) | Entropy interpretation (Part III) | What the combination adds |
|---|---|---|
| Append-only **op log** + content-addressed objects (jj) | Bennett's **history tape**: repo transitions are injective, so they destroy 0 bits, *provided every working-copy-overwriting op snapshots first* (validated, E2) | **Landauer ledger**: every op records `bits_destroyed_ub` (an upper bound); only signed erasure ops, including op-log abandonment, record more than 0 |
| Undo / `op restore` | Reversibility follows from injectivity | Undo is guaranteed by construction, not just a feature |
| **Derived data** framework (Mononoke/Scion) | Data D is derived iff H(D \| authoritative, algorithm id, version) = 0 (E11) | A testable invariant: delete all derived data, recompute *under the same algorithm version*, assert byte-equality (DST) |
| **Phases / macrostates** (Mercurial phases, Sprint Loops derived phase) | Macrostate maps M: 𝒮 → 𝓜 (coarse-graining) | Phase, intent status and confirmation are all *functions of the microstate*, never stored pointers |
| **Conflict terms** stored as data (jj) | A conflict is a macrostate with k compatible resolutions: log₂ k bits undecided | **Decision-bit attribution** by Shapley value across sources (order-independent; may be negative when new candidates add entropy) (E5) |
| **ResolutionRecords** (rerere done properly) | Decision entropy already paid | **Measured: 69–75% of conflict blocks recur under repeated rebase (R2-E28)**, so reuse answers about 70% of decisions; with no silent-correctness gap found in 5,325 real cases (R2-E19/21), the Pijul patch core (format v2) is deprioritized |
| **Evolution markers** (Mercurial evolve) | A second-order history: rewrites stay injective | Rewrites never destroy bits; the provenance of rewrites is auditable |
| **Confirmation gate** (array-test root, verifier key) | Tests are measurements that partition program space | **Information-gain test ordering** (FLINT/EntBug) plus **specification entropy S_spec = log₂(1+D)** (corrected definition, E3) as the gate for agent code |
| **Multi-agent workspaces**, lock-free concurrency | The agent swarm as a Maxwell's demon; promoting any acceptable candidate costs ≥ log₂(N/a) bits (E6; log₂C(N,k) is only the specific-subset cost) | A **verifier-throughput law**: load depends on the acceptable fraction among candidates that survive verification, and effort ≈ 2^bits, so invest in verifier precision rather than worrying about agent count; divergence budget on **log₂(head count)** (E13). Also invest in high-information tests; **divergence budget** H_heads |
| **DAG want/have negotiation** for op log/objects (R2-E26: 10–30× fewer bytes than RBSR); **RBSR for flat sets only** | Mutual information between peers; cost scales with the difference | Sync-cost property tests: bytes ≤ c·d·log₂(n/d) + overhead for RBSR, with full-transfer fallback at large d/n (E9) |
| **Per-path reference compression** (zstd, predecessor as dictionary) for text; **FastCDC + BLAKE3** for large/binary files only | Delta size is bounded below by H(new \| old). On helix history: predecessor-dictionary zstd 3.5–4.4 MB vs git 8.5 MB vs best CDC 25.4 MB (R2-E27b) | Storage targets stated against measured baselines |
| **Ledger, negative lineage** (Biblio-era design) | The demon must keep its records; erasing them is where the cost lies | A reason in theory never to erase rejected candidates or evaluations |
| **Deterministic simulation testing** | Entropy metrics must be reproducible functions of the state | All entropy metrics are pure functions over derived data, so they can be checked under DST |
| **Agent Trace / EU AI Act Art. 50** provenance | Line attribution plus decision bits | Richer machine-readable provenance: who wrote it *and* who decided it |

## I.3 Unified design principles
1. **The authoritative state is reversible; erasure is explicit, signed and counted.** (Op log, evolution markers, Landauer ledger.)
2. **Everything else is derived, and derived means recomputable with zero information.** (Bonsai changesets + derived-data engine + the H(D|A)=0 invariant.)
3. **Macrostates are functions, never pointers.** (Phases, intent lifecycle, sprint phase, confirmation status.)
4. **Conflicts and choices are data, with entropy.** (Conflict terms, candidate sets, decision bits.)
5. **"Merged" means "measured".** (Confirmation roots; information-gain testing, validated on a real suite in R2-E16; S_spec *advisory* until validated, per R2-E24; verifier keys agents can't reach; structured/loose auto-merges always pass build/test, since R2-E22 found 2/45 duplicated moved blocks.)
6. **Concurrency without locks; bounded divergence.** (A single canonical n-way merge of all op heads, with results in a normal form (E12); divergence policies; a budget on log₂(live head count).)
7. **Every metric declares its microstate set or probability model**, and agents can never sign their own evaluations. (Defends against Goodhart's law.)
8. **Spec before code; hash agility; DST from day one.** (Lessons from Git's SHA-256 transition, TigerBeetle, turmoil.)
9. **Credit prior art explicitly.** (`docs/prior-art.md`, given the Scion/Mononoke attribution dispute.)

## I.4 Unified architecture
```
┌──────────────────────── Interfaces ─────────────────────────┐
│ peri (CLI/TUI) · peri --json · peri-mcp (agents) · Book (mdBook)   │
│ peri entropy (S_spec, change entropy, decision bits, H_heads, ledger)│
├──────────────────────── Agent-native layer ─────────────────┤
│ Actors & keys · Intents · PCOs (CRDT) · Phases & gates · Verifier  │
│ Confirmation roots · Ledger attestations · Agent Trace export       │
├──────────────────────── Entropy layer (peri-entropy) ───────┤
│ Landauer ledger · conflict/decision entropy · S_spec · info-gain    │
│ test ordering · change entropy · surprisal · selection accounting   │
├──────────────────────── History layer ──────────────────────┤
│ Bonsai changesets · change IDs · evolution markers · op log/views   │
│ conflict terms · resolution records · query language · derived data │
├──────────────────────── Content layer ──────────────────────┤
│ blobs (per-path ref-compressed) · CDC for large/binary · trees      │
│ histogram diff3 · resolution store · structured merge · confirm     │
├──────────────────────── Storage & sync ─────────────────────┤
│ op files/log (heads derived) · fsck · DAG negotiation · RBSR (flat) │
└──────────── all I/O, time, rng behind traits → DST ─────────────┘
```
The entropy layer is **pure**: it reads derived data and the op log, and computes values that are themselves derived data. It never changes authoritative state, apart from the `bits_destroyed` field the op log records at write time.

## I.5 Unified execution plan (in dependency order, no timelines)

### Phase 0: Specification and scaffolding
1. Rename the repo to `Animus_Periapsis`; update the README; add `docs/adr/`, `docs/prior-art.md`, `docs/spec/`.
2. **Format spec v1:** canonical encoding, `HashRef = (alg_id, digest)`, signatures that name their algorithm, version bytes, and the **`bits_destroyed` field on Operation**.
3. **Concurrency spec:** op/view merge and divergence policies; model-check them.
4. **Entropy spec:** a definition for every metric (microstate set or model declaration, units, and how it is computed).
5. Workspace crates: `peri-hash`, `peri-encoding`, `peri-store`, `peri-diff`, `peri-merge`, `peri-graph`, `peri-oplog`, `peri-wc`, `peri-derived`, `peri-entropy`, `peri-pco`, `peri-ledger`, `peri-sync`, `peri-cli`, `peri-mcp`, `peri-sim`.
6. `peri-sim` (filesystem, clock, RNG and network traits; crash injection); toolchain pin; `forbid(unsafe_code)`; `cargo deny`/`vet`; fuzz and property harnesses.
7. Reserve prefixed crate names (bare `periapsis` is taken).
7b. **Proof pairing foundations (R4):** pin the Lean toolchain; Book intent records = prose + optional Lean proposition (edits need a human signature); evidence-calculus library (`Status`); certificate format and gate (reject `sorryAx` and unregistered axioms; recompute evidence profiles).
7c. **Cheat-resistant verifier (R5):**
   - a 7-layer gate (import policy, raw extension-entry allowlist, attribute policy, kernel replay into a Book-only environment, axiom audit, statement pinning, human-signed `book.lock` + `allowed_signers`);
   - a planted-regression probe suite (every layer must be shown able to fail) as a required CI job;
   - the **intent-calculus library** (`Spec` modalities, `Certified`/`Refuted`, exact kernel statistics), which supersedes `Status` data for qualitative intents.

### Phase 1: Reversible single-user core
8. Object store (per-path reference compression for text, CDC for large/binary; index; atomic commit; `fsck`), verified under DST with crash injection and negative controls (prototype: `prototypes/peri-store`, 0 violations / 2,600 scenarios). **No HEAD file**: op heads derived from the checksummed op log (R2-E23b). Init fsyncs the directory (bug found by R2-E23).
9. Working copy with auto-snapshot; changesets and change IDs; histogram/Myers diff.
10. Op log and views; `undo`; `op restore`.
11. **Reversibility by default:** enforce *snapshot-before-overwrite*; erasure (GC, truncation, op-log abandonment) only via explicit signed erasure ops that record erased object IDs and bytes. *(v4: the per-op `bits_destroyed` field is removed as redundant with op type; R3 §4.1.)*
12. **Derived-data invariant test** under DST (delete everything derived, recompute under the same algorithm version, assert byte-equality); derived data keyed by algorithm id and version.
13. **Exit gate:** Periapsis tracks its own source; crash suite (with negative controls) passes; decoder fuzzing is clean; no non-erasure op loses information (snapshot-before-overwrite property test).

### Phase 2: Rewriting, conflicts, decision entropy
14. diff3 conflict terms; rebase that never fails; automatic rebase of descendants.
15. Rewrite ops record old → new mappings (evolution markers are a *derived index*, R3-E); `squash`, `split`, `absorb`, `restack`; divergence detection from derived markers.
16. ResolutionRecords (key: full conflict block) with replay; **resolution-reuse savings metric** (expected about 70% of repeat decisions, R2-E28). Merge core: histogram diff3 (prototype `prototypes/peri-merge`, 99.75% agreement with git, R2-E20); optional loose-adjacency mode (−12–14% conflicts, 0 errors observed, R2-E29), gated by confirmation.
17. Conflict attribution: record which actor resolved each conflict block, and how (manual, resolution store, structured merge). *(v4: entropy-based decision bits removed, R3 §4.1.)*
18. Query language over changes, actors, intents, phases and conflict state (e.g. `resolved_by(human) & conflicted()`).
19. Derived-data engine (trees, per-file history, segmented graph).
19b. **Tier-P proofs for core algorithms (R4):** canonical op-head merge (prototype done), conflict-term normal form, reversibility, derived-data invariant, phase-gate monotonicity; Lean executable specs used for differential testing of the Rust implementations (tier C).
19c. **Negative-witness registry (R5):** every formal Book intent must kernel-reject its registered mutants (known-bad implementations); every Book edit shows a mutation-score diff, and a drop needs a signed justification.
20. **Exit gate:** concurrency model checker with N writers; no lost commits; a canonical n-way op-head merge that is order-independent, criss-cross-correct and grouping-independent in normal form (E12); decision-bit attribution order-invariant (Shapley) with a complete ledger (E5).

### Phase 3: Agent-native layer plus measured confirmation
21. Actors and keys; signed ops; intents, work, sprints and reviews as **immutable event records in the op DAG** (no CRDT framework: per-record events remove all spurious conflicts while surfacing genuine same-record collisions, R3-D).
22. Book projection (derived view of event records).
23. Phases and gates; verifier service; array-test confirmation root as an attestation.
24. *(v4: specification entropy removed. No predictive value beyond free size/complexity metrics in 127 real bugs, R3 §2. Mutation testing stays only as array-test's optional test-improvement advisor.)*
25. **Information-gain ordering** for confirmation re-runs and bisection.
26. *(v4: change entropy demoted to an optional analytics plugin; surprisal removed, R1-E8.)*
27. Divergence budget on live-head count per intent; operational dashboards (candidates, verifier pass rate, review time). *(v4: selection-information accounting removed.)*
28. **Every change carries a proof-pairing certificate (R4)**; the Book view renders each intent's evidence profile (PROVED / TESTED / ASSERTED). Confirmations as signed op records (DSSE/in-toto only as an export format); optional Agent Trace exporter; `peri-mcp`.
28b. **RR pipeline (R5):**
   - lanes = workspaces over one op-log;
   - Review Requests = immutable signed events keyed by change IDs;
   - dev = a derived speculative-integration view;
   - **Gate 1** (per RR): full proof-pairing gate + confirmations + ledger checks; tier and rollup class are computed here, never claimed;
   - **Gate 2** (dev → main): revset-style filter queries select batches, packed module-disjoint, sized by an AIMD window, bisected on failure, promoted by one signed op; human review per batch, grouped by intent.
28c. **Evidence ledger service (R5):** writes signed trial, blinded-ballot and rubric-score records with sequence numbers; generates the trusted ledger module whose hash is pinned in `book.lock`.
29. **Exit gate:** the R5 red-team suite (all cheats rejected, honest controls accepted) passes, and ablation shows ≥ 2 independent layers for every attack class; Sprint Loops runs a full cycle on Periapsis with three or more agents and no Git; concurrent record updates produce no spurious conflicts; the sprint closes on green confirmations and a divergence head count under budget.

### Phase 4: Distribution
30. Staged sync: verify identity and signed pointers, then **DAG want/have negotiation** (the single sync mechanism, for ops, changesets, event records and per-signer attestation chains), then fetch objects. Optional RBSR over very large head sets only (R3-B).
31. Per-path predecessor-prefix zstd for all blobs, text and binary (R2-E27b, R3-C); keyframes at most every 50 versions. CDC only as an optional extension (cross-file dedup, files above 2 GiB).
32. **Sync-cost property tests**: DAG negotiation bytes ≈ 32·d + O(heads); rounds bounded by exponential batching (R2-E26).
33. Git importer and exporter.
34. **Exit gate:** a three-peer DST with partitions, a hostile peer, and protocol fuzzing.
34b. **Production verifier (R5):** agent builds only in a sandbox; export with `lean4export`; replay with the Lean kernel **and** nanoda; comparator as an independent statement/axiom check; toolchain pinned at ≥ 4.33.1 (after the 2026 kernel-bug fixes).

### Phase 5: Scale and research
35. Optional server mode (caches, virtual checkouts, pushrebase).
36. Structured (AST) merge as the second merge stage (Mergiraf-style resolves 34–40% of real diff3 conflicts, R2-E22), with **move detection** to avoid duplicated blocks and mandatory build/test confirmation; evaluated by human decision bits saved.
37. *(v4: patch-theory core removed from the roadmap, R2-E19/21/28; revisit only on real multi-agent evidence.)*
38. Empirical validation with real multi-agent telemetry: conflict and recurrence rates, structured-merge error rate, reviewer effort.
39. Reproducible, attested releases recorded in Periapsis's own ledger.
40. Aeneas extraction of core Rust modules to Lean for tier-P proofs about the actual code (R4).
41. Authority logic (Abadi-style "says / speaks-for") for judge panels, delegations and signer sets; kernel replay of agent-defined inductive types (R5).

## I.6 Unified risk register
| Risk | Mitigation |
|---|---|
| Scope of a from-scratch VCS | Vertical slices; self-hosting after each phase; spec first |
| Data loss | DST + crash injection + `fsck` + fuzzing; assertions on in production; the Landauer ledger flags any unexpected bits destroyed |
| Format lock-in | Version bytes; hash agility; golden files |
| Concurrency edge cases | Formal spec; canonical n-way merge with a normal form (E12); model checking; divergence policies; head-count budget |
| Entropy metrics without a defined basis | Every metric stores its microstate set or model declaration (principle 7) |
| Agents gaming metrics (Goodhart) | Verifier-owned metrics; separate keys; combined signals; surprisal is never a gate on its own |
| Overclaiming physics | The energy is negligible (about 3×10⁻²¹ J/bit); claim information accounting only |
| Adoption without Git | Git bridge before any public release |
| Attribution and licensing | `prior-art.md`; adopt designs, not incompatible code |
| Agents cheating on proofs (weakened or shadowed statements, kernel bypass, compiler-trust smuggling) (R5) | 7-layer verifier with statement pinning; N-version the single-point layers (comparator, nanoda); planted-regression probes for every layer |
| Agents weakening the Book (the "delete the failing test" pattern) (R5) | Human-signed `book.lock` + `allowed_signers`; kernel-checked negative witnesses as tripwires; mutation-score diff on every Book edit |
| Lean kernel soundness bugs (e.g. CVE-2026-72844, the 2026 AI-found exploits) (R5) | Pinned patched toolchain; multi-kernel replay; sandboxed builds; exported terms instead of trusted `.olean`s |
| Gaming qualitative evidence (sock puppets, cherry-picking, post-hoc rubrics) (R5) | Evidence = propositions about a verifier-owned signed ledger; pre-registration, blinding, independence, completeness, exact statistics (S1–S9 proved uncertifiable) |
| Main-branch breakage from parallel promotion (R5) | Module-disjoint packing + bisection + AIMD window; post-merge sweep if file-level packing is ever enabled |

## I.7 Combined research agenda
1. A minimal-decision-bit merge pipeline for agent conflict distributions.
2. Snapshot-plus-reuse vs a commutative patch core: transplanted real history shows no correctness gap and about 70% reuse (R2); confirm with real multi-agent telemetry.
3. Does S_spec predict or reduce defects? (Toolz study null and underpowered, R2-E24; needs a multi-project SZZ study.)
4. Does reviewer effort track 2^(log₂(N_surv/a_surv)), and does verifier precision (false-pass rate) dominate agent count as the scaling lever?
5. Entropy-driven sprint exit criteria for Sprint Loops.
6. Concurrency: actor-aware divergence policies and their decision-bit cost.
7. A literature check on whether "specification entropy" already exists under another name.
8. (R5) Calibrate the intent calculus with real human panels: inter-rater reliability; does certified preference evidence predict later user satisfaction?
9. (R5) Validate the RR-pipeline simulation with real multi-agent telemetry (interaction rates; the right disjointness unit: crate, module or build target).

## I.8 Combined confidence
- **High (primary sources):** the prior-art traits (jj, Mercurial/Sapling, Mononoke, Radicle, Git hash transition, Pijul claims as self-reported, FastCDC/Xet, RBSR, diff studies, DST); Landauer and Bérut; Bennett; Wolpert; Hassan; naturalness research; FLINT/EntBug.
- **Analysis (mine), now experimentally checked (Part IV):** the Landauer ledger (confirmed with conditions), the derived-data invariant (corrected), decision bits (attribution corrected), S_spec (definition corrected; additivity refuted), the throughput law (rewritten), H_heads (replaced), information-gain ordering (confirmed in simulation), Hassan change entropy (partially confirmed on 4 repos), surprisal (not replicated for bugs), RBSR (bound corrected), CDC locality (confirmed), op-head merging (new requirement).
- **Still analysis only:** the architecture, PCO design, phases and gates, phase ordering, and everything listed in Part IV §4.
- **Round 5:** the red-team results are high-confidence *for the attacks tested* (13 cheats, 4 honest controls, all reproducible); they don't prove completeness. The RR simulation is moderate-confidence: change shapes are real, failure and interaction rates are assumptions.
- **Not covered:** filesystem watchers; Windows and case-insensitive filesystems; submodules; trademark clearance for "Periapsis"; whether the `peri` command name is free; whether committed code falls within EU AI Act Art. 50 text marking.

---

# PART II: Systems Research and Architecture (full Deep Research Report v3)

## Animus_Periapsis: Deep Research Report and Execution Plan (v3)

*A from-scratch, agent-human hybrid distributed version control system in Rust.*
*Research date: 2026-10-31. Replaces v1 (built on jj-lib) and v2 (first from-scratch plan). Product name: **Animus_Periapsis** (short form "Periapsis"; proposed CLI `peri`, not yet checked for collisions).*

---

### Executive summary

Animus_Periapsis should be a **snapshot-based, content-addressed DVCS**. It adopts proven ideas from other systems and implements every one of them in this repository:

| # | Core decision | Borrowed from | Confidence |
|---|---|---|---|
| 1 | Lock-free **operation log** of "views"; concurrent ops are merged on the next load | Jujutsu | High (primary docs) |
| 2 | **Stable change IDs**, a working copy that is itself a change, conflicts stored as data | Jujutsu | High |
| 3 | **Evolution markers** (a second-order history of rewrites) plus **phases** | Mercurial evolve / Sapling | High |
| 4 | **Bonsai-style changesets** (changed files only, deterministic encoding) plus a generic **derived-data** framework | Mononoke | High |
| 5 | Commit **graph stored separately from metadata**; a segmented graph index | Sapling | High |
| 6 | Project records (intents, reviews, agent ledger) stored as **signed CRDT objects in the repo** | Radicle COBs, Fossil | High |
| 7 | **Ed25519 identities**, self-certifying repos, verify signed refs *before* downloading data | Radicle | High |
| 8 | **Range-based set reconciliation** for metadata sync; want-driven object fetch | Negentropy / Scion | High |
| 9 | **FastCDC + BLAKE3** chunking for large/binary files; text uses per-path reference compression (Part V R2-E27b) | Xet | High |
| 10 | **Histogram diff**; line merge, then a structured (AST) merge on the conflicting regions only | git/jgit, Mergiraf/Spork | High |
| 11 | **Hash agility** designed in from day one | Lessons from Git's SHA-256 transition | High |
| 12 | **Deterministic simulation testing** of storage and sync from Phase 1 | TigerBeetle, turmoil, madsim | High |
| 13 | A Pijul-style patch core goes on a research track for format v2, not into v1 | Pijul / Darcs | Medium |

What makes Periapsis different from these systems is the **agent-native layer**: signed actors, intents (from CubiKan), confirmation roots (from array-test), Sprint Loops' "state is derived from evidence" rule, and a JSON/MCP interface for agents. The surveyed systems lack this layer, and regulation is now pushing in this direction (§6).

---

### II.1. What the project is for (restated)

Periapsis combines the Sprint Loops Project Book (intents → work → sprints, with state derived from evidence), CubiKan's Intent Unit lifecycle, and array-test's Merkle DAG of confirmations into the version-control system itself. Every change answers three questions: **why** (intent), **who** (a human or a named agent with a key), and **is it proven** (a signed confirmation root). The system is built for many concurrent agents and remains readable by humans.

**Name:** in orbital terms the periapsis is the closest approach and the point of highest speed. It also matches the README's flyby imagery, since gravity assists happen at closest approach. In system terms: agents explore outward, and Periapsis is where their work converges into confirmed history.

---

### II.2. Deep findings by source system

#### II.2.1 Jujutsu: concurrency and change identity
- **How the op log works.** It is a DAG of operations, each pointing to a "view": the visible heads, bookmarks, tags, and the working-copy commit in each workspace. If a load finds multiple op heads, it 3-way-merges the views and records conflicts in the view itself (for example, a bookmark "moved from A to B or C").
- **Commands can't fail to commit.** Each command runs against a consistent snapshot and writes its operation with the start operation as parent. Detecting divergence is left to the next command, which simplifies every command. This is exactly the guarantee an agent needs.
- **Limits to design against.** jj warns that multi-machine concurrent modification "might conceivably lose some bookmark pointers", although commits are never lost. Divergence has to be resolved; jj's `converge` command for that is still experimental. **Periapsis implication:** make pointer conflicts first-class, and use change-ID divergence detection plus actor-aware auto-resolution rules (for example, "the human wins over the agent" or "the verifier-confirmed side wins").
- **Agents still conflict.** Independent reviewers point out that no VCS removes semantic conflicts; task partitioning and tests are still required. That is why the confirmation gate exists.

#### II.2.2 Mercurial evolve and Sapling: safe rewriting at scale
- **Obsolescence markers** form a second-order history of how changesets were rewritten. They sync without the obsolete changesets themselves and allow automatic handling of concurrent rewrites. The instabilities they detect are orphans, content-divergence and phase-divergence.
- **Sapling** applies the same idea at scale. Mutation tracking based on evolve rebuilds commit stacks after any edits, and the user-facing commands are `undo`, `redo`, `hide`, `unhide`, `restack` and `absorb`.
- **Separating graph from metadata.** The segmented changelog downloads the shape of the graph in a few MB and answers ancestry queries in O(number of merges). Commit metadata is fetched lazily.
- **Per-file history** makes log and blame O(changes to that file). Renames are recorded explicitly in file headers instead of guessed from content similarity, which avoids O(N²) heuristics.
- **Pushrebase:** the server rebases pushes onto the target bookmark, and fails only if the same file was edited.

#### II.2.3 Mononoke / Scion: changesets plus derived data
- Mononoke uses a canonical, VCS-neutral model (**Bonsai**) as its single source of truth. Expensive indexes are computed asynchronously, and the same backend serves both Sapling and Git clients.
- Bonsai specifics worth copying: ordered parents, sorted sets and maps for deterministic serialization, and no distinction between added and modified files.
- After a public attribution dispute, Anthropic credited Mononoke as having "pioneered many of the techniques we use in Scion, in particular the Bonsai data model". A Mononoke engineer alleges the copying was deliberate (unnamed source, not verified). Its reported results: worktree creation went from 7.1 minutes to 5.1 seconds, and total source-control time per change from 34 minutes to 3.7. **Lesson for Periapsis:** keep a `docs/prior-art.md` file crediting sources from day one.

#### II.2.4 Pijul / Darcs: the patch-theory option
- **The promise:** a merge is a categorical pushout, conflicted states are first-class, independent changes commute, and resolutions stay resolved. Pijul says it solved Darcs's exponential merge problem and works in time logarithmic in history for non-conflicting patches. Its argument against diff3: diff3 can reorder lines without reporting a conflict, and Git doesn't guarantee associativity.
- **The cost:** files become "graphs of lines" and the data structure is tied to the atomic patch operations. Pijul's adoption is small, and Darcs's history shows that patch systems can be opaque when things go wrong.
- **Decision:** v1 uses snapshots plus jj-style conflict terms plus content-keyed resolution reuse. The patch model goes on a format v2 research track behind the format version byte. Revisit it when the snapshot core is stable and test data from real agent workloads exists.

#### II.2.5 Radicle: sovereign identity and in-repo collaboration
- Repositories are self-certifying: updates are signed and verifiable without a trusted third party. Node ID equals public key. The implementation is a Rust workspace, with Ed25519 signing.
- **COBs:** issues and patches are stored in the repo as signed operation DAGs that merge as CRDTs. They are extensible via reverse-domain types (for example `com.acme.task`) with no protocol change.
- **Staged fetch:** verify identity and signed refs first, then download the data. This defends against hostile peers.
- **Periapsis implication:** intents, work items, sprints, reviews and agent-ledger entries become **Periapsis Collaborative Objects (PCOs)**, typed as `dev.animus.intent` and so on. Radicle builds this on top of Git; Periapsis makes it native.

#### II.2.6 Git: what to copy and what to avoid
- **Copy:** content-addressed Merkle objects, cheap snapshots, and integrity that propagates through history.
- **Avoid:** a single hard-coded hash. Git's SHA-256 transition design needs bidirectional SHA-1↔SHA-256 mapping tables, and SHA-1 and SHA-256 objects can't be mixed in one repo. Git 3.0 still has no release date, and the default switch waits on the ecosystem. Signed payloads don't name their hash algorithm, which creates ambiguity whenever two hashes have the same length. Scott Chacon's critique proposes separating object addressing from a strong signed tree hash.
- **Periapsis rule:** every hash is serialized as `(algorithm_id, digest)`; every signed payload names its algorithm; the format allows a repo to carry multiple hash algorithms during a migration.

#### II.2.7 Diff and merge algorithms
- **Histogram** diff: a jgit refinement of patience diff, with readability close to patience and faster in practice. A study found it describes code changes better than Myers. One optimized implementation reports it 10–100% faster than Myers, with a fallback to Myers for highly repetitive tokens.
- **Structured merge** (Mergiraf, following Spork): run a line merge first; on conflict, use tree-sitter parsing, GumTree matching, and a PCS-triple reconstruction restricted to the conflicting regions; fall back to diff3 when parsing fails.
- **Periapsis:** histogram as the default diff and Myers-minimal as an option, both written in-house with fuzzing. Merge pipeline: diff3, then a structured merge for supported languages, then a conflict term. Store zdiff3-style terms that include the base.

#### II.2.8 Large files and sync
- **FastCDC:** about 10× faster than Rabin-based CDC with nearly the same dedup ratio. Xet's spec uses about 64 KiB chunks hashed with BLAKE3.
- **RBSR / Negentropy:** logarithmic rounds, and it doesn't care how either side stores its data; its author argues Merkle trees should be kept for cases needing membership proofs. Deployed at scale in Nostr relays (NIP-77).
- **Periapsis:** *(corrected by Part V R2-E26: the op log, changesets and DAG-structured PCOs use DAG want/have negotiation, 10–30× cheaper; RBSR is kept for flat sets such as markers and attestations)* metadata sets (ops, markers, PCO operations, attestations) sync via RBSR. Content syncs via want-driven fetch from the heads (Scion), so caches can stack.

#### II.2.9 Testing a storage and sync system
- Deterministic simulation testing (DST): randomized whole-system runs reproducible from a seed. TigerBeetle simulates months of faults in minutes and enforces byte-identical storage determinism, with assertions on in production. turmoil provides a simulated network plus a crash-consistency filesystem shim; madsim overrides time and entropy. A known pitfall is randomized `HashMap` seeds.
- **Periapsis:** all I/O, clocks and randomness go behind traits from day one. Use `BTreeMap` or seeded hashers in any code that is serialized or affects ordering. DST runs on every PR.

---

### II.3. Architecture

```
┌────────────── Interfaces ──────────────┐
│ peri (human CLI/TUI) · peri --json · peri-mcp (agents) · Book renderer (mdBook) │
├────────────── Agent-native layer ──────┤
│ Actors & keys · Intents (CubiKan) · PCOs (CRDT) · Phases & gates · Verifier    │
│ Confirmation roots (array-test) · Ledger attestations · Agent Trace export     │
├────────────── History layer ───────────┤
│ Changesets (Bonsai) · Change IDs · Evolution markers · Op log + views          │
│ Conflict terms · Resolution records · Query language · Derived-data engine      │
├────────────── Content layer ───────────┤
│ Blobs · FastCDC chunk lists · Trees (derived) · Histogram diff · Merge pipeline │
├────────────── Storage & sync ──────────┤
│ Append-only pack log + index · atomic commit · fsck · RBSR · want-fetch · QUIC/SSH│
└──────────── all I/O/time/rng behind traits (DST) ───────────┘
```

#### II.3.1 Object model (format v1)
All objects are immutable, use canonical binary encoding, start with a `format_version` byte, and are addressed as `HashRef = (alg_id: u8, digest)`. The default algorithm is BLAKE3-256.

| Object | Contents |
|---|---|
| `Blob` | zstd-compressed bytes (small files) |
| `ChunkList` | ordered `(HashRef, len)` (FastCDC, large files) |
| `Changeset` | ordered parents · `change_id` (128-bit random) · sorted `path → {Add/Modify(HashRef, mode) \| Delete \| Rename(from)}` · `author: ActorId` · `intent_refs[]` · description · optional `conflict_terms` |
| `Operation` | parent ops · `View` HashRef · actor · wall clock (advisory only) · signature |
| `View` | heads · named pointers (with conflict states) · per-workspace working-copy change |
| `Marker` | predecessors → successors · kind (`rewrite`/`prune`/`split`/`fold`) · effect flags · actor · intent · signature |
| `PcoOp` | type (`dev.animus.intent`, `.work`, `.sprint`, `.review`, `.ledger`) · object ID · parent PcoOps · payload · signature |
| `Attestation` | DSSE envelope, in-toto-style statement (`confirmation`, `birth`, `task`, `eval`, `sunset`, `descent`) |

Trees, blame, path history, segments, intent↔change and actor↔change indexes are all **derived data**. They can be recomputed at any time and are never authoritative.

#### II.3.2 Phases and gates
`secret → draft → confirmed → public`
- **draft → confirmed** requires a confirmation attestation (an array-test root over exactly this change's tree), signed by a **verifier key that agents can't access**.
- **confirmed → public** follows per-repo policy, e.g. human approval for named intent species, or a linear trunk via pushrebase.
- Rewriting `public` history produces a phase-divergence error.

#### II.3.3 Concurrency semantics (spec this formally)
- Each op sees one consistent view and always commits. The next load merges divergent op heads.
- Pointer conflicts are stored in the view. Change-ID divergence is detected and resolved by **policy rules**, with interactive prompts for humans and structured results for agents.
- Agent workspaces are sparse or API-only (writes without a checkout).

#### II.3.4 Identity and the ledger
Per-actor Ed25519 keys (BIP-340 optional for Nostr publishing). The op log provides ordering, so `created_at` timestamps are never trusted. Sunset means "nothing signed after this op counts". Descendants are configuration manifests with descent attestations. Evaluation keys are kept separate from agent keys.

---

### II.4. Execution plan (in dependency order, no timelines)

#### Phase 0: Specification and scaffolding
1. Rename the repo to `Animus_Periapsis`; update the README; add `docs/adr/`, `docs/prior-art.md`, `docs/spec/`.
2. **Write the format spec v1** (§3.1): canonical encoding, hash agility, signature payloads that name the algorithm, and the version byte.
3. **Write the concurrency spec**: op/view merge rules and divergence policies. Model-check it with TLA+ or a Rust state-machine model.
4. Cargo workspace: `peri-hash`, `peri-encoding`, `peri-store`, `peri-diff`, `peri-merge`, `peri-graph`, `peri-oplog`, `peri-wc`, `peri-derived`, `peri-pco`, `peri-ledger`, `peri-sync`, `peri-cli`, `peri-mcp`, `peri-sim`.
5. `peri-sim`: traits for filesystem, clock, RNG and network, with deterministic implementations and crash injection.
6. Pin the toolchain; add `forbid(unsafe_code)`, `cargo deny`/`vet`, and fuzz plus property-test harnesses.
7. Reserve crate names under a prefix (bare `periapsis` is taken on crates.io).

#### Phase 1: Single-user snapshot core
8. Object store: append-only pack log, index, atomic commit, `fsck`, crash recovery verified under DST.
9. Working copy: scan with an mtime/inode cache, ignore rules, auto-snapshot into the working-copy change.
10. Changesets, change IDs, history graph; histogram/Myers diff; `log`, `diff`, `new`, `describe`, `edit`.
11. Op log, views, `undo`, `op restore`.
12. **Exit gate:** Periapsis tracks its own source; DST crash suite passes; decoder fuzzing is clean; round-trip properties hold.

#### Phase 2: Rewriting and conflicts
13. diff3 merge producing conflict terms; rebase that never fails; automatic rebase of descendants.
14. Evolution markers; `squash`, `split`, `absorb`, `abandon`, `restack`; orphan and divergence detection.
15. Resolution records (keyed by the hashes of the conflicting hunks) with automatic replay.
16. Query language over changes, actors, intents and phases.
17. Derived-data engine: trees, per-file history, segmented graph index.
18. **Exit gate:** a concurrency model checker with N simulated writers; deterministic op-merge results; no lost commits.

#### Phase 3: Agent-native layer
19. Actors and keys; signed ops and markers.
20. PCO framework (CRDT op DAG) with Intent (CubiKan port), Work, Sprint, Review and Ledger types.
21. Book projection: Sprint Loops docs rendered from PCOs (mdBook).
22. Phases and gates; verifier service; array-test confirmation root as derived data plus an attestation.
23. Ledger attestations; Agent Trace export (per-line attribution: human, ai, mixed, unknown).
24. `peri-mcp` / `--json`: never prompts, structured errors, sparse or API-only workspaces.
25. **Exit gate:** Sprint Loops runs a full Research→Plan→Build→Test→Loop cycle on Periapsis with three or more concurrent agents and no Git.

#### Phase 4: Distribution
26. Staged sync: verify identity and signed pointers, then RBSR for metadata sets, then want-driven object fetch.
27. Transports (SSH, QUIC); bundles; phase-aware push (never push secret).
28. FastCDC chunk storage and transfer.
29. Git importer and exporter, written against the Git object format spec, for GitHub mirroring.
30. **Exit gate:** a three-peer DST with partitions, a hostile peer, and protocol fuzzing.

#### Phase 5: Scale and depth
31. Optional server mode: horizontally scaled caches, virtual or sparse checkouts, server-side pushrebase.
32. Structured merge engine for Rust, TOML, JSON and Markdown first.
33. Format v2 research: a commutative patch layer, evaluated against real agent conflict data.
34. Reproducible, attested releases of Periapsis recorded in its own ledger.

---

### II.5. Risks and mitigations
| Risk | Mitigation |
|---|---|
| Scope (a full VCS from scratch) | Vertical slices; each phase self-hosts; spec before code |
| Data loss | DST + crash injection + `fsck` + fuzzing from Phase 1; assertions on in production (TigerBeetle approach) |
| Format lock-in | Version bytes, hash agility, golden-file tests |
| Concurrency edge cases (lost pointers, divergence) | A formal concurrency spec, model checking, and divergence handling by policy |
| Adoption without Git | A Git bridge before any public release |
| Attribution and licensing | `prior-art.md`; adopt designs, never code from incompatible licenses |
| Gaming of agent evaluations | Verifier keys separate from agents; independent tests |

### II.6. Regulatory tailwind (context, not legal advice)
EU AI Act Article 50 transparency obligations apply from **2 August 2026**. Providers of generative systems must mark outputs as AI-generated in a machine-readable form. The Commission says no single marking technique currently meets the standard, so a multi-layered approach is expected. Fines reach €15M or 3% of worldwide turnover. Final guidelines (20 July 2026) say that intermediate reasoning and machine-facing actions need not be marked. Whether source code committed to a VCS is "text content" within scope is **not settled by these sources**. Even so, per-line signed attribution in Periapsis (Agent Trace-compatible) is a credible way to provide machine-readable provenance.

### II.7. Confidence and gaps
- **High:** jj concurrency semantics; Mercurial evolve; Sapling and Mononoke design; Radicle COBs and identity; Git hash-transition facts; FastCDC, Xet and RBSR; diff algorithm findings; DST practices. Most of these come from primary docs or papers.
- **Medium:** Pijul's performance claims (self-reported); Scion's metrics (first-party, recapped by a secondary source); the readability benefit of histogram diff (one academic study plus practitioners).
- **Not researched:** filesystem-monitor integration (watchman-style), Windows and case-insensitive filesystem semantics, submodules or nested repos, internationalized paths, trademark clearance for "Periapsis", and whether the `peri` CLI name is free.
- **Pure analysis (no source):** the PCO design, the phase/gate model, divergence policy rules, and the phase ordering.


---

# PART III: Microstates, Entropy and the State Machine (full report)

## Animus_Periapsis: Microstates, Entropy and the State Machine

*Research report. Date: 2026-10-31. Companion to "Animus_Periapsis_Deep_Research_Report.md".*

**The question:** *If each state a state machine can occupy is a microstate, what can entropy do for us, made mathematically precise and applied to Periapsis?*

**Short answer:** it gives Periapsis five things, each a real calculation:

1. A **reversibility law for the VCS** (Landauer and Bennett). Every operation either keeps information or provably destroys a measurable number of bits. Periapsis can account for those bits and require a signature for each destructive operation.
2. A **precise definition of "derived data"**: data whose conditional entropy given the authoritative store is zero. Such data can be deleted at zero informational cost; everything else can't.
3. **Conflicts and agent candidates as macrostates.** Their entropy counts the decisions still owed, and lets Periapsis attribute *decision bits* to the human or agent who supplied them.
4. **Tests as measurements.** Test selection maximizes expected information gain. Mutation testing measures **specification entropy**: how many nearby programs your tests can't tell apart from yours.
5. **Empirically validated risk signals.** Change entropy (Hassan) and code naturalness / cross-entropy (Hindle, Ray et al.) are published fault predictors that become built-in derived data.

The physical energy argument (Landauer's kT ln 2) is real but irrelevant to cost at this scale (§8). What carries over exactly is the **information accounting**.

---

### III.1. The formal model

#### III.1.1 Microstates
Let the **authoritative state** of a Periapsis repository be

  σ = (O, L)

where O is the set of content-addressed objects and L is the operation log (a DAG of operations, each pointing to a view). In the v3 design, every operation *adds* objects and an op-log entry. That makes σ a point in an enormous but well-defined discrete space 𝒮. **This is the microstate.**

Other microstate levels are useful for specific measurements:
- **Working-tree microstate** w ∈ 𝒲: the bytes of the checked-out files.
- **Program microstate** p ∈ 𝒫: one specific program text. Used with mutation operators in §4.

#### III.1.2 Transitions
Every command is a function F: 𝒮 × X → 𝒮 (X = command and inputs). With the snapshot/op-log design, F is append-only:

  F(σ, x) = (O ∪ ΔO, L ∪ {op(x, parent = head(L))})

#### III.1.3 Macrostates (coarse-grainings)
A **macrostate map** is a function M: 𝒮 → 𝓜 onto a small set of labels. Periapsis already has several, without having named them as such:

| Macrostate map | Labels |
|---|---|
| Phase of a change | secret / draft / confirmed / public |
| Intent lifecycle (CubiKan) | caller-defined phases → terminal |
| Sprint phase (Sprint Loops) | Research / Plan / Build / Test / Loop, *derived from evidence* |
| Confirmation | green root / red / unknown |
| Working-copy status | clean / dirty / conflicted |

Sprint Loops' rule that phase is "derived, never a mutable pointer" says exactly that **the phase is a macrostate function of the microstate**, not extra state. That is the statistical-mechanics structure, already present in the design.

#### III.1.4 Entropies (define all of them; be explicit about the probability model)
- **Boltzmann (counting) entropy of a macrostate:** S_B(m) = log₂ |M⁻¹(m)|, the number of microstates compatible with label m, in bits. It needs a *finite, declared* microstate set (for example, mutants of a program, or candidate resolutions of a conflict).
- **Shannon/Gibbs entropy of a belief:** H(P) = −Σ P(s) log₂ P(s). It needs an explicit distribution P, for example from an agent's sampled candidates, a test-outcome model, or a language model.
- **Conditional entropy** H(A | B): the bits still needed to know A once B is known. **This is the workhorse quantity for Periapsis.**

The coarse-graining is *chosen*: macrostates are relative to the observer. That is a feature here. The Book chooses which macrostates matter.

---

### III.2. Reversibility: Landauer and Bennett as a VCS design law

#### III.2.1 The physics
- Landauer's principle says that any logically irreversible transformation, such as deleting a classical bit, dissipates heat. Bérut et al. (Nature, 2012) verified it with a one-bit colloidal memory: mean dissipated heat saturates at the Landauer bound for long erasure cycles.
- Bennett (1973) showed that any irreversible machine can be made logically reversible at every step by **saving all intermediate results** (a history tape). It can then copy out the result and *uncompute* the history. The history tape can be erased reversibly only because it is not random: it shares a mutual redundancy with the machine that produced it.
- Modern stochastic thermodynamics (Wolpert) formalizes this. The minimal cost (the "Landauer cost") is the **drop in entropy of the distribution over states**, so it depends on the input distribution, not only on the function. Circuit-level analyses show that the cost varies with how a computation is decomposed into gates and with correlations between the gates' inputs.

#### III.2.2 The precise statement for Periapsis
For a deterministic transition F applied to an uncertain prior state X:

  **Bits destroyed by F = H(X) − H(F(X)) = H(X | F(X)) ≥ 0**

with equality to 0 **iff F is injective on the support of X**.

- **Git-style VCS:** `reset --hard` over a dirty tree, `gc` of unreachable commits, force-push, `filter-branch`, squash without a record, and stash drop are all many-to-one maps. They destroy information about prior states.
- **Periapsis (op log + content addressing + evolution markers):** almost every operation is **injective at the repository level**, because the op log *is* Bennett's history tape. Undo and `op restore` are possible because nothing was erased. This is the formal reason jj-style undo works.

#### III.2.3 Design consequences
1. **A Landauer ledger.** *(Corrected, see Part IV E1/E2.)* Every op records `bits_destroyed_ub`, an **upper bound** on H(X | F(X)) (equal to it only under a uniform prior over collapsed predecessors). In practice this is log₂ of the number of distinct predecessor states that map to the result: the number of unreachable objects collected, overwritten working-copy states, or dropped heads. Normal ops record 0. Only explicit **erasure ops** (GC, redaction, sunset key destruction, history truncation) record more than 0, and those must be **signed, policy-gated and logged**.
2. **A formal definition of derived data.** *(Corrected, see Part IV E11.)* Data D is *derived* iff H(D | authoritative store, algorithm id, version) = 0, meaning it is recomputable. Deleting derived data destroys 0 bits; deleting authoritative data destroys more than 0. This turns the v3 rule "derived data is never authoritative" into a **checkable invariant**: the DST suite can delete all derived data, recompute it, and assert byte-equality.
3. **Bennett-style compaction.** The history can be compressed but not erased, except through an explicit erasure op. The working tree is the "output tape". The op log and objects are the "history tape". Checkpoints and summaries are reversible *only* if they keep enough to reconstruct the history (mutual redundancy). This is a principled basis for the cost tradeoffs of shallow clones and partial replication: a shallow clone isn't wrong, but it records that it lacks the bits.

---

### III.3. Merges and conflicts as macrostates

#### III.3.1 A merge is many-to-one, but only in the working tree
merge(A, B) → C maps two states to one at the **working-tree level** (𝒲), which is the A,B → C example from the question. At the **repository level** (𝒮) it is injective, because C records both parents. Periapsis can therefore claim:

  *Logical irreversibility is confined to views; the authoritative state is reversible.*

#### III.3.2 Conflict entropy
A conflicted file is stored as a conflict term (base, side₁, side₂, …), following jj. It is a **macrostate** whose compatible microstates are the candidate resolutions R = {r₁…r_k} (take-left, take-right, both orders, structured-merge outputs, agent proposals…).

- **Counting entropy:** S_B = log₂ k.
- **With a model:** H(R) = −Σ P(rᵢ) log₂ P(rᵢ), where P comes from structured-merge confidence, agent votes, or test outcomes.

Resolving a conflict reduces the entropy to 0. **Someone supplied H(R) bits of decision.** Periapsis can record *who*:

- `decision_bits(actor) += H(R_before) − H(R_after)` for every resolution, review approval or candidate selection.

This gives a precise, auditable answer to "how much of this history did humans actually decide, versus agents?". It is finer than "authored by" and complements Agent Trace's line attribution.

#### III.3.3 Resolution reuse is entropy that was already paid
A ResolutionRecord keyed by the hashes of the conflicting hunks means a repeated conflict costs 0 new decision bits. Periapsis can measure how much decision entropy reuse saved. That is a measurable benefit, and a test of whether a Pijul-style patch core (format v2) would earn its complexity: if snapshot-plus-reuse already removes most repeated decision bits, v2 is less urgent.

---

### III.4. Tests as measurements, and specification entropy

#### III.4.1 Tests partition microstate space
A deterministic test t is a predicate on program microstates, t: 𝒫 → {pass, fail}. A suite T is a coarse-graining M_T: 𝒫 → {pass, fail}^|T|. The array-test confirmation root is a signed record of one macrostate measurement.

#### III.4.2 Choosing tests by information gain
Given uncertainty about where a fault lies, the best next test maximizes **expected entropy reduction**. This is established in the literature:
- **FLINT** (Yoo, Harman, Clark, TOSEM 2013) orders tests so that each next test maximally reduces fault-locality entropy, and found information-theoretic ordering beat coverage-based and similarity-based orderings.
- **EntBug** (Campos et al., ASE 2013) generates tests that reduce the entropy of a test suite; the information gain of a new test is the reduction in the size of the top-ranked suspect set.

**Periapsis feature:** when a change fails confirmation, the verifier orders re-runs and bisection steps by expected information gain, rather than running everything. The cost of a confirmation is then roughly the bits needed, not the size of the suite.

#### III.4.3 Specification entropy (a new, precise metric)
Fix a set of mutation operators. This gives a finite **neighborhood** N(p) of programs "near" program p. Define

  Ω_pass(p, T) = |{ q ∈ N(p) : M_T(q) = M_T(p) }|  (mutants T can't distinguish from p)

  **S_spec(p, T) = log₂ Ω_pass** *(corrected in Part IV E3 to log₂(1 + D), where D counts behaviourally distinct non-equivalent survivors)*

- S_spec = 0 means the tests pin the program down within that neighborhood.
- A high S_spec means many different programs satisfy the suite, so the specification is loose.
- This is the Boltzmann entropy of the "passes my tests" macrostate. Ordinary mutation score is a linear proxy for it. *(Refuted in Part IV E3: for first-order mutation, S_spec does **not** add across modules; aggregate as log₂(1+ΣDᵢ).)*

array-test already has a mutation tier, so S_spec can be computed from data it already produces. It is the most natural *quality gate for agent-written code*: an agent that adds code without tests raises S_spec, which can be measured and blocked by policy. **Caveat:** the neighborhood depends on which mutation operators are chosen, and equivalent mutants inflate Ω. Report S_spec together with its operator set.

---

### III.5. Entropy of the change process: published risk signals

#### III.5.1 Change entropy (Hassan, ICSE 2009)
Hassan modeled the code-change process with Shannon entropy over how changes are spread across files in a period. Across six large open-source projects, these change-complexity metrics predicted faults better than prior modifications and prior faults. *(Part IV E7: across 4 Rust repos, change entropy added incremental signal beyond change counts in 8/8 splits but didn't consistently beat prior-fault counts.)* **Periapsis feature:** a derived-data index `change_entropy(window, path-prefix)`, broken down by actor class (human, agent, mixed). Agent swarms that scatter edits across many files will show up immediately. A policy can require extra confirmation when entropy spikes.

#### III.5.2 Naturalness and cross-entropy (Hindle 2012; Ray et al. 2016)
- Code is highly predictable. n-gram models give entropies below 2 bits in some projects, lower than English.
- Buggy lines are more entropic (less natural) and become less so after fixes. Inspecting files in order of entropy is about as cost-effective as standard defect prediction, and line-level entropy is comparable to static bug finders.
- *(Part IV E8: in our replication, buggy lines were no more surprising than their fixes, though changed lines were more surprising than unchanged ones. Treat as a weak hint.)*
- **Periapsis feature:** a per-hunk *surprisal* score, the cross-entropy of new code under a model trained on the repo at the parent changeset. Store it as derived data and use it to order human review of agent-generated diffs, so humans spend their decision bits where the surprise is.
- **Caveats:** naturalness depends on tokenization and model choice, and a 2026 benchmark notes that AI-generated code has its own statistical signatures. Treat surprisal as a prioritization signal, never as a gate on its own.

---

### III.6. Agent swarms: a Maxwell's-demon view (precise version)

#### III.6.1 The setup
N agents explore and produce N candidate microstates (changes) for an intent. The verifier-plus-human gate selects k of them to promote. Physically this is a demon: it measures, then sorts noise into order.

#### III.6.2 The information bound
Choosing k items from N requires at least

  **I_select = log₂ C(N, k) bits** *(refuted as a lower bound in Part IV E6: this is the cost of identifying one specific k-subset; promoting any acceptable candidate needs ≥ log₂(N/a) bits)*

of decision information, supplied by tests, verifiers or humans. It can be supplied more cheaply if the distribution over candidates is skewed: the cost is H(selection), which is at most log₂ C(N,k).

**Consequence (a throughput law, as a model; rewritten in Part IV E6: load depends on the acceptable fraction among candidates that survive verification, and effort ≈ 2^bits):** if the trusted decision sources (human reviewers plus signed verifiers) supply B bits per unit of work, then accepted selections are bounded by roughly B / H(selection). Spawning more agents raises N, which raises H(selection) unless the verifier's tests absorb the extra entropy. **Scaling agents is limited by the verifier's information throughput, not by how fast agents generate.** This gives a quantitative basis for Anthropic's observation that the bottleneck moved to review and CI. It also says what to invest in: tests with high information gain (§4.2), and an S_spec that keeps falling (§4.3).

#### III.6.3 The demon must keep records
Bennett's resolution of Maxwell's demon is that the demon's memory has to be reset, and the erasure is where the cost lies. In Periapsis the equivalent is the **ledger**. The selection records (evaluations, decision bits, rejected candidates as negative lineage) are kept, never erased. That matches the Biblio-era design principle that failed agents stay in the record.

#### III.6.4 Divergence budget
Live divergent heads per intent, weighted by activity, form a distribution. Its entropy H_heads is a measure of how much unresolved exploration the repo carries. *(Part IV E13: a Shannon budget can be gamed by one dominant head plus many trickle heads; put the budget on log₂(count).)* A **policy budget** (for example H_heads ≤ 4 bits per intent, about 16 equally live lines) keeps exploration bounded without fixing a head count.

---

### III.7. Storage, sync and similarity: entropy as cost

| Quantity | Meaning in Periapsis |
|---|---|
| H(new version \| old version) | Lower bound on delta size; FastCDC gives about one chunk per localized edit, far above this bound (Part IV E10) |
| Mutual information between peers' states | What sync doesn't need to send. RBSR's cost scales with the *difference*, not the total |
| Compression-based distance | Rename and copy similarity without hand-tuned heuristics *(my analysis; I didn't research benchmarks)* |
| KL divergence between two peers' views | How far apart two replicas' beliefs are; drives sync scheduling |

These turn performance targets into information-theoretic budgets. For example, "sync bytes ≤ c · d · log₂(n/d) + overhead, with full-transfer fallback" (corrected per Part IV E9) is a testable property for the DST suite.

---

### III.8. Where the analogy breaks (be honest)

1. **Physical energy is negligible.** The Landauer bound is about 3×10⁻²¹ J per bit at room temperature, and the Bérut paper notes real devices then sat about 1,000× above it. A repo GC that erases 10¹² bits has a Landauer bound of about 3 nJ. Periapsis gains nothing physically. **The value is purely in the information accounting.**
2. **Entropy needs a declared probability model or a finite microstate set.** "Entropy of a codebase" with no stated P is meaningless. Every Periapsis metric must declare its microstate set (mutants, candidate resolutions, files in a window) or its model (n-gram or LLM), and store that declaration next to the value.
3. **Microstate spaces are astronomically large.** S_B is only computable over *defined neighborhoods*. Global "entropy of all possible repos" is not a usable quantity.
4. **Stochastic thermodynamics doesn't map directly.** Wolpert notes that visible discrete computation generally needs hidden states in the underlying physical process. Use the *mathematics* (entropy drops, input-distribution dependence, decomposition effects), not claims about physical dissipation.
5. **Goodhart risk.** Any entropy metric used as a gate will be optimized against by agents, for example by padding tests to lower S_spec, or rewriting code to look "natural". Mitigation: verifier-owned metrics, signed by keys agents can't access, and combined signals.

---

### III.9. Concrete additions to the Periapsis plan

**New crate:** `peri-entropy` (pure functions over derived data; no I/O).

| Feature | Formal definition | Phase |
|---|---|---|
| Landauer ledger (`bits_destroyed_ub` per op) | Upper bound on H(X \| F(X)); compressed size of unique erased data for erasures; requires snapshot-before-overwrite | 1 (op log) |
| Derived-data invariant test | Delete all derived data, recompute, assert byte-equality (H(D \| authoritative) = 0) | 1 (DST) |
| Signed erasure ops (GC, redaction, truncation) | The only ops allowed to record `bits_destroyed > 0` | 1–2 |
| Conflict entropy + decision-bit attribution | S_B = log₂ k or H(R); `decision_bits(actor)` | 2 |
| Resolution-reuse savings metric | Decision bits avoided by ResolutionRecords | 2 |
| Specification entropy S_spec | log₂(1 + D) over a declared mutation neighborhood (D = distinct non-equivalent survivors) | 3 (array-test mutation tier) |
| Info-gain test ordering for confirmation and bisect | Maximize expected ΔH (FLINT/EntBug style) | 3 |
| Change-entropy index by actor class | Hassan BCC/HCM over windows and path prefixes | 3 |
| Hunk surprisal for review ordering | Cross-entropy under a repo-trained model at the parent change | 3–5 (optional model) |
| Divergence budget per intent | log₂(live head count) gate; Shannon H_heads reported | 3 |
| Selection-information accounting | ≥ log₂(N_surv/a_surv) per promotion; inspection effort ≈ 2^bits; dashboards | 3 |
| Sync-cost property tests | Bytes ≤ c·d·log₂(n/d) + overhead, with fallback, under DST | 4 |

**Output surface:** `peri entropy [--intent X] [--json]` reports S_spec, change entropy, decision bits by actor, open divergence (H_heads), and the Landauer ledger. The Book renders these as macrostate panels.

---

### III.10. Open research questions
1. **The minimal decision-bit cost of a merge policy.** Following Wolpert's result that cost depends on the input distribution and the decomposition: given the empirical distribution of conflicts in agent swarms, which merge pipeline (diff3 → structured → agent proposal → human) minimizes expected human decision bits?
2. **S_spec as an acceptance criterion.** Does lowering S_spec for agent-written changes reduce post-merge defects? This could be measured on Periapsis's own history once it self-hosts.
3. **Snapshot vs patch core, measured in bits.** Compare decision bits per merge under snapshot-plus-reuse against a commutative patch core on the same agent workloads. That would decide format v2 with data instead of argument.
4. **A verifier information-throughput law.** Validate empirically that accepted changes per day track B / H(selection) as agent count grows.
5. **Entropy in the sprint loop.** Can the Loop phase be driven by entropy targets, for example "close the sprint when S_spec and H_heads for the sprint's intents fall below thresholds"? That would be a precise, evidence-derived exit criterion, consistent with Sprint Loops.

### III.11. Confidence
- **High (primary sources):** Landauer and its experimental verification; Bennett's history-tape construction; Wolpert's input-distribution dependence; Hassan's change-entropy fault prediction; naturalness and buggy-code entropy; FLINT and EntBug information-gain test ordering.
- **Analysis (mine, formally stated but not empirically validated):** the Landauer ledger, the derived-data invariant, decision-bit attribution, S_spec, the selection-throughput bound, H_heads budgets, and compression-based rename detection.
- **Not researched:** benchmarks for compression-based similarity in VCSs; prior work defining "specification entropy" under that name (it may exist in mutation-testing literature under another name, so do a literature check before claiming novelty).


---

# PART IV: Correctness Assessment and Experimental Validation

## Animus_Periapsis: Correctness Assessment and Experimental Validation

*Date: 2026-10-31. Scope: the whole Master Report, especially Part III (entropy) and every claim marked "analysis / untested". All experiments are reproducible from `experiments/` (Python 3, numpy/scipy). The real-repository tests used ripgrep, fd, bat and helix, cloned on 2026-10-31.*

---

### 1. Verdict summary

| # | Claim (where it appears in the Master Report) | Test | Result | Verdict |
|---|---|---|---|---|
| 1 | Bits destroyed by a deterministic op = H(X) − H(F(X)) = H(X\|F(X)) (III §2.2) | E1: random distributions and maps | Identity holds to 4 d.p. in all trials | ✅ **Confirmed** |
| 2 | Ledger value "log₂ of the number of collapsed predecessors" equals H(X\|F(X)) (III §2.3) | E1 | It is an **upper bound** (e.g. 1.73 vs true 1.41 bits); equal only when the prior over predecessors is uniform | ⚠️ **Corrected** |
| 3 | Periapsis ops are injective at repo level, "because the op log is Bennett's history tape" (III §2.2, I.2) | E2: toy op-log VCS, 200 pre-state pairs | 0/200 collapses **with** auto-snapshot; **200/200 collapses without it** | ⚠️ **Conditional**: holds only if every working-copy-overwriting op snapshots first, and op-log abandonment counts as erasure |
| 4 | Derived data D has H(D\|authoritative) = 0, so a "delete and recompute; byte-equal" invariant works (III §2.3, I.5 step 12) | E11: same input, two settings of one diff algorithm | Outputs differ (28 vs 30 hunks) | ⚠️ **Corrected**: true only if the algorithm id and version are part of the input; derived data must be keyed by them |
| 5 | Merge is many-to-one only in the working tree (III §3.1) | Follows from #3 (parents recorded) | Holds under the same auto-snapshot condition | ✅ Confirmed (conditional) |
| 6 | "Decision bits are conserved: bits supplied = entropy resolved" (I.5 step 20 exit gate) | E5 | True, but only because the sum **telescopes**, so it can't fail as a test. **Credit depends on order** (the test gets 1.0 or 0.0 bits depending on whether it runs before or after the human). Entropy can **rise** when new candidates arrive, so "decision bits" go negative | ❌ **Refuted as an exit gate.** Replace with order-independent (Shapley) attribution; allow negative entries |
| 7 | Choosing k of N candidates needs **at least** log₂C(N,k) bits (III §6.2, I.2) | E6: exact computation | That figure is what it takes to **identify one specific subset** under a uniform prior. Promoting *any* acceptable candidate needs as little as log₂(N/a) (20 candidates, 5 acceptable: 2.0 bits vs the claimed 4.32 for k=1; 8.84 vs 13.92 bits for "any 3") | ❌ **Refuted as a lower bound.** Correct bound: ≥ log₂(N/a) (a = number acceptable), reduced further by tests |
| 8 | Throughput law: accepted ≈ B / H(selection); "more agents raise the decision load" (III §6.2) | E6 table, E-effort | With a **constant acceptable fraction**, the required bits **don't grow with N** (2.0 bits at N = 4…1024). They grow only if the number acceptable stays fixed. Bits also **understate effort**: inspections ≈ 2^bits (measured 99.6 inspections for 6.64 bits) | ⚠️ **Rewritten** (§3.4) |
| 9 | Specification entropy S_spec = log₂ Ω_pass (III §4.3) | E3/E3b: real AST mutation testing on two functions | S_spec = 0 while a surviving mutant still existed (Ω = 1 gives 0 bits); equivalent mutants inflate it | ⚠️ **Definition corrected**: S_spec = log₂(1 + #behaviourally distinct non-equivalent survivors) |
| 10 | S_spec "adds up across independent modules" (III §4.3) | E3b | With standard first-order mutants, sum = 4.91 bits vs combined = 3.58 bits (the neighbourhood is a *union*, not a product) | ❌ **Refuted.** Additive only with a product neighbourhood (one mutation per module at once) |
| 11 | Greedy information-gain test ordering beats coverage and random ordering (III §4.2, citing FLINT/EntBug) | E4 (deterministic) and E4b (noisy, Bayesian) | Tests to localize 1 fault among 256: **9.7** (info-gain) vs 15.3 (coverage) vs 19.7 (random); floor 8.0. With 40% coincidental correctness: residual entropy 0.02 vs 0.28 vs 0.55 bits, top-1 = 100% / 95% / 82% | ✅ **Confirmed** in simulation, given a known noise model |
| 12 | Change entropy (Hassan) predicts faults *better than* prior modifications and prior faults (III §5.1) | E7: replication on 4 Rust repos × 2 splits | Spearman ρ with future fix-commits: HCM 0.05–0.55, comparable to the baselines, **not consistently better** than prior faults. HCM2 adds incremental signal beyond prior modifications in **8/8** splits (partial ρ +0.11 … +0.33) | ⚠️ **Partially confirmed**: a useful extra signal, not demonstrably superior |
| 13 | Buggy lines are more entropic than their fixes; use surprisal for review ordering (III §5.2) | E8: trigram model, held-out training files, 80 fix commits each in helix and bat | Buggy vs fixed: **no difference** (helix 6.98 vs 6.90 bits/token, p = 0.57; bat 8.39 vs 8.40, p = 0.91). Changed lines **are** more entropic than unchanged lines (p = 0.003 and 0.0008) | ⚠️ **Not replicated** for bugs specifically. Surprisal flags unusual or churn-prone code, not bugs. Downgrade to a weak ordering hint |
| 14 | RBSR sync: "bytes ≤ c·(difference bits) + overhead" (III §7, I.5 step 32) | E9: implemented RBSR, n up to 10⁶ | Correct reconciliation in all cases; 32 bytes when identical. Bytes per difference grow with **log n** (963 → 1,570 → 1,840 for d = 10). At d/n = 10%, RBSR (176 KB) **exceeds** sending all ids (80 KB) | ⚠️ **Corrected**: bytes ≈ c·d·log(n/d) + overhead, plus a **fallback to full transfer** when d/n is large |
| 15 | FastCDC: an edit disturbs only nearby chunks (II §2.8) | E10: Gear/normalized CDC on 3 MB of helix source | 37-byte insert: **1** new chunk (CDC) vs 245 (fixed size). Insert at start: 1 vs 367 | ✅ **Confirmed** |
| 16 | "FastCDC plus compression approximates H(new\|old)" (III §7) | E10 | A 37-byte edit uploads one whole chunk (~7.6–14 KB). That is chunk-granular, orders of magnitude above the conditional entropy | ❌ **Refuted as worded.** CDC bounds cost at about one chunk per localized edit; nearing H(new\|old) needs delta compression inside chunks |
| 17 | Divergence budget H_heads ≤ 4 bits ≈ 16 live heads (III §6.4) | E-heads | One dominant head plus 1,000 trickle heads gives **H = 0.18 bits** and passes the budget | ❌ **Gameable.** Budget on log₂(count) (Hartley); Shannon only as an extra signal |
| 18 | Merging divergent op heads (concurrency design, II §2.1/§3.3) | E12–E12e: model-checked bookmark merges | Pairwise 3-way, common base: order-independent ✅. Pure signed-sum algebra **fails criss-cross correctness** (466/4000) without a "both sides equal" rule. Adding that rule **breaks order-independence** (1472/5000). A **canonical n-way merge** (deduplicate sides, then merge all heads in one step) is order-independent and criss-cross-correct (0 failures). **Merge-of-merges ≠ flat merge** (7756/20000) unless a **support-set normal form** is used (0 failures) | ⚠️ **New requirement** found (§3.6) |
| 19 | Landauer bound ≈ 3×10⁻²¹ J/bit; 10¹² bits ≈ 3 nJ (III §8) | E1 | 2.87×10⁻²¹ J; 2.87 nJ | ✅ Confirmed |
| 20 | Pijul licence possibly GPL-2.0-only, so incompatible (v1/v2 reports) | crates.io API | `pijul`, `pijul-core`, `libpijul` are all **GPL-2.0-or-later** | ✅ **Compatible** with GPL-3.0; the earlier warning is withdrawn |
| 21 | "Scion copied the Bonsai model and stated so" (II §2.3) | Re-read sources | Anthropic's footnote says Mononoke "pioneered many of the techniques we use in Scion, in particular the Bonsai data model". The *deliberate-copying* claim is Rain's, from an unnamed source | ⚠️ **Overstated.** Reworded |
| 22 | Crate/CLI names | crates.io API | `periapsis` taken; `peri` taken (a toy game); `animus-periapsis`, `peri-cli`, `peri-core`, `periapsis-core` free | ⚠️ Use `peri-*` / `animus-periapsis`; the binary can still be called `peri` |

**Score:** 22 claims tested. 7 confirmed, 9 corrected or conditional, 6 refuted as written. None of the refutations breaks the architecture. They change definitions, bounds and exit gates.

---

### 2. Experiments in detail

#### E1: Bits destroyed by a deterministic map
Random 16-state distributions and random maps to 6 outputs. H(X) − H(F(X)) equalled H(X|F(X)) exactly in every trial (e.g. 1.4072 = 1.4072). The ledger formula Σ_y p(y)·log₂|F⁻¹(y)| gave 1.47–1.89 bits against true values of 1.34–1.81 bits, so it is the maximum-entropy upper bound.
**Fix:** the ledger records an *upper bound*. For erasing content (GC, redaction), the right figure is the information content of the *unique* erased data, estimated by its compressed size. That is also log₂ of the number of possible erased contents under a realistic model, so the two views agree.

#### E2: Is the op log really a history tape?
A toy content-addressed VCS with an op log; checkouts start from pre-states that differ only in uncommitted working-copy content. With auto-snapshot (jj-style), 0/200 distinct pre-states collapsed. Without it, 200/200 collapsed: the uncommitted edits were destroyed. Reachability-based GC removed nothing, because every snapshot is reachable from the op log.
**Fix:** state the precondition explicitly. *Every op that overwrites the working copy must snapshot it first.* Op-log abandonment or truncation (jj's `op abandon`) is an erasure op.

#### E3/E3b: Specification entropy on real mutation testing
AST mutation (comparison, boolean, constant ±1) of `clamp` (10 mutants) and `is_leap` (23 mutants). Equivalence was checked exhaustively over each function's domain.

| Program | Survivors | Equivalent | S_spec (old def) | S_spec (fixed def) |
|---|---|---|---|---|
| clamp, weak tests | 4 | 2 | 2.00 | 1.58 |
| clamp, strong tests | 2 | 2 | 1.00 | **0.00** |
| leap, weak tests | 10 | 1 | 3.32 | 3.32 |
| leap, strong tests | 1 | 1 | **0.00** (wrong: a survivor exists) | 0.00 (correct: it's equivalent) |
| combined, weak tests | 14 | 3 | 3.81 (≠ 2.00 + 3.32) | 3.58 (≠ 1.58 + 3.32) |

**Fixes:**
- **Definition:** S_spec = log₂(1 + D), where D = the number of non-equivalent surviving mutants with *distinct behaviour*. This counts p itself, and equivalent mutants *are* p.
- **Additivity:** additive only over a product neighbourhood. For ordinary first-order mutation, report S_spec per module and aggregate with log₂(1 + Σ D_i).
- **Practical:** equivalent-mutant detection is undecidable in general. Report S_spec as an upper bound unless equivalence is checked, along with the mutation operator set.

#### E4/E4b: Information-gain test ordering
Single fault among 256 components, 400 tests with random coverage.

| Strategy | Deterministic: tests to localize | Noisy (q = 0.6): residual entropy after 40 tests | Top-1 |
|---|---|---|---|
| Random | 19.7 | 0.55 bits | 82% |
| Max coverage | 15.3 | 0.28 bits | 95% |
| **Max expected info gain** | **9.7** (floor 8.0) | **0.02 bits** | **100%** |

This confirms the FLINT/EntBug direction. Caveats: the model assumes a known noise rate, a single fault and independent tests.

#### E5: Decision-bit attribution
Eight candidate resolutions (3 bits). The test suite eliminates half, the human picks one.

| Order | Test credit | Human credit |
|---|---|---|
| test → human | 1.0 | 2.0 |
| human → test | 0.0 | 3.0 |
| **Shapley (average over orders)** | **0.5** | **2.5** |

(The first run of the script mis-coded this line; the shipped script is fixed and prints these values.)

A new candidate with a comparable prior raised H from 0.47 to 1.24 bits, so decision accounting must allow negative entries ("entropy injected").
**Fix:** attribute by Shapley value over the sources (tests, verifiers, humans, agents). Replace the "conservation" exit gate with *order-invariance of attribution* plus *ledger completeness*: every change in entropy is attributed to some source.

#### E6: The selection bound and the throughput law
For N = 20, a = 5 acceptable:

| Task | Bits |
|---|---|
| Identify the exact acceptable set | log₂C(20,5) = **13.92** |
| Promote any 1 acceptable (one concrete strategy) | **3.00** |
| Information-theoretic floor for any 1 | log₂(N/a) = **2.00** |
| Promote any 3 acceptable | **8.84** (vs log₂C(20,3) = 10.15) |

Each output value is valid for at most a fraction a/N of possible acceptable sets, which gives the floor log₂(N/a).

| N agents | 25% acceptable | Fixed 2 acceptable |
|---|---|---|
| 4 | 2.0 bits | 1.0 |
| 64 | 2.0 bits | 5.0 |
| 1024 | 2.0 bits | 9.0 |

Tests with a false-pass rate of 1% cut human bits from 2.00 to 0.04. **Effort ≠ bits:** to find an acceptable candidate by inspection takes about 1/f = 2^bits inspections (measured 2.0 / 4.0 / 9.9 / 99.6 inspections for 1 / 2 / 3.32 / 6.64 bits).
**Corrected law:** human decision information per promotion ≥ log₂(N_surviving / a_surviving), and inspection effort scales as 2^that. Scaling agents is costly **only when it lowers the acceptable fraction among candidates that survive verification.** The lever is therefore verifier precision (false-pass rate), not agent count.

#### E7: Hassan change entropy on real repos
Commits to `.rs` files, bots excluded, 30-day periods normalized by log₂ of the number of known files. Faults are counted as future fix-commits (regex on the message) within 365 days of the split.

| Repo / split | prior mods ρ | prior faults ρ | HCM1 ρ | HCM2 ρ | partial ρ(HCM2 \| mods) |
|---|---|---|---|---|---|
| ripgrep 0.5 | 0.17 | 0.30 | 0.13 | 0.20 | +0.11 |
| ripgrep 0.7 | 0.07 | 0.16 | 0.05 | 0.15 | +0.19 |
| fd 0.5 | 0.42 | 0.40 | 0.42 | 0.48 | +0.33 |
| fd 0.7 | 0.21 | 0.21 | 0.21 | 0.29 | +0.31 |
| bat 0.5 | 0.29 | 0.39 | 0.35 | 0.33 | +0.17 |
| bat 0.7 | 0.39 | 0.42 | 0.39 | 0.45 | +0.25 |
| helix 0.5 | 0.51 | 0.53 | 0.55 | 0.52 | +0.15 |
| helix 0.7 | 0.44 | 0.43 | 0.47 | 0.46 | +0.17 |

**Reading:** HCM2 ≥ prior modifications in 7/8 splits and adds positive signal in 8/8, but it doesn't consistently beat prior faults. Limits: small repos, message-regex fault labels, no significance correction. **Use change entropy as one input to a risk model, not as a standalone gate.**

#### E8: Naturalness of buggy vs fixed code
A trigram model (interpolated) trained on the snapshot before the oldest evaluated fix, excluding every file touched by the evaluated fixes, so there is no leakage. 80 fix commits per repo, each with ≤ 10 deleted lines.

| Repo | Buggy (deleted) | Fix (added) | Unchanged (same file) | Buggy > fix | Buggy > unchanged |
|---|---|---|---|---|---|
| helix | 6.98 | 6.90 | 6.03 | 50%, p = 0.57 | 62%, **p = 0.003** |
| bat | 8.39 | 8.40 | 7.49 | 43%, p = 0.91 | 71%, **p = 0.0008** |

**Reading:** lines involved in fixes, on both sides, are more surprising than ordinary lines. A simple model does **not** show buggy lines to be more surprising than their fixes. Ray et al. used a cache language model plus line-type normalization, which this didn't replicate, so the original result isn't refuted. But the master report's "spend review where the surprise is" works only as a weak hint.

#### E9: RBSR sync cost
A from-scratch implementation (XOR-of-BLAKE2 fingerprints, branching factor 16, threshold 4). Correct in every case.

| n | d = 10 | d = 100 | d = 1000 | Send all ids |
|---|---|---|---|---|
| 10⁴ | 9.6 KB | 57 KB | **176 KB** | 80 KB |
| 10⁵ | 15.7 KB | 104 KB | 620 KB | 800 KB |
| 10⁶ | 18.4 KB | 150 KB | 1.08 MB | 8 MB |

Rounds: 4–6. The unencoded, untuned implementation is about 5–10× heavier than Negentropy's compact framing would be, but it shows the right scaling.
**Fix:** the property test should be `bytes ≤ c·d·log₂(n/d) + overhead`, and the protocol should switch to sending the full set when d/n exceeds a threshold.

#### E10: Content-defined chunking
3 MB of helix source; FastCDC-style Gear hash with normalized chunking (avg 8 KiB).

| Edit | CDC new chunks / bytes | Fixed-8 KiB new chunks / bytes |
|---|---|---|
| 37-byte insert at 1/3 | **1 / 7,657** | 245 / 2,000,610 |
| Insert at start | **1 / 14,186** | 367 / 3,000,034 |

Locality is confirmed. The cost is chunk-granular, not near the conditional entropy.

#### E11: Derived data depends on the algorithm version
`difflib` with `autojunk` on vs off on the same input: 28 vs 30 hunks. **Fix:** key derived data by (input hashes, algorithm id, algorithm version). The DST invariant then compares recomputation under the *same* version, and version upgrades become explicit derived-data migrations.

#### E12: Concurrency: merging divergent op heads (bookmark/pointer values)

| Merge rule | Order-independent (3–5 heads) | Criss-cross correct | Flat = nested |
|---|---|---|---|
| Naive value-or-set, pairwise | ✅ 0/5000 | ✅ 0/4000 | — |
| Pure signed-sum algebra, pairwise | ✅ 0/5000 | ❌ 466/4000 | — |
| Algebra + "sides equal" rule, pairwise | ❌ 1472/5000 | ✅ 0/4000 | — |
| **Canonical n-way** (deduplicate sides, single step) | ✅ 0/20000 | ✅ 0/4000 | ❌ 7756/20000 |
| **Canonical n-way + support-set normal form** | ✅ | ✅ | ✅ 0/20000 |

**Requirement for the concurrency spec:** merge all divergent heads in **one canonical n-way step** relative to the LCA (or the virtual base, for criss-cross histories), and compare and store results in a **normal form**, so that replicas that merged in different groupings converge. For pointer conflicts a support-set form works (jj's "moved from A to B or C"). For file-content conflict terms, multiplicities may matter; that needs a dedicated property-test suite (commutativity, idempotence, associativity up to normal form, convergence) and has **not** been settled here.

---

### 3. Corrected statements (to replace text in the Master Report)

1. **Landauer ledger:** "Each op records `bits_destroyed_ub`, an *upper bound* on H(X|F(X)): 0 for injective ops; for erasure ops, the compressed size of the unique data erased. Valid only if every working-copy-overwriting op snapshots first; op-log abandonment is an erasure op."
2. **Derived data:** "D is derived iff H(D | authoritative, algorithm id, version) = 0. Derived data is keyed by its algorithm version; the DST invariant compares recomputation under the same version."
3. **Decision bits:** "Attributed by Shapley value across sources; entries may be negative (entropy injected by new candidates). Exit gate: attribution is order-invariant and the ledger is complete." (Not "conservation".)
4. **Selection bound:** "Promoting any acceptable candidate needs ≥ log₂(N/a) bits (a = acceptable count), minus what tests supply; log₂C(N,k) is the cost of identifying one specific k-subset."
5. **Throughput:** "Human decision information per promotion ≥ log₂(N_surv/a_surv); inspection effort ≈ 2^that. More agents raise load only if they lower the acceptable fraction among candidates that pass verification. Invest in verifier precision."
6. **S_spec:** "S_spec = log₂(1 + D), with D = behaviourally distinct, non-equivalent surviving mutants over a declared operator set. An upper bound unless equivalence is checked. Not additive across modules for first-order mutation; aggregate as log₂(1 + Σ D_i)."
7. **Change entropy:** "Adds incremental fault signal beyond change counts (replicated in 4 Rust repos), but isn't consistently superior to prior-fault counts. One input to a risk model."
8. **Surprisal:** "Flags unusual, change-prone code (replicated); does not distinguish buggy lines from their fixes in our replication. A weak review-ordering hint only."
9. **Sync cost:** "bytes ≤ c·d·log₂(n/d) + overhead, with fallback to full transfer above a d/n threshold."
10. **CDC:** "Bounds upload to about one chunk per localized edit; nearing H(new|old) needs intra-chunk delta compression."
11. **Divergence budget:** "Budget on log₂(live head count) (Hartley); Shannon entropy of activity is reported, not used as the gate."
12. **Concurrency:** "One canonical n-way merge of all op heads relative to the LCA or virtual base, with results in a normal form; property-tested for order-independence, criss-cross correctness and grouping-independence."
13. **Pijul licence:** GPL-2.0-or-later, compatible with GPL-3.0.
14. **Scion:** "Anthropic credits Mononoke as having pioneered many of Scion's techniques, in particular the Bonsai data model; a Mononoke engineer alleges the copying was deliberate (unverified source)."

---

### 4. What is still untested, and how to test it

| Claim | Why it wasn't tested here | How to test |
|---|---|---|
| S_spec gating reduces post-merge defects | Needs a real agent workflow plus defect labels | Run on Periapsis's own history after self-hosting; A/B test the gate threshold |
| Info-gain ordering on *real* suites | E4 used a synthetic coverage model | Defects4J-style benchmark with real coverage matrices; compare with FLINT |
| Content-conflict term algebra (multiplicities) | E12 covered pointer values only | Property tests over line-level conflict terms; compare with jj's merge implementation |
| Snapshot-plus-reuse vs patch core in decision bits | Needs both implementations | Phase 5 experiment on recorded agent conflict streams |
| Hassan/naturalness at larger scale | 4 repos, regex labels | SZZ-labelled datasets; cache LM / modern code LM; significance testing |
| RBSR with production framing | Untuned prototype | Implement Negentropy V1 framing; DST with partitions |
| Throughput law in practice | Needs multi-agent operation | Telemetry: candidates, survivors, acceptable fraction, reviewer time |

---

### 5. Reproducibility
Scripts are in `experiments/` (E1–E12 plus variants). Each runs standalone with Python 3.10+, numpy and scipy. E7 and E8 need local clones of ripgrep, fd, bat and helix in the working directory. Random seeds are fixed. The results in this report are those scripts' outputs. The E5 Shapley line was mis-coded in the first run; the shipped script is fixed and prints the values in §2 E5 (test 0.5, human 2.5). The throughput and H_heads figures come from `e6b_throughput.py` and `e13_heads_effort.py`.


---

# PART V: Experimental Research, Round 2 (merge, storage, sync, real-suite validation)

## Animus_Periapsis: Experimental Research Report, Round 2

*Date: 2026-10-31. Follows the Validation Report (Round 1). It tests the items Round 1 left open, plus the core merge, storage, sync and content-layer assumptions, using real repositories (helix, bat, toolz) and three from-scratch Rust prototypes (`peri-merge`, `peri-store`, `peri-content`). Scripts: `experiments/round2/`. Prototype sources: `prototypes/`.*

---

### 1. Summary of findings

| # | Question | Method | Result | Design consequence |
|---|---|---|---|---|
| R2-E14 | Is diff3 order-dependent or silently wrong on non-overlapping concurrent edits (Pijul's critique)? | `git merge-file`, 1,500 + 7,200 synthetic three-agent scenarios with ground truth | **Never order-dependent.** Silent wrong results **0.03–0.3%**, only in files with 70% repeated lines (identical `}` / `return` inserted near each other). Edits that merely touch conflict about 10% of the time by design | diff3 is sound in practice; the risk sits in repeated boilerplate |
| R2-E19/E21 | Same question on **real** histories | 5,325 reconstructed concurrent edit pairs (consecutive real commits on the same file, made concurrent by transplanting), helix + bat | 21% / 37% conflict. Clean merges ≠ truth in 2.2% / 8.1%, but **every divergence is explained by dependent changes or revert/re-add ambiguity. 0 genuine silent errors found** | **Deprioritize the Pijul-style patch core** (format v2); it has no observed correctness gap |
| R2-E20 | Can a from-scratch Rust diff3 match git? | `peri-merge` (imara-diff), 800 random cases | Clean/conflict status agrees in **99.75%** (histogram); identical output whenever both are clean; identity, idempotence and symmetry hold | The merge core is implementable; prototype included |
| R2-E21 | Does histogram vs Myers diff matter? | Same 5,325 real pairs | Marginal: conflicts 21.1 → 20.8% (helix), 37.4 → 37.0% (bat) | Use histogram (readability), but don't expect it to change merge outcomes |
| R2-E29 | Can conflicts on edits that merely *touch* be relaxed safely? | `peri-merge` loose mode, real pairs | Conflicts −12% to −14% relative (19.6 → 16.9%, 36.2 → 31.8%); **independent clean merges 100% exact** (3,028/3,028) | Offer loose adjacency, gated by build/test confirmation |
| R2-E22 | How much does structured merge (Mergiraf 0.20) resolve, and how safely? | All 1,418 real git-conflicted cases | Resolves **40% (helix) / 34% (bat)**. Of 45 verifiable resolutions: 35 exact, 8 formatting-only or harmless reorder, **2 duplicated a moved block** (compile-breaking) | Structured merge is a strong second stage, but its output must pass build/test confirmation plus a duplicate-definition check |
| R2-E28 | How often do the *same* conflicts recur when an agent rebases repeatedly? | Agent change rebased onto 10 successive trunk versions, 573 changes | 33% / 59% of rebase steps conflict; **69–75% of conflict blocks are exact repeats** | A resolution store (keyed by the full conflict block) auto-answers about 70% of decisions. This is the empirical case for "snapshot + reuse" |
| R2-E16 | Does information-gain test ordering work on a **real** suite? | toolz: real per-test coverage (coverage.py contexts) + 113 real injected faults | Beats random and coverage-first ordering at every budget. **Reaches the full-suite Ochiai localization quality after a median of 11 tests (19% of the suite)** for 111/113 faults. Its posterior with *all* tests is a bit weaker than Ochiai on mean rank (19 vs 14) | Use info-gain for *ordering* re-runs and bisection; use Ochiai-style scoring once all results are in |
| R2-E24/E24t | Does S_spec predict defects? | toolz: 461 + 362 real mutants; S_spec at HEAD and at a 2017 tag vs later fix-commits | No relationship (ρ = +0.10, p = 0.49 at HEAD; ρ = −0.04 and Fisher p = 1.0 in the temporal test). **Underpowered:** 32 functions, 8 later fixes, 6–9% survival | S_spec stays an **advisory metric, not a gate**, until a multi-project study shows value |
| R2-E23 | Does the append-only store survive crashes? | `peri-store`: simulated FS with torn/lost unsynced writes; crash at every syscall × 40 seeds | Correct design: **0 violations / 2,600 scenarios.** Negative controls caught (84 and 1,457 violations). Strict POSIX mode **found a real bug** (missing directory fsync at init: 48 violations), fixed to 0 | DST pays off: it found a classic durability bug. Init must fsync the directory |
| R2-E23b | Is a HEAD pointer file needed? | Same harness | No-HEAD design (heads derived from the checksummed op log): **0 violations, 37% fewer syscalls** | **Op heads are derived data.** Remove the HEAD file from the format |
| R2-E27 | Do concurrent processes interleave records appended with O_APPEND? | 16 processes, 61,600 appends up to 4 MiB, ext4 | 0 corrupt records | Works locally but isn't guaranteed by POSIX: use per-op files for portability; keep checksummed framing |
| R2-E27a | Hash choice | Rust microbenchmark | BLAKE3 4.9 GB/s vs SHA-256 0.21 GB/s (**23×**) vs SHA-1 0.62 GB/s; 2.7 GB/s on 4 KiB objects. *This CPU appears to lack SHA extensions* | BLAKE3 confirmed |
| R2-E27b | Is content-defined chunking right for source code? | All 9,170 historical `.rs` blob versions of helix (370 MB raw) | git delta packs **8.49 MB**; best CDC (2 KiB) + zstd **25.4 MB**; whole-blob zstd 86.7 MB; **zstd with the previous version as dictionary: 4.35 MB (level 3) / 3.48 MB (level 19)** | **Text/source: per-path reference compression** (2–2.4× better than git). CDC only for large or binary files |
| R2-E26 | Is RBSR right for op-log sync? | Simulated op DAGs (10⁴–10⁵ ops, 8 live lines of work) | DAG want/have negotiation: about **32 bytes per missing op**, independent of n, in 4–8 rounds; **10–30× fewer bytes than RBSR** | **Correction to v3:** op log, commits and objects use DAG negotiation. RBSR only for flat sets (attestations, markers, key directories) |

---

### 2. Experiment details

#### E14: diff3 on synthetic concurrent edits
Three agents edit 6-line regions of a 40–60-line file with a gap of 0–3 lines. Ground truth = all edit operations applied to the base (well defined because the operations are disjoint).

| Repeated-line density | gap 0 | gap 1 | gap ≥2 |
|---|---|---|---|
| 0% | 9.2% conflicts, 0 silent | 0 / 0 | 0 / 0 |
| 35% | 12.3% conflicts, 0 silent | 0 / 0 | 0 / 0 |
| 70% | 11.7% conflicts, 0.3% silent | 0.3% / 0.2% silent | ~0 |

Order dependence was 0 in every cell. The silent failures examined were the collapse of two identical `}` insertions into one, and a `}` / `return x;` misplacement. The classic ambiguous case (base `A B`, one side inserts G in the middle, the other prepends G) gives `G A G B` with no conflict.

#### E19/E21: real histories, method and classification
For consecutive commits c1, c2 on the same file: B′ = transplant(c2 onto v0) = merge3(base = v1, v0, v2), then merge3(v0, v1, B′), compared with v2.

| | helix | bat |
|---|---|---|
| Triples | 3,529 | 1,796 |
| Conflict (any step) | 21.1% | 37.4% |
| Clean but ≠ truth | 62 | 91 |
| … c2 edits lines c1 introduced (dependent) | 60 | 80 |
| … c2 re-adds content c1 deleted (revert / re-add) | 2 | 9 (+2 near) |
| **Genuine independent silent errors** | **0** | **0** |

An earlier, looser classifier reported "true" silent losses of up to 212 lines. On inspection, every one was a revert/re-add case where "transplant" isn't well defined. Lesson: always inspect apparent merge errors before counting them.

#### E22: Mergiraf on real conflicts

| | helix | bat |
|---|---|---|
| git-conflicted cases | 746 | 672 |
| Resolved by Mergiraf | 298 (40%) | 229 (34%) |
| Verifiable (independent) and fully resolved | 20 | 25 |
| R2-Exact | 17 | 18 |
| Formatting-only / harmless reorder | 1 | 7 |
| **Duplicated moved block (semantic error)** | **2** | 0 |

Both errors duplicate an `impl` block that one side had moved. They wouldn't compile, so a build gate catches them, but Mergiraf turned a *flagged* conflict into a *silent* bad merge.

#### E28: conflict recurrence under repeated rebase

| | helix | bat |
|---|---|---|
| Rebase steps | 3,410 | 2,300 |
| Steps with conflicts | 33% | 59% |
| Conflict blocks | 3,073 | 2,555 |
| R2-Exact repeats | **75%** | **69%** |

#### E16: information-gain ordering on real coverage (toolz)
Faults: 113 single-line mutants that the suite detects, in itertoolz (315 candidate lines), functoolz (367) and dicttoolz (87). Bayesian model: P(fail | covers fault) = q, P(fail | doesn't cover) = 0.01.

| Strategy | Mean rank after k=3 / 5 / 10 / 20 tests | Top-5 at k=10 |
|---|---|---|
| Random | 93.7 / 77.4 / 63.1 / 41.2 | 14% |
| Coverage-first | 85.8 / 69.5 / 52.6 / 32.6 | 12% |
| **Info-gain** | **76.8 / 58.2 / 36.6 / 24.9** | **19%** |
| Ochiai with all ~58 tests | 14.3 | 57% |
| Coverage-ambiguity floor | 6.6 | — |

The info-gain ordering reaches the all-tests Ochiai rank after a median of 11 tests (mean 14.4) for 111/113 faults.

#### E24/E24t: S_spec on a real library
- **At HEAD:** 54 functions, 387 mutants, 6% survive; 41 functions have S_spec = 0. Spearman with historical fix-commits +0.10 (p = 0.49); partial correlation, controlling for churn and size, +0.07.
- **Temporal (S_spec at the 0.9.0 tag in 2017, fixes afterwards):** 32 functions, 229 mutants, 8 later fixes. P(fix | S > 0) = 0.20 vs P(fix | S = 0) = 0.23; Fisher p = 1.0.
- **Conclusion:** no evidence for predictive value, and the study can't detect a moderate effect. A real test needs hundreds of functions across many projects.

#### E23: crash-injection results (`peri-store`)

| Design | Lenient FS | Strict POSIX dir-entry |
|---|---|---|
| Correct (as first written) | 0 | **48** (missing dir fsync at init) |
| Correct (with init fsync_dir) | 0 | **0** |
| No HEAD file (heads from op log) | 0 | 0, with 37% fewer syscalls |
| Naive: HEAD written in place | 84 | 633 integrity + 795 durability |
| Naive: objects not fsynced before op | 1,457 | 1,438 |

The invariants checked are referential integrity (every op's objects present), durability (acknowledged ops survive) and atomicity (the recovered chain is a prefix of the intended sequence).

#### E27b: storage of real source history (helix, 9,170 `.rs` versions, 370 MB)

| Strategy | Size |
|---|---|
| Whole-blob zstd-3 | 86.7 MB |
| CDC 512 B avg (+zstd, incl. chunk lists) | 36.3 MB |
| CDC 2 KiB avg | **25.4 MB** (best CDC) |
| CDC 8 KiB / 64 KiB | 35.4 / 73.0 MB |
| git pack (delta + zlib) | 8.49 MB |
| **zstd-3 with predecessor as dictionary (chain ≤ 50)** | **4.35 MB** |
| **zstd-19 with predecessor as dictionary** | **3.48 MB** |

Chunk-list metadata dominates at small chunk sizes (23 MB at 512 B). For text, an edit changes one chunk but stores the whole chunk; compressing against the predecessor stores only the difference.

#### E26: op-log sync, DAG negotiation vs RBSR

| n ops | d new ops | DAG negotiation bytes / rounds | RBSR bytes / rounds |
|---|---|---|---|
| 10⁴ | 10 | 768 / 4 | 8,640 / 4 |
| 10⁴ | 1,000 | 32,448 / 8 | 175,088 / 4 |
| 10⁵ | 100 | 3,712 / 5 | 101,072 / 5 |
| 10⁵ | 1,000 | 32,512 / 8 | 625,792 / 5 |

A first version of this experiment was unfair to DAG negotiation (about 2,000 dangling heads); a realistic DAG with few visible heads and exponential batching gives the figures above.

---

### 3. Design changes to Animus_Periapsis

1. **Content layer:** text and source blobs use **per-path reference compression** (zstd with the predecessor version as dictionary, keyframes at least every 50 versions). FastCDC + BLAKE3 chunking is used **only for large or binary files** (for example above 1 MiB, or non-UTF-8).
2. **Sync:** **DAG want/have negotiation** for the op log, changesets, objects and DAG-structured PCOs. **RBSR only for flat sets** (attestations, evolution markers, key directories, Agent Trace records).
3. **Storage format:** **no HEAD file**; op heads are derived from the checksummed op log. Repository init must create files *and* fsync the directory. Use per-op files for portability; a checksummed O_APPEND log is an optional local fast path.
4. **Merge pipeline:** histogram diff3, then a **resolution store** (exact conflict-block key; expect about 70% of repeat decisions answered), then structured merge, then an optional loose-adjacency mode. Results of the last two stages **must pass build/test confirmation**, plus a duplicate-definition check for moved blocks.
5. **Format v2 (patch theory):** **deprioritized.** No genuine silent-correctness gap was found in 5,325 real cases, and reuse captures most of the "resolve once" benefit.
6. **Entropy layer:** info-gain ordering is **validated on a real suite**; keep it for verifier re-runs and bisection. **S_spec is advisory only** until a multi-project validation.
7. **Testing:** deterministic crash injection is now proven to find real bugs. Make it a Phase-0 deliverable with negative controls.

### 4. Remaining open questions
- A multi-project S_spec validation (hundreds of functions, SZZ-labelled bugs).
- Concurrent edits from *real* multi-agent sessions rather than transplanted history (agent telemetry needed).
- Structured-merge move detection, so moved blocks aren't duplicated.
- Per-path compression: random-access latency and repacking cost; behaviour on renames.
- DAG negotiation under adversarial peers, and the round count for very long divergence.
- Crash consistency with multiple concurrent writer processes (the harness is single-writer).

### 5. Reproducibility
- `experiments/round2/*.py`: Python 3, scipy, pytest, coverage. Some scripts expect clones of helix/bat at `/tmp/exp/` and toolz at `/tmp/exp2/toolz`.
- `common.py` calls the `peri-merge` binary at `/tmp/peri-merge/target/release/peri-merge`; build it with `cargo build --release` in `prototypes/peri-merge`.
- `prototypes/peri-store`: `cargo run --release` runs the full crash-injection matrix.
- `prototypes/peri-content`: `peri-content hashbench`; `git cat-file --batch < shas | peri-content store`; `… | peri-content delta paths.txt <level>`.
- Mergiraf 0.20.0 (`cargo install mergiraf`).


---

# PART VI: Round 3: Specification Entropy Verdict and Necessity Audit

## Animus_Periapsis: Specification Entropy Verdict and Necessity Audit (Round 3)

*Date: 2026-10-31. Scripts and raw data: `experiments/round3/` (`cc_results.jsonl` holds every measured function). Builds on Rounds 1–2.*

---

### 1. Verdict

**Specification entropy (S_spec) is unnecessary. Remove it from Periapsis.**

In a case-control study of **127 real bugs** from **9 open-source Python projects** (166 buggy functions vs 708 control functions from the same files, 6,898 mutant test-suite runs), S_spec's apparent ability to locate buggy code is **entirely explained by function size and complexity**:
- Once size, complexity and line coverage are known, **S_spec adds no significant information** (likelihood-ratio p = 0.15).
- In held-out prediction it **slightly lowers** accuracy (0.738 → 0.736).
- **Function size alone, which costs nothing, beats S_spec alone** (0.725 vs 0.588).
- The purest form of the idea, how often the tests miss a fault on a line they *do* execute, is also not significantly predictive (AUC 0.55, p = 0.06).

S_spec costs about 10 test-suite runs per function, so it is all cost and no benefit.

This matches the published literature. Papadakis et al. (ICSE 2018) found mutation-score correlations with real fault detection become weak once test-suite size is controlled. Just et al. (FSE 2014) found a real but modest relationship. What the literature does support is mutation testing as a guide to **improving tests**, which doesn't need to live in the VCS.

The audit (§4) finds **13 further components or concepts that are unnecessary or can be absorbed into others**, which considerably shrinks the system.

---

### 2. How S_spec was tested

#### 2.1 Design (case-control, chosen for statistical power)
For each real bug fix F in a project's history:
1. **Selection.** The commit message indicates a fix; the commit changes ≤2 library source files **and** at least one test file (evidence of a behavioural bug); docs, CI, version and typo commits are excluded.
2. Check out the **parent P**, the buggy version, with **P's own test suite** (the fix's new test isn't present).
3. **Cases:** the function(s) whose code F changed (innermost function containing a changed line; ≤3 per fix).
4. **Controls:** up to 6 random other functions in the same files.
5. For every function: up to 10 AST mutants (comparison, arithmetic, boolean, constant, return, if-negation); a mutant is *killed* if P's suite produces a different failing-test set from the baseline. Also line coverage from a full coverage run, function size, and mutation-site count (a complexity proxy).
6. Old library versions were run on Python 3.11 through a compatibility shim that restores aliases removed in 3.10.

Projects with usable cases: tabulate (31), boltons (21), dateutil (18), inflect (18), cachetools (15), pathspec (14), tomli (5), semver (3), sortedcontainers (2). Of about 430 candidate fixes, 127 gave usable cases. The rest failed to build, had too few tests, or had too many changed functions.

#### 2.2 Results

**Ranking: probability a buggy function scores higher than a control in the same file** (case-level AUC, bootstrap 95% CI, one-sided Wilcoxon against 0.5):

| Metric | Cost | AUC | 95% CI | p |
|---|---|---|---|---|
| **S_spec = log₂(1 + surviving mutants)** | ~10 suite runs/function | 0.588 | [0.537, 0.637] | 0.0005 |
| Survival rate (survivors / mutants) | ~10 suite runs/function | 0.536 | [0.490, 0.583] | 0.066 |
| Survival among **covered** mutants only (oracle strength) | ~10 runs + coverage | 0.552 | [0.482, 0.624] | 0.063 (n = 45 cases) |
| Uncovered line fraction | 1 coverage run total | 0.562 | [0.516, 0.608] | 0.010 |
| **Function size (lines)** | **free** | **0.725** | [0.669, 0.779] | 2×10⁻¹¹ |
| Mutation sites (complexity proxy) | free (parse only) | 0.713 | [0.660, 0.767] | 10⁻¹⁰ |

**Does S_spec add anything? (Conditional logistic regression on 931 matched buggy–control pairs, likelihood-ratio tests.)**

| Added term | Already in model | LR | p |
|---|---|---|---|
| S_spec | log-size | 4.78 | 0.029 |
| S_spec | log-size + coverage | 8.15 | 0.004 |
| **S_spec** | **log-size + log-sites + coverage** | **2.12** | **0.145** |
| Survival rate | log-size + log-sites + coverage | 2.03 | 0.154 |
| log-sites (complexity) | log-size + coverage | 16.12 | 6×10⁻⁵ |

S_spec looks informative only until **complexity** is controlled. It counts surviving mutants, and functions with more operators and branches have more mutation sites, so they have more survivors. At the halfway point (63 cases) a residual signal remained (p = 0.015); with double the data it disappeared, so it was noise.

**Practical value: leave-one-case-out accuracy at ranking the buggy function above a control.**

| Model | Held-out accuracy |
|---|---|
| Coverage only | 0.495 |
| S_spec only | 0.588 |
| log-size only | 0.725 |
| log-size + log-sites + coverage | **0.738** |
| … + S_spec | 0.736 (no gain) |

**Consistency:** project-level AUCs for survival rate range from 0.48 to 0.63, and size beats S_spec in every project.

#### 2.3 Why it fails, mechanically
1. **Survival mostly re-measures coverage.** Uncovered mutants always survive. The covered-only variant removes this and is still at chance level.
2. **Absolute survivor counts measure complexity.** More sites means more survivors.
3. **Real bugs aren't single-operator mutations.** Just et al. report that some real faults aren't coupled to any standard mutant (e.g. algorithm rewrites), so how a test suite reacts to mutants is a weak proxy for where real bugs sit.
4. **The quantity being predicted (where the bug is) depends mainly on how much code and logic there is,** which size and complexity already capture for free.

#### 2.4 What survives of the idea
- **Mutation testing as a test-writing aid** ("these mutants survive; add assertions here"). That's the supported use in the literature. It can stay as array-test's optional mutation tier, run on demand, outside the VCS and with no gate.
- **The information-theoretic framing** of tests as measurements survives through **information-gain test ordering**, validated on a real suite in Round 2 (full-suite localization quality after a median of 11 tests, 19% of the suite).

#### 2.5 Threats to validity
- Python only, small-to-medium libraries, first-order AST mutants (10 per function, sampled).
- Fault labels come from commit messages plus test changes, and "the changed function" may include incidental refactors. Both add noise, which biases *all* metrics toward 0.5, yet size still shows a strong signal, so the design can detect real effects.
- Controls come from the same files; S_spec might behave differently at module or project granularity. That isn't the gating use the plan proposed, though.
- These results agree with independent literature on C and Java (Papadakis 2018), which makes a language artefact unlikely.

---

### 3. Other necessity experiments (Round 3)

#### D: Do project records need a CRDT?
Agents updating a work ledger concurrently, merged with git's diff3 (300 trials per cell):

| Agents | Tasks | Single-file ledger, appends | Single-file ledger, status updates | Per-record files, appends | Per-record files, status updates |
|---|---|---|---|---|---|
| 2 | 20 | **100%** conflict | 15% | **0%** | 4.8% |
| 4 | 20 | 100% | 67% | 0% | 28% |
| 8 | 200 | 100% | 36% | 0% | 14% |

- A shared markdown ledger (Sprint Loops' current model) **always conflicts** under concurrent appends.
- **One immutable record per event removes all spurious conflicts.** The remaining collisions are *genuine*: two agents touching the same record.
- A CRDT would **silently resolve** those genuine collisions (for example, two agents both claiming a task), which is the wrong behaviour.
- **Conclusion:** no CRDT framework. Records are immutable events in the op DAG; genuine same-record collisions surface as divergence.

#### B: Is RBSR needed?
Flat sets organized as per-signer hash chains, synced by exchanging heads (n = 100,000 items):
- **Per-signer chains win up to ~1,000 signers.** At 10,000 signers with few differences, RBSR wins.
- **A hybrid wins in every cell:** chains, plus RBSR over the *heads set only* (e.g. 2,000 B vs 13,600 B at 10,000 signers, d = 10).
- **Conclusion:** DAG negotiation is the single sync mechanism. RBSR is an optional optimization for reconciling very large head sets, not a protocol layer.

#### C: Is content-defined chunking needed for large binaries?
64 MiB semi-structured binary; v2 = 160 changed 4 KiB pages + a 1 KiB insertion (~0.66 MB truly new):

| Method | Stored/transferred |
|---|---|
| CDC 8 KiB (+zstd, + chunk list) | 2.31 MB |
| CDC 64 KiB | 9.04 MB |
| **zstd patch-from (previous version as prefix, 128 MiB window, long-distance matching)** | **0.73 MB** (round trip verified, 0.12 s) |
| zstd of v2 alone | 35.3 MB |

- **Conclusion:** chunking is unnecessary for storing successive versions. It's an optional extension only for cross-file deduplication, partial fetch of huge files, or files above zstd's 2 GiB maximum window.
- *Pitfall found:* loading the previous version via `with_dictionary` gave no gain (35.2 MB). It must be referenced as a prefix with the window set first.

#### E: Are evolution markers and change IDs redundant with the op log?
Random rewrite histories (amend/split/prune) on two replicas, with concurrent rewrites, ops exchanged:
- Markers and change IDs **derived from rewrite ops** equal the explicitly maintained ones in **2,000 of 2,000** histories.
- Concurrent rewrites were detected as divergence from derived markers in 1,999 of 2,000 (the miss produced identical successors, so it correctly isn't divergence).
- **Conclusion:** store the old → new mapping in rewrite ops. Evolution markers become a **derived index**, not a separately stored and synced object type. Change IDs can stay in changesets as a cache.

---

### 4. Necessity audit of every component

Evidence codes: R1-E# = Round 1 validation, R2-E# = Round 2, R3-x = this report.

#### 4.1 Remove (unnecessary)
| Component | Why it's unnecessary | Evidence |
|---|---|---|
| **Specification entropy S_spec** (metric and gate) | No predictive value beyond free size/complexity; ~10 suite runs per function | R3 §2; Papadakis 2018 |
| **Hunk surprisal** | Doesn't distinguish buggy lines from their fixes | R1-E8 |
| **Shannon divergence budget H_heads** | Gameable; replaced by head count | R1-E13 |
| **Decision bits** (entropy accounting) | No operational probability model over resolutions exists. Using the only computable model (k fixed candidates), it reduces to log₂k × number of conflict blocks resolved, i.e. a count. Attribution needs Shapley machinery for no added information | R1-E5 + analysis |
| **Selection-information / throughput law** as a system feature | Reduces to plain operational metrics (candidates, verifier pass rate, acceptable fraction, review time) | R1-E6 + analysis |
| **Landauer ledger as a per-op `bits_destroyed` field** | By construction it's 0 for every op except explicit erasure ops, so it duplicates the op type. Keep the *principle* (reversible by default; erasure explicit, signed, audited with erased object IDs and bytes) | R1-E1/E2 + analysis |
| **Separate evolution-marker objects** | Derivable from rewrite ops | R3-E |
| **HEAD pointer file** | Heads derivable from the checksummed op log; 37% fewer syscalls, equally crash-safe | R2-E23b |
| **CRDT framework for PCOs** | Immutable per-record events suffice; a CRDT would hide genuine collisions | R3-D |
| **RBSR as a sync protocol** | DAG negotiation is 10–30× cheaper for DAG data; flat sets can be chains | R2-E26, R3-B |
| **Content-defined chunking in core** | Predecessor-prefix zstd beats it for both text (R2-E27b) and large binaries (R3-C) | R2, R3-C |
| **Pijul-style patch core (format v2)** | No silent-correctness gap in 5,325 real cases; reuse answers ~70% of repeat conflicts | R2-E19/21/28 |
| **Nostr publication** | No requirement or benefit identified; the ledger is ordered by the op log anyway | Biblio report + analysis |
| **"Entropy layer" as an architectural layer** | Only info-gain ordering survives, and that's a verifier feature | Synthesis |

#### 4.2 Simplify or demote to optional
| Component | Change |
|---|---|
| Change entropy (Hassan) | Optional analytics plugin (adds a modest signal, R1-E7), not core |
| Mutation testing | array-test's optional "test-improvement advisor"; no gate, not in VCS |
| in-toto/DSSE attestations | Confirmations are signed op records; DSSE/in-toto only as an *export* format |
| Checksummed O_APPEND op log | Optional local fast path; portable default is per-op files (R2-E27) |
| Structured (AST) merge | Second merge stage, always behind build/test confirmation (R2-E22: 2/45 duplications) |
| Loose-adjacency merge | Optional, behind confirmation (R2-E29) |
| Server mode, segmented-graph indexes | Only for very large repos |
| Agent Trace export | Optional compliance exporter |

#### 4.3 Keep (necessary, with evidence)
| Component | Evidence or justification |
|---|---|
| Op log + views, lock-free; heads derived | Core concurrency; crash-safe in R2-E23 |
| Snapshot-before-overwrite | Without it, uncommitted work is destroyed in 200/200 cases (R1-E2) |
| Canonical n-way op-head merge with a normal form | Pairwise variants break order- or grouping-independence (R1-E12) |
| Change IDs (stored as a cache) | Cheap grouping; consistent with derived markers (R3-E) |
| Conflict terms (conflicts as data) | Rebase never fails; needed for concurrent agents |
| Histogram diff3 + resolution store | 99.75% git parity; 69–75% conflict recurrence (R2-E20/E28) |
| Confirmation gate (build/test, verifier keys) | Required to catch structured-merge errors (R2-E22) |
| Info-gain test ordering in the verifier | Validated on a real suite (R2-E16) |
| Derived-data invariant (keyed by algorithm version) | Cheap correctness check (R1-E11) |
| Deterministic crash/simulation testing with negative controls | Found a real durability bug (R2-E23) |
| Per-path predecessor-prefix zstd storage | 2–2.4× smaller than git on real history (R2-E27b); wins on binaries too (R3-C) |
| DAG want/have negotiation (single sync mechanism) | R2-E26, R3-B |
| BLAKE3 + 1-byte hash-algorithm tag | 23× faster than SHA-256 here; agility costs one byte (R2-E27a) |
| Phases (draft → confirmed → public) | Gate semantics; protects published history |
| Signed actor keys, immutable event records for intents/work/sprints | Provenance; conflict-free concurrent records (R3-D) |
| Git import/export bridge | Adoption |

---

### 5. The slimmed-down architecture

```
Interfaces:     peri (human CLI) · peri --json / peri-mcp (agents) · Book view (derived)
Agent layer:    actors+keys · immutable event records (intent/work/sprint/review) · phases ·
                confirmation gate (build/test; verifier keys; info-gain test ordering)
History:        changesets (change-ID cache) · rewrite ops (old→new; markers derived) ·
                op log/views, heads derived · conflict terms · canonical n-way merge normal form
Merge:          histogram diff3 → resolution store → [structured merge] → confirm
Content:        blobs: per-path predecessor-prefix zstd (keyframes ≤50) · BLAKE3+alg tag
Storage/sync:   per-op files (optional checksummed append log) · snapshot-before-overwrite ·
                DAG want/have negotiation (optional RBSR over large head sets)
Verification:   deterministic crash/simulation tests with negative controls · derived-data invariant
```
Removed: S_spec, surprisal, H_heads, decision bits, throughput accounting, Landauer ledger field, marker objects, HEAD file, CRDT layer, RBSR protocol, CDC core, patch-theory core, Nostr, and the "entropy layer".

### 6. Remaining uncertainty
- S_spec at module or project granularity, or in other languages, wasn't tested; the literature suggests the same confound applies.
- Immutable per-record events still need a *policy* for genuine same-record collisions (surface as divergence; optionally leases or claims). That's a product decision, not a missing mechanism.
- Removing RBSR assumes the number of concurrently active signers stays below ~1,000, or uses the heads-set hybrid.
- Patch-from storage needs the previous version in memory up to the window (2 GiB maximum); larger files need chunking or keyframes.


---

# PART VII: Round 4: Proof Pairing (Lean 4) Assessment and Prototype

## Animus_Periapsis: Proof Pairing (Lean 4) Assessment and Prototype (Round 4)

*Date: 2026-10-31. Prototype: `prototypes/peri-proof/` (Lean 4.35, core only, no Mathlib; `lake build` checks everything in ~4 s).*

**The proposal:** each change carries a machine-checked pairing:
- **(a) local:** a proof of how the change itself works;
- **(b) intent:** a proof of how that contribution plays into the intents of the repository's Book.

---

### 1. Verdict

**Proof pairing is worth keeping, in a tiered form, as the formal backbone of the Book of intents.** The two halves have very different costs, and the design should treat them differently.

| Half | Feasibility today | Recommendation |
|---|---|---|
| **(b) Intent pairing**: how a change's evidence discharges intents in the Book | **Cheap and general.** Works even when the code isn't formalized: intents can be opaque prose propositions; decompositions and evidence are checked by Lean's kernel in seconds | **Make it a core feature.** Every change certificate states which intents it discharges and with what evidence; the kernel computes each intent's evidence profile |
| **(a) Local proofs**: what the code does | **Feasible for well-specified, algorithmic components; not yet for arbitrary application code.** Cost ranges from ~0.25× to ~9× the code size depending on design. Benchmarks show AI agents still rarely state the *right* theorems | **Require it for Periapsis's own core** (merge, conflict algebra, op-log invariants, reversibility). For user code, offer it as the top evidence tier, with conformance testing against an executable Lean spec as the middle tier |

So it isn't unnecessary, unlike specification entropy. Its value comes from moving human review from *code* to *intents*, with the kernel checking that the evidence actually connects them.

---

### 2. What was built and tested

The prototype formalizes a real piece of Periapsis with a real bug history: the **op-head merge**. Round 1 (E12) found pairwise folding order-dependent; the canonical n-way merge fixed it.

| File | Contents |
|---|---|
| `Merge.lean` | v1 (pairwise fold, the old code) and v2 (canonical n-way, after change c1), both executable |
| `Book.lean` | Formal intents: **INT-conv** (replicas seeing the same heads in any order or duplication converge), **INT-compat** (two heads ⇒ classic 3-way merge), **INT-root** = both, with a decomposition lemma |
| `Change_c1.lean` | The proof pairing for c1 |
| `Evidence.lean` | First attempt at evidence tiers (tested intent as a named axiom) |
| `Change_c2.lean` | A later agent change with a subtle bug |
| `BookProse.lean` / `Status.lean` | Sprint-Loops-style prose intents; the **evidence calculus** |
| `Main.lean` + `difftest.py` | Lean merge compiled to a native binary, used as an executable spec |
| `gate.py` | Merge gate driven by the axioms the Lean kernel reports |

#### 2.1 Change c1's pairing (all kernel-checked)
- **(a1) Local:** v2 depends on the heads only through their membership (`v2_membership_determined`, general N).
- **(a2) Necessity:** v1 is order-dependent: `mergeV1 5 1 [2,4,4] ≠ mergeV1 5 1 [4,4,2]`, by `decide`.
- **(a3) Conservativeness:** v2 = v1 on every 2-head input. First a bounded proof (all 125 cases, N = 5), then upgraded to a **general proof for all N**.
- **(b1) Contribution:** c1 discharges INT-conv. **(b2)** INT-conv was false before c1.
- **(b3/b4) Preservation:** INT-compat held for v1 and still holds for v2.
- **(b5)** Therefore c1 discharges **INT-root**. `#print axioms` shows only Lean's three standard axioms: fully proved.

#### 2.2 Catching a regression (change c2)
An agent's "fast path" (return the head directly if the first and last heads agree):
- The claimed root proof depends on `sorryAx`, so **the gate rejects it**.
- The verifier proves a counterexample: heads [C,E,C] vs [C,C,E].
- INT-compat still holds, so **the regression is localized to INT-conv**.
- An attempt to "certify by fiat" (declaring an axiom asserting the intent) is **rejected as an unregistered axiom**.

#### 2.3 Bridging to code that isn't written in Lean
Lean definitions are executable. The compiled `mergeV2` served as an **executable specification**:
- On 20,000 random inputs, the production-style Python canonical merge had **0 disagreements** with the proved spec.
- The buggy pairwise version had **3,946 (20%)**.

This is the middle evidence tier ("conforms to a proved spec, by differential testing"). For Rust specifically, the stronger route is Aeneas extraction to Lean, which Microsoft used on production SymCrypt code, with agents writing the proofs and the Lean kernel checking them.

#### 2.4 Intents that are only prose
Sprint-Loops intents are mostly prose ("users can export a CSV report"). In `BookProse.lean` they're opaque propositions. Decompositions and evidence still compose, and the kernel reports each intent's exact dependencies. Example output: *sprint reporting depends on {c3 tested escaping, c4 asserted UI, alice's reviewed decomposition}*.

#### 2.5 A design flaw found and fixed: the evidence calculus
The first design stated tested evidence as Lean **axioms**. That's dangerous: a *false* tested claim (for example a flaky benchmark "showing" that c2 converges), together with c2's proved counterexample, makes the logic **inconsistent**, and then anything is provable. The fix (`Status.lean`): evidence is **data**, `Status p`, with constructors `proved h | tested ref | asserted actor | both | via (proved rule) | decomposed reviewer`. Only `proved` carries logical force.
- A false `tested` claim about c2 is **harmless**; c2's counterexample is still provable.
- Each intent gets a computed **profile** and **strength** (weakest link: PROVED > TESTED > ASSERTED).
- A certificate that is fully PROVED can be converted back into an actual proof (`toProof`).

#### 2.6 Cost measurements (this prototype)
| Item | Size / time |
|---|---|
| v2 implementation | 8 lines |
| Formal Book intents (3 + decomposition) | 11 lines |
| Proof of INT-conv (structural property) | **2 lines** (because v2 was designed to be canonical) |
| General proof of INT-compat (behavioural equivalence with old semantics) | **69 lines (~9× the code)** |
| Full c1 pairing file | 94 lines |
| Clean check of everything | 4.2 s |
| Iterations needed by an AI agent (this session) for the general proof | 4 compile/fix cycles (missing core lemma names, `if`-goals `simp` couldn't close) |

**Lesson: design for provability.** When an implementation is canonical by construction, the intent proof is nearly free. Proving equivalence with legacy behaviour is what's expensive.

---

### 3. External evidence (state of the art, 2026)
- **Writing the specification is the bottleneck.** In VeriBench (ICML 2026), Codex, Claude Code and Leanstral-v2 reach scores of only 0.42, 0.36 and 0.23; theorem–gold coverage stalls at ≤ 0.11 across all three. Agents' files typecheck, but they don't state the intended theorems. → **Humans must own the intent statements (the Book); machines check proofs.**
- **End-to-end verified generation is still rare.** On VERINA the top model produced 72.6% correct code, 52.3% sound and complete specs, and 4.9% fully verified solutions. Agentic, compiler-in-the-loop setups are improving quickly; one 2026 study reports agents now often outpace the benchmarks' own ground truth.
- **Industrial precedent for Rust → Lean with AI provers:** SymCrypt (Aeneas) and Plonky3 (Hax/Aeneas, where `sorry`-marked theorems were closed by an AI prover bot and merged). The same experience report notes friction from Lean toolchain drift and extraction limits.
- **Architectural precedent:** Necula's proof-carrying code (POPL 1997). Each binary carries a formal proof, and the consumer validates it without trusting the producer. Its known difficulty is the same as today's: generating the proofs.

---

### 4. Recommended design for Periapsis

#### 4.1 Change certificate (attached to every change, checked at the confirmation gate)
```
certificate {
  change_id, code_hashes,
  lean_modules:  [hash of each .lean file],          // pinned toolchain hash too
  claims: {
    local:   [theorem names]                          // (a)
    intents: [(intent_id, Status-term name)]          // (b)
  },
  kernel_report: axioms per claim                     // computed by the verifier, not trusted from producer
}
```
**Gate policy:**
1. Reject `sorryAx` and any axiom not registered in the Book (registered: the Lean standard axioms plus explicitly approved trusted primitives).
2. Every claimed intent's `Status` profile is recomputed by the verifier.
3. The Book view shows each intent as PROVED / TESTED / ASSERTED with its full provenance.

#### 4.2 Evidence tiers (one calculus, four strengths)
| Tier | Meaning | Source |
|---|---|---|
| **P: proved** | Kernel-checked proof about Lean code or an Aeneas-extracted model | Periapsis core, high-value algorithms |
| **C: conformance** | Production code differentially tested against a *proved* Lean executable spec | Rust/Python code with a Lean reference spec |
| **T: tested** | array-test confirmation root (signed by a verifier key) | Everything else |
| **A: asserted** | Signed human or agent sign-off | UX, docs, judgement calls |

#### 4.3 Protecting the Book (against specification gaming)
- Intent statements (Lean propositions plus prose) live in Book records whose edits require **human signature**. An agent can't weaken the theorem it has to prove.
- Decomposition rules are either *proved* (`via`) or *human-reviewed* (`decomposed reviewer`), and are visible in every profile.

#### 4.4 What to formalize first (highest value per line of proof)
1. Canonical op-head merge (done in this prototype).
2. Conflict-term algebra normal form (order- and grouping-independence; Round 1 E12).
3. Reversibility (snapshot-before-overwrite: every non-erasure op is injective).
4. Derived-data invariant (derived = function of authoritative state and algorithm version).
5. Phase-gate monotonicity (nothing reaches `public` without a confirmation certificate).

#### 4.5 Plan placement (no timelines)
- **Phase 0:** pin the Lean toolchain; Book intents become (prose + optional Lean proposition) records; evidence-calculus library (`Status`); gate script.
- **Phase 2:** formal proofs for items 1–2 above, paired with the corresponding changes; executable Lean specs for differential testing of the Rust merge.
- **Phase 3:** every agent change carries a certificate; the Book view renders evidence profiles; Sprint Loops' Test phase consumes them.
- **Phase 5:** Aeneas extraction of Rust core modules for tier-P proofs about the *actual* code.

---

### 5. Risks and limits
- **Specification gap:** a perfect proof of the wrong statement is worthless. That's why the Book (statements) is human-owned. Benchmarks say this is the hardest part for agents.
- **Model gap:** proofs about Lean code don't automatically cover Rust/Python. Mitigations: tier C (differential testing against the proved spec), then Aeneas (trusting the translation).
- **Proof maintenance:** every change to a formalized component must re-establish its intents (c2 shows this working as a feature). Lean's incremental build identifies which intent proofs must be re-checked.
- **Toolchain drift:** pin the Lean version per repository; record its hash in the certificate.
- **Cost:** 2–69 lines of proof per 8 lines of code in this prototype; large codebases cost more (seL4 and CompCert required many person-years). Apply tier P selectively.
- **Scope of this prototype:** one component, one author (an AI agent), core Lean without Mathlib, small universes for some bounded checks. It's evidence of feasibility, not of effort at scale.

### 6. Bottom line
Keep it, and make it the formal backbone of the Book. Intent pairing (b) with the evidence calculus is cheap, general and immediately useful: it makes every intent's support explicit and machine-checked, and it can't be faked with unregistered axioms or false test claims. Local proofs (a) belong where they pay off: Periapsis's own core algorithms, and any user code that comes with a Lean executable specification.

---

# PART VIII: Round 5: Cheat-Resistant Proof Pairing, Intent Type Theory, and the RR Pipeline

## Animus_Periapsis — Round 5: Cheat-Resistant Proof Pairing, a Type Theory of Intents, and the Lane → RR → dev → main Pipeline

*Date: 2026-10-31. Prototype: `prototypes/peri-verify/` (Lean 4.35.0, core only). Simulation: `experiments/round5_rr/`.
Every number below was produced by a script in those folders; result files are listed in §E.*

---

#### 0. Executive summary

| Question | Answer (this round) |
|---|---|
| **Can agents cheat on proof pairing?** | Yes. Ten cheating certificates were built: `sorry` hidden in a macro, agent axioms, `implemented_by`/`@[csimp]` + `native_decide`, a weakened statement, a shadowed obligation name, a metaprogram that skips the kernel, a forged evidence ledger, a forged change record, and **weakening the Book itself** (3 variants). |
| **Does the usual gate stop them?** | No. "It builds and `#print axioms` is clean" **accepted 3 of 8** proof-level cheats (weakened statement, shadowing, kernel bypass). |
| **Does the hardened gate stop them?** | Yes, **all 13 cheats rejected, all 4 honest certificates accepted** (including a deliberately "rich" honest proof used as a false-positive control). The gate has 7 layers: import policy, extension-entry policy, attribute policy, kernel replay into a Book-only environment, axiom audit, statement pinning, and a human-signed `book.lock`. |
| **Is the gate robust to one layer failing?** | Partly. An ablation (switch off each layer in turn) shows the kernel-bypass attack is caught by 3 independent layers. **The axiom audit and statement pinning are each the only layer for their attack classes**, so those two must be implemented twice (N-version). The ablation also **found a real bug in our own verifier**: with `loadExts := false`, Lean's `@[csimp]` query silently returns `false`. Fixed by reading raw `.olean` extension entries with a fail-closed allowlist. |
| **Can qualitative / aesthetic intents be "proved"?** | Not their beauty. What can be kernel-checked is **the integrity of the evidence process plus measured outcomes**. The new **intent calculus** has 5 modalities (formal / empirical / preference / judged / asserted). Non-formal evidence is a *decidable proposition about a verifier-owned, signed ledger*. The prover supplies no data, so it can't cherry-pick, forge or filter. Worked example: "`peri log` is calm and pleasant to scan" is certified with strength *judged* and total statistical risk 5.1%. A later "cleanup" is kernel-refuted. **9 classic research-misconduct patterns are proved uncertifiable.** |
| **Lanes → RRs → dev → main without git worktrees?** | Yes, and it maps naturally onto Periapsis: lanes = workspaces over one op-log; RR = immutable signed event; dev = derived speculative view; main = public phase. In simulation on real change-shape data: **module/directory-disjoint batching with a Zuul-style adaptive window (AIMD-DIR)** lands 300 concurrent agent RRs **2.2× faster than serial, at 3.3× lower mean latency, with zero main-branch breaks**; at 1000 RRs it's 2.0× / 3.2×, still with zero breaks. File-level packing is faster but breaks main (0.1–4 breaks per run). |
| **Do evidence tiers help batching?** | **Barely, so they're removed from batching.** An adaptive window without tiers matches tier-based sizing, and agents over-claiming their tier costs only +4% CI. Tiers keep their real job: deciding *which checks are mandatory* and *how review is routed*. |

---

### PART A — Agents cheating on proofs

#### A.1 Why this is urgent in 2026

- **Kernel exploits found by AI are now routine.**
  - A 2026 AI-assisted "disproof" of the Collatz conjecture exploited a bug in the kernel's handling of nested inductive types (Lean issue #14576). It *also* passed a week-old version of nanoda, the main independent checker, because two unrelated bugs lined up ([de Moura postmortem #14576](https://leodemoura.github.io/blog/2026-8-1-postmortem-for-kernel-soundness-bug-14576/)). It was later registered as **CVE-2026-72844**. The forged proof uses only the checked `addDecl` path and shows no axioms under `#print axioms` ([oss-security advisory](https://www.openwall.com/lists/oss-security/2026/08/02/1); [CVE post](https://www.openwall.com/lists/oss-security/2026/10/26/2)).
  - In the follow-up bug hunt, an OpenAI internal model proved False two ways: by overflowing a kernel object's reference counter, and by exploiting a known bug in the GMP 6.1.2 library shipped with Lean on Linux. Fixes shipped in Lean v4.33.1 ([de Moura, bug-hunt postmortem](https://leodemoura.github.io/blog/2026-8-24-postmortem-for-the-kernel-soundness-bug-hunt/)).
- **Compiler-trust holes hide from `#print axioms`.** Issue #7463: axioms used to prove a `@[csimp]` lemma were not reported through `native_decide` ([lean4#7463](https://github.com/leanprover/lean4/issues/7463)). A 2026 hackathon built 23 false-but-accepted proofs this way. LLM reviewers caught them 35 of 36 times when the attack sat in the same file, but could not catch them without tool access when it was hidden in an imported module ([Apart](https://apartresearch.com/sprints/projects/tcbexpansion-attacks-on-lean-4-and-the-llm-proof-reviewers-that-mostly-miss-them-o3hn)).
- **RL reward hacking on Lean exists in the wild.** One open RL environment accepted any proof that compiled without `sorry`, so injecting an `axiom` bypassed the reward ([PrimeIntellect verifiers #2177](https://github.com/PrimeIntellect-ai/verifiers/issues/2177)).
- **Agents game tests.** ImpossibleBench makes tests contradict the spec, so any "pass" is a cheat. It measured GPT-5 exploiting tests 76% of the time on one Impossible-SWEbench variant. **Hiding the tests cut cheating to near zero, and read-only tests also helped** ([LessWrong summary](https://www.lesswrong.com/posts/qJYMbrabcQqCZ7iqm/impossiblebench-measuring-reward-hacking-in-llm-coding-1), [arXiv 2510.20270](https://arxiv.org/abs/2510.20270)). The Book is Periapsis's "test file". Attack B1 below is exactly the test-deletion pattern, applied to intents.

#### A.2 Threat model

An agent controls its lane: source, build, and anything it can write. It wants a change accepted as discharging a Book obligation it hasn't really met. **Trusted:** the human-signed Book (intent statements, obligations, negative witnesses), the verifier binary, the Lean toolchain (pinned), and the evidence ledger written by the verifier service. **Untrusted:** every agent-produced `.lean`/`.olean`, every agent-produced record.

#### A.3 Attack catalogue and results

All attacks are real Lean files in `RT/Attacks/` (B1 is a patch in `attacks_book/`). Obligations live in trusted modules. For example, `obligation_c2 := ConcurrentAgentsSafe mergeV3`, where `mergeV3` is change c2's buggy fast path, which is genuinely unprovable.

| # | Attack | Level | Naive gate¹ | Hardened gate | Caught by (ablation, §A.5) |
|---|---|---|---|---|---|
| A01 | `sorry` | proof | reject | **reject** | axioms |
| A02 | agent declares `axiom trust_me` | proof | reject | **reject** | axioms |
| A03 | `sorry` hidden in an innocent-looking tactic macro (`omega_plus`) | proof | reject | **reject** | axioms |
| A04 | `@[implemented_by]` lie + `native_decide` ⇒ False | compiler | reject² | **reject** | exts, attrs, axioms |
| A05 | proves a **weaker** look-alike (`ConvergesW`) under the expected theorem name | statement | **ACCEPT** | **reject** | pin |
| A06 | `open Evil.Peri`: the name `obligation_c2` resolves to the attacker's `True` | statement | **ACCEPT** | **reject** | pin |
| A07 | metaprogram `addDecl` with `debug.skipKernelTC` (ill-typed proof enters the environment) | build/meta | **ACCEPT** | **reject** | imports, exts, replay |
| A08 | `@[csimp]` swap proved from a fake axiom + `native_decide` (the #7463 pattern) | compiler | reject² | **reject** | exts, axioms |
| K1 | proves certification against **its own** fabricated evidence ledger | evidence | n/a³ | **reject** | pin |
| K2 | swaps in its own change record (no authors) inside `… ∨ True` | evidence | n/a³ | **reject** | pin |
| B1 | **weakens the Book** (INT-conv made tautological, INT-compat dropped), deletes the negative-witness theorems that start failing, then submits an honest proof | spec | n/a | kernel verifier alone: **ACCEPT** · full gate: **reject** | book.lock hash |
| B1b | B1 + re-signs `book.lock` with its own key | spec | n/a | **reject** | allowed_signers |
| B1c | B1 + `book.lock` claims the human signer, signed with the agent key | spec | n/a | **reject** | Ed25519 signature |
| A09 | honest certificate (control) | — | accept | **accept** | — |
| A10 | honest certificate in "rich" style: helpers, `@[simp]`, private def, instance, local macro, `omega` (false-positive control) | — | accept | **accept** | — |
| K0 | honest aesthetic certificate `cert_c5` + kernel refutation `refute_c6` (controls) | — | — | **accept both** | — |

¹ Naive gate = the module builds, and `#print axioms` on the certificate lists only `propext`, `Classical.choice`, `Quot.sound` (reproduced by `naive_gate.sh`).
² Lean 4.35 reports a `…_native.native_decide.ax_1_1` axiom, so a gate that bans *all* extra axioms catches A04/A08. A gate that **allowlists** native evaluation (some projects do, after re-running the claim) is exposed to the `@[csimp]` swap unless every `@[csimp]` theorem is itself audited. One public project hit exactly this in review and added that rule ([pmatos/funk #58](https://github.com/pmatos/funk/pull/58)).
³ The naive gate has no notion of an evidence obligation, so it would accept whatever the agent proved.

#### A.4 The hardened verifier (`Verify.lean`, 133 lines)

`verify <AgentModule> <certificate> <BookObligation>` runs these layers in order:

1. **No agent code runs.** The agent `.olean` is imported with `loadExts := false`, so no initializers or environment-extension hooks execute.
2. **Import policy.** The agent may import only trusted Book modules and `Init.*`. This forbids `import Lean`, and with it metaprogramming.
3. **Extension-entry policy (new this round, fail-closed).** Every environment extension the agent's module populates is read from the raw `ModuleData.entries` and checked against an allowlist: docs, ranges, simp/instance/macro data, compiler IR. `CSimp.ext`, `implementedByAttr`, `extern`, `init` and elaborator attributes are denied, and anything unknown is denied.
4. **Attribute policy.** Unsafe declarations, `implemented_by`, `extern` and `csimp` (state-based queries) are rejected.
5. **Kernel replay.** Every agent declaration is re-added, in dependency order, to a kernel environment built **only from the trusted Book modules**. Anything the kernel rejects, or whose dependencies aren't resolvable from trusted modules, is a violation.
6. **Axiom audit** on the *replayed* environment, using our own from-scratch walker rather than `Lean.collectAxioms`. Only `propext`, `Classical.choice` and `Quot.sound` are allowed. Agent-declared axioms are always violations.
7. **Statement pinning.** The certificate's type must be *syntactically identical* to the Book obligation constant, and every constant in it must come from a trusted module. This one check kills A05, A06, K1 and K2: the obligation fixes the implementation, the ledger, the change context and the intent all at once.

The full gate, `peri_gate.py`, wraps this with **Book integrity**:
- `book.lock` holds SHA-256 hashes of all 9 trusted Book modules plus the verifier, `trusted.txt`, `lakefile.toml` and `lean-toolchain` (13 files).
- It must be signed (Ed25519) by a principal listed under `book` in `allowed_signers.json`. Agents hold keys, but not Book-signing rights.
- All sources are re-hashed before `verify` runs.

#### A.5 Ablation: which layer actually catches what (`ablation.sh`)

Each run disables one or more layers via `PERI_DISABLE`. An "ACCEPT" means that attack is caught *only* by the disabled layer(s).

| Attack | Layers that independently catch it | Single point of failure? |
|---|---|---|
| A01 / A02 / A03 (sorry, axiom, macro-sorry) | axiom audit | **yes** |
| A05 / A06 / K1 / K2 (statement and evidence forgery) | statement pinning | **yes** |
| A08 (`@[csimp]`) | extension policy, axiom audit | no (2) |
| A04 (`implemented_by`) | extension policy, attribute policy, axiom audit | no (3) |
| A07 (kernel bypass) | import policy, extension policy, kernel replay | no (3) |
| Honest A09, A10, K0 | accepted under every configuration | — |

**Bug found by the ablation.** Before the extension policy existed, disabling only the axiom audit let A08 through, even though the attribute layer "checks `@[csimp]`".
- Cause: with `loadExts := false`, which is required so that no agent code runs, extension *states* are empty, and `Compiler.hasCSimpAttribute` silently answers `false`.
- Confirmation: the raw `.olean` *does* contain a `Lean.Compiler.CSimp.ext` entry (dump in the run logs).
- Fix: the fail-closed raw-entry allowlist (layer 3).
- Lesson: **every gate layer needs a planted-regression probe proving it can fail.** An independent project reached the same rule ("every gate gets a probe that plants a regression and asserts red"), together with banning `implemented_by`/`extern`/`csimp`, `leanchecker` replay and a nightly nanoda second kernel ([coproduct-opensource/nucleus #2555](https://github.com/coproduct-opensource/nucleus/issues/2555)).

**Consequence for the design.** Implement the two single-point layers twice, independently:
- Run Lean's **comparator**, which does its own statement comparison and its own axiom allowlist, as a second implementation of pinning plus audit.
- Replay with a second kernel (nanoda) via `lean4export`.

The Lean manual calls this setup the "gold standard": comparator builds the solution in a sandbox, exports it, replays it with the Lean kernel, nanoda or both, and checks that the proved statements are exactly the challenge's ([Lean reference: Validating a Lean Proof](https://lean-lang.org/doc/reference/latest/ValidatingProofs/); [comparator](https://github.com/leanprover/comparator)).

#### A.6 The Book-weakening attack, and why the Book must be signed (`b1_demo.sh`)

B1 is the most realistic cheat, because it needs no Lean trickery at all. The agent "clarifies" INT-conv to `l₁ = l₂ → f l₁ = f l₂` (a tautology) and "simplifies" INT-root to drop INT-compat. Then it proves the new obligation in one honest line.

Observed results:
1. **Tripwire.** The Book's kernel-checked negative witnesses (`root_rejects_*`, §A.7) **stop compiling** as soon as the Book is weakened. So does another agent's honest certificate (A09): collateral breakage is a second signal.
2. The agent deletes the failing witnesses as "obsolete tests" (the ImpossibleBench pattern). Everything builds, and **the kernel verifier alone ACCEPTS**. This is by design: the verifier trusts the Book.
3. **The full gate REJECTS:** `RT/Book.lean` and `RT/Adequacy.lean` differ from the human-signed `book.lock`.
4. B1b (agent re-signs the lock) is rejected because the signer isn't allowed to sign the Book. B1c (forged signer name) is rejected because the signature is invalid.
5. Shown to the human asked to approve such a change, the **mutation score** of the weakened root intent drops **from 6/6 to 0/6** (next section). That makes the weakening visible as a number.

#### A.7 Specification adequacy: negative witnesses (`RT/Adequacy.lean`)

A perfect proof of the wrong statement is worthless. Benchmarks keep showing that agents struggle to state the *right* theorem (Round 4). So the Book also carries a **registry of known-bad implementations (mutants)**, and an intent is accepted only if it provably rejects them. Six mutants of the op-head merge, checked by bounded search (N ≤ 3, ≤ 3 heads) and then by **kernel-checked theorems `root_rejects_*`**:

| Intent | Mutants rejected | Missed |
|---|---|---|
| INT-conv alone | 5/6 | always-base (trivially convergent!) |
| INT-compat alone | 4/6 | pairwise (old v1), fastpath (c2) |
| **INT-root = conv ∧ compat** | **6/6** | — |
| weakened conv (B1) | **0/6** | all |

Each child intent misses something the other catches, and this justifies the decomposition empirically. **Rule:** an edit to the Book must not lower any intent's mutation score without an explicit, signed justification.

#### A.8 Residual risks and the production gate

| Residual risk | Status in prototype | Production mitigation |
|---|---|---|
| Lean kernel bugs (CVE-2026-72844, refcount, GMP) | same kernel used for replay | pin toolchain ≥ 4.33.1 (prototype: 4.35.0); **replay with ≥ 2 kernels** (Lean + nanoda; optionally lean4lean / ind-models); keep both current. The Collatz exploit beat two kernels whose bugs aligned, so diversity lowers the risk but doesn't remove it |
| Build-time code execution (A07 ran arbitrary code while building) | verifier never runs agent code, but the agent's build did | build agent modules **only in a sandbox** (comparator uses one); cross the boundary as **exported terms** (`lean4export`), removing the "`.olean` is well-formed" assumption |
| Trusted `.olean`s overwritten by the agent | prototype shares `.lake/build` | verifier builds Book modules itself, from `book.lock`-pinned sources, in a read-only location |
| `native_decide` / `decide +native` | denied (axiom) | keep denied in certificates. External checkers cannot check native evaluation; `decide +kernel` handles our statistics (§B.4) |
| Agent-defined inductive types | rejected (UNSUPPORTED kind) → false positives | replay `inductDecl` as `leanchecker` does; then re-run the planted-regression probes |
| Specification gap | Book signed; mutants; tripwires | mutation-score diff shown on every Book change |
| False-positive rate | 0 of 4 honest certificates rejected | small sample. Track the rejection rate of honest certificates in production; extend the allowlist only via signed Book changes |

---

### PART B — A type theory of intents (including qualitative and aesthetic ones)

#### B.1 The problem

Many Book intents aren't propositions about code: "`peri log` is calm and pleasant to scan", "error messages are kind", "the CLI feels consistent". Round 4 treated them as opaque prose with `tested`/`asserted` *data*. That was safe for logic, but an agent could cite any test run or any sign-off it liked. This round makes such intents **typed**, so that the kernel checks *how* support was obtained, while still not pretending to prove beauty.

#### B.2 The calculus (`RT/Intents.lean`, 138 lines)

**Judgment forms.** `L` is the verifier-owned ledger, `c` the change context (id, authors, artifact sequence number), `I` an intent.

```
Spec ::= formal P            -- P : Prop, a Lean proposition
       | empirical E         -- pass-rate claim:   rate > p,   with error δ
       | preference R        -- blind A/B claim:   new preferred > 1/2, with error δ
       | judged J            -- rubric:           ≥ quorum panel scores ≥ threshold
       | asserted k W        -- sign-off by some principal in W who is not an author

Intent ::= ⟨id, prose, necessary : List Spec, supporting : List Spec⟩

⟦formal P⟧_{L,c}   = P
⟦empirical E⟧_{L,c} = (empiricalOK L E c = true)     -- a decidable Prop about L
⟦preference R⟧_{L,c}= (preferenceOK L R c = true)
⟦judged J⟧_{L,c}   = (judgedOK L J c = true)
⟦asserted k W⟧_{L,c}= (assertedOK L k W c = true)

Certified L c I  :=  (∀ s ∈ I.necessary, ⟦s⟧) ∧ (∀ s ∈ I.supporting, ⟦s⟧)
Refuted   L c I  :=  ∃ s ∈ I.necessary, ¬⟦s⟧
```

**Design rules.**
1. **Evidence is a proposition about the ledger, never prover-supplied data.** The prover can only *point at* the ledger and *prove* a check succeeds. It cannot choose which trials count, add ballots, or drop failures. The ledger module is generated by the verifier service from signed records (`gen_ledger.py`) and pinned in `book.lock`.
2. **Polarity.** `necessary` specs are *falsifiers*: failing one **refutes** the intent. `supporting` specs are *evidence of sufficiency*: failing one leaves the intent **undetermined**, not refuted. There are three outcomes: Certified, Refuted, Undetermined.
3. **Consistency theorem (kernel-checked):** `Certified L c I → ¬ Refuted L c I`.
4. **Strength = weakest link** across modalities, ranked formal > empirical > preference > judged > asserted.
5. **Risk = union bound.** Σ δᵢ over the statistical specs is an exact rational bound on the chance that *some* statistical claim passed although false.
6. **Integrity conditions are part of the types.** Each `…OK` check enforces:
   - **pre-registration:** rubric and seed sequence numbers precede the artifact, and ballots and scores follow it;
   - **independence:** judge ∉ authors;
   - **blinding**;
   - **panel membership**;
   - **distinct judges;**
   - **completeness:** trial ids are exactly 0…n−1, so nothing was dropped;
   - **minimum sample size;**
   - **exact statistics.**

#### B.3 Exact statistics inside the kernel (`RT/Stats.lean`)

`tailOK n k a b dNum dDen` decides P[X ≥ k] ≤ dNum/dDen for X ~ Binomial(n, a/b) **exactly in ℕ**, by cross-multiplication with no floating point. It is a one-sided exact binomial test: passing certifies "rate > a/b" at confidence 1 − δ. Values match scipy:

| Claim | Tail probability | δ | Kernel result | Time |
|---|---|---|---|---|
| 10/12 judges prefer new (sign test vs ½) | 0.019287 | 1/20 | certified | < 1 s |
| 199/200 no-wrap trials, rate > 95% | 0.000404 | 1/1000 | certified | < 1 s |
| 970/1000 trials, rate > 95% | 0.001277 | 1/100 | certified (no axioms) | 0.76 s for both rows |
| 960/1000 trials, rate > 95% | 0.0806 | 1/100 | **correctly refused** | (same run) |

#### B.4 Worked example: an aesthetic intent (`RT/AestheticBook.lean`, `RT/Attacks/K0_HonestAesthetic.lean`)

**INT-log-aesthetic** — "`peri log` output is calm, readable and pleasant to scan."

| Polarity | Spec | Pre-registered parameters (human-signed Book) | Ledger outcome for change c5 |
|---|---|---|---|
| necessary | **formal** `LogFits`: no rendered line exceeds terminal width (w ≥ 1) | — | **proved** (∀ inputs) |
| supporting | **empirical** no-wrap on sampled real logs | rate > 95%, δ = 1/1000, n ≥ 200, seed before artifact | 199/200 → pass |
| supporting | **preference** blind A/B vs old layout, 12-judge panel | δ = 1/20, rubric before artifact | 10/12 → pass |
| supporting | **judged** design rubric | ≥ 2 of 3 designers score ≥ 4 | 5, 4, 3 → pass |

Results:
- The kernel accepts `cert_c5 : obligation_c5`. Computed strength is **"judged"**; risk is **1020/20000 = 5.1%**.
- Later change c6, an agent's "cleanup", drops truncation. It is **kernel-refuted**: the necessary proxy fails on `"abc"` at width 2. No panel is needed to reject a change that breaks a falsifier.

**What this certifies, honestly:** that the formal proxy holds; that measured outcomes clear pre-registered thresholds with stated error; and that the process was clean. It does **not** certify that the output *is* beautiful. That claim rests on (i) the humans who chose proxies and thresholds and (ii) the panel's judgment, and the Book records both as attributed, signed facts.

#### B.5 Cheating on qualitative evidence is provably impossible inside the calculus (`RT/Scenarios.lean`)

Each scenario edits the ledger or Book the way a cheater would. The kernel proves the intent **cannot** then be certified:

| # | Misconduct pattern | Violated condition | Kernel result |
|---|---|---|---|
| S1 | sock puppet: the authoring agent casts a ballot | judge ∉ authors | uncertifiable |
| S2 | cherry-picking: failing trial #137 dropped | completeness (ids = 0…n−1) | uncertifiable |
| S3 | post-hoc rubric (written after the artifact) | pre-registration | uncertifiable |
| S4 | one unblinded ballot | blinding | uncertifiable |
| S5 | a judge votes twice | distinct judges | uncertifiable |
| S6 | overclaimed confidence (10/12 at δ = 1/1000) | exact tail bound | uncertifiable |
| S7 | underpowered: only 100 trials run | n ≥ n_min | uncertifiable |
| S8 | rubber-stamp scores recorded before the artifact existed | temporal order | uncertifiable |
| S9 | sampling seed chosen after the artifact | pre-registration | uncertifiable |

Forging the ledger or change record *in the proof* (K1, K2) is rejected by statement pinning (§A.4). So an agent's only path to a qualitative intent is to **produce an artifact that real, independent, blinded humans prefer under pre-registered rules.**

#### B.6 What remains outside the calculus

- **Sampling assumptions.** "Trials are i.i.d. draws from real logs" is a property of the verifier service's sampler, not provable in the kernel. The calculus makes it *auditable*: seed and sequence numbers are in the ledger.
- **Proxy validity (Goodhart).** `LogFits` is necessary, not sufficient. That is why it is a falsifier, and why sufficiency comes from human preference evidence, not more proxies.
- **Panel authority.** Who may judge is a Book decision. The natural next step is an Abadi-style authorization logic ("A says s", "A speaks for B"): panels, delegations and `allowed_signers` become checkable statements, with the Book's human signers as the root of authority. *(Design only; not prototyped.)*
- **Relation to Round 4.** This calculus supersedes Round 4's `Status.tested ref` / `asserted actor` *data* for qualitative intents. `formal` keeps Round 4's proof pairing unchanged.

---

### PART C — Lanes → Review Requests → dev → main (without git worktrees)

#### C.1 Mapping the workflow onto Periapsis

| Workflow concept | Periapsis primitive | Why it removes the worktree problem |
|---|---|---|
| Agent **lane** | a *workspace*: its own working-copy change and op-head over the **one shared op-log and object store** (jj-style) | no per-lane clone or `.git/worktrees` bookkeeping; lanes are cheap views; snapshots are automatic |
| **Review Request (RR)** | an **immutable signed event record**: `{rr_id, lane, change_ids (stable across rewrites), base_op, certificate_hash, claimed intents, verifier-computed tier, touched modules (derived), status events}` | RRs follow change IDs through rebases; there is no "force-push breaks review" |
| **dev** | a **derived speculative-integration view** (base + admitted RRs), recomputed by the queue; nobody pushes to it | no shared mutable branch for agents to race on |
| **main** | the **public phase**; promotion is a signed op moving a confirmed batch of change IDs to `public` | phase is a function of evidence (Round 3 principle) |

#### C.2 Prior art the pipeline should copy

- **Zuul (OpenStack).** Its dependent pipeline tests changes in parallel, speculatively, "exactly as if they had been tested one at a time". Its window grows by one per merge and halves per failure, like TCP. Shared queues group coupled projects ([Zuul gating docs](https://zuul-ci.org/docs/zuul/latest/gating.html)).
- **Rust's bors rollups.** Low-risk PRs are batched into one CI cycle. Reviewers label each PR `rollup=always|maybe|iffy|never`, and `never` covers e.g. performance-sensitive changes or those hard to bisect ([Rust Forge review policy](https://forge.rust-lang.org/compiler/reviews.html), [rollup procedure](https://forge.rust-lang.org/release/rollups.html)). This is "filterable batches", done by humans.
- **Uber SubmitQueue.**
  - It speculates over a tree of future HEADs.
  - Its conflict analyzer treats two changes as conflicting if they affect a common set of build targets ([morning paper summary](https://blog.acolyer.org/2019/04/18/keeping-master-green-at-scale/)).
  - 2024 enhancements cut CI resource use by ~53% and P95 waiting time by 37% ([CI at Scale, arXiv 2501.03440](https://arxiv.org/html/2501.03440)).

#### C.3 Simulation (`sim.py`, `sim_extra.py`)

**Model.**
- **Change shapes** are drawn from 4,997 real changes in the helix editor's history: median 1 file, mean 2.43. The hot files are `languages.toml` (20% of changes), `Cargo.lock` (13%) and a generated docs page (9%).
- **RRs:** 300 concurrent RRs, 8 CI workers, 8 seeds.
- **Evidence-tier mix:** proved 15% (P(bad) 0.01), tested 55% (0.05), asserted 30% (0.20).
- **Conflict and interaction rates:**
  - textual conflict per overlapping landed change: q = 0.3 (Round 2: 0.21–0.37);
  - resolution-store reuse: 70% (Round 2);
  - semantic interaction failure: 3% for a shared file, 0.2% for a shared top-level directory.
- **Time:** one round = one CI cycle.
- **Main break:** two RRs landed in the same round by *different* parallel jobs that interact (never co-tested).

**Results (flaky = 0, generated files treated as derived data):**

| Strategy | Rounds | CI runs / RR | Mean latency | p95 | **Main breaks / run** |
|---|---|---|---|---|---|
| SERIAL (one at a time) | 637 | 1.45 | 293.6 | 599 | 0 |
| FIFO batches of 4 (merge-queue style) | 494 | 1.63 | 182.2 | 454 | 0 |
| FIFO-SPEC (crude speculation, see caveats) | 416 | 10.35 | 133.6 | 378 | 1.00 |
| DISJOINT-B8 (file-disjoint packing) | 271 | 1.39 | 51.3 | 228 | 0.12 |
| RISK (file-disjoint, batch size from tier) | 243 | 1.25 | 41.1 | 190 | 0.62 |
| DISJOINT-DIR (module/dir-disjoint, batch 8) | 316 | 1.45 | 97.5 | 278 | 0 |
| RISK-DIR (dir-disjoint, size from tier) | 287 | 1.40 | 85.3 | 247 | 0 |
| **AIMD-DIR (dir-disjoint, Zuul-style window, no tiers)** | **289** | **1.40** | **88.9** | **251** | **0** |

**Variants:**

| Variant | Result |
|---|---|
| Flaky 2% (single failures confirmed once) | ranking unchanged. AIMD-DIR 289 rounds / 1.92 CI per RR / latency 90.4 / 0 breaks; RISK-DIR 294 / 1.91 / 85.8 / 0 |
| Flaky 5% | AIMD-DIR 90.2, RISK-DIR 87.2, SERIAL 297.2 latency; all 0 breaks |
| Generated files kept as text (`raw`) vs treated as derived data | treating them as derived data removes 17.5–20% of textual conflicts and 8–18% of mean latency (e.g. SERIAL 332.6 → 293.6; RISK-DIR 94.0 → 85.3) |
| **Structured (key-level) merge of `languages.toml`** | textual conflicts −54% to −57% (e.g. RISK-DIR 309 → 143); latency −11% (RISK-DIR 85.3 → 75.7), −13% (SERIAL), −21% (RISK) |
| **Scale: 1000 RRs, 16 workers** | SERIAL 3464 rounds / latency 1441. **AIMD-DIR 1707 / 455 / 0 breaks** (2.0× / 3.2×). RISK-DIR 1810 / 468 / 0. RISK 1574 / 221 but **4.12 breaks**; DISJOINT-B8 284, 2.12 breaks |
| **Tiers claimed by agents** (50% of non-proved RRs over-claim) | RISK CI per RR 1.25 → 1.30; RISK-DIR latency 85 → 81. Batching is robust to tier lies |

#### C.4 Findings

1. **Disjointness at module/directory level is what makes parallel promotion safe.** File-level disjointness lets cross-file semantic interactions land untested together (0.12–0.62 breaks per 300 RRs; 2–4 per 1000). Directory/module-level packing had **0 breaks in every configuration**. SubmitQueue reaches the same conclusion with build targets as the unit.
2. **Adaptive window ≈ tier-based sizing.** AIMD-DIR matches RISK-DIR at 300 RRs and is slightly better at 1000. Combined with finding 5, **tiers are removed from batch sizing**: a simpler design with nothing for agents to game.
3. **Batching beats serial by 2–3× in throughput and latency without sacrificing a green main**, provided batches are dependency-disjoint and failures are bisected.
4. **Correction to an earlier note.** The ~+0.5 CI per RR seen once flakiness was switched on is the cost of the *flake-confirmation retest* (each genuine single-RR failure is retested once, about +0.47), not of the flake rate itself. Raising flakiness from 2% to 5% barely changes anything.
5. **Tier over-claiming barely affects throughput.** The real danger of a claimed tier is letting it *skip a check*. Rule: tiers are computed by the gate, never read from the RR.
6. **Two cheap wins:** treat generated files (lockfiles, generated docs) as derived data, and structured-merge hot config files.

#### C.5 Recommended pipeline

**Gate 1 — lane → dev (per RR, before admission to the dev view):**
- the full proof-pairing gate (`peri_gate`: Book signature + hashes + hardened verifier) for every certificate;
- array-test confirmation for tested claims;
- ledger-based checks for qualitative intents.

The RR's **tier and rollup class are computed here**, never taken from the agent:
- `always`: proved/tested, no hot files;
- `maybe`: default;
- `iffy`: touches hot or generated files or wide fan-out;
- `never`: **any Book edit** (also needs a human signature), migrations, performance-sensitive.

**Gate 2 — dev → main (batched promotion):**
1. **Select** with a filter query over RRs (revset-style), e.g.
   - `rr(dev) & ready() & intent("INT-log-*") & ~rollup(never)` — a coherent, reviewable batch for one intent;
   - `rr(dev) & module(peri-merge) & tier(>=tested)`;
   - `rr(dev) & lane(agent-7) & since(op:…)`.
2. **Pack** the selection into module-disjoint groups (crate/module dependency graph; generated files excluded as derived data).
3. **Size** groups with an AIMD window (start 8; +1 per green batch; halve on red; cap 32).
4. **Test** each group against the current main view; on failure, **bisect**; confirm single failures once (flake defence); feed resolution-store hits back.
5. **Promote** a green group with one signed op that moves its change IDs to `public`.
6. **Human review is per batch, grouped by intent.** The reviewer sees each intent's evidence profile, strength and risk, plus the mutation-score diff if the Book changed.

**Optional fast lane:** for latency-critical repos, file-level packing (RISK) cut latency a further ~2× in simulation but breaks main occasionally. Use it only together with a **post-merge sweep** on main and automatic revert.

#### C.6 Caveats of the simulation

- A single real repository supplies the change shapes. Tier mix, failure and interaction probabilities are assumptions, not measurements.
- Time is in CI rounds; build-time variance is not modelled.
- **FIFO-SPEC is not a faithful model of Zuul or SubmitQueue.** It tests parallel batches against the same snapshot instead of stacking speculative states, so its break count and CI cost overstate the real systems'.
- 8 seeds per configuration; differences under ~5% between AIMD-DIR and RISK-DIR are within noise.

---

### PART D — Changes to the Periapsis plan (no timelines)

**Phase 0 (spec):**
- verifier spec with the 7 layers;
- `book.lock` format + `allowed_signers` (namespaces `book`, `ledger`);
- the intent-calculus library (`Spec`, `Intent`, `Certified`/`Refuted`, exact statistics);
- planted-regression probe suite (`ablation.sh` pattern) as a required CI job.

**Phase 2:** the negative-witness/mutant registry for every formal Book intent; mutation-score diff on Book edits.

**Phase 3:**
- RR records as immutable signed events;
- dev as a derived speculative view;
- Gate 1 (per RR) and Gate 2 (AIMD + module-disjoint packing + bisection + filter queries + rollup classes computed by the gate);
- the verifier service that writes signed ledger records (trials with seeds, blinded ballots, rubric scores) and generates the ledger module;
- qualitative intents in the Book use the five modalities.

**Phase 4:** production verifier = sandboxed build + `lean4export` + replay with Lean kernel **and** nanoda + comparator as the independent statement/axiom check.

**Phase 5:** Abadi-style authority logic for panels and delegations; replay of agent-defined inductive types.

**Removed / simplified:** tier-based batch sizing (replaced by AIMD); trusting agent-declared tiers anywhere.

---

### §E Artifacts and reproduction

| Path | Contents |
|---|---|
| `prototypes/peri-verify/Verify.lean` | hardened verifier (7 layers incl. raw extension-entry policy; `PERI_DISABLE` for ablation) |
| `prototypes/peri-verify/peri_gate.py`, `book.lock`, `book.lock.sig`, `allowed_signers.json`, `keys/*.pub` | Book-integrity gate (Ed25519). Private demo keys are not shipped |
| `prototypes/peri-verify/RT/` | trusted Book (`Merge`, `Book`, `Obligations`, `Stats`, `Intents`, `Ledger`, `Render`, `AestheticBook`, `Adequacy`) + `Scenarios.lean` + `StatsN1000_example.lean` |
| `prototypes/peri-verify/RT/Attacks/` | A01–A10, K0–K2 |
| `prototypes/peri-verify/attacks_book/` | B1 patch + certificate |
| `naive_gate.sh`, `run_redteam.sh`, `ablation.sh`, `b1_demo.sh` | reproduce the tables in §A.3, §A.5 and §A.6 |
| `prototypes/peri-verify/results/` | outputs of every run above + adequacy/aesthetic evals |
| `experiments/round5_rr/sim.py`, `sim_extra.py`, `helix_changes.json`, `sim_out_*.txt` | pipeline simulation and outputs |

Run `lake build && bash run_redteam.sh` (about 17 s on this machine), `./ablation.sh`, `./b1_demo.sh`, `python3 sim.py 0`, and `python3 sim_extra.py` (about 3 min).

### §F Sources

- Lean reference, "Validating a Lean Proof": https://lean-lang.org/doc/reference/latest/ValidatingProofs/
- leanprover/comparator: https://github.com/leanprover/comparator
- L. de Moura, Postmortem for Kernel Soundness Bug #14576: https://leodemoura.github.io/blog/2026-8-1-postmortem-for-kernel-soundness-bug-14576/
- L. de Moura, Postmortem for the Kernel Soundness Bug Hunt: https://leodemoura.github.io/blog/2026-8-24-postmortem-for-the-kernel-soundness-bug-hunt/
- CVE-2026-72844 advisory (oss-security): https://www.openwall.com/lists/oss-security/2026/10/26/2
- Lean 4.32.2 release notes: https://lean-lang.org/doc/reference/latest/releases/v4.32.2/
- lean4 issue #7463 (`@[csimp]` smuggles axioms): https://github.com/leanprover/lean4/issues/7463
- Apart Research hackathon, TCB-Expansion Attacks on Lean 4: https://apartresearch.com/sprints/projects/tcbexpansion-attacks-on-lean-4-and-the-llm-proof-reviewers-that-mostly-miss-them-o3hn
- pmatos/funk PR #58 (csimp-swap rule): https://github.com/pmatos/funk/pull/58
- coproduct-opensource/nucleus #2555 (Lean trust chain epic): https://github.com/coproduct-opensource/nucleus/issues/2555
- PrimeIntellect verifiers #2177 (axiom injection bypasses reward): https://github.com/PrimeIntellect-ai/verifiers/issues/2177
- ImpossibleBench (ICLR 2026): https://arxiv.org/abs/2510.20270 ; summary: https://www.lesswrong.com/posts/qJYMbrabcQqCZ7iqm/impossiblebench-measuring-reward-hacking-in-llm-coding-1
- Lean4Lean (Carneiro): https://arxiv.org/html/2403.14064v3
- Zuul project gating: https://zuul-ci.org/docs/zuul/latest/gating.html
- Rust Forge review policy / rollups: https://forge.rust-lang.org/compiler/reviews.html , https://forge.rust-lang.org/release/rollups.html
- Ananthanarayanan et al., Keeping Master Green at Scale (EuroSys'19), summary: https://blog.acolyer.org/2019/04/18/keeping-master-green-at-scale/
- Juloori et al., CI at Scale: Lean, Green, and Fast (ICSE 2025): https://arxiv.org/html/2501.03440
- Background (not re-fetched this round): Abadi, Burrows, Lampson, Plotkin, "A Calculus for Access Control in Distributed Systems" (1993).

