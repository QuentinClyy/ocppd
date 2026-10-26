use std::fs::{self, OpenOptions};
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const START_WITHIN: Duration = Duration::from_secs(2);

pub fn default_socket(env: impl Fn(&str) -> Option<String>) -> PathBuf {
    let set = |key: &str| env(key).filter(|value| !value.is_empty());
    if let Some(socket) = set("OCPPD_SOCKET") {
        return socket.into();
    }
    if let Some(runtime) = set("XDG_RUNTIME_DIR") {
        return Path::new(&runtime).join("ocppd.sock");
    }
    Path::new(&set("HOME").unwrap_or_default()).join(".ocppd.sock")
}

pub fn log_file(env: impl Fn(&str) -> Option<String>) -> PathBuf {
    let set = |key: &str| env(key).filter(|value| !value.is_empty());
    let state = match set("XDG_STATE_HOME") {
        Some(state) => PathBuf::from(state),
        None => Path::new(&set("HOME").unwrap_or_default()).join(".local/state"),
    };
    state.join("ocppd/ocppd.log")
}

pub fn ensure(socket: &Path, csms: &str, ocppd: &Path, log: &Path) -> Result<(), String> {
    if UnixStream::connect(socket).is_ok() {
        return Ok(());
    }
    let unwritable = |err: std::io::Error| format!("cannot write {}: {err}", log.display());
    if let Some(dir) = log.parent() {
        fs::create_dir_all(dir).map_err(unwritable)?;
    }
    let errors = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log)
        .map_err(unwritable)?;
    Command::new(ocppd)
        .args(["--csms", csms, "--listen"])
        .arg(format!("unix:{}", socket.display()))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(errors)
        .process_group(0)
        .spawn()
        .map_err(|err| format!("cannot start the daemon: {err}"))?;
    let deadline = Instant::now() + START_WITHIN;
    while Instant::now() < deadline {
        if UnixStream::connect(socket).is_ok() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    Err(format!("the daemon did not start; see {}", log.display()))
}

#[cfg(test)]
#[path = "attach.test.rs"]
mod tests;
