<script setup lang="ts">

import { usePackagesStore } from "~/store/packageStore.js";
import {Crate} from "~/repository/modules/packages";

const packagesStore = usePackagesStore()
packagesStore.getAll()

const projectClicked = ref(false)

const selectPackage = (packageE) => {
  const p : Crate = packageE
  console.log(packageE)
  console.log("Package was selected: " + p.name)
  packagesStore.selectPackage(packageE)
  console.log(packagesStore.selectedPackage.name)
  projectClicked.value = true;
}

const refreshPackageList = () => {
  console.log("Refreshing list of packages...")
  packagesStore.getAll()
}
defineProps({
  cardTitle: { type: String, default: "Packages" }
});
</script>

<template>
  <v-card :title="cardTitle" :subtitle="cardSubtitle" variant="elevated">
    <v-divider></v-divider>
    <v-list>
      <v-list-item v-for="packageE in packagesStore.packages" :key="packageE.name" :value="packageE" color="primary"
        :title="packageE.name" :subtitle="packageE.version" @click="selectPackage(packageE)"></v-list-item>
    </v-list>
    <v-card-actions>
      <v-row class="mb-2 mt-2">
        <v-col class="d-flex justify-space-around">
          <v-btn prepend-icon="mdi-open-in-app" color="blue" :disabled="!projectClicked" @click="openProjectAsFlow()">
            Show Details
          </v-btn>
          <v-btn prepend-icon="mdi-refresh" color="orange" @click="refreshPackageList()">Refresh list</v-btn>
        </v-col>
      </v-row>
    </v-card-actions>
  </v-card>
</template>

<style scoped lang="scss"></style>
