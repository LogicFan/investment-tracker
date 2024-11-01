<script setup lang="ts">
import { useForm } from 'vuestic-ui';
import { reactive, ref } from 'vue';
import { kindOptions } from '@/composables/account';
import { getUserId } from '@/composables/user';
import axios from 'axios';

const emits = defineEmits<{
    insert: []
}>()

const { isLoading, isValid, reset, validateAsync } = useForm('formRef')

const hover = ref(false);
const modal = ref(false);
const form = reactive({
    name: "",
    alias: "",
    kind: "",
})

async function beforeOk(hide: () => void) {
    await validateAsync();
    if (isLoading.value || !isValid.value) { return; }

    if (form.alias == "") {
        form.alias = form.name
    }

    await axios.post('/api/investment/account/insert', {
        token: localStorage.getItem('token'),
        account: {
            name: form.name,
            alias: form.alias,
            owner: getUserId(),
            kind: form.kind,
        }
    });
    emits("insert");
    hide();
}
</script>

<template>
    <div>
        <VaHover v-model="hover" class="h-36">
            <VaCard :gradient="hover" @click="modal = true"
                    class="h-full flex items-center justify-center">
                <VaIcon name="ms-add" size="64px" class="flex-grow-0" />
            </VaCard>
        </VaHover>
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
                <VaSelect v-model="form.kind" :options="kindOptions"
                          placeholder="Select an option" label="Account Kind"
                          :rules="[(x) => x != '' || 'kind must be selected']"
                          class="w-4/5 flex-grow-0 mt-2" />
            </VaForm>
        </VaModal>
    </div>
</template>
