<script setup lang="ts">
import { AccountKind } from '@/composables/account';
import { TxnActionDep, TxnActionDepType } from '@/composables/transaction';
import { reactive } from 'vue';
const props = defineProps<{
    accountKind: AccountKind,
    actionType: TxnActionDepType
}>()
const action = defineModel<TxnActionDep>()

const form = reactive({
    value: [0.0, ""],
    fee: [0.0, ""],
})

const currencyOptions = [
    'CURRENCY:USD',
    'CURRENCY:CAD'
];

let valueOptions = currencyOptions;
let feeOptions = currencyOptions;
if (['TFSA', 'RRSP', 'FHSA'].includes(props.accountKind) && ['Deposit', 'Withdrawal'].includes(props.actionType)) {
    valueOptions = [
        'CURRENCY:CAD'
    ]
}

console.log(props, action)
</script>

<template>
    <div class="w-full flex-grow-0 grid grid-cols-2 gap-2">
        <div>
            <VaInput v-model="form.value[0]" :label="String(actionType)" />
        </div>
        <div>
            <VaSelect v-model="form.value[1]" :options="valueOptions"
                      placeholder="Select an option"
                      :rules="[(x) => x != '' || 'currency must be selected']"
                      class="no-label">
                <template #option-content="{ option }">
                    {{ option.toString().split(':')[1] }}
                </template>
            </VaSelect>
        </div>
    </div>
    <div class="w-full flex-grow-0 grid grid-cols-2 gap-2">
        <div>
            <VaInput v-model="form.fee[0]" label="Fee" />
        </div>
        <div>
            <VaSelect v-model="form.fee[1]" :options="feeOptions"
                      placeholder="Select an option"
                      :rules="[(x) => x != '' || 'currency must be selected']"
                      class="no-label">
                <template #option-content="{ option }">
                    {{ option.toString().split(':')[1] }}
                </template>
            </VaSelect>
        </div>
    </div>
</template>

<style>
.no-label {
    margin-top: 18px !important;
}
</style>
