use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Field {
    User,
    Uid,
    Gid,
    Groups,
    Host,
    Shell,
    Pid,
    Ppid,
    Tty,
    Privilege,
    Sudo,
    Ssh,
    Network,
    Pwd,
    Context,
}

impl Field {
    pub fn defaults() -> Vec<Self> {
        vec![
            Self::User,
            Self::Uid,
            Self::Gid,
            Self::Groups,
            Self::Host,
            Self::Shell,
            Self::Pid,
            Self::Ppid,
            Self::Tty,
            Self::Privilege,
            Self::Sudo,
            Self::Ssh,
            Self::Network,
            Self::Pwd,
            Self::Context,
        ]
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Uid => "uid",
            Self::Gid => "gid",
            Self::Groups => "groups",
            Self::Host => "host",
            Self::Shell => "shell",
            Self::Pid => "pid",
            Self::Ppid => "ppid",
            Self::Tty => "tty",
            Self::Privilege => "privilege",
            Self::Sudo => "sudo",
            Self::Ssh => "ssh",
            Self::Network => "network",
            Self::Pwd => "pwd",
            Self::Context => "context",
        }
    }
}

impl FromStr for Field {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().replace('-', "_").as_str() {
            "user" => Ok(Self::User),
            "uid" => Ok(Self::Uid),
            "gid" => Ok(Self::Gid),
            "groups" | "group" => Ok(Self::Groups),
            "host" | "hostname" => Ok(Self::Host),
            "shell" => Ok(Self::Shell),
            "pid" => Ok(Self::Pid),
            "ppid" => Ok(Self::Ppid),
            "tty" => Ok(Self::Tty),
            "privilege" => Ok(Self::Privilege),
            "sudo" => Ok(Self::Sudo),
            "ssh" => Ok(Self::Ssh),
            "network" | "ip" => Ok(Self::Network),
            "pwd" | "cwd" => Ok(Self::Pwd),
            "context" => Ok(Self::Context),
            other => Err(format!("unknown field '{other}'")),
        }
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.key())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemIdentity {
    pub user: String,
    pub uid: u32,
    pub gid: u32,
    pub groups: Vec<String>,
    pub host: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeInfo {
    pub shell: Option<String>,
    pub pid: u32,
    pub ppid: Option<u32>,
    pub tty: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct NetworkInfo {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ipv4_local_ips: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ipv6_local_ips: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PwdInfo {
    pub raw: String,
    pub display: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SshContext {
    pub remote: bool,
    pub connection: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContainerContext {
    pub kind: String,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectContext {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_count: Option<usize>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub details: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GitContext {
    pub branch: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ContextInfo {
    pub ssh: Option<SshContext>,
    pub container: Option<ContainerContext>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub projects: Vec<ProjectContext>,
    pub git: Option<GitContext>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MeInfo {
    pub identity: SystemIdentity,
    pub runtime: RuntimeInfo,
    pub privilege: String,
    pub sudo: bool,
    pub ssh: bool,
    pub network: NetworkInfo,
    pub pwd: Option<PwdInfo>,
    pub context: ContextInfo,
}
