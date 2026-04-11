use crate::{
    config::IconMode,
    model::{Field, MeInfo},
    output::{
        RenderOptions,
        config_fmt::{compact_list, display_host, value_for},
    },
};
use owo_colors::OwoColorize;

pub fn render_block(info: &MeInfo, fields: &[Field], options: &RenderOptions) -> String {
    let mut out = String::new();
    let requested_subset = fields != Field::defaults().as_slice();
    if !requested_subset {
        let user = if options.color {
            info.identity.user.bold().green().to_string()
        } else {
            info.identity.user.clone()
        };
        let host = display_host(&info.identity.host);
        let host = if options.color && info.ssh {
            host.yellow().bold().to_string()
        } else {
            host
        };
        let shell = info
            .runtime
            .shell
            .as_deref()
            .map(|s| format!("  {}{s}", icon_for(options.icons, "shell")))
            .unwrap_or_default();
        out.push_str(&format!("{user}@{host}{shell}\n\n"));
    }

    let rows = rows(info, fields, options, requested_subset);
    let width = rows.iter().map(|row| row.key.len()).max().unwrap_or(0);
    for row in rows {
        let key = row.key;
        let value = row.value;
        let label = format!("{key}:");
        let gap = if requested_subset { " " } else { "  " };
        if value.starts_with('\n') {
            if options.color {
                out.push_str(&format!("{}{}\n", label.dimmed(), value));
            } else {
                out.push_str(&format!("{label}{value}\n"));
            }
        } else if options.color {
            let padded_label = format!("{label:<width$}", width = width + 1);
            out.push_str(&format!("{}{}{}\n", padded_label.dimmed(), gap, value));
        } else {
            out.push_str(&format!("{label:<width$}{gap}{value}\n", width = width + 1));
        }
        if row.add_gap_after {
            out.push('\n');
        }
    }

    if fields.contains(&Field::Context)
        && let Some(context) = context_summary(info)
    {
        if !out.is_empty() {
            out.push('\n');
        }
        let label = "context:";
        if options.color {
            let padded_label = format!("{label:<width$}", width = width + 1);
            out.push_str(&format!("{}  {}\n", padded_label.dimmed(), context));
        } else {
            out.push_str(&format!("{label:<width$}  {context}\n", width = width + 1));
        }
    }
    out
}

struct Row {
    key: &'static str,
    value: String,
    add_gap_after: bool,
}

fn rows(
    info: &MeInfo,
    fields: &[Field],
    options: &RenderOptions,
    requested_subset: bool,
) -> Vec<Row> {
    fields
        .iter()
        .filter(|field| {
            !matches!(field, Field::Context)
                && (requested_subset || !matches!(field, Field::User | Field::Host))
        })
        .filter_map(|field| row(info, *field, options))
        .collect()
}

fn row(info: &MeInfo, field: Field, options: &RenderOptions) -> Option<Row> {
    match field {
        Field::Groups if options.full => multiline_row("groups", &info.identity.groups),
        Field::Network if options.full => multiline_row("network", &info.network.local_ips),
        Field::Groups => Some(Row {
            key: "groups",
            value: compact_list(&info.identity.groups, 3),
            add_gap_after: false,
        })
        .filter(|row| !row.value.is_empty()),
        Field::Network => Some(Row {
            key: "network",
            value: prefixed_value(options, "network", compact_list(&info.network.local_ips, 1)),
            add_gap_after: false,
        })
        .filter(|row| !row.value.is_empty()),
        Field::Tty => value_for(info, field).map(|value| Row {
            key: field.key(),
            value,
            add_gap_after: false,
        }),
        Field::Privilege => value_for(info, field).map(|value| Row {
            key: field.key(),
            value: prefixed_value(options, "privilege", value),
            add_gap_after: false,
        }),
        _ => value_for(info, field).map(|value| Row {
            key: field.key(),
            value,
            add_gap_after: false,
        }),
    }
}

fn multiline_row(key: &'static str, values: &[String]) -> Option<Row> {
    if values.is_empty() {
        return None;
    }
    Some(Row {
        key,
        value: format!("\n  {}", values.join("\n  ")),
        add_gap_after: false,
    })
}

fn context_summary(info: &MeInfo) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(container) = &info.context.container {
        parts.push(container.kind.clone());
    }
    if let Some(project) = &info.context.project {
        parts.push(match &project.version {
            Some(version) => format!("{} ({version})", project.kind),
            None => project.kind.clone(),
        });
    }
    Some(parts.join(", ")).filter(|summary| !summary.is_empty())
}

fn prefixed_value(options: &RenderOptions, icon: &str, value: String) -> String {
    format!("{}{}", icon_for(options.icons, icon), value)
}

fn icon_for(mode: IconMode, icon: &str) -> &'static str {
    match mode {
        IconMode::On => crate::util::icons::field_icon(icon),
        IconMode::Auto | IconMode::Off => "",
    }
}
