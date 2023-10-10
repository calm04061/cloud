<script setup lang="ts">
import {ref} from 'vue'
import router, {routes} from "./route";
import {Route} from "./inter/type.ts";

const appTitle = ref("Cloud");
const drawer = ref(true);
const itemClick = (route: Route) => {
  appTitle.value = route.title;
  router.push(route.path)

}
</script>
<template>
  <v-app>
    <v-theme-provider theme="high-contrast">

      <v-navigation-drawer location="left" v-model="drawer">
        <v-list>
          <v-list-item prepend-avatar="https://randomuser.me/api/portraits/women/85.jpg"
                       title="Sandra Adams" subtitle="sandra_a88@gmailcom"></v-list-item>
        </v-list>

        <v-divider></v-divider>

        <v-list density="compact">
          <v-list-item v-for="r in routes" prepend-icon="mdi-folder" value="myfiles"
                       @click="itemClick(r)">{{ r.title }}
          </v-list-item>
        </v-list>
        <!--      <template v-slot:append>-->
        <!--        <v-divider></v-divider>-->
        <!--        <v-list >-->
        <!--          <v-list-item prepend-icon="mdi-star" title="Starred" value="starred"></v-list-item>-->
        <!--        </v-list>-->
        <!--      </template>-->
        <!--  -->
      </v-navigation-drawer>
      <v-app-bar>
        <v-app-bar-nav-icon variant="text" @click.stop="drawer = !drawer"></v-app-bar-nav-icon>
        <v-app-bar-title>{{ appTitle }}</v-app-bar-title>

      </v-app-bar>
      <v-main>
        <router-view></router-view>
      </v-main>
    </v-theme-provider>
  </v-app>
</template>

<style scoped>
</style>
