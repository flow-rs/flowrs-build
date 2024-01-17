<script setup lang="ts">
import {useProjectsStore} from "~/store/projectStore";

// The metric panel is displaying the grafana dashoards as iframe integration.

const projectsStore = useProjectsStore()

const currentProcessId = computed(() => projectsStore.getCurrentProcessId());
</script>

<template>
  <v-row>
    <v-col>
      <v-card class="mx-auto">
        <v-card-title>Metric Panel
          <a href="http://localhost:3030/d/home/home?orgId=1">
            <v-icon icon="mdi-open-in-new" size="18" class="me-1 pb-1"></v-icon>
          </a>
        </v-card-title>
      </v-card>
    </v-col>
  </v-row>
  <v-row align-content="center">
    <v-col col="3">
      <div  class="container">
        <iframe  class="responsive-iframe"
                 :src="`http://localhost:3030/d-solo/flowrs-prometheus/flowrs-live-metrics?orgId=1&refresh=1s&var-job=flowrs-${currentProcessId}&from=now-5m&to=now&panelId=1&theme=light`"
                 frameborder="0"></iframe>
      </div>
    </v-col>
    <v-col col="3">
      <div  class="container">
        <iframe  class="responsive-iframe"
                 :src="`http://localhost:3030/d-solo/flowrs-prometheus/flowrs-live-metrics?orgId=1&refresh=1s&var-job=flowrs-${currentProcessId}&from=now-5m&to=now&theme=light&panelId=2`"
                 frameborder="0"></iframe>
      </div>
    </v-col>
    <v-col col="3">
      <div class="container">
        <iframe  class="responsive-iframe"
                 :src="`http://localhost:3030/d-solo/flowrs-prometheus/flowrs-live-metrics?orgId=1&refresh=1s&var-job=flowrs-${currentProcessId}&from=now-5m&to=now&theme=light&panelId=3`"
                 frameborder="0"></iframe>
      </div>
    </v-col>
    <v-col col="3">
      <div  class="container">
        <iframe  class="responsive-iframe"
                 :src="`http://localhost:3030/d-solo/flowrs-prometheus/flowrs-live-metrics?orgId=1&refresh=1s&var-job=flowrs-${currentProcessId}&from=now-5m&to=now&theme=light&panelId=4`"
                 frameborder="0"></iframe>
      </div>
    </v-col>
  </v-row>
</template>

<style scoped lang="scss">

.container {
  position: relative;
  overflow: hidden;
  width: 100%;
  padding-top: 56.25%; /* 16:9 Aspect Ratio (divide 9 by 16 = 0.5625) */
}

.responsive-iframe {
  position: absolute;
  top: 0;
  left: 0;
  bottom: 0;
  right: 0;
  width: 100%;
  height: 100%;
}
</style>
