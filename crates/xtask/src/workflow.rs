//! A structural reading of `.github/workflows/ci.yml` against the T-007
//! workflow clause, without a YAML dependency. It reads the subset workflows
//! use: lines nest by indentation, `key: value` lines are mapping entries,
//! `- ` lines are sequence items, and `[a, b]` is an inline sequence.

/// A trimmed line and the lines nested under it.
struct Node<'a> {
    text: &'a str,
    children: Vec<Node<'a>>,
}

/// What the workflow lacks against the T-007 clause, as Sprint 5 T-004
/// revised it; empty when it meets every point.
pub fn problems(workflow: &str) -> Vec<String> {
    let top = parse(workflow);
    let mut problems = Vec::new();
    // CI runs on pushes to `main` only: `dev` is built through its pull
    // request (INT-0003 AC-2).
    match under(&top, &["on", "push"]) {
        None => problems.push("push does not trigger on main".to_owned()),
        Some(push) => match field(&push.children, "branches") {
            None => problems.push("push has no branch list".to_owned()),
            Some(branches) => {
                let branches = list(branches);
                if !branches.contains(&"main") {
                    problems.push("push does not trigger on main".to_owned());
                }
                // GitHub applies branch patterns in order, so a later `!`
                // pattern can exclude a branch an earlier one names.
                if branches.iter().any(|branch| branch.starts_with('!')) {
                    problems.push("push excludes branches with a `!` pattern".to_owned());
                }
                for branch in branches.iter().filter(|branch| **branch != "main") {
                    problems.push(format!(
                        "push triggers on another branch or a pattern: {branch}"
                    ));
                }
            }
        },
    }
    if under(&top, &["on", "pull_request"]).is_none() {
        problems.push("there is no pull_request trigger".to_owned());
    }
    if under(&top, &["on", "workflow_dispatch"]).is_none() {
        problems.push("there is no workflow_dispatch trigger".to_owned());
    }
    // A filter can keep a trigger from ever firing, so none is allowed; a
    // `pull_request` branch list must name both branches.
    for (event, filters) in [
        ("push", &["paths", "paths-ignore", "branches-ignore"][..]),
        (
            "pull_request",
            &["paths", "paths-ignore", "branches-ignore", "types"],
        ),
    ] {
        let Some(trigger) = under(&top, &["on", event]) else {
            continue;
        };
        if event == "pull_request" && !value_of(trigger).is_empty() {
            problems.push("pull_request is configured inline".to_owned());
        }
        for filter in filters {
            if field(&trigger.children, filter).is_some() {
                problems.push(format!("{event} is filtered by `{filter}`"));
            }
        }
    }
    if let Some(branches) = under(&top, &["on", "pull_request", "branches"]) {
        let branches = list(branches);
        for branch in ["dev", "main"] {
            if !branches.contains(&branch) {
                problems.push(format!("pull_request does not trigger on {branch}"));
            }
        }
        if branches.iter().any(|branch| branch.starts_with('!')) {
            problems.push("pull_request excludes branches with a `!` pattern".to_owned());
        }
    }
    problems.extend(concurrency_problems(&top));
    // A job's own concurrency would cancel or queue jobs apart from the
    // workflow's rule.
    let jobs_list = under(&top, &["jobs"]).map_or(&[][..], |jobs| jobs.children.as_slice());
    if jobs_list
        .iter()
        .any(|job| field(&job.children, "concurrency").is_some())
    {
        problems.push("a job sets its own concurrency".to_owned());
    }
    if field(&top, "defaults").is_some() {
        problems.push("the workflow sets step defaults".to_owned());
    }
    let jobs = under(&top, &["jobs"]).map_or(&[][..], |jobs| jobs.children.as_slice());
    if !jobs
        .iter()
        .any(|job| runs_on_both_platforms(job, &[CARGO_CI]))
    {
        problems.push(
            "no job runs `cargo ci` on a matrix of ubuntu-latest and windows-latest".to_owned(),
        );
    }
    let lint = |job: &Node| {
        runs_on_both_platforms(job, &[FMT, CLIPPY])
            && !steps(job).any(|(_, entries)| entries.contains(&("run", CARGO_CI)))
    };
    if !jobs.iter().any(lint) {
        problems.push(
            "no lint job of its own runs fmt and Clippy on a matrix of ubuntu-latest and \
             windows-latest"
                .to_owned(),
        );
    }
    if !field(jobs, "deny").is_some_and(runs_deny_on_ubuntu) {
        problems.push("no deny job runs cargo-deny on ubuntu-latest".to_owned());
    }
    if continues_on_error(workflow) {
        problems.push("a job or step continues on error".to_owned());
    }
    if handles_failure_in_shell(workflow) {
        problems.push("a command handles its own failure with `||` or `set +e`".to_owned());
    }
    problems
}

