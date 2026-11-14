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
Periapsis can record a working copy, describe its changes and return it to
any earlier state. `praps` alone lists its commands, and this table is that
list:

| Command | What it does |
|---|---|
| `praps status [DIR]` | List what the working copy changes |
| `praps describe TEXT [DIR]` | Say what the current change is for |
| `praps new [DIR]` | Start the next change |
| `praps log [DIR]` | List the changes that lead to the working copy, newest first |
| `praps undo [DIR]` | Take back the last operation |
| `praps lane DIR` | Make a working copy of your own over this repository |
| `praps lanes [DIR]` | List the working copies over this repository |
| `praps ops [DIR]` | List every operation, newest first |
| `praps restore OP [DIR]` | Return to the state an operation left |
| `praps init [DIR]` | Make a directory a repository |
| `praps verify [DIR]` | Check that everything stored is intact |
| `praps help [WORD]` | Explain the commands, or one command |
| `praps version` | Name this build |

`praps help WORD` explains one command, with an example. A command works
from any directory inside the working copy, records the working copy before
it does anything else, and follows `.gitignore` rules. A command that cannot
do what was asked says what was wrong and ends with a `hint:` line that says
what to do next.

Nothing is lost: `undo` and `restore` are operations themselves, and the
edits they replace stay reachable. A write that a crash interrupts is
finished by the next command, and names the filesystem cannot hold (case
collisions, Windows device names) are refused before anything is written.
Several agents can work in one repository at once. `praps lane DIR` gives
each a working copy of its own over the same store, and `praps lanes` lists
them. Commands that run at the same time, in one working copy or in several,
each record their operation, and every later command works from their merge,
writing nothing unless it changes something. `undo` and `restore` act on the
working copy that runs them and leave the others as they are. Checking out
another change comes later.

| Crate | What it does |
|---|---|
| `hash` | Algorithm-tagged hashes: BLAKE3 by default, SHA-256 as a second algorithm |
| `codec` | Canonical binary encoding and object envelopes |
| `sim` | Filesystem and randomness, real and deterministically simulated with crashes |
| `store` | The crash-safe object store and op log, with its integrity check |
| `workspace` | The working copy: init, snapshot (with `.gitignore` rules), describe, new, status, the lists of changes and operations, undo and restore; lanes, and the merge of operation heads |
| `praps` | The Periapsis command: its registry of commands, help, hints and the commands themselves |
| `book` | Checks the Book's typed acceptance criteria |
| `xtask` | Project tooling behind `cargo ci`, `cargo policy` and `cargo reach` |

## Build and test

Requires rustup; the toolchain is pinned in `rust-toolchain.toml`.

```bash
cargo ci
```

`cargo ci` runs formatting, Clippy, the test suite, the reach check, the
workspace policy and the Book check, the same way on Windows, Linux and GitHub
Actions. Tests build optimised. The suite builds every test executable once,
then runs the test files in parallel. It fails if any executable Cargo built
goes unrun.
GitHub Actions runs on every pull request, on every push to `main`, and on
demand. Formatting and Clippy, the suite and cargo-deny show as separate
checks, and a newer push to a pull request cancels the run it replaces.
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
