use crate::cli::{Command, InstallArgs, InteractiveMode, LoginMode, Shell, UninstallArgs};
use anyhow::{Context, bail};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

const START_MARKER: &str = "# >>> me install >>>";
const END_MARKER: &str = "# <<< me install <<<";
const SIGNATURE_PREFIX: &str = "# me-managed:";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum TargetRole {
    Login,
    Interactive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TargetFile {
    shell: Shell,
    path: PathBuf,
    roles: BTreeSet<TargetRole>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockChange {
    Added,
    Updated,
    Unchanged,
}

#[derive(Debug, Clone)]
struct ExistingIntegration {
    shell: Option<Shell>,
    path: PathBuf,
    full_blocks: usize,
    partial_or_ambiguous: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CrossShellChoice {
    Keep,
    Remove,
    Abort,
}

pub fn run(command: Command) -> anyhow::Result<()> {
    match command {
        Command::Install(args) => install(args),
        Command::Uninstall(args) => uninstall(args),
        Command::Update(_) => unreachable!("update command is handled by app"),
    }
}

fn install(args: InstallArgs) -> anyhow::Result<()> {
    let shell = args
        .shell
        .or_else(detect_current_shell)
        .context("could not detect shell; pass --shell <zsh|bash|fish|nushell>")?;
    let login = choose_login_mode(&args)?;
    let interactive = choose_interactive_mode(&args)?;
    let home = shell_home_dir().context("could not determine home directory")?;
    let config_dir = shell_config_dir(&home);
    let existing = scan_known_integrations(&home, &config_dir)?;
    let other_shells = existing_for_other_shells(&existing, shell);

    if !other_shells.is_empty() {
        if args.non_interactive {
            warn_existing_other_shells(&other_shells);
        } else {
            match prompt_cross_shell_choice(shell, &other_shells)? {
                CrossShellChoice::Keep => {}
                CrossShellChoice::Remove => remove_full_blocks_from_integrations(&other_shells)?,
                CrossShellChoice::Abort => {
                    println!("aborted");
                    return Ok(());
                }
            }
        }
    }

    let targets = resolve_targets(
        &home,
        &config_dir,
        shell,
        args.file.as_deref(),
        login,
        interactive,
    );
    if targets.is_empty() {
        println!("me install: no integration selected");
        return Ok(());
    }

    for target in &targets {
        if target.roles.contains(&TargetRole::Interactive)
            && interactive == InteractiveMode::Prompt
            && prompt_integration_looks_unsafe(target.shell, &target.path)
        {
            if args.non_interactive {
                bail!(
                    "prompt integration looks unsafe in {}; choose --interactive compact or edit manually",
                    display_path(&target.path)
                );
            }
            eprintln!(
                "me: prompt integration looks unsafe in {}; using compact instead",
                display_path(&target.path)
            );
        }
    }

    let mut summary = Vec::new();
    for target in targets {
        let effective_interactive = if target.roles.contains(&TargetRole::Interactive)
            && interactive == InteractiveMode::Prompt
            && prompt_integration_looks_unsafe(target.shell, &target.path)
        {
            InteractiveMode::Compact
        } else if target.roles.contains(&TargetRole::Interactive) {
            interactive
        } else {
            InteractiveMode::None
        };
        let effective_login = if target.roles.contains(&TargetRole::Login) {
            login
        } else {
            LoginMode::None
        };

        if effective_login == LoginMode::None && effective_interactive == InteractiveMode::None {
            continue;
        }
        if effective_login != LoginMode::None {
            summary.push((
                TargetRole::Login,
                effective_login.name(),
                target.path.clone(),
            ));
        }
        if effective_interactive != InteractiveMode::None {
            summary.push((
                TargetRole::Interactive,
                effective_interactive.name(),
                target.path.clone(),
            ));
        }

        let existing = read_optional(&target.path)?;
        let block = render_block(target.shell, effective_login, effective_interactive);
        let (updated, change) = upsert_block(&existing, &block);
        write_file(&target.path, &updated)?;
        println!(
            "me install: {} {} ({})",
            match change {
                BlockChange::Added => "installed",
                BlockChange::Updated => "updated",
                BlockChange::Unchanged => "unchanged",
            },
            display_path(&target.path),
            target.shell.name()
        );
    }

    print_install_summary(shell, &summary);
    Ok(())
}

fn uninstall(args: UninstallArgs) -> anyhow::Result<()> {
    let home = shell_home_dir().context("could not determine home directory")?;
    let config_dir = shell_config_dir(&home);
    let integrations = if let Some(file) = &args.file {
        scan_paths(std::slice::from_ref(file))?
    } else if args.non_interactive && !args.yes {
        bail!("global non-interactive uninstall requires --yes or --file")
    } else {
        scan_known_integrations(&home, &config_dir)?
    };

    let removable: Vec<_> = integrations
        .iter()
        .filter(|integration| integration.full_blocks > 0)
        .cloned()
        .collect();
    let partials: Vec<_> = integrations
        .iter()
        .filter(|integration| integration.partial_or_ambiguous)
        .cloned()
        .collect();

    for partial in &partials {
        eprintln!(
            "me: possible partial me integration in {}; manual cleanup required",
            display_path(&partial.path)
        );
        if let Ok(contents) = read_optional(&partial.path)
            && let Some(snippet) = partial_me_snippet(&contents)
        {
            eprintln!("me: snippet:\n{snippet}");
        }
    }

    if removable.is_empty() {
        println!("me uninstall: no managed integration blocks found");
        return Ok(());
    }

    if !args.non_interactive {
        println!("me uninstall will remove managed blocks from:");
        for integration in &removable {
            println!("- {}", display_path(&integration.path));
        }
        if !confirm("Remove these blocks? [y/N] ")? {
            println!("aborted");
            return Ok(());
        }
    }

    let mut removed_blocks = 0;
    let mut affected_files = BTreeSet::new();
    for integration in removable {
        let existing = read_optional(&integration.path)?;
        let (updated, removed) = remove_full_blocks(&existing);
        if removed > 0 {
            write_file(&integration.path, &updated)?;
            removed_blocks += removed;
            affected_files.insert(integration.path.clone());
            println!(
                "me uninstall: removed {} block(s) from {}",
                removed,
                display_path(&integration.path)
            );
        }
    }
    println!("Removed {} managed block(s).", removed_blocks);
    println!(
        "Removed me shell integration from {} file(s).",
        affected_files.len()
    );
    println!("Restart or reopen your shell to finish uninstalling.");
    Ok(())
}

fn print_install_summary(shell: Shell, summary: &[(TargetRole, &str, PathBuf)]) {
    println!();
    println!("Installed me shell integration for {}", shell.name());
    for (role, mode, path) in summary {
        let label = match role {
            TargetRole::Login => "login",
            TargetRole::Interactive => "interactive",
        };
        println!("- {label}: {mode} ({})", display_path(path));
    }
    println!();
    println!("Restart or reopen your shell to start using it.");
}

fn choose_login_mode(args: &InstallArgs) -> anyhow::Result<LoginMode> {
    if args.non_interactive {
        return Ok(args.login.unwrap_or(LoginMode::Full));
    }
    if let Some(mode) = args.login {
        return Ok(mode);
    }
    prompt_login_mode()
}

fn choose_interactive_mode(args: &InstallArgs) -> anyhow::Result<InteractiveMode> {
    if args.non_interactive {
        return Ok(args.interactive.unwrap_or(InteractiveMode::None));
    }
    if let Some(mode) = args.interactive {
        return Ok(mode);
    }
    prompt_interactive_mode()
}

fn prompt_login_mode() -> anyhow::Result<LoginMode> {
    println!("How should me run in login shells?");
    println!("1. full (recommended)");
    println!("2. compact");
    println!("3. none");
    match prompt("Choose [1]: ")?.trim() {
        "" | "1" | "full" => Ok(LoginMode::Full),
        "2" | "compact" => Ok(LoginMode::Compact),
        "3" | "none" => Ok(LoginMode::None),
        value => bail!("unknown login mode: {value}"),
    }
}

fn prompt_interactive_mode() -> anyhow::Result<InteractiveMode> {
    println!("How should me run in interactive shells?");
    println!("1. compact (recommended)");
    println!("2. prompt");
    println!("3. none");
    match prompt("Choose [1]: ")?.trim() {
        "" | "1" | "compact" => Ok(InteractiveMode::Compact),
        "2" | "prompt" => Ok(InteractiveMode::Prompt),
        "3" | "none" => Ok(InteractiveMode::None),
        value => bail!("unknown interactive mode: {value}"),
    }
}

fn prompt_cross_shell_choice(
    shell: Shell,
    integrations: &[ExistingIntegration],
) -> anyhow::Result<CrossShellChoice> {
    println!("Detected existing me integration in:");
    for integration in integrations {
        println!("- {}", display_path(&integration.path));
    }
    println!("Current shell: {}", shell.name());
    println!("1. Install for current shell only (keep existing)");
    println!("2. Install for current shell and remove old shell integrations");
    println!("3. Abort");
    match prompt("Choose [1]: ")?.trim() {
        "" | "1" => Ok(CrossShellChoice::Keep),
        "2" => Ok(CrossShellChoice::Remove),
        "3" => Ok(CrossShellChoice::Abort),
        value => bail!("unknown choice: {value}"),
    }
}

fn confirm(message: &str) -> anyhow::Result<bool> {
    Ok(matches!(prompt(message)?.trim(), "y" | "Y" | "yes" | "YES"))
}

fn prompt(message: &str) -> anyhow::Result<String> {
    print!("{message}");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input)
}

fn resolve_targets(
    home: &Path,
    config_dir: &Path,
    shell: Shell,
    override_file: Option<&Path>,
    login: LoginMode,
    interactive: InteractiveMode,
) -> Vec<TargetFile> {
    let mut targets: BTreeMap<PathBuf, TargetFile> = BTreeMap::new();
    if login != LoginMode::None {
        let path = override_file
            .map(Path::to_path_buf)
            .unwrap_or_else(|| login_path(home, config_dir, shell));
        targets
            .entry(path.clone())
            .or_insert_with(|| TargetFile {
                shell,
                path,
                roles: BTreeSet::new(),
            })
            .roles
            .insert(TargetRole::Login);
    }
    if interactive != InteractiveMode::None {
        let path = override_file
            .map(Path::to_path_buf)
            .unwrap_or_else(|| interactive_path(home, config_dir, shell));
        targets
            .entry(path.clone())
            .or_insert_with(|| TargetFile {
                shell,
                path,
                roles: BTreeSet::new(),
            })
            .roles
            .insert(TargetRole::Interactive);
    }
    targets.into_values().collect()
}

fn login_path(home: &Path, config_dir: &Path, shell: Shell) -> PathBuf {
    match shell {
        Shell::Zsh => home.join(".zprofile"),
        Shell::Bash => {
            let bash_profile = home.join(".bash_profile");
            let profile = home.join(".profile");
            if bash_profile.exists() || !profile.exists() {
                bash_profile
            } else {
                profile
            }
        }
        Shell::Fish => config_dir.join("fish/config.fish"),
        Shell::Nushell => config_dir.join("nushell/config.nu"),
    }
}

fn interactive_path(home: &Path, config_dir: &Path, shell: Shell) -> PathBuf {
    match shell {
        Shell::Zsh => home.join(".zshrc"),
        Shell::Bash => home.join(".bashrc"),
        Shell::Fish => config_dir.join("fish/config.fish"),
        Shell::Nushell => config_dir.join("nushell/config.nu"),
    }
}

fn known_paths(home: &Path, config_dir: &Path) -> Vec<(Shell, PathBuf)> {
    [
        (Shell::Zsh, home.join(".zprofile")),
        (Shell::Zsh, home.join(".zshrc")),
        (Shell::Bash, home.join(".bash_profile")),
        (Shell::Bash, home.join(".profile")),
        (Shell::Bash, home.join(".bashrc")),
        (Shell::Fish, config_dir.join("fish/config.fish")),
        (Shell::Nushell, config_dir.join("nushell/config.nu")),
    ]
    .into()
}

fn scan_known_integrations(
    home: &Path,
    config_dir: &Path,
) -> anyhow::Result<Vec<ExistingIntegration>> {
    let paths = known_paths(home, config_dir)
        .into_iter()
        .map(|(_, path)| path)
        .collect::<Vec<_>>();
    scan_paths(&paths)
}

fn scan_paths(paths: &[PathBuf]) -> anyhow::Result<Vec<ExistingIntegration>> {
    let mut integrations = Vec::new();
    let mut seen = BTreeSet::new();
    for path in paths {
        if !seen.insert(path.clone()) || !path.exists() {
            continue;
        }
        let contents = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", display_path(path)))?;
        let full_blocks = count_full_blocks(&contents);
        let partial_or_ambiguous = has_partial_or_ambiguous_me_snippet(&contents);
        if full_blocks > 0 || partial_or_ambiguous {
            integrations.push(ExistingIntegration {
                shell: detect_shell_from_contents(&contents)
                    .or_else(|| infer_shell_from_path(path)),
                path: path.clone(),
                full_blocks,
                partial_or_ambiguous,
            });
        }
    }
    Ok(integrations)
}

fn existing_for_other_shells(
    existing: &[ExistingIntegration],
    current_shell: Shell,
) -> Vec<ExistingIntegration> {
    existing
        .iter()
        .filter(|integration| {
            integration
                .shell
                .is_some_and(|shell| shell != current_shell)
        })
        .cloned()
        .collect()
}

fn warn_existing_other_shells(integrations: &[ExistingIntegration]) {
    eprintln!("me: existing me integration found for another shell; leaving it unchanged:");
    for integration in integrations {
        eprintln!("me: - {}", display_path(&integration.path));
    }
}

fn remove_full_blocks_from_integrations(
    integrations: &[ExistingIntegration],
) -> anyhow::Result<()> {
    for integration in integrations {
        let existing = read_optional(&integration.path)?;
        let (updated, removed) = remove_full_blocks(&existing);
        if removed > 0 {
            write_file(&integration.path, &updated)?;
        }
    }
    Ok(())
}

fn render_block(shell: Shell, login: LoginMode, interactive: InteractiveMode) -> String {
    let mut lines = vec![
        START_MARKER.to_string(),
        format!(
            "{SIGNATURE_PREFIX} shell={} login={} interactive={} version=v{}",
            shell.name(),
            login.name(),
            interactive.name(),
            env!("CARGO_PKG_VERSION")
        ),
    ];
    match login {
        LoginMode::None => {}
        LoginMode::Full => lines.push("me".to_string()),
        LoginMode::Compact => lines.push("me --compact".to_string()),
    }
    match interactive {
        InteractiveMode::None => {}
        InteractiveMode::Compact => lines.push("me --compact".to_string()),
        InteractiveMode::Prompt => lines.extend(prompt_lines(shell)),
    }
    lines.push(END_MARKER.to_string());
    lines.join("\n") + "\n"
}

fn prompt_lines(shell: Shell) -> Vec<String> {
    match shell {
        Shell::Zsh => vec![
            "setopt PROMPT_SUBST".to_string(),
            "PROMPT='$(me --compact --fast | tr -d \"\\n\") '${PROMPT:-'%n@%m %~ %# '}".to_string(),
        ],
        Shell::Bash => vec![
            "PS1='$(me --compact --fast | tr -d \"\\n\") '${PS1:-'\\u@\\h:\\w\\$ '}".to_string(),
        ],
        Shell::Fish => vec![
            "function fish_prompt".to_string(),
            "    me --compact --fast | string trim".to_string(),
            "    echo -n ' '".to_string(),
            "end".to_string(),
        ],
        Shell::Nushell => {
            vec!["$env.PROMPT_COMMAND = { || me --compact --fast | str trim }".to_string()]
        }
    }
}

fn upsert_block(existing: &str, block: &str) -> (String, BlockChange) {
    if let Some((start, end)) = find_first_full_block(existing) {
        let mut updated = String::with_capacity(existing.len() + block.len());
        updated.push_str(&existing[..start]);
        updated.push_str(block);
        updated.push_str(&existing[end..]);
        let change = if updated == existing {
            BlockChange::Unchanged
        } else {
            BlockChange::Updated
        };
        return (updated, change);
    }

    let mut updated = existing.to_string();
    if !updated.is_empty() && !updated.ends_with('\n') {
        updated.push('\n');
    }
    if !updated.is_empty() {
        updated.push('\n');
    }
    updated.push_str(block);
    (updated, BlockChange::Added)
}

fn remove_full_blocks(existing: &str) -> (String, usize) {
    let mut remaining = existing;
    let mut updated = String::new();
    let mut removed = 0;

    while let Some(start) = remaining.find(START_MARKER) {
        if let Some(relative_end) = remaining[start..].find(END_MARKER) {
            let end_marker_end = start + relative_end + END_MARKER.len();
            let end = if remaining[end_marker_end..].starts_with('\n') {
                end_marker_end + 1
            } else {
                end_marker_end
            };
            updated.push_str(&remaining[..start]);
            remaining = &remaining[end..];
            removed += 1;
        } else {
            break;
        }
    }
    updated.push_str(remaining);
    (updated, removed)
}

fn find_first_full_block(existing: &str) -> Option<(usize, usize)> {
    let start = existing.find(START_MARKER)?;
    let relative_end = existing[start..].find(END_MARKER)?;
    let end_marker_end = start + relative_end + END_MARKER.len();
    let end = if existing[end_marker_end..].starts_with('\n') {
        end_marker_end + 1
    } else {
        end_marker_end
    };
    Some((start, end))
}

fn count_full_blocks(contents: &str) -> usize {
    let mut count = 0;
    let mut remaining = contents;
    while let Some(start) = remaining.find(START_MARKER) {
        if let Some(relative_end) = remaining[start..].find(END_MARKER) {
            let end = start + relative_end + END_MARKER.len();
            count += 1;
            remaining = &remaining[end..];
        } else {
            break;
        }
    }
    count
}

fn has_partial_or_ambiguous_me_snippet(contents: &str) -> bool {
    let start_count = contents.matches(START_MARKER).count();
    let end_count = contents.matches(END_MARKER).count();
    start_count != end_count
        || (contents.contains(SIGNATURE_PREFIX) && count_full_blocks(contents) == 0)
        || (!contents.contains(START_MARKER)
            && !contents.contains(SIGNATURE_PREFIX)
            && (contents.contains("me --compact --fast")
                || contents.contains("me --compact")
                || contents.lines().any(|line| line.trim() == "me")))
}

fn partial_me_snippet(contents: &str) -> Option<String> {
    let lines: Vec<_> = contents.lines().collect();
    let start = lines.iter().position(|line| {
        line.contains(START_MARKER)
            || line.contains(END_MARKER)
            || line.contains(SIGNATURE_PREFIX)
            || line.contains("me --compact --fast")
            || line.contains("me --compact")
            || line.trim() == "me"
    })?;
    let end = (start + 6).min(lines.len());
    Some(lines[start..end].join("\n"))
}

fn detect_shell_from_contents(contents: &str) -> Option<Shell> {
    for line in contents.lines() {
        let line = line.trim();
        if !line.starts_with(SIGNATURE_PREFIX) {
            continue;
        }
        for part in line.split_whitespace() {
            if let Some(value) = part.strip_prefix("shell=") {
                return Shell::from_name(value);
            }
        }
    }
    None
}

fn infer_shell_from_path(path: &Path) -> Option<Shell> {
    let filename = path.file_name()?.to_string_lossy();
    match filename.as_ref() {
        ".zprofile" | ".zshrc" => Some(Shell::Zsh),
        ".bash_profile" | ".bashrc" | ".profile" => Some(Shell::Bash),
        "config.fish" => Some(Shell::Fish),
        "config.nu" => Some(Shell::Nushell),
        _ => None,
    }
}

fn prompt_integration_looks_unsafe(shell: Shell, path: &Path) -> bool {
    let Ok(contents) = fs::read_to_string(path) else {
        return false;
    };
    looks_complex_prompt(shell, &contents)
}

fn looks_complex_prompt(shell: Shell, contents: &str) -> bool {
    let shared = [
        "starship init",
        "oh-my-zsh",
        "powerlevel10k",
        "p10k",
        "oh-my-posh",
        "atuin init",
    ];
    if shared.iter().any(|needle| contents.contains(needle)) {
        return true;
    }
    match shell {
        Shell::Zsh => contents.contains("PROMPT=") || contents.contains("RPROMPT="),
        Shell::Bash => contents.contains("PS1=") || contents.contains("PROMPT_COMMAND"),
        Shell::Fish => contents.contains("fish_prompt"),
        Shell::Nushell => {
            contents.contains("PROMPT_COMMAND") || contents.contains("PROMPT_INDICATOR")
        }
    }
}

fn detect_current_shell() -> Option<Shell> {
    let shell = env::var("SHELL").ok()?;
    let name = Path::new(&shell).file_name()?.to_string_lossy();
    Shell::from_name(&name)
}

fn shell_home_dir() -> Option<PathBuf> {
    env_path("HOME").or_else(dirs::home_dir)
}

fn shell_config_dir(home: &Path) -> PathBuf {
    env_path("XDG_CONFIG_HOME")
        .or_else(dirs::config_dir)
        .unwrap_or_else(|| home.join(".config"))
}

fn env_path(key: &str) -> Option<PathBuf> {
    let value = env::var_os(key)?;
    if value.is_empty() {
        None
    } else {
        Some(PathBuf::from(value))
    }
}

fn read_optional(path: &Path) -> anyhow::Result<String> {
    match fs::read_to_string(path) {
        Ok(contents) => Ok(contents),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(error).with_context(|| format!("failed to read {}", display_path(path))),
    }
}

fn write_file(path: &Path, contents: &str) -> anyhow::Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", display_path(parent)))?;
    }
    fs::write(path, contents).with_context(|| format!("failed to write {}", display_path(path)))
}

