import {FetchOptions} from "ofetch";
import FetchFactory from "~/repository/factory";

export type ProcessIdentifier = {
    process_id : number;
}

class ProcessesModule extends FetchFactory {

    private STOP_PROCESS: string = '/processes/{process_id}/stop';
    private LOGS_PROCESS: string = '/processes/{process_id}/logs';


    async stopProcess(process: ProcessIdentifier): Promise<void> {
        const fetchOptions: FetchOptions<'json'> = {
            headers: {
                'Content-Type': 'application/json',
            }
        }
        return await this.call<void>('POST', this.STOP_PROCESS.replace("{process_id}", String(process.process_id)), process, fetchOptions)
    }

    async getProcessLogs(process: ProcessIdentifier): Promise<string[]> {
        return await this.call<string[]>('GET', this.LOGS_PROCESS.replace("{process_id}", String(process.process_id)))
    }

}

export default ProcessesModule;
