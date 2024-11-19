<script setup lang="ts">
import { Account } from '@/composables/account';
import { actionOptions, TxnAction } from '@/composables/transaction';
import axios from 'axios';
import { computed, reactive, ref } from 'vue';
import { useForm } from 'vuestic-ui';
import EditTxnActionDep from './edit/EditTxnActionDep.vue';
const props = defineProps<{ account: Account }>()
const emits = defineEmits<{
    insert: []
}>()

const { isLoading, isValid, reset, validateAsync } = useForm('formRef')

const modal = ref(false);
const form = reactive({
    date: null as null | Date,
    actionType: '',
    action: null as null | TxnAction,
})

async function beforeOk(hide: () => void) {
    await validateAsync();
    if (isLoading.value || !isValid.value) { return; }

    await axios.post('/api/investment/account/insert', {
        token: localStorage.getItem('token'),
        transaction: {
            account: props.account.id,
            date: form.date,
            action: form.action,
        }
    });
    emits("insert");
    hide();
}

const isTxnActionDep = computed(
    () => ['Deposit', 'Withdrawal', 'Interest'].includes(form.actionType))
</script>

<template>
    <div>
        <VaButton icon="ms-add" size="small" @click="modal = true" />
        <VaModal v-model="modal" ok-text="Save" size="auto" @open="reset"
                 :before-ok="beforeOk">
            <VaForm ref="formRef" class="w-80 flex flex-col items-center">
                <div class="w-full flex-grow-0 grid grid-cols-2 gap-2">
                    <div>
                        <VaDateInput v-model="form.date" label="Date" />
                    </div>
                    <div>
                        <VaSelect v-model="form.actionType"
                                  :options="actionOptions"
                                  placeholder="Select an option" label="Action"
                                  :rules="[(x) => x != '' || 'action type must be selected']" />
                    </div>
                </div>
                <template v-if="isTxnActionDep">
                    <!-- @vue-skip -->
                    <EditTxnActionDep v-model="form.action"
                                      :action-type="form.actionType"
                                      :account="account" />
                </template>
            </VaForm>
        </VaModal>
    </div>
</template>

<style scoped></style>
