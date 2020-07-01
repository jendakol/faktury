<template>
    <v-app>
        <v-content>
            <v-container fluid fill-height>
                <v-layout align-center justify-center>
                    <v-flex xs12 sm8 md4>
                            <v-card class="elevation-12">
                                <v-card-text>
                                    <v-form @submit="login">
                                        <v-text-field prepend-icon="mdi-account" name="login" type="text"
                                                      v-model="loginForm.username"></v-text-field>
                                        <v-text-field prepend-icon="mdi-lock" name="password" type="password"
                                                      v-model="loginForm.password"></v-text-field>
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
        </v-content>
    </v-app>
</template>

<script>
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

                this.asyncActionWithNotification("login", {
                        username: this.loginForm.username,
                        password: this.loginForm.password,
                    }, "Logging in", (resp) => new Promise((success, error) => {
                        this.formDisabled = false;

                        if (resp.success === true) {
                            success("Login successful!");
                            this.$emit("login")
                        } else if (resp.success === false) {
                            error("Login unsuccessful")
                        } else {
                            error("Login unsuccessful: " + resp.error)
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
