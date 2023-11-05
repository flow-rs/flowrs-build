<script setup lang="ts">

import { usePackagesStore } from "~/store/packageStore";
import { Crate } from "~/repository/modules/packages";


const packagesStore = usePackagesStore();
const selectedPackage: Crate = computed(() => packagesStore.selectedPackage);



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
    <v-col class=" text-center mt-5 mr-5">
      <v-card v-if="selectedPackage !== null" :title="selectedPackage ? selectedPackage.name : 'No package selected!'"
        subtitle="flow-project.json">
        <v-divider></v-divider>
        <v-expansion-panels variant="popout" class="my-4">
          <v-expansion-panel v-for="(value, key) in selectedPackage.crates" :key="key" :title="key">
            <v-expansion-panel-text>
              <v-col v-for="(value2, key2) in value.modules.nodes.modules" class=" text-center mt-5 mr-5">
                {{ key2 }}
              </v-col>
            </v-expansion-panel-text>
          </v-expansion-panel>
        </v-expansion-panels>
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
