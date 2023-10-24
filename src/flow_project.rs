use serde::{Deserialize, Serialize};

use crate::flow_model::FlowModel;
use crate::package_manager::PackageManager;

use std::collections::{HashMap, VecDeque};
use std::{fs, thread};
use std::io;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::{Arc, Mutex};

use serde_json;
use handlebars::Handlebars;

use anyhow::Result;
use serde_json::from_str;
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
pub struct Process {
    process_id: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuildType {
    pub build_type: String,
}

pub struct FlowProcess {
    process: Child,
    outputs: Arc<Mutex<VecDeque<String>>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlowProjectManagerConfig {
    #[serde(default = "project_folder_default")]
    pub project_folder: String,

    #[serde(default = "project_json_file_name_default")]
    pub project_json_file_name: String,

    #[serde(default = "builtin_dependencies_default")]
    pub builtin_dependencies: Vec<String>,

    #[serde(default = "rust_fmt_path_default")]
    pub rust_fmt_path: String,

    #[serde(default = "do_formatting_default")]
    pub do_formatting: bool,
}

impl Default for FlowProjectManagerConfig {
    fn default() -> Self {
        Self {
            project_folder: project_folder_default(),
            project_json_file_name: project_json_file_name_default(),
            builtin_dependencies: builtin_dependencies_default(),
            rust_fmt_path: rust_fmt_path_default(),
            do_formatting: do_formatting_default(),
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

pub struct FlowProjectManager {
    config: FlowProjectManagerConfig,
    pub projects: HashMap<String, FlowProject>,
    processes: HashMap<u32, FlowProcess>,
}

impl FlowProjectManager {
    pub fn new(config: FlowProjectManagerConfig) -> Self {
        Self {
            config: config,
            projects: HashMap::new(),
            processes: HashMap::new(),
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

    pub fn compile_flow_project(
        &mut self,
        project_name: &str,
        build_type: String,
    ) -> Result<String, anyhow::Error> {
        // check if project exists
        let option_project = self.projects.get(project_name);
        if option_project.is_none() {
            return Err(anyhow::anyhow!("{project_name} does not exist!"));
        }

        // construct path to folder
        let project_folder_path = self.config.project_folder.clone();
        let flow_project_path = format!("{project_folder_path}/{project_name}");

        let output;
        if build_type.eq("cargo") {
            output = Self::compile_cargo(flow_project_path.clone());
        } else if build_type.eq("wasm") {
            output = Self::compile_wasm(flow_project_path.clone());
        } else {
            return Err(anyhow::anyhow!("{build_type} is not an allowed build_type"));
        }


        return if output.status.success() {
            Ok("Das Rust-Projekt wurde erfolgreich kompiliert.".parse()?)
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Das Rust-Projekt wurde nicht erfolgreich kompiliert.\nWorkingdirectory:{flow_project_path}\nError:\n {error}"))
        };
    }

    fn compile_cargo(flow_project_path: String) -> Output {
        // construct command for cargo build
        let mut binding = Command::new("cargo");
        let command = binding
            .current_dir(flow_project_path)
            .arg("build");

        // add release option if this rest-service is executed in release mode
        if !cfg!(debug_assertions) {
            command.arg("--release");
        }

        command
            .output()
            .expect("Fehler beim Ausführen von cargo build")
    }

    fn compile_wasm(flow_project_path: String) -> Output {
        let mut binding = Command::new("wasm-pack");
        let command = binding
            .current_dir(flow_project_path)
            .arg("build");

        // add release option if this rest-service is executed in release mode
        if !cfg!(debug_assertions) {
            command.arg("--release");
        }

        command
            .arg("--target")
            .arg("web")
            .output()
            .expect("Fehler beim Ausführen von wasm-pack build --release --target web")
    }

    pub fn run_flow_project(
        &mut self,
        project_name: &str,
        build_type: String,
    ) -> Result<Process, anyhow::Error> {
        let mut child;
        if build_type.eq("cargo") {
            child  = match self.run_cargo_project(project_name) {
                Ok(value) => value,
                Err(value) => return Err(value),
            };
        } else if build_type.eq("wasm") {
            child  = match self.run_wasm_project(project_name) {
                Ok(value) => value,
                Err(value) => return Err(value),
            };
        } else {
            return Err(anyhow::anyhow!("{build_type} is not an allowed build_type"));
        }

        // Create a VecDeque to store the combined output lines
        let outputs_mutex = Arc::new(Mutex::new(VecDeque::new()));

        Self::start_logs_export_thread(&mut child, outputs_mutex.clone());

        // save the new child process for later to be killed on request
        let id = child.id().clone();
        self.processes.insert(id, FlowProcess { outputs: outputs_mutex, process: child });
        Ok(Process { process_id: id })
    }

    fn run_cargo_project(&mut self, project_name: &str) -> Result<Child, anyhow::Error> {
        // get path to the projects executable
        let option_path_to_executable = self.get_path_to_executable(project_name, false);
        if option_path_to_executable.is_none() {
            return Err(anyhow::anyhow!("Couldn't find path to executable for project {project_name}"));
        }

        // execute runner_main --flow
        let runner_executable_path = if cfg!(debug_assertions) {
            "target/debug/runner_main"
        } else {
            "target/release/runner_main"
        };

        Ok(Command::new(runner_executable_path)
            .arg("--flow")
            .arg(option_path_to_executable.unwrap())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Fehler beim Ausführen"))
    }

    fn run_wasm_project(&mut self, project_name: &str) -> Result<Child, anyhow::Error>  {
        // get path to the projects executable
        let wasm_build_directory = self.get_path_to_executable(project_name, true);
        if wasm_build_directory.is_none() {
            return Err(anyhow::anyhow!("Couldn't find path to executable for project {project_name}"));
        }

        Ok(Command::new("python")
            .arg("-m")
            .arg("http.server")
            .current_dir(wasm_build_directory.unwrap())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Fehler beim Ausführen"))
    }

    fn get_path_to_executable(&mut self, project_name: &str, is_wasm: bool) -> Option<String> {
        let project_folder_path = self.config.project_folder.clone();
        let build_type = if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        };
        let project_dir_path= format!("{project_folder_path}/{project_name}");
        if is_wasm {
            return Some(project_dir_path);
        }
        let base_path = format!("{project_dir_path}/target/{build_type}");

        // name and ending combinations for windows, mac and linux
        let binding = format!("lib{project_name}");
        let possible_file_names = [project_name, binding.as_str()];
        let possible_file_endings = [".dll", ".dylib", ".so"];
        // find correct executable
        for possible_file_name in possible_file_names {
            for possible_file_ending in possible_file_endings {
                let formatted_path = format!("{base_path}/{possible_file_name}{possible_file_ending}");
                let possible_path_to_executable = Path::new(formatted_path.as_str());
                if possible_path_to_executable.exists() && possible_path_to_executable.to_str().is_some() {
                    let correct_path_to_executable = possible_path_to_executable.to_str().unwrap().to_string();
                    return Some(correct_path_to_executable);
                }
            }
        }

        return None;
    }

    fn start_logs_export_thread(child: &mut Child, thread_outputs_mutex: Arc<Mutex<VecDeque<String>>>) {
        // Spawn a thread to read and store both stdout and stderr lines
        let stdout = child.stdout.take().expect("Failed to capture stdout");
        let stderr = child.stderr.take().expect("Failed to capture stderr");
        thread::spawn(move || {
            let stdout_reader = BufReader::new(stdout);
            let stderr_reader = BufReader::new(stderr);

            for line in stdout_reader.lines().chain(stderr_reader.lines()) {
                let line = line.expect("Error reading line");
                thread_outputs_mutex.lock().unwrap().push_back(line);
            }
        });
    }

    pub fn stop_process(
        &mut self,
        process_id: String,
    ) -> Result<String, anyhow::Error> {
        let id = match Self::convert_to_process_id(process_id) {
            Ok(value) => value,
            Err(value) => return Err(value),
        };

        let process = match self.get_process(&id) {
            Ok(value) => value,
            Err(value) => return Err(value),
        };

        process.process.kill()?;
        Ok("Process killed".parse()?)
    }

    pub fn get_process_logs(
        &mut self,
        process_id: String,
    ) -> Result<Vec<String>, anyhow::Error> {
        let id = match Self::convert_to_process_id(process_id) {
            Ok(value) => value,
            Err(value) => return Err(value),
        };

        let process = match self.get_process(&id) {
            Ok(value) => value,
            Err(value) => return Err(value),
        };

        let mut outputs = process.outputs.lock().unwrap();
        let mut lines = Vec::new();
        for _i in 0..outputs.len() {
            let option = outputs.pop_front();
            if option.is_none() {
                break;
            }
            lines.push(option.unwrap())
        }

        return Ok(lines);
    }

    // convert to u32 type
    fn convert_to_process_id(process_id: String) -> Result<u32, anyhow::Error> {
        let result = from_str::<u32>(process_id.as_str());
        if result.is_err() {
            return Err(anyhow::anyhow!("supplied process_id wasn't of type u32"));
        }
        let id = result.unwrap();
        Ok(id)
    }

    fn get_process(&mut self, id: &u32) -> Result<&mut FlowProcess, anyhow::Error> {
        let process = self.processes.get_mut(&id);
        if process.is_none() {
            let msg = format!("No registered process found with id {}", id);
            return Err(anyhow::anyhow!(msg));
        }
        Ok(process.unwrap())
    }

    pub fn create_flow_project(
        &mut self,
        flow_project: FlowProject,
        package_manager: &PackageManager,
    ) -> Result<FlowProject, anyhow::Error> {
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
        let content =
            format!("[package]\n name = \"{}\" \n version = \"{}\"\nedition = \"2021\"\n\n[dependencies]\n{}\n{}\n\n[lib]\ncrate-type = [\"cdylib\"]",
                    flow_project.name,
                    flow_project.version,
                    flow_project.packages.iter().map(|x| self.create_project_dependencies(x)).collect::<Vec<String>>().join("\n"),
                    self.create_builtin_dependencies()
            );

        self.create_project_file(project_folder_name, &"Cargo.toml".to_string(), &content)
    }

    fn create_project_file(
        &self,
        folder_name: &PathBuf,
        file_name: &String,
        content: &String,
    ) -> Result<(), anyhow::Error> {
        let file_path = folder_name.join(file_name);
        let mut file = fs::File::create(&file_path)?;
        file.write_all(content.as_bytes())?;

        Ok(())
    }

    fn create_flow_proj_json(
        &self,
        flow_project: &FlowProject,
        project_folder_name: &PathBuf,
    ) -> Result<(), anyhow::Error> {
        let content = serde_json::to_string(&flow_project)?;
        self.create_project_file(project_folder_name, &self.config.project_json_file_name, &content)
    }

    fn create_index_html(
        &self,
        flow_project: &FlowProject,
        project_folder_name: &PathBuf,
    ) -> Result<(), anyhow::Error> {
        let mut handlebars = Handlebars::new();
        let source = r#"
        <!DOCTYPE html>
        <html>
          <head>
            <meta charset="UTF-8" />
            <title>{{project_name}} {{project_version}} </title>
          </head>
          <body>
            <script type="module">
              import init, {wasm_run} from '/pkg/{{project_name}}.js'
        
              // Always required for wasm.
              await init();

              // Running flow.
              wasm_run();
              
            </script>
          </body>
        </html>
        "#;

        handlebars.register_template_string("index", source)?;

        let mut data = HashMap::new();
        data.insert("project_name", &flow_project.name);
        data.insert("project_version", &flow_project.version);

        let content = handlebars.render("index", &data)?;

        self.create_project_file(project_folder_name, &"index.html".to_string(), &content)
    }

    fn create_flow_rust_code(
        &self,
        flow_project: &FlowProject,
        src_folder: &PathBuf,
        package_manager: &PackageManager,
    ) -> Result<(), anyhow::Error> {
        let emitter = StandardCodeEmitter {};
        let content = &emitter.emit_flow_code(&flow_project.flow, package_manager)?;

        self.create_project_file(src_folder, &"lib.rs".to_string(), &content)?;
        if self.config.do_formatting {
            self.run_rust_fmt(&src_folder.join("lib.rs"));
        }
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
    ) -> Result<(), anyhow::Error> {

        // Create the main project folder using the FlowProject's name
        let project_folder_name = Path::new(&self.config.project_folder).join(&flow_project.name);
        fs::create_dir(&project_folder_name)?;

        // Create the 'src' subfolder
        let src_folder = project_folder_name.join("src");
        fs::create_dir(&src_folder)?;

        self.create_flow_proj_json(flow_project, &project_folder_name)?;

        self.create_cargo_toml(flow_project, &project_folder_name)?;

        self.create_index_html(flow_project, &project_folder_name)?;

        self.create_flow_rust_code(flow_project, &src_folder, package_manager)?;

        Ok(())
    }

    pub fn delete_flow_project(&mut self, name: &str) -> Result<(), anyhow::Error> {
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

    pub fn update_flow_project_flow_model(&mut self, name: &str, flow: FlowModel) -> Result<(), anyhow::Error> {
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
