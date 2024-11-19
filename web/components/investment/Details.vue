<script setup lang="ts">
import { Account } from '@/composables/account';
import EditAccount from './accounts/EditAccount.vue';
import DeleteAccount from './accounts/DeleteAccount.vue';
import Transactions from './Transactions.vue';

const emits = defineEmits<{ exit: [] }>()
const props = defineProps<{ account: Account }>()
</script>

<template>
    <div class="flex items-center px-4 mt-2">
        <VaButton background-opacity="0" color="textPrimary"
                  icon="arrow_back_ios_new" @click="emits('exit')"
                  class="flex-grow-0 flex-shrink-0" />
        <VaChip square readonly class="flex-grow-0 flex-shrink-0 ml-2">{{
            props.account.kind }}</VaChip>
        <div class="flex-grow-0 flex-shrink-0 text-xl ml-4 font-bold">{{
            props.account.alias
            }}</div>
        <div v-if="props.account.alias != props.account.name"
             class="flex-grow flex-shrink text-xl min-w-0 truncate ml-2">
            ({{ props.account.name
            }})</div>
        <VaSpacer v-else />
        <EditAccount v-model="props.account" />
        <DeleteAccount :account_id="props.account.id" @delete="emits('exit')" />
    </div>
    <VaDivider />
    <div class="px-4 max-w-6xl flex gap-4 justify-center mx-auto">
        <div class="flex-grow flex-shrink flex flex-col gap-4">
            <VaCard class="flex-grow-0 h-72 w-full">
                <VaCardTitle>
                    Performance
                </VaCardTitle>
            </VaCard>
            <Transactions :account="account" />
        </div>
        <div class="w-64 flex-grow-0 flex-shrink-0 flex flex-col gap-4">
            <VaCard class="flex-grow-0 h-72 w-full">
                <VaCardTitle>
                    Composition
                </VaCardTitle>
            </VaCard>
            <VaCard class="flex-grow-0 h-72 w-full">
                <VaCardTitle>
                    Composition
                </VaCardTitle>
            </VaCard>
            <VaCard class="flex-grow-0 h-72 w-full">
                <VaCardTitle>
                    Composition
                </VaCardTitle>
            </VaCard>
            <VaCard class="flex-grow-0 h-72 w-full">
                <VaCardTitle>
                    Composition
                </VaCardTitle>
            </VaCard>
        </div>
    </div>
</template>

<style scoped></style>
