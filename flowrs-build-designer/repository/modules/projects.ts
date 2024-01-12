// Define the types for the API response
import FetchFactory from "~/repository/factory";

import type {ProcessIdentifier} from "~/repository/modules/processes";
import type {FetchOptions} from "ofetch";

// File contains API Module to work with projects and also the type definitons of a project.

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

export type NodeModel = {
    node_type: string;
    type_parameters: { [key: string]: string };
    constructor: string;
};

export type ConnectionModel = {
    from_node: string;
    from_output: string;
    to_node: string;
    to_input: string;
};

export type CompileError = {
    title: string,
    message: string,
}
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
        nodes: { [key: string]: NodeModel };
        connections: ConnectionModel[];
        data: { [key: string]: any };
    };
};

export type LastCompile = {
    modified_time: string;
}

// The projects module uses the fetch factory to get the infos about a project from the backend and to work with a project.
class ProjectsModule extends FetchFactory {

    // Endpoint paths to stop a process and log a process
    private RESOURCE: string = '/projects/';
    private PROJECT_PATH: string = this.RESOURCE + '{project_name}/';
    private COMPILE_PATH: string = this.PROJECT_PATH + 'compile?build_type=';
    private RUN_PROJECT: string = this.PROJECT_PATH + 'run?build_type=';
    private LAST_COMPILE: string = this.PROJECT_PATH + 'last_compile?build_type=';

    /**
     * API Method to get the projects from the backend.
     */
    async getProjects(): Promise<FlowProject[]> {
        return await this.call<FlowProject[]>('GET', `${this.RESOURCE}`)
    }

    /**
     * API Method to create a project.
     */
    async createProject(project: FlowProject): Promise<Response> {
        const fetchOptions: FetchOptions<'json'> = {
            headers: {
                'Content-Type': 'application/json',
            }
        }
        return await this.call<Response>('POST', `${this.RESOURCE}`, project, fetchOptions)
    }

    /**
     * API Method to delete a project.
     */
    async deleteProject(project: ProjectIdentifier): Promise<string> {
        const fetchOptions: FetchOptions<'json'> = {
            headers: {
                'Content-Type': 'application/json',
            }
        }
        return await this.call<string>('DELETE', `${this.PROJECT_PATH.replace("{project_name}", project.project_name)}`, project, fetchOptions)
    }

    /**
     * API Method to compile a project.
     */
    async compileProject(project: ProjectIdentifier, buildType: string): Promise<Response> {
        const fetchOptions: FetchOptions<'json'> = {
            headers: {
                'Content-Type': 'application/json',
            }
        }
        return await this.call<Response>('POST', `${this.COMPILE_PATH.replace("{project_name}", project.project_name)}${buildType}`, project, fetchOptions)
    }

    /**
     * API Method to get the last compile time of a project.
     */
    async lastCompileOfProject(project: ProjectIdentifier, buildType: string): Promise<LastCompile> {
        return await this.call<LastCompile>('GET', `${this.LAST_COMPILE.replace("{project_name}", project.project_name)}${buildType}`)
    }

    /**
     * API Method to run the project with the specified buildType.
     */
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
