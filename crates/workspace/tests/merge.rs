//! The canonical merge of op heads (INT-0010; Sprint 7 T-003).
//!
//! Histories are generated from a seed: operations made in one to four
//! workspaces, each over one to three parents, each changing only its own
//! workspace's entry, with changesets from a pool of five so that equal
//! values and values set back are frequent. Each operation's view holds what
//! its maker saw, the reference merge of its parents, with its own entry
//! changed as a command changes it.
//!
//! The library's merge is checked:
//! - for AC-1, over every history;
//! - for AC-2 and AC-3, where each workspace's concurrent writers diverge
//!   from one base, the scope INT-0010 records;
//! - for AC-4, against a reference written here from the definition alone,
//!   with every ancestor set found by brute force.
//!
//! Wrong merges, written here too, show that the checks can fail (AC-6).

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::convert::Infallible;

use codec::{Operation, View, WorkspaceEntry};
use hash::{HashAlg, HashRef};
use sim::Rng;
use workspace::merge::{self, Ops};

/// Histories for each of AC-1 to AC-3.
const HISTORIES: u64 = 2_000;
/// Criss-cross histories for AC-4 (R1-E12's count).
const CRISS_CROSS: u64 = 4_000;
/// The changesets a pointer takes: few, so values repeat and come back.
const POOL: u64 = 5;

/// A stand-in changeset id, from the pool.
fn cs(n: u64) -> HashRef {
    HashAlg::Blake3.hash(&n.to_le_bytes())
}

/// An entry, from pool numbers.
fn entry(working_copy: &[u64], heads: &[u64]) -> WorkspaceEntry {
    WorkspaceEntry {
        working_copy: working_copy.iter().map(|n| cs(*n)).collect(),
        heads: heads.iter().map(|n| cs(*n)).collect(),
    }
}

fn set(ids: &[HashRef]) -> BTreeSet<HashRef> {
    ids.iter().copied().collect()
}

/// An op log, and the views its operations name.
#[derive(Clone, Default)]
struct History {
    ops: Ops,
    views: BTreeMap<HashRef, View>,
    /// The operations, in the order they were made.
    order: Vec<HashRef>,
    /// Each ancestor set found so far. Adding an operation changes none.
    found: RefCell<BTreeMap<HashRef, BTreeSet<HashRef>>>,
}

impl History {
    /// Adds an operation; `None` if the same one is there already.
    fn add(&mut self, parents: &BTreeSet<HashRef>, workspace: &str, view: View) -> Option<HashRef> {
        let view_id = HashAlg::Blake3.hash(&view.to_envelope());
        let op = Operation {
            parents: parents.clone(),
            workspace: workspace.to_owned(),
            view: view_id,
        };
        let id = HashAlg::Blake3.hash(&op.to_envelope());
        if self.ops.contains_key(&id) {
            return None;
        }
        self.views.insert(view_id, view);
        self.ops.insert(id, op);
        self.order.push(id);
        Some(id)
    }

    /// Adds an operation made in `workspace` over `parents` that sets its
    /// own entry to `own` in what it saw, the merge of its parents.
    fn commit(&mut self, parents: &[HashRef], workspace: &str, own: WorkspaceEntry) -> HashRef {
        let parents = set(parents);
        let mut view = if parents.is_empty() {
            View::default()
        } else {
            reference(self, &parents)
        };
        view.workspaces.insert(workspace.to_owned(), own);
        self.add(&parents, workspace, view)
            .expect("each hand-built operation is new")
    }

    fn view(&self, op: &HashRef) -> &View {
        &self.views[&self.ops[op].view]
    }

    /// Every operation `op` follows, itself excluded: a walk of the whole
    /// graph above it, kept for the next ask.
    fn ancestors(&self, op: &HashRef) -> BTreeSet<HashRef> {
        if let Some(found) = self.found.borrow().get(op) {
            return found.clone();
        }
        let mut seen = BTreeSet::new();
        let mut stack: Vec<HashRef> = self.ops[op].parents.iter().copied().collect();
        while let Some(id) = stack.pop() {
            if seen.insert(id) {
                stack.extend(self.ops[&id].parents.iter().copied());
            }
        }
        self.found.borrow_mut().insert(*op, seen.clone());
        seen
    }

    /// The operations no other operation follows.
    fn heads(&self) -> BTreeSet<HashRef> {
        let parents: BTreeSet<&HashRef> = self.ops.values().flat_map(|op| &op.parents).collect();
        self.ops
            .keys()
            .filter(|id| !parents.contains(id))
            .copied()
            .collect()
    }
}

/// A merge under test, given the heads in an order, repeats kept.
type MergeFn = dyn Fn(&History, &[HashRef]) -> View;

/// The library's merge. It takes a set, so an order or a repeat reaches it
/// as the one set a caller collects.
fn library(h: &History, heads: &[HashRef]) -> View {
    let Ok(view) = merge::merge(&h.ops, &set(heads), |id| {
        Ok::<_, Infallible>(h.views[id].clone())
    });
    view
}

