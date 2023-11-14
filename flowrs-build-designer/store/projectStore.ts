import {defineStore} from 'pinia'
import {FlowProject, ProjectIdentifier} from "~/repository/modules/projects";

export const useProjectsStore = defineStore({
    id: 'projects',
    state: () => {
        return ({
            projects: [] as FlowProject[],
            selectedProject: null as FlowProject | null,
            loading: false,
            activeFilter: "",
            logEntries: []
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

        async compileProjectRequest(buildType: String) {
            const {$api} = useNuxtApp();
            const projectIdentifier: ProjectIdentifier = {
                project_name: this.selectedProject!.name
            }
            this.loading = true
            $api.projects.compileProject(projectIdentifier, buildType).then(response => {
                console.log("Flow Project is compiling!")
                this.addLogEntry(response)
            }).catch((error) => {
                console.log("Error compiling projects:" + error)
            })
                .finally(() => (this.loading = false))
        },

        selectProject(project: FlowProject) {
            this.selectedProject = project;
            this.activeFilter = 'noFilter'
        },
        setActiveFilter(value: string) {
            this.activeFilter = value
        },

        addLogEntry(entry) {
          const timestamp = new Date().toLocaleString();
          const logEntry = `[${timestamp}] ${entry}`;
          this.logEntries.push(logEntry)
        }
    }
})
