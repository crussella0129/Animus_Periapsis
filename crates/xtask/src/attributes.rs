//! The attributes a workspace's own `.gitattributes` files give its paths, as
//! Git itself evaluates them. Git runs at the workspace, against a private,
//! empty repository whose work tree is the workspace, without the caller's Git
//! settings or this machine's global and system attributes, and with pattern
//! case and long-path handling pinned. The answer therefore depends only on the
//! workspace's files: not on whether it is a checkout, on an enclosing
//! repository, on who owns it, on the current directory or on this machine.

use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, PoisonError};
use std::thread;
use std::time::Duration;

/// One evaluated attribute: `(path, attribute, value)`, where the value is
/// `set`, `unset`, `unspecified` or a string such as `auto`.
pub type Value = (String, String, String);

/// The values the workspace at `root` gives each of `attributes` for each of
/// `paths` (relative to `root`). A warning from Git, such as a nested
/// `.gitattributes` it could not read, is an error.
pub fn values(root: &Path, attributes: &[&str], paths: &[String]) -> Result<Vec<Value>, String> {
    let git_dir = private_repository()?;
    let output = run(&mut check_attr(&git_dir, root, attributes, paths))?;
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
    let mut command = isolated(git_dir);
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

/// The private, empty bare repository beside the running executable, so
/// inside cargo's target directory. One that is incomplete or could add
/// attributes is replaced; a new one is built aside and moved into place, so
/// no process sees it half made.
fn private_repository() -> Result<PathBuf, String> {
    static CREATE: Mutex<()> = Mutex::new(());
    static STAGED: AtomicUsize = AtomicUsize::new(0);
    let base = env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
        .unwrap_or_else(env::temp_dir);
    let git_dir = base.join("attributes.git");
    let _creating = CREATE.lock().unwrap_or_else(PoisonError::into_inner);
    if is_pristine(&git_dir) {
        return Ok(git_dir);
    }
    let staged = base.join(format!(
        "attributes.{}.{}.git",
        std::process::id(),
        STAGED.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&staged);
    run(isolated(&staged)
        .args(["init", "-q", "--bare", "--template="])
        .arg(&staged))?;
    if git_dir.exists() {
        let _ = fs::remove_dir_all(&git_dir);
    }
    // Windows can hold the new directory open for a moment after `git init`.
    let mut moved = Err(String::new());
    for attempt in 0..50 {
        moved = fs::rename(&staged, &git_dir).map_err(|error| error.to_string());
        if moved.is_ok() || is_pristine(&git_dir) {
            break;
        }
        thread::sleep(Duration::from_millis(10 * attempt.min(10)));
    }
    let _ = fs::remove_dir_all(&staged);
    match moved {
        Ok(()) => Ok(git_dir),
        // Another process put its own in place first.
        Err(_) if is_pristine(&git_dir) => Ok(git_dir),
        Err(error) => Err(format!("could not create {}: {error}", git_dir.display())),
    }
}

/// Whether `git_dir` is a complete repository as `git init --bare --template=`
/// makes it, with nothing that could add attributes: no `info/`, no `git/`
/// (it doubles as `XDG_CONFIG_HOME`) and no attributes file or include in its
/// config.
fn is_pristine(git_dir: &Path) -> bool {
    let config = fs::read_to_string(git_dir.join("config"))
        .unwrap_or_default()
        .to_ascii_lowercase();
    git_dir.join("HEAD").is_file()
        && git_dir.join("objects").is_dir()
        && git_dir.join("refs").is_dir()
        && !git_dir.join("info").exists()
        && !git_dir.join("git").exists()
        && !config.is_empty()
        && !config.contains("attributesfile")
        && !config.contains("include")
}

/// `git` without the caller's Git settings: no `GIT_*` variable, no global or
/// system configuration or attributes, and messages in one language.
fn isolated(git_dir: &Path) -> Command {
    let mut command = Command::new("git");
    for (name, _) in env::vars_os() {
        if is_git_variable(&name) {
            command.env_remove(name);
        }
    }
    command
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_ATTR_NOSYSTEM", "1")
        // Neither path exists: no global configuration and no global attributes.
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
    use std::ffi::OsStr;
    use std::path::Path;

    use super::{check_attr, is_git_variable, values};

    #[test]
    fn check_attr_runs_at_the_workspace_and_isolated() {
        let root = Path::new("workspace-root");
        let command = check_attr(
            Path::new("private.git"),
            root,
            &["text"],
            &["a.bin".to_owned()],
        );
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
        for (name, value) in [
            ("GIT_CONFIG_NOSYSTEM", "1"),
            ("GIT_ATTR_NOSYSTEM", "1"),
            ("LC_ALL", "C"),
        ] {
            assert!(
                envs.contains(&(OsStr::new(name), Some(OsStr::new(value)))),
                "{name}={value} must be set"
            );
        }
        for name in ["GIT_CONFIG_GLOBAL", "XDG_CONFIG_HOME"] {
            assert!(
                envs.iter()
                    .any(|(key, value)| *key == OsStr::new(name) && value.is_some()),
                "{name} must point into the private repository"
            );
        }
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
}
