use std::ffi::OsStr;
use std::io::IsTerminal;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const NAME: &str = "ocppd-tui";

pub fn interactive() -> bool {
    std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}

pub fn find(beside: Option<&Path>, path: Option<&OsStr>) -> Option<PathBuf> {
    let beside = beside.map(|dir| dir.join(NAME));
    let on_path = path
        .into_iter()
        .flat_map(std::env::split_paths)
        .map(|dir| dir.join(NAME));
    beside
        .into_iter()
        .chain(on_path)
        .find(|file| file.is_file())
}

pub fn open(tui: &Path, csms: &str, ocppd: &Path) -> std::io::Error {
    Command::new(tui)
        .args(["--csms", csms])
        .env("OCPPD", ocppd)
        .exec()
}

#[cfg(test)]
#[path = "tui.test.rs"]
mod tests;
