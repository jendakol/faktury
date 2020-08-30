<template>
    <v-app>
        <v-main>
            <v-container fluid fill-height>
                <v-layout align-center justify-center>
                    <v-flex xs12 sm8 md4>
                        <v-card class="elevation-12">
                            <v-card-text>
                                <v-form @submit="login">
                                    <v-text-field placeholder="Username" prepend-icon="mdi-account" name="login"
                                                  type="text" v-model="loginForm.username"/>
                                    <v-text-field placeholder="Password" prepend-icon="mdi-lock" name="password"
                                                  type="password" v-model="loginForm.password"/>
                                    <v-card-actions>
                                        <v-spacer></v-spacer>
                                        <v-btn color="primary" type="submit" :disabled="this.formDisabled">Login</v-btn>
                                    </v-card-actions>
                                </v-form>
                            </v-card-text>

                        </v-card>
                    </v-flex>
                </v-layout>
            </v-container>
        </v-main>
    </v-app>
</template>

<script>
let sha256 = require("sha256")

export default {
    name: 'LoginForm',
    data() {
        return {
            loginForm: {
                username: null,
                password: null,
            },
            formDisabled: false,
            selectedTab: 0
        }
    },
    methods: {
        login(evt) {
            evt.preventDefault();

            if (this.loginForm.username == null || this.loginForm.username.trim() === "") {
                this.$snotify.warning("You must enter username");
                return
            }
            if (this.loginForm.password == null || this.loginForm.password.trim() === "") {
                this.$snotify.warning("You must enter password");
                return
            }

            this.formDisabled = true;

            this.asyncActionWithNotification("account-login", {
                    username: this.loginForm.username,
                    password: sha256(this.loginForm.password),
                }, "Logging in", (resp) => new Promise((success, error) => {
                    this.formDisabled = false

                    if (resp.error != null) {
                        console.log(resp.error)

                        if (resp.error.response.status === 401) {
                            error("Login unsuccessful!")
                        } else {
                            error("Login has failed due to uknown error!")
                        }
                    } else if (resp.encodedValue !== "") {
                        success("Login successful!")

                        let session = JSON.parse(atob(resp.encodedValue))
                        console.log("Logged in as " + this.loginForm.username + ": " + JSON.stringify(session))

                        this.$storage.set('login-session', resp.encodedValue, {ttl: resp.ttl})
                        this.$storage.set('entrepreneur', 3) // TODO hardcoded
                        this.$store.commit('login', session.accountId)
                        this.$store.commit('setEntrepreneur', 3)
                        this.$emit("login")
                    } else {
                        error("Login unsuccessful!")
                    }
                })
            );

        },
    }
}
</script>

<style scoped lang="scss">

div#loginFormTabs {
    border: 1px solid black;
    border-radius: 5px;
    display: block;
    padding: 5px;
    width: 500px;
    height: 200px;
    position: absolute;
    left: 50%;
    top: 50%;
    margin: -100px 0 0 -250px;
}

.formInput {
    margin: 10px 0 0 0;
}

.formButton {
    margin: 10px 0 0 -75px;
    display: block;
    width: 150px;
    position: relative;
    left: 50%;
}
</style>
