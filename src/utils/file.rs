use anyhow::Result;
use chrono::Local;
use semver::Version;
use std::fs::OpenOptions;
use std::io::Write;

pub fn create_changelog(version: &Version) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("CHANGELOG.md")?;

    // 获取当前日期
    let date = Local::now().format("%Y-%m-%d").to_string();

    // 写入标准 Changelog 头部
    writeln!(file, "# Changelog")?;
    writeln!(file)?;
    writeln!(
        file,
        "This document records all notable changes to the project."
    )?;
    writeln!(file)?;
    writeln!(file, "## Change Types")?;
    writeln!(file)?;
    writeln!(file, "- **Features**: New features or improvements")?;
    writeln!(file, "- **Bug Fixes**: Bug fixes and patches")?;
    writeln!(file, "- **Breaking Changes**: Incompatible changes")?;
    writeln!(file)?;
    writeln!(file, "## Commit Guidelines")?;
    writeln!(file)?;
    writeln!(file, "Please follow the [Conventional Commits](https://www.conventionalcommits.org/) specification when making commits.")?;
    writeln!(file)?;

    // 获取远程仓库 URL 和上一个版本号
    let remote_url = crate::utils::git::get_remote_url()?;
    let previous_version = crate::utils::git::get_previous_version()?;
    let compare_url = if previous_version == "initial" {
        format!("{}/commits/v{}", remote_url, version)
    } else {
        format!("{}/compare/{}...v{}", remote_url, previous_version, version)
    };

    // 写入最新版本信息
    writeln!(file, "### [{}]({}) ({})", version, compare_url, date)?;
    writeln!(file)?;

    // 获取 git 提交信息
    let commits = crate::utils::git::get_git_commits()?;

    // 分类处理提交信息
    let mut features = Vec::new();
    let mut fixes = Vec::new();
    let mut breaking_changes = Vec::new();

    for commit in commits {
        if commit.message.starts_with("feat:") {
            features.push(commit.message);
        } else if commit.message.starts_with("fix:") {
            fixes.push(commit.message);
        } else if commit.message.contains("BREAKING CHANGE:") {
            breaking_changes.push(commit.message);
        }
    }

    // 写入分类内容
    writeln!(file, "## {}", version)?;
    writeln!(file)?;

    if !features.is_empty() {
        writeln!(file, "### Features")?;
        for feat in features {
            writeln!(file, "- {}", feat)?;
        }
        writeln!(file)?;
    }

    if !fixes.is_empty() {
        writeln!(file, "### Bug Fixes")?;
        for fix in fixes {
            writeln!(file, "- {}", fix)?;
        }
        writeln!(file)?;
    }

    if !breaking_changes.is_empty() {
        writeln!(file, "### Breaking Changes")?;
        for bc in breaking_changes {
            writeln!(file, "- {}", bc)?;
        }
        writeln!(file)?;
    }
    println!("✔ [Created] CHANGELOG.md");
    Ok(())
}

pub fn create_gitignore() -> Result<()> {
    let mut gitignore = OpenOptions::new()
        .write(true)
        .create(true)
        .open(".gitignore")?;

    writeln!(gitignore, "/target")?;
    writeln!(gitignore, "/Cargo.lock")?;
    writeln!(gitignore, "/.idea")?;
    writeln!(gitignore, "/.vscode")?;

    Ok(())
}
