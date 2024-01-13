<script setup lang="ts">

import {useProjectsStore} from "~/store/projectStore.js";
import type {FlowProject} from "~/repository/modules/projects";
import {emptyFlowProject, dummyFlowProject} from "~/repository/api_sample_data";


// The project list shows all projects which are stored in the backend. The user can select and delete a project.
// The user can also create a project with empty nodes and connections or an example project with some nodes and
// connections inserted.

const emits = defineEmits(['project-selected']);


const projectsStore = useProjectsStore()

const loading = computed(() => projectsStore.loading);
const projectList = computed(() => projectsStore.projects);
const projectClicked = computed(() => projectsStore.projectClickedInList);

/**
 * If a project is selected the event is emitted to deliver the currently selected project to the top level components.
 */
const emitProjectSelectionEvent = () => {
  emits('project-selected')
}

/**
 * Called if the user selects a project. The selected project is stored in a pinia store.
 */
const selectProject = (project: FlowProject) => {
  console.log("Project was selected: " + project.name)
  projectsStore.selectProject(project)
  emitProjectSelectionEvent()
}

/**
 * Called if the user presses the open button. The user is navigated to the flow builder page and the project is opened.
 */
const openProjectAsFlow = () => {
  navigateTo('/flowbuilder')
}

/**
 * Called if the user presses the delete button next to a project item. The item will be deleted in backend.
 * @param name
 */
const deleteProject = (name: string) => {
  console.log("Deletion of flow project was triggered")
  projectsStore.deleteProject(name);
}

/**
 * Called if the user press the button to create a new flow project. A project without nodes and connections will be
 * created in the backend.
 */
const createProject = () => {
  let projectToCreate = emptyFlowProject
  projectToCreate.name = "flow_project_" + Math.floor(Math.random() * 2000) + 1;
  projectsStore.createProject(projectToCreate)
}

/**
 * Called if the user press the button to create a new example flow project. A project wiht nodes and connections will be
 * created in the backend.
 */
const createDummyProject = () => {
  let projectToCreate = dummyFlowProject
  projectToCreate.name = "flow_project_" + Math.floor(Math.random() * 2000) + 1;
  projectsStore.createProject(projectToCreate)
}

/**
 * Called if the user press the refresh button. The projects get fetched from the backend.
 */
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
  <v-overlay :value="loading">
    <v-progress-circular indeterminate color="primary"></v-progress-circular>
  </v-overlay>
  <v-container fluid>
    <v-row>
      <v-col>
        <v-card variant="elevated">
          <v-row>
            <v-col class="text-center" style="align-items: center; justify-content: start" cols="11">
              <v-card-title>
                {{ cardTitle }}
              </v-card-title>
              <v-card-subtitle>
                {{ cardSubtitle }}
              </v-card-subtitle>
            </v-col>
            <v-col class="text-center" style="align-items: center; justify-content: end" cols="1">
              <v-btn color="transparent" elevation="0" @click="refreshProjectList()" icon>
                <v-icon color="warning">mdi-refresh</v-icon>
              </v-btn>
            </v-col>
          </v-row>

          <v-divider></v-divider>
          <v-row>
            <v-col>
              <v-list>
                <v-list-item
                    v-for="project in projectList"
                    :key="project.name"
                    :value="project"
                    color="primary"
                    :title="project.name"
                    :subtitle="project.version"
                    @click="selectProject(project)"
                >
                  <template v-slot:append="{ isActive }">
                    <v-icon @click.stop="deleteProject(project.name)" color="error">mdi-delete-forever</v-icon>
                  </template>
                </v-list-item>
              </v-list>


            </v-col>
          </v-row>
          <v-card-actions>
            <v-row class="mb-2 mt-2">
              <v-col class="d-flex justify-space-around">
                <v-btn prepend-icon="mdi-open-in-app" color="primary"
                       :disabled="!projectClicked"
                       @click="openProjectAsFlow()">
                  Open flow
                </v-btn>
                <v-btn prepend-icon="mdi-plus" color="success" @click="createProject()">add new project</v-btn>
                <v-btn prepend-icon="mdi-creation" color="success"
                       @click="createDummyProject()">
                  create Example project
                </v-btn>

              </v-col>
            </v-row>
          </v-card-actions>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<style scoped lang="scss">

</style>
