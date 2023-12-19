<template>
   <PackageDrawer></PackageDrawer>
  <v-alert
      v-model="showAlert"
      :type="errorMessage.length==0 ? 'info':'error'"
      :title="errorMessage.length==0 ? 'Saving...': 'Error on save'"
      :text="errorMessage"
      :closable="true"
      @click:close="() => {showAlert = false}"
  >
    <v-progress-linear v-if="isLoadingSave"/>
  </v-alert>
  <div class="rete" ref="rete"></div>

</template>

<script setup lang="ts">
import {useEventsStore} from "~/store/eventStore";

const userStore = useEventsStore()
const {isSaveButtonClicked} = storeToRefs(userStore)
</script>

<script lang="ts">
import {createEditor} from "~/rete";
import {useEventsStore} from "~/store/eventStore";
import {navigateTo} from "#app";
import {ContextCreator} from "~/rete/flowrs/contextCreator";

export default {
  mounted() { // TODO add a wait cycle
    createEditor(this.$refs.rete).then((res) => {
      console.log("Rete Editor loaded!");
      console.log(res)
    });
    const eventsStore = useEventsStore();

    eventsStore.$subscribe((mutation, state) => {
      if (state.isSaveButtonClicked) {
        this.handleSaveButtonClick();
      }
    })

  },
  data() {
    return {
      errorMessage: "",
      showAlert: false,
      isLoadingSave: false
    }
  },
  methods: {
    handleSaveButtonClick() {
      if (this.isLoadingSave) {
        return;
      }
      this.isLoadingSave = true;
      this.errorMessage = "";
      this.showAlert = true;
      const eventsStore = useEventsStore();
      eventsStore.setSaveButtonClicked(false);
      // Handle the save button click in the Rete component
      ContextCreator.saveBuilderStateAsProject().then(() => {
        this.isLoadingSave = false;
        this.showAlert = false;
        navigateTo("/");
      }).catch((e) => {
        console.error("Error caught", e);
        this.isLoadingSave = false;
        this.errorMessage = e.message;
        this.showAlert = true;
      });

    }
  }
};
</script>

<style scoped>
.rete {
  position: relative;
  width: 80vw;
  height: 90vh;
  font-size: 1rem;
  background: white;
  margin: 1em auto 3em auto;
  border-radius: 1em;
  text-align: left;
  border: 3px solid #55b881;
}

.v-alert {
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  width: auto; /* Adjust the width as needed */
  max-width: 95%;
  z-index: 9999; /* Ensure it's above other elements on the page */
}
</style>
