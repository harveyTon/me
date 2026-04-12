use crate::{
    cli::{Cli, Command, OutputFormat},
    config::{Config, PlainMode, View},
    model::{Field, MeInfo},
    output::{
        RenderOptions, render_block, render_compact, render_config, render_json,
        semantics::display_host,
    },
};
use std::io::{self, Write};

#[derive(Debug, Clone, Copy)]
pub enum AppMode {
    Block,
    Compact,
    Config,
    Json,
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse_args();
    if let Some(command) = cli.command.clone() {
        return match command {
            Command::Install(_) | Command::Uninstall(_) => crate::shell_integration::run(command),
            Command::Update(args) => crate::update::run(args),
        };
    }

    let (config, config_warning) = Config::load();
    if let Some(warning) = config_warning {
        eprintln!("me: {warning}");
    }
    let selected_fields = cli.selected_fields();
    let explicit_network = selected_fields.contains(&Field::Network);
    let fields = resolve_fields(selected_fields, &config);
    let collect_network = should_collect_network(cli.fast, explicit_network);
    let options = resolve_render_options(&cli, &config);
    let mode = resolve_mode(&cli, &config);

    if cli.watch {
        return crate::interactive::watch::run(
            &config,
            &fields,
            mode,
            options,
            watch_interval(&cli, &config),
            cli.fast,
            collect_network,
        );
    }

    let info = crate::providers::collect(&config.context, cli.fast, collect_network);
    if let Some(target) = cli.copy {
        return crate::interactive::copy::run(&info, target);
    }
    if cli.plain {
        write_stdout(&plain(&info, config.plain_mode))?;
        return Ok(());
    }
    write_stdout(&render(&info, &fields, mode, &options)?)?;
    Ok(())
}

pub fn render(
    info: &MeInfo,
    fields: &[Field],
    mode: AppMode,
    options: &RenderOptions,
) -> anyhow::Result<String> {
    match mode {
        AppMode::Block => Ok(render_block(info, fields, options)),
        AppMode::Compact => Ok(render_compact(info, fields)),
        AppMode::Config => Ok(render_config(info, fields, options)),
        AppMode::Json => render_json(info, fields),
    }
}

fn resolve_fields(selected: Vec<Field>, config: &Config) -> Vec<Field> {
    if !selected.is_empty() {
        selected
    } else if !config.fields.is_empty() {
        config.fields.clone()
    } else {
        Field::defaults()
    }
}

pub(crate) fn should_collect_network(_fast: bool, _explicit_network: bool) -> bool {
    true
}

fn resolve_mode(cli: &Cli, config: &Config) -> AppMode {
    if cli.json {
        AppMode::Json
    } else if cli.format == OutputFormat::Config {
        AppMode::Config
    } else if cli.compact || config.view == View::Compact {
        AppMode::Compact
    } else {
        AppMode::Block
    }
}

fn resolve_render_options(cli: &Cli, config: &Config) -> RenderOptions {
    RenderOptions {
        color: crate::util::color::should_color(cli.no_color, config.color),
        icons: if cli.no_color {
            crate::config::IconMode::Off
        } else {
            config.icons
        },
        full: cli.full,
        light_theme: config.is_light_theme(),
    }
}

fn watch_interval(cli: &Cli, config: &Config) -> u64 {
    cli.interval.unwrap_or(config.watch.interval)
}

fn plain(info: &MeInfo, mode: PlainMode) -> String {
    match mode {
        PlainMode::User => format!("{}\n", info.identity.user),
        PlainMode::UserHost => format!(
            "{}@{}\n",
            info.identity.user,
            display_host(&info.identity.host)
        ),
    }
}

fn write_stdout(value: &str) -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    stdout.write_all(value.as_bytes())?;
    stdout.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fast_keeps_network_for_default_fields() {
        assert!(should_collect_network(true, false));
    }

    #[test]
    fn fast_keeps_network_for_explicit_network_request() {
        assert!(should_collect_network(true, true));
    }
}
