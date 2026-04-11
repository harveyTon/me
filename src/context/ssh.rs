use crate::model::SshContext;
#[cfg(unix)]
use std::process::Command;

pub fn detect() -> Option<SshContext> {
    detect_with(ancestor_commands, |key| std::env::var(key).ok())
}

fn detect_with(
    ancestors: impl Fn() -> Option<Vec<String>>,
    env: impl Fn(&str) -> Option<String>,
) -> Option<SshContext> {
    if let Some(value) = env("SSH_CONNECTION")
        .or_else(|| env("SSH_CLIENT"))
        .or_else(|| env("SSH_TTY"))
    {
        return Some(SshContext {
            remote: true,
            connection: Some(value),
        });
    }

    let sudo_context = env("SUDO_USER").is_some() || env("SUDO_UID").is_some();
    if sudo_context
        && ancestors()
            .unwrap_or_default()
            .iter()
            .any(|name| is_sshd_command(name))
    {
        return Some(SshContext {
            remote: true,
            connection: None,
        });
    }

    None
}

fn is_sshd_command(name: &str) -> bool {
    name.split_whitespace()
        .next()
        .and_then(|command| command.rsplit('/').next())
        .is_some_and(|base| {
            base == "sshd"
                || base
                    .strip_prefix("sshd")
                    .is_some_and(|rest| rest.starts_with([' ', ':', '-']))
        })
}

fn ancestor_commands() -> Option<Vec<String>> {
    #[cfg(not(unix))]
    {
        return None;
    }

    #[cfg(unix)]
    {
        let mut commands = Vec::new();
        let mut pid = unsafe { libc::getppid() };

        for _ in 0..8 {
            if pid <= 1 {
                break;
            }
            if let Some(command) = process_command(pid) {
                commands.push(command);
            }
            pid = parent_pid(pid)?;
        }

        Some(commands)
    }
}

#[cfg(unix)]
fn process_command(pid: libc::pid_t) -> Option<String> {
    let output = Command::new("ps")
        .args(["-o", "comm=", "-p", &pid.to_string()])
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .filter(|value| !value.is_empty())
}

#[cfg(unix)]
fn parent_pid(pid: libc::pid_t) -> Option<libc::pid_t> {
    let output = Command::new("ps")
        .args(["-o", "ppid=", "-p", &pid.to_string()])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout).trim().parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn local_session_is_not_ssh() {
        let env: HashMap<String, String> = HashMap::new();
        assert!(detect_with(|| None, |key| env.get(key).cloned()).is_none());
    }

    #[test]
    fn ssh_env_marks_session_remote() {
        let env = HashMap::from([("SSH_CONNECTION".to_owned(), "client server".to_owned())]);
        let detected = detect_with(|| None, |key| env.get(key).cloned());
        assert!(detected.is_some_and(|ctx| ctx.remote));
    }

    #[test]
    fn sudo_ssh_session_uses_parent_chain_fallback() {
        let env = HashMap::from([("SUDO_USER".to_owned(), "tiger".to_owned())]);
        let detected = detect_with(
            || Some(vec!["sudo".into(), "zsh".into(), "sshd".into()]),
            |key| env.get(key).cloned(),
        );
        assert!(detected.is_some_and(|ctx| ctx.remote));
    }

    #[test]
    fn sudo_ssh_session_accepts_sshd_session_command_names() {
        let env = HashMap::from([("SUDO_UID".to_owned(), "501".to_owned())]);
        let detected = detect_with(
            || {
                Some(vec![
                    "sudo".into(),
                    "bash".into(),
                    "sshd: tiger@pts/0".into(),
                ])
            },
            |key| env.get(key).cloned(),
        );
        assert!(detected.is_some_and(|ctx| ctx.remote));
    }

    #[test]
    fn sudo_ssh_session_accepts_sshd_command_paths() {
        let env = HashMap::from([("SUDO_USER".to_owned(), "tiger".to_owned())]);
        let detected = detect_with(
            || {
                Some(vec![
                    "sudo".into(),
                    "zsh".into(),
                    "/usr/sbin/sshd -D".into(),
                ])
            },
            |key| env.get(key).cloned(),
        );
        assert!(detected.is_some_and(|ctx| ctx.remote));
    }
}
