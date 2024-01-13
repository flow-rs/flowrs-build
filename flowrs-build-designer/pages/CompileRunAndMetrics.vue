<script setup lang="ts">
import {useProjectsStore} from "~/store/projectStore";
import {onBeforeUnmount, onMounted, ref} from 'vue';
import MetricPanelPlaceholder from "~/components/MetricPanelPlaceholder.vue";
import MetricPanel from "~/components/MetricPanel.vue";

// The compile, run and metrics page displays the log, control, and metric panel components.

// Set up an interval variable
let interval: NodeJS.Timeout;
const projectsStore = useProjectsStore()
projectsStore.getAll()
// Get the compile errors of a project.
const compileErrorObjects = computed(() => projectsStore.getCurrentCompileErrorsOfProject());
const currentProcessId = computed(() => projectsStore.getCurrentProcessId());
let openCompileErrorDialog = ref(false);

/**
 * Catch the event if the compile error button is clicked in log panel component.
 */
const handleCompileErrorButtonClick = () => {
  openCompileErrorDialog.value = true
}

/**
 * Called to close the dialog.
 */
const closeDialog = () => {
  openCompileErrorDialog.value = false;
}

/**
 * Get the current logs of the project.
 */
const getCurrentLogs = () => {
  projectsStore.getLogs()
}

/**
 * Formatting the compile error message.
 * @param message - unformatted error message.
 */
const formattedErrorMessage = (message: string) => {
  const formatted = message.replace(/\\n/g, '<br>');
  return formatted
}

// if the page is mounted the current logs are fetched and the timer to automatically fetch logs is called
onMounted(() => {
  getCurrentLogs(); // Fetch data immediately when the component is mounted
  interval = setInterval(getCurrentLogs, 2500);
});

// clear the timer
onBeforeUnmount(() => {
  clearInterval(interval);
});


</script>

<template>
  <v-container fluid>
    <v-row justify="center">
      <v-dialog
          v-model="openCompileErrorDialog"
          scrollable>
        <v-card title="Compile Error Overview">
          <v-card-actions>
            <v-btn prepend-icon="mdi-close" color="primary"
                   @click="closeDialog">
              Close
            </v-btn>
          </v-card-actions>
          <v-expansion-panels>
            <v-expansion-panel
                v-for="i in compileErrorObjects"
                :key="i"
            >
              <v-expansion-panel-title>{{ i.title }}</v-expansion-panel-title>
              <v-expansion-panel-text>
                <div v-html="formattedErrorMessage(i.message)" class="preserve-whitespace"></div>
              </v-expansion-panel-text>
            </v-expansion-panel>
          </v-expansion-panels>

        </v-card>
      </v-dialog>
    </v-row>
    <MetricPanel v-if="currentProcessId"></MetricPanel>
    <MetricPanelPlaceholder v-else></MetricPanelPlaceholder>

    <v-row>
      <v-col cols="3">
        <ControlPanel></ControlPanel>
      </v-col>
      <v-col cols="9">
        <LogPanel @compile-error-button-clicked="handleCompileErrorButtonClick"></LogPanel>
      </v-col>
    </v-row>
  </v-container>
</template>

<style scoped>


.preserve-whitespace {
  white-space: pre-wrap;
}


</style>
