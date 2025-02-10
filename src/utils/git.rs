use anyhow::{Context, Result};
use dirs::home_dir;
use git2::{Remote, Repository};
use std::fs;
use std::io::Write;
use std::path::Path;

use super::file::create_gitignore;
use super::project::ProjectFile;
use super::version::Version;

// Git 操作相关功能模块
//
// 本模块提供了与 Git 仓库交互的核心功能，包括：
// - 标签管理
// - 版本控制
// - 仓库操作

/// 初始化 Git 仓库
///
/// 此函数会在当前目录创建一个新的 Git 仓库，并设置基本的 .gitignore 文件
pub fn initialize_git_repo() -> Result<()> {
    let repo = Repository::init(".")?;
    create_gitignore()?;
    fs::write(
        ".git/description",
        "Repository managed by RustyTag - https://github.com/sichang824/rustytag",
    )?;
    let mut index = repo.index()?;
    index.add_path(Path::new(".gitignore"))?;
    index.write()?;
    let oid = index.write_tree()?;
    let tree = repo.find_tree(oid)?;
    let signature = repo.signature()?;
    repo.commit(
        Some("refs/heads/main"),
        &signature,
        &signature,
        "Initial commit by RustyTag",
        &tree,
        &[],
    )?;
    repo.set_head("refs/heads/main")?;
    Ok(())
}

pub fn get_latest_tag() -> Result<Version> {
    let repo = Repository::open(".")?;
    let tags = repo.tag_names(None)?;

    // 如果没有标签，返回初始版本
    if tags.is_empty() {
        println!("⚠️ 没有找到任何标签，使用初始版本");
        return Ok(Version::new(semver::Version::new(0, 1, 0)));
    }

    // 收集所有标签并解析为 Version
    let mut versions: Vec<_> = tags
        .iter()
        .flatten()
        .filter_map(|t| Version::parse(t).ok())
        .collect();

    // 按版本号降序排序
    versions.sort_by(|a, b| b.version.cmp(&a.version));

    // 获取最新版本
    let latest_version = versions.first().cloned().ok_or_else(|| {
        println!("⚠️ 没有找到有效的版本标签，使用初始版本");
        anyhow::anyhow!("No valid version tags found")
    })?;

    // 如果有前缀且未配置，自动保存到配置中
    if !latest_version.prefix.is_empty() {
        let mut config = super::config::LocalConfig::load()?;
        if config.version_prefix.is_none() {
            config.version_prefix = Some(latest_version.prefix.clone());
            config.save()?;
            println!("✨ 已自动配置版本前缀: {}", latest_version.prefix);
        }
    }

    Ok(latest_version)
}

pub fn commit_changes(repo: &Repository, version: &Version) -> Result<()> {
    let mut index = repo.index()?;
    index.add_path(Path::new("CHANGELOG.md"))?;
    index.write()?;

    let oid = index.write_tree()?;
    let tree = repo.find_tree(oid)?;
    let signature = repo.signature()?;
    let parent_commit = repo.head()?.peel_to_commit()?;
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &format!("chore: release {}", version.version),
        &tree,
        &[&parent_commit],
    )?;
    println!("✔ [Committed]");
    Ok(())
}

/// 创建新的 Git 标签
///
/// # 参数
///
/// * `repo` - Git 仓库引用
/// * `version` - 版本号
///
/// # 示例
///
/// ```no_run
/// use git2::Repository;
/// use version::Version;
/// use rustytag::utils::git::create_tag;
///
/// # fn main() -> anyhow::Result<()> {
/// let repo = Repository::open(".")?;
/// let version = Version::new(1, 0, 0);
/// create_tag(&repo, &version)?;
/// # Ok(())
/// # }
/// ```
pub fn create_tag(repo: &Repository, version: &Version) -> Result<()> {
    let obj = repo.head()?.peel_to_commit()?.into_object();
    let signature = repo.signature()?;
    let changelog = std::fs::read_to_string("CHANGELOG.md")?;
    let version_content = changelog
        .split("---\n")
        .nth(1)
        .unwrap_or("No changelog content")
        .trim()
        .to_string();
    repo.tag(
        &version.to_string(),
        &obj,
        &signature,
        &version_content,
        false,
    )?;
    println!("✔ [Created] tag {}", version);
    Ok(())
}

