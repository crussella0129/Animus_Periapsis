//! A structural reading of `.github/workflows/ci.yml` against the T-007
//! workflow clause, without a YAML dependency. It reads the subset workflows
//! use: lines nest by indentation, `key: value` lines are mapping entries,
//! `- ` lines are sequence items, and `[a, b]` is an inline sequence.

/// A trimmed line and the lines nested under it.
struct Node<'a> {
    text: &'a str,
    children: Vec<Node<'a>>,
}

/// What the workflow lacks against the T-007 clause; empty when it meets
/// every point.
pub fn problems(workflow: &str) -> Vec<String> {
    let top = parse(workflow);
    let mut problems = Vec::new();
    let branches = under(&top, &["on", "push", "branches"])
        .map(list)
        .unwrap_or_default();
    for branch in ["dev", "main"] {
        if !branches.contains(&branch) {
            problems.push(format!("push does not trigger on {branch}"));
        }
    }
    if under(&top, &["on", "pull_request"]).is_none() {
        problems.push("there is no pull_request trigger".to_owned());
    }
    let jobs = under(&top, &["jobs"]).map_or(&[][..], |jobs| jobs.children.as_slice());
    if !jobs.iter().any(runs_ci_on_both_platforms) {
        problems.push(
            "no job runs `cargo ci` on a matrix of ubuntu-latest and windows-latest".to_owned(),
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

/// A job whose matrix holds both platforms and nothing that adds or removes a
/// leg, that runs on the matrix's platform, and whose own steps run
/// `cargo ci`. Neither the job nor that step may carry an `if:`.
fn runs_ci_on_both_platforms(job: &Node) -> bool {
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
        && runs_on(job) == Some("${{ matrix.os }}")
        && steps(job).any(|(_, entries)| {
            entries.contains(&("run", "cargo ci")) && !entries.iter().any(|&(key, _)| key == "if")
        })
}

/// A job on ubuntu-latest whose own steps run the check: cargo-deny's action
/// (unless its `with: command` is something other than `check`), or a plain
/// `cargo deny ... check` command. A step that only installs cargo-deny does
/// not count. Neither the job nor that step may carry an `if:`.
fn runs_deny_on_ubuntu(job: &Node) -> bool {
    runs_on(job) == Some("ubuntu-latest")
        && unconditional(&job.children)
        && steps(job).any(|(step, entries)| {
            !entries.iter().any(|&(key, _)| key == "if")
                && entries.iter().any(|&(key, value)| match key {
                    "uses" => {
                        value
                            .to_ascii_lowercase()
                            .starts_with("embarkstudios/cargo-deny-action@")
                            && field(&step.children, "with")
                                .and_then(|with| field(&with.children, "command"))
                                .is_none_or(|command| value_of(command).starts_with("check"))
                    }
                    "run" => runs_deny_check(value),
                    _ => false,
                })
        })
}

/// A plain `cargo deny ... check ...` command: no shell operator could replace
/// or ignore its exit status.
fn runs_deny_check(command: &str) -> bool {
    command.starts_with("cargo deny")
        && !command.contains(['|', '&', ';', '<', '>', '$', '`'])
        && command.split_whitespace().any(|word| word == "check")
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
      - \"dev\"
      - main
  pull_request:
jobs:
  ci:
    name: ${{ github.head_ref || 'suite' }}
    strategy:
      matrix:
        os:
        - ubuntu-latest
        - windows-latest
    runs-on: ${{ matrix.os }}  # each platform
    steps:
    - uses: actions/checkout@v4
    - name: Suite
      run: \"cargo ci\"
      continue-on-error: false
  deny:
    runs-on: \"ubuntu-latest\"
    steps:
      - run: cargo deny --all-features check
      - uses: embarkstudios/cargo-deny-action@v2
        with:
          command: check bans
";
    assert_eq!(problems(workflow), Vec::<String>::new());
    let develop = workflow.replace("- \"dev\"", "- develop");
    assert_eq!(problems(&develop), ["push does not trigger on dev"]);
}
