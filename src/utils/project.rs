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

        if Path::new(".rustytag.json").exists() {
            println!("Detected rustytag project file: .rustytag.json");
            files.push(Self {
                path: PathBuf::from(".rustytag.json"),
                file_type: ProjectFileType::Other,
            });
        }

        Ok(files)
    }
}
