
import FetchFactory from '../factory';

type Crate = {
    name: string;
    version: string;
    crates: Record<string, CrateType>;
}

type CrateType = {
    types: Record<string, TypeDefinition>;
    modules: Record<string, ModuleDefinition>;
}

type TypeDefinition = {
    inputs: string[] | null;
    outputs: string[] | null;
    type_parameters: string[] | null;
    constructors: Record<string, string>;
}

type ModuleDefinition = {
    types: Record<string, TypeDefinition>;
    modules: Record<string, ModuleDefinition>;
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
