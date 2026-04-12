use crate::model::Field;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Clone, Parser)]
#[command(
    name = "me",
    version,
    about = "A modern, context-aware replacement for whoami.",
    disable_help_flag = true,
    help_template = "{about}\n\nUsage:\n  {usage}\n\n{all-args}\nExamples:\n  me\n  me --compact\n  me --json\n  me -u -h -n\n"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
    #[arg(long = "help", action = clap::ArgAction::Help, help = "Print help", help_heading = "General")]
    pub help: Option<bool>,
    #[arg(long, help = "Use compact one-line output", help_heading = "Output")]
    pub compact: bool,
    #[arg(long, help = "Emit structured JSON", help_heading = "Output")]
    pub json: bool,
    #[arg(long, value_enum, default_value_t = OutputFormat::Labeled, help = "Select labeled or config-style output", help_heading = "Output")]
    pub format: OutputFormat,
    #[arg(
        long,
        help = "Print plain identity, defaulting to user@host",
        help_heading = "Output"
    )]
    pub plain: bool,
    #[arg(long, help = "Disable ANSI color", help_heading = "Output")]
    pub no_color: bool,
    #[arg(
        long,
        help = "Show full multi-line values where the default is condensed",
        help_heading = "Output"
    )]
    pub full: bool,
    #[arg(long, help = "Refresh output in place", help_heading = "Interactive")]
    pub watch: bool,
    #[arg(
        long,
        help = "Watch refresh interval in seconds",
        help_heading = "Interactive"
    )]
    pub interval: Option<u64>,
    #[arg(
        long,
        help = "Skip slower context version checks for prompt usage",
        help_heading = "Interactive"
    )]
    pub fast: bool,
    #[arg(long, num_args = 0..=1, value_name = "FIELD", help = "Copy a field, or prompt when omitted", help_heading = "Interactive")]
    pub copy: Option<Option<String>>,
    #[arg(
        long = "field",
        value_delimiter = ',',
        help = "Select fields by name, comma-separated",
        help_heading = "Fields"
    )]
    pub fields: Vec<Field>,
    #[arg(short = 'u', help = "Select user", help_heading = "Fields")]
    pub user: bool,
    #[arg(short = 'h', help = "Select host", help_heading = "Fields")]
    pub host: bool,
    #[arg(short = 's', help = "Select shell", help_heading = "Fields")]
    pub shell: bool,
    #[arg(short = 'p', help = "Select pid", help_heading = "Fields")]
    pub pid: bool,
    #[arg(short = 'P', help = "Select ppid", help_heading = "Fields")]
    pub ppid: bool,
    #[arg(short = 't', help = "Select tty", help_heading = "Fields")]
    pub tty: bool,
    #[arg(short = 'g', help = "Select groups", help_heading = "Fields")]
    pub groups: bool,
    #[arg(short = 'i', help = "Select uid and gid", help_heading = "Fields")]
    pub ids: bool,
    #[arg(short = 'n', help = "Select network", help_heading = "Fields")]
    pub network: bool,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    #[command(about = "Install shell integration")]
    Install(InstallArgs),
    #[command(about = "Remove shell integration")]
    Uninstall(UninstallArgs),
    #[command(about = "Update me itself")]
    Update(UpdateArgs),
}

#[derive(Debug, Clone, clap::Args)]
pub struct InstallArgs {
    #[arg(long = "help", action = clap::ArgAction::Help, help = "Print help")]
    pub help: Option<bool>,
    #[arg(long, help = "Run without interactive prompts")]
    pub non_interactive: bool,
    #[arg(long, value_enum, help = "Select login shell behavior")]
    pub login: Option<LoginMode>,
    #[arg(long, value_enum, help = "Select interactive shell behavior")]
    pub interactive: Option<InteractiveMode>,
    #[arg(long, value_enum, help = "Override shell detection")]
    pub shell: Option<Shell>,
    #[arg(long, value_name = "PATH", help = "Override target shell config file")]
    pub file: Option<PathBuf>,
}

#[derive(Debug, Clone, clap::Args)]
pub struct UninstallArgs {
    #[arg(long = "help", action = clap::ArgAction::Help, help = "Print help")]
    pub help: Option<bool>,
    #[arg(long, help = "Run without interactive prompts")]
    pub non_interactive: bool,
    #[arg(long, help = "Confirm global uninstall without prompting")]
    pub yes: bool,
    #[arg(long, value_enum, help = "Override shell detection")]
    pub shell: Option<Shell>,
    #[arg(long, value_name = "PATH", help = "Override target shell config file")]
    pub file: Option<PathBuf>,
}

#[derive(Debug, Clone, clap::Args)]
pub struct UpdateArgs {
    #[arg(long = "help", action = clap::ArgAction::Help, help = "Print help")]
    pub help: Option<bool>,
    #[arg(long, help = "Check for an update without upgrading")]
    pub check: bool,
    #[arg(long, help = "Run update without interactive prompts")]
    pub non_interactive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum LoginMode {
    None,
    Full,
    Compact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum InteractiveMode {
    None,
    Compact,
    Prompt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum)]
pub enum Shell {
    Zsh,
    Bash,
    Fish,
    Nushell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Labeled,
    Config,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }

    pub fn selected_fields(&self) -> Vec<Field> {
        let mut fields = self.fields.clone();
        if self.user {
            fields.push(Field::User);
        }
        if self.host {
            fields.push(Field::Host);
        }
        if self.shell {
            fields.push(Field::Shell);
        }
        if self.pid {
            fields.push(Field::Pid);
        }
        if self.ppid {
            fields.push(Field::Ppid);
        }
        if self.tty {
            fields.push(Field::Tty);
        }
        if self.groups {
            fields.push(Field::Groups);
        }
        if self.ids {
            fields.push(Field::Uid);
            fields.push(Field::Gid);
        }
        if self.network {
            fields.push(Field::Network);
        }
        dedupe(fields)
    }
}

fn dedupe(fields: Vec<Field>) -> Vec<Field> {
    let mut out = Vec::new();
    for field in fields {
        if !out.contains(&field) {
            out.push(field);
        }
    }
    out
}
