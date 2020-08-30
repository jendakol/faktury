<template>
    <v-app>
        <LoginForm v-if="clientStatus === 'LOGGED-OUT'" v-on:login="refreshLoginState"/>
        <MainApp v-else-if="clientStatus === 'READY'" v-on:logout="refreshLoginState"/>

        <vue-snotify/>
    </v-app>

</template>

<script>
import LoginForm from './components/LoginForm.vue';
import MainApp from './components/MainApp.vue';

export default {
    components: {
        MainApp,
        LoginForm
    },
    mounted() {
        console.log('Loading saved login')

        let savedLogin = this.getLoggedSession();
        let savedEntrepreneur = this.$storage.get('entrepreneur');

        if (savedLogin != null) console.log("Saved login: " + JSON.stringify(savedLogin))
        if (savedEntrepreneur != null) console.log("Saved entrepreneur: " + JSON.stringify(savedEntrepreneur))

        if (savedLogin != null) {
            this.$store.commit('login', savedLogin.accountId)
            this.$store.commit('setEntrepreneur', savedEntrepreneur)
        }

        this.refreshLoginState()

        // TODO check validity of login
    },
    data() {
        return {
            clientStatus: undefined
        }
    },
    methods: {
        refreshLoginState: function () {
            let userId = this.$store.state.loggedUserId;
            if (userId > 0) console.log('User id: ' + userId)
            this.clientStatus = userId > 0 ? "READY" : "LOGGED-OUT"
        },
    },
}
</script>
