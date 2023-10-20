// Define the types for the API response
import FetchFactory from "~/repository/factory";
import {FetchOptions} from "ofetch";
import {integer} from "vscode-languageserver-types";

type TimerConfigNode = {
    value : {
        duration: {
        nanos: number;
        secs: number;
    },
    };
};

type TimerTokenNode = {
    value : number
};

type Node<T> = {
    node_type: string;
    type_parameters: T;
    constructor: string;
};

type Connection = {
    from_node: string;
    to_node: string;
    to_input: string;
    from_output: string;
};

type FlowData = {
    timer_config_node: TimerConfigNode | null
    timer_token_node: TimerTokenNode | null
};

export type FlowProject = {
    name: string;
    version: string;
    packages: Array<{
        name: string;
        version: string;
        path: string;
    }>;
    flow: {
        nodes: { [key: string]: Node<any> };
        connections: Connection[];
        data: FlowData;
    };
};

class ProjectsModule extends FetchFactory {
    private RESOURCE = '/projects/';
    private BUILD_PATH: string = '/build/';
    // "/build/:project_name"

    async getProjects() : Promise<FlowProject[]> {
        return await this.call<FlowProject[]>('GET', `${this.RESOURCE}`)
    }

    async createProject(project : FlowProject) : Promise<FlowProject> {
        const fetchOptions: FetchOptions<'json'> = {
            headers: {
                'Content-Type': 'application/json',
            }
        }
        return await this.call<FlowProject>('POST', `${this.RESOURCE}`, project, fetchOptions)
    }

    async buildProject(projectName : string) : Promise<FlowProject> {
        return await this.call<FlowProject>('GET', `${this.BUILD_PATH}${projectName}`)
    }




}

export default ProjectsModule;
