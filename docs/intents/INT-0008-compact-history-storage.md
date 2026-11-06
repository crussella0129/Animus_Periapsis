# INT-0008 — Compact history: per-path predecessor compression

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0008
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Successive versions of a file are stored compressed against their predecessor:
zstd with the previous version referenced as a prefix (window set first, with
long-distance matching for large files) and a keyframe at least every 50
versions. This applies to every file alike, whatever it holds
([INT-0034](INT-0034-any-content.md)). Content-defined chunking is not part
of core; it may return as an optional extension for cross-file deduplication,
partial fetch of huge files, or files larger than zstd's 2 GiB window. Core
stores files of any size without that extension: a version whose predecessor
is larger than the window is stored as a keyframe, compressed and
decompressed as a stream, so no file larger than the window is ever held
whole in memory.

Boundaries: this intent owns blob storage representation. Logical object
identity is [INT-0006](INT-0006-format-v1.md); durability is
[INT-0007](INT-0007-crash-safe-storage.md).

## Acceptance criteria
- **AC-1** `necessary` `empirical` — On the helix editor's history of `.rs` blobs (9,170 versions, 370 MB raw), total stored size at the default compression level is at most 4.4 MB and below a Git delta pack of the same blobs.
  - Note: R2-E27b measured 4.35 MB at zstd level 3, 3.48 MB at level 19, and 8.49 MB for the Git pack.
- **AC-2** `necessary` `formal` — Storage is lossless: reading any stored version returns exactly the content of the blob whose object id ([INT-0006](INT-0006-format-v1.md): the hash of the blob's whole envelope) is its `HashRef`.
  - Obligation: `unstated`.
  - Proxy: round-trip property tests and `fsck`.
- **AC-3** `necessary` `empirical` — Reading any version applies at most 50 deltas.
- **AC-4** `supporting` `empirical` — Large binaries: a 64 MiB semi-structured file with 160 changed 4 KiB pages and a 1 KiB insertion stores its new version in at most 1 MB.
  - Note: R3-C measured 0.73 MB, against 2.31 MB for 8 KiB CDC.
- **AC-5** `supporting` `empirical` — Random-access latency, repacking cost and rename behaviour are measured and published before the storage representation is frozen at the Phase 1 exit gate.
- **AC-6** `necessary` `empirical` — With no chunking extension installed, a file larger than 2 GiB and a later version of it are stored and read back byte for byte, and neither operation holds a whole version in memory.
  - Sample: files of 3 GiB and 5 GiB, each with one later version.
  - Threshold: 0 differing bytes, and the peak memory of each operation stays below the size of the version it stores or reads.

## Rationale
- On real source history, predecessor-dictionary zstd beat every alternative:
  4.35 MB (level 3) and 3.48 MB (level 19) against 8.49 MB for a Git pack,
  25.4 MB for the best CDC configuration and 86.7 MB for whole-blob zstd
  (R2-E27b). Chunk-list metadata dominates at small chunk sizes.
- It also wins on large binaries (0.73 MB against 2.31 MB for CDC) (R3-C).
- Pitfall to avoid: passing the previous version via `with_dictionary` gave no
  gain (35.2 MB); it must be referenced as a prefix with the window set first
  (R3-C).
- Sources: [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md)
  R2-E27b, R3-C, R3 §4.1.

## Alternatives
- **FastCDC + BLAKE3 chunking in core (Xet-style).** Removed from core: worse
  on both text and binaries; edits pay a whole chunk (R1-E10, R2-E27b, R3-C).
- **Git-style delta packs.** About twice the size on the measured history.

## Consequences
- The reference zstd implementation is C. A pure-Rust encoder may not support
  prefix references with long-distance matching, so this intent may need a
  recorded exception to INT-0003 AC-6. Evaluate before Phase 1 storage work.
- Decompression needs the previous version in memory up to the window (2 GiB
  maximum), so a version whose predecessor is larger is a keyframe (R3 §6).
  Until the chunking extension exists, every version of such a file costs a
  whole compressed copy.

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 R2 §3 and R3-C).
- 2026-10-31: revised while `proposed`, after the Sprint 0 plan critique. AC-2 names the object id of INT-0006 (hash of the whole blob envelope) rather than a bare-content digest. AC-5's deadline is the freezing of the storage representation at the Phase 1 exit gate, not Sprint 0's freezing of logical encodings.
- 2026-11-01: revised while `proposed`, on owner direction: core must store content of any size. The Intent says a version whose predecessor is larger than zstd's window is stored as a keyframe, streamed in and out, without the chunking extension; AC-6 checks this for files over 2 GiB; Consequences records that every version of such a file costs a whole compressed copy.
