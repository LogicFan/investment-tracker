import { createApp } from "vue";
import App from "./App.vue";
import router from "./router.ts";
import vuestic from "./vuestic.ts";
import "vuestic-ui/css";
import "./style.css";

createApp(App)
    .use(router)
    .use(vuestic)
    .mount("#app");
