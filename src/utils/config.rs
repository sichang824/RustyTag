use anyhow::Result;
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub github_token: Option<String>,
}

#[derive(Debug)]
pub enum ConfigKey {
    GithubToken,
}

impl FromStr for ConfigKey {
    type Err = anyhow::Error;

    fn from_str(key: &str) -> Result<Self, Self::Err> {
        match key {
            "GITHUB_TOKEN" => Ok(ConfigKey::GithubToken),
            _ => Err(anyhow::anyhow!("Unknown configuration key: {}", key)),
        }
    }
}

impl ConfigKey {
    fn as_str(&self) -> &'static str {
        match self {
            ConfigKey::GithubToken => "GITHUB_TOKEN",
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = get_config_path()?;
        if !config_path.exists() {
            return Ok(Config::default());
        }
        let content = fs::read_to_string(config_path)?;
        Ok(serde_json::from_str(&content)?)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = get_config_path()?;
        // 确保配置目录存在
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match ConfigKey::from_str(key)? {
            ConfigKey::GithubToken => self.github_token = Some(value.to_string()),
        }
        self.save()?;
        println!("✔ Configuration saved to {}", get_config_path()?.display());
        Ok(())
    }

    pub fn display(&self) {
        println!("\n⚙️  Current Configuration");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        self.display_value(ConfigKey::GithubToken);
        println!();
    }

    fn display_value(&self, key: ConfigKey) {
        match key {
            ConfigKey::GithubToken => {
                if let Some(token) = &self.github_token {
                    println!(
                        "🔑 {}: {}...{}",
                        key.as_str(),
                        &token[..7],
                        &token[token.len() - 4..]
                    );
                } else {
                    println!("🔑 {}: Not set", key.as_str());
                }
            }
        }
    }
}

fn get_config_path() -> Result<PathBuf> {
    let home = home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    Ok(home.join(".rustytag").join("config.json"))
}

/// 处理配置命令的便捷函数
pub fn handle_config_command(set: Option<String>) -> Result<()> {
    let mut config = Config::load()?;
    if let Some(set_str) = set {
        let parts: Vec<&str> = set_str.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid format. Use KEY=VALUE"));
        }
        config.set(parts[0], parts[1])?;
        println!("✔ Configuration updated");
    } else {
        config.display();
    }
    Ok(())
}
