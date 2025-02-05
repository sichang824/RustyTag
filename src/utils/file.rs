use anyhow::Result;
use chrono::Local;
use semver::Version;
use std::fs::OpenOptions;
use std::io::Write;

pub fn create_changelog(version: &Version) -> Result<()> {
    println!("🔄 开始生成 CHANGELOG...");
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("CHANGELOG.md")?;

    // 获取当前日期
    let date = Local::now().format("%Y-%m-%d").to_string();
    println!("📅 当前日期: {}", date);

    // 如果文件为空，写入标准头部
    if file.metadata()?.len() == 0 {
        writeln!(file, "# Changelog")?;
        writeln!(file)?;
        writeln!(file, "## Change Types")?;
        writeln!(file)?;
        writeln!(file, "- **Features**: New features or improvements")?;
        writeln!(file, "- **Bug Fixes**: Bug fixes and patches")?;
        writeln!(file, "- **Breaking Changes**: Incompatible changes")?;
        writeln!(file)?;
        writeln!(file, "## Commit Guidelines")?;
        writeln!(file)?;
        writeln!(file, "All notable changes to this project will be documented in this file. See [Conventional Commits](https://www.conventionalcommits.org/) specification when making commits.")?;
        writeln!(file)?;
        writeln!(file, "---")?;
    }

    // 获取远程仓库 URL 和上一个版本号
    let remote_url = crate::utils::git::get_remote_url()?;
    let previous_version = crate::utils::git::get_previous_version()?;
    println!("🔗 远程仓库 URL: {}", remote_url);
    println!("📌 上一个版本: {}", previous_version);

    // 写入版本标题和对比链接
    if previous_version == "initial" {
        writeln!(
            file,
            "### [{}]({}/commits/{}) ({})",
            version,
            remote_url.trim_end_matches(".git"),
            version,
            date
        )?;
    } else {
        writeln!(
            file,
            "### [{}]({}/compare/{}...{}) ({})",
            version,
            remote_url.trim_end_matches(".git"),
            previous_version,
            version,
            date
        )?;
    }
    writeln!(file)?;

    // 获取提交记录
    println!("🔍 获取提交记录...");
    let commits = if previous_version == "initial" {
        println!("⚠️ 未找到上一个版本，获取所有提交");
        crate::utils::git::get_git_commits()?
    } else {
        println!("📊 获取 {} 之后的新提交", previous_version);
        crate::utils::git::get_commits_after_tag(&previous_version)?
    };
    println!("✅ 获取到 {} 条提交记录", commits.len());

    // 写入所有提交
    writeln!(file, "### Commits")?;
    writeln!(file)?;
    for commit in &commits {
        // 跳过 "chore: release" 提交
        if commit.message.starts_with("chore: release") {
            continue;
        }
        writeln!(
            file,
            "* {} ([{}]({}/commit/{}))",
            commit.message.lines().next().unwrap_or("").trim(),
            &commit.hash[..7],
            remote_url.trim_end_matches(".git"),
            commit.hash
        )?;
    }
    writeln!(file)?;

    println!("✨ CHANGELOG.md 生成完成");
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
