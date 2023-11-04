<script setup lang="ts">

import {newFlowProject} from "~/repository/api_sample_data";
import {FlowProject} from "~/repository/modules/projects";






</script>


<template>
  <v-row>
    <v-col class="text-center mt-5 ml-5">
      <PackageList :card-title="'Package'" :card-subtitle="'Choose a package'"></PackageList>
    </v-col>



  </v-row>

</template>

<style scoped>

div.scroll {
  height: 650px;
  overflow-x: hidden;
  overflow-y: auto;
  padding: 20px;
}
</style>
