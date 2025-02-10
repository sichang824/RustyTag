use anyhow::Result;
use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;

use super::version::Version;

pub fn create_changelog(version: &Version) -> Result<()> {
    println!("🔄 Generating CHANGELOG...");
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("CHANGELOG.md")?;

    // Get current date
    let date = Local::now().format("%Y-%m-%d").to_string();
    println!("📅 Current date: {}", date);

    // If file is empty, write standard header
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

    // Get remote repository URL and previous version
    let remote_url = crate::utils::git::get_remote_url()?;
    let previous_version = crate::utils::git::get_latest_tag()?;
    println!("🔗 Remote repository URL: {}", remote_url);
    println!("📌 Previous version: {}", previous_version);

    // Write version title and comparison link
    let initial_version =
        Version::new(semver::Version::new(0, 1, 0)).with_prefix(previous_version.prefix.clone());

    if previous_version.version == initial_version.version {
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

    // Get commit history
    println!("🔍 Getting commit history...");
    let commits = if previous_version.version == initial_version.version {
        println!("⚠️ No previous version found, getting all commits");
        crate::utils::git::get_git_commits()?
    } else {
        println!("📊 Getting new commits after {}", previous_version);
        crate::utils::git::get_commits_after_tag(&previous_version.to_string())?
    };
    println!("✅ Found {} commits", commits.len());

    // Write all commits
    writeln!(file, "### Commits")?;
    writeln!(file)?;
    for commit in &commits {
        // Skip "chore: release" commits
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

    println!("✨ CHANGELOG.md generated successfully");
    Ok(())
}

pub fn create_gitignore() -> Result<()> {
    let mut gitignore = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(".gitignore")?;

    writeln!(gitignore, "/target")?;
    writeln!(gitignore, "/Cargo.lock")?;
    writeln!(gitignore, "/.idea")?;
    writeln!(gitignore, "/.vscode")?;

    Ok(())
}