fn display_path(path: &Path) -> String {
    if let Some(home) = shell_home_dir()
        && let Ok(stripped) = path.strip_prefix(&home)
    {
        return format!("~/{}", stripped.display());
    }
    path.display().to_string()
}

trait ShellExt {
    fn name(self) -> &'static str;
    fn from_name(name: &str) -> Option<Shell>;
}

impl ShellExt for Shell {
    fn name(self) -> &'static str {
        match self {
            Shell::Zsh => "zsh",
            Shell::Bash => "bash",
            Shell::Fish => "fish",
            Shell::Nushell => "nushell",
        }
    }

    fn from_name(name: &str) -> Option<Shell> {
        match name {
            "zsh" => Some(Shell::Zsh),
            "bash" => Some(Shell::Bash),
            "fish" => Some(Shell::Fish),
            "nu" | "nushell" => Some(Shell::Nushell),
            _ => None,
        }
    }
}

trait LoginModeExt {
    fn name(self) -> &'static str;
}

impl LoginModeExt for LoginMode {
    fn name(self) -> &'static str {
        match self {
            LoginMode::None => "none",
            LoginMode::Full => "full",
            LoginMode::Compact => "compact",
        }
    }
}

trait InteractiveModeExt {
    fn name(self) -> &'static str;
}

