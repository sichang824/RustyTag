use anyhow::Result;
use serde_json::json;

pub async fn create_release(
    token: &str,
    repo_url: &str,
    tag: &str,
    name: &str,
    body: &str,
) -> Result<()> {
    // 从 repo_url 提取 owner 和 repo
    // 例如 https://github.com/sichang824/rustytag -> (sichang824, rustytag)
    let parts: Vec<&str> = repo_url
        .trim_end_matches(".git")
        .split('/')
        .collect();
    let owner = parts[parts.len() - 2];
    let repo = parts[parts.len() - 1];

    let client = reqwest::Client::new();
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases",
        owner, repo
    );

    let response = client
        .post(url)
        .header("Accept", "application/vnd.github.v3+json")
        .header("Authorization", format!("token {}", token))
        .header("User-Agent", "RustyTag")
        .json(&json!({
            "tag_name": tag,
            "name": name,
            "body": body,
            "draft": false,
            "prerelease": false
        }))
        .send()
        .await?;

    if response.status().is_success() {
        println!("✨ Successfully created GitHub release for {}", tag);
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Failed to create release: {}",
            response.text().await?
        ))
    }
} 