# flowrs-build
Tools for flow development. This repository contains the Frontend (flowrs designer) and the
corresponding REST-API.

## Getting started
1. Clone the repository.
2. Start both the backend service and the frontend server with docker using:
   ```docker compose up --build```. With docker compose you are starting also the analytics with grafana.
3. navigate to localhost:3001 in your browser to access the frontend
4. to test the backend, use the http requests listed in the rest.http file or use the browser ui
    - create a new project
    - compile the project
    - run the project
    - use the process endpoint to retrieve logs

## Frontend
The code for the frontend is located in `flowrs-build-designer`. It is developed using
Nuxt and Vue.js in combination with Rete.js.

### Run Frontend without docker
1. Go into the directory `flowrs-build-designer`.
2. Copy the file `.env.example` and rename to `.env` to set all environment variables. 
3. Run `yarn install`. After installing all dependencies you can start the frontend with `yarn run dev`.

## Backend
The backend is located inside `flowrs-build`. It is programmed with Rust and is used as REST-API.

### Run Backend without docker
1. Run ``cargo build`` to install the dependencies.
2. Copy the file `.env.example` and rename to `.env` to set all environment variables.
3. Start the project with the `service_main` located in `./target/debug` with the command `target/debug/service_main
   ` (depending on the operating system). Further information below.

## Service 
REST service to create and maintain new flow projects and flow packages.
Code is located in src/bin/service_main.rs. 
Call ./service_main[.exe] --help for instructions. The executable is located in target/{debug|release}.
The config file has the following format: 
```json
{
   "flow_project_manager_config":{
      "project_folder":"flow-projects",
      "project_json_file_name":"flow-project.json",
      "builtin_dependencies":[
         "wasm-bindgen = \"0.2.87\"",
         "serde_json = \"1.0.105\""
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
 ./target/debug/service_main.exe
```

The flag --config-file is replaced by an environment variable. You can set the variable in the `.env` file in top-level directory or directly in the `docker-compose.yml`.

### Endpoints

| HTTP-Method | Endpoint URL                           | Description                                                                                       |
|-------------|----------------------------------------|---------------------------------------------------------------------------------------------------|
| GET         | /packages/:package\_name               | Get a specific flow-package by name.                                                              |
| GET         | /packages/                             | Get all flow-packages.                                                                            |
| POST        | /projects/                             | Create a new project with a flow-project.json file.                                               |
| GET         | /projects/                             | Get all projects from backend.                                                                    |
| DELETE      | /projects/:project\_name/              | Delete a project by name.                                                                         |
| POST        | /projects/:project\_name/compile       | Compile a project (project\_name). Specify the build\_type with a query param (wasm or cargo).    |
| GET         | /projects/:project\_name/last\_compile | Get last compile time of the project. Specify the build\_type with a query param (wasm or cargo). |
| POST        | /projects/:project\_name/run           | Run a specific project as a process. Specify the build\_type with a query param (wasm or cargo).  |
| POST        | /processes/:process\_id/stop           | Stop the process with the process id returned by the run request.                                 |
| GET         | /processes/:process\_id/logs           | Get the current log output of a process with the process id.                                      |



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
## Desktop Runner 
Console application to run flows compiled to shared objects. 
Code is located in src/bin/runner_main.rs. 
Call ``./runner_main[.exe] --help` for instructions. 
To compile a flow for execution on the desktop, execute the following steps 
1. Goto the flow-project folder [flow-project].
2. run `cargo build` which will generate a the shared object file (*.dll or *.so) in target/[debug|release] (in this case debug).
3. run `.\runner_main.exe  --flow [flow-project]\target\[debug|release]\[flow-project].[dll|so] --workers [number of worker threads]`
4. stop flow execution with `ctrl+C`.
   
## Browser Runner
Flow projects also run in the browser. 
To compile a flow for execution in the browser, execute the following steps: 
1. Goto the flow-project folder [flow-project].
2. run `wasm-pack build --release --target web` which will generate a the shared object file (*.dll or *.so) in target/[debug|release] (in this case: release).
3. run `python -m http.server` in the very same directory (or any other webserver)
4. Open your browser and browse to `localhost:8000`
5. Open your browser's console viewer. 


## Testing and Code Coverage

Running ```cargo build``` locally does not run any unit tests. Normally, one would use ```cargo test``` for this purpose. To easily integrate [source based code coverage](https://doc.rust-lang.org/rustc/instrument-coverage.html) in rust, you can use the **test.sh** script on *nix machines and MacOs and the **test.ps1** script on Windows machines. This runs the unit tests and generates very accurate code coverage, which is better that most coverage plugins for rust right now, as it does support correct branch coverage tracking. The scripts generate coverage data in form of an **coverage.lcov** file ([https://github.com/linux-test-project/lcov](https://github.com/linux-test-project/lcov)), which can easily be interpreted by most coverage tools. For Visual Studio Code Integration, we recommend the [Coverage Gutters](https://marketplace.visualstudio.com/items?itemName=ryanluker.vscode-coverage-gutters) Extension.

## Current Problems
Due to the fact that the live-analytics are using Grafana and an incompatible library
for wasm, an execution with wasm is currently not possible. To enable the execution with wasm changes need to be made in the following repositories and files:

1. flowrs-build / Cargo.toml: Replace the line `flowrs = { git = "https://github.com/flow-rs/flowrs", branch = "feature-project7"}` with
`flowrs = { git = "https://github.com/flow-rs/flowrs", branch = "dev"}`. --> Setting the dependency to dev.
2. flowrs-std / Cargo.toml: Replace the line `flowrs = {git = "https://github.com/flow-rs/flowrs", branch = "feature-project7"}` with
   `flowrs = { git = "https://github.com/flow-rs/flowrs", branch = "dev"}`. --> Setting the dependency to dev.
