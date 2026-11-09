//! `cargo reach`: the reach table (INT-0005 AC-6).
//!
//! Each simulation scenario's designated test records the named points its
//! cells reached, summed, in `target/tmp/reach/<file>.txt` (through
//! `sim::reach::record`). This command reads those files and the point
//! registries and writes `docs/reach.md`: one row per scenario, by the
//! sprint that added it; one column per unit, by the sprint that built it;
//! each cell the number of point hits. The owner's triangle of regression
//! runs passing back through earlier units is this table.
//!
//! It fails, writing nothing, if a registered point has no hit, a scenario's
//! file is missing, a file names a point no registry declares (names with
//! the reserved `test.` prefix are ignored), or a scenario no longer reaches
//! a point `EXPECTED` lists for it. `cargo ci`'s `reach` step
//! generates the same table in memory and compares it with the committed
//! one, writing nothing.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::points::{self, Entry};
use crate::workspace::members;

/// The scenarios, by the file their designated test writes and the name the
/// table shows.
pub const SCENARIOS: [(&str, &str); 5] = [
    ("store", "store"),
    ("two-writers", "two writers"),
    ("snapshot", "Snapshot"),
    ("materialise", "Materialise"),
    ("four-writers", "four writers"),
];

/// The points each scenario is expected to reach, as the Sprint 4 build
/// plan lists them (T-006), by the name the table shows. A scenario that
/// stops reaching one fails the table, even if another scenario still
/// reaches it (test critique C-011). An expectation is checked when its
/// point is registered and its scenario recorded; `expectations_are_real`
/// keeps this list naming real points and scenarios.
pub const EXPECTED: [(&str, &[&str]); 4] = [
    (
        "store",
        &[
            "store.init.layout_created",
            "store.sync_layout.synced",
            "store.put_object.present_synced",
            "store.put_object.temp_written",
            "store.put_object.temp_synced",
            "store.put_object.renamed",
            "store.put_object.dir_synced",
            "store.commit_op.temp_written",
            "store.commit_op.temp_synced",
            "store.commit_op.renamed",
            "store.commit_op.dir_synced",
            "store.commit_op.references_resynced",
        ],
    ),
    ("four writers", &["store.commit_op.present_synced"]),
    (
        "Snapshot",
        &["workspace.snapshot.scanned", "workspace.snapshot.committed"],
    ),
    (
        "Materialise",
        &[
            "workspace.record.written",
            "workspace.record.dropped",
            "workspace.apply.path_written",
            "workspace.apply.path_removed",
            "workspace.resume.started",
            "workspace.resume.dropped",
        ],
    ),
];

/// The committed table.
pub const TABLE: &str = "docs/reach.md";

/// Points with this prefix belong to tests and are never registered.
const TEST_PREFIX: &str = "test.";

/// Where the designated tests write, under the workspace root: Cargo's
/// `CARGO_TARGET_TMPDIR` for integration tests, `target/tmp`.
pub fn reach_dir(root: &Path) -> PathBuf {
    root.join("target").join("tmp").join("reach")
}

/// One scenario's recorded reach.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recorded {
    pub name: String,
    pub sprint: u32,
    pub counts: BTreeMap<String, u64>,
}

/// Reads a reach file: `scenario <name>`, `sprint <n>`, then one
/// `<point> <count>` line per point.
pub fn parse(text: &str) -> Result<Recorded, String> {
    let mut lines = text.lines();
    let name = lines
        .next()
        .and_then(|line| line.strip_prefix("scenario "))
        .ok_or("no `scenario` line")?
        .to_owned();
    let sprint = lines
        .next()
        .and_then(|line| line.strip_prefix("sprint "))
        .and_then(|n| n.parse().ok())
        .ok_or("no `sprint` line")?;
    let mut counts = BTreeMap::new();
    for line in lines {
        let (point, count) = line
            .split_once(' ')
            .ok_or_else(|| format!("not `<point> <count>`: {line}"))?;
        let count = count
            .parse()
            .map_err(|_| format!("not `<point> <count>`: {line}"))?;
        counts.insert(point.to_owned(), count);
    }
    Ok(Recorded {
        name,
        sprint,
        counts,
    })
}

