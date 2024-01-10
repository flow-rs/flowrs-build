import {defineStore} from 'pinia'
import {type Package, type Type} from "~/repository/modules/packages";

export const usePackagesStore = defineStore({
    id: 'packages',
    state: () => ({
        selectedPackage: null,
        packagesMap: new Map<string, Type>(),
        packages: [] as Package[],
        selectedMap: new Map<string, Type>(),
        loading: false,
        currentActive:["flowrs","primitives"] as string[]
    }),
    actions: {
        async getAll() {
            const {$api} = useNuxtApp();
            const response = await $api.packages.getFlowrsTypeDefinitionsMap(this.currentActive)
                .catch((error) => {
                        console.log("Error fetching package map!");
                        return new Map<string, Type>();
                    }
                );
            this.packagesMap = response
            const packages = await $api.packages.getFlowrsPackages().catch((error) => {
                console.log("Error fetching packages!");
                return [];
            });
            this.packages = packages;
            console.log(packages)

        },
        async getByName(name: string) {
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
