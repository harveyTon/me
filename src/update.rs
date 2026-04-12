use crate::cli::UpdateArgs;
use anyhow::{Context, bail};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::Command,
};

const OWNER: &str = "harveyTon";
const REPO: &str = "me";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, PartialEq, Eq)]
enum InstallSource {
    Homebrew,
    ReleaseBinary(PathBuf),
    Unknown(PathBuf),
}

impl InstallSource {
    fn label(&self) -> &'static str {
        match self {
            Self::Homebrew => "homebrew",
            Self::ReleaseBinary(_) => "release binary",
            Self::Unknown(_) => "unknown",
        }
    }

    fn path(&self) -> Option<&Path> {
        match self {
            Self::Homebrew => None,
            Self::ReleaseBinary(path) | Self::Unknown(path) => Some(path),
        }
    }
}

pub fn run(args: UpdateArgs) -> anyhow::Result<()> {
    let current = clean_version(CURRENT_VERSION);
    let latest = latest_version().context("could not determine latest release version")?;
    let source = detect_source();

    println!("me update");
    println!("current: {current}");
    println!("latest: {latest}");
    println!("install source: {}", source.label());
    if let Some(path) = source.path() {
        println!("path: {}", path.display());
    }

    if !is_newer(&latest, &current) {
        println!("me is already up to date.");
        return Ok(());
    }

    println!("update available: {current} -> {latest}");
    if args.check {
        return Ok(());
    }

    if !args.non_interactive && !confirm("Upgrade now? [Y/n] ")? {
        println!("aborted");
        return Ok(());
    }

    match source {
        InstallSource::Homebrew => update_homebrew(),
        InstallSource::ReleaseBinary(path) => update_release_binary(&path, &latest),
        InstallSource::Unknown(_) => {
            bail!("unsupported or unknown install source; please update me manually")
        }
    }
}

fn latest_version() -> anyhow::Result<String> {
    if let Ok(version) = env::var("ME_UPDATE_LATEST_VERSION") {
        return Ok(clean_version(&version));
    }

    let url = format!("https://github.com/{OWNER}/{REPO}/releases/latest");
    let output = Command::new("curl")
        .args(["-fsSL", "-o", "/dev/null", "-w", "%{url_effective}", &url])
        .output()
        .context("failed to run curl")?;
    if !output.status.success() {
        bail!("curl failed while resolving the latest release");
    }

    let effective = String::from_utf8(output.stdout).context("latest release URL was not UTF-8")?;
    let version = effective
        .trim()
        .rsplit('/')
        .next()
        .filter(|value| !value.is_empty())
        .context("latest release URL did not include a version")?;
    Ok(clean_version(version))
}

fn detect_source() -> InstallSource {
    let exe = env::var_os("ME_UPDATE_EXE")
        .map(PathBuf::from)
        .or_else(|| env::current_exe().ok())
        .unwrap_or_else(|| PathBuf::from("me"));

    if let Ok(source) = env::var("ME_UPDATE_SOURCE") {
        return match source.as_str() {
            "homebrew" => InstallSource::Homebrew,
            "release" => InstallSource::ReleaseBinary(exe),
            _ => InstallSource::Unknown(exe),
        };
    }

    detect_source_for(&exe, env::consts::OS, brew_prefix_for("me").as_deref())
}

fn detect_source_for(exe: &Path, os: &str, brew_prefix: Option<&Path>) -> InstallSource {
    if os == "macos" && is_homebrew_candidate(exe, brew_prefix) {
        return InstallSource::Homebrew;
    }

    if looks_like_release_binary_for(exe, os) {
        InstallSource::ReleaseBinary(exe.to_path_buf())
    } else {
        InstallSource::Unknown(exe.to_path_buf())
    }
}

fn brew_prefix_for(formula: &str) -> Option<PathBuf> {
    let output = Command::new("brew")
        .args(["--prefix", formula])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let prefix = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if prefix.is_empty() {
        None
    } else {
        Some(PathBuf::from(prefix))
    }
}

fn is_homebrew_candidate(exe: &Path, brew_prefix: Option<&Path>) -> bool {
    let canonical = fs::canonicalize(exe).unwrap_or_else(|_| exe.to_path_buf());
    is_homebrew_path(exe, &canonical, brew_prefix)
}

