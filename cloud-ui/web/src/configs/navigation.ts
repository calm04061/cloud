// import menuApps from "./menus/apps.menu";
import menuPages from "./menus/pages.menu";

export default {
  menu: [
    {
      text: "",
      key: "",
      items: [
        {
          key: "menu.cloud",
          text: "Cloud",
          link: "/cloud",
          icon: "mdi-view-dashboard-outline",
        },
      ],
    },
    {
      text: "",
      key: "",
      items: [
        {
          key: "menu.dashboard",
          text: "Dashboard",
          link: "/dashboard",
          icon: "mdi-view-dashboard-outline",
        },
      ],
    },

    {
      text: "Pages",
      key: "menu.pages",
      items: menuPages,
    },
  ],
};
