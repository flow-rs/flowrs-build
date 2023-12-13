<script setup lang="ts">
import { useProjectsStore } from "~/store/projectStore";
import { type FlowProject } from "~/repository/modules/projects";
import { usePackagesStore } from "~/store/packageStore.js";
import {createEditor} from "~/rete";
import {ContextMenuPlugin, Presets as ContextMenuPresets} from "rete-context-menu-plugin";

const packagesStore = usePackagesStore()
const projectsStore = useProjectsStore();
const selectedProjectUnwrapped = computed(() => projectsStore.selectedProject);
const selectedProject: FlowProject | null = selectedProjectUnwrapped.value;
const packages = reactive({ values: ["flowrs","primitives"] });
if (selectedProject) {
    selectedProject.packages.forEach(element => {
        if (element.name != "flowrs") {
            packages.values.push(element.name);

        }
    });
}

const drawer = reactive({ visible: false });

const updateSelected = () => {
    console.log(ContextMenuPlugin)
    console.log(packages)
    packagesStore.currentActive = toRaw(packages).values
    console.log(packagesStore.currentActive)
    console.log(packagesStore.ContextMenu)
}
</script>

<template>
    <v-navigation-drawer class="overflow-visible" absolute :width="237" location="right" temporary v-model="drawer.visible">
        <v-list-item title="Packages" subtitle="List of available Packages"></v-list-item>
        <v-divider></v-divider>
        <v-list-item v-if="selectedProject != null" v-for="packageE in selectedProject.packages" :key="packageE.name"
            :value="packageE" color="primary" :title="packageE.name" :subtitle="packageE.version">
            <v-switch :value="packageE.name" v-model="packages.values" @change="updateSelected()"
                v-if="packageE.name != 'flowrs' && packageE.name != 'built-in'" color="primary"
                class="d-flex justify-center" @click.stop label="Active" inset></v-switch></v-list-item>

        <v-btn size="large" class="overflow-visible" @click="drawer.visible = !drawer.visible"
            :style="{ top: '50%', transform: 'translate(-75%, -50%)' }">
            <v-icon v-if="!drawer.visible">mdi-chevron-left</v-icon>
            <v-icon v-else>mdi-chevron-right</v-icon>
            Packages
        </v-btn>
    </v-navigation-drawer>
</template>

<style>
.v-navigation-drawer--mini-variant,
.v-navigation-drawer__content,
.v-navigation-drawer {
    overflow: visible !important;

}</style>
