<script setup lang="ts">

import {useProjectsStore} from "~/store/projectStore";
import JsonEditorVue from "~/components/JsonEditorVue.client.vue";
import {FlowProject} from "~/repository/modules/projects";
import {FetchError} from "ofetch";

const projectsStore = useProjectsStore();
projectsStore.getAll()
const selectedProject = computed(() => projectsStore.selectedProject);
const errorMessage = computed(() => projectsStore.errorMessage);
let json = ref();
const showAlert = ref(false);
const isLoadingSave = ref(false);
const errorMessageAlert = ref("");

const saveDisabled = ref(true);

onMounted(() => {
  // if (selectedProject.value != null) {
  //   json.value = selectedProject.value
  // }
});

const handleSaveButtonClick = async () => {
  // if (isLoadingSave) {
  //   return;
  // }
  isLoadingSave.value = true;
  errorMessageAlert.value = "";
  showAlert.value = true;
  await saveProjectFromTextEditor().then(() => {
    isLoadingSave.value = false;
    showAlert.value = false;
  }).catch((e) => {
    console.error("Error caught", e);
    isLoadingSave.value = false;
    errorMessageAlert.value = e.message;
    showAlert.value = true;
  });

}

const saveProjectFromTextEditor = async () => {
  let fileModified = true;
  try {
    projectsStore.selectProject(JSON.parse(json.value), false)
  } catch (e) {
    fileModified = false;
    console.log("File not modified")
  }

  if (fileModified) {
    let flowProject: FlowProject = JSON.parse(JSON.stringify(selectedProject.value));

    let original_name = flowProject.name

    // try creating the new setup and clean it up on success
    flowProject.name = "tmp_" + flowProject.name
    try {
      await projectsStore.createProject(flowProject)
    } catch (error) {
      if (error instanceof FetchError && error.data) {
        await projectsStore.deleteProject(flowProject.name);
        throw new Error(error.data)
      } else {
        throw new Error("Save was unsuccessful 🐛 Please check your configuration 🔧");
      }
    }
    try {
      await projectsStore.deleteProject(flowProject.name);
    } catch (e) {
      console.log("Delete failed", e)
    }
    // delete the old & create the new setup, knowing that it will succeed
    flowProject.name = original_name
    try {
      await projectsStore.deleteProject(flowProject.name);
    } catch (e) {
      console.log("Delete failed", e)
    }
    console.log("New Project", flowProject)
    try {
      await projectsStore.createProject(flowProject);
    } catch (e) {
      console.error("Error occurred on save", e);
      throw new Error("Save was unsuccessful 🐛 Please check your configuration 🔧");
    }
  }

}


const handleProjectSelection = () => {
  json.value = selectedProject.value
  saveDisabled.value = false;
}


</script>


<template>
  <v-container fluid>
    <v-alert
        v-model="showAlert"
        :type="errorMessageAlert.length==0 ? 'info':'error'"
        :title="errorMessageAlert.length==0 ? 'Saving...': 'Error on save'"
        :text="errorMessageAlert"
        :closable="true"
        @click:close="() => {showAlert = false}"
    >
      <v-progress-linear v-if="isLoadingSave"/>
    </v-alert>
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
              <v-btn :disabled="saveDisabled" color="success" @click="handleSaveButtonClick()">Save</v-btn>
            </v-col>

          </v-row>

          <v-divider></v-divider>

          <v-col class="scroll">


            <client-only>
              <JsonEditorVue v-model="json" mode='text'/>
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
