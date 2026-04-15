import { createApp } from "vue";
import App from "./App.vue";
import "quasar/dist/quasar.css";
import "@quasar/extras/material-icons/material-icons.css";
import router from "./routers";
import { Quasar } from "quasar";
import { Notify } from "quasar";
import { Dialog } from "quasar";

const app = createApp(App);
app.use(router);

app.use(Quasar, {
  plugins: {
    Notify,
    Dialog,
  },
  config: {
    brand: {
      primary: "#061931",
      secondary: "#313454",
      accent: "#00B0FF",

      // 其他状态色可根据需要调整
      dark: "#1d1d1d",
      positive: "#21BA45",
      negative: "#C10015",
      info: "#31CCEC",
      warning: "#F2C037",
    },
    notify: {
      /* look at QuasarConfOptions from the API card */
    },
  },
});

app.mount("#app");
