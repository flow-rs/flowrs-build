// Define the types for the API response
import FetchFactory from "~/repository/factory";
import {FetchOptions} from "ofetch";

export type TimerConfigNode = {
    value : {
        duration: {
        nanos: number;
        secs: number;
    },
    };
};

export type ProjectIdentifier = {
    project_name : string;
}

export type TimerTokenNode = {
    value : number
};

export type FlowNode<T> = {
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
        nodes: { [key: string]: FlowNode<any> };
        connections: Connection[];
        data: FlowData;
    };
};

class ProjectsModule extends FetchFactory {
    private RESOURCE: string = '/projects/';
    private BUILD_PATH: string = '/build/';
    //TODO: Placeholder string
    private COMPILE_PATH: string = '/projects/{}/compile';
    private RUN_JOBS: string = '/run_jobs/';

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

    async compileProject(project : ProjectIdentifier) : Promise<FlowProject> {
        const fetchOptions: FetchOptions<'json'> = {
            headers: {
                'Content-Type': 'application/json',
            }
        }
        return await this.call<FlowProject>('POST', '/projects/flow_project_100/compile', project, fetchOptions)
    }

    async runProject(project : ProjectIdentifier) : Promise<FlowProject> {
        const fetchOptions: FetchOptions<'json'> = {
            headers: {
                'Content-Type': 'application/json',
            }
        }
        return await this.call<FlowProject>('POST', `${this.RUN_JOBS}`, project, fetchOptions)
    }

    async buildProject(projectName : string) : Promise<FlowProject> {
        return await this.call<FlowProject>('GET', `${this.BUILD_PATH}${projectName}`)
    }




}

export default ProjectsModule;
