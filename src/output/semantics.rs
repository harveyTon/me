use crate::{
    context::project::project_priority,
    model::{GitContext, MeInfo, ProjectContext},
};

const MAX_TEXT_CONTEXT_ITEMS: usize = 3;
const MAX_NODE_DETAILS: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextTextStyle {
    Block,
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

pub fn compact_project_parts(info: &MeInfo) -> Vec<String> {
    let mut summary = project_related_items(info, ContextDisplayStyle::Compact);
    if summary.overflow > 0
        && let Some(last) = summary.visible.last_mut()
    {
        last.push_str(&format!(" (+{})", summary.overflow));
    }
    summary.visible
}

fn container_text(info: &MeInfo, style: ContextTextStyle) -> Option<String> {
    let container = info.context.container.as_ref()?;
    Some(match style {
        ContextTextStyle::Config => match &container.id {
            Some(id) => format!("{}:{id}", container.kind),
            None => container.kind.clone(),
        },
        ContextTextStyle::Block => container.kind.clone(),
    })
}

fn project_group_text(info: &MeInfo, style: ContextTextStyle) -> Option<String> {
    let summary = project_related_items(
        info,
        match style {
            ContextTextStyle::Block => ContextDisplayStyle::Block,
            ContextTextStyle::Config => ContextDisplayStyle::Config,
        },
    );

    if summary.visible.is_empty() {
        return None;
    }

    Some(match style {
        ContextTextStyle::Block => join_visible_items(&summary.visible, summary.overflow, " · "),
        ContextTextStyle::Config => summary.visible.join(", "),
    })
}

fn project_text(project: &ProjectContext, style: ContextDisplayStyle) -> String {
    let mut value = match &project.version {
        Some(version) => format!("{} {version}", project.kind),
        None => project.kind.clone(),
    };
    let details = project_details(project, style);
    if !details.is_empty() {
        value.push_str(&format!(" ({})", details.join(", ")));
    }
    value
}

fn project_text_items(projects: &[ProjectContext], style: ContextDisplayStyle) -> Vec<String> {
    let mut ordered: Vec<&ProjectContext> = projects.iter().collect();
    ordered.sort_by(|left, right| {
        project_priority(&left.kind)
            .cmp(&project_priority(&right.kind))
            .then_with(|| left.kind.cmp(&right.kind))
    });
    ordered
        .into_iter()
        .map(|project| project_text(project, style))
        .collect()
}

fn project_details(project: &ProjectContext, style: ContextDisplayStyle) -> Vec<String> {
    if project.kind == "node" && style.limits_density() {
        return project
            .details
            .iter()
            .take(MAX_NODE_DETAILS)
            .cloned()
            .collect();
    }
    project.details.clone()
}

enum CompactGitStyle {
    Block,
    Compact,
    Config,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContextDisplayStyle {
    Block,
    Compact,
    Config,
}

impl ContextDisplayStyle {
    fn limits_density(self) -> bool {
        matches!(self, Self::Block | Self::Compact)
    }
}

struct ContextItemSummary {
    visible: Vec<String>,
    overflow: usize,
}

fn project_related_items(info: &MeInfo, style: ContextDisplayStyle) -> ContextItemSummary {
    let mut items = project_text_items(&info.context.projects, style);
    if let Some(git) = &info.context.git {
        items.push(git_text(
            git,
            match style {
                ContextDisplayStyle::Block => CompactGitStyle::Block,
                ContextDisplayStyle::Compact => CompactGitStyle::Compact,
                ContextDisplayStyle::Config => CompactGitStyle::Config,
            },
        ));
    }

    if !style.limits_density() || items.len() <= MAX_TEXT_CONTEXT_ITEMS {
        return ContextItemSummary {
            visible: items,
            overflow: 0,
        };
    }

    let overflow = items.len() - MAX_TEXT_CONTEXT_ITEMS;
    items.truncate(MAX_TEXT_CONTEXT_ITEMS);
    ContextItemSummary {
        visible: items,
        overflow,
    }
}

fn join_visible_items(items: &[String], overflow: usize, separator: &str) -> String {
    let mut joined = items.join(separator);
    if overflow > 0 {
        joined.push_str(&format!(" (+{})", overflow));
    }
    joined
}

fn git_text(git: &GitContext, style: CompactGitStyle) -> String {
    match style {
        CompactGitStyle::Block => format!("git({})", git.branch),
        CompactGitStyle::Compact | CompactGitStyle::Config => format!("git:{}", git.branch),
    }
}
