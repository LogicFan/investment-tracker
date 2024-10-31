<script setup lang="ts">
import { getUserId } from '@/composables/user';
import axios from 'axios';
import { reactive } from 'vue';
import { useForm } from 'vuestic-ui';
const { isLoading, isValid, validateAsync, reset } = useForm('formRef')

const modal = reactive({
    show: false,
    text: "",
    action: async () => { },
});
const form = reactive({
    password: "",
    status200: true,
});

async function deleteUser() {
    await axios.post('/api/user/delete', {
        token: localStorage.getItem('token'),
        password: form.password,
        id: getUserId(),
    });
}

function showDeleteUserModal() {
    modal.show = true;
    modal.text = "Are you sure you want to delete your user account?";
    modal.action = deleteUser
}

async function beforeOk(hide: () => void) {
    await validateAsync();
    if (isLoading.value || !isValid.value) { return; }

    try {
        await modal.action()

        hide();
    } catch {
        form.status200 = false;
        await validateAsync();
        form.status200 = true;
    }
}
</script>

<template>
    <VaCard stripe stripe-color="danger">
        <VaCardTitle>Danger Zone</VaCardTitle>
        <VaCardContent class="grid grid-cols-2 gap-4">
            <VaButton @click="showDeleteUserModal">Delete User</VaButton>
        </VaCardContent>
        <VaModal v-model="modal.show" ok-text="Confirm" size="auto"
                 @open="reset" :before-ok="beforeOk">
            <div class="w-80">
                <p>{{ modal.text }}</p>
                <p class="mt-2">This operation is <span
                          class="font-bold">irreversible</span>.
                </p>
                <p class="mt-2">Please enter password to confirm:</p>
            </div>
            <VaForm ref="formRef" class="w-80 flex flex-col items-center mt-2">
                <VaInput v-model="form.password" label="Password"
                         type="password"
                         :rules="[(_) => form.status200 || 'incorrect password']"
                         class="w-4/5 flex-grow-0 mt-2" />
            </VaForm>
        </VaModal>
    </VaCard>
</template>
