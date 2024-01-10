import FetchFactory from '../factory';

export type Package = {
    name: string;
    version: string;
    crates: Record<string, Crate>;
}

export type Crate = {
    types: Record<string, Type>;
    modules: Record<string, Module>;
}

export type Type = {
    inputs: Record<string, TypeDescriptionWrapper>;
    outputs: Record<string, TypeDescriptionWrapper>;
    type_parameters?: string[];
    constructors: ConstructorDefinition;
}

export type TypeDescription = {
    Generic?: {
        name: string;
        type_parameters?: TypeDescriptionWrapper[];
    };
    Type?: {
        name: string;
        type_parameters?: TypeDescriptionWrapper[];
    }
};
export type TypeDescriptionWrapper = {
    type: TypeDescription
}

export type Module = {
    types: Record<string, Type>;
    modules: Record<string, Module>;
}

export type ConstructorDefinition = Record<string, string | Record<string, ConstructorDescription>>

export type ConstructorDescription = {
    function_name: string,
    arguments?: ArgumentDefinition[],
}

export type ArgumentDefinition = {
    type: TypeDescription,
    name: string,
    passing: string,
    construction: {
        Constructor?: string,
        ExistingObject?: string,
    }
}

class PackagesModule extends FetchFactory {
    private RESOURCE = '/packages/';

    /**
     * Asynchronously retrieves Flowrs packages.
     * @returns A Promise resolving to an array of Flowrs packages.
     */
    async getFlowrsPackages(): Promise<Package[]> {
        return await this.call<Package[]>('GET', `${this.RESOURCE}`)
    }

    /**
     * Asynchronously retrieves Flowrs type definitions and maps them to their full name.
     * @param currentActive - An array of currently active packages.
     * @returns A Promise resolving to a Map containing Flowrs type definitions as values, with the full type name as keys.
     */
    async getFlowrsTypeDefinitionsMap(currentActive: string[]): Promise<Map<string, Type>> {
        const packages = await this.getFlowrsPackages();

        console.log('mapped packages to js-objects', packages)
        const packageMap: Map<string, Type> = new Map<string, Type>();


        for (const currentPackage of packages) {
            if (!currentPackage) {
                continue
            }

            let crateEntries = currentPackage.crates;

            for (let crateName in crateEntries) {
                let crate = crateEntries[crateName];
                this.populatePackageMap(crate.modules, crate.types, packageMap, crateName);
            }
        }

        return this.filterInactive(packageMap, currentActive)
    }

    /**
     * Filters inactive types from the provided map based on the currently active packages.
     * @param map - The map of type definitions to filter.
     * @param currentActive - An array of currently active packages.
     * @returns The filtered map of type definitions.
     */
    private filterInactive(map: Map<string, Type>, currentActive: string[]) {
        const keysToRemove: string[] = [];
        console.log(toRaw(currentActive));
        map.forEach((value, key) => {
            let keyToSearch = key.substring(0, key.indexOf("::")).replace("_", "-");
            if (!toRaw(currentActive).includes(keyToSearch) && keyToSearch) {
                console.log(key);
                keysToRemove.push(key);
            }
        });
        keysToRemove.forEach((key) => {
            map.delete(key);
        });
        console.log(map);
        return map;
    }

    /**
     * Asynchronously retrieves Flowrs type definitions for a specific package name.
     * @param name - The name of the package.
     * @returns A Promise resolving to a Map containing Flowrs type definitions as values, with the full type name as keys.
     */
    async getFlowrsTypeDefinitionsMapByName(name: string): Promise<Map<string, Type>> {
        const obj = await this.getFlowrsPackageByName(name);
        const crates = [obj]
        console.log('mapped packages to js-objects by Name', crates)
        const packageMap: Map<string, Type> = new Map<string, Type>();


        for (const crate of crates) {
            if (!crate) {
                continue
            }

            let crateTypes = crate.crates;

            for (let crateName in crateTypes) {
                let crateType = crateTypes[crateName];
                this.populatePackageMap(crateType.modules, crateType.types, packageMap, crateName);
            }
        }

        return packageMap
    }

    /**
     * Asynchronously retrieves Flowrs package by name.
     * @param packageName - The name of the package.
     * @returns A Promise resolving to a Flowrs package.
     */
    async getFlowrsPackageByName(packageName: string): Promise<Package> {
        return await this.call<Package>('GET', `${this.RESOURCE}${packageName}`)
    }

    /**
     * Recursively populates the package map with type definitions from modules and types.
     * @param moduleDefinition - The definition of modules.
     * @param typeDefinition - The definition of types.
     * @param packageMap - The map to populate with type definitions.
     * @param packagePath - The current package path.
     */
    private populatePackageMap(moduleDefinition: Record<string, Module>, typeDefinition: Record<string, Type>, packageMap: Map<string, Type>, packagePath: string) {
        // add all typeDefinitions under this name
        this.addTypeDefinitions(typeDefinition, packageMap, packagePath);

        // go to the next module
        for (const moduleName in moduleDefinition) {
            const nextPackagePath = packagePath + "::" + moduleName;
            const nextModuleDefinition = moduleDefinition[moduleName];
            this.populatePackageMap(nextModuleDefinition.modules, nextModuleDefinition.types, packageMap, nextPackagePath);
        }
    }

    /**
     * Adds type definitions to the package map.
     * @param typeDefinitions - The type definitions to add to the map.
     * @param packageMap - The map to add the type definitions to.
     * @param packagePath - The current package path.
     */
    private addTypeDefinitions(typeDefinitions: Record<string, Type>, packageMap: Map<string, Type>, packagePath: string) {
        for (const typeDefinitionName in typeDefinitions) {
            let prefix = packagePath + "::";
            if (prefix == "primitives::") {
                prefix = ""
            }
            packageMap.set(prefix + typeDefinitionName, typeDefinitions[typeDefinitionName]);
        }
    }
}

export default PackagesModule;
