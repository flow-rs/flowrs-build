import {type FetchOptions} from "ofetch";
import FetchFactory from "~/repository/factory";

export type ProcessIdentifier = {
    process_id : number;
}

// The process module uses the fetch factory to get the infos about a process from the backend.
class ProcessesModule extends FetchFactory {

    // Endpoint paths to stop a process and log a process
    private STOP_PROCESS: string = '/processes/{process_id}/stop';
    private LOGS_PROCESS: string = '/processes/{process_id}/logs';


    /**
     * API Method which kills a running process in the backend.
     * @param process - the process to stop.
     */
    async stopProcess(process: ProcessIdentifier): Promise<void> {
        const fetchOptions: FetchOptions<'json'> = {
            headers: {
                'Content-Type': 'application/json',
            }
        }
        return await this.call<void>('POST', this.STOP_PROCESS.replace("{process_id}", String(process.process_id)), process, fetchOptions)
    }

    /**
     * API Method to get the logs of a process.
     * @param process - the process to get the logs from.
     */
    async getProcessLogs(process: ProcessIdentifier): Promise<string[]> {
        return await this.call<string[]>('GET', this.LOGS_PROCESS.replace("{process_id}", String(process.process_id)))
    }

}

export default ProcessesModule;
