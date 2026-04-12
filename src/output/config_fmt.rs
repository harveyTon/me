use crate::{
    model::{Field, MeInfo},
    output::{
        RenderOptions,
        semantics::{ContextTextStyle, compact_list, context_text, display_host, network_summary},
    },
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
        Field::Groups => compact_list(&info.identity.groups, 3),
        Field::Host => Some(display_host(&info.identity.host)),
        Field::Shell => info.runtime.shell.clone(),
        Field::Pid => Some(info.runtime.pid.to_string()),
        Field::Ppid => info.runtime.ppid.map(|v| v.to_string()),
        Field::Tty => info.runtime.tty.clone(),
        Field::Privilege => Some(info.privilege.clone()),
        Field::Sudo => Some(if info.sudo { "yes" } else { "no" }.into()),
        Field::Ssh => Some(if info.ssh { "yes" } else { "no" }.into()),
        Field::Network => network_summary(&info.network, 1),
        Field::Pwd => info.pwd.as_ref().map(|pwd| pwd.display.clone()),
        Field::Context => context_text(info, ContextTextStyle::Config),
    }
}

fn finish(lines: Vec<String>) -> String {
    if lines.is_empty() {
        String::new()
    } else {
        format!("{}\n", lines.join("\n"))
    }
}
