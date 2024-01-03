

<template>
  <div class="mb-5">
    <v-app-bar elevation="0">
      <v-img class="mx-2" src="~/assets/images/flowers-logo.png" max-height="40" max-width="40" contain></v-img>
      <v-toolbar-title>
        Flowrs - Flow Builder
      </v-toolbar-title>
      <v-btn v-if="route.name=='FlowBuilder'" prepend-icon="mdi-content-save-all-outline" variant="elevated"
             class="ma-1" color="success" @click="handleSaveButtonClick">Save Project
      </v-btn>
      <v-btn v-if="route.name=='FlowEditor'" prepend-icon="mdi-content-save-all-outline" variant="elevated"
             class="ma-1" color="success" @click="handleSaveButtonClick">Save Project
      </v-btn>
      <v-btn prepend-icon="mdi-open-in-app" variant="elevated" class="ma-1" color="primary" to="/">Project Select
      </v-btn>
      <v-btn prepend-icon="mdi-package-variant" variant="elevated" class="ma-1" color="primary" to="/packageSelect">
        Package
        Info
      </v-btn>
      <v-btn prepend-icon="mdi-run" variant="elevated" class="ma-1" color="primary" to="/compileRunAndMetrics">Compile,
        Run and Metrics
      </v-btn>
      <v-spacer></v-spacer>
    </v-app-bar>
  </div>
</template>

<script lang="ts">
import {useEventsStore} from "~/store/eventStore";
export default {
  methods: {
    handleSaveButtonClick() {
      const store = useEventsStore();
      store.setSaveButtonClicked(true);
    },
  },
}
</script>

<script setup lang="ts">
import {useRoute} from 'vue-router'

const route = useRoute()
</script>
