# Animus_Periapsis

Rust based, agent-human hybrid Distributed Version Control System (DCVS), based on the concepts of the Animus_Sprint_Loops recursive development system, CubiKan, array-test (both my repositories), with research on the best parts of Git, Jujitsu, Mercurial, and other DCVSs

Visual Branding intent for the readme (and github pages if i go that route) - like general Radahn's sword, very crude yet mystical looking bent field lines and objects trajectories, emblazoned in stone, associated with 'gravity magick', bc the name of the product is Apsis, which is an orbit like term. Perhaps the Readme hero is like a figure doing a GFOB around a celestial body or something, emblazoned in the stone. 

## The Book

What Periapsis is for, and why, lives in the Project Book: start at
[`docs/SUMMARY.md`](docs/SUMMARY.md). Each intent chapter under
[`docs/intents/`](docs/intents/) states a desired outcome with typed acceptance
criteria ([INT-0002](docs/intents/INT-0002-typed-acceptance-criteria.md)), and
[INT-0001](docs/intents/INT-0001-periapsis-charter.md) maps the whole system.
The project is developed with
[Sprint Loops](https://github.com/crussella0129/Animus_Sprint_Loops); every
sprint's research, plans and test evidence are under
[`docs/sprints/`](docs/sprints/).

## Status

Phase 1 is under way. The format is specified in
[`docs/spec/format-v1.md`](docs/spec/format-v1.md), the crash-tested store in
[`docs/spec/storage-durability.md`](docs/spec/storage-durability.md), and the
working copy in [`docs/spec/working-copy.md`](docs/spec/working-copy.md).
Periapsis can record a working copy and return it to any earlier state:

| Command | What it does |
|---|---|
| `praps init [DIR]` | Makes `DIR` (default: `.`) a repository, or says it already is one |
| `praps status [DIR]` | Snapshots the working copy, following `.gitignore` rules, and lists what changed against its parent (`A`, `M`, `D`), plus entries it cannot track |
| `praps new [DIR]` | Snapshots, then starts a new working-copy change on top of the current one |
| `praps log [DIR]` | Snapshots, then lists every operation, newest first |
| `praps undo [DIR]` | Snapshots, then undoes the last operation, writing the files back; a second `undo` undoes the first |
| `praps restore OP [DIR]` | Snapshots, then returns the working copy to the state operation `OP` (an id or unique prefix) left |
| `praps fsck [DIR]` | Checks the store of a repository, or a bare store: checksums and references |

Nothing is lost: `undo` and `restore` are operations themselves, and the
edits they replace stay reachable. A write that a crash interrupts is
finished by the next command, and names the filesystem cannot hold (case
collisions, Windows device names) are refused before anything is written.
Checking out another change, merging concurrent operations and several
workspaces come later.

| Crate | What it does |
|---|---|
| `hash` | Algorithm-tagged hashes: BLAKE3 by default, SHA-256 as a second algorithm |
| `codec` | Canonical binary encoding and object envelopes |
| `sim` | Filesystem and randomness, real and deterministically simulated with crashes |
| `store` | The crash-safe object store and op log, with `fsck` |
| `workspace` | The working copy: init, snapshot (with `.gitignore` rules), new, status, log, undo and restore |
| `praps` | The Periapsis command (`praps init`, `new`, `status`, `log`, `undo`, `restore`, `fsck`) |
| `book` | Checks the Book's typed acceptance criteria |
| `xtask` | Project tooling behind `cargo ci`, `cargo policy` and `cargo reach` |

## Build and test

Requires rustup; the toolchain is pinned in `rust-toolchain.toml`.

```bash
cargo ci
```

`cargo ci` runs formatting, Clippy, the test suite, the reach check, the
workspace policy and the Book check, the same way on Windows, Linux and GitHub
Actions.
`cargo policy` and `cargo book` run those last two checks on their own.
`cargo reach` rewrites the reach table, [`docs/reach.md`](docs/reach.md), from the points the latest test run recorded; `cargo ci` fails when it is stale.
`cargo ci`, `cargo policy`, `cargo book` and `cargo reach` take no flags.
Licence and advisory checks use
[cargo-deny](https://github.com/EmbarkStudios/cargo-deny), run exactly as:

```bash
cargo deny check
```

Designs borrowed from other systems are credited in
[`docs/prior-art.md`](docs/prior-art.md).

## License

GPL-3.0; see [`LICENSE`](LICENSE).
