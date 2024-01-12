<script setup lang="ts">
import {useProjectsStore} from "~/store/projectStore";

// The LogPanel is responsible for listing the current logs of a running process. It also
// shows the current process status, id, build type and compile errors.

const projectsStore = useProjectsStore()
// the current log entries of a project
const logEntries = computed(() => projectsStore.getCurrentLogEntries());
const projects = computed(() => projectsStore.projects);
const selectedProject = computed(() => projectsStore.selectedProject)
// indicates if a project has a compile error
const compileError = computed(() => projectsStore.compileErrorForSelectedProjectExist())
const emits = defineEmits(['compile-error-button-clicked']);

/**
 * If the compile error button is clicked, the event is emitted to open the dialog on top level page.
 */
const emitCompileErrorButtonClickEvent = () => {
  emits('compile-error-button-clicked')
}

</script>

<template>
  <v-card title="Log Panel">
    <div class="pb-2 pl-2 flex-content">
      <div v-if="selectedProject !== null" class="flex-content">
        <v-chip color="primary" class="mr-2" label>Project name: {{ selectedProject.name }}</v-chip>
        <v-chip color="primary" class="mr-2" label>Build/Run type: {{ projectsStore.selectedBuildType }}</v-chip>
        <div v-if="projectsStore.getCurrentProcessId() != undefined" class="flex-content">
          <v-chip color="primary" class="mr-2" label>Process id: {{ projectsStore.getCurrentProcessId() }}</v-chip>
          <v-chip color="success" label>Status: Runnning</v-chip>
        </div>
        <div v-else-if="compileError">
          <v-chip color="error" class="mr-2" label>Status: Compile error</v-chip>
        </div>
        <div v-else>
          <v-chip color="warning" class="mr-2" label>Status: Process not running</v-chip>
        </div>
      </div>
    </div>
  </v-card>
  <div v-if="compileError" class="mb-2 mt-2">
    <v-btn @click="emitCompileErrorButtonClickEvent" color="error" prepend-icon="mdi-open-in-new" size="small"
           class="mt-1 mb-1">Show error details
    </v-btn>
  </div>
  <v-card class="mt-3 pl-2 pt-2 scroll" height="400px">
    <div v-for="logEntry in logEntries" :key="logEntry">{{ logEntry }}</div>
  </v-card>

</template>

<style scoped lang="scss">
.scroll {
  overflow-y: scroll
}

.flex-content {
  display: flex;
  align-items: center
}
</style>
