# Periapsis format v1

This is the normative description of how Periapsis encodes values, hashes
and objects. It realises [INT-0006](../intents/INT-0006-format-v1.md) and is
written before the code that implements it: the `hash` and `codec` crates
follow it, `crates/codec/tests/spec.rs` checks that their registries match the
tables below and that every golden file matches its worked example, and
`cargo policy` (rule `spec`) checks that this document keeps its sections.

**Status:** draft. Sprint 0 specifies the primitives, `hashref`, the envelope
header and the blob body. Those encodings freeze when the owner signs off this
document (INT-0006 AC-6). Object kinds marked *reserved* have no body format
yet. Format v1 as a whole freezes at the Phase 1 exit gate, once every kind and
the storage representation are specified. See [Change control](#change-control).

## Conventions

- Bytes are written in hex, e.g. `80 01`.
- `‖` is concatenation.
- The single-byte tags (bool, option tag, hash algorithm id, format version,
  object kind) are raw bytes. Every other integer is a `uvarint`; v1 has no
  fixed-width integers.

## Primitives

| Type | Encoding | Canonical rules |
|---|---|---|
| `uvarint` | unsigned LEB128, 1–10 bytes, low group first | Minimal: the last byte is non-zero unless the value is 0. The tenth byte, if present, is `00` or `01`. |
| `u32` | `uvarint` | Value ≤ 2³² − 1. |
| `bool` | one byte | `00` is false, `01` is true; any other byte is rejected. |
| `bytes` | `uvarint` length ‖ raw bytes | — |
| `string` | `uvarint` length ‖ UTF-8 bytes | Must be valid UTF-8. No Unicode normalisation is applied. |
| `option<T>` | `00`, or `01` ‖ `T` | Any other tag is rejected. |
| `seq<T>` | `uvarint` count ‖ `T`… | Order is significant. |
| `set<T>` | `uvarint` count ‖ `T`… | Elements strictly increasing in canonical order. |
| `map<K, V>` | `uvarint` count ‖ (`K` ‖ `V`)… | Keys strictly increasing in canonical order. |
| `pair<A, B>` | `A` ‖ `B` | — |
| `hashref` | algorithm id ‖ digest | See [Hash algorithms](#hash-algorithms). |

Examples: `0` → `00`; `127` → `7f`; `128` → `80 01`; `300` → `ac 02`;
2⁶⁴ − 1 → `ff ff ff ff ff ff ff ff ff 01`; the string `"é"` → `02 c3 a9`.

### Canonical order

Sets and maps are ordered by their elements' or keys' canonical order:

- `uvarint`, `u32`: numeric.
- `bool`: false before true.
- `bytes`, `string`: lexicographic by raw bytes (for strings, the UTF-8 bytes,
  which is code-point order).
- `hashref`: algorithm id first, then digest bytes lexicographically.
- `option<T>`: absent before present, then by `T`.
- `seq<T>`: lexicographic by elements.
- `pair<A, B>`: by `A`, then by `B`.

## Hash algorithms

| id | name | digest bytes | status |
|---|---|---|---|
| `0x01` | `blake3` | 32 | default |
| `0x02` | `sha256` | 32 | supported |

Id `0x00` is invalid. Ids `0x03`–`0xff` are unassigned. Decoders reject both.
`blake3` is BLAKE3 with a 256-bit output; `sha256` is SHA-256 (FIPS 180-4).

A `hashref` always carries its id, so equal digest bytes under different
algorithms are different values, ordered by id first. A repository may hold
objects hashed under more than one algorithm during a migration.

Text form: `<name>:<lowercase hex digest>`, for example
`blake3:af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262`
(the BLAKE3 hash of empty input). Uppercase hex is rejected, so every value has
exactly one text form.

## Objects

Every object is an envelope:

```text
format_version (01) ‖ kind ‖ body
```

A format version other than `01` is rejected.

### Object kinds

| id | kind | status |
|---|---|---|
| `0x01` | `blob` | defined |
| `0x02` | `changeset` | reserved |
| `0x03` | `operation` | reserved |
| `0x04` | `view` | reserved |
| `0x05` | `event` | reserved |

Id `0x00` is invalid and `0x06`–`0xff` are unassigned; decoders reject them as
unknown kinds. Reserved kinds are rejected with a distinct reserved-kind error
until their bodies are specified here.

- **blob** — the body is the file's raw content, with no length prefix: the
  envelope ends where the content ends.

### Object identity

An object's id is the `hashref` of its whole envelope under the repository's
hash algorithm: `id = H(01 ‖ kind ‖ body)`. Because the version and kind bytes
are hashed, the same body under two kinds has two ids, and a blob's id is not
the hash of its bare content.

## Worked examples

The exact encoding of every value in the golden corpus
(`crates/codec/tests/golden/v1/<name>.bin`), derived by hand from the rules
above. `crates/codec/tests/spec.rs` checks every golden file against this
table, so the encoder is tested against the spec rather than against itself.

| name | value | hex |
|---|---|---|
| `uvarint-0` | `uvarint` 0 | `00` |
| `uvarint-127` | `uvarint` 127 | `7f` |
| `uvarint-128` | `uvarint` 128 | `80 01` |
| `uvarint-300` | `uvarint` 300 | `ac 02` |
| `uvarint-max` | `uvarint` 2⁶⁴ − 1 | `ff ff ff ff ff ff ff ff ff 01` |
| `u32-max` | `u32` 2³² − 1 | `ff ff ff ff 0f` |
| `bool-false` | `bool` false | `00` |
| `bool-true` | `bool` true | `01` |
| `bytes-empty` | `bytes` (empty) | `00` |
| `bytes-short` | `bytes` 00 7f 80 ff | `04 00 7f 80 ff` |
| `string-empty` | `string` "" | `00` |
| `string-ascii` | `string` "periapsis" | `09 70 65 72 69 61 70 73 69 73` |
| `string-utf8` | `string` "π → é 🜨" | `0e cf 80 20 e2 86 92 20 c3 a9 20 f0 9f 9c a8` |
| `hashref-blake3` | `hashref` BLAKE3 of empty input | `01 af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262` |
| `hashref-sha256` | `hashref` SHA-256 of "abc" | `02 ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad` |
| `option-none` | `option<uvarint>` absent | `00` |
| `option-some` | `option<string>` "present" | `01 07 70 72 65 73 65 6e 74` |
| `seq-u64` | `seq<uvarint>` 3, 1, 4, 1, 5, 9, 2, 6 | `08 03 01 04 01 05 09 02 06` |
| `set-string` | `set<string>` "a", "ab", "b" | `03 01 61 02 61 62 01 62` |
| `map-string-u64` | `map<string, uvarint>` "dev" → 12, "main" → 7 | `02 03 64 65 76 0c 04 6d 61 69 6e 07` |
| `map-nested` | `map<string, map<uvarint, option<bytes>>>` "intent" → (1 → absent, 2 → 78) | `01 06 69 6e 74 65 6e 74 02 01 00 02 01 01 78` |
| `pair` | `pair<string, bool>` ("key", true) | `03 6b 65 79 01` |
| `blob-envelope` | blob envelope of "Periapsis" and a newline | `01 01 50 65 72 69 61 70 73 69 73 0a` |

## Signatures (reserved)

Signing arrives with [INT-0016](../intents/INT-0016-actors-and-agent-lifecycle.md).
Every signed payload will begin with
`format_version ‖ hash algorithm id ‖ signature algorithm id`, so a signature
never leaves its hash algorithm ambiguous. Signature algorithm id `0x01` is
reserved for Ed25519.

## Decoder requirements

- **Total:** decoding any byte string returns a value or an error; it never
  panics or aborts.
- **Exact:** a value must use the whole input; trailing bytes are an error.
- **Bounded:** a length or count larger than the remaining input is rejected
  before anything of that size is allocated. Every encoded value takes at least
  one byte, so a count can never exceed the remaining input. A decoded element
  can still take many times its encoded size, so no decoder reserves memory
  from a declared count: a collection grows only as its elements decode.
- **Located:** every error reports the absolute byte offset where the faulty
  value starts. A string that is not UTF-8 is reported at its first content
  byte, after its length prefix. An error in the envelope header names its
  field instead, because the field's position is fixed: the format version is
  at offset 0 and the kind at offset 1.
- **Canonical:** only the encoding defined here is accepted, so
  `encode(decode(b)) = b` for every accepted `b`.

## Change control

- `crates/codec/tests/golden/v1/*.bin` pins the encoding of a fixed corpus;
  the golden test fails if any byte changes, and the spec test fails if a file
  disagrees with its worked example.
- Before the owner's sign-off, a corrected encoding changes this spec's worked
  example, the golden file and the encoder together, in one commit.
- After sign-off, files under `v1/` never change. Changing a frozen encoding
  requires a new format version, a new golden directory and a revision of
  INT-0006.
- Specifying a reserved kind's body, adding a kind or adding a hash algorithm
  adds rows and worked examples here, and lands before the code that
  implements it. It never changes an existing encoding.
- Golden files are marked `binary` in `.gitattributes`, so every platform
  checks out the same bytes.
