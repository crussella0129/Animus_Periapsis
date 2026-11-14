//! The canonical merge of op heads (INT-0010; Sprint 7 T-003).
//!
//! A command that finds several op heads works from the view they leave
//! together. That view is computed here and never committed: it depends only
//! on the set of heads and on the operations they follow, so every process
//! that holds the same operations computes the same view, whatever order it
//! found the heads in.
//!
//! The merge goes entry by entry. An entry is written only by operations
//! made in its workspace (INT-0016), so its value comes only from those. Its
//! **writers** are the latest operations made in that workspace among the
//! heads and everything they follow: those that no other operation made
//! there follows. One writer gives its own entry. Several are merged against
//! their **base**:
//! - the entry at their latest common operation made in that workspace;
//! - or, when the workspace's history criss-crosses and there are several
//!   such operations, the merge of those, found the same way;
//! - or no entry, when they have none in common.
//!
//! Copies of an entry that other workspaces' operations carry are only what
//! those operations saw. They are never read, so an agent that loaded early
//! cannot undo what an entry's owner did since.
//!
//! INT-0010 states the rule and its known limit; T-003's tests pin both.

use std::collections::{BTreeMap, BTreeSet};

use codec::{Operation, View, WorkspaceEntry};
use hash::HashRef;

/// A log's operations, by id.
pub type Ops = BTreeMap<HashRef, Operation>;

/// The view `heads` leave together. `read` gives a view by its id; each is
/// asked for at most once. With one head this is that head's view.
///
/// Every head, and every operation a head follows, must be in `ops`.
pub fn merge<E>(
    ops: &Ops,
    heads: &BTreeSet<HashRef>,
    read: impl FnMut(&HashRef) -> Result<View, E>,
) -> Result<View, E> {
    let mut merger = Merger::new(ops, read);
    if let [head] = heads.iter().collect::<Vec<_>>()[..] {
        return merger.view_of(head).cloned();
    }
    let mut view = View::default();
    for (name, writers) in writers(ops, heads) {
        if let Some(entry) = merger.entry(&name, &writers)? {
            view.workspaces.insert(name, entry);
        }
    }
    Ok(view)
}

/// Each workspace's writers among `heads` and the operations they follow:
/// the operations made there that no other operation made there follows.
///
/// One pass over those operations, children before parents: an operation is
/// a writer of its workspace unless an operation made in the same workspace
/// lies below it, among the operations that follow it.
pub fn writers(ops: &Ops, heads: &BTreeSet<HashRef>) -> BTreeMap<String, BTreeSet<HashRef>> {
    // Every operation the heads follow, with its children among them.
    let mut seen: BTreeSet<HashRef> = BTreeSet::new();
    let mut children: BTreeMap<HashRef, Vec<HashRef>> = BTreeMap::new();
    let mut stack: Vec<HashRef> = heads.iter().copied().collect();
    while let Some(id) = stack.pop() {
        if !seen.insert(id) {
            continue;
        }
        for parent in &ops[&id].parents {
            children.entry(*parent).or_default().push(id);
            stack.push(*parent);
        }
    }
    // Children before parents (Kahn's order over the reversed edges).
    let mut waiting: BTreeMap<HashRef, usize> = seen
        .iter()
        .map(|id| (*id, children.get(id).map_or(0, Vec::len)))
        .collect();
    let mut ready: Vec<HashRef> = waiting
        .iter()
        .filter(|(_, count)| **count == 0)
        .map(|(id, _)| *id)
        .collect();
    // The workspaces that have an operation below each operation.
    let mut below: BTreeMap<HashRef, BTreeSet<String>> = BTreeMap::new();
    let mut found: BTreeMap<String, BTreeSet<HashRef>> = BTreeMap::new();
    while let Some(id) = ready.pop() {
        let op = &ops[&id];
        let mut names = BTreeSet::new();
        for child in children.get(&id).into_iter().flatten() {
            names.extend(below[child].iter().cloned());
            names.insert(ops[child].workspace.clone());
        }
        if !names.contains(&op.workspace) {
            found.entry(op.workspace.clone()).or_default().insert(id);
        }
        below.insert(id, names);
        for parent in &op.parents {
            let count = waiting.get_mut(parent).expect("a parent the walk saw");
            *count -= 1;
            if *count == 0 {
                ready.push(*parent);
            }
        }
    }
    found
}

