
import PackagesModule from "~/repository/modules/packages";
import {$fetch, type FetchOptions} from "ofetch";
import ProjectsModule from "~/repository/modules/projects";
import ProcessesModule from "~/repository/modules/processes";

interface IApiInstance {
    packages: PackagesModule;
    projects: ProjectsModule;
    processes: ProcessesModule;
}

export default defineNuxtPlugin((nuxtApp) => {
    useRuntimeConfig();
    const fetchOptions: FetchOptions = {
        baseURL: "http://127.0.0.1:3000/api"
    };

    // Create a new instance of $fetcher with custom option
    const apiFetcher = $fetch.create(fetchOptions);

    // An object containing all repositories we need to expose
    const modules: IApiInstance = {
        packages: new PackagesModule(apiFetcher),
        projects: new ProjectsModule(apiFetcher),
        processes: new ProcessesModule(apiFetcher)
    };

    return {
        provide: {
            api: modules
        }
    };
});