pub fn get_remote(repo: &Repository) -> Result<Remote> {
    match repo.find_remote("origin") {
        Ok(remote) => Ok(remote),
        Err(e) if e.code() == git2::ErrorCode::NotFound => {
            println!("⚠ Warning: Remote 'origin' does not exist");
            println!("ℹ Please add a remote repository first using:");
            println!("   git remote add origin <repository-url>");
            Err(e.into())
        }
        Err(e) => Err(e.into()),
    }
}

pub fn reset_tags(repo: &Repository) -> Result<()> {
    // 获取远程仓库
    let mut remote = get_remote(repo)?;

    // 获取远程 tag 列表
    let remote_tags = fetch_remote_tags(&mut remote)?;

    // 删除本地所有 tag
    let local_tags = repo.tag_names(None)?;
    for tag in local_tags.iter().flatten() {
        repo.tag_delete(tag)?;
    }

    // 从远程获取 tag
    for tag in remote_tags {
        repo.tag(
            &tag,
            &repo.revparse_single(&format!("refs/remotes/origin/{}", tag))?,
            &repo.signature()?,
            &format!("Reset tag {}", tag),
            false,
        )?;
    }

    println!("✔ [Reset] local tags to match remote");
    Ok(())
}

fn fetch_remote_tags(remote: &mut Remote) -> Result<Vec<String>> {
    let mut remote_tags = Vec::new();
    println!("🔄 正在连接远程仓库...");

    // 创建认证回调
    let create_callbacks = || {
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            let home_dir = home_dir().ok_or_else(|| git2::Error::from_str("无法获取用户家目录"))?;
            let private_key_path = home_dir.join(".ssh/keys/privite/github");
            git2::Cred::ssh_key(
                username_from_url.unwrap_or("git"),
                None,
                &private_key_path,
                None,
            )
        });
        callbacks
    };

    // 设置 fetch 选项
    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(create_callbacks());
    fetch_options.download_tags(git2::AutotagOption::All);

    // 执行 fetch
    println!("🔍 正在获取远程 tags...");
    remote.fetch(&["refs/tags/*:refs/tags/*"], Some(&mut fetch_options), None)?;

    // 获取远程 tag 列表
    let connection = remote.connect_auth(git2::Direction::Fetch, Some(create_callbacks()), None)?;

    for head in connection.list()?.iter() {
        if let Some(tag_name) = head.name().strip_prefix("refs/tags/") {
            if !tag_name.ends_with("^{}") {
                println!("🏷️ 发现远程 tag: {}", tag_name);
                remote_tags.push(tag_name.to_string());
            }
        }
    }

    println!("✅ 成功获取 {} 个远程 tags", remote_tags.len());
    Ok(remote_tags)
}

#[derive(Debug)]
pub struct GitCommit {
    #[allow(dead_code)]
    pub hash: String,
    pub message: String,
}

/// 获取当前仓库的所有提交信息
pub fn get_git_commits() -> Result<Vec<GitCommit>> {
    // 打开当前目录的 git 仓库
    let repo = Repository::open(Path::new(".")).context("Failed to open git repository")?;

    // 获取 HEAD 引用
    let head = repo.head().context("Failed to get HEAD reference")?;

    // 获取 HEAD 指向的 commit
    let commit = head.peel_to_commit().context("Failed to peel to commit")?;

    // 遍历提交历史
    let mut commits = Vec::new();
    let mut walk = repo.revwalk().context("Failed to create revwalk")?;
    walk.push(commit.id()).context("Failed to push commit")?;

    for oid in walk {
        let oid = oid.context("Failed to get oid")?;
        let commit = repo.find_commit(oid).context("Failed to find commit")?;
        let message = commit
            .message()
            .context("Failed to get commit message")?
            .to_string();

        commits.push(GitCommit {
            hash: commit.id().to_string(),
            message,
        });
    }

    Ok(commits)
}