/// The table from the registries and the recorded scenarios (in
/// `SCENARIOS` order), or every problem found.
pub fn table(
    registries: &BTreeMap<String, Vec<Entry>>,
    scenarios: &[(&str, Recorded)],
) -> Result<String, Vec<String>> {
    let mut problems = Vec::new();
    let points: BTreeMap<&str, &Entry> = registries
        .values()
        .flatten()
        .map(|entry| (entry.name.as_str(), entry))
        .collect();
    let mut totals: BTreeMap<&str, u64> = points.keys().map(|name| (*name, 0)).collect();
    for (shown, recorded) in scenarios {
        for (point, count) in &recorded.counts {
            if point.starts_with(TEST_PREFIX) {
                continue;
            }
            match totals.get_mut(point.as_str()) {
                Some(total) => *total += count,
                None => problems.push(format!(
                    "scenario `{shown}` names point `{point}`, which no registry declares"
                )),
            }
        }
    }
    for (point, total) in &totals {
        if *total == 0 {
            problems.push(format!("point `{point}` is reached by no scenario"));
        }
    }
    for (shown, expected) in EXPECTED {
        let Some((_, recorded)) = scenarios.iter().find(|(name, _)| *name == shown) else {
            continue;
        };
        for point in expected {
            if points.contains_key(point) && recorded.counts.get(*point).copied().unwrap_or(0) == 0
            {
                problems.push(format!(
                    "scenario `{shown}` no longer reaches point `{point}`, which it is expected to"
                ));
            }
        }
    }
    if !problems.is_empty() {
        return Err(problems);
    }
    // Columns: units, by building sprint, then name.
    let mut units: Vec<(u32, &str)> = points
        .values()
        .map(|entry| (entry.sprint, entry.unit.as_str()))
        .collect();
    units.sort();
    units.dedup();
    // Rows: scenarios, by adding sprint, then `SCENARIOS` order.
    let mut rows: Vec<(u32, usize, &str, &Recorded)> = scenarios
        .iter()
        .enumerate()
        .map(|(i, (shown, recorded))| (recorded.sprint, i, *shown, recorded))
        .collect();
    rows.sort_by_key(|(sprint, i, ..)| (*sprint, *i));

    let mut out = String::new();
    out.push_str("# Reach\n\n");
    out.push_str(
        "Generated by `cargo reach`; do not edit. `cargo ci` fails when this table differs from \
         the one the latest test run records ([INT-0005](intents/INT-0005-deterministic-simulation-testing.md) \
         AC-6).\n\n",
    );
    out.push_str(
        "Each row is a simulation scenario, by the sprint that added it. Each column is a unit \
         of a boundary crate, by the sprint that built it. Each cell counts the named points \
         of that unit the scenario reached, summed over all its cells. A later scenario that \
         reaches an earlier unit is a regression run passing back through that unit.\n\n",
    );
    out.push_str("| Scenario (sprint) |");
    for (sprint, unit) in &units {
        out.push_str(&format!(" S{sprint} `{unit}` |"));
    }
    out.push_str("\n|---|");
    for _ in &units {
        out.push_str("---:|");
    }
    out.push('\n');
    for (sprint, _, shown, recorded) in &rows {
        out.push_str(&format!("| {shown} (S{sprint}) |"));
        for (unit_sprint, unit) in &units {
            let hits: u64 = recorded
                .counts
                .iter()
                .filter(|(point, _)| {
                    points
                        .get(point.as_str())
                        .is_some_and(|e| e.unit == *unit && e.sprint == *unit_sprint)
                })
                .map(|(_, count)| count)
                .sum();
            out.push_str(&format!(" {hits} |"));
        }
        out.push('\n');
    }
    out.push_str("\n## Points\n\n| Point | Unit | Sprint | Hits |\n|---|---|---:|---:|\n");
    let mut listed: Vec<&&Entry> = points.values().collect();
    listed.sort_by_key(|entry| (entry.sprint, entry.unit.as_str(), entry.name.as_str()));
    for entry in listed {
        out.push_str(&format!(
            "| `{}` | `{}` | {} | {} |\n",
            entry.name,
            entry.unit,
            entry.sprint,
            totals[entry.name.as_str()]
        ));
    }
    Ok(out)
}

