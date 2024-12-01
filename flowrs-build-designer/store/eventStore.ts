import {defineStore} from 'pinia'

export const useEventsStore = defineStore({
    id: 'events',
    state: () => ({
        isSaveButtonClicked: false,
        showAlert: false,
        isLoadingSave: false,
        errorMessageAlert: "",
        isTextFieldClicked: false,
        loadingPrompt: false,
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
        },

        setIsTextFieldClicked(value: boolean) {
            this.isTextFieldClicked = value
        },
        setLoadingPrompt(value: boolean) {
            this.loadingPrompt = value
        }

    },
})
