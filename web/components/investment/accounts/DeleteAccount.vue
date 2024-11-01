<script setup lang="ts">
import { ref } from 'vue';
import axios from 'axios';
const props = defineProps<{ account_id: string }>()
const emits = defineEmits<{ delete: [] }>()

const modal = ref(false);

async function beforeOk(hide: () => void) {
    await axios.post('/api/investment/account/delete', {
        token: localStorage.getItem('token'),
        account_id: props.account_id
    });
    emits('delete');
    hide();
}
</script>

<template>
    <div class="flex-grow-0 flex-shrink-0 ml-2">
        <VaButton icon="delete" @click="modal = true" />
        <VaModal v-model="modal" ok-text="Save" size="auto"
                 :before-ok="beforeOk">
            <div class="w-80">
                <p>Are you sure you want to delete this account?</p>
                <p class="mt-2">This operation is <span
                          class="font-bold">irreversible</span>.
                </p>
            </div>
        </VaModal>
    </div>
</template>