/// The table for the workspace at `root`, from its registries and the
/// latest recorded run. `None` when the workspace registers no point.
pub fn generate(root: &Path) -> Result<Option<String>, Vec<String>> {
    let (registries, mut problems) = points::registries(&members(root));
    if registries.values().all(Vec::is_empty) && problems.is_empty() {
        return Ok(None);
    }
    let dir = reach_dir(root);
    let mut scenarios = Vec::new();
    for (file, shown) in SCENARIOS {
        let path = dir.join(format!("{file}.txt"));
        match fs::read_to_string(&path) {
            Ok(text) => match parse(&text) {
                Ok(recorded) => scenarios.push((shown, recorded)),
                Err(why) => problems.push(format!("{}: {why}", path.display())),
            },
            Err(_) => problems.push(format!(
                "scenario `{shown}` has no reach file ({}); run `cargo test` first",
                path.display()
            )),
        }
    }
    if !problems.is_empty() {
        return Err(problems);
    }
    table(&registries, &scenarios).map(Some)
}

/// `cargo reach`: writes the table, or reports why it cannot.
pub fn run(root: &Path, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    match generate(root) {
        Ok(Some(table)) => match fs::write(root.join(TABLE), table) {
            Ok(()) => {
                writeln!(out, "reach: wrote {TABLE}").ok();
                0
            }
            Err(error) => {
                writeln!(err, "reach: {TABLE}: {error}").ok();
                1
            }
        },
        Ok(None) => {
            writeln!(out, "reach: no point is registered; nothing to write").ok();
            0
        }
        Err(problems) => {
            for problem in problems {
                writeln!(err, "reach: {problem}").ok();
            }
            1
        }
    }
}

/// `cargo ci`'s `reach` step: the generated table equals the committed one.
/// Writes nothing.
pub fn check(root: &Path) -> Result<(), String> {
    match generate(root) {
        Ok(None) => Ok(()),
        Ok(Some(table)) => match fs::read_to_string(root.join(TABLE)) {
            Ok(committed) if committed == table => Ok(()),
            Ok(_) => Err(format!("{TABLE} is stale; run `cargo reach` and commit it")),
            Err(_) => Err(format!(
                "{TABLE} is missing; run `cargo reach` and commit it"
            )),
        },
        Err(problems) => Err(problems.join("; ")),
    }
}

