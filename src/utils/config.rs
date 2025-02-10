use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct GlobalConfig {
    pub github_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct LocalConfig {
    pub version_prefix: Option<String>,
}

impl GlobalConfig {
    fn config_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("~"))
            .join(".rustytag/config.json")
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}

impl LocalConfig {
    fn config_path() -> PathBuf {
        PathBuf::from(".rustytag.json")
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}

pub fn handle_config_command(set: Option<String>, global: bool, local: bool) -> Result<()> {
    if let Some(set_str) = set {
        let parts: Vec<&str> = set_str.split('=').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid format. Use KEY=VALUE"));
        }

        // 如果没有指定 local 或 global，根据配置类型自动选择
        let (is_global, is_local) = if !global && !local {
            match parts[0] {
                "GITHUB_TOKEN" => (true, false),
                "VERSION_PREFIX" => (false, true),
                _ => return Err(anyhow::anyhow!("Unknown configuration key")),
            }
        } else {
            (global, local)
        };

        match (parts[0], is_global, is_local) {
            ("GITHUB_TOKEN", true, _) => {
                let mut config = GlobalConfig::load()?;
                config.github_token = Some(parts[1].to_string());
                config.save()?;
                println!("✔ Global configuration saved");
            }
            ("VERSION_PREFIX", _, true) => {
                let mut config = LocalConfig::load()?;
                config.version_prefix = Some(parts[1].to_string());
                config.save()?;
                println!("✔ Local configuration saved");
            }
            _ => return Err(anyhow::anyhow!(
                "Invalid configuration: GITHUB_TOKEN must be global, VERSION_PREFIX must be local"
            )),
        }
    } else {
        // 显示当前配置
        println!("\n⚙️  Current Configuration");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        // 显示全局配置
        println!("🌍 Global Configuration:");
        let global_config = GlobalConfig::load()?;
        println!(
            "  🔑 GITHUB_TOKEN: {}",
            global_config
                .github_token
                .as_ref()
                .map(|t| format!("{}...{}", &t[..6], &t[t.len() - 4..]))
                .unwrap_or_else(|| "Not set".to_string())
        );

        // 显示本地配置
        println!("\n📍 Local Configuration:");
        let local_config = LocalConfig::load()?;
        println!(
            "  📌 VERSION_PREFIX: {}",
            local_config.version_prefix.as_deref().unwrap_or("Not set")
        );
        println!();
    }

    Ok(())
}
