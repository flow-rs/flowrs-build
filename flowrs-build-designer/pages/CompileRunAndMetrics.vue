<script setup lang="ts">
import {useProjectsStore} from "~/store/projectStore";
import { ref, onMounted, onBeforeUnmount } from 'vue';


// Set up an interval variable
let interval: NodeJS.Timeout;

const projectsStore = useProjectsStore()
const compileErrorObjects = computed(() => projectsStore.compileErrorObjects);
const currentProcessId = computed(() => projectsStore.getCurrentProcessId());
let openCompileErrorDialog = ref(false);
const handleCompileErrorButtonClick = () => {
  console.log('catch event')
  openCompileErrorDialog.value = true
}

const closeDialog = () => {
  openCompileErrorDialog.value = false;
}

const getCurrentLogs = () => {
  projectsStore.getLogs()
}

onMounted(() => {
  getCurrentLogs(); // Fetch data immediately when the component is mounted
  interval = setInterval(getCurrentLogs, 5000);
});

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
            <v-expansion-panel-title>{{i.title}}</v-expansion-panel-title>
            <v-expansion-panel-text>{{i.message}}</v-expansion-panel-text>
          </v-expansion-panel>

        </v-expansion-panels>

      </v-card>
    </v-dialog>
    </v-row>
    <v-row>
      <v-col>
        <v-card title="Metric Panel"></v-card>
      </v-col>
    </v-row>
    <v-row align-content="center">
      <v-col col="3">
        <div class="container">
          <iframe v-if="currentProcessId" class="responsive-iframe" :src="`http://localhost:3030/d-solo/flowrs-prometheus/flowrs-live-metrics?orgId=1&refresh=1s&var-job=flowrs-${currentProcessId}&from=now-5m&to=now&panelId=1&theme=light`" frameborder="0"></iframe>
        </div>
      </v-col>
      <v-col col="3">
        <div class="container">
          <iframe v-if="currentProcessId" class="responsive-iframe" :src="`http://localhost:3030/d-solo/flowrs-prometheus/flowrs-live-metrics?orgId=1&refresh=1s&var-job=flowrs-${currentProcessId}&from=now-5m&to=now&theme=light&panelId=2`" frameborder="0"></iframe>
        </div>
      </v-col>
      <v-col col="3">
        <div class="container">
          <iframe v-if="currentProcessId" class="responsive-iframe" :src="`http://localhost:3030/d-solo/flowrs-prometheus/flowrs-live-metrics?orgId=1&refresh=1s&var-job=flowrs-${currentProcessId}&from=now-5m&to=now&theme=light&panelId=3`" frameborder="0"></iframe>
        </div>
      </v-col>
      <v-col col="3">
        <div class="container">
          <iframe v-if="currentProcessId" class="responsive-iframe" :src="`http://localhost:3030/d-solo/flowrs-prometheus/flowrs-live-metrics?orgId=1&refresh=1s&var-job=flowrs-${currentProcessId}&from=now-5m&to=now&theme=light&panelId=4`" frameborder="0"></iframe>
        </div>
      </v-col>
    </v-row>

    <v-row>
      <v-col cols="3">
        <ProjectActionPane></ProjectActionPane>
      </v-col>
      <v-col cols="9">
        <Terminal @compile-error-button-clicked="handleCompileErrorButtonClick"></Terminal>
      </v-col>
    </v-row>
  </v-container>
</template>

<style scoped>
.responsive-iframe {
  position: absolute;
  top: 0;
  left: 0;
  bottom: 0;
  right: 0;
  width: 100%;
  height: 100%;
}

.container {
  position: relative;
  overflow: hidden;
  width: 100%;
  padding-top: 56.25%; /* 16:9 Aspect Ratio (divide 9 by 16 = 0.5625) */
}
</style>