/// Removes the recorded reach, so a scenario that no longer records cannot
/// leave a stale file behind (`cargo ci` does this before `test`).
pub fn clear(root: &Path) {
    let _ = fs::remove_dir_all(reach_dir(root));
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{Recorded, parse, table};
    use crate::points::Entry;

    fn entry(name: &str, unit: &str, sprint: u32) -> Entry {
        Entry {
            name: name.to_owned(),
            unit: unit.to_owned(),
            sprint,
        }
    }

    fn registries() -> BTreeMap<String, Vec<Entry>> {
        [(
            "store".to_owned(),
            vec![
                entry("store.a.x", "store::a", 1),
                entry("store.b.x", "store::b", 2),
            ],
        )]
        .into()
    }

    fn recorded(name: &str, sprint: u32, counts: &[(&str, u64)]) -> Recorded {
        Recorded {
            name: name.to_owned(),
            sprint,
            counts: counts.iter().map(|(p, c)| ((*p).to_owned(), *c)).collect(),
        }
    }

    #[test]
    fn reach_table_is_generated() {
        let scenarios = [
            (
                "later",
                recorded("later", 2, &[("store.a.x", 3), ("store.b.x", 4)]),
            ),
            ("first", recorded("first", 1, &[("store.a.x", 5)])),
        ];
        let table = table(&registries(), &scenarios).unwrap();
        assert!(
            table.contains("| Scenario (sprint) | S1 `store::a` | S2 `store::b` |\n"),
            "{table}"
        );
        // Rows by sprint: `first` (S1) before `later` (S2).
        let first = table.find("| first (S1) | 5 | 0 |").unwrap();
        let later = table.find("| later (S2) | 3 | 4 |").unwrap();
        assert!(first < later);
        assert!(table.contains("| `store.a.x` | `store::a` | 1 | 8 |"));
    }

    /// A fixture workspace registering `store.a.x` (S1) and `store.b.x`
    /// (S2), with every scenario's reach file holding `counts`.
    fn fixture(label: &str, counts: &[(&str, u64)]) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!("xtask-reach-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let write = |path: &std::path::Path, text: &str| {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        };
        write(
            &root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/*\"]\n",
        );
        write(
            &root.join("crates/store/Cargo.toml"),
            "[package]\nname = \"store\"\n",
        );
        write(
            &root.join("crates/store/src/points.rs"),
            "pub const POINTS: &[Point] = &[\n    \
             Point { name: \"store.a.x\", unit: \"store::a\", sprint: 1 },\n    \
             Point { name: \"store.b.x\", unit: \"store::b\", sprint: 2 },\n];\n",
        );
        let body: String = counts.iter().map(|(p, c)| format!("{p} {c}\n")).collect();
        for (file, shown) in super::SCENARIOS {
            write(
                &super::reach_dir(&root).join(format!("{file}.txt")),
                &format!("scenario {shown}\nsprint 1\n{body}"),
            );
        }
        std::fs::create_dir_all(root.join("docs")).unwrap();
        root
    }

    /// `cargo reach` run twice writes the same bytes, and the check then
    /// passes (test critique C-010: through `run`, not only `table`).
    #[test]
    fn reach_is_idempotent() {
        let root = fixture("idempotent", &[("store.a.x", 1), ("store.b.x", 2)]);
        let mut written = Vec::new();
        for _ in 0..2 {
            let (mut out, mut err) = (Vec::new(), Vec::new());
            assert_eq!(super::run(&root, &mut out, &mut err), 0);
            written.push(std::fs::read(root.join(super::TABLE)).unwrap());
        }
        assert_eq!(written[0], written[1]);
        assert_eq!(super::check(&root), Ok(()));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn reach_fails_on_an_unreached_point() {
        let scenarios = [("first", recorded("first", 1, &[("store.a.x", 1)]))];
        let problems = table(&registries(), &scenarios).unwrap_err();
        assert_eq!(problems, ["point `store.b.x` is reached by no scenario"]);
        // Through `cargo reach`: exit 1, the point named, nothing written.
        let root = fixture("unreached", &[("store.a.x", 1)]);
        let (mut out, mut err) = (Vec::new(), Vec::new());
        assert_eq!(super::run(&root, &mut out, &mut err), 1);
        let err = String::from_utf8(err).unwrap();
        assert!(
            err.contains("point `store.b.x` is reached by no scenario"),
            "{err}"
        );
        assert!(!root.join(super::TABLE).exists());
        let _ = std::fs::remove_dir_all(&root);
    }

    /// A scenario that stops reaching a point the plan expects of it fails,
    /// though another scenario still reaches the point (test critique
    /// C-011). Expected points no registry declares are not checked here;
    /// `expectations_are_real` keeps the list real.
    #[test]
    fn reach_fails_on_an_expected_point_missed() {
        let registries: BTreeMap<String, Vec<Entry>> = [(
            "store".to_owned(),
            vec![
                entry("store.a.x", "store::a", 1),
                entry("store.commit_op.present_synced", "store::commit_op", 1),
            ],
        )]
        .into();
        let both = [("store.a.x", 1), ("store.commit_op.present_synced", 1)];
        let reaching = [
            ("store", recorded("store", 1, &both)),
            ("four writers", recorded("four writers", 4, &both)),
        ];
        assert!(table(&registries, &reaching).is_ok());
        let missing = [
            ("store", recorded("store", 1, &both)),
            (
                "four writers",
                recorded("four writers", 4, &[("store.a.x", 1)]),
            ),
        ];
        assert_eq!(
            table(&registries, &missing).unwrap_err(),
            ["scenario `four writers` no longer reaches point \
              `store.commit_op.present_synced`, which it is expected to"]
        );
    }

    /// Every expectation names a scenario `cargo reach` reads and a point
    /// this workspace registers, and every registered point is expected of
    /// some scenario, as the plan lists them.
    #[test]
    fn expectations_are_real() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(std::path::Path::parent)
            .unwrap();
        let (registries, problems) = crate::points::registries(&crate::workspace::members(root));
        assert!(problems.is_empty(), "{problems:?}");
        let registered: std::collections::BTreeSet<&str> = registries
            .values()
            .flatten()
            .map(|entry| entry.name.as_str())
            .collect();
        let mut expected = std::collections::BTreeSet::new();
        for (shown, points) in super::EXPECTED {
            assert!(
                super::SCENARIOS.iter().any(|(_, name)| *name == shown),
                "no scenario {shown}"
            );
            for point in points {
                assert!(registered.contains(point), "{point} is not registered");
                assert!(expected.insert(*point), "{point} is expected twice");
            }
        }
        assert_eq!(expected, registered);
    }

    #[test]
    fn reach_fails_on_an_unregistered_point() {
        let scenarios = [(
            "first",
            recorded(
                "first",
                1,
                &[("store.a.x", 1), ("store.b.x", 1), ("store.z.z", 1)],
            ),
        )];
        let problems = table(&registries(), &scenarios).unwrap_err();
        assert_eq!(
            problems,
            ["scenario `first` names point `store.z.z`, which no registry declares"]
        );
        // Through `cargo reach`: exit 1, the point named, nothing written.
        let root = fixture(
            "unregistered",
            &[("store.a.x", 1), ("store.b.x", 1), ("store.z.z", 1)],
        );
        let (mut out, mut err) = (Vec::new(), Vec::new());
        assert_eq!(super::run(&root, &mut out, &mut err), 1);
        let err = String::from_utf8(err).unwrap();
        assert!(err.contains("names point `store.z.z`"), "{err}");
        assert!(!root.join(super::TABLE).exists());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn reach_ignores_test_points() {
        let scenarios = [(
            "first",
            recorded(
                "first",
                1,
                &[
                    ("store.a.x", 1),
                    ("store.b.x", 1),
                    ("test.writers.listing", 9),
                ],
            ),
        )];
        assert!(table(&registries(), &scenarios).is_ok());
    }

    /// A workspace with a registered point and no recorded scenario: every
    /// scenario's missing file is named, and nothing is written.
    #[test]
    fn reach_fails_on_a_missing_scenario() {
        let root = std::env::temp_dir().join(format!("xtask-reach-missing-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let write = |path: &str, text: &str| {
            let path = root.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        };
        write("Cargo.toml", "[workspace]\nmembers = [\"crates/*\"]\n");
        write("crates/store/Cargo.toml", "[package]\nname = \"store\"\n");
        write(
            "crates/store/src/points.rs",
            "pub const POINTS: &[Point] = &[Point { name: \"store.a.x\", unit: \"store::a\", sprint: 1 }];\n",
        );
        let problems = super::generate(&root).unwrap_err();
        assert_eq!(problems.len(), super::SCENARIOS.len(), "{problems:?}");
        assert!(
            problems[0].contains("scenario `store` has no reach file"),
            "{problems:?}"
        );
        let (mut out, mut err) = (Vec::new(), Vec::new());
        assert_eq!(super::run(&root, &mut out, &mut err), 1);
        assert!(!root.join(super::TABLE).exists());
        let _ = std::fs::remove_dir_all(&root);
    }

    /// The committed table shows the owner's triangle: later scenarios
    /// pass back through earlier units (INT-0005 AC-6; plan critique
    /// C-013).
    #[test]
    fn reach_table_shows_the_triangle() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(std::path::Path::parent)
            .unwrap();
        let table = std::fs::read_to_string(root.join(super::TABLE)).unwrap();
        let rows: Vec<Vec<String>> = table
            .lines()
            .filter(|line| line.starts_with("| ") && !line.starts_with("| Point"))
            .map(|line| {
                line.trim_matches('|')
                    .split('|')
                    .map(|cell| cell.trim().to_owned())
                    .collect()
            })
            .collect();
        let header = &rows[0];
        let cell = |row: &str, column: &str| -> u64 {
            let at = header
                .iter()
                .position(|h| h == column)
                .unwrap_or_else(|| panic!("no column {column} in {header:?}"));
            let row = rows
                .iter()
                .find(|r| r[0] == row)
                .unwrap_or_else(|| panic!("no row {row}"));
            row[at].parse().unwrap()
        };
        for unit in ["S1 `store::commit_op`", "S1 `store::put_object`"] {
            assert!(cell("four writers (S4)", unit) > 0, "{unit}");
            assert!(cell("Materialise (S3)", unit) > 0, "{unit}");
        }
        assert!(cell("Materialise (S3)", "S2 `workspace::snapshot`") > 0);
    }

    /// `cargo ci`'s `reach` step passes on the table `cargo reach` wrote,
    /// fails on a stale or missing one, and writes nothing (INT-0005 AC-6).
    #[test]
    fn reach_check_fails_on_a_stale_table() {
        let root = std::env::temp_dir().join(format!("xtask-reach-stale-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let write = |path: &std::path::Path, text: &str| {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        };
        write(
            &root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/*\"]\n",
        );
        write(
            &root.join("crates/store/Cargo.toml"),
            "[package]\nname = \"store\"\n",
        );
        write(
            &root.join("crates/store/src/points.rs"),
            "pub const POINTS: &[Point] = &[Point { name: \"store.a.x\", unit: \"store::a\", sprint: 1 }];\n",
        );
        for (file, shown) in super::SCENARIOS {
            write(
                &super::reach_dir(&root).join(format!("{file}.txt")),
                &format!("scenario {shown}\nsprint 1\nstore.a.x 1\n"),
            );
        }
        let table = root.join(super::TABLE);
        std::fs::create_dir_all(table.parent().unwrap()).unwrap();
        assert!(super::check(&root).unwrap_err().contains("missing"));
        let (mut out, mut err) = (Vec::new(), Vec::new());
        assert_eq!(super::run(&root, &mut out, &mut err), 0);
        let written = std::fs::read_to_string(&table).unwrap();
        assert_eq!(super::check(&root), Ok(()));
        std::fs::write(&table, written.replace("| 1 |", "| 2 |")).unwrap();
        assert!(super::check(&root).unwrap_err().contains("stale"));
        // The check wrote nothing: the stale table is still there.
        assert_ne!(std::fs::read_to_string(&table).unwrap(), written);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn reach_files_parse() {
        let recorded = parse("scenario store\nsprint 1\nstore.a.x 3\n").unwrap();
        assert_eq!(recorded.sprint, 1);
        assert_eq!(recorded.counts["store.a.x"], 3);
        assert!(parse("sprint 1\n").is_err());
        assert!(parse("scenario s\nsprint x\n").is_err());
        assert!(parse("scenario s\nsprint 1\nstore.a.x many\n").is_err());
    }
}
