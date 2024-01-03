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
    inputs: Record<string, TypeWrapper>;
    outputs: Record<string, TypeWrapper>;
    type_parameters?: string[];
    constructors: ConstructorDefinition;
}

export type TypeDescription = {
    Generic?: {
        name: string;
        type_parameters?: TypeWrapper[];
    };
    Type?: {
        name: string;
        type_parameters?: TypeWrapper[];
    }
};
export type TypeWrapper = {
    type: TypeDescription
}

export type ModuleDefinition = {
    types: Record<string, TypeDefinition>;
    modules: Record<string, ModuleDefinition>;
}

//TODO @mafried change to one to one relation ?
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

    async getFlowrsPackages(): Promise<Crate[]> {
        return await this.call<Crate[]>('GET', `${this.RESOURCE}`)
    }

    // returns a parsed map of all packages, where the full type name is mapped to its typeDefinition
       // returns a parsed map of all packages, where the full type name is mapped to its typeDefinition
       async getFlowrsTypeDefinitionsMap(currentActive: string[]): Promise<Map<string, TypeDefinition>> {
        const crates = await this.getFlowrsPackages();

        console.log('mapped packages to js-objects', crates)
        const packageMap: Map<string, TypeDefinition> = new Map<string, TypeDefinition>();


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

        return this.filterInActive(packageMap,currentActive)
    }

    private filterInActive(map: Map<string, TypeDefinition>,currentActive: string[]) {
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

    async getFlowrsTypeDefinitionsMapByName(name: string): Promise<Map<string, TypeDefinition>> {
        const obj = await this.getFlowrsPackageByName(name);
        const crates = [obj]
        console.log('mapped packages to js-objects by Name', crates)
        const packageMap: Map<string, TypeDefinition> = new Map<string, TypeDefinition>();


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

    async getFlowrsPackageByName(packageName: string): Promise<Crate> {
        return await this.call<Crate>('GET', `${this.RESOURCE}${packageName}`)
    }

    // recursively constructs the name and adds all type definition underneath that name to the map
    private populatePackageMap(moduleDefinition: Record<string, ModuleDefinition>, typeDefinition: Record<string, TypeDefinition>, packageMap: Map<string, TypeDefinition>, packagePath: string) {
        // add all typeDefinitions under this name
        this.addTypeDefinitions(typeDefinition, packageMap, packagePath);

        // go to the next module
        for (const moduleName in moduleDefinition) {
            const nextPackagePath = packagePath + "::" + moduleName;
            const nextModuleDefinition = moduleDefinition[moduleName];
            this.populatePackageMap(nextModuleDefinition.modules, nextModuleDefinition.types, packageMap, nextPackagePath);
        }
    }

    private addTypeDefinitions(typeDefinitions: Record<string, TypeDefinition>, packageMap: Map<string, TypeDefinition>, packagePath: string) {
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
