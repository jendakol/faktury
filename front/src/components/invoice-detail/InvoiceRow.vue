<template>
    <v-row class="pl-3 pr-3">
        <v-col cols="8">
            <v-text-field label="Item name" v-model="name" counter="200" :rules="rules.name"/>
        </v-col>
        <v-col cols="2">
            <v-currency-field
                    label="Item price"
                    v-model="price"
                    :suffix="currency"
            />

        </v-col>
        <v-col cols="1">
            <v-text-field
                    label="Count"
                    v-model="count"
                    type="number"
                    :rules="rules.count"/>
        </v-col>
        <v-col cols="1">
            <v-tooltip top>
                <template v-slot:activator="{ on, attrs }">
                    <v-btn icon v-bind="attrs" v-on="on" @click="saveRow(data.id)">
                        <v-icon color="green lighten-1">mdi-content-save</v-icon>
                    </v-btn>
                </template>
                <span>Save row</span>
            </v-tooltip>
            <v-tooltip top>
                <template v-slot:activator="{ on, attrs }">
                    <v-btn icon v-bind="attrs" v-on="on" @click="deleteRow(data.id)">
                        <v-icon color="red lighten-1">mdi-delete</v-icon>
                    </v-btn>
                </template>
                <span>Delete row</span>
            </v-tooltip>
        </v-col>
    </v-row>
</template>

<script>

    import {SnotifyPosition, SnotifyStyle} from 'vue-snotify';

    // TODO support enter-confirmation

    export default {
        name: 'InvoiceRow',
        props: {
            invoiceId: Number,
            data: Object
        },
        data() {
            return {
                currency: "Kč",
                rules: {
                    // TODO check non-empty
                    name: [v => (v != null && v.length <= 200) || 'Max 200 characters'],
                    price: [v => { // TODO apply price rule
                        const pattern = /^\d+([ ]\d+)*([.,]\d+)?$/
                        let fixedValue = this.lodash.replace(v, ' ', '');
                        console.log(fixedValue)
                        return (v != null && pattern.test(fixedValue)) || 'Must be a valid price.'
                    }],
                    count: [v => {
                        const pattern = /^\d+$/
                        return pattern.test(v) || 'Must be a valid count.'
                    }],
                },
            }
        },
        computed: {
            name: {
                get: function () {
                    return this.data.itemName
                },
                set: function (value) {
                    let rowData = this.data;
                    rowData.itemName = value;
                    this.$emit("row-updated", rowData)
                }
            },
            count: {
                get: function () {
                    return this.data.itemCount
                },
                set: function (value) {
                    let rowData = this.data;
                    rowData.itemCount = parseInt(value, 10);
                    this.$emit("row-updated", rowData)
                }
            },
            price: {
                get: function () {
                    return this.data.itemPrice
                },
                set: function (value) {
                    let rowData = this.data;
                    rowData.itemPrice = value;
                    this.$emit("row-updated", rowData)
                }
            }
        },
        methods: {
            saveRow: function (id) {
                console.log("Save: " + id)

                this.asyncActionWithNotification("update/invoice-row", this.data, "Saving", (resp) => new Promise((success, error) => {
                        if (resp.success) {
                            success("Row saved")
                            this.$emit("row-saved", id)
                        } else {
                            error("Could not save row")
                        }
                    })
                );
            },
            deleteRow: function (id) {
                this.$snotify.confirm('Really delete whole row?', 'Delete', {
                    timeout: 5000,
                    type: SnotifyStyle.warning,
                    closeOnClick: false,
                    pauseOnHover: true,
                    position: SnotifyPosition.centerCenter,
                    buttons: [
                        {
                            text: 'Yes', action: (toast) => {
                                this.ajax("delete/invoice-row/" + id, {}, 1000).then(r => {
                                    if (r.success) {
                                        this.$emit("row-deleted", id)
                                    } else {
                                        this.$snotify.error("Could not delete the row!", "Delete");
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
        },
    }
</script>