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
        parts.push(compact_identity(info));
        parts.push(compact_env(info).to_owned());
        if let Some(project) = compact_project(info) {
            parts.push(project);
        }
    }

    format!("{}\n", parts.join(" · "))
}

fn compact_identity(info: &MeInfo) -> String {
    let user = if info.identity.uid == 0 {
        "root"
    } else {
        &info.identity.user
    };
    format!("{}@{}", user, display_host(&info.identity.host))
}

fn compact_env(info: &MeInfo) -> String {
    if info.ssh {
        "ssh".into()
    } else if let Some(container) = &info.context.container {
        container.kind.clone()
    } else {
        "local".into()
    }
}

fn compact_project(info: &MeInfo) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(project) = &info.context.project {
        parts.push(project.kind.clone());
    }
    if let Some(git) = &info.context.git {
        parts.push(format!("git:{}", git.branch));
    }
    Some(parts.join(" ")).filter(|s| !s.is_empty())
}
