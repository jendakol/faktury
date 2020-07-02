<template>
    <v-row class="pl-3 pr-3">
        <v-col cols="8">
            <v-text-field label="New item name" v-model="name" counter="200" :rules="rules.name"/>
        </v-col>
        <v-col cols="2">
            <v-currency-field
                    label="New item price"
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
                    <v-btn icon v-bind="attrs" v-on="on" @click="saveRow">
                        <v-icon color="green lighten-1">mdi-content-save</v-icon>
                    </v-btn>
                </template>
                <span>Save row</span>
            </v-tooltip>
        </v-col>
    </v-row>
</template>

<script>
    // TODO support enter-confirmation

    export default {
        name: 'InvoiceNewRow',
        props: {
            invoiceId: Number,
        },
        data() {
            return {
                currency: "Kč",
                name: null,
                count: null,
                price: null,
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
        mounted() {
            this.reset()
        },
        methods: {
            saveRow: function () {
                let row = {
                    invoiceId: parseInt(this.invoiceId, 10),
                    itemName: this.name,
                    itemPrice: parseFloat(this.price),
                    itemCount: parseInt(this.count)
                };

                this.asyncActionWithNotification("insert/invoice-row", row, "Saving", (resp) => new Promise((success, error) => {
                        if (resp.id >= 0) {
                            success("Row added")
                            this.$emit("row-inserted", resp)
                        } else {
                            error("Could not insert row")
                        }
                    })
                );
            },
            reset: function () {
                this.name = "";
                this.count = 1;
                this.price = 100;
            }
        },
    }
</script>
