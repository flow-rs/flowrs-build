import { defineStore } from 'pinia'
export const useProjectsStore = defineStore({
    id: 'projects',
    state: () => ({
        projects: {},
        loading: false
    }),
    actions: {
        async getAll() {
            const { $api } = useNuxtApp();
            // this.projects = { loading: true };
            const response = $api.projects.getProjects().then(projects => {
                this.projects = projects;
            }).catch((error) => console.log("Error fetching projects!"))
                .finally(() => (this.loading = false))

        },
        async deleteProject(project) {
            const { $api } = useNuxtApp();

            const response = $api.projects.deleteProject().then(projects => {
                console.log("Flow Project was deleted")
            }).catch((error) => console.log("Error fetching projects!"))
                .finally(() => (this.loading = false))

        }
    }
})
