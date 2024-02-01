import {createRouter, createWebHistory} from "vue-router";

export const routes = [{
    path: "/",
    title: "Dashboard",
    redirect: "/cloud",
}, {
    path: "/cloud",
    title: "Cloud",
    meta: {
        requiresAuth: true,
        layout: "landing",
    },
    component: () => import("@/view/Cloud.vue"),
}]

const router = createRouter({
    history: createWebHistory(),
    routes,
});
export default router;
