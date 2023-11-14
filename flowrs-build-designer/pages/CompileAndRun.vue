<script setup lang="ts">

import {useProjectsStore} from "~/store/projectStore";

const projectsStore = useProjectsStore()
//TODO: new tab or new page?
const selectedProject = computed(() => projectsStore.selectedProject);
const loading = computed(() => projectsStore.loading);
const logEntries = computed(() => projectsStore.logEntries);

  const compile = () => {
  projectsStore.compileProjectRequest("cargo")

  }

  const run = () => {

  }

  const stop = () => {

  }
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
      <v-card :title="selectedProject ? selectedProject.name : 'No project selected!'" subtitle="Choose an action!">

        <v-divider></v-divider>

        <v-card-actions>
          <v-col>
            <v-btn prepend-icon="mdi-code-braces" rounded="0" size="large" @click="compile()" class="mb-2 ml-2" :loading="loading">
              <template v-slot:loader>
                <v-progress-linear indeterminate="true" color="teal" rounded height="25"> Compiling</v-progress-linear>
              </template>
              Compile project
            </v-btn>

            <v-btn prepend-icon="mdi-play" rounded="0" size="large" @click="run()" class="mb-2">
              Run project
            </v-btn>

            <v-btn prepend-icon="mdi-stop" rounded="0" size="large" @click="stop()">
              Stop execution
            </v-btn>
          </v-col>
        </v-card-actions>



      </v-card>


    </v-col>

    <v-col cols="9" class="border-col">
        <div v-for="logEntry in logEntries" :key="logEntry">{{ logEntry }}</div>

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
