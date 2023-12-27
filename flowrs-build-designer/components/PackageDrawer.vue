<script setup lang="ts">
import { useProjectsStore } from "~/store/projectStore";
import { type FlowProject } from "~/repository/modules/projects";
import { usePackagesStore } from "~/store/packageStore.js";
import { ContextCreator } from "~/rete/flowrs/contextCreator";
import {type Crate} from "~/repository/modules/packages";

const packagesStore = usePackagesStore()
const projectsStore = useProjectsStore();
const selectedProjectUnwrapped = computed(() => projectsStore.selectedProject);
const selectedProject: FlowProject | null = selectedProjectUnwrapped.value;
const packages = reactive({ values: ["flowrs", "primitives"] });
const inactivePackages: Crate[] = [];
await packagesStore.getAll()
packagesStore.currentActive = toRaw(packages).values



if (selectedProject) {
    selectedProject.packages.forEach(element => {
        if (element.name != "flowrs") {
            packages.values.push(element.name);

        }
    });
}
packagesStore.packages.forEach(element => {

    if (element.name != "flowrs" && element.name != "built-in") {
        console.log(element.name)
        if (!packages.values.includes(element.name)) {
            inactivePackages.push(element)
        }
    }
});

const drawer = reactive({ visible: false });

const updateSelected = async () => {
    //console.log(ContextMenuPlugin)
    //console.log(packages)
    packagesStore.currentActive = toRaw(packages).values
    deleteNodes(toRaw(packages).values);
    const contextMenu = await ContextCreator.updateContextMenu();
    packagesStore.Area.use(contextMenu);
    //console.log(packagesStore.packagesMap)
    //console.log(packagesStore.packages)
}

const deleteNodes = (activePackages: string[]) => {
    let toDelete = createToDelete(activePackages)
    let editor = packagesStore.NodeEditor
    console.log(editor)
    console.log(toDelete)
    /*toDelete.connections.forEach(element => {
        let res =editor.removeConnection(element)
        console.log(res)
    });
    console.log(editor)
    toDelete.nodes.forEach(element => {
        editor.removeNode(element)
    });*/
    console.log(editor)
}

const createToDelete = (activePackages: string[]) => {
    activePackages = activePackages.map(function (x) { return x.replace("-", "_"); });
    let result = { nodes: [], connections: [] };
    let editor = packagesStore.NodeEditor
    console.log(editor)
    editor.nodes.forEach(element => {
        let keyToSearch = (element.fullTypeName.substring(0, element.fullTypeName.indexOf("::")));
        if (!activePackages.includes(keyToSearch)) {
            result.nodes.push(element.id)
        }
    });
    editor.connections.forEach(element => {
        if (result.nodes.includes(element.target) || result.nodes.includes(element.source)) {
            result.connections.push(element.id)
        }
    });
    return result;
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
        <v-list-item v-if="selectedProject != null" v-for="packageE in inactivePackages" :key="packageE.name"
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

}
</style>
