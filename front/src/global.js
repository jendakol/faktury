import axios from "axios";

let GlobalFunctions = {
    mounted: function () {
        this.$vuetify.theme.dark = true
    },
    data() {
        return {
            hostUrl: process.env.NODE_ENV === 'development' ? "http://localhost:8081" : window.location.origin,
        }
    },
    methods: {
        getUserId: function () {
            return this.$store.getters.loggedUserId;
        },
        getEntrepreneurId: function () {
            return this.$store.getters.entrepreneurId;
        },
        getLoggedSession: function () {
            if (this.$storage.has('login-session')) {
                try {
                    return JSON.parse(atob(this.$storage.get('login-session')))
                } catch (e) {
                    console.error(e);
                    return null;
                }
            }

            return null;
        },
        ajax(name, data, timeout) {
            try {
                return axios.post(this.hostUrl + "/" + name, data, {
                    timeout: timeout === undefined ? 5000 : timeout,
                    headers: {"X-Faktury-Auth": this.$storage.get('login-session')}
                })
                    .then(t => {
                        return t.data;
                    }).catch(err => {
                        return Promise.resolve({error: err})
                    })
            } catch (e) {
                return Promise.resolve({error: e})
            }
        }, asyncActionWithNotification(name, data, initialText, responseToPromise) {
            this.$snotify.async(initialText, () => new Promise((resolve, reject) => {
                this.ajax(name, data, 60000).then(resp => {
                    responseToPromise(resp)
                        .then(d => {
                            let text;
                            let timeout;

                            if (typeof d === 'object' && d !== null) {
                                text = d.text
                                timeout = d.timeout === undefined ? 3500 : d.timeout
                            } else {
                                text = d
                                timeout = 3500
                            }

                            resolve({
                                body: text,
                                config: {
                                    closeOnClick: true,
                                    timeout: timeout
                                }
                            })
                        }, errText => {
                            reject({
                                    body: errText,
                                    config: {
                                        closeOnClick: true,
                                        timeout: 3500
                                    }
                                }
                            )
                        })
                }).catch(err => {
                    console.log("ERROR:" + JSON.stringify(err.response.data));
                    responseToPromise(err.response.data)
                        .then(text => {
                            resolve({
                                body: text,
                                config: {
                                    closeOnClick: true,
                                    timeout: 3500
                                }
                            })
                        }, errText => reject({
                            body: errText,
                            config: {
                                // TODO HTML formatting
                                // html: '<div class="snotifyToast__title"><b>Html Bold Title</b></div><div class="snotifyToast__body"><i>Html</i> <b>toast</b> <u>content</u></div>',
                                closeOnClick: true,
                                timeout: 3500
                            }
                        }))
                })
            }));
        },
        formatVat: function (vatRaw) {
            switch (vatRaw) {
                case undefined :
                    return
                case "DontDisplay":
                    return ""
                case "NotTaxPayer":
                    return "Neplátce DPH" // TODO harcoded lang value
                default:
                    return vatRaw.Code
            }
        }
    },
}

export default GlobalFunctions