// ---------------------------------------------------------------------------
// The reference: the definition, by brute force.
// ---------------------------------------------------------------------------

/// `heads` and every operation they follow.
fn closure(h: &History, heads: &BTreeSet<HashRef>) -> BTreeSet<HashRef> {
    let mut all = heads.clone();
    for head in heads {
        all.extend(h.ancestors(head));
    }
    all
}

/// The members of `of` that no other member follows.
fn latest(h: &History, of: &BTreeSet<HashRef>) -> BTreeSet<HashRef> {
    of.iter()
        .filter(|id| !of.iter().any(|other| h.ancestors(other).contains(id)))
        .copied()
        .collect()
}

/// The workspaces that made an operation among `heads` and what they follow.
fn names(h: &History, heads: &BTreeSet<HashRef>) -> BTreeSet<String> {
    closure(h, heads)
        .iter()
        .map(|id| h.ops[id].workspace.clone())
        .collect()
}

/// Workspace `name`'s writers: its operations among `heads` and what they
/// follow that no other of its operations there follows.
fn writers_ref(h: &History, heads: &BTreeSet<HashRef>, name: &str) -> BTreeSet<HashRef> {
    let made_there = closure(h, heads)
        .into_iter()
        .filter(|id| h.ops[id].workspace == name)
        .collect();
    latest(h, &made_there)
}

/// The latest operations made in `name` that every one of `writers` follows.
fn bases_ref(h: &History, name: &str, writers: &BTreeSet<HashRef>) -> BTreeSet<HashRef> {
    let mut common: Option<BTreeSet<HashRef>> = None;
    for writer in writers {
        let below: BTreeSet<HashRef> = h
            .ancestors(writer)
            .into_iter()
            .filter(|id| h.ops[id].workspace == name)
            .collect();
        common = Some(match common {
            None => below,
            Some(common) => common.intersection(&below).copied().collect(),
        });
    }
    latest(h, &common.unwrap_or_default())
}

/// How entries are merged against their base.
type Rule = fn(Option<&WorkspaceEntry>, &[&WorkspaceEntry]) -> Option<WorkspaceEntry>;

/// The rule, from its definition: a pointer no writer changed keeps the
/// base's value, and otherwise holds every changed writer's value; a head
/// stays if it is in the base and every writer kept it, or is new and some
/// writer has it.
fn rule(base: Option<&WorkspaceEntry>, members: &[&WorkspaceEntry]) -> Option<WorkspaceEntry> {
    let base = base.cloned().unwrap_or_default();
    let changed: Vec<&&WorkspaceEntry> = members
        .iter()
        .filter(|m| m.working_copy != base.working_copy)
        .collect();
    let working_copy: BTreeSet<HashRef> = if changed.is_empty() {
        base.working_copy.clone()
    } else {
        changed
            .iter()
            .flat_map(|m| m.working_copy.iter().copied())
            .collect()
    };
    let candidates: BTreeSet<HashRef> = base
        .heads
        .iter()
        .chain(members.iter().flat_map(|m| &m.heads))
        .copied()
        .collect();
    let heads = candidates
        .into_iter()
        .filter(|id| {
            if base.heads.contains(id) {
                members.iter().all(|m| m.heads.contains(id))
            } else {
                members.iter().any(|m| m.heads.contains(id))
            }
        })
        .collect();
    (!working_copy.is_empty()).then_some(WorkspaceEntry {
        working_copy,
        heads,
    })
}

/// The merge of `heads` from the definition, with `rule` for entries.
/// `first_base` takes the smallest of several bases in place of merging
/// them: a wrong merge, for AC-6.
fn reference_with(h: &History, heads: &BTreeSet<HashRef>, rule: Rule, first_base: bool) -> View {
    if let [head] = heads.iter().collect::<Vec<_>>()[..] {
        return h.view(head).clone();
    }
    let mut view = View::default();
    for name in names(h, heads) {
        let writers = writers_ref(h, heads, &name);
        if let Some(entry) = entry_ref(h, &name, &writers, rule, first_base) {
            view.workspaces.insert(name, entry);
        }
    }
    view
}

fn entry_ref(
    h: &History,
    name: &str,
    writers: &BTreeSet<HashRef>,
    rule: Rule,
    first_base: bool,
) -> Option<WorkspaceEntry> {
    if let [writer] = writers.iter().collect::<Vec<_>>()[..] {
        return h.view(writer).workspaces.get(name).cloned();
    }
    if writers.is_empty() {
        return None;
    }
    let mut bases = bases_ref(h, name, writers);
    if first_base {
        bases = bases.first().into_iter().copied().collect();
    }
    let base = entry_ref(h, name, &bases, rule, first_base);
    let members: Vec<&WorkspaceEntry> = writers
        .iter()
        .map(|writer| &h.view(writer).workspaces[name])
        .collect();
    rule(base.as_ref(), &members)
}

fn reference(h: &History, heads: &BTreeSet<HashRef>) -> View {
    reference_with(h, heads, rule, false)
}

