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
      { path: "tts_generate", component: () => import("../pages/TtsGenerate.vue") },
      { path: "audio_library", component: () => import("../pages/AudioLibrary.vue") },
      {
        name: "audioPlayer",
        path: "audio_player/:chapterId",
        component: () => import("../pages/AudioPlayer.vue"),
      },
      {
        name: "bookDetail",
        path: "bookDetail/:bookId",
        component: () => import("../pages/BookDetails.vue"),
      },
      {
        name: "reader",
        path: "reader/:bookId/:chapterIndex",
        component: () => import("../pages/Reader.vue"),
      },
    ],
  },
];

export default routes;