pub fn get_remote_url() -> Result<String> {
    let repo = Repository::open(Path::new("."))?;

    // 获取远程仓库
    let remote = get_remote(&repo)?;

    let url = remote
        .url()
        .ok_or_else(|| anyhow::anyhow!("Failed to get remote URL"))?;
    Ok(convert_ssh_to_https(url))
}

fn convert_ssh_to_https(url: &str) -> String {
    if url.starts_with("git@") {
        // 转换 git@github.com:user/repo.git 为 https://github.com/user/repo
        let parts: Vec<&str> = url.split('@').collect();
        if parts.len() == 2 {
            let domain_and_path = parts[1].replace(':', "/");
            return format!("https://{}", domain_and_path.trim_end_matches(".git"));
        }
    }
    url.to_string()
}

pub struct ProjectInfo {
    pub version: Version,
    pub repo_url: Option<String>,
    pub commit_count: usize,
    pub branch_name: Option<String>,
}

pub fn get_project_info(repo: &Repository) -> Result<ProjectInfo> {
    let version = get_latest_tag().unwrap_or_else(|_| Version::new(semver::Version::new(0, 1, 0)));

    let repo_url = get_remote_url().ok();
    let commits = get_git_commits()?;
    let commit_count = commits.len();
    let branch_name = repo.head()?.shorthand().map(|s| s.to_string());

    Ok(ProjectInfo {
        version,
        repo_url,
        commit_count,
        branch_name,
    })
}

pub fn add_project_files(repo: &Repository) -> Result<()> {
    let mut index = repo.index()?;
    let project_files = ProjectFile::detect_all()?;
    for file in project_files {
        if let Err(e) = index.add_path(&file.path) {
            println!("⚠️ 无法添加文件 {:?}: {}", file.path, e);
        }
    }
    index.write()?;
    Ok(())
}

/// 获取指定 tag 之后的所有新提交
pub fn get_commits_after_tag(tag: &str) -> Result<Vec<GitCommit>> {
    let repo = Repository::open(".")?;
    let mut commits = Vec::new();

    // 获取 tag 对应的 commit
    let tag_obj = repo.revparse_single(tag)?;
    let tag_commit = tag_obj.peel_to_commit()?;

    // 获取 HEAD commit
    let head = repo.head()?.peel_to_commit()?;

    // 创建一个版本遍历器
    let mut revwalk = repo.revwalk()?;
    revwalk.push(head.id())?;

    // 设置遍历范围：从 HEAD 到 tag
    revwalk.hide(tag_commit.id())?;

    // 遍历所有提交
    for oid in revwalk {
        let commit_id = oid?;
        let commit = repo.find_commit(commit_id)?;
        let message = commit.message().unwrap_or("").trim().to_string();

        commits.push(GitCommit {
            hash: commit.id().to_string(),
            message,
        });
    }

    Ok(commits)
}

/// 获取本地所有标签
pub fn get_local_tags(repo: &Repository) -> Result<Vec<String>> {
    let tags = repo.tag_names(None)?;
    Ok(tags
        .iter()
        .filter_map(|tag| tag.map(|s| s.to_string()))
        .collect())
}

/// 创建 Git 认证回调
fn create_callbacks() -> git2::RemoteCallbacks<'static> {
    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        let home_dir =
            home_dir().ok_or_else(|| git2::Error::from_str("Cannot get home directory"))?;
        let private_key_path = home_dir.join(".ssh/keys/privite/github");
        git2::Cred::ssh_key(
            username_from_url.unwrap_or("git"),
            None,
            &private_key_path,
            None,
        )
    });
    callbacks
}

/// 获取远程所有标签
pub fn get_remote_tags(repo: &Repository) -> Result<Vec<String>> {
    let mut remote = repo.find_remote("origin")?;

    // 设置认证回调
    let callbacks = create_callbacks();
    remote.connect_auth(git2::Direction::Fetch, Some(callbacks), None)?;
    let remote_list = remote.list()?;

    let tags: Vec<String> = remote_list
        .iter()
        .filter(|r| r.name().starts_with("refs/tags/"))
        .map(|r| r.name().trim_start_matches("refs/tags/").to_string())
        .filter(|name| !name.ends_with("^{}")) // 过滤掉注释标签
        .collect();

    remote.disconnect()?;
    Ok(tags)
}

