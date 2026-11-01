//! The attributes a workspace's own `.gitattributes` files give its paths, as
//! Git itself evaluates them. Git runs at the workspace, against a private,
//! empty repository whose work tree is the workspace, without the caller's Git
//! settings or this machine's global and system attributes, and with pattern
//! case and long-path handling pinned. Each evaluation makes its repository in
//! a directory it has just created and removes it afterwards, so no other run
//! or process shares it and nothing left over from one can add attributes.
//! The answer therefore depends only on the workspace's files: not on whether
//! it is a checkout, on an enclosing repository, on who owns it, on the
//! current directory or on this machine.

use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

/// One evaluated attribute: `(path, attribute, value)`, where the value is
/// `set`, `unset`, `unspecified` or a string such as `auto`.
pub type Value = (String, String, String);

/// The values the workspace at `root` gives each of `attributes` for each of
/// `paths` (relative to `root`). A warning from Git, such as a nested
/// `.gitattributes` it could not read, is an error.
pub fn values(root: &Path, attributes: &[&str], paths: &[String]) -> Result<Vec<Value>, String> {
    let repository = PrivateRepository::new()?;
    let output = run(&mut check_attr(&repository.0, root, attributes, paths))?;
    let warnings = String::from_utf8_lossy(&output.stderr);
    if !warnings.trim().is_empty() {
        return Err(format!("git warned: {}", warnings.trim()));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .map(|line| {
            let (path, rest) = line.split_once(": ")?;
            let (attribute, value) = rest.split_once(": ")?;
            Some((path.to_owned(), attribute.to_owned(), value.to_owned()))
        })
        .collect::<Option<Vec<Value>>>()
        .ok_or_else(|| format!("`git check-attr` printed {:?}", stdout.trim()))
}

/// The `git check-attr` command, run at `root` so that `paths` are read
/// relative to it whatever this process's current directory is.
fn check_attr(git_dir: &Path, root: &Path, attributes: &[&str], paths: &[String]) -> Command {
    let mut command = isolated(git_dir, env::vars_os().map(|(name, _)| name));
    command
        .current_dir(root)
        .arg("--git-dir")
        .arg(git_dir)
        .arg("--work-tree")
        .arg(root)
        // Patterns match case-sensitively, as on Linux, whatever filesystem
        // holds the private repository; nested files beyond Windows' MAX_PATH
        // are read rather than skipped.
        .args(["-c", "core.ignorecase=false", "-c", "core.longpaths=true"])
        .arg("check-attr")
        .args(attributes)
        .arg("--")
        .args(paths);
    command
}

/// A private, empty bare repository for one evaluation, in a new directory
/// under the OS temp directory. Dropping it removes the directory.
struct PrivateRepository(PathBuf);

impl PrivateRepository {
    /// Creates a directory that did not exist and has Git make the repository
    /// in it. Without a template Git makes no `info/` and no hooks.
    fn new() -> Result<Self, String> {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let temp = env::temp_dir();
        let names = (0..100).map(|_| {
            temp.join(format!(
                "xtask-attributes-{}-{}.git",
                process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ))
        });
        // The guard removes the directory even if Git fails.
        let repository = PrivateRepository(claim(names)?);
        run(
            isolated(&repository.0, env::vars_os().map(|(name, _)| name))
                .args(["init", "-q", "--bare", "--template="])
                .arg(&repository.0),
        )?;
        Ok(repository)
    }
}

/// The first of `names` that this call creates as a new directory. A name
/// something already has, perhaps left by an earlier process with this id,
/// is skipped and never used.
fn claim(names: impl IntoIterator<Item = PathBuf>) -> Result<PathBuf, String> {
    let mut taken = None;
    for name in names {
        match fs::create_dir(&name) {
            Ok(()) => return Ok(name),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => taken = Some(name),
            Err(error) => return Err(format!("could not create {}: {error}", name.display())),
        }
    }
    Err(match taken {
        Some(name) => format!(
            "could not create a new directory: {} and every name before it exist",
            name.display()
        ),
        None => "could not create a new directory: no name to try".to_owned(),
    })
}

impl Drop for PrivateRepository {
    fn drop(&mut self) {
        // Windows can keep the directory in use for a moment after Git has
        // run in it, so removal backs off for up to about 2 s. The first try
        // normally succeeds, and a directory left behind is never used again.
        for attempt in 0..25 {
            if fs::remove_dir_all(&self.0).is_ok() || !self.0.exists() {
                return;
            }
            thread::sleep(Duration::from_millis(10 * attempt.min(10)));
        }
    }
}

/// `git` without the caller's Git settings, given the names of the variables
/// it would inherit: none of the `GIT_*` ones, no global or system
/// configuration or attributes, and messages in one language.
fn isolated(git_dir: &Path, inherited: impl IntoIterator<Item = OsString>) -> Command {
    let mut command = Command::new("git");
    for name in inherited {
        if is_git_variable(&name) {
            command.env_remove(name);
        }
    }
    command
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_ATTR_NOSYSTEM", "1")
        // A new private repository holds neither file: no global
        // configuration, and no global attributes (`git/attributes` under
        // `XDG_CONFIG_HOME`).
        .env("GIT_CONFIG_GLOBAL", git_dir.join("no-global-config"))
        .env("XDG_CONFIG_HOME", git_dir)
        .env("LC_ALL", "C");
    command
}

/// Whether Git reads the variable `name`, in any letter case.
fn is_git_variable(name: &OsStr) -> bool {
    name.to_str()
        .is_some_and(|name| name.to_ascii_uppercase().starts_with("GIT_"))
}

/// Runs `command`, or says why it failed, quoting Git.
fn run(command: &mut Command) -> Result<Output, String> {
    let output = command
        .output()
        .map_err(|error| format!("could not run git: {error}"))?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(format!(
            "git exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::ffi::{OsStr, OsString};
    use std::fs;
    use std::path::Path;
    use std::process::{self, Command};

    use super::{PrivateRepository, check_attr, claim, is_git_variable, isolated, values};

    /// Whether `envs` sets `name` to `value`. Windows compares variable names
    /// without regard to case, and `Command` keeps an inherited name's case
    /// when a later call sets it.
    fn sets(envs: &[(&OsStr, Option<&OsStr>)], name: &str, value: &OsStr) -> bool {
        envs.iter().any(|(key, set)| {
            let same = if cfg!(windows) {
                key.eq_ignore_ascii_case(name)
            } else {
                *key == OsStr::new(name)
            };
            same && *set == Some(value)
        })
    }

    /// The variables `isolated` sets for a private repository at `git_dir`.
    fn pinned_variables(git_dir: &Path) -> [(&'static str, OsString); 5] {
        [
            ("GIT_CONFIG_NOSYSTEM", "1".into()),
            ("GIT_ATTR_NOSYSTEM", "1".into()),
            // Paths into the repository, where no such file ever exists.
            ("GIT_CONFIG_GLOBAL", git_dir.join("no-global-config").into()),
            ("XDG_CONFIG_HOME", git_dir.into()),
            ("LC_ALL", "C".into()),
        ]
    }

    #[test]
    fn check_attr_runs_at_the_workspace_and_isolated() {
        let git_dir = Path::new("private.git");
        let root = Path::new("workspace-root");
        let command = check_attr(git_dir, root, &["text"], &["a.bin".to_owned()]);
        assert_eq!(
            command.get_current_dir(),
            Some(root),
            "paths are read from the root"
        );
        let args: Vec<&OsStr> = command.get_args().collect();
        for pinned in ["core.ignorecase=false", "core.longpaths=true"] {
            assert!(
                args.contains(&OsStr::new(pinned)),
                "{pinned} must be pinned"
            );
        }
        let envs: Vec<(&OsStr, Option<&OsStr>)> = command.get_envs().collect();
        for (name, value) in pinned_variables(git_dir) {
            assert!(sets(&envs, name, &value), "{name} must be {value:?}");
        }
    }

    /// Every inherited Git variable, in any letter case, is removed unless
    /// `isolated` sets it, and every other variable is left as inherited.
    #[test]
    fn isolated_removes_every_inherited_git_variable() {
        let git_dir = Path::new("private.git");
        // No two names are equal ignoring case, as Windows compares them.
        let removed = [
            "GIT_CONFIG_PARAMETERS",
            "GIT_CONFIG_COUNT",
            "GIT_CONFIG_KEY_0",
            "GIT_COMMON_DIR",
            "GIT_ATTR_SOURCE",
            "GIT_DIR",
            "git_work_tree",
            "Git_Namespace",
        ];
        let set = [
            "GIT_CONFIG_NOSYSTEM",
            "GIT_ATTR_NOSYSTEM",
            "GIT_CONFIG_GLOBAL",
        ];
        let kept = ["PATH", "HOME"];
        let inherited = removed.into_iter().chain(set).chain(kept);
        let command = isolated(git_dir, inherited.map(OsString::from));
        let envs: Vec<(&OsStr, Option<&OsStr>)> = command.get_envs().collect();
        for name in removed {
            assert!(
                envs.contains(&(OsStr::new(name), None)),
                "{name} must be removed: {envs:?}"
            );
        }
        // Removing an inherited variable never undoes a value `isolated` sets.
        for (name, value) in pinned_variables(git_dir) {
            assert!(
                sets(&envs, name, &value),
                "{name} must be {value:?}: {envs:?}"
            );
        }
        for name in kept {
            assert!(
                !envs.iter().any(|(key, _)| key.eq_ignore_ascii_case(name)),
                "{name} must be left as inherited: {envs:?}"
            );
        }
    }

    /// Each evaluation's repository is its own: a directory that did not
    /// exist, holding what `git init --bare --template=` makes and nothing
    /// that adds attributes, and gone once its guard is dropped.
    #[test]
    fn every_private_repository_is_new_and_removed_after_use() {
        let first = PrivateRepository::new().unwrap();
        let second = PrivateRepository::new().unwrap();
        assert_ne!(first.0, second.0, "two repositories share a directory");
        for git_dir in [&first.0, &second.0] {
            let shown = git_dir.display();
            assert!(git_dir.join("HEAD").is_file(), "{shown} lacks HEAD");
            for dir in ["objects", "refs"] {
                assert!(git_dir.join(dir).is_dir(), "{shown} lacks {dir}/");
            }
            for dir in ["info", "hooks"] {
                assert!(!git_dir.join(dir).exists(), "{shown} has {dir}/");
            }
        }
        let (first_dir, second_dir) = (first.0.clone(), second.0.clone());
        drop(first);
        assert!(
            !first_dir.exists(),
            "{} is left behind",
            first_dir.display()
        );
        assert!(
            second_dir.is_dir(),
            "a guard removes only its own repository"
        );
        drop(second);
        assert!(
            !second_dir.exists(),
            "{} is left behind",
            second_dir.display()
        );
    }

    #[test]
    fn every_git_variable_is_scrubbed() {
        for name in [
            "GIT_DIR",
            "GIT_ATTR_SOURCE",
            "Git_Config_Parameters",
            "git_index_file",
        ] {
            assert!(is_git_variable(OsStr::new(name)), "{name}");
        }
        for name in ["PATH", "LC_ALL", "XDG_CONFIG_HOME", "DIGIT_X"] {
            assert!(!is_git_variable(OsStr::new(name)), "{name}");
        }
    }

    #[test]
    fn a_missing_workspace_is_an_error() {
        let missing = Path::new(env!("CARGO_MANIFEST_DIR")).join("no-such-workspace");
        assert!(values(&missing, &["text"], &["a.bin".to_owned()]).is_err());
    }

    /// A name something already has is skipped, and what is there stays as it
    /// was, even a `git/attributes` that Git would read as global attributes.
    #[test]
    fn taken_names_are_never_used() {
        let base = env::temp_dir().join(format!("xtask-attributes-{}-claim", process::id()));
        let _ = fs::remove_dir_all(&base);
        let [empty, planted, free] = ["empty", "planted", "free"].map(|name| base.join(name));
        fs::create_dir_all(&empty).unwrap();
        fs::create_dir_all(planted.join("git")).unwrap();
        fs::write(planted.join("git/attributes"), "*.bin binary\n").unwrap();

        let claimed = claim([empty.clone(), planted.clone(), free.clone()]);
        let all_taken = claim([empty.clone(), planted.clone()]);
        let empty_entries = fs::read_dir(&empty).unwrap().count();
        let planted_entries = fs::read_dir(&planted).unwrap().count();
        let _ = fs::remove_dir_all(&base);

        assert_eq!(claimed, Ok(free));
        assert!(all_taken.is_err(), "{all_taken:?}");
        assert_eq!((empty_entries, planted_entries), (0, 1));
    }

    /// An empty `.gitattributes` leaves a file unmarked. The suite runs this
    /// as is; `caller_git_settings_are_shut_out` also runs it alone, in a
    /// child process whose environment would mark the file if Git saw it.
    #[test]
    fn empty_attributes_leave_files_unmarked() {
        let root = env::temp_dir().join(format!("xtask-attributes-{}-unmarked", process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join(".gitattributes"), "").unwrap();
        let found = values(&root, &["text"], &["a.bin".to_owned()]);
        let _ = fs::remove_dir_all(&root);
        assert_eq!(
            found,
            Ok(vec![("a.bin".into(), "text".into(), "unspecified".into())])
        );
    }

    /// The caller's Git variables never reach the rule's Git. Both forms Git
    /// reads point `core.attributesFile` at a file that marks `*.bin` binary:
    /// a plain `git` given them reports the file binary, while the rule,
    /// run in a child process with the same variables, finds it unmarked.
    #[test]
    fn caller_git_settings_are_shut_out() {
        let dir = env::temp_dir().join(format!("xtask-attributes-{}-hostile", process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("attributes");
        fs::write(&file, "*.bin binary\n").unwrap();
        // Git reads forward slashes on every platform.
        let path = file.to_string_lossy().replace('\\', "/");
        let hostile = [
            (
                "GIT_CONFIG_PARAMETERS",
                format!("'core.attributesfile'='{path}'"),
            ),
            ("GIT_CONFIG_COUNT", "1".to_owned()),
            ("GIT_CONFIG_KEY_0", "core.attributesFile".to_owned()),
            ("GIT_CONFIG_VALUE_0", path),
        ];

        let repository = PrivateRepository::new().unwrap();
        let control = Command::new("git")
            .current_dir(&dir)
            .arg("--git-dir")
            .arg(&repository.0)
            .arg("--work-tree")
            .arg(&dir)
            .args(["check-attr", "text", "--", "a.bin"])
            .envs(hostile.clone())
            .output()
            .unwrap();
        let child = Command::new(env::current_exe().unwrap())
            .args([
                "--exact",
                "attributes::tests::empty_attributes_leave_files_unmarked",
            ])
            .envs(hostile)
            .output()
            .unwrap();
        drop(repository);
        let _ = fs::remove_dir_all(&dir);

        let control = String::from_utf8_lossy(&control.stdout);
        assert_eq!(control.trim(), "a.bin: text: unset", "the control");
        let stdout = String::from_utf8_lossy(&child.stdout);
        assert!(
            child.status.success() && stdout.contains("test result: ok. 1 passed"),
            "{stdout}{}",
            String::from_utf8_lossy(&child.stderr)
        );
    }
}
