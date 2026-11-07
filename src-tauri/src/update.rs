//! 自动更新：查 GitHub 最新 release、下载对应安装包、交给系统包管理器安装。
//!
//! 不引入 HTTP/TLS 依赖：项目本来就依赖系统里的 `ip` / `pkexec`，更新同样走
//! 系统 `curl`，既省下十几兆二进制，也能直接继承系统的代理与证书配置。
//!
//! 下载完成的包和元数据放在配置目录的 `updates/` 下。下次启动时重新查一次
//! GitHub：只有「本地包的版本 == 最新版」才允许直接安装，避免装回旧版本。

use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::i18n::{is_en, tr};
use crate::settings;

/// 发布仓库。迁移仓库时只改这一行。
const REPO: &str = "ja-plus/softether-tray";

/// 返回给界面的更新状态。字段用 camelCase，和前端其它接口保持一致。
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatus {
    pub current_version: String,
    pub latest_version: Option<String>,
    /// 有新版本可装（含"已下载待安装"）
    pub available: bool,
    /// 最新版安装包已经下载到本地
    pub downloaded: bool,
    pub asset_name: Option<String>,
    pub asset_size: u64,
    /// 检查失败时的原因；失败不代表不能安装此前已下载的包
    pub error: Option<String>,
}

/// 已下载、待安装的版本。写在磁盘上，重启后仍然有效。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PendingUpdate {
    pub version: String,
    /// 只存文件名，安装时再拼回 updates 目录，避免把外部字符串当路径
    pub file_name: String,
    pub size: u64,
    /// GitHub 给出的 `sha256:...`，没有就跳过校验
    pub digest: Option<String>,
}

impl PendingUpdate {
    pub fn path(&self) -> PathBuf {
        updates_dir().join(&self.file_name)
    }
}

#[derive(Clone, Debug)]
struct Release {
    version: String,
    asset_name: String,
    url: String,
    size: u64,
    digest: Option<String>,
}

/// 当前系统装的是哪种包，决定下载 .deb 还是 .rpm。
#[derive(Clone, Copy, PartialEq, Debug)]
enum PackageKind {
    Deb,
    Rpm,
}

impl PackageKind {
    fn detect() -> Self {
        let installed = |program: &str, args: &[&str]| {
            Command::new(program)
                .args(args)
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
        };
        if installed("dpkg", &["-s", "soft-ether-tray"]) {
            return Self::Deb;
        }
        if installed("rpm", &["-q", "soft-ether-tray"]) {
            return Self::Rpm;
        }
        // 没装本应用（开发机）时按系统里存在的包管理器猜
        if installed("dpkg", &["--version"]) {
            Self::Deb
        } else {
            Self::Rpm
        }
    }

    fn extension(self) -> &'static str {
        match self {
            Self::Deb => "deb",
            Self::Rpm => "rpm",
        }
    }

    fn matches(self, asset_name: &str) -> bool {
        asset_name.to_ascii_lowercase().ends_with(&format!(".{}", self.extension()))
    }

    fn install_command(self, package: &Path) -> Command {
        let mut command = Command::new("pkexec");
        match self {
            Self::Deb => {
                command.args(["dpkg", "-i"]);
            }
            Self::Rpm => {
                command.args(["rpm", "-U", "--replacepkgs"]);
            }
        }
        command.arg(package);
        command
    }
}

fn updates_dir() -> PathBuf {
    settings::config_dir().join("updates")
}

fn pending_path() -> PathBuf {
    updates_dir().join("pending.json")
}

fn load_pending() -> Option<PendingUpdate> {
    let text = fs::read_to_string(pending_path()).ok()?;
    serde_json::from_str(&text).ok()
}

fn write_pending(pending: &PendingUpdate) -> Result<(), String> {
    let dir = updates_dir();
    fs::create_dir_all(&dir).map_err(|error| {
        if is_en() {
            format!("Cannot create {}: {error}", dir.display())
        } else {
            format!("无法创建目录 {}: {error}", dir.display())
        }
    })?;
    let text = serde_json::to_string_pretty(pending).map_err(|error| error.to_string())?;
    let path = pending_path();
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, text).map_err(|error| error.to_string())?;
    fs::rename(&tmp, &path).map_err(|error| error.to_string())?;
    Ok(())
}

/// 删除下载残留（元数据 + 安装包）。传 None 只清元数据。
fn remove_pending(pending: Option<&PendingUpdate>) {
    if let Some(pending) = pending {
        let _ = fs::remove_file(pending.path());
    }
    let _ = fs::remove_file(pending_path());
}

