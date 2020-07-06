<template>
    <v-app>
        <LoginForm v-if="clientStatus === 'LOGGED-OUT'" :ajax="this.ajax"
                   :asyncActionWithNotification="this.asyncActionWithNotification"/>

        <MainApp v-else-if="clientStatus === 'READY'" :hostUrl="this.hostUrl" :ajax="this.ajax"
                 :asyncActionWithNotification="this.asyncActionWithNotification"/>

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
        },
        data() {
            return {
                clientStatus: this.$store.state.loggedUserId >= 0 ? "READY" : "LOGGED-OUT",
            }
        },
        methods: {},
    }
</script>
