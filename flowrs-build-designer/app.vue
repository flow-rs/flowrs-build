<template>
  <h1>Packages list</h1>
  <div
      v-if="pending"
      class="spinner-wrapper"
  >
    <span class="loader"></span>
  </div>
  <div
      v-else
      class="product-wrapper"
  >

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
  </div>
</template>

<script setup lang="ts">
import Rete from './components/Rete.vue'
import {FlowProject} from "~/repository/modules/projects";
import api from "~/plugins/api";

const { $api } = useNuxtApp();

const flowrPackages = await $api.packages.getFlowrsPackages();

const flowrPackage = await $api.packages.getFlowrsPackageByName("flowrs-std");

const flowrProjects = await $api.projects.getProjects();

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

const createdflowProject = await $api.projects.createProject(newFlowProject);

const buildProject = await $api.projects.buildProject("flow_project_81");

console.log(JSON.stringify(createdflowProject, null, 2))

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
