<script setup lang="ts">
import {useProjectsStore} from "~/store/projectStore";
import {ref} from "vue";
import {FlowProject} from "~/repository/modules/projects";

const projectsStore = useProjectsStore()

const selectedProject = ref(projectsStore.selectedProject);
const loading = computed(() => projectsStore.loading);
const projects = computed(() => projectsStore.projects);
const processes = computed(() => projectsStore.runningProcessesMap);
const lastCompiled = computed(() => projectsStore.getLastCompileTimeOfProject());
const buildType = ref(projectsStore.getBuildTypeArray());
const selectedBuildType = ref(projectsStore.selectedBuildType)
const runningProcesses = computed(() => projectsStore.getRunningFlowProjects());

//TODO: disable run button if process is started; prevent multiple processes to run for the same project

//TODO: add "status led" which indicates if one project is running / List of running projects?
//
watch(selectedProject, () => projectsStore.selectProject(selectedProject.value as FlowProject))

watch(selectedBuildType, () => projectsStore.selectBuildType(selectedBuildType.value))

const run = () => {
  //TODO: check that the run type is the same as the compile type
  if (selectedProject.value != null) {
    projectsStore.runProjectRequest(selectedProject.value.name, selectedBuildType.value)
  }
}

const stop = () => {
  projectsStore.stopProcessRequest()
}

const getStatus = () => {
  const valuesArray = [...processes.value.values()];
  const containsNumber = valuesArray.some(value => typeof value === 'number');
  if (containsNumber) {
    return "green"
  }
  return "red"
}

const compile = () => {
  if (selectedProject.value != null) {
    projectsStore.compileProjectRequest(selectedProject.value.name, selectedBuildType.value)
  }
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
      <h4 class="mt-2 ml-2">Choose an action:</h4>
      <v-card-actions>
        <v-col>
          <v-btn prepend-icon="mdi-code-braces" rounded="0" size="large" @click="compile()" class="mb-2 ml-2"
                 :loading="loading">
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
      <h4 v-if="lastCompiled" class="mt-2 ml-2">Last compiled: {{ lastCompiled }}</h4>

    </div>

  </v-card>
  <v-card title="Overall status" class="mt-3">
    <div class="flex-content">
      <v-icon class="mt-2 ml-2 mb-2" :color="getStatus()" icon="mdi-circle"></v-icon>
      <div class="mt-2 ml-2 mb-2" v-if="getStatus() === 'green'">Running flow:</div>
      <div class="mt-2 ml-2 mb-2" v-if="getStatus() !== 'green'">No running flow</div>
    </div>
    <div class="ml-5">
      <ul>
        <li v-for="projectName in runningProcesses" :key="projectName">
          {{ projectName }}
        </li>
      </ul>
    </div>
  </v-card>
</template>

<style scoped lang="scss">
.flex-content {
  display: flex;
  align-items: center
}
</style>
