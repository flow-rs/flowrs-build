// import { $fetch, FetchOptions } from 'ofetch';

// locals
import PackagesModule from "~/repository/modules/packages";
// import Packages from "~/repository/modules/packages";
import axios from "axios";
import {$fetch, FetchOptions} from "ofetch";

interface IApiInstance {
    packages: PackagesModule;
}

export default defineNuxtPlugin((nuxtApp) => {
    const config = useRuntimeConfig();

    const fetchOptions: FetchOptions = {
        // baseURL: config.public.apiBaseUrl
        baseURL: "http://127.0.0.1:3000"
    };

    // Create a new instance of $fecther with custom option
    const apiFecther = $fetch.create(fetchOptions);

    // An object containing all repositories we need to expose
    const modules: IApiInstance = {
        packages: new PackagesModule(apiFecther),
    };

    return {
        provide: {
            api: modules
        }
    };
});
