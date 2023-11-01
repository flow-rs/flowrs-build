<script setup lang="ts">
import {useProjectsStore} from "~/store/projectStore.js";

const projectsStore = useProjectsStore()
projectsStore.getAll()
const selectedProject = ref(null)
const projectClicked = ref(false)

const selectProject = (project) => {
  selectedProject.value = project
  projectClicked.value = true;
  console.log(projectClicked)
}

const openProjectAsFlow = () => {
  console.log("TODO: Navigate to flow page and open the project as flow")
}

const deleteProject = () => {
  console.log("Deleting of flow project was triggered")
  projectsStore.deleteProject();
}

</script>


<template>

  <v-row>
    <v-col class="text-center mt-5 ml-5">
      <v-card title="Projects" subtitle="Choose your project!" variant="elevated">
        <v-divider></v-divider>
        <v-list>
          <v-list-item v-for="project in projectsStore.projects" :key="project.name" :value="project" color="primary"
                       :title="project.name" :subtitle="project.version" @click="selectProject(project)">
          </v-list-item>
        </v-list>
        <v-card-actions>
          <v-row class="mb-2 mt-2">
            <v-col class="d-flex justify-space-around">
              <v-btn prepend-icon="mdi-open-in-app" color="blue" :disabled="!projectClicked" @click="openProjectAsFlow()">Open</v-btn>
              <v-btn prepend-icon="mdi-delete-forever" color="red" :disabled="!projectClicked" @click="deleteProject()">Delete</v-btn>
              <v-btn prepend-icon="mdi-refresh" color="orange">Refresh list</v-btn>
            </v-col>
          </v-row>
        </v-card-actions>

      </v-card>


    </v-col>

    <v-col class="mt-5 ml-5">Das ist eine zweite Spalte</v-col>


  </v-row>

</template>
