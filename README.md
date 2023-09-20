# flowrs-build
Tools for flow development. Two different tools: 

## Service 
REST service to create and maintain new flow projects and flow packages.
Code is located in src/bin/service_main.rs. 
Call ./service_main[.exe] --help for instructions. 
The config file has the following format: 
```json
{
   "flow_project_manager_config":{
      "project_folder":"flow-projects",
      "project_json_file_name":"flow-project.json",
      "builtin_dependencies":[
         "wasm-bindgen = "0.2.87",
         "serde_json = "1.0.105"
      ],
      "rust_fmt_path":"rustfmt",
      "do_formatting":true
   },
   "flow_packages_folder":"flow-packages"
}
```
All fields are not mandatory. However, it is important that `flow_package_folder` is set to a folder with all necessary packages.

**Example** (Windows Powershell):
```bash
 .\service_main.exe --config-file config.json
```
Runs the service with a config file named "config.json". 

## Runner 
Console application to run flows compiled to shared objects. 
Code is located in src/bin/runner_main.rs. 
Call ./runner_main[.exe] --help for instructions. 

**Example** (Windows Powershell):
```bash
 .\runner_main.exe  --flow ..\..\..\flow_project_78\target\debug\flow_project_78.dll --workers 4
```
Runs a flow flow_project_78[.dll] with 4 worker threads.
