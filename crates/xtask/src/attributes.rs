//! The attributes a workspace's own `.gitattributes` files give a path, as Git
//! itself evaluates them. Git runs against a private, empty repository whose
//! work tree is the workspace, without the caller's Git settings or this
//! machine's global and system attributes. The answer therefore depends only on
//! the workspace's files: not on whether it is a checkout, on an enclosing
//! repository, on who owns it, or on this machine.

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{Mutex, PoisonError};

/// The value the workspace at `root` gives `attribute` for `path` (relative to
/// `root`): `set`, `unset`, `unspecified` or a string such as `auto`.
pub fn value(root: &Path, attribute: &str, path: &str) -> Result<String, String> {
    let git_dir = private_repository()?;
    let output = run(isolated(&git_dir)
        .arg("--git-dir")
        .arg(&git_dir)
        .arg("--work-tree")
        .arg(root)
        .args(["check-attr", attribute, "--", path]))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .trim()
        .strip_prefix(&format!("{path}: {attribute}: "))
        .map(str::to_owned)
        .ok_or_else(|| format!("`git check-attr` printed {:?}", stdout.trim()))
}

/// The private, empty bare repository: created once beside the running
/// executable, so inside cargo's target directory.
fn private_repository() -> Result<PathBuf, String> {
    static CREATE: Mutex<()> = Mutex::new(());
    let base = env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
        .unwrap_or_else(env::temp_dir);
    let git_dir = base.join("attributes.git");
    let _creating = CREATE.lock().unwrap_or_else(PoisonError::into_inner);
    if !git_dir.join("HEAD").is_file() {
        run(isolated(&git_dir)
            .args(["init", "-q", "--bare", "--template="])
            .arg(&git_dir))?;
    }
    Ok(git_dir)
}

/// `git` without the caller's Git settings: no `GIT_*` variable, no global or
/// system configuration or attributes, and messages in one language.
fn isolated(git_dir: &Path) -> Command {
    let mut command = Command::new("git");
    for (name, _) in env::vars_os() {
        let is_git = name
            .to_str()
            .is_some_and(|name| name.to_ascii_uppercase().starts_with("GIT_"));
        if is_git {
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
