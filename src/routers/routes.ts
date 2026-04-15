
const routes = [
  {
    path: "/",
    redirect: "/booklist",

    children: [
      {
        path: "booklist",
        component: () => import("../pages/BookCollection.vue"),
      },
      { path: "character", component: () => import("../pages/Character.vue") },
      { path: "list", component: () => import("../pages/GenerateList.vue") },

      {
        name: "bookDetail",
        path: "bookDetail/:bookId",
        component: () => import("../pages/BookDetails.vue"),
      },
    ],
  },

  // {
  //   path: "/readbook",
  // },
];

export default routes;
