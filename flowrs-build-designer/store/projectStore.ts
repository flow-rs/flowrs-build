import {defineStore} from 'pinia'
import {type FlowProject, type ProjectIdentifier} from "~/repository/modules/projects";
import type {ProcessIdentifier} from "~/repository/modules/processes";

export const useProjectsStore = defineStore({
    id: 'projects',
    state: () => {
        return ({
            projects: [] as FlowProject[],
            selectedProject: null as FlowProject | null,
            selectedBuildType: 'cargo',
            loading: false,
            activeFilter: "",
            logEntries: [] as string[],
            runningProcessesMap: new Map() as Map<string, number | undefined>
        });
    },
    actions: {
        async getAll() {
            const {$api} = useNuxtApp();
            const response = $api.projects.getProjects().then(listOfFlowProjects => {
                this.projects = listOfFlowProjects;
            }).catch((error) => console.log("Error fetching projects!"))
                .finally(() => (this.loading = false))

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
            }).catch((error) => {
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
                this.addLogEntry("Flow Projekt Ausführung wurde gestartet.")
                this.runningProcessesMap.set(projectIdentifier.project_name, response.process_id)
            }).catch((error) => {
                console.log("Error compiling projects:" + error)
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
                    this.addLogEntry("Ausführung vom Flow-Projekt gestoppt.")
                    this.runningProcessesMap.set(this.selectedProject.name, undefined)
                }).catch((error) => {
                    console.log("Error compiling projects:" + error)
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
            $api.projects.compileProject(projectIdentifier, buildType).then(response => {
                console.log("Flow Project is compiled!")
                this.addLogEntry(response)
            }).catch((error) => {
                console.log("Error compiling projects:" + error)
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
                    console.log(response)
                    response.forEach((item) => this.addLogEntry(item))
                }).catch((error) => {
                    console.log("Error compiling projects:" + error)
                })
                    .finally(() => (this.loading = false))
            }
        },

        selectProject(project: FlowProject) {
            this.selectedProject = project;
            this.activeFilter = 'noFilter'
        },

        selectBuildType(buildType: string) {
            this.selectedBuildType = buildType;
        },

        setActiveFilter(value: string) {
            this.activeFilter = value
        },

        addLogEntry(entry: string) {
          const timestamp = new Date().toLocaleString();
          const logEntry = `[${timestamp}] ${entry}`;
          this.logEntries.push(logEntry)
        },

        getCurrentProcessId() {
            let processId = this.runningProcessesMap.get(this.selectedProject.name)
            if (processId != undefined) {
                return processId
            }
        }
    }
})
