
import FetchFactory from '../factory';

export type Crate = {
    name: string;
    version: string;
    crates: Record<string, CrateType>;
}

export type CrateType = {
    types: Record<string, TypeDefinition>;
    modules: Record<string, ModuleDefinition>;
}

export type TypeDefinition = {
    inputs: Record<string, Record<string, TypeDescription>>;
    outputs: Record<string, Record<string, TypeDescription>>;
    type_parameters: string[] | null;
    constructors: Record<string, string>;
}

export type TypeDescription = {
    name: string;
    type_parameters: TypeDescription[];
}

export type ModuleDefinition = {
    types: Record<string, TypeDefinition>;
    modules: Record<string, ModuleDefinition>;
}

// TODO refactor to a correct place
export async function createAllPackagesToTypeDefintionMap() : Promise<Map<string, TypeDefinition>> {
    const {$api} = useNuxtApp();
    const packages = await $api.packages.getFlowrsPackages();
    const packageMap: Map<string, TypeDefinition> = new Map<string, TypeDefinition>();

    for (const packagesKey in packages) {
        let crate = packages[packagesKey];
        if (!crate) {
            continue
        }
        let crateTypes = crate.crates;
        for (let crateName in crateTypes) {
            let crateType = crateTypes[crateName];
            populatePackageMap(crateType.modules, crateType.types, packageMap, crateName);
        }
    }

    return packageMap
}

function populatePackageMap(moduleDefinition: Record<string, ModuleDefinition>, typeDefinition: Record<string, TypeDefinition>, packageMap: Map<string, TypeDefinition>, packagePath: string) {
    addTypeDefinitions(typeDefinition, packageMap, packagePath);
    for (const moduleDefinitionKey in moduleDefinition) {
        const newPackagePath = packagePath + "::" + moduleDefinitionKey;
        const newModuleDefinition = moduleDefinition[moduleDefinitionKey];
        populatePackageMap(newModuleDefinition.modules, newModuleDefinition.types, packageMap, newPackagePath);
    }
}

function addTypeDefinitions(typeDefinition: Record<string, TypeDefinition>, packageMap: Map<string, TypeDefinition>, packagePath: string) {
    for (const typeDefinitionKey in typeDefinition) {
        packageMap.set(packagePath+ "::"+ typeDefinitionKey, typeDefinition[typeDefinitionKey])
    }
}

class PackagesModule extends FetchFactory {
    private RESOURCE = '/packages/';

    async getFlowrsPackages() : Promise<Crate[]> {
        return await this.call<Crate[]>('GET', `${this.RESOURCE}`)
    }

    async getFlowrsPackageByName(packageName : string) : Promise<Crate> {
        return await this.call<Crate>('GET', `${this.RESOURCE}${packageName}`)
    }
}

export default PackagesModule;
