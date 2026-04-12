use crate::model::{GitContext, MeInfo, ProjectContext};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextTextStyle {
    Block,
    Compact,
    Config,
}

pub fn display_host(host: &str) -> String {
    host.strip_suffix(".local").unwrap_or(host).to_owned()
}

pub fn compact_list(values: &[String], keep: usize) -> Option<String> {
    if values.is_empty() {
        return None;
    }
    Some(if values.len() <= keep {
        values.join(", ")
    } else {
        format!("{} (+{})", values[..keep].join(", "), values.len() - keep)
    })
}

pub fn context_text(info: &MeInfo, style: ContextTextStyle) -> Option<String> {
    let mut sections = Vec::new();
    if let Some(container) = container_text(info, style) {
        sections.push(container);
    }
    if let Some(project_group) = project_group_text(info, style) {
        sections.push(project_group);
    }
    Some(sections.join(", ")).filter(|value| !value.is_empty())
}

pub fn compact_env_label(info: &MeInfo) -> String {
    if info.ssh {
        "ssh".into()
    } else if let Some(container) = &info.context.container {
        container.kind.clone()
    } else {
        "local".into()
    }
}

pub fn compact_cwd_name(info: &MeInfo) -> Option<String> {
    let raw = info.pwd.as_ref()?.raw.as_str();
    std::path::Path::new(raw)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
}

pub fn display_user(info: &MeInfo) -> &str {
    if info.identity.uid == 0 {
        "root"
    } else {
        &info.identity.user
    }
}

fn container_text(info: &MeInfo, style: ContextTextStyle) -> Option<String> {
    if matches!(style, ContextTextStyle::Compact) {
        return None;
    }
    let container = info.context.container.as_ref()?;
    Some(match style {
        ContextTextStyle::Config => match &container.id {
            Some(id) => format!("{}:{id}", container.kind),
            None => container.kind.clone(),
        },
        ContextTextStyle::Block | ContextTextStyle::Compact => container.kind.clone(),
    })
}

fn project_group_text(info: &MeInfo, style: ContextTextStyle) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(project) = &info.context.project {
        parts.push(project_text(project));
    }
    if let Some(git) = &info.context.git {
        parts.push(git_text(git, style));
    }
    Some(parts.join(project_git_separator(style))).filter(|value| !value.is_empty())
}

fn project_text(project: &ProjectContext) -> String {
    match &project.version {
        Some(version) => format!("{} {version}", project.kind),
        None => project.kind.clone(),
    }
}

fn git_text(git: &GitContext, style: ContextTextStyle) -> String {
    match style {
        ContextTextStyle::Block => format!("git({})", git.branch),
        ContextTextStyle::Compact | ContextTextStyle::Config => format!("git:{}", git.branch),
    }
}

fn project_git_separator(style: ContextTextStyle) -> &'static str {
    match style {
        ContextTextStyle::Block => " · ",
        ContextTextStyle::Compact => " ",
        ContextTextStyle::Config => ", ",
    }
}