impl InteractiveModeExt for InteractiveMode {
    fn name(self) -> &'static str {
        match self {
            InteractiveMode::None => "none",
            InteractiveMode::Compact => "compact",
            InteractiveMode::Prompt => "prompt",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn zsh_targets_split_login_and_interactive_files() {
        let home = PathBuf::from("/tmp/home");
        let config = home.join(".config");

        let targets = resolve_targets(
            &home,
            &config,
            Shell::Zsh,
            None,
            LoginMode::Full,
            InteractiveMode::Compact,
        );

        assert_eq!(targets.len(), 2);
        assert!(
            targets
                .iter()
                .any(|target| target.path == home.join(".zprofile")
                    && target.roles.contains(&TargetRole::Login))
        );
        assert!(
            targets
                .iter()
                .any(|target| target.path == home.join(".zshrc")
                    && target.roles.contains(&TargetRole::Interactive))
        );
    }

    #[test]
    fn override_file_combines_roles() {
        let home = PathBuf::from("/tmp/home");
        let config = home.join(".config");
        let file = home.join(".customrc");

        let targets = resolve_targets(
            &home,
            &config,
            Shell::Zsh,
            Some(&file),
            LoginMode::Full,
            InteractiveMode::Compact,
        );

        assert_eq!(targets.len(), 1);
        assert!(targets[0].roles.contains(&TargetRole::Login));
        assert!(targets[0].roles.contains(&TargetRole::Interactive));
    }

    #[test]
    fn bash_login_uses_profile_when_bash_profile_is_absent() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join(".profile"), "profile\n").unwrap();

