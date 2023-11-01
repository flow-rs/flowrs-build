<script setup lang="ts">

import {defineProps} from 'vue'

import {useProjectsStore} from "~/store/projectStore.js";


const projectsStore = useProjectsStore()
projectsStore.getAll()

const selectedProject = ref(null)
const projectClicked = ref(false)

const selectProject = (project) => {
  const p: FlowProject = project
  console.log("Project was selected: " + p.name)
  selectedProject.value = project
  projectClicked.value = true;
}

const openProjectAsFlow = () => {
  console.log("TODO: Navigate to flow page and open the project as flow")
  navigateTo('/')
}

const deleteProject = () => {
  console.log("Deletion of flow project was triggered")
  projectsStore.deleteProject(selectedProject.value);
}

const refreshProjectList = () => {
  console.log("Refreshing list of projects...")
  projectsStore.getAll()
}
defineProps({
  cardTitle: {type: String, default: "Projects"},
  cardSubtitle: {type: String, default: "Choose your project!"},
});
</script>

<template>
  <v-card :title="cardTitle" :subtitle="cardSubtitle" variant="elevated">
    <v-divider></v-divider>
    <v-list>
      <v-list-item
          v-for="project in projectsStore.projects"
          :key="project.name"
          :value="project"
          color="primary"
          :title="project.name"
          :subtitle="project.version"
          @click="selectProject(project)"
      ></v-list-item>
    </v-list>
    <v-card-actions>
      <v-row class="mb-2 mt-2">
        <v-col class="d-flex justify-space-around">
          <v-btn prepend-icon="mdi-open-in-app" color="blue" :disabled="!projectClicked" @click="openProjectAsFlow()">
            Open
          </v-btn>
          <v-btn prepend-icon="mdi-delete-forever" color="red" :disabled="!projectClicked" @click="deleteProject()">
            Delete
          </v-btn>
          <v-btn prepend-icon="mdi-refresh" color="orange" @click="refreshProjectList()">Refresh list</v-btn>
        </v-col>
      </v-row>
    </v-card-actions>
  </v-card>
</template>

<style scoped lang="scss">

</style>
