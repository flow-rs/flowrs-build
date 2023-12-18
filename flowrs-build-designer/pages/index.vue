<script setup lang="ts">

import {useProjectsStore} from "~/store/projectStore";
import JsonEditorVue from 'json-editor-vue'

const projectsStore = useProjectsStore();
const selectedProject = computed(() => projectsStore.selectedProject);
const errorMessage = computed(() => projectsStore.errorMessage);
const loading = computed(() => projectsStore.loading);
let json = ref()
const saveDisabled = ref(true);

onMounted(() => {
  projectsStore.getAll()
  if (selectedProject.value != null) {
    json.value = selectedProject.value
  }
});

const saveProject = async () => {
  console.log(json.value)
  try {
    if (selectedProject) {
      await projectsStore.deleteProject()
    } else {
      throw new Error("No project selected!")
    }
  } catch (e) {
    console.log("Delete failed", e)
  }
  console.log("New Project", json)
  try {
    await projectsStore.createProject(json.value)
  } catch (e) {
    console.error("Error occurred on save", e);
    throw new Error("Save was unsuccessful 🐛 Please check your configuration 🔧");
  }
}

const handleProjectSelection = () => {
  json.value = selectedProject.value
  saveDisabled.value = false;
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
              <v-card-title>Editor:
                {{
                  selectedProject !== null ? selectedProject.name : 'No project selected!'
                }}
              </v-card-title>
              <v-card-subtitle>flow-project.json</v-card-subtitle>
            </v-col>
            <v-col class="d-flex align-center justify-end mr-4">
              <v-btn :disabled="saveDisabled" color="success" @click="saveProject()">Save</v-btn>
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
