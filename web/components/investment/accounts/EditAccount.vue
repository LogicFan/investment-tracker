<script setup lang="ts">
import { useForm } from 'vuestic-ui';
import { reactive, ref } from 'vue';
import { Account } from '@/composables/account';
import axios from 'axios';
const account = defineModel<Account>({ required: true })

const { isLoading, isValid, validateAsync } = useForm('formRef')

const modal = ref(false);
const form: Account = reactive({
    id: account.value.id,
    name: account.value.name,
    alias: account.value.alias,
    owner: account.value.owner,
    kind: account.value.kind,
})

function reset() {
    form.id = account.value.id;
    form.name = account.value.name;
    form.alias = account.value.alias;
    form.owner = account.value.owner;
    form.kind = account.value.kind;
}

async function beforeOk(hide: () => void) {
    await validateAsync();
    if (isLoading.value || !isValid.value) { return; }

    if (form.alias == "") {
        form.alias = form.name
    }

    await axios.post('/api/investment/account/update', {
        token: localStorage.getItem('token'),
        account: form
    });
    account.value.name = form.name;
    account.value.alias = form.alias;
    hide();
}
</script>

<template>
    <div class="flex-grow-0 flex-shrink-0 ml-2">
        <VaButton icon="ms-edit" @click="modal = true" />
        <VaModal v-model="modal" ok-text="Save" size="auto" @open="reset"
                 :before-ok="beforeOk">
            <VaForm ref="formRef" class="w-80 flex flex-col items-center">
                <VaInput v-model="form.name" label="Account ID"
                         name="Account ID"
                         :rules="[(x) => x.length >= 4 || 'id too short']"
                         class="w-4/5 flex-grow-0" />
                <VaInput v-model="form.alias" label="Account Alias"
                         name="Account Alias" messages="Optional"
                         :rules="[(x) => x.length == 0 || x.length >= 4 || 'alias too short']"
                         class="w-4/5 flex-grow-0 mt-2" />
                <VaSelect v-model="form.kind" disabled label="Account Kind"
                          class="w-4/5 flex-grow-0 mt-2" />
            </VaForm>
        </VaModal>
    </div>
</template>
