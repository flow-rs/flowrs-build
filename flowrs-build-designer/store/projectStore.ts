import {defineStore} from 'pinia'
import {type FlowProject, type ProjectIdentifier} from "~/repository/modules/projects";
import type {ProcessIdentifier} from "~/repository/modules/processes";


export enum BuildType {
    Wasm = "wasm",
    Cargo = "cargo",
}

export const useProjectsStore = defineStore({
    id: 'projects',
    state: () => {
        return ({
            projects: [] as FlowProject[],
            selectedProject: null as FlowProject | null,
            selectedBuildType: BuildType.Cargo,
            loading: false,
            activeFilter: "",
            logEntriesMap: new Map() as Map<string, string[]>,
            runningProcessesMap: new Map() as Map<string, number | undefined>,
            projectClickedInList: false,
            errorMessage: "",
            showDialog: false,
            compileError: false,
            compileErrorText: ''
        });
    },
    actions: {
        async getAll() {
            const {$api} = useNuxtApp();
            $api.projects.getProjects().then(listOfFlowProjects => {
                this.projects = listOfFlowProjects;
                this.selectedProject = listOfFlowProjects[0]
            }).catch((error) =>
            {
                const errorString = "Error fetching projects " + error
                this.setCurrentErrorMessage(errorString)
                console.log("Error fetching projects!")
            })

                .finally(() => (this.loading = false));
        },
        async deleteProject() {
            const {$api} = useNuxtApp();
            const projectIdentifier: ProjectIdentifier = {
                project_name: this.selectedProject!.name
            }
            $api.projects.deleteProject(projectIdentifier).then(projects => {
                console.log("Flow Project was deleted")
                // remove item from list in store to update the ui list
                this.projects = this.projects.filter((object) => {
                    return object.name != this.selectedProject!.name
                })
                this.selectedProject = null
                this.projectClickedInList = false
            }).catch((error) => {
                this.setCurrentErrorMessage("Error deleting projects:" + error)
                console.log("Error deleting projects:" + error)
            })
                .finally(() => (this.loading = false))
        },

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

        async stopProcessRequest() {
            const {$api} = useNuxtApp();

            let processId = this.runningProcessesMap.get(this.selectedProject.name)
            if (processId != undefined && processId != -1) {
                const processIdentifier: ProcessIdentifier = {
                    process_id: processId
                }
                this.loading = true
                $api.processes.stopProcess(processIdentifier).then(response => {
                    console.log("Flow Project is stopped!")
                    this.writeLogEntry("Ausführung vom Flow-Projekt gestoppt.")
                    this.runningProcessesMap.set(this.selectedProject.name, undefined)
                }).catch((error) => {
                    this.writeLogEntry(error)
                    console.log("Error stopping project:" + error)
                })
                    .finally(() => (this.loading = false))
            }
        },

        async compileProjectRequest(projectName: string, buildType: string) {
            const {$api} = useNuxtApp();
            const projectIdentifier: ProjectIdentifier = {
                project_name: projectName
            }
            this.loading = true
            this.compileError = false;
            $api.projects.compileProject(projectIdentifier, buildType).then(response => {
                console.log("Flow Project is compiled!")
                const response_txt = `${response}`
                this.writeLogEntry(response_txt)
            }).catch((error) => {
                console.log("error compiling project")
                this.compileError = true;
                let converted = error.data as string
                converted = converted.replace(/(?:\r\n|\r|\n)/g, '<br>');
                console.log(converted)
                const errorMessages = error.data
                    .split('\\n')
                    .filter((line: string | string[]) => line.includes('error['));
                // console.log(errorMessages)
                this.compileErrorText = errorMessages
                this.writeLogEntry(converted)
            })
                .finally(() => (this.loading = false))
        },

        async getLogs() {
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
        },

        selectProject(project: FlowProject) {
            this.selectedProject = project;
            this.activeFilter = 'noFilter';
            this.projectClickedInList = true
        },

        selectBuildType(buildType: string) {
            this.selectedBuildType = buildType;
        },

        setActiveFilter(value: string) {
            this.activeFilter = value
        },

        writeLogEntry(logEntryToAdd: string) {
            let logEntries = this.logEntriesMap.get(this.selectedProject.name)
            this.updateLogEntryMap(logEntries, logEntryToAdd)
        },

        updateLogEntryMap(entries: string[] | undefined, entryToAdd: string) {
            if (entries != undefined) {
                const updatedEntries = this.addLogEntry(entryToAdd, entries)
                this.logEntriesMap.set(this.selectedProject.name, updatedEntries)
            } else {
                const entries: string[] = []
                const updatedEntries = this.addLogEntry(entryToAdd, entries)
                this.logEntriesMap.set(this.selectedProject.name, updatedEntries)
            }
        },

        addLogEntry(entry: string, entryList: string[]): string[] {
            const timestamp = new Date().toLocaleString();
            const logEntry = `[${timestamp}] ${entry}`;
            entryList.push(logEntry)
            return entryList
        },

        getCurrentProcessId() {
            let processId = this.runningProcessesMap.get(this.selectedProject.name)
            if (processId != undefined) {
                return processId
            }
        },

        getCurrentLogEntries() {
            if (this.selectedProject === null) {
                return []
            }
            return this.logEntriesMap.get(this.selectedProject.name)
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
        }

    }
})