/// 查一次 GitHub 并给出当前该展示的状态。
pub fn check(current: &str) -> UpdateStatus {
    let pending = load_pending();
    match fetch_latest() {
        Ok(release) => {
            let available = version_gt(&release.version, current);
            let same_as_release = pending
                .as_ref()
                .is_some_and(|item| item.version == release.version);
            let downloaded = available && same_as_release && pending.as_ref().is_some_and(|item| item.path().is_file());

            if !available {
                // 已经是最新（下载的包已被安装，或本来就比线上新）：清理残留
                if pending.is_some() {
                    remove_pending(pending.as_ref());
                }
            } else if !same_as_release && pending.is_some() {
                // 线上又发了新版本，之前下的包作废
                remove_pending(pending.as_ref());
            } else if same_as_release && !downloaded {
                // 元数据还在但文件被删了
                remove_pending(pending.as_ref());
            }

            UpdateStatus {
                current_version: current.to_string(),
                latest_version: Some(release.version),
                available,
                downloaded,
                asset_name: Some(release.asset_name),
                asset_size: release.size,
                error: None,
            }
        }
        Err(error) => {
            // 断网时不能把"已下载"也判掉：用户仍然可以装之前下好的包。
            // 只认版本确实比当前新的，避免把旧包当成更新。
            let usable = pending
                .as_ref()
                .is_some_and(|item| version_gt(&item.version, current) && item.path().is_file());
            UpdateStatus {
                current_version: current.to_string(),
                latest_version: pending.as_ref().map(|item| item.version.clone()),
                available: usable,
                downloaded: usable,
                asset_name: pending.as_ref().map(|item| item.file_name.clone()),
                asset_size: pending.as_ref().map(|item| item.size).unwrap_or(0),
                error: Some(error),
            }
        }
    }
}

/// 下载最新版安装包。返回值里 `downloaded` 为 true 就可以提示用户安装。
pub fn download(current: &str) -> Result<UpdateStatus, String> {
    let release = fetch_latest()?;
    if !version_gt(&release.version, current) {
        return Err(tr("当前已经是最新版本", "Already up to date").to_string());
    }

    let kind = PackageKind::detect();
    if !kind.matches(&release.asset_name) {
        return Err(if is_en() {
            format!(
                "The latest release has no {} package ({}).",
                kind.extension(),
                release.asset_name
            )
        } else {
            format!(
                "最新版本没有适用于当前系统的 {} 安装包（{}）",
                kind.extension(),
                release.asset_name
            )
        });
    }

    let dir = updates_dir();
    fs::create_dir_all(&dir).map_err(|error| {
        if is_en() {
            format!("Cannot create {}: {error}", dir.display())
        } else {
            format!("无法创建目录 {}: {error}", dir.display())
        }
    })?;

    let file_name = format!(
        "softether-tray-{}.{}",
        sanitize_version(&release.version),
        kind.extension()
    );
    let target = dir.join(&file_name);
    let part = dir.join(format!("{file_name}.part"));
    let _ = fs::remove_file(&part);

    let output = Command::new("curl")
        .args([
            "-fL",
            "--retry",
            "2",
            "--connect-timeout",
            "15",
            "--max-time",
            "1800",
            "-o",
        ])
        .arg(&part)
        .arg(&release.url)
        .output()
        .map_err(|error| {
            if is_en() {
                format!("Cannot run curl (required for updates): {error}")
            } else {
                format!("无法执行 curl（自动更新依赖它）：{error}")
            }
        })?;

    if !output.status.success() {
        let _ = fs::remove_file(&part);
        let detail = String::from_utf8_lossy(&output.stderr);
        let reason = detail.lines().last().unwrap_or("curl").trim().to_string();
        return Err(if is_en() {
            format!("Download failed: {reason}")
        } else {
            format!("下载失败：{reason}")
        });
    }

    if release.size > 0 {
        if let Ok(metadata) = fs::metadata(&part) {
            if metadata.len() != release.size {
                let _ = fs::remove_file(&part);
                return Err(tr(
                    "下载的文件大小与发布信息不一致，请重试",
                    "The downloaded file does not match the published size; please retry.",
                )
                .to_string());
            }
        }
    }

    if let Some(digest) = &release.digest {
        if let Err(error) = verify_sha256(&part, digest) {
            let _ = fs::remove_file(&part);
            return Err(error);
        }
    }

    fs::rename(&part, &target).map_err(|error| error.to_string())?;

    // 只保留这一份包，避免 updates/ 越堆越大
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let is_package = matches!(
                path.extension().and_then(|value| value.to_str()),
                Some("deb") | Some("rpm")
            );
            if is_package && path != target {
                let _ = fs::remove_file(path);
            }
        }
    }

    let pending = PendingUpdate {
        version: release.version.clone(),
        file_name,
        size: release.size,
        digest: release.digest.clone(),
    };
    write_pending(&pending)?;

    Ok(UpdateStatus {
        current_version: current.to_string(),
        latest_version: Some(release.version),
        available: true,
        downloaded: true,
        asset_name: Some(release.asset_name),
        asset_size: release.size,
        error: None,
    })
}

