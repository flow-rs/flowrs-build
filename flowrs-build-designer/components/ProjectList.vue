<script setup lang="ts">

import {useProjectsStore} from "~/store/projectStore.js";
import type {FlowProject} from "~/repository/modules/projects";
import {newFlowProject} from "~/repository/api_sample_data";


const projectsStore = useProjectsStore()
projectsStore.getAll()

const projectClicked = ref(false)

const selectProject = (project: FlowProject) => {
  console.log("Project was selected: " + project.name)
  projectsStore.selectProject(project)
  projectClicked.value = true;
}

const openProjectAsFlow = () => {
  navigateTo('/flowbuilder')
}

const deleteProject = () => {
  console.log("Deletion of flow project was triggered")
  projectClicked.value = false;
  projectsStore.deleteProject();
}

// Testing method to create project with UI TODO: should be open flow creation page
const createProject = () => {
  let projectToCreate = newFlowProject
  projectToCreate.name = "flow_project_" + Math.floor(Math.random() * 2000) + 1;
  const {$api} = useNuxtApp();
  $api.projects.createProject(projectToCreate);
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
          <v-btn prepend-icon="mdi-open-in-app" color="primary" :disabled="!projectClicked" @click="openProjectAsFlow()">
            Open
          </v-btn>
          <v-btn prepend-icon="mdi-plus" color="success" @click="createProject()">Create flow</v-btn>
          <v-btn prepend-icon="mdi-delete-forever" color="error" :disabled="!projectClicked" @click="deleteProject()">
            Delete
          </v-btn>
          <v-btn prepend-icon="mdi-refresh" color="warning" @click="refreshProjectList()">Refresh list</v-btn>
        </v-col>
      </v-row>
    </v-card-actions>
  </v-card>
</template>

<style scoped lang="scss">

</style>
