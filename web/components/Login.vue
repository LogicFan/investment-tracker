<script setup lang="ts">
import { reactive } from "vue";
import { useForm } from 'vuestic-ui';
import axios from "axios";
import { logout, login, getUsername, duplicateUsername } from '@composables/user';

const authorize = defineModel<boolean>({ required: true })
const { isLoading, isValid, validateAsync, reset } = useForm('formRef')

const modal = reactive({
    show: false,
    login: true,
});

const form = reactive({
    username: '',
    password: '',
    password2: '',
    response: true as boolean | string,
})

async function beforeOk(hide: () => void) {
    await validateAsync();
    if (isLoading.value || !isValid.value) { return; }

    if (modal.login) {
        try {
            let response = await axios.post("/api/user/login", {
                username: form.username,
                password: form.password,
            });
            hide();
            login(response.data);
            authorize.value = true;
        } catch (error) {
            console.log(error)
            form.response = error.response.data;
            await validateAsync();
            form.response = true;
            authorize.value = false;
        }
    } else {
        await axios.post("/api/user/register", {
            username: form.username,
            password: form.password,
        });
        modal.login = true;
    }
}

function rotateToken() {
    let token = localStorage.getItem('token');
    if (token != null) {
        axios.post("/api/user/rotate", {
            token: token
        }).then(response => {
            login(response.data);
            authorize.value = true;
        }).catch(_ => {
            logout();
            authorize.value = false;
        })
    } else {
        authorize.value = false;
    }
}
rotateToken();
setInterval(rotateToken, 1000 * 60);
</script>

<template>
    <template v-if="!authorize">
        <VaButton icon="ms-login" background-opacity="0" color="textPrimary"
                  @click="modal.show = true; modal.login = true"
                  class="login-button" />
    </template>
    <template v-else>
        <VaButtonDropdown background-opacity="0" color="textPrimary" hide-icon
                          :close-on-content-click="false"
                          placement="bottom-start" class="login-button">
            <template #label>
                <VaIcon name="ms-account_circle" size="24px" />
            </template>
            <div class="flex flex-col">
                <div class="text-sm text-center">
                    {{ getUsername() }}
                </div>
                <VaDivider />
                <VaButton background-opacity="0" color="textPrimary"
                          to="/settings">
                    Settings
                </VaButton>
                <VaButton background-opacity="0" color="textPrimary"
                          @click="authorize = false; logout()">
                    Logout
                </VaButton>
            </div>
        </VaButtonDropdown>
    </template>
    <VaModal v-model="modal.show" size="auto"
             :ok-text="modal.login ? 'Login' : 'Signup'" @open="reset"
             :before-ok="beforeOk">
        <VaForm ref="formRef" class="w-80 flex flex-col items-center">
            <template v-if="modal.login">
                <VaInput v-model="form.username" label="Username"
                         class="w-4/5 flex-grow-0" />
                <VaInput v-model="form.password" label="Password"
                         type="password"
                         :rules="[(_) => form.response]"
                         class="w-4/5 flex-grow-0 mt-2" />
                <div class="w-4/5 flex-grow-0 mt-4">
                    Don't have an account?
                    <span class="text-bold cursor-pointer underline"
                          @click="modal.login = false; reset()">Sign up</span>
                </div>
            </template>
            <template v-else>
                <VaInput v-model="form.username" label="Username" :rules="[
                    ((x) => x.length >= 6 || 'username too short'),
                    duplicateUsername
                ]" class="w-4/5 flex-grow-0" />
                <VaInput v-model="form.password" label="Password"
                         type="password"
                         :rules="[(x) => x.length >= 8 || 'password too short']"
                         class="w-4/5 flex-grow-0 mt-2" />
                <VaInput v-model="form.password2" label="Repeat Password"
                         type="password"
                         :rules="[(x) => x == form.password || 'password does not match']"
                         class="w-4/5 flex-grow-0 mt-2" />
                <div class="w-4/5 flex-grow-0 mt-4">
                    Already have an account?
                    <span class="text-bold cursor-pointer underline"
                          @click="modal.login = true; reset()">Log
                        in</span>
                </div>
            </template>
        </VaForm>
    </VaModal>
</template>

<style>
.login-button {
    --va-button-size: 32px;
    --va-button-content-py: 4px;
}
</style>
