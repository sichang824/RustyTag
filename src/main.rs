use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use git2::Repository;
use std::io::{self, Write};
use std::path::Path;

mod utils;

use utils::{
    file::create_changelog,
    git::{
        add_project_files, commit_changes, create_tag, get_project_info, initialize_git_repo,
        reset_tags,
    },
    version::{get_latest_version, update_version_to_project, BumpType, Version},
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Git repository
    Init,
    /// Bump patch version (e.g., 1.0.0 → 1.0.1)
    Patch {
        /// Set specific version instead of bumping
        #[arg(short = 'V', long)]
        version: Option<String>,
    },
    /// Bump minor version (e.g., 1.0.0 → 1.1.0)
    Minor {
        /// Set specific version instead of bumping
        #[arg(short = 'V', long)]
        version: Option<String>,
    },
    /// Bump major version (e.g., 1.0.0 → 2.0.0)
    Major {
        /// Set specific version instead of bumping
        #[arg(short = 'V', long)]
        version: Option<String>,
    },
    /// Reset local tags to match the remote repository
    Reset,
    /// Show the current version
    Show,
    /// Create or list releases
    Release {
        /// List all available releases
        #[arg(short = 'l', long)]
        list: bool,
        /// Create a release for the specified version
        #[arg(short = 't', long)]
        tag: Option<String>,
    },
    /// Synchronize local tags with the remote repository
    Sync,
    /// Manage RustyTag configuration settings
    Config {
        /// Set a configuration value (e.g., GITHUB_TOKEN=xxx)
        #[arg(short, long)]
        set: Option<String>,
        /// Set as global configuration
        #[arg(short, long)]
        global: bool,
        /// Set as local configuration
        #[arg(short, long)]
        local: bool,
    },
}

fn show_project_info(repo: &Repository) -> Result<()> {
    let info = get_project_info(repo)?;
    println!("\n📦 Project Information");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🏷️  Current Version: {}", info.version);
    if let Some(branch) = info.branch_name {
        println!("🌿 Current Branch: {}", branch);
    }
    println!("📝 Commit Count: {}", info.commit_count);
    if let Some(url) = info.repo_url {
        println!("🔗 Repository URL: {}", url);
    }
    println!("\n🛠️  RustyTag Information");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📋 Version: {}", env!("CARGO_PKG_VERSION"));
    println!("📘 Description: A Git tag and version management tool");
    println!("🏠 Homepage: https://github.com/sichang824/rustytag");
    println!("👤 Author: sichang <sichang824@gmail.com>");
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
            match &cli.command {
                Commands::Patch { version } | Commands::Minor { version } | Commands::Major { version } => {
                    let bump_type = match cli.command {
                        Commands::Patch { .. } => BumpType::Patch,
                        Commands::Minor { .. } => BumpType::Minor,
                        Commands::Major { .. } => BumpType::Major,
                        _ => unreachable!(),
                    };
                    
                    let new_version = if let Some(version_str) = version {
                        Version::parse(version_str).context("Invalid version format")?
                    } else {
                        let latest_version = get_latest_version()?;
                        latest_version.bump(bump_type)
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
                Commands::Release { tag, list } => {
                    if *list {
                        tokio::runtime::Runtime::new()?
                            .block_on(async { utils::github::list_github_releases().await })?;
                    } else {
                        let version = if let Some(tag_str) = tag {
                            Version::parse(tag_str).context("Invalid version format")?
                        } else {
                            get_latest_version()?
                        };

                        print!(
                            "\n🚀 Are you sure you want to create release {}? [y/N] ",
                            version
                        );
                        io::stdout().flush()?;

                        let mut input = String::new();
                        io::stdin().read_line(&mut input)?;

                        if input.trim().to_lowercase() == "y" {
                            tokio::runtime::Runtime::new()?.block_on(async {
                                utils::github::create_github_release(&version).await
                            })?;
                        } else {
                            println!("❌ Release cancelled");
                        }
                    }
                }
                Commands::Sync => {
                    utils::git::show_and_sync_tags(&repo)?;
                }
                Commands::Config { set, global, local } => {
                    utils::config::handle_config_command(set.clone(), *global, *local)?;
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
