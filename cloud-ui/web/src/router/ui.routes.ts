export default [
    {
        path: "/cloud",
        name: "cloud",
        component: () =>
            import(
                /* webpackChunkName: "ui-playground" */ "@/views/cloud/Cloud.vue"
                ),
        meta: {
            requiresAuth: true,
            layout: "ui",
            category: "UI",
            title: "Cloud",
        },
    },
];
