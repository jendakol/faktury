import Vue from "vue";
import Vuex from 'vuex'

Vue.use(Vuex)

const StateStore = new Vuex.Store({
    state: {
        // loggedUserId: 1, // TODO support login
        // entrepreneurId: 1,
        loggedUserId: 2, // TODO support login
        entrepreneurId: 3,
        locale: "cs-CZ"
    },
    getters: {
        loggedUserId: function(state) {
            console.log("Getting user id: " + state.loggedUserId)
            return state.loggedUserId
        },
        entrepreneurId: function(state) {
            console.log("Getting entrepreneur id: " + state.entrepreneurId)
            return state.entrepreneurId
        }
    },
    mutations: {
        login(state, id) {
            console.log("Logged user id: " + id)
            state.loggedUserId = id
        },
        setEntrepreneur(state, id) {
            console.log("New entrepreneur id: " + id)
            state.entrepreneurId = id
        },
    }
})

export default StateStore
