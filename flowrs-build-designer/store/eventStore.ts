import {defineStore} from 'pinia'

export const useEventsStore = defineStore({
    id: 'events',
    state: () => ({
        isSaveButtonClicked: false,
    }),
    actions: {
        setSaveButtonClicked(value:boolean) {
            console.log("Save button event", value);
            this.isSaveButtonClicked = value;
        },
    },
})
