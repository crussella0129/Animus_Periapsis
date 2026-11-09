//! Recording what a simulation scenario reached, for the reach table
//! (INT-0005 AC-6; `cargo reach`).
//!
//! A scenario's designated test sums [`crate::SimFs::reach`] over its cells
//! and writes the totals with [`record`]: a file `<file>.txt` under the
//! given directory (`$CARGO_TARGET_TMPDIR/reach`), through a temporary name
//! and a rename, so a reader never sees half a file. The format is
//! `scenario <name>`, `sprint <n>`, then one `<point> <count>` line per
//! point, sorted.

use std::collections::BTreeMap;
use std::io::{self, ErrorKind};
use std::path::Path;

use crate::fs::Fs;
use crate::real::RealFs;

/// Point hits summed over a scenario's cells.
pub type Totals = BTreeMap<&'static str, u64>;

/// Adds one cell's reach to `totals`.
pub fn add(totals: &mut Totals, reach: &[(&'static str, u64)]) {
    for (name, count) in reach {
        *totals.entry(name).or_default() += count;
    }
}

/// The text of a reach file.
pub fn render(scenario: &str, sprint: u32, totals: &Totals) -> String {
    let mut text = format!("scenario {scenario}\nsprint {sprint}\n");
    for (name, count) in totals {
        text.push_str(&format!("{name} {count}\n"));
    }
    text
}

/// Writes `dir/<file>.txt` for `scenario`, added in `sprint`, through a
/// temporary file, a sync and a rename.
pub fn record(
    dir: &Path,
    file: &str,
    scenario: &str,
    sprint: u32,
    totals: &Totals,
) -> io::Result<()> {
    match RealFs.create_dir(dir) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error),
    }
    let new = dir.join(format!("{file}.txt.new"));
    match RealFs.remove_file(&new) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    RealFs.create_new(&new)?;
    RealFs.write(&new, render(scenario, sprint, totals).as_bytes())?;
    RealFs.sync_file(&new)?;
    RealFs.rename(&new, &dir.join(format!("{file}.txt")))?;
    RealFs.sync_dir(dir)
}
