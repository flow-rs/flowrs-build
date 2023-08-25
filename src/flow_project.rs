use serde::{Deserialize, Serialize};

use crate::version::Version;
use crate::flow_model::FlowModel; 

use std::fs;
use std::collections::HashMap;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::io;

use serde_json;

use anyhow::{Result};

use crate::flow_model::{CodeEmitter, StandardCodeEmitter};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlowPackage {
    name: String,
    version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlowProject {
    name: String,
    version: String,
    packages: Vec<FlowPackage>,
    flow: FlowModel
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlowProjectManager {
    project_folder: String,
    pub projects: HashMap<String, FlowProject>
}

impl FlowProjectManager {

    pub fn new(project_folder: &str) -> Self {
        Self {
            project_folder: project_folder.to_string(), 
            projects: HashMap::new()
        }
    }

    pub fn load_projects(&mut self) -> Result<()> {
        self.projects.clear();
        for entry in fs::read_dir(&self.project_folder)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let folder_name = entry.file_name().to_string_lossy().to_string();
                let json_file_path = Path::new(&entry.path()).join("flow-project.json");
                if json_file_path.exists() {
                    let json_content = fs::read_to_string(&json_file_path)?;
                    let flow_project: FlowProject = serde_json::from_str(&json_content)?;
                    self.projects.insert(folder_name, flow_project);
                }
            }
        }
        Ok(())
    }

    pub fn create_flow_project(&mut self, flow_project: FlowProject ) -> Result<(FlowProject)> { 
        if self.projects.contains_key(&flow_project.name) {
            return Ok(flow_project);
        }

        self.projects.insert(flow_project.name.clone(), flow_project.clone());
        
        self.create_flow_project_folder(&flow_project)?;

        Ok(flow_project)
    }

    fn create_flow_project_folder(&self, flow_project: &FlowProject) -> Result<()> {
        // Create the main project folder using the FlowProject's name
        let project_folder_name = Path::new(&self.project_folder).join(&flow_project.name);
        fs::create_dir(&project_folder_name)?;
    
        // Create the 'src' subfolder
        let src_folder = project_folder_name.join("src");
        fs::create_dir(&src_folder)?;
    
        // Create the 'flow-project.json' file and serialize the FlowProject object
        let flow_project_json_path = project_folder_name.join("flow-project.json");
        let flow_project_json_content = serde_json::to_string(&flow_project)?;
        let mut flow_project_json_file = fs::File::create(&flow_project_json_path)?;
        flow_project_json_file.write_all(flow_project_json_content.as_bytes())?;
    
        // Create the 'Cargo.toml' file
        let cargo_toml_path = project_folder_name.join("Cargo.toml");
        let cargo_toml_content = "[package]\nname = \"".to_string() + &flow_project.name + "\"\nversion = \"0.1.0\"\n";
        let mut cargo_toml_file = fs::File::create(&cargo_toml_path)?;
        cargo_toml_file.write_all(cargo_toml_content.as_bytes())?;
    
        // Create the flow as rust source
        let emitter = StandardCodeEmitter{};
        let flow_code = emitter.emit_flow_code(&flow_project.flow);
        let flow_code_path = src_folder.join("flow.rs");
        let mut flow_code_file = fs::File::create(&flow_code_path)?;
        flow_code_file.write_all(flow_code.as_bytes())?;
    
        Ok(())
    }

    pub fn delete_flow_project(&mut self, name: &str) -> Result<()>{
        if !self.projects.contains_key(name) {
            return Ok(());
        }
    
        if let Err(err) = delete_folder_recursive(&PathBuf::from(&self.project_folder).join(name)) {
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
            let project_folder_name = Path::new(&self.project_folder).join(&fp.name);
            let flow_model_json_content = serde_json::to_string(&fp)?;
            let flow_project_json_path = project_folder_name.join("flow-project.json");
            replace_file_contents(&flow_project_json_path, &flow_model_json_content)?;
        
            // Update Cargo.toml TODO 

            // Update flow code (flow.rs)
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