/// The classic three-way merge of views, entry by entry: a value both sides
/// hold, or the one side that changed, or both sides' values together; a
/// head by the same rule, on its own.
fn three_way(base: &View, a: &View, b: &View) -> View {
    let names: BTreeSet<&String> = base
        .workspaces
        .keys()
        .chain(a.workspaces.keys())
        .chain(b.workspaces.keys())
        .collect();
    let mut view = View::default();
    for name in names {
        let get = |v: &View| v.workspaces.get(name).cloned().unwrap_or_default();
        let (o, x, y) = (get(base), get(a), get(b));
        let working_copy = if x.working_copy == y.working_copy || y.working_copy == o.working_copy {
            x.working_copy.clone()
        } else if x.working_copy == o.working_copy {
            y.working_copy.clone()
        } else {
            x.working_copy.union(&y.working_copy).copied().collect()
        };
        let heads = o
            .heads
            .iter()
            .chain(&x.heads)
            .chain(&y.heads)
            .filter(|id| {
                let (i, j, k) = (
                    o.heads.contains(id),
                    x.heads.contains(id),
                    y.heads.contains(id),
                );
                if j == k || j != i { j } else { k }
            })
            .copied()
            .collect();
        if !working_copy.is_empty() {
            view.workspaces.insert(
                name.clone(),
                WorkspaceEntry {
                    working_copy,
                    heads,
                },
            );
        }
    }
    view
}

// ---------------------------------------------------------------------------
// Histories.
// ---------------------------------------------------------------------------

/// A command's change to its own entry `old`: the pointer moves to a
/// changeset of the pool, and the one it continued from (the smallest, as
/// INT-0014 says) leaves the heads when the new one follows it; now and then
/// another head comes or goes. With `merges`, one operation in five changes
/// nothing and only merges what it saw.
fn change(rng: &mut Rng, old: Option<&WorkspaceEntry>, merges: bool) -> WorkspaceEntry {
    let to = rng.below(POOL);
    let Some(old) = old else {
        return entry(&[to], &[to]);
    };
    if merges && rng.below(5) == 0 {
        return old.clone();
    }
    let mut next = old.clone();
    let from = *old.working_copy.first().expect("a working copy");
    next.working_copy = [cs(to)].into();
    if rng.coin() {
        next.heads.remove(&from);
    }
    next.heads.insert(cs(to));
    match rng.below(4) {
        0 => {
            next.heads.insert(cs(rng.below(POOL)));
        }
        1 => {
            let pick = rng.below(next.heads.len() as u64) as usize;
            if let Some(id) = next.heads.iter().nth(pick).copied() {
                next.heads.remove(&id);
            }
        }
        _ => {}
    }
    next
}

/// Picks one of `items`.
fn pick<'a, T>(rng: &mut Rng, items: &'a [T]) -> &'a T {
    &items[rng.below(items.len() as u64) as usize]
}

/// Shuffles `items` (Fisher and Yates).
fn shuffle<T>(rng: &mut Rng, items: &mut [T]) {
    for i in (1..items.len()).rev() {
        let j = rng.below(i as u64 + 1) as usize;
        items.swap(i, j);
    }
}

/// Adds `count` random operations to `h`: each over one to three operations
/// taken at random, reduced to those no other taken one follows, and made in
/// a workspace its parents' merge holds or, now and then, in a new one, up to
/// `workspaces` in all.
fn grow(h: &mut History, rng: &mut Rng, count: usize, workspaces: usize, merges: bool) {
    let target = h.order.len() + count;
    for _ in 0..count * 10 {
        if h.order.len() >= target {
            return;
        }
        let taken: BTreeSet<HashRef> = (0..1 + rng.below(3))
            .map(|_| *pick(rng, &h.order))
            .collect();
        let parents = latest(h, &taken);
        let seen = reference(h, &parents);
        let made = names(h, &h.ops.keys().copied().collect()).len();
        let name = if made < workspaces && rng.below(3) == 0 {
            format!("w{made}")
        } else {
            let held: Vec<&String> = seen.workspaces.keys().collect();
            (*pick(rng, &held)).clone()
        };
        let own = change(rng, seen.workspaces.get(&name), merges);
        let mut view = seen;
        view.workspaces.insert(name.clone(), own);
        h.add(&parents, &name, view);
    }
}

/// A history from `seed`: the root, made in `w0`, then 3 to 12 operations by
/// [`grow`], in one to four workspaces.
fn history(seed: u64) -> History {
    let mut rng = Rng::new(seed);
    let workspaces = 1 + rng.below(4) as usize;
    let count = 3 + rng.below(10) as usize;
    let mut h = History::default();
    let first = rng.below(POOL);
    h.commit(&[], "w0", entry(&[first], &[first]));
    grow(&mut h, &mut rng, count, workspaces, true);
    h
}

