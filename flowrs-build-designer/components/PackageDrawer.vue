<script setup lang="ts">
import { useProjectsStore } from "~/store/projectStore";
import { type FlowProject } from "~/repository/modules/projects";

const projectsStore = useProjectsStore();
const selectedProjectUnwrapped = computed(() => projectsStore.selectedProject);
const selectedProject: FlowProject | null = selectedProjectUnwrapped.value;

const packages = reactive({drawer:false});
</script>

<template>
    <v-navigation-drawer class="overflow-visible" absolute :width="237" location="right" temporary v-model="packages.drawer">
        <v-list-item title="Packages" subtitle="List of available Packages"></v-list-item>
        <v-divider></v-divider>
        <v-list-item v-if="selectedProject != null" v-for="packageE in selectedProject.packages" :key="packageE.name"
            :value="packageE" color="primary" :title="packageE.name" :subtitle="packageE.version">
            <v-switch :value="packageE.name" v-if="packageE.name != 'flowrs' && packageE.name != 'built-in'" color="primary"
                class="d-flex justify-center" @click.stop label="Active" inset></v-switch></v-list-item>

        <v-btn size="large" class="overflow-visible" @click="packages.drawer = !packages.drawer" :style="{ top: '50%', transform: 'translate(-75%, -50%)'}">
            <v-icon v-if="!packages.drawer">mdi-chevron-left</v-icon>
            <v-icon v-else>mdi-chevron-right</v-icon>
            Packages
        </v-btn>
    </v-navigation-drawer>
</template>

<style>
.v-navigation-drawer--mini-variant, .v-navigation-drawer__content, .v-navigation-drawer {
  overflow: visible !important;
  
}
</style>