/// The suite, and the two lint commands (INT-0003 AC-4), as the jobs run them.
const CARGO_CI: &str = "cargo ci";
const FMT: &str = "cargo fmt --all --check";
const CLIPPY: &str = "cargo clippy --workspace --all-targets --locked -- -D warnings";

/// The only concurrency group accepted: one group per pull request, and one
/// per commit for every other event, so no push to `main` shares a group.
const GROUP: &str = "ci-${{ github.ref }}-${{ github.event_name == 'pull_request' || github.sha }}";

/// The only `cancel-in-progress` value accepted: a newer push to a pull
/// request cancels the run it replaces, and nothing else is cancelled.
const CANCEL_PULL_REQUESTS: &str = "${{ github.event_name == 'pull_request' }}";

/// The top-level `concurrency` block: a group that keeps each pull request
/// apart (`github.ref`) and gives each push its own group (`github.sha`), and
/// cancelling for pull-request runs only (INT-0003 AC-2).
fn concurrency_problems(top: &[Node]) -> Vec<String> {
    let Some(concurrency) = field(top, "concurrency") else {
        return vec!["there is no top-level concurrency".to_owned()];
    };
    let mut problems = Vec::new();
    if field(&concurrency.children, "group").map(value_of) != Some(GROUP) {
        problems.push(format!("the concurrency group is not exactly {GROUP}"));
    }
    if field(&concurrency.children, "cancel-in-progress").map(value_of)
        != Some(CANCEL_PULL_REQUESTS)
    {
        problems.push(format!(
            "concurrency does not cancel exactly the superseded pull-request runs \
             (cancel-in-progress: {CANCEL_PULL_REQUESTS})"
        ));
    }
    problems
}

/// Whether any `continue-on-error` is set to something other than `false`.
fn continues_on_error(workflow: &str) -> bool {
    code_lines(workflow).any(|line| {
        let line = line.trim();
        let line = line.strip_prefix("- ").unwrap_or(line);
        entry(line).is_some_and(|(key, value)| key == "continue-on-error" && value != "false")
    })
}

/// Whether any command can outlive its own failure: a shell `||` (one inside a
/// `${{ }}` expression is not shell) or `set +e`. Matching every `||`, wherever
/// a line break puts it, also covers `|| exit 0`, `|| echo` and the like.
fn handles_failure_in_shell(workflow: &str) -> bool {
    code_lines(workflow).any(|line| {
        let shell = outside_expressions(line);
        let words: Vec<&str> = shell.split_whitespace().collect();
        shell.contains("||")
            || words
                .windows(2)
                .any(|pair| pair == ["set", "+e"] || pair == ["+o", "errexit"])
    })
}

/// `line` without its `${{ ... }}` expressions.
fn outside_expressions(line: &str) -> String {
    let mut shell = String::new();
    let mut rest = line;
    while let Some(start) = rest.find("${{") {
        shell.push_str(&rest[..start]);
        rest = rest[start..]
            .find("}}")
            .map_or("", |end| &rest[start + end + 2..]);
    }
    shell.push_str(rest);
    shell
}

/// Whether a step's entries leave it unconditional and running where and how
/// it says: no `if:`, `working-directory` or `shell`.
fn plain(entries: &[(&str, &str)]) -> bool {
    !entries
        .iter()
        .any(|&(key, _)| ["if", "working-directory", "shell"].contains(&key))
}

/// A job whose matrix holds both platforms and nothing that adds or removes a
/// leg, that runs on the matrix's platform, sets no step defaults, and whose
/// own plain steps run each of `commands`. The job carries no `if:`.
fn runs_on_both_platforms(job: &Node, commands: &[&str]) -> bool {
    let matrix = under(&job.children, &["strategy", "matrix"]);
    let os = matrix
        .and_then(|matrix| field(&matrix.children, "os"))
        .map(list)
        .unwrap_or_default();
    let fixed = matrix.is_some_and(|matrix| {
        field(&matrix.children, "exclude").is_none() && field(&matrix.children, "include").is_none()
    });
    os.contains(&"ubuntu-latest")
        && os.contains(&"windows-latest")
        && fixed
        && unconditional(&job.children)
        && field(&job.children, "defaults").is_none()
        && runs_on(job) == Some("${{ matrix.os }}")
        && commands.iter().all(|command| {
            steps(job).any(|(_, entries)| entries.contains(&("run", command)) && plain(&entries))
        })
}

