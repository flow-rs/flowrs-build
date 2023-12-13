import {defineStore} from 'pinia'

export const usePackagesStore = defineStore({
    id: 'packages',
    state: () => ({
        selectedPackage:null,
        packagesMap: null,
        packages: [],
        selectedMap:[],
        currentActive: [],
        NodeEditor: null,
        ContextMenu: null,
        loading: false
    }),
    actions: {
        async getAll() {
            const {$api} = useNuxtApp();
            const response = $api.packages.getFlowrsTypeDefinitionsMap().then(packages => {
                this.packagesMap = packages;
            }).then( ()=>{
                return $api.packages.getFlowrsPackages();
            }).then( packages =>{
                console.log(packages)
                this.packages=packages;
            })
            .catch((error) => console.log("Error fetching packages!"))
                .finally(() => (this.loading = false))

        },
        async getByName(name : string){
            const {$api} = useNuxtApp();
            await $api.packages.getFlowrsTypeDefinitionsMapByName(name).then(packages => {
                console.log(packages)
                this.selectedMap = packages;
            }).catch((error) => console.log("Error fetching packages!"))
                .finally(() => (this.loading = false))
        },
        selectPackage(packageE) {
            this.selectedPackage = packageE;
        }
    }
})