/// A history whose concurrent writers diverge from one base: a line of
/// commands that makes every workspace, then two to four lines from its last
/// operation, each one to three commands over its own line only. Every
/// operation is a command, so every base's pointer holds one changeset.
fn forked_history(seed: u64) -> History {
    let mut rng = Rng::new(seed);
    let names: Vec<String> = (0..1 + rng.below(3)).map(|i| format!("w{i}")).collect();
    let mut h = History::default();
    let first = rng.below(POOL);
    let mut tip = h.commit(&[], &names[0], entry(&[first], &[first]));
    /// A command made in `name` over `over` alone; `over` again when the
    /// same operation is already there (another line made it).
    fn command(h: &mut History, rng: &mut Rng, over: HashRef, name: &str) -> HashRef {
        let seen = reference(h, &set(&[over]));
        let own = change(rng, seen.workspaces.get(name), false);
        let mut view = seen;
        view.workspaces.insert(name.to_owned(), own);
        h.add(&set(&[over]), name, view).unwrap_or(over)
    }
    for name in &names[1..] {
        tip = command(&mut h, &mut rng, tip, name);
    }
    for _ in 0..rng.below(3) {
        let name = pick(&mut rng, &names).clone();
        tip = command(&mut h, &mut rng, tip, &name);
    }
    for _ in 0..2 + rng.below(3) {
        let mut line = tip;
        for _ in 0..1 + rng.below(3) {
            let name = pick(&mut rng, &names).clone();
            line = command(&mut h, &mut rng, line, &name);
        }
    }
    h
}

/// Whether, in every workspace, the writers of `heads` diverge from one
/// base: every two of them have the one latest common writer all have.
fn one_base(h: &History, heads: &BTreeSet<HashRef>) -> bool {
    names(h, heads).iter().all(|name| {
        let writers: Vec<HashRef> = writers_ref(h, heads, name).into_iter().collect();
        if writers.len() < 2 {
            return true;
        }
        let all = bases_ref(h, name, &writers.iter().copied().collect());
        all.len() == 1
            && writers.iter().enumerate().all(|(i, a)| {
                writers[i + 1..]
                    .iter()
                    .all(|b| bases_ref(h, name, &set(&[*a, *b])) == all)
            })
    })
}

/// Whether `head` carries no merged copy: in its own history, every
/// workspace has one latest operation.
fn carries_no_merged_copy(h: &History, head: &HashRef) -> bool {
    let own = set(&[*head]);
    names(h, &own)
        .iter()
        .all(|name| writers_ref(h, &own, name).len() <= 1)
}

// ---------------------------------------------------------------------------
// The properties, for any merge.
// ---------------------------------------------------------------------------

/// AC-1 for one history: its heads in three random orders, each with one or
/// two repeats, give the view they give in id order.
fn check_ac1(h: &History, seed: u64, merge: &MergeFn) -> Result<(), String> {
    let heads: Vec<HashRef> = h.heads().into_iter().collect();
    let expected = merge(h, &heads);
    let mut rng = Rng::new(seed);
    for _ in 0..3 {
        let mut order = heads.clone();
        shuffle(&mut rng, &mut order);
        for _ in 0..=rng.below(2) {
            let repeat = *pick(&mut rng, &heads);
            let at = rng.below(order.len() as u64 + 1) as usize;
            order.insert(at, repeat);
        }
        if merge(h, &order) != expected {
            return Err(format!("{} heads in another order", heads.len()));
        }
    }
    Ok(())
}

/// AC-2 for one history: every two operations, neither following the other,
/// with one latest common ancestor, neither carrying a merged copy. Returns
/// how many pairs it checked.
fn check_ac2(h: &History, merge: &MergeFn) -> Result<usize, String> {
    let mut checked = 0;
    for (i, a) in h.order.iter().enumerate() {
        for b in &h.order[i + 1..] {
            let (below_a, below_b) = (h.ancestors(a), h.ancestors(b));
            if below_a.contains(b) || below_b.contains(a) {
                continue;
            }
            let common = below_a.intersection(&below_b).copied().collect();
            let [ancestor] = latest(h, &common).into_iter().collect::<Vec<_>>()[..] else {
                continue;
            };
            if !carries_no_merged_copy(h, a) || !carries_no_merged_copy(h, b) {
                continue;
            }
            let expected = three_way(h.view(&ancestor), h.view(a), h.view(b));
            if merge(h, &[*a, *b]) != expected {
                return Err("two heads differ from the three-way merge".to_owned());
            }
            checked += 1;
        }
    }
    Ok(checked)
}

/// The `HISTORIES` histories AC-1 and AC-2 run over.
fn histories() -> Vec<History> {
    (0..HISTORIES).map(history).collect()
}

/// How many of `histories` fail `check`, given each history and its seed.
fn failures(histories: &[History], check: impl Fn(&History, u64) -> Result<(), String>) -> usize {
    (0..)
        .zip(histories)
        .filter(|(seed, h)| check(h, *seed).is_err())
        .count()
}

// ---------------------------------------------------------------------------
// The library's merge.
// ---------------------------------------------------------------------------

#[test]
fn one_head_is_its_view() {
    // A view with an entry for a workspace that made no operation: the
    // merge of one head gives the head's view as it is.
    let mut h = History::default();
    let mut view = View::default();
    view.workspaces.insert("w0".to_owned(), entry(&[0], &[0]));
    view.workspaces
        .insert("elsewhere".to_owned(), entry(&[1, 2], &[]));
    let root = h.add(&BTreeSet::new(), "w0", view.clone()).unwrap();
    assert_eq!(library(&h, &[root]), view);
    assert_eq!(library(&h, &[root, root]), view);
}

