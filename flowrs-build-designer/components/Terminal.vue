<script setup lang="ts">
import {useProjectsStore} from "~/store/projectStore";

const projectsStore = useProjectsStore()
const logEntries = computed(() => projectsStore.logEntries);
projectsStore.getAll()


//TODO: Autoscrolling
</script>

<template>
  <v-card title="Log Panel" >
    <div class="pb-2 pl-2 flex-content" >
      <div v-if="projectsStore.selectedProject != null" class="flex-content">
        <v-chip color="primary" class="mr-2" label>Project name: {{projectsStore.selectedProject.name}}</v-chip>
      </div>
      <v-chip color="primary" class="mr-2" label>Build/Run type: {{projectsStore.selectedBuildType}}</v-chip>
      <div v-if="projectsStore.getCurrentProcessId() != undefined" class="flex-content">
        <v-chip color="primary" class="mr-2" label>Process id: {{projectsStore.getCurrentProcessId()}}</v-chip>
        <v-chip color="success" label>Status: Runnning</v-chip>
      </div>
      <div v-else>
        <v-chip color="warning" label>Status: No process running</v-chip>
      </div>
    </div>

  </v-card>
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
