<script setup lang="ts">
import axios from 'axios';
import { ref } from 'vue';
import { Account } from '@/composables/account';
import AccountCard from './accounts/AccountCard.vue';
import NewAccount from './accounts/NewAccount.vue';
const emits = defineEmits<{ select: [value: Account] }>()

const accounts = ref<Account[]>([]);

function fetch() {
    axios.post('/api/investment/account/fetch', {
        token: localStorage.getItem('token'),
    }).then(response => {
        accounts.value = response.data;
    })
}

fetch();
</script>

<template>
    <div class="px-4 !mt-4 flex gap-4">
        <VaCard class="h-72">
            <VaCardTitle>
                Performance
            </VaCardTitle>
        </VaCard>
        <VaCard class="h-72 w-64 flex-grow-0">
            <VaCardTitle>
                Composition
            </VaCardTitle>
        </VaCard>
        <VaCard class="h-72 w-64 flex-grow-0">
            <VaCardTitle>
                Contribution Room
            </VaCardTitle>
        </VaCard>
    </div>
    <div class="px-4 !mt-4 grid gap-4 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 2xl:grid-cols-4">
        <template v-for="account in accounts">
            <AccountCard :account="account" @click="emits('select', account)" />
        </template>
        <NewAccount @insert="() => fetch()" />
    </div>
</template>

<style scoped></style>