#[test]
fn entries_merge_by_their_rules() {
    let base = entry(&[0], &[0, 1]);
    let merged = |members: &[WorkspaceEntry]| {
        let members: Vec<Option<&WorkspaceEntry>> = members.iter().map(Some).collect();
        merge::merge_entries(Some(&base), &members).unwrap()
    };
    // The pointer: no writer changed it; one did; two set one value; two set
    // two values; one set it back while another moved it.
    assert_eq!(
        merged(&[entry(&[0], &[0, 1]), entry(&[0], &[0, 1])]).working_copy,
        set(&[cs(0)])
    );
    assert_eq!(
        merged(&[entry(&[0], &[0, 1]), entry(&[2], &[0, 1])]).working_copy,
        set(&[cs(2)])
    );
    assert_eq!(
        merged(&[entry(&[2], &[0, 1]), entry(&[2], &[0, 1])]).working_copy,
        set(&[cs(2)])
    );
    assert_eq!(
        merged(&[entry(&[2], &[0, 1]), entry(&[3], &[0, 1])]).working_copy,
        set(&[cs(2), cs(3)])
    );
    assert_eq!(
        merged(&[entry(&[0], &[0, 1]), entry(&[3], &[0, 1])]).working_copy,
        set(&[cs(3)])
    );
    // The heads: kept by all; removed by one; added by one.
    assert_eq!(
        merged(&[entry(&[0], &[0, 1]), entry(&[0], &[0, 1])]).heads,
        set(&[cs(0), cs(1)])
    );
    assert_eq!(
        merged(&[entry(&[0], &[0]), entry(&[0], &[0, 1])]).heads,
        set(&[cs(0)])
    );
    assert_eq!(
        merged(&[entry(&[0], &[0, 1, 4]), entry(&[0], &[0, 1])]).heads,
        set(&[cs(0), cs(1), cs(4)])
    );
    // No base: every writer's value is a change.
    let members = [entry(&[2], &[2]), entry(&[3], &[3])];
    let members: Vec<Option<&WorkspaceEntry>> = members.iter().map(Some).collect();
    assert_eq!(
        merge::merge_entries(None, &members),
        Some(entry(&[2, 3], &[2, 3]))
    );
}

#[test]
fn writers_and_bases_are_found() {
    // r (a) -> x (b) -> a1 (a), a2 (a) both over x -> m (a) over a1, a2;
    // c (c) over a1; and a3 (a) over r alone.
    let mut h = History::default();
    let r = h.commit(&[], "a", entry(&[0], &[0]));
    let x = h.commit(&[r], "b", entry(&[1], &[1]));
    let a1 = h.commit(&[x], "a", entry(&[2], &[2]));
    let a2 = h.commit(&[x], "a", entry(&[3], &[3]));
    let c = h.commit(&[a1], "c", entry(&[4], &[4]));
    let a3 = h.commit(&[r], "a", entry(&[1], &[1]));
    let heads = set(&[c, a2, a3]);
    let writers = merge::writers(&h.ops, &heads);
    assert_eq!(writers["a"], set(&[a1, a2, a3]));
    assert_eq!(writers["b"], set(&[x]));
    assert_eq!(writers["c"], set(&[c]));
    // a1 and a2 share r only as an operation made in a, though both follow x.
    assert_eq!(merge::bases(&h.ops, "a", &set(&[a1, a2])), set(&[r]));
    assert_eq!(merge::bases(&h.ops, "a", &set(&[a1, a2, a3])), set(&[r]));
    // A criss-cross: m1 and m2 each over a1 and a2.
    let m1 = h.commit(&[a1, a2], "a", entry(&[2], &[2, 3]));
    let m2 = h.commit(&[a1, a2], "a", entry(&[3], &[2, 3]));
    assert_eq!(merge::bases(&h.ops, "a", &set(&[m1, m2])), set(&[a1, a2]));
    // A workspace whose writers have no operation in common.
    let mut g = History::default();
    let p = g.commit(&[], "a", entry(&[0], &[0]));
    let q = g.commit(&[], "a", entry(&[1], &[1]));
    assert_eq!(merge::bases(&g.ops, "a", &set(&[p, q])), BTreeSet::new());
    assert_eq!(
        library(&g, &[p, q]).workspaces["a"],
        entry(&[0, 1], &[0, 1])
    );
    for heads in [heads, set(&[m1, m2, c, a3])] {
        for name in names(&h, &heads) {
            assert_eq!(
                merge::writers(&h.ops, &heads)[&name],
                writers_ref(&h, &heads, &name)
            );
        }
    }
}

#[test]
fn merge_ignores_order_and_repeats() {
    assert_eq!(
        failures(&histories(), |h, seed| check_ac1(h, seed, &library)),
        0
    );
}

