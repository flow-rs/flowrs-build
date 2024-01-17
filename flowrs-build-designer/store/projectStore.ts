import {defineStore} from 'pinia'
import {type CompileError, type FlowProject, type ProjectIdentifier} from "~/repository/modules/projects";
import type {ProcessIdentifier} from "~/repository/modules/processes";


/**
 * Build type enum is used in UI dropdown.
 */
export enum BuildType {
    Wasm = "wasm",
    Cargo = "cargo",
}


/**
 * Defining a pinia store. It is used to store project related infos and to work with the api methods defined in the
 * repository.
 */
export const useProjectsStore = defineStore({
    id: 'projects',
    state: () => {
        return ({
            projects: [] as FlowProject[],
            selectedProject: null as FlowProject | null,
            selectedBuildType: BuildType.Cargo,
            loading: false,
            logEntriesMap: new Map() as Map<string, string[]>,
            runningProcessesMap: new Map() as Map<string, number | undefined>,
            compileErrorMap: new Map() as Map<string, CompileError[] | undefined>,
            compileTimestampMap: new Map() as Map<string, string | undefined>,
            projectClickedInList: false,
            errorMessage: "",
            showDialog: false,

        });
    },
    actions: {

        /**
         * Calling the api method to get all projects. Sets the UI State like the error message.
         */
        async getAll() {
            const {$api} = useNuxtApp();
            $api.projects.getProjects().then(listOfFlowProjects => {
                this.projects = listOfFlowProjects;
                this.setCurrentErrorMessage("")
            }).catch((error) => {
                const errorString = "Error fetching projects " + error
                this.setCurrentErrorMessage(errorString)
                console.log("Error fetching projects!")
            })

                .finally(() => (this.loading = false));
        },

        /**
         * Calling the api method to delete the project specified with project name. Sets the UI State like the error message.
         */
        async deleteProject(project_name?: string) {
            const {$api} = useNuxtApp();
            let name = ""
            if (project_name) {
                name = project_name
            } else {
                name = this.selectedProject!.name
            }
            const projectIdentifier: ProjectIdentifier = {
                project_name: name
            }
            $api.projects.deleteProject(projectIdentifier).then(response => {
                console.log("Flow Project was deleted:", response)
                // remove item from list in store to update the ui list
                this.projects = this.projects.filter((object) => {
                    return object.name != name
                })
                this.selectedProject = null
                this.projectClickedInList = false
            }).catch((error) => {
                this.setCurrentErrorMessage("Error deleting projects: " + error.data)
                console.log("Error deleting projects:" + error)
            })
                .finally(() => (this.loading = false))
        },

        /**
         * Calling the api method to run a project specified with project name and build type.
         * Sets the UI State like the error message.
         */
        async runProjectRequest(projectName: string, buildType: string) {
            const {$api} = useNuxtApp();
            const projectIdentifier: ProjectIdentifier = {
                project_name: projectName
            }
            this.loading = true
            $api.projects.runProject(projectIdentifier, buildType).then(response => {
                console.log("Flow Project is running!")
                this.writeLogEntry("Flow Projekt Ausführung wurde gestartet.")
                this.runningProcessesMap.set(projectIdentifier.project_name, response.process_id)
            }).catch((error) => {
                this.writeLogEntry(error)
                console.log("Error running project:" + error)
            })
                .finally(() => (this.loading = false))
        },

        /**
         * Calling the api method to stop the currently selected project.
         */
        async stopProcessRequest() {
            const {$api} = useNuxtApp();
            if (this.selectedProject !== null) {
                let processId = this.runningProcessesMap.get(this.selectedProject.name)
                if (processId != undefined && processId != -1) {
                    const processIdentifier: ProcessIdentifier = {
                        process_id: processId
                    }
                    this.loading = true
                    $api.processes.stopProcess(processIdentifier).then(response => {
                        console.log("Flow Project is stopped:", response)
                        if (this.selectedProject !== null) {
                            this.runningProcessesMap.set(this.selectedProject.name, undefined)
                        }
                        this.writeLogEntry("Ausführung vom Flow-Projekt gestoppt.")
                    }).catch((error) => {
                        this.writeLogEntry(error)
                        console.log("Error stopping project:" + error)
                    })
                        .finally(() => (this.loading = false))
                }
            }
        },

        /**
         * Calling the api method to compile a project specified with project name and build type.
         * Sets the UI State like the error message like the compile errors.
         */
        async compileProjectRequest(projectName: string, buildType: string) {
            const {$api} = useNuxtApp();
            const projectIdentifier: ProjectIdentifier = {
                project_name: projectName
            }
            this.loading = true
            $api.projects.compileProject(projectIdentifier, buildType).then(response => {
                console.log("Flow Project is compiled!")
                const response_txt = `${response}`
                this.writeLogEntry(response_txt)
                this.setCurrentCompileErrorsOfProject(projectName, undefined)
                if (this.selectedProject !== null) {
                    this.compileTimestampMap.set(this.selectedProject.name, this.getCurrentTimestamp())
                }
            }).catch((error) => {
                console.log("error compiling project")
                let converted = error.data as string
                let result = [] as CompileError[]
                const rawValues = this.extractErrors(converted)
                if (converted.includes('collect2: error: ld returned 1 exit status')) {
                    const object: CompileError = {
                        title: "collect2: error: ld returned 1 exit status",
                        message: converted
                    }
                    result.push(object)
                }
                for (error in rawValues) {
                    const errorTitle = rawValues[error].split('\\n').filter((line: string | string[]) => line.includes('error['));
                    const object: CompileError = {
                        title: errorTitle[0],
                        message: rawValues[error].replace(errorTitle[0], "")
                    }
                    result.push(object)
                }
                this.setCurrentCompileErrorsOfProject(projectName, result)
            })
                .finally(() => (this.loading = false))
        },

        /**
         * Extracting the error messages from process output from the backend.
         * @param text - the logs containing the error messages.
         */
        extractErrors(text: string): string[] {
            const pattern = /error\[\s*([\s\S]*?)(?=error\[|error: could not|$)/g;
            const matches = text.match(pattern);
            return matches || [];
        },

        /**
         * Calling the API method to get the logs of the currently selected project.
         */
        async getLogs() {
            if (this.selectedProject !== null) {
                const {$api} = useNuxtApp();
                let processId = this.runningProcessesMap.get(this.selectedProject.name)
                if (processId != undefined && processId != -1) {
                    const processIdentifier: ProcessIdentifier = {
                        process_id: processId
                    }
                    this.loading = true
                    $api.processes.getProcessLogs(processIdentifier).then(response => {
                        console.log("Getting Logs of process  with the id", processIdentifier.process_id)
                        response.forEach((item) => {
                            this.writeLogEntry(item)
                        })
                    }).catch((error) => {
                        this.writeLogEntry(error)
                        console.log("Error getting logs:" + error)
                    })
                        .finally(() => (this.loading = false))
                }
            }
        },

        /**
         * Called by the user via UI to set the selected project as state in the pinia store.
         */
        selectProject(project: FlowProject, setLastCompile?: boolean) {
            this.selectedProject = project;
            this.projectClickedInList = true
            if (setLastCompile) {
                this.getLastCompileOfProject()
            }
        },

        /**
         * Calling the API method to create the flow project.
         * @param project - the flow project to create.
         */
        async createProject(project: FlowProject) {
            const { $api } = useNuxtApp();

            try {
                const flowProject = await $api.projects.createProject(project);
                console.log("Project created!");
                this.projects.push(flowProject);
            } catch (error) {
                console.error("Error creating project:", error);
                throw error;
            }
        },

        /**
         * Used to set the build type which is selected via Dropdown Menu on the compile, run and metrics page.
         * @param buildType
         */
        selectBuildType(buildType: BuildType) {
            this.selectedBuildType = buildType;
        },

        writeLogEntry(logEntryToAdd: string) {
            if (this.selectedProject !== null) {
                let logEntries = this.logEntriesMap.get(this.selectedProject.name)
                this.updateLogEntryMap(logEntries, logEntryToAdd)
            }
        },

        /**
         * Updating the map of log entries of all projects.
         * @param entries - an array of log entries.
         * @param entryToAdd - the log entry to append at the array of log entries.
         */
        updateLogEntryMap(entries: string[] | undefined, entryToAdd: string) {
            if (this.selectedProject !== null) {
                if (entries != undefined) {
                    const updatedEntries = this.addLogEntry(entryToAdd, entries)
                    this.logEntriesMap.set(this.selectedProject.name, updatedEntries)
                } else {
                    const entries: string[] = []
                    const updatedEntries = this.addLogEntry(entryToAdd, entries)
                    this.logEntriesMap.set(this.selectedProject.name, updatedEntries)
                }
            }
        },

        addLogEntry(entry: string, entryList: string[]): string[] {
            const timestamp = new Date().toLocaleString();
            const logEntry = `[${timestamp}] ${entry}`;
            entryList.unshift(logEntry)
            return entryList
        },

        /**
         * Get the current process id of the selected project.
         */
        getCurrentProcessId() {
            if (this.selectedProject != null) {
                let processId = this.runningProcessesMap.get(this.selectedProject.name)
                if (processId != undefined) {
                    return processId
                }
            }
            return undefined

        },

        /**
         * Get the current logs of the selected project.
         */
        getCurrentLogEntries() {
            if (this.selectedProject === null) {
                return []
            }
            return this.logEntriesMap.get(this.selectedProject.name)
        },

        getCurrentCompileErrorsOfProject(): CompileError[] | undefined {
            if (this.selectedProject !== null) {
                return this.compileErrorMap.get(this.selectedProject.name)
            }
            return undefined
        },

        /**
         * Check if a compile error of the currently selected project exists.
         */
        compileErrorForSelectedProjectExist(): boolean {
            if (this.selectedProject !== null) {
                const compileErrors = this.compileErrorMap.get(this.selectedProject.name)
                return compileErrors !== undefined;
            }
            return false;
        },

        /**
         * Set the current compile errors of a project in a map to save it for later use.
         * @param projectName
         * @param compileErrors
         */
        setCurrentCompileErrorsOfProject(projectName: string, compileErrors: CompileError[] | undefined) {
            this.compileErrorMap.set(projectName, compileErrors)
        },

        /**
         * Calling the api backend method to get the last compile timestamp of a project.
         */
        async getLastCompileOfProject() {
            const {$api} = useNuxtApp();
            if (this.selectedProject !== null) {
                const projectIdentifier: ProjectIdentifier = {
                    project_name: this.selectedProject.name
                }
                $api.projects.lastCompileOfProject(projectIdentifier, this.selectedBuildType).then(response => {
                    console.log("Getting last compile of project", response)
                    this.compileTimestampMap.set(this.selectedProject.name, response.modified_time)
                }).catch((error) => {
                    this.compileTimestampMap.set(this.selectedProject.name, undefined)
                    console.log("Error getting last compile of project:" + error)
                })
                    .finally(() => (this.loading = false))
            }
        },

        /**
         * Get the last compile timestamp for the selected project from pinia store.
         */
        getLastCompileFromMap(): string | undefined {
            if (this.selectedProject !== null) {
                return this.compileTimestampMap.get(this.selectedProject.name)
            }
            return undefined
        },

        /**
         * Formatting timestamp (last compile of a project).
         */
        getCurrentTimestamp(): string {
            const currentDate = new Date();
            const formatter = new Intl.DateTimeFormat('de-DE', {
                year: 'numeric',
                month: '2-digit',
                day: '2-digit',
                hour: '2-digit',
                minute: '2-digit',
                second: '2-digit',
                hour12: false, // 24-hour format
            });

            return formatter.format(currentDate);
        },

        /**
         * Get all running flow projects as an array of strings with the names of the projects.
         */
        getRunningFlowProjects(): string[] {
            const stringsWithDefinedNumbers: string[] = [];

            this.runningProcessesMap.forEach((value, key) => {
                if (value !== undefined) {
                    stringsWithDefinedNumbers.push(key);
                }
            });
            return stringsWithDefinedNumbers;
        },

        getBuildTypeArray(): string[] {
            return Object.values(BuildType) as string[];
        },

        setCurrentErrorMessage(errorMessage: string) {
            this.errorMessage = errorMessage
            this.setDialog(true)
        },

        setDialog(active: boolean) {
            this.showDialog = active
        },



    }
})
