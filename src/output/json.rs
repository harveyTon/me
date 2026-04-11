use crate::model::{Field, MeInfo};
use crate::output::config_fmt::display_host;
use serde_json::{Map, Value, json};

pub fn render_json(info: &MeInfo, fields: &[Field]) -> anyhow::Result<String> {
    let mut out = Map::new();
    for field in fields {
        match field {
            Field::User => insert(&mut out, "user", json!(info.identity.user)),
            Field::Uid => insert(&mut out, "uid", json!(info.identity.uid)),
            Field::Gid => insert(&mut out, "gid", json!(info.identity.gid)),
            Field::Groups => insert(&mut out, "groups", json!(info.identity.groups)),
            Field::Host => insert(&mut out, "host", json!(display_host(&info.identity.host))),
            Field::Shell => insert_optional(&mut out, "shell", info.runtime.shell.as_ref()),
            Field::Pid => insert(&mut out, "pid", json!(info.runtime.pid)),
            Field::Ppid => {
                if let Some(ppid) = info.runtime.ppid {
                    insert(&mut out, "ppid", json!(ppid));
                }
            }
            Field::Tty => insert_optional(&mut out, "tty", info.runtime.tty.as_ref()),
            Field::Privilege => insert(&mut out, "privilege", json!(info.privilege)),
            Field::Sudo => insert(&mut out, "sudo", json!(info.sudo)),
            Field::Ssh => insert(&mut out, "ssh", json!(info.ssh)),
            Field::Network => insert(&mut out, "network", json!(info.network)),
            Field::Context => {
                if let Some(context) = context_json(info) {
                    insert(&mut out, "context", context);
                }
            }
        }
    }
    Ok(format!(
        "{}\n",
        serde_json::to_string_pretty(&Value::Object(out))?
    ))
}

fn insert(out: &mut Map<String, Value>, key: &str, value: Value) {
    out.insert(key.to_owned(), value);
}

fn insert_optional(out: &mut Map<String, Value>, key: &str, value: Option<&String>) {
    if let Some(value) = value {
        insert(out, key, json!(value));
    }
}

fn context_json(info: &MeInfo) -> Option<Value> {
    let mut context = Map::new();
    if let Some(ssh) = &info.context.ssh {
        insert(&mut context, "ssh", json!(ssh));
    }
    if let Some(container) = &info.context.container {
        insert(&mut context, "container", json!(container));
    }
    if let Some(git) = &info.context.git {
        insert(&mut context, "git", json!(git));
    }
    if let Some(project) = &info.context.project {
        insert(&mut context, "project", json!(project));
    }
    (!context.is_empty()).then_some(Value::Object(context))
}
