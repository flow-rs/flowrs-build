<script setup lang="ts">

import {newFlowProject} from "~/repository/api_sample_data";
import {useProjectsStore} from "~/store/projectStore";
import {FlowProject} from "~/repository/modules/projects";

const projectsStore = useProjectsStore();
const selectedProject: FlowProject = computed(() => projectsStore.selectedProject);

const displayedObject = computed(() => projectsStore.displayedJSON);

const activeFilter = computed(() => projectsStore.activeFilter);

const handleFilterSelection = (value) => {
  projectsStore.setActiveFilter(value)
  switch (value) {
    case 'noFilter':
      projectsStore.setDisplayedJSON(selectedProject.value)
      break;
    case 'packages':
      projectsStore.setDisplayedJSON(selectedProject.value.packages)
      break;
    case 'flow':
      projectsStore.setDisplayedJSON(selectedProject.value.flow)
      break;
    default:
      projectsStore.setDisplayedJSON(null)
  }
};


// Testing method to create project with UI
const createProject = () => {
  let projectToCreate = newFlowProject
  projectToCreate.name = "Name_" + Math.random()
  const {$api} = useNuxtApp();
  $api.projects.createProject(projectToCreate);
}

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
              <v-btn @click="handleFilterSelection('noFilter')" :active="activeFilter==='noFilter'">
                <v-icon>mdi-filter-off</v-icon>
              </v-btn>
              <v-btn @click="handleFilterSelection('packages')" :active="activeFilter==='packages'">
                <v-icon>mdi-package</v-icon>
              </v-btn>
              <v-btn @click="handleFilterSelection('flow')" :active="activeFilter==='flow'">
                <v-icon>mdi-call-split</v-icon>
              </v-btn>
            </v-btn-toggle>
          </v-col>
        </v-row>
        <div class=" scroll">
              <pre class="language-json">
      <code>{{ displayedObject }}</code>
    </pre>

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
