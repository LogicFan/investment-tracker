<script setup lang="ts">
const value = defineModel<[number, string]>()
const props = defineProps<{
    label: string,
    options: string[]
}>()
</script>

<template>
    <div class="w-full grid grid-cols-2 gap-2">
        <div>
            <VaInput v-model="value[0]" :label="props.label" />
        </div>
        <div>
            <VaSelect v-model="value[1]" :options="props.options"
                      placeholder="Select an option"
                      :rules="[(x) => x != '' || 'currency must be selected']"
                      class="no-label">
                <template #option-content="{ option }">
                    {{ option.toString().split(':')[1] }}
                </template>
            </VaSelect>
        </div>
    </div>
</template>

<style>
.no-label {
    margin-top: 18px !important;
}
</style>
