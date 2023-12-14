<script setup lang="ts">

import {useProjectsStore} from "~/store/projectStore";
import JsonEditorVue from 'json-editor-vue'

const projectsStore = useProjectsStore();
projectsStore.getAll()
const selectedProject = computed(() => projectsStore.selectedProject);
const errorMessage = computed(() => projectsStore.errorMessage);
const loading = computed(() => projectsStore.loading);
let json = ref()

onMounted(() => {
  if (selectedProject.value != null) {
    json.value = selectedProject.value
  }
});

const saveProject = () => {
  //TODO:
  // projectsStore.saveProject(json)
  console.log(json.value)
}

const handleProjectSelection = () => {
  json.value = selectedProject.value
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
          <ErrorPopup :error-message="errorMessage"></ErrorPopup>
        </div>
        <v-card>
          <v-row>
            <v-col>
              <v-card-title>
                {{
                  selectedProject !== null ? selectedProject.name : 'No project selected!'
                }}
              </v-card-title>
              <v-card-subtitle>flow-project.json</v-card-subtitle>
            </v-col>
            <v-col class="align-content-center">
              <v-btn @click="saveProject()">Klick hier</v-btn>

            </v-col>

          </v-row>

          <v-divider></v-divider>

          <client-only>
            <JsonEditorVue class="scroll" v-model="json" mode='text'/>
          </client-only>

        </v-card>

      </v-col>
    </v-row>
    <v-row>

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
</style>