#[test]
fn two_heads_equal_the_three_way_merge() {
    let mut pairs = 0;
    for (seed, h) in histories().iter().enumerate() {
        pairs += check_ac2(h, &library).unwrap_or_else(|why| panic!("seed {seed}: {why}"));
    }
    println!("AC-2: {pairs} pairs checked");
    assert!(pairs >= 1_000, "{pairs} pairs");
}

#[test]
fn committed_partial_merges_change_nothing() {
    let mut nontrivial = 0;
    for seed in 0..HISTORIES {
        let h = forked_history(seed);
        let heads: Vec<HashRef> = h.heads().into_iter().collect();
        assert!(one_base(&h, &set(&heads)), "seed {seed}");
        if heads.len() < 2 {
            continue;
        }
        let flat = library(&h, &heads);
        assert_eq!(flat, reference(&h, &set(&heads)), "seed {seed}");
        let mut rng = Rng::new(seed);
        let mut order = heads.clone();
        shuffle(&mut rng, &mut order);
        let taken = 2 + rng.below(heads.len() as u64 - 1) as usize;
        let (merged, rest) = order.split_at(taken);
        // The partial merge: an operation over `merged`, made in a workspace
        // its merge holds, changing nothing.
        let mut g = h.clone();
        let seen = reference(&g, &set(merged));
        let held: Vec<&String> = seen.workspaces.keys().collect();
        let name = (*pick(&mut rng, &held)).clone();
        if writers_ref(&g, &set(merged), &name).len() >= 2 {
            nontrivial += 1;
        }
        let m = g.add(&set(merged), &name, seen).expect("a new operation");
        let mut after = rest.to_vec();
        after.push(m);
        assert_eq!(library(&g, &after), flat, "seed {seed}");
    }
    println!("AC-3: {nontrivial} partial merges made in a workspace with two writers or more");
    assert!(nontrivial >= 200, "{nontrivial}");
}

/// A history from `seed` with a criss-cross: [`history`], then in one
/// workspace two commands over one operation, two more each over both, and
/// a few more operations by [`grow`].
fn criss_cross_history(seed: u64) -> History {
    let mut h = history(seed);
    let mut rng = Rng::new(seed ^ 0x5eed);
    let over = *pick(&mut rng, &h.order);
    let seen = reference(&h, &set(&[over]));
    let held: Vec<String> = seen.workspaces.keys().cloned().collect();
    let name = pick(&mut rng, &held).clone();
    let command = |h: &mut History, rng: &mut Rng, parents: &[HashRef]| {
        let seen = reference(h, &set(parents));
        for _ in 0..20 {
            let own = change(rng, seen.workspaces.get(&name), false);
            let mut view = seen.clone();
            view.workspaces.insert(name.clone(), own);
            if let Some(id) = h.add(&set(parents), &name, view) {
                return id;
            }
        }
        panic!("no new command");
    };
    let a = command(&mut h, &mut rng, &[over]);
    let b = command(&mut h, &mut rng, &[over]);
    command(&mut h, &mut rng, &[a, b]);
    command(&mut h, &mut rng, &[a, b]);
    let workspaces = 1 + rng.below(4) as usize;
    let more = rng.below(4) as usize;
    grow(&mut h, &mut rng, more, workspaces, true);
    h
}

/// Whether some workspace's writers among `heads` have several bases.
fn criss_crosses(h: &History, heads: &BTreeSet<HashRef>) -> bool {
    names(h, heads).iter().any(|name| {
        let writers = writers_ref(h, heads, name);
        writers.len() >= 2 && bases_ref(h, name, &writers).len() >= 2
    })
}

/// Checks `merge` against the reference over `CRISS_CROSS` criss-cross
/// histories, and returns how many differ.
fn criss_cross_failures(merge: &MergeFn) -> u64 {
    let mut found = 0;
    let mut failed = 0;
    let mut seed = 0;
    while found < CRISS_CROSS {
        let h = criss_cross_history(seed);
        seed += 1;
        let heads = h.heads();
        if !criss_crosses(&h, &heads) {
            continue;
        }
        found += 1;
        let list: Vec<HashRef> = heads.iter().copied().collect();
        if merge(&h, &list) != reference(&h, &heads) {
            failed += 1;
        }
    }
    failed
}

#[test]
fn criss_cross_histories_match_the_reference() {
    assert_eq!(criss_cross_failures(&library), 0);
}

#[test]
fn a_resolution_is_not_reopened() {
    // Two commands of w0 move its pointer apart; a third sees both and
    // resolves it; an operation of x, made before the resolution, still
    // carries the conflict.
    let mut h = History::default();
    let r = h.commit(&[], "w0", entry(&[0], &[0]));
    let a = h.commit(&[r], "w0", entry(&[1], &[1]));
    let b = h.commit(&[r], "w0", entry(&[2], &[2]));
    let x = h.commit(&[a, b], "x", entry(&[9], &[9]));
    assert_eq!(h.view(&x).workspaces["w0"], entry(&[1, 2], &[1, 2]));
    let resolved = h.commit(&[a, b], "w0", entry(&[1], &[1, 2]));
    let merged = library(&h, &[x, resolved]);
    assert_eq!(merged.workspaces["w0"], entry(&[1], &[1, 2]));
    // Two commands that each resolved it the same way, from both sides: a
    // criss-cross whose base is the conflict. It stays resolved.
    let again = h.commit(&[a, b], "w0", entry(&[1], &[1]));
    let merged = library(&h, &[x, resolved, again]);
    assert_eq!(merged.workspaces["w0"].working_copy, set(&[cs(1)]));
    assert_eq!(merged, reference(&h, &set(&[x, resolved, again])));
}

