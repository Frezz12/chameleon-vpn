<template>
  <div class="rule-editor">
    <div v-if="rule" class="space-y-4">
      <div>
        <label class="block text-xs text-secondary mb-1">Type</label>
        <select v-model="localRule.rule_type" class="input w-full">
          <option value="domain_full">Domain (exact match)</option>
          <option value="domain_suffix">Domain (suffix / wildcard)</option>
          <option value="domain_keyword">Domain (keyword)</option>
          <option value="domain_regex">Domain (regex)</option>
          <option value="ip_cidr">IP (CIDR)</option>
          <option value="process_name">Process name</option>
          <option value="geoip">GeoIP</option>
          <option value="geosite">Geosite</option>
        </select>
      </div>

      <div>
        <label class="block text-xs text-secondary mb-1">Value</label>
        <input v-model="localRule.value" type="text" placeholder="e.g., google.com" class="input w-full" />
      </div>

      <div>
        <label class="block text-xs text-secondary mb-1">Target Node</label>
        <select v-model="localRule.node_id" class="input w-full">
          <option value="direct">Direct (bypass VPN)</option>
          <option value="block">Block</option>
          <option v-for="node in nodeOptions" :key="node.id" :value="node.id">{{ node.name }}</option>
        </select>
      </div>

      <div class="flex items-center gap-2">
        <input v-model="localRule.enabled" type="checkbox" id="editor-enabled" class="rounded" />
        <label for="editor-enabled" class="text-sm">Enabled</label>
      </div>

      <div class="flex justify-end gap-3 pt-2">
        <button @click="$emit('cancel')" class="btn-secondary">Cancel</button>
        <button @click="save" class="btn-primary">Save</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive, watch } from 'vue'

const props = defineProps<{
  rule: any | null
  nodeOptions: { id: string; name: string }[]
}>()

const emit = defineEmits<{
  save: [rule: any]
  cancel: []
}>()

const localRule = reactive({
  rule_type: 'domain_suffix',
  value: '',
  node_id: 'direct',
  enabled: true,
})

watch(() => props.rule, (rule) => {
  if (rule) {
    Object.assign(localRule, rule)
  }
}, { immediate: true })

function save() {
  emit('save', { ...localRule })
}
</script>

<style scoped>
.input {
  @apply rounded-lg bg-[var(--bg-tertiary)] border border-[var(--border-color)] px-3 py-2 text-sm
    focus:outline-none focus:border-accent-blue transition-colors;
}
.btn-primary {
  @apply px-4 py-2 rounded-lg bg-accent-blue text-white text-sm font-medium hover:opacity-90 transition-opacity;
}
.btn-secondary {
  @apply px-4 py-2 rounded-lg bg-tertiary text-primary text-sm font-medium hover:bg-border transition-colors;
}
</style>
