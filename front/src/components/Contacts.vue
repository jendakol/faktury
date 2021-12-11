<template>
    <v-card width="1000" outlined raised :loading="loading" class="ma-0 pa-2 pt-1">
        <v-card-title>
            <h1 class="faktury-page-header">Your contacts</h1>
        </v-card-title>

        <v-text-field
            v-model="search"
            clearable
            flat
            solo
            hide-details
            prepend-inner-icon="mdi-account-search"
            label="Search"
        />

        <v-card v-for="contact in filteredContacts" :key="contact.id"
                :to="{ name: 'ContactDetail', params: { id: contact.id } }"
                class="ma-0 mt-1 pa-0"
                no-gutters
                hover
                link
                outlined
                rounded>
            <v-card-text>
                <v-row class="contact-row d-flex pa-1">
                    <v-col cols="3" class="align-center ma-0 pa-0 pt-1 col-name">{{ contact.name }}</v-col>
                    <v-col class="align-center ma-0 pa-0 pt-1 col-addr">{{ contact.address }}</v-col>
                    <v-col class="align-center ma-0 pa-0 pt-1">
                        <v-container pa-0 ma-0 class="d-flex justify-end">
                            <v-tooltip top>
                                <template v-slot:activator="{ on, attrs }">
                                    <v-btn small icon v-bind="attrs" v-on="on" @click.stop.prevent="newInvoice(contact.id)">
                                        <v-icon color="green lighten-1">mdi-text-box-plus</v-icon>
                                    </v-btn>
                                </template>
                                <span>Create new invoice for {{ contact.name }}</span>
                            </v-tooltip>
                            <v-tooltip top>
                                <template v-slot:activator="{ on, attrs }">
                                    <v-btn small icon v-bind="attrs" v-on="on" @click.stop.prevent="deleteContact(contact.id)">
                                        <v-icon color="red lighten-1">mdi-delete</v-icon>
                                    </v-btn>
                                </template>
                                <span>Delete '{{ contact.name }}' contact</span>
                            </v-tooltip>
                        </v-container>
                    </v-col>
                </v-row>
            </v-card-text>
        </v-card>


        <v-card-actions class="mt-1 pa-0">
            <v-dialog v-model="newDialog.shown" persistent max-width="600px" overlay-opacity="0.9">
                <v-card outlined raised>
                    <v-card-title>
                        <span class="headline">Add new contact</span>
                    </v-card-title>
                    <v-card-text>
                        <v-container fluid>
                            <v-row>
                                <v-col>
                                    <v-text-field label="Name*" v-model="newDialog.name" counter="250" required/>
                                </v-col>
                            </v-row>
                            <v-row>
                                <v-col>
                                    <v-text-field label="Code" type="number" v-model="newDialog.code" counter="100"/>
                                </v-col>
                            </v-row>
                            <v-row>
                                <v-col>
                                    <v-select
                                        :items="vatSelectItems"
                                        v-model="newDialog.vat.type"
                                        label="VAT type"
                                        outlined
                                    ></v-select>

                                    <v-text-field v-if="newDialog.vat.type === 'Code'" label="VAT code" v-model="newDialog.vat.value"
                                                  counter="100"/>
                                </v-col>
                            </v-row>
                            <v-row>
                                <v-col>
                                    <v-textarea label="Address*" v-model="newDialog.address" counter="250" required/>
                                </v-col>
                            </v-row>
                        </v-container>
                        <small>*indicates required field</small></v-card-text>
                    <v-card-actions>
                        <v-spacer></v-spacer>
                        <v-btn color="red darken-1" text @click="newDialog.shown=false">Close</v-btn>
                        <v-btn color="green darken-1" text @click="saveNewContactAndClose">Save</v-btn>
                        <v-btn color="green darken-1" text @click="saveNewContactAndCreateInvoice">Save & create invoice</v-btn>
                    </v-card-actions>
                </v-card>
            </v-dialog>
            <v-spacer/>
            <v-btn color="green darken-1" text @click="showNewDialog">New</v-btn>
        </v-card-actions>
    </v-card>
</template>

<style scoped lang="scss">
.contact-row {
    font-size: large;
    color: white;
}

.col-name {
    font-weight: bold;
}

</style>

<script>
import {SnotifyPosition} from "vue-snotify";

