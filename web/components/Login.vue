<script setup lang="ts">
import { reactive, ref } from "vue";
import { VaCard, VaInput, VaButton } from 'vuestic-ui';
import axios from "axios";

const emit = defineEmits<{
    (e: 'token', token: string): void
}>()

const form = reactive({
    username: "",
    password: "",
});

const loading = ref(false)
const failed = ref(false)

function login() {
    loading.value = true
    axios.post("/api/auth/login", {
        username: form.username,
        password: form.password,
    }).then(response => {
        emit("token", response.data)
        loading.value = false
    }).catch(_ => {
        failed.value = true
        loading.value = false
    })
}
</script>

<template>
    <div class="h-screen w-screen flex items-center justify-center">
        <VaCard class="login-card flex flex-col flex-grow-0 items-center justify-evenly">
            <VaInput v-model="form.username" label="Username" name="Username" class="w-3/5 flex-grow-0" />
            <VaInput v-model="form.password" label="Password" type="password" name="Password" immediate-validation
                error-messages="Login failed" :error="failed" @input="failed = false" @keyup.enter="login()"
                class="w-3/5 flex-grow-0" />
            <VaButton @click="login()" :loading="loading" class="w-1/3 flex-grow-0"> Login </VaButton>
        </VaCard>
    </div>
</template>

<style scoped></style>
