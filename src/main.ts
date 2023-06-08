import { createApp } from "vue"
import { createPinia } from 'pinia'

import App from "./App.vue"
import router from "./router"

import "./style/reset.css"

import { closeSplashscreen } from './api/other'

document.addEventListener('DOMContentLoaded', () => {
  closeSplashscreen().then(res => {
  })
})

createApp(App).use(router)
  .use(createPinia())
  .mount("#app")
