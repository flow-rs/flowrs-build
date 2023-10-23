// import { $fetch, FetchOptions } from 'ofetch';

// locals
import PackagesModule from "~/repository/modules/packages";
// import Packages from "~/repository/modules/packages";
import axios from "axios";
import {$fetch, FetchOptions} from "ofetch";
import ProjectsModule from "~/repository/modules/projects";

interface IApiInstance {
    packages: PackagesModule;
    projects: ProjectsModule
}

export default defineNuxtPlugin((nuxtApp) => {
    const config = useRuntimeConfig();

    const fetchOptions: FetchOptions = {
        // baseURL: config.public.apiBaseUrl
        baseURL: "http://127.0.0.1:3000/api"
    };

    // Create a new instance of $fetcher with custom option
    const apiFetcher = $fetch.create(fetchOptions);

    // An object containing all repositories we need to expose
    const modules: IApiInstance = {
        packages: new PackagesModule(apiFetcher),
        projects: new ProjectsModule(apiFetcher),
    };

    return {
        provide: {
            api: modules
        }
    };
});
