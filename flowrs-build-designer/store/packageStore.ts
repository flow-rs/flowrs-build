import {defineStore} from 'pinia'

export const usePackagesStore = defineStore({
    id: 'packages',
    state: () => ({
        selectedPackage:null,
        packages: [],
        loading: false
    }),
    actions: {
        async getAll() {
            const {$api} = useNuxtApp();
            const response = $api.packages.getFlowrsPackages().then(packages => {
                this.packages = packages;
            }).catch((error) => console.log("Error fetching packages!"))
                .finally(() => (this.loading = false))

        },
        selectPackage(packageE) {
            this.selectedPackage = packageE;
        }
    }
})