fn is_homebrew_path(exe: &Path, canonical: &Path, brew_prefix: Option<&Path>) -> bool {
    if matches_homebrew_layout(exe) || matches_homebrew_layout(canonical) {
        return true;
    }

    if let Some(prefix) = brew_prefix {
        let bin = prefix.join("bin").join(binary_name());
        let opt = prefix.join("opt").join("me");
        let cellar = prefix.join("Cellar").join("me");
        if exe == bin || canonical == bin {
            return true;
        }
        if exe.starts_with(&opt)
            || canonical.starts_with(&opt)
            || exe.starts_with(&cellar)
            || canonical.starts_with(&cellar)
        {
            return true;
        }
    }

    false
}

fn matches_homebrew_layout(path: &Path) -> bool {
    let path = path.to_string_lossy();
    path == "/opt/homebrew/bin/me"
        || path == "/usr/local/bin/me"
        || path.starts_with("/opt/homebrew/Cellar/me/")
        || path.starts_with("/usr/local/Cellar/me/")
        || path.starts_with("/opt/homebrew/opt/me/")
        || path.starts_with("/usr/local/opt/me/")
}

fn binary_name() -> &'static str {
    if cfg!(windows) { "me.exe" } else { "me" }
}

fn looks_like_release_binary(exe: &Path) -> bool {
    let Some(file_name) = exe.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if file_name != "me" && file_name != "me.exe" {
        return false;
    }
    let path = exe.to_string_lossy();
    !path.contains("/target/") && !path.contains("\\target\\") && !path.contains(".cargo")
}

fn looks_like_release_binary_for(exe: &Path, os: &str) -> bool {
    if !looks_like_release_binary(exe) {
        return false;
    }

    if os != "macos" {
        return true;
    }

    let path = exe.to_string_lossy();
    if path.ends_with("/.local/bin/me") {
        return true;
    }

    if path == "/usr/local/bin/me" {
        return fs::symlink_metadata(exe)
            .map(|metadata| !metadata.file_type().is_symlink())
            .unwrap_or(false);
    }

    false
}

fn update_homebrew() -> anyhow::Result<()> {
    if env::var_os("ME_UPDATE_DRY_RUN").is_some() {
        println!("would run: brew update");
        println!("would run: brew upgrade me");
        println!("updated via Homebrew");
        return Ok(());
    }

    run_command("brew", &["update"])?;
    run_command("brew", &["upgrade", "me"])?;
    println!("updated via Homebrew");
    Ok(())
}

fn update_release_binary(path: &Path, latest: &str) -> anyhow::Result<()> {
    let temp_dir = env::temp_dir().join(format!("me-update-{}", std::process::id()));
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).with_context(|| {
            format!("failed to clear temporary directory {}", temp_dir.display())
        })?;
    }
    fs::create_dir_all(&temp_dir).with_context(|| {
        format!(
            "failed to create temporary directory {}",
            temp_dir.display()
        )
    })?;

    let candidate = if let Ok(artifact) = env::var("ME_UPDATE_RELEASE_ARTIFACT") {
        let artifact = PathBuf::from(artifact);
        if let Some(manifest) = checksum_manifest_override()? {
            verify_checksum_manifest(&artifact, &manifest)?;
        }
        artifact
    } else {
        download_release_binary(&temp_dir, latest)?
    };

    replace_binary(path, &candidate)?;
    let _ = fs::remove_dir_all(&temp_dir);
    println!("updated release binary at {}", path.display());
    Ok(())
}

fn download_release_binary(temp_dir: &Path, latest: &str) -> anyhow::Result<PathBuf> {
    let tag = format!("v{latest}");
    let artifact = artifact_name(&tag).context("no release artifact for this platform")?;
    let archive = temp_dir.join(&artifact);
    let checksum_manifest = download_checksum_manifest(temp_dir, &tag)?;
    let url = format!("https://github.com/{OWNER}/{REPO}/releases/download/{tag}/{artifact}");
    run_command(
        "curl",
        &[
            "-fsSL",
            "-o",
            archive
                .to_str()
                .context("temporary archive path is not valid UTF-8")?,
            &url,
        ],
    )?;
    verify_checksum_manifest(&archive, &checksum_manifest)?;

    if artifact.ends_with(".zip") {
        extract_zip(&archive, temp_dir)?;
    } else {
        run_command(
            "tar",
            &[
                "-xzf",
                archive
                    .to_str()
                    .context("temporary archive path is not valid UTF-8")?,
                "-C",
                temp_dir
                    .to_str()
                    .context("temporary directory path is not valid UTF-8")?,
            ],
        )?;
    }

    find_binary(temp_dir).context("release archive did not contain a me binary")
}

