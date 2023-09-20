use serde::{Deserialize, Serialize};

use crate::flow_model::FlowModel;
use crate::package_manager::PackageManager;

use std::collections::HashMap;
use std::fs;

use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde_json;

use anyhow::Result;

use crate::flow_model::{CodeEmitter, StandardCodeEmitter};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlowPackage {
    name: String,
    version: String,
    path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlowProject {
    name: String,
    version: String,
    packages: Vec<FlowPackage>,
    flow: FlowModel,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlowProjectManagerConfig{
    
    #[serde(default = "project_folder_default")]
    project_folder: String,
    
    #[serde(default = "project_json_file_name_default")]
    project_json_file_name: String,
    
    #[serde(default = "builtin_dependencies_default")] 
    builtin_dependencies: Vec<String>,
    
    #[serde(default = "rust_fmt_path_default")] 
    rust_fmt_path: String,

    #[serde(default = "do_formatting_default")] 
    do_formatting: bool
}

impl Default for FlowProjectManagerConfig {
    fn default() -> Self {
        Self {
            project_folder: project_folder_default(),
            project_json_file_name: project_json_file_name_default(),
            builtin_dependencies: builtin_dependencies_default(),
            rust_fmt_path: rust_fmt_path_default(),
            do_formatting: do_formatting_default()
        }
    }
}


fn project_folder_default() -> String {
    "flow-projects".to_string()
}

fn project_json_file_name_default() -> String {
    "flow-project.json".to_string()
}

fn builtin_dependencies_default() -> Vec<String> {
    vec!["wasm-bindgen = \"0.2.87\"".to_string(), "serde_json = \"1.0.105\"".to_string()]
}

fn rust_fmt_path_default() -> String {
    "rustfmt".to_string()
}

const fn do_formatting_default() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlowProjectManager {
    config: FlowProjectManagerConfig,
    pub projects: HashMap<String, FlowProject>,
}

impl FlowProjectManager {
    pub fn new(config: FlowProjectManagerConfig) -> Self {
        Self {
            config: config,
            projects: HashMap::new(),
        }
    }

    pub fn load_projects(&mut self) -> Result<()> {
        self.projects.clear();
        for entry in fs::read_dir(&self.config.project_folder)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let folder_name = entry.file_name().to_string_lossy().to_string();
                let json_file_path = Path::new(&entry.path()).join(&self.config.project_json_file_name);
                if json_file_path.exists() {
                    let json_content = fs::read_to_string(&json_file_path)?;
                    let flow_project: FlowProject = serde_json::from_str(&json_content)?;
                    self.projects.insert(folder_name, flow_project);
                }
            }
        }
        Ok(())
    }

    pub fn create_flow_project(
        &mut self,
        flow_project: FlowProject,
        package_manager: &PackageManager,
    ) -> Result<(FlowProject)> {
        if self.projects.contains_key(&flow_project.name) {
            return Ok(flow_project);
        }

        self.projects
            .insert(flow_project.name.clone(), flow_project.clone());

        self.create_flow_project_folder(&flow_project, package_manager)?;

        Ok(flow_project)
    }

    fn create_project_dependencies(&self, p: &FlowPackage) -> String {
        if let Some(path) = &p.path {
            format!("{} = {{path = \"{}\"}}", p.name, path)
        } else {
            format!("{} = \"{}\"", p.name, p.version)
        }
    }

    fn create_builtin_dependencies(&self) -> String {
        self.config.builtin_dependencies.join("\n")
    }

    fn create_cargo_toml(
        &self,
        flow_project: &FlowProject,
        project_folder_name: &PathBuf,
    ) -> Result<()> {
        let cargo_toml_path = project_folder_name.join("Cargo.toml");
        let cargo_toml_content =
            format!("[package]\n name = \"{}\" \n version = \"{}\"\nedition = \"2021\"\n\n[dependencies]\n{}\n{}\n\n[lib]\ncrate-type = [\"cdylib\"]", 
            flow_project.name,
            flow_project.version,
            flow_project.packages.iter().map(|x| self.create_project_dependencies(x)).collect::<Vec<String>>().join("\n"),
            self.create_builtin_dependencies()
        );

        let mut cargo_toml_file = fs::File::create(&cargo_toml_path)?;
        cargo_toml_file.write_all(cargo_toml_content.as_bytes())?;

        Ok(())
    }

    fn create_flow_proj_json(
        &self,
        flow_project: &FlowProject,
        project_folder_name: &PathBuf,
    ) -> Result<()> {
        let flow_project_json_path = project_folder_name.join(&self.config.project_json_file_name);
        let flow_project_json_content = serde_json::to_string(&flow_project)?;
        let mut flow_project_json_file = fs::File::create(&flow_project_json_path)?;
        flow_project_json_file.write_all(flow_project_json_content.as_bytes())?;

        Ok(())
    }

    fn create_flow_rust_code(
        &self,
        flow_project: &FlowProject,
        src_folder: &PathBuf,
        package_manager: &PackageManager,
    ) -> Result<()> {
        let emitter = StandardCodeEmitter {};
        let flow_code = emitter.emit_flow_code(&flow_project.flow, package_manager);
        let flow_code_path = src_folder.join("lib.rs");
        let mut flow_code_file = fs::File::create(&flow_code_path)?;
        flow_code_file.write_all(flow_code.as_bytes())?;

        self.run_rust_fmt(&flow_code_path);

        Ok(())
    }

    fn run_rust_fmt(&self, file_path: &PathBuf) {
        let mut command = std::process::Command::new(&self.config.rust_fmt_path);
        command.arg(file_path.to_str().unwrap());

        let status = command.output().unwrap();

        //TODO: better error reporting. also: make fmt optional and add the possibility to change its path.
        if status.status.code() != Some(0) {
            println!(
                "An error occurred while formatting {}: {}",
                file_path.to_string_lossy(),
                String::from_utf8(status.stderr).expect("")
            );
        }
    }

    fn create_flow_project_folder(
        &self,
        flow_project: &FlowProject,
        package_manager: &PackageManager,
    ) -> Result<()> {
        // Create the main project folder using the FlowProject's name
        let project_folder_name = Path::new(&self.config.project_folder).join(&flow_project.name);
        fs::create_dir(&project_folder_name)?;

        // Create the 'src' subfolder
        let src_folder = project_folder_name.join("src");
        fs::create_dir(&src_folder)?;

        // Create the 'flow-project.json' file and serialize the FlowProject object
        self.create_flow_proj_json(flow_project, &project_folder_name)?;

        // Create the 'Cargo.toml' file
        self.create_cargo_toml(flow_project, &project_folder_name)?;

        // Create the flow as rust source
        self.create_flow_rust_code(flow_project, &src_folder, package_manager)?;

        Ok(())
    }

    pub fn delete_flow_project(&mut self, name: &str) -> Result<()> {
        if !self.projects.contains_key(name) {
            return Ok(());
        }

        if let Err(err) = delete_folder_recursive(&PathBuf::from(&self.config.project_folder).join(name)) {
            return Err(err.into());
        } else {
            self.projects.remove(name);
        }

        Ok(())
    }

    pub fn update_flow_project_flow_model(&mut self, name: &str, flow: FlowModel) -> Result<()> {
        if let Some(fp) = self.projects.get_mut(name) {
            fp.flow = flow;

            // Write project file.
            let project_folder_name = Path::new(&self.config.project_folder).join(&fp.name);
            let flow_model_json_content = serde_json::to_string(&fp)?;
            let flow_project_json_path = project_folder_name.join(&self.config.project_json_file_name);
            replace_file_contents(&flow_project_json_path, &flow_model_json_content)?;

            // Update Cargo.toml TODO

            // Update flow code (lib.rs) TODO
        }

        Ok(())
    }
}

fn delete_folder_recursive(folder_path: &Path) -> io::Result<()> {
    if folder_path.is_dir() {
        for entry in fs::read_dir(folder_path)? {
            let entry = entry?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                delete_folder_recursive(&entry_path)?;
            } else {
                fs::remove_file(&entry_path)?;
            }
        }
        fs::remove_dir(folder_path)?;
    }
    Ok(())
}

fn replace_file_contents(file_path: &Path, new_content: &str) -> io::Result<()> {
    let mut file = fs::File::create(file_path)?;
    file.write_all(new_content.as_bytes())?;
    Ok(())
}