        assert_eq!(
            login_path(dir.path(), &dir.path().join(".config"), Shell::Bash),
            dir.path().join(".profile")
        );
    }

    #[test]
    fn fish_and_nushell_use_config_dir() {
        let home = PathBuf::from("/tmp/home");
        let config = PathBuf::from("/tmp/xdg");

        assert_eq!(
            login_path(&home, &config, Shell::Fish),
            config.join("fish/config.fish")
        );
        assert_eq!(
            interactive_path(&home, &config, Shell::Nushell),
            config.join("nushell/config.nu")
        );
    }

    #[test]
    fn block_upsert_is_idempotent() {
        let first = render_block(Shell::Zsh, LoginMode::Full, InteractiveMode::Compact);
        let second = render_block(Shell::Zsh, LoginMode::Full, InteractiveMode::None);

        let (contents, change) = upsert_block("before\n", &first);
        assert_eq!(change, BlockChange::Added);
        let (contents, change) = upsert_block(&contents, &second);

        assert_eq!(change, BlockChange::Updated);
        assert_eq!(contents.matches(START_MARKER).count(), 1);
        assert!(contents.contains("interactive=none"));
        assert!(!contents.contains("interactive=compact"));
    }

    #[test]
    fn remove_full_block_keeps_unmanaged_lines() {
        let block = render_block(Shell::Zsh, LoginMode::Full, InteractiveMode::Compact);
        let (updated, removed) = remove_full_blocks(&format!("before\n{block}after\n"));

        assert_eq!(removed, 1);
        assert_eq!(updated, "before\nafter\n");
    }

    #[test]
    fn partial_signature_is_not_removed_as_full_block() {
        let contents =
            "# me-managed: shell=zsh login=full interactive=compact version=v0.3.2\nme --compact\n";

        let (updated, removed) = remove_full_blocks(contents);

        assert_eq!(removed, 0);
        assert_eq!(updated, contents);
        assert!(has_partial_or_ambiguous_me_snippet(contents));
    }

    #[test]
    fn prompt_safety_detects_existing_prompt_tools() {
        assert!(looks_complex_prompt(
            Shell::Zsh,
            "eval \"$(starship init zsh)\"\n"
        ));
        assert!(looks_complex_prompt(Shell::Bash, "PS1='demo'\n"));
        assert!(!looks_complex_prompt(Shell::Zsh, "alias ll='ls -la'\n"));
    }

    #[test]
    fn scan_detects_other_shell_from_signature() {
        let dir = tempdir().unwrap();
        let bashrc = dir.path().join(".bashrc");
        fs::write(
            &bashrc,
            "# >>> me install >>>\n# me-managed: shell=bash login=full interactive=none version=v0.3.2\nme\n# <<< me install <<<\n",
        )
        .unwrap();

        let integrations = scan_paths(&[bashrc]).unwrap();

        assert_eq!(integrations.len(), 1);
        assert_eq!(integrations[0].shell, Some(Shell::Bash));
        assert_eq!(integrations[0].full_blocks, 1);
    }

    #[test]
    fn scan_infers_shell_from_known_path_when_signature_is_missing() {
        let dir = tempdir().unwrap();
        let zshrc = dir.path().join(".zshrc");
        fs::write(
            &zshrc,
            "# >>> me install >>>\nme --compact\n# <<< me install <<<\n",
        )
        .unwrap();

        let integrations = scan_paths(&[zshrc]).unwrap();

        assert_eq!(integrations.len(), 1);
        assert_eq!(integrations[0].shell, Some(Shell::Zsh));
    }
}