fn checksum_manifest_override() -> anyhow::Result<Option<String>> {
    if let Ok(path) = env::var("ME_UPDATE_RELEASE_CHECKSUM_FILE") {
        return fs::read_to_string(&path)
            .map(Some)
            .with_context(|| format!("failed to read checksum manifest {path}"));
    }
    Ok(env::var("ME_UPDATE_RELEASE_CHECKSUMS").ok())
}

fn download_checksum_manifest(temp_dir: &Path, tag: &str) -> anyhow::Result<String> {
    let path = temp_dir.join("SHA256SUMS.txt");
    let url = format!("https://github.com/{OWNER}/{REPO}/releases/download/{tag}/SHA256SUMS.txt");
    run_command(
        "curl",
        &[
            "-fsSL",
            "-o",
            path.to_str()
                .context("temporary checksum path is not valid UTF-8")?,
            &url,
        ],
    )?;
    fs::read_to_string(&path)
        .with_context(|| format!("failed to read checksum manifest {}", path.display()))
}

fn extract_zip(archive: &Path, temp_dir: &Path) -> anyhow::Result<()> {
    #[cfg(windows)]
    {
        run_command(
            "powershell",
            &[
                "-NoProfile",
                "-Command",
                &format!(
                    "Expand-Archive -LiteralPath '{}' -DestinationPath '{}' -Force",
                    archive.display(),
                    temp_dir.display()
                ),
            ],
        )
    }
    #[cfg(not(windows))]
    {
        run_command(
            "unzip",
            &[
                "-q",
                archive
                    .to_str()
                    .context("temporary archive path is not valid UTF-8")?,
                "-d",
                temp_dir
                    .to_str()
                    .context("temporary directory path is not valid UTF-8")?,
            ],
        )
    }
}

fn find_binary(root: &Path) -> Option<PathBuf> {
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        let entries = fs::read_dir(path).ok()?;
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if matches!(
                path.file_name().and_then(|name| name.to_str()),
                Some("me") | Some("me.exe")
            ) {
                return Some(path);
            }
        }
    }
    None
}

fn replace_binary(current: &Path, candidate: &Path) -> anyhow::Result<()> {
    let parent = current
        .parent()
        .context("current executable path has no parent directory")?;
    let file_name = current
        .file_name()
        .and_then(|name| name.to_str())
        .context("current executable path has no valid file name")?;
    let staged = parent.join(format!(".{file_name}.me-update-{}", std::process::id()));
    fs::copy(candidate, &staged).with_context(|| {
        format!(
            "failed to stage replacement binary from {}",
            candidate.display()
        )
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = fs::metadata(current) {
            let mode = metadata.permissions().mode();
            fs::set_permissions(&staged, fs::Permissions::from_mode(mode))?;
        }
    }

    let backup = parent.join(format!(".{file_name}.me-backup-{}", std::process::id()));
    if current.exists()
        && let Err(error) = fs::rename(current, &backup)
    {
        let _ = fs::remove_file(&staged);
        return Err(error)
            .with_context(|| format!("failed to stage backup for {}", current.display()));
    }
    if let Err(error) = fs::rename(&staged, current) {
        if backup.exists() {
            let _ = fs::rename(&backup, current);
        }
        let _ = fs::remove_file(&staged);
        return Err(error).with_context(|| format!("failed to replace {}", current.display()));
    }
    let _ = fs::remove_file(&backup);
    Ok(())
}

fn run_command(program: &str, args: &[&str]) -> anyhow::Result<()> {
    let status = Command::new(program)
        .args(args)
        .status()
        .with_context(|| format!("failed to run {program}"))?;
    if status.success() {
        Ok(())
    } else {
        bail!("{program} exited with {status}")
    }
}

fn verify_checksum_manifest(artifact: &Path, manifest: &str) -> anyhow::Result<()> {
    let file_name = artifact
        .file_name()
        .and_then(|name| name.to_str())
        .context("artifact path has no valid file name")?;
    let expected = manifest
        .lines()
        .find_map(|line| {
            let mut parts = line.split_whitespace();
            let checksum = parts.next()?;
            let name = parts.next()?.trim_start_matches("./");
            (name == file_name).then_some(checksum)
        })
        .context("checksum manifest did not include the downloaded artifact")?;
    let actual = sha256_hex(artifact)?;
    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        bail!("checksum verification failed for {}", artifact.display())
    }
}

