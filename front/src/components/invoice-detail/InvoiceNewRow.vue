<template>
    <v-row class="pl-3 pr-3">
        <v-col cols="8">
            <v-textarea label="New item name" rows="2" v-model="name" counter="400" :rules="rules.name"/>
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
            <v-btn text color="green lighten-1" @click="addRow">
                Add
            </v-btn>
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
        // TODO don't allow to save when rules fail
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
                    return (pattern.test(v) && parseInt(v) <= 65000) || 'Must be a valid count, less than 65000.'
                }],
            },
        }
    },
    mounted() {
        this.reset()
    },
    methods: {
        addRow: function () {
            let row = {
                invoiceId: parseInt(this.invoiceId, 10),
                itemName: this.name.replaceAll("\n", "\r\n").replaceAll("\r\r\n", "\r\n").replaceAll("\r\n\n", "\r\n"),
                itemPrice: parseFloat(this.price),
                itemCount: parseInt(this.count)
            };

            this.ajax("data-insert/invoice-row", row).then(resp => {
                if (resp.id >= 0) {
                    this.$emit("row-inserted", resp)
                }
            })
        },
        reset: function () {
            this.name = "";
            this.count = 1;
            this.price = 1;
        }
    },
}
</script>
