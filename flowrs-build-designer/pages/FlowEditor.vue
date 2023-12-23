<script setup lang="ts">

import {navigateTo} from "#app";
import JsonEditorVue from "~/components/JsonEditorVue.client.vue";
import {useProjectsStore} from "~/store/projectStore";
import {FlowProject} from "~/repository/modules/projects";
import {useEventsStore} from "~/store/eventStore";
import {FetchError} from "ofetch";

const projectsStore = useProjectsStore();
const eventStore = useEventsStore();
const selectedProject = computed(() => projectsStore.selectedProject);
let json = ref();


const eventsStore = useEventsStore();

eventsStore.$subscribe((mutation, state) => {
  if (state.isSaveButtonClicked) {
    handleSaveButtonClick();
  }
})
const handleSaveButtonClick = async () => {

  eventStore.setLoading(true)
  eventStore.setErrorMessage("")
  eventStore.setAlert(true)
  eventsStore.setSaveButtonClicked(false);
  saveProjectFromTextEditor().then(() => {
    eventStore.setLoading(false)
    eventStore.setAlert(false)
    navigateTo("/");
  }).catch((e) => {
    console.error("Error caught", e);
    eventStore.setLoading(false)
    eventStore.setErrorMessage(e.message)
    eventStore.setAlert(true)
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
      await useNuxtApp().$api.projects.createProject(flowProject);
    } catch (error) {
      if (error instanceof FetchError && error.data) {
        // if a fetch error is thrown an empty project dir is created in backend because the error is thrown afterwards
        console.log("Fetch error occured on save", error)
        await useNuxtApp().$api.projects.deleteProject({project_name: flowProject.name});
        throw new Error(error.data)
      } else {
        console.error("Error occurred on save", error);
        throw new Error("Save was unsuccessful 🐛 Please check your configuration 🔧");
      }
    }
    try {
      await useNuxtApp().$api.projects.deleteProject({project_name: flowProject.name});
    } catch (e) {
      console.log("Delete failed", e)
    }

    // delete the old & create the new setup, knowing that it will succeed
    flowProject.name = original_name
    try {
      await useNuxtApp().$api.projects.deleteProject({project_name: flowProject.name});
    } catch (e) {
      console.log("Delete failed", e)
    }
    console.log("New Project", flowProject)
    try {
      await useNuxtApp().$api.projects.createProject(flowProject);
    } catch (e) {
      console.error("Error occurred on save", e);
      throw new Error("Save was unsuccessful 🐛 Please check your configuration 🔧");
    }
  }

}



onMounted(() => {
  json.value = selectedProject.value
})


</script>

<template>
  <v-container fluid>
    <AlertComponent
    />
    <v-row>
      <v-col class="scroll">
        <client-only>
          <JsonEditorVue v-model="json" mode='text'/>
        </client-only>
      </v-col>
    </v-row>
  </v-container>
</template>

<style scoped lang="scss">

</style>
