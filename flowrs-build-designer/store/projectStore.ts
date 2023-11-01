import {defineStore} from 'pinia'
import {ProjectIdentifier} from "~/repository/modules/projects";

export const useProjectsStore = defineStore({
    id: 'projects',
    state: () => ({
        projects: [],
        selectedProject: null,
        displayedJSON: null,
        loading: false,
        activeFilter: ""
    }),
    actions: {
        async getAll() {
            const {$api} = useNuxtApp();
            const response = $api.projects.getProjects().then(projects => {
                this.projects = projects;
            }).catch((error) => console.log("Error fetching projects!"))
                .finally(() => (this.loading = false))

        },
        async deleteProject(project) {
            const {$api} = useNuxtApp();
            const projectIdentifier: ProjectIdentifier = {
                project_name: project.name
            }
            $api.projects.deleteProject(projectIdentifier).then(projects => {
                console.log("Flow Project was deleted")
                // remove item from list in store to update the ui list
                this.projects = this.projects.filter((object) => {
                    return object.name != project.name
                })
            }).catch((error) => {
                console.log(error)
                console.log("Error deleting projects!")
            })
                .finally(() => (this.loading = false))

        },
        selectProject(project) {
            this.selectedProject = project;
            this.displayedJSON = project;
            this.activeFilter = 'noFilter'
        },
        setDisplayedJSON(object) {
            this.displayedJSON = object
        },
        setActiveFilter(value) {
            this.activeFilter = value
        }
    }
})
