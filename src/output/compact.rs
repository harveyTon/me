use crate::{
    model::{Field, MeInfo},
    output::{
        config_fmt::value_for,
        semantics::{
            compact_cwd_name, compact_env_label, compact_project_parts, display_host, display_user,
        },
    },
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
        parts.push(compact_env_label(info));
        parts.extend(compact_project_parts(info));
        if let Some(cwd) = compact_cwd_name(info) {
            parts.push(cwd);
        }
    }

    format!("{}\n", parts.join(" · "))
}

fn compact_identity(info: &MeInfo) -> String {
    format!(
        "{}@{}",
        display_user(info),
        display_host(&info.identity.host)
    )
}