/// A job on ubuntu-latest whose own steps install cargo-deny with
/// `taiki-e/install-action` (checksum verified, which may not be turned off,
/// and downloads retried) and then
/// run a plain `cargo deny check`, which runs every check (advisories, bans,
/// licences and sources) on this workspace. A check that narrows what it
/// checks does not count. Neither the job nor those steps may carry an `if:`,
/// and none may change where or how a step runs (`defaults`,
/// `working-directory`, `shell`). cargo-deny's own action is not accepted:
/// its unchecked download failed Sprint 4's checkpoint.
fn runs_deny_on_ubuntu(job: &Node) -> bool {
    let jobs_steps: Vec<_> = steps(job).collect();
    let install = jobs_steps.iter().position(|(step, entries)| {
        plain(entries)
            && entries.iter().any(|&(key, value)| {
                key == "uses"
                    && value
                        .to_ascii_lowercase()
                        .starts_with("taiki-e/install-action@")
            })
            && field(&step.children, "with").is_some_and(|with| {
                let tool = field(&with.children, "tool").map(value_of);
                let checksum = field(&with.children, "checksum").map(value_of);
                tool.is_some_and(|tool| tool == "cargo-deny" || tool.starts_with("cargo-deny@"))
                    && matches!(checksum, None | Some("true"))
            })
    });
    let check = jobs_steps.iter().position(|(_, entries)| {
        plain(entries)
            && entries
                .iter()
                .any(|&(key, value)| key == "run" && runs_deny_check(value))
    });
    runs_on(job) == Some("ubuntu-latest")
        && unconditional(&job.children)
        && field(&job.children, "defaults").is_none()
        && matches!((install, check), (Some(install), Some(check)) if install < check)
}

/// Flags that leave what `cargo deny check` checks, and where, unchanged.
const PLAIN_FLAGS: [&str; 2] = ["--all-features", "--locked"];

/// A plain `cargo deny check` command, with at most `PLAIN_FLAGS`: no shell
/// operator could replace or ignore its exit status, and no word names a check,
/// a path or another configuration.
fn runs_deny_check(command: &str) -> bool {
    let words: Vec<&str> = command.split_whitespace().collect();
    words.starts_with(&["cargo", "deny"])
        && words[2..].iter().filter(|&&word| word == "check").count() == 1
        && words[2..]
            .iter()
            .all(|word| *word == "check" || PLAIN_FLAGS.contains(word))
}

/// Whether the mapping `nodes` has no `if:` entry.
fn unconditional(nodes: &[Node]) -> bool {
    field(nodes, "if").is_none()
}

fn runs_on<'a>(job: &Node<'a>) -> Option<&'a str> {
    field(&job.children, "runs-on").map(value_of)
}

/// A job's own steps: each step's node and its `key: value` entries.
fn steps<'n, 'a>(
    job: &'n Node<'a>,
) -> impl Iterator<Item = (&'n Node<'a>, Vec<(&'a str, &'a str)>)> + 'n {
    field(&job.children, "steps")
        .into_iter()
        .flat_map(|steps| steps.children.iter())
        .filter_map(|step| Some((step, item(step)?)))
}

/// The entries of a sequence item: the one on its `- ` line and those
/// nested under it.
fn item<'a>(node: &Node<'a>) -> Option<Vec<(&'a str, &'a str)>> {
    let first = node.text.strip_prefix("- ")?;
    let nested = node.children.iter().filter_map(|child| entry(child.text));
    Some(entry(first).into_iter().chain(nested).collect())
}

/// The workflow's non-blank lines with their comments removed.
fn code_lines(workflow: &str) -> impl Iterator<Item = &str> {
    workflow
        .lines()
        .map(|line| line.split(" #").next().unwrap_or_default())
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
}

/// The workflow's lines as a forest, without blank lines and comments.
fn parse(workflow: &str) -> Vec<Node<'_>> {
    let lines: Vec<(usize, &str)> = code_lines(workflow)
        .map(|line| (line.len() - line.trim_start().len(), line.trim()))
        .collect();
    nest(&lines)
}

