<template>
  <div class="node-list space-y-2">
    <div
      v-for="node in nodes"
      :key="node.id"
      @click="$emit('select', node.id)"
      class="flex items-center justify-between p-3 rounded-lg cursor-pointer transition-all duration-200"
      :class="[
        selectedId === node.id
          ? 'bg-accent-blue/10 border border-accent-blue/30'
          : 'bg-tertiary/50 hover:bg-tertiary border border-transparent',
      ]"
    >
      <div class="flex items-center gap-3">
        <span
          class="w-2.5 h-2.5 rounded-full"
          :class="{
            'bg-accent-green': node.latency_ms && node.latency_ms < 300,
            'bg-accent-yellow': node.latency_ms && node.latency_ms >= 300 && node.latency_ms < 1000,
            'bg-accent-red': !node.latency_ms || node.latency_ms >= 1000,
          }"
        ></span>
        <div>
          <p class="text-sm font-medium">{{ node.name }}</p>
          <p class="text-xs text-secondary">{{ node.server }}:{{ node.port }}</p>
        </div>
      </div>
      <div class="flex items-center gap-3">
        <span class="text-xs px-2 py-0.5 rounded bg-tertiary uppercase text-secondary">{{ node.protocol }}</span>
        <span class="text-xs font-mono">{{ node.latency_ms ? `${node.latency_ms.toFixed(0)}ms` : '---' }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { VpnNode } from '~/stores/nodes'

defineProps<{
  nodes: VpnNode[]
  selectedId: string | null
}>()

defineEmits<{
  select: [nodeId: string]
}>()
</script>
