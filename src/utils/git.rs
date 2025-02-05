use anyhow::{Context, Result};
use dirs::home_dir;
use git2::{Remote, Repository};
use semver::Version;
use std::fs;
use std::path::Path;

use super::file::create_gitignore;
use super::project::ProjectFile;

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

pub fn get_latest_tag() -> Result<String> {
    let repo = Repository::open(".")?;
    let tags = repo.tag_names(None)?;

    // 如果没有标签，返回初始版本
    if tags.is_empty() {
        println!("⚠️ 没有找到任何标签，使用初始版本");
        return Ok("initial".to_string());
    }

    // 获取最新的有效版本标签
    let latest_tag = tags
        .iter()
        .filter_map(|t| t)
        .filter_map(|t| Version::parse(t).ok())
        .max()
        .map(|v| v.to_string())
        .unwrap_or_else(|| {
            println!("⚠️ 没有找到有效的版本标签，使用初始版本");
            "initial".to_string()
        });

    Ok(latest_tag)
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
        &format!("chore: release {}", version),
        &tree,
        &[&parent_commit],
    )?;
    println!("✔ [Committed]");
    Ok(())
}

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
        &version_content, // 使用 "---" 之后的所有内容作为 tag 消息
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
            println!("🏷️ 发现远程 tag: {}", tag_name);
            remote_tags.push(tag_name.to_string());
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

pub fn get_previous_tag() -> Result<String> {
    let repo = Repository::open(Path::new("."))?;
    let tags = repo.tag_names(None)?;

    // 收集并排序所有有效的版本
    let mut versions: Vec<Version> = tags
        .iter()
        .filter_map(|t| t)
        .filter_map(|t| Version::parse(t).ok())
        .collect();
    versions.sort();

    // 如果没有标签或只有一个标签，返回 "initial"
    if versions.len() <= 1 {
        return Ok("initial".to_string());
    }

    // 返回倒数第二个版本
    Ok(versions[versions.len() - 2].to_string())
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
    pub version: String,
    pub repo_url: Option<String>,
    pub commit_count: usize,
    pub branch_name: Option<String>,
}

pub fn get_project_info(repo: &Repository) -> Result<ProjectInfo> {
    let version = get_latest_tag()?;
    let repo_url = get_remote_url().ok();

    // 获取提交数量
    let commits = get_git_commits()?;
    let commit_count = commits.len();

    // 获取当前分支名
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
