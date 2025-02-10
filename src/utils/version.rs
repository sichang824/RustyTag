use crate::utils::project::{ProjectFile, ProjectFileType};
use anyhow::Result;
use semver::Version;
use std::fs;
use toml_edit::{value, Document};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BumpType {
    Patch,
    Minor,
    Major,
}

pub fn bump_version(version: &Version, bump: BumpType) -> Version {
    match bump {
        BumpType::Patch => Version::new(version.major, version.minor, version.patch + 1),
        BumpType::Minor => Version::new(version.major, version.minor + 1, 0),
        BumpType::Major => Version::new(version.major + 1, 0, 0),
    }
}

pub fn update_version_to_project(version: &Version) -> Result<()> {
    let formatted_version = format_version(version)?;
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
    println!(
        "✔ [Updated] version to {} in project files",
        formatted_version
    );
    Ok(())
}

fn update_cargo_toml(version: &Version) -> Result<()> {
    let cargo_toml = fs::read_to_string("Cargo.toml")?;
    let mut doc = cargo_toml.parse::<Document>()?;

    if let Some(package) = doc.get_mut("package") {
        if let Some(package) = package.as_table_mut() {
            package.insert("version", value(version.to_string()));
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
            serde_json::Value::String(version.to_string()),
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
                poetry.insert("version", value(version.to_string()));
            }
        }
    }

    fs::write("pyproject.toml", doc.to_string())?;
    Ok(())
}

pub fn get_latest_version() -> Result<Version> {
    let latest_tag = crate::utils::git::get_latest_tag()?;

    if latest_tag == "initial" {
        return Ok(Version::new(0, 1, 0));
    }

    let version_str = latest_tag.trim_start_matches(|c: char| !c.is_ascii_digit());
    Ok(Version::parse(version_str)?)
}

pub fn format_version(version: &Version) -> Result<String> {
    let config = crate::utils::config::Config::load()?;
    let prefix = config.version_prefix.unwrap_or_default();
    Ok(format!("{}{}", prefix, version))
}
