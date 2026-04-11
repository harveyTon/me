use crate::{
    cli::{Cli, OutputFormat},
    config::{Config, PlainMode, View},
    model::{Field, MeInfo},
    output::{RenderOptions, render_block, render_compact, render_config, render_json},
};

#[derive(Debug, Clone, Copy)]
pub enum AppMode {
    Block,
    Compact,
    Config,
    Json,
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse_args();
    let config = Config::load()?;
    let fields = fields(&cli, &config);
    let options = RenderOptions {
        color: crate::util::color::should_color(cli.no_color),
        icons: if cli.no_color {
            crate::config::IconMode::Off
        } else {
            config.icons
        },
        full: cli.full,
    };
    let mode = mode(&cli, &config);

    if cli.watch {
        return crate::interactive::watch::run(
            &config,
            &fields,
            mode,
            options,
            cli.interval.unwrap_or(config.watch.interval),
        );
    }

    let info = crate::providers::collect(&config.context);
    if let Some(target) = cli.copy {
        return crate::interactive::copy::run(&info, target);
    }
    if cli.plain {
        print!("{}", plain(&info, config.plain_mode));
        return Ok(());
    }
    print!("{}", render(&info, &fields, mode, &options)?);
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

fn fields(cli: &Cli, config: &Config) -> Vec<Field> {
    let selected = cli.selected_fields();
    if !selected.is_empty() {
        selected
    } else if !config.fields.is_empty() {
        config.fields.clone()
    } else {
        Field::defaults()
    }
}

fn mode(cli: &Cli, config: &Config) -> AppMode {
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

fn plain(info: &MeInfo, mode: PlainMode) -> String {
    match mode {
        PlainMode::User => format!("{}\n", info.identity.user),
        PlainMode::UserHost => format!(
            "{}@{}\n",
            info.identity.user,
            crate::output::config_fmt::display_host(&info.identity.host)
        ),
    }
}
