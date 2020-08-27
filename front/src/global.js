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
        ajax(name, data, timeout) {
            return axios.post(this.hostUrl + "/data-" + name, data, {timeout: timeout === undefined ? 5000 : timeout})
                .then(t => {
                    return t.data;
                }).catch(err => {
                    console.log(err);
                    this.$snotify.error("Error", err, {
                        closeOnClick: true,
                        timeout: 5000
                    })
                })
        }, asyncActionWithNotification(name, data, initialText, responseToPromise) {
            this.$snotify.async(initialText, () => new Promise((resolve, reject) => {
                this.ajax(name, data, 60000).then(resp => {
                    responseToPromise(resp)
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
                                closeOnClick: true,
                                timeout: 3500
                            }
                        }))
                }).catch(err => {
                    console.log(JSON.stringify(err.response.data));
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
                    console.log("VAT RAW: " + vatRaw)
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
