use anyhow::bail;

#[cfg(any(target_os = "macos", target_os = "linux"))]
use anyhow::Context;
#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::{
    io::Write,
    process::{Command, Stdio},
};

pub fn copy_to_clipboard(value: &str) -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    {
        copy_with("pbcopy", &[], value)
    }

    #[cfg(target_os = "linux")]
    {
        if which::which("wl-copy").is_ok() {
            return copy_with("wl-copy", &[], value);
        }
        if which::which("xclip").is_ok() {
            return copy_with("xclip", &["-selection", "clipboard"], value);
        }
        bail!("no clipboard backend found; install wl-copy or xclip on Linux")
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = value;
        bail!("clipboard is not supported on this platform")
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn copy_with(program: &str, args: &[&str], value: &str) -> anyhow::Result<()> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to start clipboard backend '{program}'"))?;
    child
        .stdin
        .as_mut()
        .context("clipboard backend stdin was unavailable")?
        .write_all(value.as_bytes())?;
    let status = child.wait()?;
    if !status.success() {
        bail!("clipboard backend '{program}' exited with {status}");
    }
    Ok(())
}
