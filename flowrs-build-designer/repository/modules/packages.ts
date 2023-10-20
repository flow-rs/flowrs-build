
import { FetchOptions } from 'ofetch';
import { AsyncDataOptions } from '#app';


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


class PackagesModule extends FetchFactory<Crate[]> {
    private RESOURCE = '/packages/';

    /**
     * Return the packages as array
     * @param asyncDataOptions options for `useAsyncData`
     * @returns
     */
    async getPackages(
        asyncDataOptions?: AsyncDataOptions<Crate[]>
    ) {

        return useAsyncData(
            () => {
                const fetchOptions: FetchOptions<'json'> = {
                    headers: {
                        'Accept-Language': 'en-US'
                    }
                };
                return this.call(
                    'GET',
                    `${this.RESOURCE}`,
                    undefined, // body
                    fetchOptions
                )
            },
            asyncDataOptions
        )
    }
}

export default PackagesModule;
