<template>
  <div class="tray-indicator flex items-center gap-2 px-2 py-1 rounded-lg text-xs"
    :class="vpnStore.isConnected ? 'bg-accent-green/10' : 'bg-tertiary'">
    <span class="w-2 h-2 rounded-full"
      :class="vpnStore.isConnected ? 'bg-accent-green animate-pulse' : 'bg-accent-red'">
    </span>
    <span :class="vpnStore.isConnected ? 'text-accent-green' : 'text-secondary'">
      {{ vpnStore.isConnected ? 'Connected' : 'Disconnected' }}
    </span>
    <span v-if="vpnStore.isConnected" class="text-secondary ml-1">
      {{ vpnStore.status.download_speed > 0 ? formatSpeed(vpnStore.status.download_speed) : '' }}
    </span>
  </div>
</template>

<script setup lang="ts">
import { useVpnStore } from '~/stores/vpn'

const vpnStore = useVpnStore()

function formatSpeed(bps: number): string {
  if (bps === 0) return ''
  if (bps < 1000) return `${bps.toFixed(0)} b/s`
  if (bps < 1_000_000) return `${(bps / 1000).toFixed(1)} Kb/s`
  return `${(bps / 1_000_000).toFixed(1)} Mb/s`
}
</script>
