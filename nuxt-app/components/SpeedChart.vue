<template>
  <div class="speed-chart">
    <canvas ref="chartRef" class="w-full" style="height:64px"></canvas>
    <div v-if="vpnStore.connectionState === 'connected'" class="grid grid-cols-4 gap-2 mt-2.5">
      <div class="text-center">
        <div class="text-xs tabular-nums" style="color:#3b82f6">{{ formatByteSpeed(vpnStore.status.download_speed) }}</div>
        <div class="text-[10px] mt-0.5" style="color:rgba(255,255,255,0.25)">Download</div>
      </div>
      <div class="text-center">
        <div class="text-xs tabular-nums" style="color:#22bb66">{{ formatByteSpeed(vpnStore.status.upload_speed) }}</div>
        <div class="text-[10px] mt-0.5" style="color:rgba(255,255,255,0.25)">Upload</div>
      </div>
      <div class="text-center">
        <div class="text-xs tabular-nums" style="color:rgba(255,255,255,0.6)">{{ peakDownload }}</div>
        <div class="text-[10px] mt-0.5" style="color:rgba(255,255,255,0.25)">Peak ↓</div>
      </div>
      <div class="text-center">
        <div class="text-xs tabular-nums" style="color:rgba(255,255,255,0.6)">{{ totalTraffic }}</div>
        <div class="text-[10px] mt-0.5" style="color:rgba(255,255,255,0.25)">Total</div>
      </div>
    </div>
    <div v-else class="flex items-center justify-center text-xs mt-3 py-2" style="color:rgba(255,255,255,0.2)">
      Connect to see real-time statistics
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useVpnStore } from '~/stores/vpn'
import { useI18n } from '~/composables/useI18n'
import { Chart, registerables } from 'chart.js'

Chart.register(...registerables)

const vpnStore = useVpnStore()
const { t } = useI18n()
const chartRef = ref<HTMLCanvasElement | null>(null)
let chartInstance: Chart | null = null

const labels = ref<string[]>([])
const downloadData = ref<number[]>([])
const uploadData = ref<number[]>([])

const peakDownload = computed(() => {
  if (downloadData.value.length === 0) return '-'
  const max = Math.max(...downloadData.value)
  return formatSize(max)
})

const totalTraffic = computed(() => {
  const dl = vpnStore.status.total_download || 0
  const ul = vpnStore.status.total_upload || 0
  const total = dl + ul
  return formatBytes(total)
})

function formatSize(mbps: number): string {
  if (mbps >= 1000) return `${(mbps / 1000).toFixed(1)} Gbps`
  if (mbps >= 1) return `${mbps.toFixed(1)} Mbps`
  return `${(mbps * 1000).toFixed(0)} Kbps`
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  if (bytes < 1024) return `${bytes.toFixed(0)} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`
}

function formatByteSpeed(bps: number): string {
  if (bps === 0) return '0'
  if (bps < 1000) return `${bps.toFixed(0)} b/s`
  if (bps < 1_000_000) return `${(bps / 1000).toFixed(1)} Kb/s`
  return `${(bps / 1_000_000).toFixed(1)} Mb/s`
}

function initChart() {
  if (!chartRef.value) return
  chartInstance = new Chart(chartRef.value, {
    type: 'line',
    data: {
      labels: labels.value,
      datasets: [
        {
          label: 'Download',
          data: downloadData.value,
          borderColor: '#3b82f6',
          backgroundColor: 'rgba(59, 130, 246, 0.08)',
          fill: true,
          tension: 0.3,
          pointRadius: 0,
          borderWidth: 1.5,
        },
        {
          label: 'Upload',
          data: uploadData.value,
          borderColor: '#22bb66',
          backgroundColor: 'rgba(34, 187, 102, 0.08)',
          fill: true,
          tension: 0.3,
          pointRadius: 0,
          borderWidth: 1.5,
        },
      ],
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      animation: { duration: 200 },
      scales: {
        x: { display: false, grid: { display: false } },
        y: {
          beginAtZero: true,
          grid: { color: 'rgba(255,255,255,0.03)' },
          ticks: {
            display: false,
          },
        },
      },
      plugins: {
        legend: { display: false },
        tooltip: {
          mode: 'index',
          intersect: false,
          backgroundColor: '#1a1d27',
          borderColor: '#2e3142',
          borderWidth: 1,
          titleColor: '#e4e6f0',
          bodyColor: '#9ca0b0',
          callbacks: {
            label: (ctx) => `${ctx.dataset.label}: ${formatSize(ctx.parsed.y)}`,
          },
        },
      },
      interaction: { intersect: false, mode: 'index' },
    },
  })
}

function updateChart() {
  if (!chartInstance) return
  const now = new Date().toLocaleTimeString()
  labels.value.push(now)
  downloadData.value.push(vpnStore.status.download_speed / 1_000_000)
  uploadData.value.push(vpnStore.status.upload_speed / 1_000_000)
  if (labels.value.length > 60) {
    labels.value.shift()
    downloadData.value.shift()
    uploadData.value.shift()
  }
  chartInstance.data.labels = labels.value
  chartInstance.data.datasets[0].data = downloadData.value
  chartInstance.data.datasets[1].data = uploadData.value
  chartInstance.update('none')
}

let interval: ReturnType<typeof setInterval> | null = null

onMounted(() => {
  initChart()
  interval = setInterval(() => {
    if (vpnStore.isConnected) {
      updateChart()
    }
  }, 1000)
})

onUnmounted(() => {
  if (interval) clearInterval(interval)
  if (chartInstance) chartInstance.destroy()
})
</script>
