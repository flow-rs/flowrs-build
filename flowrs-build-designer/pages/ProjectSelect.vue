<template>
  <v-row>
    <v-col class="mt-5 ml-5">
      <AppBar></AppBar>
    </v-col>
  </v-row>
  <v-row>
    <v-col class="text-center mt-5 ml-5">
      <v-card class="mx-auto" max-width="300">
        <v-card-title> Projekte verwalten</v-card-title>

        <v-divider></v-divider>
        <v-btn class="ma-2" color="primary">
          Projekt erstellen
          <v-icon end icon="mdi-wrench"></v-icon>
        </v-btn>
        <v-virtual-scroll :items="items" height="420" item-height="48">
          <template v-slot:default="{ item }">
            <v-list-item>
              <template v-slot:prepend>
                <v-btn to="/" nuxt :text="`Project #${item}`"></v-btn>
              </template>
              <template v-slot:append>
                <v-btn icon="mdi-delete" size="x-small" variant="tonal"></v-btn>
              </template>
            </v-list-item>
          </template>
        </v-virtual-scroll>
      </v-card>
    </v-col>
  </v-row>
</template>
<script>
export default {
  data: () => ({
    items: Array.from({ length: 10 }, (k, v) => v + 1),
  }),
};
</script>
