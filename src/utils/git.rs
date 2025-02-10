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

    // If no tags found, return initial version
    if tags.is_empty() {
        println!("⚠️ No tags found, using initial version");
        return Ok(Version::new(semver::Version::new(0, 1, 0)));
    }

    // Collect all tags and parse to Version
    let mut versions: Vec<_> = tags
        .iter()
        .flatten()
        .filter_map(|t| Version::parse(t).ok())
        .collect();

    // Sort by version number in descending order
    versions.sort_by(|a, b| b.version.cmp(&a.version));

    // Get latest version
    let latest_version = versions.first().cloned().ok_or_else(|| {
        println!("⚠️ No valid version tags found, using initial version");
        anyhow::anyhow!("No valid version tags found")
    })?;

    // If prefix exists and not configured, save to config
    if !latest_version.prefix.is_empty() {
        let mut config = super::config::LocalConfig::load()?;
        if config.version_prefix.is_none() {
            config.version_prefix = Some(latest_version.prefix.clone());
            config.save()?;
            println!("✨ Version prefix auto-configured: {}", latest_version.prefix);
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
    // Get remote repository
    let mut remote = get_remote(repo)?;

    // Get remote tag list
    let remote_tags = fetch_remote_tags(&mut remote)?;

    // Delete all local tags
    let local_tags = repo.tag_names(None)?;
    for tag in local_tags.iter().flatten() {
        repo.tag_delete(tag)?;
    }

    // Get tags from remote
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
    println!("🔄 Connecting to remote repository...");

    // Create auth callbacks
    let create_callbacks = || {
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            let home_dir = home_dir().ok_or_else(|| git2::Error::from_str("Cannot get home directory"))?;
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

    // Set fetch options
    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(create_callbacks());
    fetch_options.download_tags(git2::AutotagOption::All);

    // Execute fetch
    println!("🔍 Fetching remote tags...");
    remote.fetch(&["refs/tags/*:refs/tags/*"], Some(&mut fetch_options), None)?;

    // Get remote tag list
    let connection = remote.connect_auth(git2::Direction::Fetch, Some(create_callbacks()), None)?;

    for head in connection.list()?.iter() {
        if let Some(tag_name) = head.name().strip_prefix("refs/tags/") {
            if !tag_name.ends_with("^{}") {
                println!("🏷️ Found remote tag: {}", tag_name);
                remote_tags.push(tag_name.to_string());
            }
        }
    }

    println!("✅ Successfully fetched {} remote tags", remote_tags.len());
    Ok(remote_tags)
}

#[derive(Debug)]
pub struct GitCommit {
    #[allow(dead_code)]
    pub hash: String,
    pub message: String,
}

/// Get all commit information for the current repository
pub fn get_git_commits() -> Result<Vec<GitCommit>> {
    // Open git repository for current directory
    let repo = Repository::open(Path::new(".")).context("Failed to open git repository")?;

    // Get HEAD reference
    let head = repo.head().context("Failed to get HEAD reference")?;

    // Get HEAD pointing commit
    let commit = head.peel_to_commit().context("Failed to peel to commit")?;

    // Iterate through commit history
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

    // Get remote repository
    let remote = get_remote(&repo)?;

    let url = remote
        .url()
        .ok_or_else(|| anyhow::anyhow!("Failed to get remote URL"))?;
    Ok(convert_ssh_to_https(url))
}

fn convert_ssh_to_https(url: &str) -> String {
    if url.starts_with("git@") {
        // Convert git@github.com:user/repo.git to https://github.com/user/repo
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
            println!("⚠️ Cannot add file {:?}: {}", file.path, e);
        }
    }
    index.write()?;
    Ok(())
}

/// Get all new commits after a specific tag
pub fn get_commits_after_tag(tag: &str) -> Result<Vec<GitCommit>> {
    let repo = Repository::open(".")?;
    let mut commits = Vec::new();

    // Get tag pointing commit
    let tag_obj = repo.revparse_single(tag)?;
    let tag_commit = tag_obj.peel_to_commit()?;

    // Get HEAD commit
    let head = repo.head()?.peel_to_commit()?;

    // Create a version walker
    let mut revwalk = repo.revwalk()?;
    revwalk.push(head.id())?;

    // Set traversal range: from HEAD to tag
    revwalk.hide(tag_commit.id())?;

    // Iterate through all commits
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

/// Get all local tags
pub fn get_local_tags(repo: &Repository) -> Result<Vec<String>> {
    let tags = repo.tag_names(None)?;
    Ok(tags
        .iter()
        .filter_map(|tag| tag.map(|s| s.to_string()))
        .collect())
}

/// Create Git auth callbacks
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

/// Get all remote tags
pub fn get_remote_tags(repo: &Repository) -> Result<Vec<String>> {
    let mut remote = repo.find_remote("origin")?;

    // Set auth callbacks
    let callbacks = create_callbacks();
    remote.connect_auth(git2::Direction::Fetch, Some(callbacks), None)?;
    let remote_list = remote.list()?;

    let tags: Vec<String> = remote_list
        .iter()
        .filter(|r| r.name().starts_with("refs/tags/"))
        .map(|r| r.name().trim_start_matches("refs/tags/").to_string())
        .filter(|name| !name.ends_with("^{}")) // Filter out comment tags
        .collect();

    remote.disconnect()?;
    Ok(tags)
}

/// Tag sync status
#[derive(Debug)]
pub struct TagSyncStatus {
    pub all_tags: Vec<String>,
    pub to_push: Vec<String>,
    pub to_pull: Vec<String>,
}

/// Compare local and remote tags
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

        // Try to parse version number for better display effect
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
