<script setup lang="ts">
import { computed } from 'vue';
import {useEventsStore} from "~/store/eventStore";
const errorMessageAlert = computed(() => eventStore.errorMessageAlert);

const eventStore = useEventsStore();
const alertType = computed(() => errorMessageAlert.value.length === 0 ? 'info' : 'error');
const icon = computed(() => errorMessageAlert.value.length === 0 ? 'mdi-information' : 'mdi-alert')
const alertTitle = computed(() => errorMessageAlert.value.length === 0 ? 'Saving...' : 'Error on save');
const alertText = errorMessageAlert
const showAlert = computed(() => eventStore.showAlert);

const handleCloseButtonClick = () => {
  eventStore.setAlert(false)
}
</script>


<template>
  <div v-if="showAlert">
  <v-alert
      :type="alertType"
      :title="alertTitle"
      :text="alertText"
      :icon="icon"
      :closable="true"
      @click:close="handleCloseButtonClick"

  >
  </v-alert>
  </div>
</template>

<style>

.v-alert {
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  width: auto; /* Adjust the width as needed */
  max-width: 95%;
  z-index: 9999; /* Ensure it's above other elements on the page */
}
</style>


