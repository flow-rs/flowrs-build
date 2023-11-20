<script setup lang="ts">

import ProjectActionPane from "~/components/ProjectActionPane.vue";
import Terminal from "~/components/Terminal.vue";



  //TODO: extract compontens in own files

  //TODO: add prefix for logs which project was executed/compiled etc.


</script>

<template>

  <v-row class="mt-2 ml-2 mb-2 mr-2">
      <v-col col="4" class="border-col">
        Metrik 1
      </v-col>
      <v-col col="3" class="border-col">
        Metrik2
    </v-col>
    <v-col col="2" class="border-col">
      Metrik3
    </v-col>
    <v-col col="3" class="border-col">
      Metrik4
    </v-col>
  </v-row>

  <v-row class="ml-2 mb-2 mr-2">


    <v-col cols="3">
      <ProjectActionPane></ProjectActionPane>
    </v-col>

    <v-col cols="9">
      <Terminal></Terminal>


    </v-col>

  </v-row>

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