/// Merges entries against `base`. A member, like the base, may be absent.
///
/// - The working copy is the base's when no member changed it, and
///   otherwise the union of the changed members' sets (the support-set
///   normal form).
/// - A changeset is among the heads when it is in the base and every member
///   kept it, or is not in the base and some member has it.
///
/// The result is absent when its working copy is empty.
pub fn merge_entries(
    base: Option<&WorkspaceEntry>,
    members: &[Option<&WorkspaceEntry>],
) -> Option<WorkspaceEntry> {
    let absent = WorkspaceEntry::default();
    let base = base.unwrap_or(&absent);
    let members: Vec<&WorkspaceEntry> = members
        .iter()
        .map(|member| member.unwrap_or(&absent))
        .collect();

    let changed: Vec<&BTreeSet<HashRef>> = members
        .iter()
        .map(|member| &member.working_copy)
        .filter(|working_copy| **working_copy != base.working_copy)
        .collect();
    let working_copy: BTreeSet<HashRef> = if changed.is_empty() {
        base.working_copy.clone()
    } else {
        changed.into_iter().flatten().copied().collect()
    };

    let candidates: BTreeSet<&HashRef> = base
        .heads
        .iter()
        .chain(members.iter().flat_map(|member| &member.heads))
        .collect();
    let heads: BTreeSet<HashRef> = candidates
        .into_iter()
        .filter(|id| {
            if base.heads.contains(id) {
                members.iter().all(|member| member.heads.contains(id))
            } else {
                members.iter().any(|member| member.heads.contains(id))
            }
        })
        .copied()
        .collect();

    if working_copy.is_empty() {
        return None;
    }
    Some(WorkspaceEntry {
        working_copy,
        heads,
    })
}

/// The latest operations made in `name` that every one of `writers`
/// follows: the common ones that no other common one follows. Empty when
/// they have none in common.
pub fn bases(ops: &Ops, name: &str, writers: &BTreeSet<HashRef>) -> BTreeSet<HashRef> {
    Ancestry::new(ops).bases(name, writers)
}

/// The ancestors of operations, each found once.
struct Ancestry<'a> {
    ops: &'a Ops,
    found: BTreeMap<HashRef, BTreeSet<HashRef>>,
}

impl<'a> Ancestry<'a> {
    fn new(ops: &'a Ops) -> Self {
        Ancestry {
            ops,
            found: BTreeMap::new(),
        }
    }

    /// Every operation `op` follows, itself excluded.
    fn of(&mut self, op: &HashRef) -> &BTreeSet<HashRef> {
        let ops = self.ops;
        self.found.entry(*op).or_insert_with(|| {
            let mut seen = BTreeSet::new();
            let mut stack: Vec<HashRef> = ops[op].parents.iter().copied().collect();
            while let Some(id) = stack.pop() {
                if seen.insert(id) {
                    stack.extend(ops[&id].parents.iter().copied());
                }
            }
            seen
        })
    }

    /// See [`bases`].
    fn bases(&mut self, name: &str, writers: &BTreeSet<HashRef>) -> BTreeSet<HashRef> {
        let ops = self.ops;
        let mut common: Option<BTreeSet<HashRef>> = None;
        for writer in writers {
            let made_there = self
                .of(writer)
                .iter()
                .filter(|id| ops[id].workspace == name)
                .copied();
            common = Some(match common {
                None => made_there.collect(),
                Some(common) => made_there.filter(|id| common.contains(id)).collect(),
            });
        }
        let common = common.unwrap_or_default();
        let mut covered = BTreeSet::new();
        for id in &common {
            let below = self.of(id);
            covered.extend(below.intersection(&common).copied());
        }
        common.difference(&covered).copied().collect()
    }
}

/// One merge in progress: the views read so far, and the ancestors found.
struct Merger<'a, F> {
    read: F,
    ops: &'a Ops,
    views: BTreeMap<HashRef, View>,
    ancestry: Ancestry<'a>,
}

impl<'a, E, F: FnMut(&HashRef) -> Result<View, E>> Merger<'a, F> {
    fn new(ops: &'a Ops, read: F) -> Self {
        Merger {
            read,
            ops,
            views: BTreeMap::new(),
            ancestry: Ancestry::new(ops),
        }
    }

    /// The view operation `op` leaves.
    fn view_of(&mut self, op: &HashRef) -> Result<&View, E> {
        if !self.views.contains_key(op) {
            let view = (self.read)(&self.ops[op].view)?;
            self.views.insert(*op, view);
        }
        Ok(&self.views[op])
    }

    /// Workspace `name`'s entry as `writers`, all made in it, leave it.
    fn entry(
        &mut self,
        name: &str,
        writers: &BTreeSet<HashRef>,
    ) -> Result<Option<WorkspaceEntry>, E> {
        if let [writer] = writers.iter().collect::<Vec<_>>()[..] {
            return Ok(self.view_of(writer)?.workspaces.get(name).cloned());
        }
        if writers.is_empty() {
            return Ok(None);
        }
        let bases = self.ancestry.bases(name, writers);
        let base = self.entry(name, &bases)?;
        let mut members = Vec::with_capacity(writers.len());
        for writer in writers {
            members.push(self.view_of(writer)?.workspaces.get(name).cloned());
        }
        let members: Vec<Option<&WorkspaceEntry>> = members.iter().map(Option::as_ref).collect();
        Ok(merge_entries(base.as_ref(), &members))
    }
}
