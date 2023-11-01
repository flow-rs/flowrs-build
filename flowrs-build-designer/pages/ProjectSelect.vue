<script setup lang="ts">

import {newFlowProject} from "~/repository/api_sample_data";
import {useProjectsStore} from "~/store/projectStore";

const projectsStore = useProjectsStore();
const selectedProject = computed(() => projectsStore.selectedProject);

const activeFilter = computed(() => projectsStore.activeFilter);

const handleFilterSelection = (value: any) => {
  projectsStore.setActiveFilter(value)
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
            <template v-if="activeFilter==='noFilter'">
              <code>{{ selectedProject }}</code>
            </template>
            <template v-else-if="activeFilter==='packages'">
              <code>{{ selectedProject ? selectedProject.packages : "nothing to show" }}</code>
            </template>
            <template v-else-if="activeFilter==='flow'">
              <code>{{ selectedProject ? selectedProject.flow : "nothing to show" }}</code>
            </template>
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
