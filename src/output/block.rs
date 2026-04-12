use crate::{
    model::{Field, MeInfo},
    output::{
        RenderOptions,
        config_fmt::value_for,
        semantics::{ContextTextStyle, compact_list, context_text, network_lines, network_summary},
    },
};
use owo_colors::OwoColorize;

pub fn render_block(info: &MeInfo, fields: &[Field], options: &RenderOptions) -> String {
    let groups = [
        identity_group(info, fields),
        system_group(info, fields, options),
        session_group(info, fields),
        network_group(info, fields, options),
        location_group(info, fields),
    ];

    let rendered: Vec<String> = groups
        .into_iter()
        .flatten()
        .map(|group| render_group(group, options))
        .collect();

    if rendered.is_empty() {
        String::new()
    } else {
        format!("{}\n", rendered.join("\n\n"))
    }
}

struct Group {
    key: &'static str,
    rows: Vec<Row>,
}

struct Row {
    key: &'static str,
    value: RowValue,
}

enum RowValue {
    Single(String),
    Multi(Vec<String>),
}

fn identity_group(info: &MeInfo, fields: &[Field]) -> Option<Group> {
    let mut rows = Vec::new();
    push_value_row(&mut rows, info, Field::User);
    push_value_row(&mut rows, info, Field::Host);
    push_value_row(&mut rows, info, Field::Shell);
    filter_group(
        "identity",
        rows,
        fields,
        &[Field::User, Field::Host, Field::Shell],
    )
}

fn system_group(info: &MeInfo, fields: &[Field], options: &RenderOptions) -> Option<Group> {
    let mut rows = Vec::new();
    push_value_row(&mut rows, info, Field::Uid);
    push_value_row(&mut rows, info, Field::Gid);
    if fields.contains(&Field::Groups) {
        if options.full {
            if !info.identity.groups.is_empty() {
                rows.push(Row {
                    key: "groups",
                    value: RowValue::Multi(info.identity.groups.clone()),
                });
            }
        } else if let Some(groups) = compact_list(&info.identity.groups, 3) {
            rows.push(Row {
                key: "groups",
                value: RowValue::Single(groups),
            });
        }
    }
    push_value_row(&mut rows, info, Field::Pid);
    push_value_row(&mut rows, info, Field::Ppid);
    push_value_row(&mut rows, info, Field::Tty);
    filter_group(
        "system",
        rows,
        fields,
        &[
            Field::Uid,
            Field::Gid,
            Field::Groups,
            Field::Pid,
            Field::Ppid,
            Field::Tty,
        ],
    )
}

fn session_group(info: &MeInfo, fields: &[Field]) -> Option<Group> {
    let mut rows = Vec::new();
    push_value_row(&mut rows, info, Field::Privilege);
    push_value_row(&mut rows, info, Field::Sudo);
    push_value_row(&mut rows, info, Field::Ssh);
    filter_group(
        "session",
        rows,
        fields,
        &[Field::Privilege, Field::Sudo, Field::Ssh],
    )
}

fn network_group(info: &MeInfo, fields: &[Field], options: &RenderOptions) -> Option<Group> {
    if !fields.contains(&Field::Network) {
        return None;
    }

    let rows = if options.full {
        let lines = network_lines(&info.network);
        if lines.is_empty() {
            Vec::new()
        } else {
            vec![Row {
                key: "",
                value: RowValue::Multi(lines),
            }]
        }
    } else if let Some(summary) = network_summary(&info.network, 1) {
        vec![Row {
            key: "summary",
            value: RowValue::Single(summary),
        }]
    } else {
        Vec::new()
    };

    filter_group("network", rows, fields, &[Field::Network])
}

fn location_group(info: &MeInfo, fields: &[Field]) -> Option<Group> {
    let mut rows = Vec::new();
    push_value_row(&mut rows, info, Field::Pwd);
    if fields.contains(&Field::Context)
        && let Some(context) = context_text(info, ContextTextStyle::Block)
    {
        rows.push(Row {
            key: "context",
            value: RowValue::Single(context),
        });
    }
    filter_group("location", rows, fields, &[Field::Pwd, Field::Context])
}

fn filter_group(
    key: &'static str,
    rows: Vec<Row>,
    fields: &[Field],
    group_fields: &[Field],
) -> Option<Group> {
    let has_requested_field = group_fields.iter().any(|field| fields.contains(field));
    (has_requested_field && !rows.is_empty()).then_some(Group { key, rows })
}

fn push_value_row(rows: &mut Vec<Row>, info: &MeInfo, field: Field) {
    if let Some(value) = value_for(info, field) {
        rows.push(Row {
            key: field.key(),
            value: RowValue::Single(value),
        });
    }
}

fn render_group(group: Group, options: &RenderOptions) -> String {
    let mut out = String::new();
    let heading = format!("{}:", group.key);
    if options.color {
        out.push_str(&heading.dimmed().to_string());
    } else {
        out.push_str(&heading);
    }
    out.push('\n');

    let width = group
        .rows
        .iter()
        .filter(|row| !row.key.is_empty())
        .map(|row| row.key.len())
        .max()
        .unwrap_or(0);

    for row in group.rows {
        match row.value {
            RowValue::Single(value) => {
                let label = format!("{}:", row.key);
                if options.color {
                    let padded = format!("  {label:<width$}", width = width + 1);
                    out.push_str(&format!("{}  {value}\n", padded.dimmed()));
                } else {
                    out.push_str(&format!("  {label:<width$}  {value}\n", width = width + 1));
                }
            }
            RowValue::Multi(values) => {
                if row.key.is_empty() {
                    for value in values {
                        out.push_str(&format!("  {value}\n"));
                    }
                } else {
                    let label = format!("{}:", row.key);
                    if options.color {
                        out.push_str(&format!("{}\n", format!("  {label}").dimmed()));
                    } else {
                        out.push_str(&format!("  {label}\n"));
                    }
                    for value in values {
                        out.push_str(&format!("    {value}\n"));
                    }
                }
            }
        }
    }

    out.trim_end().to_string()
}
