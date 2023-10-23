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

export type ProcessIdentifier = {
    process_id : number;
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
    private COMPILE_PATH: string = '/projects/{project_name}/compile';
    private RUN_PROJECT: string = '/projects/{project_name}/run';
    private STOP_PROJECT: string = '/projects/stop/{process_id}';

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

    async compileProject(project : ProjectIdentifier) : Promise<string> {
        const fetchOptions: FetchOptions<'json'> = {
            headers: {
                'Content-Type': 'application/json',
            }
        }
        return await this.call<string>('POST', this.COMPILE_PATH.replace("{project_name}", project.project_name), project, fetchOptions)
    }

    async runProject(project : ProjectIdentifier) : Promise<ProcessIdentifier> {
        const fetchOptions: FetchOptions<'json'> = {
            headers: {
                'Content-Type': 'application/json',
            }
        }
        return await this.call<ProcessIdentifier>('POST', this.RUN_PROJECT.replace("{project_name}", project.project_name), project, fetchOptions)
    }

    async stopProject(process : ProcessIdentifier) : Promise<void> {
        const fetchOptions: FetchOptions<'json'> = {
            headers: {
                'Content-Type': 'application/json',
            }
        }
        return await this.call<void>('POST', `${this.STOP_PROJECT}${process.process_id}`, process, fetchOptions)
    }

    async buildProject(projectName : string) : Promise<FlowProject> {
        return await this.call<FlowProject>('GET', `${this.BUILD_PATH}${projectName}`)
    }




}

export default ProjectsModule;
