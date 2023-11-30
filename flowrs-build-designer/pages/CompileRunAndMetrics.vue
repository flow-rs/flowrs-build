<script setup lang="ts">
import {useProjectsStore} from "~/store/projectStore";

const projectsStore = useProjectsStore()
const errorMessageText = computed(() => projectsStore.compileErrorText);
let openCompileErrorDialog = ref(false);
const handleCompileErrorButtonClick = () => {
  console.log('catch event')
  openCompileErrorDialog.value = true
}

const closeDialog = () => {
  openCompileErrorDialog.value = false;
}

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
              v-for="i in errorMessageText"
              :key="i"
          >
            <v-expansion-panel-title>{{i}}</v-expansion-panel-title>
            <v-expansion-panel-text>Blabla</v-expansion-panel-text>
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
      <v-col col="4">
        <div class="container">
          <iframe class="responsive-iframe" src="http://localhost:3000/d-solo/flowrs-prometheus/flowrs-live-metrics?orgId=1&refresh=1s&from=1700771461023&to=1700771761023&panelId=1&theme=light" frameborder="0"></iframe>
        </div>
      </v-col>
      <v-col col="3">
        <div class="container">
          <iframe class="responsive-iframe" src="http://localhost:3000/d-solo/flowrs-prometheus/flowrs-live-metrics?orgId=1&refresh=1s&from=1700772274381&to=1700772574381&theme=light&panelId=2" frameborder="0"></iframe>
        </div>
      </v-col>
      <v-col col="2">
        <div class="container">
          <iframe class="responsive-iframe" src="http://127.0.0.1:3000/d-solo/flowrs-prometheus/flowrs-live-metrics?orgId=1&refresh=1s&from=1700772347957&to=1700772647957&theme=light&panelId=3" frameborder="0"></iframe>
        </div>
      </v-col>
      <v-col col="3">
        <div class="container">
          <iframe class="responsive-iframe" src="http://localhost:3000/d-solo/flowrs-prometheus/flowrs-live-metrics?orgId=1&refresh=1s&var-job=flowrs-4883&from=now-5m&to=now&theme=light&panelId=1" frameborder="0"></iframe>
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
