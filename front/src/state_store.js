import Vue from "vue";
import Vuex from 'vuex'

Vue.use(Vuex)

const StateStore = new Vuex.Store({
    state: {
        loggedUserId: 0,
        locale: "cs-CZ"
    },
    mutations: {
        login(state, id) {
            console.log("Logged user id: " + id)
            state.loggedUserId = id
        },
    }
})

export default StateStore
