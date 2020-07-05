import Vue from 'vue'
import App from './App.vue'
import vuetify from './plugins/vuetify'

Vue.config.productionTip = false

import '../sass/style.scss';

import VuetifyConfirm from 'vuetify-confirm'
Vue.use(VuetifyConfirm, {vuetify})

import Snotify, { SnotifyPosition } from 'vue-snotify'
Vue.use(Snotify, {
  toast: {
    timeout: 3500,
    position: SnotifyPosition.rightTop,
    showProgressBar: false,
    pauseOnHover: true,
    closeOnClick: true
  }
})

import Datetime from 'vue-datetime'
Vue.use(Datetime);

import VueLodash from 'vue-lodash'
import lodash from 'lodash'
Vue.use(VueLodash, { lodash })

import 'vue-datetime/dist/vue-datetime.css'
import 'vuetify/dist/vuetify.min.css'

import VueRouter from 'vue-router'
Vue.use(VueRouter)

import routes from './routes'

const router = new VueRouter({
  mode: 'history',
  routes
})

import Vuex from 'vuex'
Vue.use(Vuex)

import GlobalFunctions from "./global"
Vue.mixin(GlobalFunctions)

import StateStore from "./state_store";

import VCurrencyField from 'v-currency-field'
Vue.use(VCurrencyField, {
  locale: StateStore.state.locale,
  decimalLength: 1,
  autoDecimalMode: false,
  min: 0,
  max: null,
  defaultValue: 0,
  valueAsInteger: false,
  allowNegative: false,
})

new Vue({
  vuetify,
  router,
  store: StateStore,
  render: h => h(App)
}).$mount('#app')

