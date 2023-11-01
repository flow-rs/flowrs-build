import {defineStore} from 'pinia'
import {FlowProject, ProjectIdentifier} from "~/repository/modules/projects";

export const useProjectsStore = defineStore({
    id: 'projects',
    state: () => {
        return ({
            projects: [] as FlowProject[],
            selectedProject: null as FlowProject | null,
            loading: false,
            activeFilter: ""
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
            }).catch((error) => {
                console.log(error)
                console.log("Error deleting projects!")
            })
                .finally(() => (this.loading = false))

        },
        selectProject(project: FlowProject) {
            this.selectedProject = project;
            this.activeFilter = 'noFilter'
        },
        setActiveFilter(value: string) {
            this.activeFilter = value
        }
    }
})
