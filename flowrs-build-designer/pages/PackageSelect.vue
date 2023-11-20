<script setup lang="ts">

import { usePackagesStore } from "~/store/packageStore";
import { Crate } from "~/repository/modules/packages";
import { KeyObject } from "crypto";


const packagesStore = usePackagesStore();
const selectedPackage: Crate = computed(() => packagesStore.selectedPackage);
const selectedMap: Map<String, Object> = computed(() => packagesStore.selectedMap);

</script>


<template>
  <v-row>
    <v-col class="mt-5 ml-5">
      <AppBar></AppBar>
    </v-col>
  </v-row>

  <v-row>
    <v-col class="text-center mt-5 ml-5">
      <PackageList :card-title="'Packages'"></PackageList>
    </v-col>



  </v-row>
  <v-row>
    <v-col class="text-center mt-5 mr-5">
      <v-card v-if="selectedPackage !== null" :title="selectedPackage ? selectedPackage.name : 'No package selected!'">
        <v-divider class="mb-5"></v-divider>
        <v-row v-if="selectedMap !== null">
          <v-col cols="12 d-flex" class="d-flex justify-center align-center">
            <v-chip class="ma-2" color="blue" label text-color="white">
              <v-icon start icon="mdi-alpha-t-box-outline"></v-icon>
              Type Parameter

            </v-chip>
            <v-chip class="ma-2" color="green" label text-color="white">
              <v-icon start icon="mdi-arrow-right-bold-outline"></v-icon>
              Input
            </v-chip>
            <v-chip class="ma-2" color="red" label text-color="white">
              Output
              <v-icon class="mx-1" start icon="mdi-arrow-right-bold-outline"></v-icon>
            </v-chip>
            
          </v-col>
        </v-row>
        <v-row v-if="selectedMap !== null">
          <v-col cols="4 d-flex" v-for="[key, value] in selectedMap">
            <PackageCard class="flex-grow-1" :name="key" :value="value"></PackageCard>
          </v-col>
        </v-row>
        <v-row>
        </v-row>
        <div class=" scroll">
          <pre class="language-json">
                                      <code></code>
                                    </pre>

        </div>
      </v-card>

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
