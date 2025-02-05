use anyhow::Result;
use std::path::{Path, PathBuf};

pub enum ProjectFileType {
    CargoToml,
    PackageJson,
    PyProjectToml,
    Other,
}

pub struct ProjectFile {
    pub path: PathBuf,
    pub file_type: ProjectFileType,
}

impl ProjectFile {
    pub fn detect_all() -> Result<Vec<Self>> {
        let mut files = Vec::new();

        // 检查并添加所有存在的项目文件
        if Path::new("Cargo.toml").exists() {
            println!("Detected Cargo.toml file");
            files.push(Self {
                path: PathBuf::from("Cargo.toml"),
                file_type: ProjectFileType::CargoToml,
            });
        }

        if Path::new("package.json").exists() {
            println!("Detected package.json file");
            files.push(Self {
                path: PathBuf::from("package.json"),
                file_type: ProjectFileType::PackageJson,
            });
        }

        if Path::new("pyproject.toml").exists() {
            println!("Detected pyproject.toml file");
            files.push(Self {
                path: PathBuf::from("pyproject.toml"),
                file_type: ProjectFileType::PyProjectToml,
            });
        }

        if Path::new(".rustytag").exists() {
            println!("Detected custom project file: .rustytag");
            files.push(Self {
                path: PathBuf::from(".rustytag"),
                file_type: ProjectFileType::Other,
            });
        }

        if files.is_empty() {
            Err(anyhow::anyhow!("Project file not found"))
        } else {
            Ok(files)
        }
    }
}