/// Nests lines by indentation: each line owns the lines after it that are
/// indented deeper, and a `key:` line also owns `- ` items at its own depth.
fn nest<'a>(lines: &[(usize, &'a str)]) -> Vec<Node<'a>> {
    let mut nodes = Vec::new();
    let mut rest = lines;
    while let Some((&(indent, text), after)) = rest.split_first() {
        let owned = |&(depth, line): &(usize, &str)| {
            depth > indent || (depth == indent && text.ends_with(':') && line.starts_with("- "))
        };
        let end = after
            .iter()
            .position(|line| !owned(line))
            .unwrap_or(after.len());
        nodes.push(Node {
            text,
            children: nest(&after[..end]),
        });
        rest = &after[end..];
    }
    nodes
}

/// The node at the mapping path `path` below `nodes`.
fn under<'n, 'a>(nodes: &'n [Node<'a>], path: &[&str]) -> Option<&'n Node<'a>> {
    let (key, rest) = path.split_first()?;
    let node = field(nodes, key)?;
    if rest.is_empty() {
        Some(node)
    } else {
        under(&node.children, rest)
    }
}

/// The mapping entry `key` among `nodes`.
fn field<'n, 'a>(nodes: &'n [Node<'a>], key: &str) -> Option<&'n Node<'a>> {
    nodes.iter().find(|node| {
        !node.text.starts_with("- ") && entry(node.text).is_some_and(|(found, _)| found == key)
    })
}

/// The value of a `key: value` node, empty for a bare `key:`.
fn value_of<'a>(node: &Node<'a>) -> &'a str {
    entry(node.text).map_or("", |(_, value)| value)
}

/// A sequence written inline, `[a, b]`, or as nested `- a` items.
fn list<'a>(node: &Node<'a>) -> Vec<&'a str> {
    match value_of(node)
        .strip_prefix('[')
        .and_then(|inner| inner.strip_suffix(']'))
    {
        Some(inner) => inner
            .split(',')
            .map(|item| unquote(item.trim()))
            .filter(|item| !item.is_empty())
            .collect(),
        None => node
            .children
            .iter()
            .filter_map(|child| child.text.strip_prefix("- "))
            .map(|item| unquote(item.trim()))
            .collect(),
    }
}

/// Splits `key: value` or `key:`, unquoting both.
fn entry(text: &str) -> Option<(&str, &str)> {
    let (key, value) = match text.split_once(": ") {
        Some(split) => split,
        None => (text.strip_suffix(':')?, ""),
    };
    Some((unquote(key.trim()), unquote(value.trim())))
}

fn unquote(text: &str) -> &str {
    ['"', '\'']
        .into_iter()
        .find_map(|quote| text.strip_prefix(quote)?.strip_suffix(quote))
        .unwrap_or(text)
}

#[test]
fn workflow_forms_read_alike() {
    // Block sequences, sequences at their key's depth, quoted scalars, trailing
    // comments, `||` inside an expression, `continue-on-error: false` and a
    // lowercase action name read like the forms ci.yml uses.
    let workflow = "\
'on':
  push:
    branches:
      - \"main\"
  pull_request:
  workflow_dispatch:
concurrency:
  group: \"ci-${{ github.ref }}-${{ github.event_name == 'pull_request' || github.sha }}\"
  cancel-in-progress: ${{ github.event_name == 'pull_request' }}
jobs:
  lint:
    strategy:
      matrix:
        os:
        - ubuntu-latest
        - windows-latest
    runs-on: ${{ matrix.os }}
    steps:
    - run: 'cargo fmt --all --check'
    - name: Clippy
      run: cargo clippy --workspace --all-targets --locked -- -D warnings
  ci:
    name: ${{ github.head_ref || 'suite' }}
    strategy:
      matrix:
        os: [\"ubuntu-latest\", windows-latest]
    runs-on: ${{ matrix.os }}  # each platform
    steps:
    - uses: actions/checkout@v4
    - name: Suite
      run: \"cargo ci\"
      continue-on-error: false
  deny:
    runs-on: \"ubuntu-latest\"
    steps:
      - uses: Taiki-E/install-action@v2
        with:
          tool: 'cargo-deny@0.20.2'
      - run: cargo deny --all-features check
";
    assert_eq!(problems(workflow), Vec::<String>::new());
    let develop = workflow.replace("- \"main\"", "- develop");
    assert_eq!(
        problems(&develop),
        [
            "push does not trigger on main",
            "push triggers on another branch or a pattern: develop"
        ]
    );
}
