<script>
    import { useOpenAI } from "~/generator/index.ts";
    import {useEventsStore} from "~/store/eventStore";
    import {useProjectsStore} from "~/store/projectStore";

    const projectsStore = useProjectsStore();
    const eventsStore = useEventsStore();


    export default {
        data: () => ({
            loaded: false,
            loading: false,
            message: ''
        }),

        methods: {

            /*
            * trigger the generation of a flow and pass the prompt.
            */
            onClick() {
                eventsStore.setLoadingPrompt(true);
                eventsStore.setIsTextFieldClicked(true);       
                projectsStore.setPrompt(this.message);
                this.loading = true;
                console.log(this.message);
                eventsStore.$subscribe((mutation, state) => {
                    if (!state.loadingPrompt) {
                        this.loading = false
                    }
                })
            },
        },
    }
</script>

<template>
    <v-card class="mx-auto"
            color="surface-light">
        <v-card-text>
            <v-text-field v-model="message"
                          :loading="loading"
                          append-inner-icon="mdi-magnify"
                          density="compact"
                          label="Generate something!"
                          variant="solo"
                          hide-details
                          single-line
                          @click:append-inner="onClick"
                          @keypress.enter="onClick"></v-text-field>
        </v-card-text>
    </v-card>
</template>