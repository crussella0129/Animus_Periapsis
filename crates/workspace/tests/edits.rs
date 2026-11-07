//! The randomized edit-and-command corpus on the simulator (INT-0014 AC-1:
//! 0 lost edits; Sprint 2 T-007).

mod common;

use std::path::Path;
use std::rc::Rc;

use common::{ROOT, edit_sequence};
use sim::{Mode, SimFs};
use workspace::Workspace;

const SEQUENCES: u64 = 500;
const STEPS: usize = 12;

#[test]
fn edit_corpus_loses_nothing() {
    let mut checked = 0;
    for seed in 0..SEQUENCES {
        let fs = Rc::new(SimFs::new(seed, Mode::Strict));
        // Half the sequences run on a filesystem with no executable bit, as
        // on Windows.
        fs.set_has_executable_bit(seed % 2 == 0);
        let mut ws = Workspace::init(Rc::clone(&fs), Path::new(ROOT), seed)
            .unwrap()
            .0;
        checked += edit_sequence(&mut ws, seed, STEPS, &|fs: &Rc<SimFs>, name, executable| {
            fs.set_executable(&Path::new(ROOT).join(name), executable)
                .unwrap();
        });
    }
    println!("edit corpus: {SEQUENCES} sequences, {checked} snapshots, 0 lost edits");
    assert_eq!(checked, SEQUENCES as usize * STEPS);
}
