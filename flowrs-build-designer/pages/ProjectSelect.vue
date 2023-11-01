<script setup lang="ts">

import {newFlowProject} from "~/repository/api_sample_data";

const createProject = () => {
  let projectToCreate = newFlowProject
  projectToCreate.name = "Name_" + Math.random()
  const {$api} = useNuxtApp();
  $api.projects.createProject(projectToCreate);
}

</script>


<template>
  <v-row>
    <v-col class="text-center mt-5 ml-5">
      <ProjectList :card-title="'Projects'" :card-subtitle="'Choose your project'"></ProjectList>

      <v-btn class="mt-5" @click="createProject()">Create project (Testing-purpose)</v-btn>


    </v-col>

    <v-col class="mt-5 ml-5">Das ist eine zweite Spalte</v-col>


  </v-row>

</template>
