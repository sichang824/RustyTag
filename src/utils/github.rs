use anyhow::{Context, Result};
use semver::Version;
use serde_json::json;

pub struct GitHubClient {
    token: String,
    repo_url: String,
}

impl GitHubClient {
    pub fn new(token: String, repo_url: String) -> Self {
        Self { token, repo_url }
    }

    /// 从环境变量或配置文件获取 GitHub token
    pub fn from_env_or_config() -> Result<String> {
        if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            Ok(token)
        } else {
            let config = crate::utils::config::Config::load()?;
            config.github_token.ok_or_else(|| {
                anyhow::anyhow!(
                    "GitHub token not found. To set it up:\n\
                     1. Visit https://github.com/settings/tokens\n\
                     2. Click 'Generate new token' (classic)\n\
                     3. Select the 'repo' scope\n\
                     4. Generate and copy the token\n\
                     5. Set it using:\n\
                        rustytag config --set GITHUB_TOKEN=your_token"
                )
            })
        }
    }

    /// 创建 GitHub Release
    pub async fn create_release(&self, version: &Version) -> Result<()> {
        // 读取 changelog 内容
        let changelog = std::fs::read_to_string("CHANGELOG.md")?;
        let release_notes = changelog
            .split("\n\n")
            .find(|section| section.starts_with(&format!("### [{}]", version)))
            .unwrap_or("No changelog content");

        // 从 repo_url 提取 owner 和 repo
        let parts: Vec<&str> = self.repo_url.trim_end_matches(".git").split('/').collect();
        let owner = parts[parts.len() - 2];
        let repo = parts[parts.len() - 1];

        let client = reqwest::Client::new();
        let url = format!("https://api.github.com/repos/{}/{}/releases", owner, repo);

        let response = client
            .post(url)
            .header("Accept", "application/vnd.github.v3+json")
            .header("Authorization", format!("token {}", self.token))
            .header("User-Agent", "RustyTag")
            .json(&json!({
                "tag_name": version.to_string(),
                "name": format!("Release {}", version),
                "body": release_notes,
                "draft": false,
                "prerelease": false
            }))
            .send()
            .await?;

        if response.status().is_success() {
            println!("✨ Successfully created GitHub release for {}", version);
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Failed to create release: {}",
                response.text().await?
            ))
        }
    }
}

/// 创建 GitHub Release 的便捷函数
pub async fn create_github_release(version: &Version) -> Result<()> {
    let token = GitHubClient::from_env_or_config()?;
    let repo_url = crate::utils::git::get_remote_url()?;

    let client = GitHubClient::new(token, repo_url);
    client.create_release(version).await
}
