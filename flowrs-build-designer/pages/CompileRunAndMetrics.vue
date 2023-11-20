<script setup lang="ts">

import ProjectActionPane from "~/components/ProjectActionPane.vue";
import Terminal from "~/components/Terminal.vue";
import {useProjectsStore} from "~/store/projectStore";


  //TODO: extract compontens in own files

  //TODO: add prefix for logs which project was executed/compiled etc.


</script>

<template>
  <v-container fluid>
  <v-row align-content="center">
    <v-col>
      <v-card title="Metric Panel"></v-card>
    </v-col>
  </v-row>
  <v-row align-content="center">
      <v-col col="4">
        <v-card title="Metric XY">
          <v-img src="~assets/images/1340003.png" aspect-ratio="2.5"></v-img>
        </v-card>
      </v-col>
      <v-col col="3">
        <v-card title="Metric XY">
          <v-img src="~assets/images/1340003.png" aspect-ratio="2.5"></v-img>
        </v-card>
    </v-col>
    <v-col col="2">
      <v-card title="Metric XY">
        <v-img src="~assets/images/1340003.png" aspect-ratio="2.5"></v-img>
      </v-card>
    </v-col>
    <v-col col="3">
      <v-card title="Metric XY">
        <v-img src="~assets/images/1340003.png" aspect-ratio="2.5"></v-img>
      </v-card>
    </v-col>
  </v-row>

  <v-row>
    <v-col cols="3">
      <ProjectActionPane></ProjectActionPane>
    </v-col>
    <v-col cols="9">
      <Terminal></Terminal>
    </v-col>
  </v-row>
  </v-container>
</template>

<style scoped>
.border-row {
  border: 4px dashed #ccc;
  padding: 30px;
}

.border-col {
  border: 2px dashed #ccc;
  padding: 10px;
}

.background-prototyping {
  background: beige;
}

</style>
