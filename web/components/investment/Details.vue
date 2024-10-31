<script setup lang="ts">
import { Account } from '@/composables/account';
import EditAccount from './details/EditAccount.vue';
import DeleteAccount from './details/DeleteAccount.vue';

const emits = defineEmits<{ back: [] }>()
const props = defineProps<{ account: Account }>()
</script>

<template>
    <div class="flex items-center px-4 mt-2">
        <VaButton background-opacity="0" color="textPrimary"
                  icon="arrow_back_ios_new" @click="emits('back')"
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
        <DeleteAccount :account_id="props.account.id" @delete="emits('back')" />
    </div>
</template>

<style scoped></style>
