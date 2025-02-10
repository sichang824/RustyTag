use crate::utils::project::{ProjectFile, ProjectFileType};
use anyhow::Result;
use semver;
use std::fs;
use toml_edit::{value, Document};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BumpType {
    Patch,
    Minor,
    Major,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub prefix: String,
    pub version: semver::Version,
    pub suffix: String,
}

impl Version {
    pub fn new(version: semver::Version) -> Self {
        Self {
            prefix: String::new(),
            version,
            suffix: String::new(),
        }
    }

    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    pub fn parse(version_str: &str) -> Result<Self> {
        // 分离前缀：找到第一个数字的位置
        let prefix_end = version_str.find(|c: char| c.is_ascii_digit()).unwrap_or(0);
        let prefix = &version_str[..prefix_end];

        // 分离后缀：找到版本号结束的位置
        let remaining = &version_str[prefix_end..];
        let version_end = remaining
            .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-' && c != '+')
            .unwrap_or(remaining.len());

        let version_str = &remaining[..version_end];
        let suffix = &remaining[version_end..];

        // 解析 semver 版本号
        let version = semver::Version::parse(version_str)
            .map_err(|e| anyhow::anyhow!("无法解析版本号: {}", e))?;

        Ok(Self {
            prefix: prefix.to_string(),
            version,
            suffix: suffix.to_string(),
        })
    }

    pub fn to_string(&self) -> String {
        format!("{}{}{}", self.prefix, self.version, self.suffix)
    }

    pub fn bump(&self, bump_type: BumpType) -> Self {
        let new_version = match bump_type {
            BumpType::Major => semver::Version::new(self.version.major + 1, 0, 0),
            BumpType::Minor => semver::Version::new(self.version.major, self.version.minor + 1, 0),
            BumpType::Patch => semver::Version::new(
                self.version.major,
                self.version.minor,
                self.version.patch + 1,
            ),
        };

        Self {
            prefix: self.prefix.clone(),
            version: new_version,
            suffix: self.suffix.clone(),
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

pub fn update_version_to_project(version: &Version) -> Result<()> {
    let version_files = ProjectFile::detect_all()?;
    for version_file in version_files {
        match version_file.file_type {
            ProjectFileType::CargoToml => update_cargo_toml(version)?,
            ProjectFileType::PackageJson => update_package_json(version)?,
            ProjectFileType::PyProjectToml => update_pyproject_toml(version)?,
            ProjectFileType::Other => {
                return Err(anyhow::anyhow!("Unsupported project file type"));
            }
        }
    }
    println!("✔ [Updated] version to {} in project files", version);
    Ok(())
}

fn update_cargo_toml(version: &Version) -> Result<()> {
    let cargo_toml = fs::read_to_string("Cargo.toml")?;
    let mut doc = cargo_toml.parse::<Document>()?;

    if let Some(package) = doc.get_mut("package") {
        if let Some(package) = package.as_table_mut() {
            package.insert("version", value(version.version.to_string()));
        }
    }

    fs::write("Cargo.toml", doc.to_string())?;
    Ok(())
}

fn update_package_json(version: &Version) -> Result<()> {
    let package_json = fs::read_to_string("package.json")?;
    let mut json: serde_json::Value = serde_json::from_str(&package_json)?;

    if let Some(obj) = json.as_object_mut() {
        obj.insert(
            "version".to_string(),
            serde_json::Value::String(version.version.to_string()),
        );
    }

    fs::write("package.json", serde_json::to_string_pretty(&json)?)?;
    Ok(())
}

fn update_pyproject_toml(version: &Version) -> Result<()> {
    let pyproject_toml = fs::read_to_string("pyproject.toml")?;
    let mut doc = pyproject_toml.parse::<Document>()?;

    if let Some(tool) = doc.get_mut("tool") {
        if let Some(poetry) = tool.get_mut("poetry") {
            if let Some(poetry) = poetry.as_table_mut() {
                poetry.insert("version", value(version.version.to_string()));
            }
        }
    }

    fs::write("pyproject.toml", doc.to_string())?;
    Ok(())
}

pub fn get_latest_version() -> Result<Version> {
    let version = crate::utils::git::get_latest_tag()?;

    // 如果有前缀但未配置，提示用户
    if !version.prefix.is_empty() {
        let config = crate::utils::config::LocalConfig::load()?;
        if config.version_prefix.is_none() {
            println!("⚠️ 检测到标签前缀 '{}' 但未配置", version.prefix);
            println!("ℹ️  您可以使用以下命令配置版本前缀：");
            println!("   rustytag config -s VERSION_PREFIX={}", version.prefix);
        }
    }

    Ok(version)
}
