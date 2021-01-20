<template>
    <v-row class="pl-3 pr-3" v-bind:class="this.rowClass">
        <v-col cols="8">
            <v-textarea label="Item name" rows="1" v-model="name" counter="400" v-on:input="delayedSave" :rules="rules.name"/>
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
                    <v-btn icon v-bind="attrs" v-on="on" @click.stop.prevent="deleteRow">
                        <v-icon color="red lighten-1">mdi-delete</v-icon>
                    </v-btn>
                </template>
                <span>Delete this row</span>
            </v-tooltip>

        </v-col>
    </v-row>
</template>

<style scoped lang="scss">
.row-unsaved {
    border: 1px solid red;
    border-radius: 3px;
}

.row-saved {
    border: 1px solid transparent;
    border-radius: 3px;
}
</style>

<script>

import {SnotifyPosition} from 'vue-snotify';

// TODO support enter-confirmation

export default {
    name: 'InvoiceRow',
    props: {
        invoiceId: Number,
        data: Object
    },
    data() {
        // TODO don't allow to save when rules fail

        return {
            currency: "Kč",
            rowSavingClass: "row-unsaved",
            rowNormalClass: "row-saved",
            saving: false,
            savingTimeoutId: null,
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
                    return (pattern.test(v) && parseInt(v) <= 65000) || 'Must be a valid count, less than 65000.'
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
        },
        rowClass: {
            get: function () {
                if (this.saving) return this.rowSavingClass

                return this.rowNormalClass
            }
        }
    },
    methods: {
        delayedSave: function () {
            clearTimeout(this.savingTimeoutId)
            this.savingTimeoutId = setTimeout(() => this.saveRow(), 300)
        },
        saveRow: function () {
            let saved = false

            setTimeout(() => {
                if (!saved) this.saving = true
            }, 300)

            this.data.itemName = this.data.itemName.replaceAll("\n", "\r\n").replaceAll("\r\r\n", "\r\n").replaceAll("\r\n\n", "\r\n")//   tady je bug!!!      Cannot read property 'replace' of undefined

            // TODO better validation?

            if (this.data.itemName === undefined || this.data.itemName.trim().length === 0 || this.data.itemCount === 0 || this.data.itemPrice === 0) {
                console.log("Invalid data, not saving row")
                return;
            }

            console.log("Save invoice row: ")
            console.log(this.data)

            this.ajax("data-update/invoice-row", this.data).then(resp => {
                if (resp.success) {
                    saved = true
                    this.$emit("row-saved", this.data.id)
                } else {
                    this.$snotify.error("Could not save the row!", "Saving")
                }

                this.saving = false
            }).catch(e => {
                this.$snotify.error("Could not save the row!", "Saving");
                console.log(e)
                this.saving = false
            })
        },
        deleteRow: function () {
            this.$snotify.confirm('Really delete whole row?', 'Delete', {
                timeout: 5000,
                closeOnClick: false,
                pauseOnHover: true,
                position: SnotifyPosition.centerCenter,
                buttons: [
                    {
                        text: 'Yes', action: (toast) => {
                            console.log("Deleting invoice row " + this.data.id)

                            this.ajax("data-delete/invoice-row/" + this.data.id).then(r => {
                                if (r.success) {
                                    this.$emit("row-deleted", this.data.id)
                                } else {
                                    this.$snotify.error("Could not delete the row!", "Delete");
                                }
                            })
                            this.$snotify.remove(toast.id)
                        }
                    },
                    {
                        text: 'No', action: (toast) => {
                            this.$snotify.remove(toast.id)
                        }
                    },
                ]
            });
        },
    },
}
</script>
