use std::path::PathBuf;
use std::process::Command;

pub fn local_bin() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".local/bin")
}

pub fn augment_path(cmd: &mut Command) {
    let local = local_bin();
    let local_str = local.to_string_lossy();
    let path = std::env::var("PATH").unwrap_or_default();
    let augmented = if path.is_empty() {
        local_str.to_string()
    } else {
        format!("{local_str}:{path}")
    };
    cmd.env("PATH", augmented);
}

pub fn command(path: &PathBuf) -> Command {
    let mut cmd = Command::new(path);
    augment_path(&mut cmd);
    cmd
}