/// 标签同步状态
#[derive(Debug)]
pub struct TagSyncStatus {
    pub all_tags: Vec<String>,
    pub to_push: Vec<String>,
    pub to_pull: Vec<String>,
}

/// 比较本地和远程标签
pub fn compare_tags(repo: &Repository) -> Result<TagSyncStatus> {
    let local_tags = get_local_tags(repo)?;
    let remote_tags = get_remote_tags(repo)?;

    let mut all_tags: Vec<String> = local_tags.clone();
    all_tags.extend(remote_tags.clone());
    all_tags.sort();
    all_tags.dedup();

    let to_push: Vec<String> = local_tags
        .iter()
        .filter(|tag| !remote_tags.contains(tag))
        .cloned()
        .collect();

    let to_pull: Vec<String> = remote_tags
        .iter()
        .filter(|tag| !local_tags.contains(tag))
        .cloned()
        .collect();

    Ok(TagSyncStatus {
        all_tags,
        to_push,
        to_pull,
    })
}

fn display_sync_status(status: &TagSyncStatus) -> bool {
    println!("📊 Tag Comparison");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let mut has_differences = false;
    for tag in &status.all_tags {
        let in_local = !status.to_pull.contains(tag);
        let in_remote = !status.to_push.contains(tag);
        let status_icon = match (in_local, in_remote) {
            (true, true) => "✅",
            (true, false) => "📤",
            (false, true) => "📥",
            (false, false) => unreachable!(),
        };

        // 尝试解析版本号以获得更好的显示效果
        let display_version = if let Ok(version) = Version::parse(tag) {
            version.to_string()
        } else {
            tag.to_string()
        };

        let status_text = match (in_local, in_remote) {
            (true, true) => "(synced)",
            (true, false) => "(local only)",
            (false, true) => "(remote only)",
            (false, false) => unreachable!(),
        };
        println!("{} {} {}", status_icon, display_version, status_text);
        if in_local != in_remote {
            has_differences = true;
        }
    }
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Legend:");
    println!("✅ Synced   📤 Local only   📥 Remote only\n");

    has_differences
}

fn pull_tags(remote: &mut Remote, tags: &[String]) -> Result<()> {
    if tags.is_empty() {
        return Ok(());
    }

    println!("🔄 Fetching remote tags...");
    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(create_callbacks());
    fetch_options.download_tags(git2::AutotagOption::All);
    remote.fetch(&["refs/tags/*:refs/tags/*"], Some(&mut fetch_options), None)?;
    println!("✨ Successfully fetched {} remote tags", tags.len());
    Ok(())
}

fn push_tags(remote: &mut Remote, tags: &[String]) -> Result<()> {
    if tags.is_empty() {
        return Ok(());
    }
    let mut push_options = git2::PushOptions::new();
    push_options.remote_callbacks(create_callbacks());

    let total = tags.len();
    for (index, tag) in tags.iter().enumerate() {
        print!("\r🏷️  Pushing tag ({}/{}) {}", index + 1, total, tag);
        std::io::stdout().flush()?;
        let refspec = format!("refs/tags/{}:refs/tags/{}", tag, tag);
        remote.push(&[&refspec], Some(&mut push_options))?;
    }
    println!("\n✨ Successfully pushed {} local tags!", total);
    Ok(())
}

pub fn show_and_sync_tags(repo: &Repository) -> Result<()> {
    let sync_status = compare_tags(repo)?;
    let has_differences = display_sync_status(&sync_status);
    if !has_differences {
        println!("✨ All tags are already in sync!");
        return Ok(());
    }
    print!("🔄 Do you want to sync these tags? [y/N] ");
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if input.trim().to_lowercase() == "y" {
        println!("\n🔄 Syncing tags with remote...");
        let mut remote = get_remote(repo)?;
        pull_tags(&mut remote, &sync_status.to_pull)?;
        push_tags(&mut remote, &sync_status.to_push)?;
        println!("✨ Successfully synced all tags!\n");
    } else {
        println!("❌ Sync cancelled");
    }
    Ok(())
}
