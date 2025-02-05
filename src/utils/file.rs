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
    }

    // 获取远程仓库 URL 和上一个版本号
    let remote_url = crate::utils::git::get_remote_url()?;
    let previous_version = crate::utils::git::get_previous_version()?;
    println!("🔗 远程仓库 URL: {}", remote_url);
    println!("📌 上一个版本: {}", previous_version);

    // 写入版本标题和对比链接
    writeln!(
        file,
        "### [{}]({}/compare/v{}...v{}) ({})",
        version,
        remote_url.trim_end_matches(".git"),
        previous_version,
        version,
        date
    )?;
    writeln!(file)?;

    // 获取提交记录
    println!("🔍 获取提交记录...");
    let commits = if previous_version == "initial" {
        println!("⚠️ 未找到上一个版本，获取所有提交");
        crate::utils::git::get_git_commits()?
    } else {
        println!("📊 获取从 v{} 到 v{} 之间的提交", previous_version, version);
        crate::utils::git::get_commits_between_tags(
            &format!("v{}", previous_version),
            &format!("v{}", version),
        )?
    };
    println!("✅ 获取到 {} 条提交记录", commits.len());

    // 分类处理提交信息
    let mut features = Vec::new();
    let mut fixes = Vec::new();
    let mut breaking_changes = Vec::new();

    for commit in &commits {
        println!("📝 处理提交: {} - {}", &commit.hash[..7], &commit.message);
        if commit.message.starts_with("feat:") {
            features.push(format!(
                "{} ([{}]({}/commit/{}))",
                commit.message[5..].trim(),
                &commit.hash[..7],
                remote_url.trim_end_matches(".git"),
                commit.hash
            ));
        } else if commit.message.starts_with("fix:") {
            fixes.push(format!(
                "{} ([{}]({}/commit/{}))",
                commit.message[4..].trim(),
                &commit.hash[..7],
                remote_url.trim_end_matches(".git"),
                commit.hash
            ));
        } else if commit.message.contains("BREAKING CHANGE:") {
            breaking_changes.push(format!(
                "{} ([{}]({}/commit/{}))",
                commit.message,
                &commit.hash[..7],
                remote_url.trim_end_matches(".git"),
                commit.hash
            ));
        }
    }

    println!("📊 统计结果:");
    println!("- Features: {}", features.len());
    println!("- Bug Fixes: {}", fixes.len());
    println!("- Breaking Changes: {}", breaking_changes.len());

    // 写入分类内容
    if !features.is_empty() {
        writeln!(file, "### Features")?;
        writeln!(file)?;
        for feat in features {
            writeln!(file, "* {}", feat)?;
        }
        writeln!(file)?;
    }

    if !fixes.is_empty() {
        writeln!(file, "### Bug Fixes")?;
        writeln!(file)?;
        for fix in fixes {
            writeln!(file, "* {}", fix)?;
        }
        writeln!(file)?;
    }

    if !breaking_changes.is_empty() {
        writeln!(file, "### BREAKING CHANGES")?;
        writeln!(file)?;
        for bc in breaking_changes {
            writeln!(file, "* {}", bc)?;
        }
        writeln!(file)?;
    }

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
