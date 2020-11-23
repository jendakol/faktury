import Vue from "vue";
import Vuex from 'vuex'

Vue.use(Vuex)

const StateStore = new Vuex.Store({
    state: {
        loggedUserId: 0,
        entrepreneurId: 0,
        locale: "cs-CZ"
    },
    getters: {
        loggedUserId: function (state) {
            // console.log("Getting user id: " + state.loggedUserId)
            return state.loggedUserId
        },
        entrepreneurId: function (state) {
            // console.log("Getting entrepreneur id: " + state.entrepreneurId)
            return state.entrepreneurId
        }
    },
    mutations: {
        login(state, id) {
            console.log("New logged user id: " + id)
            state.loggedUserId = id
        },
        setEntrepreneur(state, id) {
            console.log("New entrepreneur id: " + id)
            state.entrepreneurId = id
        },
    }
})

export default StateStore
