<script setup lang="ts">

import {useProjectsStore} from "~/store/projectStore";
import JsonEditorVue from "~/components/JsonEditorVue.client.vue";
import {FlowProject} from "~/repository/modules/projects";
import {FetchError} from "ofetch";
import {useEventsStore} from "~/store/eventStore";
import {navigateTo} from "#app";
import ProjectList from "~/components/ProjectList.vue";

const projectsStore = useProjectsStore();

projectsStore.getAll()
const selectedProject = computed(() => projectsStore.selectedProject);
const errorMessage = computed(() => projectsStore.errorMessage);
const editDisabled = ref(true);
let json = ref()

const handleEditButtonClick = async () => {
  navigateTo("/flowEditor");
}

const handleProjectSelection = () => {
  json.value = selectedProject.value
  editDisabled.value = false;
}


</script>


<template>
  <v-container fluid>
    <v-row>
      <v-col class="text-center">
        <ProjectList :card-title="
      'Projects'" :card-subtitle="'Choose your project'" @project-selected="handleProjectSelection"></ProjectList>
      </v-col>

      <v-col>
        <div v-if="errorMessage.length != 0">
          <v-alert
              :type="errorMessage.length==0 ? 'info':'error'"
              :title="errorMessage.length==0 ? 'Saving...': 'Error on save'"
              :text="errorMessage"
              icon="mdi-alert"
              :closable="true"
          >
          </v-alert>

        </div>
        <v-card>
          <v-row>
            <v-col>
              <v-card-title>Editor:
                {{
                  selectedProject !== null ? selectedProject.name : 'No project selected!'
                }}
              </v-card-title>
              <v-card-subtitle>flow-project.json</v-card-subtitle>
            </v-col>
            <v-col class="d-flex align-center justify-end mr-4">
              <v-btn prepend-icon="mdi-pencil" :disabled="editDisabled" color="primary" @click="handleEditButtonClick()">Edit</v-btn>
            </v-col>

          </v-row>

          <v-divider></v-divider>

          <v-col>
            <client-only>
              <JsonEditorVue v-model="json" readOnly mode="text"/>
            </client-only>
          </v-col>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<style scoped>

.scroll {
  height: 650px;
  overflow-x: hidden;
  overflow-y: auto;
  padding: 20px;
}

.v-alert {
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  width: auto; /* Adjust the width as needed */
  max-width: 95%;
  z-index: 9999; /* Ensure it's above other elements on the page */
}
</style>