#[test]
fn stale_copies_do_not_override_the_owner() {
    // a moves its pointer twice; b loaded before both moves, c between them.
    let mut h = History::default();
    let r = h.commit(&[], "a", entry(&[0], &[0]));
    let b = h.commit(&[r], "b", entry(&[5], &[5]));
    let a1 = h.commit(&[r], "a", entry(&[1], &[1]));
    let c = h.commit(&[a1], "c", entry(&[6], &[6]));
    let a2 = h.commit(&[a1], "a", entry(&[2], &[2]));
    let merged = library(&h, &[b, c, a2]);
    assert_eq!(merged.workspaces["a"], entry(&[2], &[2]));
    assert_eq!(merged.workspaces["b"], entry(&[5], &[5]));
    assert_eq!(merged.workspaces["c"], entry(&[6], &[6]));
    // One base for every head, their latest common ancestor r, would read
    // c's copy of a's entry, 1, as a change, and conflict with 2.
    let all_heads_against_r: BTreeSet<HashRef> = [b, c, a2]
        .iter()
        .map(|op| &h.view(op).workspaces["a"].working_copy)
        .filter(|wc| **wc != set(&[cs(0)]))
        .flatten()
        .copied()
        .collect();
    assert_eq!(all_heads_against_r, set(&[cs(1), cs(2)]));
}

#[test]
fn merged_copies_and_nested_set_backs_are_pinned() {
    // INT-0010's known limit, pinned with its values. First, a head that
    // carries a merged copy (Sprint 7 plan critique, round 1, C-001): w's
    // c1 sets 1 and wB sets 2, both over r; y, made in lane y over both,
    // carries w as {1, 2}; w's wA then replaces c1's 1 with 3, over c1.
    let mut h = History::default();
    let r = h.commit(&[], "w", entry(&[0], &[0]));
    let c1 = h.commit(&[r], "w", entry(&[1], &[1]));
    let wb = h.commit(&[r], "w", entry(&[2], &[2]));
    let y = h.commit(&[c1, wb], "y", entry(&[9], &[9]));
    let wa = h.commit(&[c1], "w", entry(&[3], &[3]));
    assert!(!carries_no_merged_copy(&h, &y));
    assert_eq!(
        library(&h, &[wa, y]).workspaces["w"].working_copy,
        set(&[cs(2), cs(3)])
    );
    let classic = three_way(h.view(&c1), h.view(&wa), h.view(&y));
    assert_eq!(
        classic.workspaces["w"].working_copy,
        set(&[cs(1), cs(2), cs(3)])
    );

    // Then nested writers with a value set back. n's pointer: r 0; p 1 and q 0, both
    // over r; L, made in x, over p and q; n_a sets 0 and n_b keeps 1, both
    // over p; heads a over L and n_a, b over L and n_b, both made in x.
    let build = |q_value: u64| {
        let mut h = History::default();
        let r = h.commit(&[], "n", entry(&[0], &[0]));
        let p = h.commit(&[r], "n", entry(&[1], &[1]));
        let q = h.commit(&[r], "n", entry(&[q_value], &[q_value]));
        let l = h.commit(&[p, q], "x", entry(&[9], &[9]));
        let n_a = h.commit(&[p], "n", entry(&[0], &[0]));
        let n_b = h.commit(&[p], "n", entry(&[1], &[1, 7]));
        let a = h.commit(&[l, n_a], "x", entry(&[8], &[8]));
        let b = h.commit(&[l, n_b], "x", entry(&[7], &[7]));
        (h, l, a, b)
    };
    // n_a's move back to 0 reads as no change against r, so the merge keeps
    // 1, which n_a moved away from; the three-way merge over L gives 0.
    let (h, l, a, b) = build(0);
    assert!(!carries_no_merged_copy(&h, &a));
    assert_eq!(
        library(&h, &[a, b]).workspaces["n"].working_copy,
        set(&[cs(1)])
    );
    let classic = three_way(h.view(&l), h.view(&a), h.view(&b));
    assert_eq!(classic.workspaces["n"].working_copy, set(&[cs(0)]));
    // With q at 2, the same reading adds 1 to the pointer, a spurious member.
    let (h, l, a, b) = build(2);
    assert_eq!(
        library(&h, &[a, b]).workspaces["n"].working_copy,
        set(&[cs(1), cs(2)])
    );
    let classic = three_way(h.view(&l), h.view(&a), h.view(&b));
    assert_eq!(classic.workspaces["n"].working_copy, set(&[cs(2)]));
}

// ---------------------------------------------------------------------------
// Wrong merges (AC-6).
// ---------------------------------------------------------------------------

