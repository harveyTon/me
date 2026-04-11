use crate::{
    model::{Field, MeInfo},
    output::RenderOptions,
};

pub fn render_config(info: &MeInfo, fields: &[Field], _options: &RenderOptions) -> String {
    let mut lines = Vec::new();
    for field in fields {
        if let Some(value) = value_for(info, *field) {
            lines.push(format!("{} = {}", field.key(), value));
        }
    }
    finish(lines)
}

pub(crate) fn value_for(info: &MeInfo, field: Field) -> Option<String> {
    match field {
        Field::User => Some(info.identity.user.clone()),
        Field::Uid => Some(info.identity.uid.to_string()),
        Field::Gid => Some(info.identity.gid.to_string()),
        Field::Groups => Some(compact_list(&info.identity.groups, 3)).filter(|s| !s.is_empty()),
        Field::Host => Some(display_host(&info.identity.host)),
        Field::Shell => info.runtime.shell.clone(),
        Field::Pid => Some(info.runtime.pid.to_string()),
        Field::Ppid => info.runtime.ppid.map(|v| v.to_string()),
        Field::Tty => info.runtime.tty.clone(),
        Field::Privilege => Some(info.privilege.clone()),
        Field::Sudo => Some(if info.sudo { "yes" } else { "no" }.into()),
        Field::Ssh => Some(if info.ssh { "yes" } else { "no" }.into()),
        Field::Network => Some(compact_list(&info.network.local_ips, 1)).filter(|s| !s.is_empty()),
        Field::Pwd => info.pwd.as_ref().map(|pwd| pwd.display.clone()),
        Field::Context => context_value(info),
    }
}

pub(crate) fn display_host(host: &str) -> String {
    host.strip_suffix(".local").unwrap_or(host).to_owned()
}

pub(crate) fn compact_list(values: &[String], keep: usize) -> String {
    if values.len() <= keep {
        return values.join(", ");
    }
    format!("{} (+{})", values[..keep].join(", "), values.len() - keep)
}

fn context_value(info: &MeInfo) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(container) = &info.context.container {
        parts.push(match &container.id {
            Some(id) => format!("{}:{id}", container.kind),
            None => container.kind.clone(),
        });
    }
    if let Some(project) = &info.context.project {
        parts.push(project.kind.clone());
    }
    if let Some(git) = &info.context.git {
        parts.push(format!("git:{}", git.branch));
    }
    Some(parts.join(", ")).filter(|s| !s.is_empty())
}

fn finish(lines: Vec<String>) -> String {
    if lines.is_empty() {
        String::new()
    } else {
        format!("{}\n", lines.join("\n"))
    }
}