export default {
    name: 'Contacts',
    mounted() {
        this.ajax('data-get/contacts/' + this.getEntrepreneurId()).then(data => {
            this.contacts = data;
            this.filteredContacts = data;
            this.loading = false;
            this.filterData()
        });
    },
    data() {
        return {
            loading: true,
            searchPhrase: "",
            contacts: [],
            filteredContacts: [],
            vatSelectItems: [
                {text: "Doesn't have", value: "DontDisplay"},
                {text: "Not a tax payer", value: "NotTaxPayer"},
                {text: "Code", value: "Code"},
            ],
            newDialog: {
                shown: false,
                name: "",
                code: "",
                address: "",
                vat: {type: "DontDisplay"}
            }
        }
    },
    computed: {
        search: {
            get: function () {
                return this.searchPhrase
            },
            set: function (value) {
                value = value == null ? "" : value

                this.searchPhrase = value.toLowerCase()
                this.filterData()
            }
        }
    },
    methods: {
        filterData: function () {
            let searchPhrase = this.searchPhrase
            this.$set(this, 'filteredContacts', this.lodash.filter(this.contacts, function (e) {
                if (searchPhrase !== "") {
                    return e.name.toLowerCase().indexOf(searchPhrase) > -1 ||
                        e.code.toLowerCase().indexOf(searchPhrase) > -1 ||
                        e.address.toLowerCase().indexOf(searchPhrase) > -1
                } else return true
            }))
        },
        showNewDialog: function () {
            this.newDialog.shown = true
            this.newDialog.name = "";
            this.newDialog.code = "";
            this.newDialog.address = "";
        },
        saveNewContact: function () {
            // TODO validation
            let contact = {
                code: this.newDialog.code !== "" ? this.newDialog.code : null,
                name: this.newDialog.name,
                address: this.newDialog.address.replaceAll("\n", "\r\n").replaceAll("\r\r\n", "\r\n").replaceAll("\r\n\n", "\r\n"),
                entrepreneurId: this.getEntrepreneurId()
            };

            switch (this.newDialog.vat.type) {
                case "Code":
                    contact.vat = {Code: this.newDialog.vat.value}
                    break
                default:
                    contact.vat = this.newDialog.vat.type
            }

            console.log("Adding new contact: ")
            console.log(contact)

            return this.asyncActionWithNotification("data-insert/contact", contact, "Creating contact", (resp) => new Promise((success, error) => {
                    if (resp.id >= 0) {
                        success("Contact saved")
                    } else {
                        error("Could not save contact")
                    }
                })
            )
        },
        saveNewContactAndCreateInvoice: function () {
            this.saveNewContact().then((resp) => this.newInvoice(resp.id))
        },
        saveNewContactAndClose: function () {
            this.saveNewContact().then((resp) => this.$router.push({name: 'ContactDetail', params: {id: resp.id}}))
        },
        deleteContact: function (id) {
            this.$snotify.confirm('Really delete this contact?', 'Delete', {
                timeout: 5000,
                closeOnClick: false,
                pauseOnHover: true,
                position: SnotifyPosition.centerCenter,
                buttons: [
                    {
                        text: 'Yes', action: (toast) => {
                            console.log("Deleting contact " + id)

                            this.ajax("data-delete/contact/" + id).then(r => {
                                if (r.success) {
                                    this.$set(this, 'contacts', this.lodash.filter(this.contacts, function (e) {
                                        return e.id !== id
                                    }))
                                    this.filterData()
                                } else {
                                    this.$snotify.error("Could not delete the contact!", "Delete");
                                }
                            })
                            this.$snotify.remove(toast.id);
                        }
                    },
                    {
                        text: 'No', action: (toast) => {
                            this.$snotify.remove(toast.id);
                        }
                    },
                ]
            });
        },
        newInvoice: function (contactId) {
            console.log("Adding new invoice for contact id " + contactId)

            let invoice = {
                accountId: this.getUserId(),
                entrepreneurId: this.getEntrepreneurId(),
                contactId: contactId
            };

            console.log("Adding new invoice: ")
            console.log(invoice)

            this.asyncActionWithNotification("data-insert/invoice", invoice, "Creating invoice", (resp) => new Promise((success, error) => {
                    if (resp.id >= 0) {
                        success("Invoice created")
                        this.$router.push({name: 'InvoiceDetail', params: {id: resp.id}})
                    } else {
                        error("Could not save invoice")
                    }
                })
            );
        }
    }
}
</script>
