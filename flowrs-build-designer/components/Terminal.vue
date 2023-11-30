<script setup lang="ts">
import {useProjectsStore} from "~/store/projectStore";
import { defineEmits } from 'vue';

const projectsStore = useProjectsStore()
const logEntries = computed(() => projectsStore.getCurrentLogEntries());
const projects = computed(() => projectsStore.projects);
const selectedProject = computed(() => projectsStore.selectedProject)
const compileError = computed(() => projectsStore.compileError)
const emits = defineEmits(['compile-error-button-clicked']);
const emitCompileErrorButtonClickEvent = () => {
  console.log("emit event")
  emits('compile-error-button-clicked')
}

//TODO: Autoscrolling
</script>

<template>
  <v-card title="Log Panel">
    <div class="pb-2 pl-2 flex-content">
      <div v-if="projects.length!==0" class="flex-content">
      <v-chip color="primary" class="mr-2" label>Project name: {{ selectedProject.name }}</v-chip>
      <v-chip color="primary" class="mr-2" label>Build/Run type: {{ projectsStore.selectedBuildType }}</v-chip>
      <div v-if="projectsStore.getCurrentProcessId() != undefined" class="flex-content">
        <v-chip color="primary" class="mr-2" label>Process id: {{ projectsStore.getCurrentProcessId() }}</v-chip>
        <v-chip color="success" label>Status: Runnning</v-chip>
      </div>
      <div v-else-if="compileError">
      <v-chip color="error"  class="mr-2" label>Status: Compile error</v-chip>
      </div>
        <div v-else><v-chip color="warning"  class="mr-2" label>Status: No process running</v-chip></div>
      </div>
    </div>
  </v-card>
  <div v-if="compileError" class="mb-2 mt-2">
      <v-btn @click="emitCompileErrorButtonClickEvent" color="error" prepend-icon="mdi-open-in-new" size="small" class="mt-1 mb-1" >Show error details</v-btn>
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
