<template>
    <v-app>
        <v-navigation-drawer
            app
            clipped
            mini-variant
            expand-on-hover>
            <v-list dense>
                <v-list-item link v-for="item in drawerMenu"
                             :to="item.link"
                             :key="item.title">
                    <v-list-item-action>
                        <v-icon>{{ item.icon }}</v-icon>
                    </v-list-item-action>
                    <v-list-item-content>
                        <v-list-item-title>{{ item.title }}</v-list-item-title>
                    </v-list-item-content>
                </v-list-item>
                <v-spacer/>
                <v-list-item link @click="logout">
                    <v-list-item-action>
                        <v-icon>mdi-account-arrow-right</v-icon>
                    </v-list-item-action>
                    <v-list-item-content>
                        <v-list-item-title>Log out</v-list-item-title>
                    </v-list-item-content>
                </v-list-item>
            </v-list>
        </v-navigation-drawer>

        <v-main>
            <v-container fluid>
                <v-row>
                    <v-col cols="12">
                        <v-row justify="center">
                            <transition name="fade">
                                <router-view/>
                            </transition>
                        </v-row>
                    </v-col>
                </v-row>
            </v-container>
        </v-main>

    </v-app>
</template>

<script>
// TODO display offline message when backend is not available

export default {
    name: "MainApp",
    data() {
        return {
            drawerMenu: [
                {
                    title: 'Dashboard', icon: 'mdi-view-dashboard', link: {name: 'Dashboard'}
                },
                {
                    title: 'Invoices', icon: 'mdi-text-box-multiple', link: {name: 'Invoices'}
                },
                {
                    title: 'Contacts', icon: 'mdi-account-multiple', link: {name: 'Contacts'}
                },
                {
                    title: 'Settings', icon: 'mdi-account-cog', link: {name: 'UserSettings'}
                }
            ]
        }
    },
    methods: {
        logout() {
            this.asyncActionWithNotification("account-logout", {}, "Logging out", (resp) => new Promise((success, error) => {
                if (resp.success) {
                    success("Logged out successfully!")
                    console.log("Logged out")

                    this.$storage.remove('login-session')
                    this.$storage.remove('entrepreneur')

                    this.$store.commit('login', 0)
                    this.$store.commit('setEntrepreneur', 0)

                    this.$emit("logout");
                } else {
                    error(resp.message)
                }
            }));
        },
    },
}
</script>

<style scoped lang="scss">
.content {
    padding: 10px;
}
</style>
