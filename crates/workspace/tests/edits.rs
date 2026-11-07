//! The randomized edit-and-command corpus on the simulator (INT-0014 AC-1:
//! 0 lost edits; Sprint 2 T-007).

mod common;

use std::path::Path;
use std::rc::Rc;

use std::collections::BTreeSet;

use common::{Model, ROOT, edit_sequence, put, recorded_tree};
use sim::{Mode, SimFs};
use workspace::Workspace;

const SEQUENCES: u64 = 500;
const STEPS: usize = 12;

#[test]
fn edit_corpus_loses_nothing() {
    let mut checked = 0;
    for seed in 0..SEQUENCES {
        let fs = Rc::new(SimFs::new(seed, Mode::Strict));
        let mut ws = Workspace::init(Rc::clone(&fs), Path::new(ROOT), seed)
            .unwrap()
            .0;
        let mut model = Model::default();
        // Half the sequences run on a filesystem with no executable bit, as
        // on Windows. They start from a checkout made where there is one, with
        // executable files tracked, so the "keep the recorded mode" rule is
        // exercised (test critique C-002).
        if seed % 2 == 1 {
            put(&*fs, "a", b"#!");
            put(&*fs, "d2/z", b"#!");
            put(&*fs, "c.txt", b"plain");
            fs.make_executable(&Path::new(ROOT).join("a"), true)
                .unwrap();
            fs.make_executable(&Path::new(ROOT).join("d2/z"), true)
                .unwrap();
            ws.snapshot().unwrap();
            let expected = model.snapshot(&*fs, &BTreeSet::new());
            assert_eq!(recorded_tree(&ws), expected);
            assert_eq!(expected["a"].1, codec::Mode::Executable);
            fs.set_has_executable_bit(false);
        }
        checked += edit_sequence(
            &mut ws,
            model,
            seed,
            STEPS,
            &|fs: &Rc<SimFs>, name, executable| {
                fs.make_executable(&Path::new(ROOT).join(name), executable)
                    .unwrap();
            },
        );
    }
    println!("edit corpus: {SEQUENCES} sequences, {checked} snapshots, 0 lost edits");
    assert_eq!(checked, SEQUENCES as usize * STEPS);
}
