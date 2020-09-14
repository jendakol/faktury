<template>
    <v-card width="600" outlined raised :loading="loading">
        <v-card-title>
            <v-text-field class="faktury-page-header contact-name-text" prefix="Contact" solo v-model="contactData.name" counter="250"/>
        </v-card-title>

        <v-card-text>
            <v-row>
                <v-col>
                    <v-text-field label="Code" v-model="contactData.code" counter="100"/>
                </v-col>
            </v-row>
            <v-row>
                <v-col>
                    <v-select
                        :items="vatSelectItems"
                        v-model="contactData.vat.type"
                        label="VAT type"
                        outlined
                    ></v-select>

                    <v-text-field v-if="contactData.vat.type === 'Code'" label="VAT code" v-model="contactData.vat.value" counter="100"/>
                </v-col>
            </v-row>
            <v-row>
                <v-col>
                    <v-textarea label="Address" v-model="contactData.address" counter="250"/>
                </v-col>
            </v-row>
        </v-card-text>
        <v-card-actions>
            <v-spacer/>
            <v-btn color="green darken-1" text @click="save">Save</v-btn>
        </v-card-actions>
    </v-card>
</template>

<script>
// TODO support enter-confirmation

export default {
    name: 'ContactDetail',
    components: {},
    mounted() {
        this.ajax("data-get/contact/" + this.$route.params.id, {}).then(r => {
            this.contactData = r

            switch (this.contactData.vat) {
                case "DontDisplay":
                    this.contactData.vat = {type: "DontDisplay"}
                    break
                case "NotTaxPayer":
                    this.contactData.vat = {type: "NotTaxPayer"}
                    break
                default:
                    this.contactData.vat = {type: "Code", value: this.contactData.vat.Code}
                    break
            }

            this.loading = false
        })
    },
    data() {
        return {
            loading: true,
            contactData: {},
            vatSelectItems: [
                {text: "Doesn't have", value: "DontDisplay"},
                {text: "Not a tax payer", value: "NotTaxPayer"},
                {text: "Code", value: "Code"},
            ]
        }
    },
    methods: {
        save: function () {
            this.contactData.address = this.contactData.address.replace("\n", "\r\n").replace("\r\r\n", "\r\n")

            // TODO check for invalid values!

            let data = Object.assign({}, this.contactData)

            switch (data.vat.type) {
                case "Code":
                    data.vat = {Code: data.vat.value}
                    break
                default:
                    data.vat = data.vat.type
            }

            data.code = data.code !== "" ? data.code : null // empty to null

            console.log("Saving contact: ")
            console.log(data)

            this.asyncActionWithNotification("data-update/contact", data, "Saving", (resp) => new Promise((success, error) => {
                    if (resp.success) {
                        success("Contact saved")
                    } else {
                        error("Could not save contact")
                    }
                })
            );
        }
    }
}
</script>
