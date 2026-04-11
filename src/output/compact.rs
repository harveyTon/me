use crate::{
    model::{Field, MeInfo},
    output::config_fmt::{display_host, value_for},
};

pub fn render_compact(info: &MeInfo, fields: &[Field]) -> String {
    let requested_subset = fields != Field::defaults().as_slice();
    let mut parts = Vec::new();

    if requested_subset {
        let combined_identity = fields.contains(&Field::User) && fields.contains(&Field::Host);
        if combined_identity {
            parts.push(format!(
                "{}@{}",
                info.identity.user,
                display_host(&info.identity.host)
            ));
        }
        for field in fields {
            if combined_identity && matches!(field, Field::User | Field::Host) {
                continue;
            }
            if let Some(value) = value_for(info, *field) {
                parts.push(value);
            }
        }
    } else {
        parts.push(format!(
            "{}@{}",
            info.identity.user,
            display_host(&info.identity.host)
        ));
        if let Some(shell) = &info.runtime.shell {
            parts.push(shell.clone());
        }
        parts.push(info.privilege.clone());
    }

    if !requested_subset {
        if info.ssh {
            parts.push("ssh".into());
        } else if let Some(container) = &info.context.container {
            parts.push(container.kind.clone());
        } else {
            parts.push("local".into());
        }
        if let Some(project) = &info.context.project
            && matches!(project.kind.as_str(), "rust" | "node")
        {
            parts.push(project.kind.clone());
        }
    }

    format!("{}\n", parts.join(" · "))
}
