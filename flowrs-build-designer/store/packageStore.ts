import {defineStore} from 'pinia'
import {type Crate, type TypeDefinition} from "~/repository/modules/packages";

export const usePackagesStore = defineStore({
    id: 'packages',
    state: () => ({
        selectedPackage: null,
        packagesMap: new Map<string, TypeDefinition>(),
        packages: [] as Crate[],
        selectedMap: new Map<string, TypeDefinition>(),
        loading: false
    }),
    actions: {
        async getAll() {
            const {$api} = useNuxtApp();
            const response = await $api.packages.getFlowrsTypeDefinitionsMap()
                .catch((error) => {
                        console.log("Error fetching package map!");
                        return new Map<string, TypeDefinition>();
                    }
                );
            this.packagesMap = response
            const packages = await $api.packages.getFlowrsPackages().catch((error) => {
                console.log("Error fetching packages!");
                return [];
            });
            this.packages = packages;

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
