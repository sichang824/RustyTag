use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use git2::Repository;
use semver::Version;
use std::path::Path;

mod utils;

use utils::{
    file::create_changelog,
    git::{
        add_project_files, commit_changes, create_tag, get_latest_tag, get_project_info,
        initialize_git_repo, reset_tags,
    },
    version::{bump_version, update_version_to_project, BumpType},
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new git repository
    Init,
    /// Bump patch version (e.g., 1.0.0 -> 1.0.1)
    Patch,
    /// Bump minor version (e.g., 1.0.0 -> 1.1.0)
    Minor,
    /// Bump major version (e.g., 1.0.0 -> 2.0.0)
    Major,
    /// Reset local tags to match remote
    Reset,
    /// Show current version information
    Show,
    /// Create a release with changelog
    Release {
        /// Specify a tag version to release
        #[arg(short, long)]
        tag: Option<String>,
    },
}

fn show_project_info(repo: &Repository) -> Result<()> {
    let info = get_project_info(repo)?;
    println!("\n📦 项目信息");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🏷️  当前版本: {}", info.version);
    if let Some(branch) = info.branch_name {
        println!("🌿 当前分支: {}", branch);
    }
    println!("📝 提交数量: {}", info.commit_count);
    if let Some(url) = info.repo_url {
        println!("🔗 仓库地址: {}", url);
    }
    println!("\n🛠️  RustyTag 信息");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📋 版本: {}", env!("CARGO_PKG_VERSION"));
    println!("📘 描述: A Git tag and version management tool");
    println!("🏠 主页: https://github.com/sichang824/rustytag");
    println!("👤 作者: sichang <sichang824@gmail.com>");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init => {
            handle_git_initialization()?;
        }
        _ => {
            if !Path::new(".git").exists() {
                println!("This directory is not a Git repository");
                println!("Please run `rustytag init` to initialize a repository");
                return Ok(());
            }
            let repo = Repository::open(".").context("Failed to open Git repository")?;
            match cli.command {
                Commands::Patch | Commands::Minor | Commands::Major => {
                    let latest_tag = get_latest_tag(&repo)?;
                    let new_version = match cli.command {
                        Commands::Patch => bump_version(&latest_tag, BumpType::Patch),
                        Commands::Minor => bump_version(&latest_tag, BumpType::Minor),
                        Commands::Major => bump_version(&latest_tag, BumpType::Major),
                        _ => unreachable!(),
                    };
                    update_version_to_project(&new_version)?;
                    add_project_files(&repo)?;
                    create_changelog(&new_version)?;
                    commit_changes(&repo, &new_version)?;
                    create_tag(&repo, &new_version)?;
                    println!("\nℹ Run the following command to publish the release");
                    println!("git push --follow-tags origin main\n");
                }
                Commands::Reset => {
                    reset_tags(&repo)?;
                }
                Commands::Show => {
                    show_project_info(&repo)?;
                }
                Commands::Release { tag } => {
                    let version = if let Some(tag_str) = tag {
                        Version::parse(&tag_str).context("Invalid version format")?
                    } else {
                        get_latest_tag(&repo)?
                    };

                    // 获取 changelog 内容
                    let changelog = std::fs::read_to_string("CHANGELOG.md")?;
                    let release_notes = changelog
                        .split("\n\n")
                        .find(|section| section.starts_with(&format!("### [{}]", version)))
                        .unwrap_or("No changelog content");

                    // 获取 GitHub token
                    let token = std::env::var("GITHUB_TOKEN").context(
                        "GITHUB_TOKEN environment variable not set. To create a GitHub token:\n\
                         1. Visit https://github.com/settings/tokens\n\
                         2. Click 'Generate new token' (classic)\n\
                         3. Select the 'repo' scope\n\
                         4. Generate and copy the token\n\
                         5. Set it in your environment:\n\
                            export GITHUB_TOKEN=your_token\n\
                         Note: For permanent setup, add it to your shell profile (~/.bashrc, ~/.zshrc, etc.)"
                    )?;

                    // 获取仓库 URL
                    let repo_url = utils::git::get_remote_url()?;

                    // 创建 release
                    tokio::runtime::Runtime::new()?.block_on(async {
                        utils::github::create_release(
                            &token,
                            &repo_url,
                            &version.to_string(),
                            &format!("Release {}", version),
                            release_notes,
                        )
                        .await
                    })?;
                }
                _ => unreachable!(),
            }
        }
    }

    Ok(())
}

fn handle_git_initialization() -> Result<()> {
    if Path::new(".git").exists() {
        println!("This directory is already a Git repository");
        return Ok(());
    }
    initialize_git_repo()?;
    println!("✔ Successfully initialized new Git repository");
    println!("✔ Created .gitignore file");
    println!("✔ Created initial commit");
    Ok(())
}
