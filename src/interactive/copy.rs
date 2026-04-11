use crate::{model::MeInfo, util::clipboard};
use anyhow::{Context, bail};
use std::io::{self, Write};

pub fn run(info: &MeInfo, target: Option<String>) -> anyhow::Result<()> {
    let target = match target {
        Some(value) if !value.is_empty() => value,
        _ => prompt()?,
    };
    let value = copy_value(info, &target)?;
    clipboard::copy_to_clipboard(&value)?;
    println!("copied {target}");
    Ok(())
}

fn prompt() -> anyhow::Result<String> {
    let choices = ["user", "host", "ssh-target"];
    for (index, choice) in choices.iter().enumerate() {
        println!("{}: {}", index + 1, choice);
    }
    print!("copy: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let index = input
        .trim()
        .parse::<usize>()
        .context("selection must be a number")?;
    choices
        .get(index.saturating_sub(1))
        .map(|value| (*value).to_owned())
        .context("selection out of range")
}

fn copy_value(info: &MeInfo, target: &str) -> anyhow::Result<String> {
    match target {
        "user" => Ok(info.identity.user.clone()),
        "host" => Ok(info.identity.host.clone()),
        "ssh-target" | "ssh_target" => Ok(format!("{}@{}", info.identity.user, info.identity.host)),
        "public-ip" | "public_ip" => {
            bail!("public IP copy is intentionally not implemented in this local-first pass")
        }
        other => bail!("unsupported copy target '{other}'"),
    }
}
