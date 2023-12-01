<script setup lang="ts">
import {useProjectsStore} from "~/store/projectStore";
import {ref, watch} from "vue";
import {type FlowProject} from "~/repository/modules/projects";

const projectsStore = useProjectsStore()
//TODO: new tab or new page?, store log messages and status of the running processes across page switch
const selectedProject = ref(projectsStore.selectedProject);
const loading = computed(() => projectsStore.loading);
const projects = computed(() => projectsStore.projects);
const buildType = ref(projectsStore.getBuildTypeArray());
const selectedBuildType = ref(projectsStore.selectedBuildType)

//TODO: disable run button if process is started; prevent multiple processes to run for the same project

//TODO: add "status led" which indicates if one project is running / List of running projects?

watch(selectedProject, () => projectsStore.selectProject(selectedProject.value as FlowProject))

watch(selectedBuildType, () => projectsStore.selectBuildType(selectedBuildType.value))

const run = () => {
  //TODO: check that the run type is the same as the compile type
  projectsStore.runProjectRequest(selectedProject.value.name, selectedBuildType.value)
}

const stop = () => {
  projectsStore.stopProcessRequest()
}

const compile = () => {
  projectsStore.compileProjectRequest(selectedProject.value.name, selectedBuildType.value)
}

</script>

<template>
  <v-card title="Control Panel">
    <v-select
        v-if="projects.length > 0"
        v-model="selectedProject"
        :items="projects"
        item-title="name"
        label="Select a project"
        return-object
    ></v-select>
    <span v-else>No projects available</span>
    <div v-show="selectedProject !== null">
      <v-select
          v-if="projects.length > 0"
          v-model="selectedBuildType"
          :items="buildType"
          label="Select a build type"
      ></v-select>
    </div>
    <v-divider></v-divider>

    <div v-show="selectedProject !== null">
      <div class="mt-2 ml-2">Choose an action:</div>
    <v-card-actions>
      <v-col>
        <v-btn prepend-icon="mdi-code-braces" rounded="0" size="large" @click="compile()" class="mb-2 ml-2" :loading="loading">
          <template v-slot:loader>
            <v-progress-linear indeterminate color="primary" rounded height="25"> Compiling</v-progress-linear>
          </template>
          Compile project
        </v-btn>

        <v-btn color="success" prepend-icon="mdi-play" rounded="0" size="large" @click="run()" class="mb-2">
          Run project
        </v-btn>

        <v-btn color="error" prepend-icon="mdi-stop" rounded="0" size="large" @click="stop()">
          Stop execution
        </v-btn>

      </v-col>
    </v-card-actions>
    </div>

  </v-card>
</template>

<style scoped lang="scss">

</style>
