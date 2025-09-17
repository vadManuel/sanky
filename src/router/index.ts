import { createRouter, createWebHashHistory } from "vue-router";
import GrpcClient from "../views/GrpcClient.vue";

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: "/",
      name: "home",
      component: GrpcClient,
    },
  ],
});

export default router;
