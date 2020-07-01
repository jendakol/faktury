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
                        <v-icon>{{item.icon}}</v-icon>
                    </v-list-item-action>
                    <v-list-item-content>
                        <v-list-item-title>{{item.title}}</v-list-item-title>
                    </v-list-item-content>
                </v-list-item>
            </v-list>
        </v-navigation-drawer>

        <v-main>
            <v-container class="fill-height" fluid>
                <v-content>
                    <transition name="fade">
                        <router-view/>
                    </transition>
                </v-content>
            </v-container>
        </v-main>

    </v-app>
</template>

<script>
    export default {
        name: "MainApp",
        data() {
            return {
                drawerMenu: [
                    {
                        title: 'Dashboard', icon: 'mdi-view-dashboard', link: {name: 'Dashboard'}
                    },
                    {
                        title: 'Settings', icon: 'mdi-account-cog', link: {name: 'UserSettings'}
                    }
                ],
                accountMenu: [
                    {
                        title: 'Logout', icon: 'mdi-logout', action: () => {
                            this.logout()
                        }
                    },
                    {
                        title: 'Ping', icon: 'mdi-information-outline', action: () => {
                            this.ping()
                        }
                    },
                ]
            }
        },
        methods: {
            logout() {
                this.asyncActionWithNotification("logout", {}, "Logging out", (resp) => new Promise((success, error) => {
                    if (resp.success) {
                        this.$emit("logout");
                        success("Logged out")
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
