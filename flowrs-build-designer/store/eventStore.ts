import {defineStore} from 'pinia'

export const useEventsStore = defineStore({
    id: 'events',
    state: () => ({
        isSaveButtonClicked: false,
        showAlert: false,
        isLoadingSave: false,
        errorMessageAlert: "",
    }),
    actions: {
        setSaveButtonClicked(value:boolean) {
            this.isSaveButtonClicked = value;
        },

        setAlert(active: boolean) {
            this.showAlert = active
        },

        setLoading(active: boolean) {
            this.isLoadingSave = active
        },

        setErrorMessage(message: string) {
            this.errorMessageAlert = message
        }



    },
})
