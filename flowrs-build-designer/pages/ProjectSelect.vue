<script setup lang="ts">

import {newFlowProject} from "~/repository/api_sample_data";
import {useProjectsStore} from "~/store/projectStore";
import {FlowProject} from "~/repository/modules/projects";

const projectsStore = useProjectsStore();
const selectedProject: FlowProject = computed(() => projectsStore.selectedProject);

const displayObject = ref(null);


const handleFilterSelection = (value) => {
  switch (value) {
    case 'noFilter':
      displayObject.value = selectedProject;
      break;
    case 'packages':
      displayObject.value = selectedProject.packages
      break;
    case 'flow':
      displayObject.value = selectedProject.flow
      break;
  }
}

// Testing method to create project with UI
const createProject = () => {
  let projectToCreate = newFlowProject
  projectToCreate.name = "Name_" + Math.random()
  const {$api} = useNuxtApp();
  $api.projects.createProject(projectToCreate);
}

const myObject = JSON.stringify(JSON.parse('[{"id":1,"name":"A green door","price":12.50,"tags":["home","green"]},' +
    '{"id":1,"name":"A green door","price":12.50,"tags":["home","green"]},' +
    '{"id":1,"name":"A green door","price":12.50,"tags":["home","green"]},' +
    '{"id":1,"name":"A green door","price":12.50,"tags":["home","green"]}' +
    ']'), null, 2)

</script>


<template>
  <v-row>
    <v-col class="text-center mt-5 ml-5">
      <ProjectList :card-title="'Projects'" :card-subtitle="'Choose your project'"></ProjectList>
      <v-btn class="mt-5" @click="createProject()">Create project (Testing-purpose)</v-btn>
    </v-col>

    <v-col class="mt-5 mr-5">
      <v-card :title="selectedProject ? selectedProject.name : 'No project selected!'" subtitle="flow-project.json">
        <v-divider></v-divider>
        <v-row>
          <v-col>
            <v-btn-toggle mandatory>
              <v-btn @click="handleFilterSelection('noFilter')">
                <v-icon>mdi-filter-off</v-icon>
              </v-btn>
              <v-btn @click="handleFilterSelection('packages')">
                <v-icon>mdi-package</v-icon>
              </v-btn>
              <v-btn @click="handleFilterSelection('flow')">
                <v-icon>mdi-call-split</v-icon>
              </v-btn>
            </v-btn-toggle>
          </v-col>
        </v-row>
        <div class=" scroll">
          <pre>{{ displayObject }}</pre>

        </div>
      </v-card>

    </v-col>


  </v-row>

</template>

<style scoped>

div.scroll {
  height: 650px;
  overflow-x: hidden;
  overflow-y: auto;
  padding: 20px;
}
</style>
