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
import {FlowProject, ProjectIdentifier, TimerConfigNode, TimerTokenNode} from "~/repository/modules/projects";
import api from "~/plugins/api";
import {newFlowProject} from "~/repository/api_sample_data";

const { $api } = useNuxtApp();

//TODO: Build project : request not working on backend
// const buildProject = await $api.projects.buildProject("flow_project_81");

// TODO: GET file of project --> necessary? / not working on backend site

// GET Flowr package by name
// const flowrPackage = await $api.packages.getFlowrsPackageByName("flowrs-std");
//
// // GET Flowr packages
// const flowrPackages = await $api.packages.getFlowrsPackages();

//GET Flowr projects
// const flowrProjects = await $api.projects.getProjects();

// POST create flow project
// const createdflowProject = await $api.projects.createProject(newFlowProject);

const data : ProjectIdentifier = {
  project_name: "flow_project_100"
}

// POST compile project
// const status = await $api.projects.compileProject(data);

// POST run project
const process_identifier = await $api.projects.runProject(data);

// POST stop project
const stopped = await $api.projects.stopProject(process_identifier)


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
