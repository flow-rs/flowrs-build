// Define the types for the API response
import FetchFactory from "~/repository/factory";
import {FetchOptions} from "ofetch";
import {ProcessIdentifier} from "~/repository/modules/processes";

export type TimerConfigNode = {
    value: {
        duration: {
            nanos: number;
            secs: number;
        },
    };
};

export type ProjectIdentifier = {
    project_name: string;
}

export type TimerTokenNode = {
    value: number
};

export type FlowNode<T> = {
    node_type: string;
    type_parameters: T;
    constructor: string;
};

type Connection = {
    from_node: string;
    from_output: string;
    to_node: string;
    to_input: string;
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

export enum BuildType {
    Wasm = "wasm",
    Cargo = "cargo",
}

class ProjectsModule extends FetchFactory {
    private RESOURCE: string = '/projects/';
    private PROJECT_PATH: string = this.RESOURCE + '{project_name}/';
    private COMPILE_PATH: string = this.PROJECT_PATH + 'compile?build_type=';
    private RUN_PROJECT: string = this.PROJECT_PATH + 'run?build_type=';


    async getProjects(): Promise<FlowProject[]> {
        return await this.call<FlowProject[]>('GET', `${this.RESOURCE}`)
    }

    async createProject(project: FlowProject): Promise<FlowProject> {
        const fetchOptions: FetchOptions<'json'> = {
            headers: {
                'Content-Type': 'application/json',
            }
        }
        return await this.call<FlowProject>('POST', `${this.RESOURCE}`, project, fetchOptions)
    }

    async deleteProject(project: ProjectIdentifier): Promise<FlowProject> {
        const fetchOptions: FetchOptions<'json'> = {
            headers: {
                'Content-Type': 'application/json',
            }
        }
        return await this.call<FlowProject>('DELETE', `${this.PROJECT_PATH.replace("{project_name}", project.project_name)}`, project, fetchOptions)
    }

    async compileProject(project: ProjectIdentifier, buildType: string): Promise<string> {
        const fetchOptions: FetchOptions<'json'> = {
            headers: {
                'Content-Type': 'application/json',
            }
        }
        return await this.call<string>('POST', `${this.COMPILE_PATH.replace("{project_name}", project.project_name)}${buildType}`, project, fetchOptions)
    }

    async runProject(project: ProjectIdentifier, buildType: string): Promise<ProcessIdentifier> {
        const fetchOptions: FetchOptions<'json'> = {
            headers: {
                'Content-Type': 'application/json',
            }
        }
        return await this.call<ProcessIdentifier>('POST', `${this.RUN_PROJECT.replace("{project_name}", project.project_name)}${buildType}`, project, fetchOptions)
    }
}

export default ProjectsModule;
