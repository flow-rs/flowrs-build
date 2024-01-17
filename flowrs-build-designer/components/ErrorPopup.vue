<script setup lang="ts">
import {useProjectsStore} from "~/store/projectStore.js";

const projectsStore = useProjectsStore()
const showDialog = computed(() => projectsStore.showDialog);

const { errorMessage } = defineProps(['errorMessage']);

/**
 * Called if the user closes the dialog.
 */
const closePopup = () => {
    projectsStore.setCurrentErrorMessage("");
    projectsStore.setDialog(false)
}

</script>

<template>

  <v-dialog
      v-model="showDialog"
      width="auto"
  >
    <v-card>
      <v-card-text>
        {{errorMessage}}
      </v-card-text>
      <v-card-actions>
        <v-btn color="primary" @click="closePopup">Close Dialog</v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<style scoped>
</style>
