<script setup lang="ts">
import { Transaction } from '@/composables/transaction';
import axios from 'axios';
import { ref } from 'vue';
import NewTransaction from './transactions/NewTransaction.vue';
import { Account } from '@/composables/account';
const props = defineProps<{ account: Account }>()

const transactions = ref<Transaction[]>([]);
const columns = [
    { key: "date", name: "date", label: "Date" },
    { key: "action", name: "content", label: "Content" },
    { key: "action", name: "fee", label: "Fee" },
    { key: "id", name: "edit", label: "Edit" },
];

function fetch() {
    axios.post('/api/investment/transaction/fetch', {
        token: localStorage.getItem('token'),
        account: props.account.id
    }).then(response => {
        transactions.value = response.data;
    })
}

fetch();
</script>

<template>
    <div class="flex-grow w-full">
        <VaCard class="w-full">
            <VaCardTitle>
                Transactions
            </VaCardTitle>
            <VaCardContent>
                <VaDataTable :items="transactions" :columns="columns">
                    <template #header(edit)="">
                        <NewTransaction :account="account" />
                    </template>
                    <template #cell(edit)="">
                        <VaButton icon="ms-edit" size="small" />
                        <VaButton icon="ms-delete" size="small" />
                    </template>
                </VaDataTable>
            </VaCardContent>
        </VaCard>
    </div>
</template>

<style scoped></style>