/// 安装前再核对一次本地包：文件还在、大小对得上、摘要对得上。
pub fn verify_pending() -> Result<PendingUpdate, String> {
    let pending = load_pending().ok_or_else(|| {
        tr(
            "没有已下载的更新包，请先下载",
            "No downloaded update package; download it first.",
        )
        .to_string()
    })?;
    let path = pending.path();
    let metadata = fs::metadata(&path).map_err(|_| {
        tr(
            "更新包已不存在，请重新下载",
            "The downloaded package is gone; please download it again.",
        )
        .to_string()
    })?;
    if pending.size > 0 && metadata.len() != pending.size {
        remove_pending(Some(&pending));
        return Err(tr(
            "更新包不完整，请重新下载",
            "The downloaded package is incomplete; please download it again.",
        )
        .to_string());
    }
    if let Some(digest) = &pending.digest {
        verify_sha256(&path, digest)?;
    }
    Ok(pending)
}

/// 交给系统包管理器安装。调用方负责先停掉 VPN 客户端并退出应用。
pub fn install_package(pending: &PendingUpdate) -> Result<(), String> {
    let path = pending.path();
    if !path.is_file() {
        return Err(tr(
            "更新包已不存在，请重新下载",
            "The downloaded package is gone; please download it again.",
        )
        .to_string());
    }
    let mut command = PackageKind::detect().install_command(&path);
    let status = command.status().map_err(|error| {
        if is_en() {
            format!("Cannot start the installer: {error}")
        } else {
            format!("无法启动安装程序：{error}")
        }
    })?;
    if !status.success() {
        return Err(tr(
            "安装被取消或失败",
            "Installation was cancelled or failed.",
        )
        .to_string());
    }
    let _ = fs::remove_file(pending_path());
    Ok(())
}

fn fetch_latest() -> Result<Release, String> {
    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");
    let output = Command::new("curl")
        .args([
            "-fsSL",
            "--max-time",
            "20",
            "-H",
            "User-Agent: softether-tray-updater",
            "-H",
            "Accept: application/vnd.github+json",
        ])
        .arg(&url)
        .output()
        .map_err(|error| {
            if is_en() {
                format!("Cannot run curl (required for updates): {error}")
            } else {
                format!("无法执行 curl（自动更新依赖它）：{error}")
            }
        })?;
    if !output.status.success() {
        return Err(tr(
            "访问 GitHub 失败，请检查网络后重试",
            "Cannot reach GitHub; check your network and retry.",
        )
        .to_string());
    }
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).map_err(|error| {
        if is_en() {
            format!("Cannot parse the GitHub response: {error}")
        } else {
            format!("GitHub 返回的数据无法解析：{error}")
        }
    })?;
    parse_release(&json, PackageKind::detect())
}

fn parse_release(json: &serde_json::Value, kind: PackageKind) -> Result<Release, String> {
    let tag = json
        .get("tag_name")
        .and_then(|value| value.as_str())
        .ok_or_else(|| {
            tr(
                "发布信息缺少版本号",
                "The release has no version tag.",
            )
            .to_string()
        })?;
    let version = tag.trim().trim_start_matches(['v', 'V']).to_string();
    if version.is_empty() {
        return Err(tr("发布信息里的版本号为空", "The release tag is empty.").to_string());
    }

    let assets = json
        .get("assets")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    let asset = assets
        .iter()
        .find(|asset| {
            asset
                .get("name")
                .and_then(|value| value.as_str())
                .is_some_and(|name| kind.matches(name))
        })
        .ok_or_else(|| {
            if is_en() {
                format!(
                    "The latest release has no {} package for this system.",
                    kind.extension()
                )
            } else {
                format!(
                    "最新版本没有适用于当前系统的 {} 安装包",
                    kind.extension()
                )
            }
        })?;

    let asset_name = asset
        .get("name")
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .to_string();
    let url = asset
        .get("browser_download_url")
        .and_then(|value| value.as_str())
        .ok_or_else(|| {
            tr(
                "发布信息缺少下载地址",
                "The release asset has no download URL.",
            )
            .to_string()
        })?
        .to_string();
    let size = asset.get("size").and_then(|value| value.as_u64()).unwrap_or(0);
    let digest = asset
        .get("digest")
        .and_then(|value| value.as_str())
        .map(str::to_string);

    Ok(Release {
        version,
        asset_name,
        url,
        size,
        digest,
    })
}