fn sha256_hex(path: &Path) -> anyhow::Result<String> {
    let mut file = fs::File::open(path).with_context(|| {
        format!(
            "failed to open {} for checksum verification",
            path.display()
        )
    })?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn artifact_name(tag: &str) -> Option<String> {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;
    match (os, arch) {
        ("macos", "aarch64") => Some(format!("me-{tag}-macos-arm64.tar.gz")),
        ("macos", "x86_64") => Some(format!("me-{tag}-macos-x64.tar.gz")),
        ("linux", "x86_64") => Some(format!("me-{tag}-linux-x64.tar.gz")),
        ("linux", "aarch64") => Some(format!("me-{tag}-linux-arm64.tar.gz")),
        ("windows", "x86_64") => Some(format!("me-{tag}-windows-x64.zip")),
        ("windows", "aarch64") => Some(format!("me-{tag}-windows-arm64.zip")),
        _ => None,
    }
}

fn confirm(prompt: &str) -> anyhow::Result<bool> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    let answer = answer.trim().to_ascii_lowercase();
    Ok(answer.is_empty() || answer == "y" || answer == "yes")
}

fn clean_version(version: &str) -> String {
    version.trim().trim_start_matches('v').to_owned()
}

fn is_newer(latest: &str, current: &str) -> bool {
    let latest = parse_version(latest);
    let current = parse_version(current);
    latest > current
}

fn parse_version(version: &str) -> Vec<u64> {
    version
        .split(|ch: char| !ch.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .map(|part| part.parse::<u64>().unwrap_or(0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{InstallSource, detect_source_for, sha256_hex, verify_checksum_manifest};
    use std::{fs, path::Path};
    use tempfile::tempdir;

    #[test]
    fn detects_macos_homebrew_from_standard_prefix_path() {
        let source = detect_source_for(Path::new("/opt/homebrew/bin/me"), "macos", None);
        assert_eq!(source, InstallSource::Homebrew);
    }

    #[test]
    fn detects_linux_release_binary_from_standard_install_path() {
        let source = detect_source_for(Path::new("/usr/local/bin/me"), "linux", None);
        assert_eq!(
            source,
            InstallSource::ReleaseBinary("/usr/local/bin/me".into())
        );
    }

    #[test]
    fn falls_back_to_unknown_when_source_is_not_safe_to_assume() {
        let source = detect_source_for(Path::new("/tmp/me"), "macos", None);
        assert_eq!(source, InstallSource::Unknown("/tmp/me".into()));
    }

    #[test]
    fn accepts_matching_sha256_manifest_entry() {
        let dir = tempdir().unwrap();
        let archive = dir.path().join("me-v0.3.4-linux-x64.tar.gz");
        fs::write(&archive, b"release-bytes").unwrap();
        let manifest = format!(
            "{}  {}\n",
            sha256_hex(&archive).unwrap(),
            archive.file_name().unwrap().to_string_lossy()
        );

        verify_checksum_manifest(&archive, &manifest).unwrap();
    }

    #[test]
    fn rejects_mismatched_sha256_manifest_entry() {
        let dir = tempdir().unwrap();
        let archive = dir.path().join("me-v0.3.4-linux-x64.tar.gz");
        fs::write(&archive, b"release-bytes").unwrap();
        let error = verify_checksum_manifest(&archive, "deadbeef  me-v0.3.4-linux-x64.tar.gz\n")
            .unwrap_err();

        assert!(error.to_string().contains("checksum verification failed"));
    }
}
