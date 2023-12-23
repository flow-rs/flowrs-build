// Define the types for the API response
import FetchFactory from "~/repository/factory";

import type {ProcessIdentifier} from "~/repository/modules/processes";
import type {FetchOptions} from "ofetch";

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

export type FlowNode = {
    node_type: string;
    type_parameters: { [key: string]: string };
    constructor: string;
};

export type FlowConnection = {
    from_node: string;
    from_output: string;
    to_node: string;
    to_input: string;
};

export type CompileError = {
    title: string,
    message: string,
}

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
        path?: string;
        git?: string;
        branch?: string;
    }>;
    flow: {
        nodes: { [key: string]: FlowNode };
        connections: FlowConnection[];
        data: { [key: string]: any };
    };
};

export type LastCompile = {
    modified_time: string;
}

class ProjectsModule extends FetchFactory {
    private RESOURCE: string = '/projects/';
    private PROJECT_PATH: string = this.RESOURCE + '{project_name}/';
    private COMPILE_PATH: string = this.PROJECT_PATH + 'compile?build_type=';
    private RUN_PROJECT: string = this.PROJECT_PATH + 'run?build_type=';
    private LAST_COMPILE: string = this.PROJECT_PATH + 'last_compile?build_type=';


    async getProjects(): Promise<FlowProject[]> {
        return await this.call<FlowProject[]>('GET', `${this.RESOURCE}`)
    }

    async createProject(project: FlowProject): Promise<Response> {
        const fetchOptions: FetchOptions<'json'> = {
            headers: {
                'Content-Type': 'application/json',
            }
        }
        return await this.call<Response>('POST', `${this.RESOURCE}`, project, fetchOptions)
    }

    async deleteProject(project: ProjectIdentifier): Promise<string> {
        const fetchOptions: FetchOptions<'json'> = {
            headers: {
                'Content-Type': 'application/json',
            }
        }
        return await this.call<string>('DELETE', `${this.PROJECT_PATH.replace("{project_name}", project.project_name)}`, project, fetchOptions)
    }

    async compileProject(project: ProjectIdentifier, buildType: string): Promise<Response> {
        const fetchOptions: FetchOptions<'json'> = {
            headers: {
                'Content-Type': 'application/json',
            }
        }
        return await this.call<Response>('POST', `${this.COMPILE_PATH.replace("{project_name}", project.project_name)}${buildType}`, project, fetchOptions)
    }

    async lastCompileOfProject(project: ProjectIdentifier, buildType: string): Promise<LastCompile> {
        return await this.call<LastCompile>('GET', `${this.LAST_COMPILE.replace("{project_name}", project.project_name)}${buildType}`)
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