/// The old pairwise fold: the heads merged two at a time in the order given,
/// each step a three-way merge of whole views against the latest common
/// ancestors of what is merged so far and the next head.
fn pairwise_fold(h: &History, heads: &[HashRef]) -> View {
    let mut done = set(&heads[..1]);
    let mut acc = h.view(&heads[0]).clone();
    for next in &heads[1..] {
        let below_done = closure(h, &done);
        let mut below_next = h.ancestors(next);
        below_next.insert(*next);
        let common = below_done.intersection(&below_next).copied().collect();
        let lcas = latest(h, &common);
        let base = if lcas.is_empty() {
            View::default()
        } else {
            reference(h, &lcas)
        };
        acc = three_way(&base, &acc, h.view(next));
        done.insert(*next);
    }
    acc
}

/// "If the first and last heads agree, return that head" (R4 change c2).
fn fast_path(h: &History, heads: &[HashRef]) -> View {
    if heads.first() == heads.last() {
        return h.view(&heads[0]).clone();
    }
    library(h, heads)
}

/// A pointer two writers moved apart keeps the base's value.
fn always_base(h: &History, heads: &[HashRef]) -> View {
    fn rule_always_base(
        base: Option<&WorkspaceEntry>,
        members: &[&WorkspaceEntry],
    ) -> Option<WorkspaceEntry> {
        let mut merged = rule(base, members)?;
        let moved: BTreeSet<&BTreeSet<HashRef>> = members
            .iter()
            .map(|m| &m.working_copy)
            .filter(|wc| base.is_none_or(|b| b.working_copy != **wc))
            .collect();
        if let (true, Some(base)) = (moved.len() >= 2, base) {
            merged.working_copy = base.working_copy.clone();
        }
        Some(merged)
    }
    reference_with(h, &set(heads), rule_always_base, false)
}

/// A head one writer removed stays because another kept it.
fn union_of_heads(h: &History, heads: &[HashRef]) -> View {
    fn rule_union(
        base: Option<&WorkspaceEntry>,
        members: &[&WorkspaceEntry],
    ) -> Option<WorkspaceEntry> {
        let mut merged = rule(base, members)?;
        merged.heads = members
            .iter()
            .flat_map(|m| m.heads.iter().copied())
            .collect();
        Some(merged)
    }
    reference_with(h, &set(heads), rule_union, false)
}

/// A pointer two writers moved apart takes the value of the one the heads
/// reach first, in the order given: Jujutsu's rule for a working-copy commit
/// both sides moved, as research found it.
fn first_side_wins(h: &History, heads: &[HashRef]) -> View {
    let all = set(heads);
    let mut view = reference(h, &all);
    for (name, merged) in &mut view.workspaces {
        let writers = writers_ref(h, &all, name);
        if writers.len() < 2 || merged.working_copy.len() < 2 {
            continue;
        }
        let base = entry_ref(h, name, &bases_ref(h, name, &writers), rule, false);
        let first = heads.iter().find_map(|head| {
            let mut below = h.ancestors(head);
            below.insert(*head);
            writers
                .iter()
                .filter(|w| below.contains(w))
                .map(|w| &h.view(w).workspaces[name].working_copy)
                .find(|wc| base.as_ref().is_none_or(|b| b.working_copy != **wc))
        });
        if let Some(first) = first {
            merged.working_copy = first.clone();
        }
    }
    view
}

/// The merge that takes the smallest of several bases by id instead of
/// merging them.
fn arbitrary_ancestor(h: &History, heads: &[HashRef]) -> View {
    reference_with(h, &set(heads), rule, true)
}

#[test]
fn wrong_merges_are_rejected() {
    // Each wrong merge, run through AC-1 over every history and AC-2 over
    // the pairs that check covers; the table is R5 §A.7's split: AC-1 alone
    // misses the convergent ones, AC-2 alone the ones that agree on two
    // heads.
    let wrong: [(&str, &MergeFn, bool, bool); 5] = [
        ("pairwise fold", &pairwise_fold, true, false),
        ("first-equals-last fast path", &fast_path, true, false),
        ("always-base", &always_base, false, true),
        ("union of heads", &union_of_heads, false, true),
        ("first-side-wins", &first_side_wins, true, true),
    ];
    let histories = histories();
    for (name, merge, fails_ac1, fails_ac2) in wrong {
        let ac1 = failures(&histories, |h, seed| check_ac1(h, seed, merge));
        let ac2 = failures(&histories, |h, _| check_ac2(h, merge).map(|_| ()));
        println!("{name}: AC-1 fails in {ac1} of {HISTORIES} histories, AC-2 in {ac2}");
        assert_eq!((ac1 > 0, ac2 > 0), (fails_ac1, fails_ac2), "{name}");
    }
}

#[test]
fn an_arbitrary_ancestor_fails_criss_cross() {
    let failed = criss_cross_failures(&arbitrary_ancestor);
    println!(
        "an arbitrary common ancestor: {failed} of {CRISS_CROSS} criss-cross histories differ"
    );
    assert!(failed > 0);
}