fn verify_sha256(path: &Path, digest: &str) -> Result<(), String> {
    let expected = digest
        .strip_prefix("sha256:")
        .unwrap_or(digest)
        .trim()
        .to_ascii_lowercase();
    if expected.is_empty() {
        return Ok(());
    }
    let output = Command::new("sha256sum")
        .arg(path)
        .output()
        .map_err(|error| {
            if is_en() {
                format!("Cannot verify the package: {error}")
            } else {
                format!("无法校验更新包：{error}")
            }
        })?;
    if !output.status.success() {
        return Err(tr("无法校验更新包", "Cannot verify the package.").to_string());
    }
    let actual = String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    if actual != expected {
        return Err(tr(
            "更新包校验失败，请重新下载",
            "The update package failed verification; please download it again.",
        )
        .to_string());
    }
    Ok(())
}

/// 版本号只保留数字和点，防止把 tag 里的其它字符带进文件名。
///
/// 先砍掉 `-rc1` / `+build` 这类后缀：只过滤字符的话，`0.6.0-rc1` 会变成
/// `0.6.01`，文件名看起来像另一个版本。
fn sanitize_version(version: &str) -> String {
    let core = version.split(['-', '+']).next().unwrap_or(version);
    let cleaned: String = core
        .chars()
        .filter(|ch| ch.is_ascii_digit() || *ch == '.')
        .collect();
    let cleaned = cleaned.trim_matches('.');
    if cleaned.is_empty() {
        "unknown".to_string()
    } else {
        cleaned.to_string()
    }
}

/// 比较点分版本号，缺省位按 0 处理：`0.5` == `0.5.0`，`0.10` > `0.9`。
fn version_gt(left: &str, right: &str) -> bool {
    compare_versions(left, right) == Ordering::Greater
}

fn compare_versions(left: &str, right: &str) -> Ordering {
    let left = version_parts(left);
    let right = version_parts(right);
    for index in 0..3 {
        let left_part = left.get(index).copied().unwrap_or(0);
        let right_part = right.get(index).copied().unwrap_or(0);
        match left_part.cmp(&right_part) {
            Ordering::Equal => continue,
            other => return other,
        }
    }
    Ordering::Equal
}

fn version_parts(version: &str) -> Vec<u64> {
    let core = version.trim().trim_start_matches(['v', 'V']);
    let core = core.split(['-', '+']).next().unwrap_or(core);
    core.split('.')
        .map(|part| part.parse::<u64>().unwrap_or(0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_versions_numerically() {
        assert!(version_gt("0.5.1", "0.5.0"));
        assert!(version_gt("0.10.0", "0.9.9"));
        assert!(version_gt("v1.0", "0.9.9"));
        assert!(!version_gt("0.5.0", "0.5.0"));
        assert!(!version_gt("0.5", "0.5.0"));
        assert!(!version_gt("0.4.9", "0.5.0"));
    }

    #[test]
    fn picks_the_package_for_the_current_system() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{
                "tag_name": "v0.6.0",
                "assets": [
                    {"name": "soft-ether-tray-0.6.0-1.x86_64.rpm", "size": 10, "browser_download_url": "https://example.test/a.rpm", "digest": "sha256:aaa"},
                    {"name": "soft-ether-tray_0.6.0_amd64.deb", "size": 20, "browser_download_url": "https://example.test/a.deb", "digest": "sha256:bbb"}
                ]
            }"#,
        )
        .unwrap();

        let deb = parse_release(&json, PackageKind::Deb).unwrap();
        assert_eq!(deb.version, "0.6.0");
        assert_eq!(deb.asset_name, "soft-ether-tray_0.6.0_amd64.deb");
        assert_eq!(deb.size, 20);
        assert_eq!(deb.digest.as_deref(), Some("sha256:bbb"));

        let rpm = parse_release(&json, PackageKind::Rpm).unwrap();
        assert_eq!(rpm.asset_name, "soft-ether-tray-0.6.0-1.x86_64.rpm");
    }

    #[test]
    fn missing_package_is_reported() {
        let json: serde_json::Value =
            serde_json::from_str(r#"{"tag_name":"v0.6.0","assets":[]}"#).unwrap();
        assert!(parse_release(&json, PackageKind::Deb).is_err());
    }

    #[test]
    fn sanitizes_version_for_file_names() {
        assert_eq!(sanitize_version("0.6.0-rc1"), "0.6.0");
        assert_eq!(sanitize_version("../x"), "unknown");
        assert_eq!(sanitize_version(""), "unknown");
    }
}
