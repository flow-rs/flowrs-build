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
### Endpoints

- /packages/[package_name]: GET (get description of package [package_name])  
- /packages/: GET (get all package descriptions)
- /projects/: GET (get all project descriptions), POST (create a new project)


**Example** (minimal package description)
```json
{
    "name":"flowrs",
    "version":"1.0.0",
    "crates":{
    }
}
```
**Example** (project description: A timer node regularly triggers a debug node that outputs the number 42)
```json
{
  "name": "flow_project_79",
  "version": "1.0.0",
  "packages": [
    {"name": "flowrs", "version": "1.0.0", "path": "../../../flowrs"}, 
    {"name": "flowrs-std", "version": "1.0.0", "path": "../../../flowrs-std"}
    ],
  "flow":{        
        "nodes": {
            "debug_node": {
                "node_type": "flowrs_std::nodes::debug::DebugNode",
                "type_parameters": {"I": "i32"},
                "constructor": "New"

            },
            "timer_config_node": {
                "node_type": "flowrs_std::nodes::value::ValueNode",
                "type_parameters": {"I": "flowrs_std::nodes::timer::TimerNodeConfig"},
                "constructor": "New"

            },
            "timer_token_node": {
                "node_type": "flowrs_std::nodes::value::ValueNode",
                "type_parameters": {"I": "i32"},
                "constructor": "New"

            },
             "timer_node": {
                "node_type": "flowrs_std::nodes::timer::TimerNode",
                "type_parameters": {"T": "flowrs_std::nodes::timer::SelectedTimer", "U": "i32"},
                "constructor": "New"
            }
        },
        "connections": [
            {
                "from_node": "timer_config_node",
                "from_output": "output",
                "to_node": "timer_node",
                "to_input": "config_input"
            },
            {
                "from_node": "timer_token_node",
                "from_output": "output",
                "to_node": "timer_node",
                "to_input": "token_input"
            },
            {
                "from_node": "timer_node",
                "from_output": "token_output",
                "to_node": "debug_node",
                "to_input": "input"
            }
        ], 
        "data" : {
            "timer_config_node": {
                "value": {"duration": {"secs": 1, "nanos": 0}}
            },
             "timer_token_node": {
                "value": 42
            }
        }
    }
}
```
## Runner 
Console application to run flows compiled to shared objects. 
Code is located in src/bin/runner_main.rs. 
Call ./runner_main[.exe] --help for instructions. 

**Example** (Windows Powershell):
```bash
 .\runner_main.exe  --flow ..\..\..\flow_project_78\target\debug\flow_project_78.dll --workers 4
```
Runs a flow flow_project_78[.dll] with 4 worker threads.
