use anyhow::Result;
use std::path::PathBuf;

pub enum ProjectFileType {
    CargoToml,
    PackageJson,
    PyProjectToml,
    Other,
}

pub struct ProjectFile {
    #[allow(dead_code)]
    pub path: PathBuf,
    pub file_type: ProjectFileType,
}

impl ProjectFile {
    pub fn detect_all() -> Result<Vec<Self>> {
        let current_dir = std::env::current_dir()?;
        let mut files = Vec::new();

        // 检查并添加所有存在的项目文件
        let cargo_toml = current_dir.join("Cargo.toml");
        if cargo_toml.exists() {
            println!("Detected Cargo.toml file");
            files.push(Self {
                path: cargo_toml,
                file_type: ProjectFileType::CargoToml,
            });
        }

        let package_json = current_dir.join("package.json");
        if package_json.exists() {
            println!("Detected package.json file");
            files.push(Self {
                path: package_json,
                file_type: ProjectFileType::PackageJson,
            });
        }

        let py_project_toml = current_dir.join("pyproject.toml");
        if py_project_toml.exists() {
            println!("Detected pyproject.toml file");
            files.push(Self {
                path: py_project_toml,
                file_type: ProjectFileType::PyProjectToml,
            });
        }

        let other_file = current_dir.join("version.txt");
        if other_file.exists() {
            println!("Detected custom project file: version.txt");
            files.push(Self {
                path: other_file,
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
