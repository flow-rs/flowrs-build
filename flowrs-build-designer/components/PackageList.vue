<script setup lang="ts">

import {usePackagesStore} from "~/store/packageStore";
import type {Package} from "~/repository/modules/packages";

const packagesStore = usePackagesStore()
const selectPackage = (p: Package) => {
  console.log("Project was selected: " + p.name)
  packagesStore.selectPackage(p)
  packagesStore.getByName(p.name)
}

const refreshPackageList = () => {
  console.log("Refreshing list of packages...")
  packagesStore.getAll()
}
const {cardTitle: string} = defineProps(['cardTitle']);
</script>

<template>
  <v-card :title="cardTitle" subtitle="Test" variant="elevated">
    <v-divider></v-divider>
    <v-list>
      <v-list-item v-for="crate in packagesStore.packages" :key="crate.name" :value="crate" color="primary"
                   :title="crate.name" :subtitle="crate.version" @click="selectPackage(crate)"></v-list-item>
    </v-list>
    <v-card-actions>
      <v-row class="mb-2 mt-2">
        <v-col class="d-flex justify-space-around">
          <v-btn prepend-icon="mdi-refresh" color="orange" @click="refreshPackageList()">Refresh list</v-btn>
        </v-col>
      </v-row>
    </v-card-actions>
  </v-card>
</template>

<style scoped lang="scss"></style>
