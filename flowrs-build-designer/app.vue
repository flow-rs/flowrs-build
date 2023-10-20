<template>
  <h1>Packages list</h1>

    <div
        v-for="fpac in flowrPackages"
        class="card"
    >
      <div class="title">Package Name: {{ fpac.name }}</div>
      <div class="version">Package Version: {{ fpac.version }}</div>
      <div v-for="crate in fpac.crates">
        <div>Crates: {{ crate }}</div>
        <div>Crate Types: {{ crate.types }}</div>
        <div>Crate Modules: {{ crate.modules }}</div>

      </div>
    </div>
    <h1>Single Package</h1>
    <div>{{flowrPackage}}</div>
    <h1>Projects</h1>
    <div
        v-for="fpro in flowrProjects"
        class="card"
    >
      <div class="title">Project Name: {{ fpro.name }}</div>

  </div>
</template>

<script setup lang="ts">
import Rete from './components/Rete.vue'
import {CompileProjectData, FlowProject, ProjectIdentifier} from "~/repository/modules/projects";
import api from "~/plugins/api";

const { $api } = useNuxtApp();

//TODO: Build project : request not working on backend
// const buildProject = await $api.projects.buildProject("flow_project_81");

// TODO: GET file of project

// GET Flowr package by name
// const flowrPackage = await $api.packages.getFlowrsPackageByName("flowrs-std");

// GET Flowr packages
// const flowrPackages = await $api.packages.getFlowrsPackages();

//GET Flowr projects
// const flowrProjects = await $api.projects.getProjects();

// Example of creating a new FlowProject
const newFlowProject: FlowProject = {
  name: 'flow_project_81',
  version: '1.0.0',
  packages: [
    {
      name: 'flowrs',
      version: '1.0.0',
      path: '../../../flowrs',
    },
    {
      name: 'flowrs-std',
      version: '1.0.0',
      path: '../../../flowrs-std',
    },
  ],
  flow: {
    nodes: {
      // Define other nodes here
    },
    connections: [
      {
        from_node: 'timer_config_node',
        to_node: 'timer_node',
        to_input: 'config_input',
        from_output: 'output',
      },
      // Define other connections here
    ],
    data: {
        timer_config_node: null,
        timer_token_node: null,
    },
  },
};

// POST create flow project
// const createdflowProject = await $api.projects.createProject(newFlowProject);

const data : ProjectIdentifier = {
  project_name: "flow_project_81"
}

// POST compile project TODO: return type
// const status = await $api.projects.compileProject(data);

// POST run project TODO: return type
const run_status = await $api.projects.runProject(data);


// console.log(JSON.stringify(createdflowProject, null, 2))

</script>

<style scoped>
.page :deep(.min-h-screen) {
  min-height: auto;
}

.page :deep(.py-14) {
  padding-top: 1.5rem;
  padding-bottom: 1.5rem;
}

.page :deep(.grid) {
  display: none;
}

.page :deep(footer) {
  display: none;
}
</style>
