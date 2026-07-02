<template>
  <div class="dashboard-page" :class="{ connected: vpn.connectionState === 'connected' }">
    <div class="dash-bg" :class="vpn.connectionState"/>

    <!-- Floating clouds when connected -->
    <div v-if="vpn.connectionState === 'connected'" class="clouds-container">
      <div class="cloud c1"/>
      <div class="cloud c2"/>
      <div class="cloud c3"/>
      <div class="cloud c4"/>
      <div class="cloud c5"/>
    </div>

    <div class="dash-center">
      <div class="dashboard-brand">
        <img src="/branding/chameleon-mark.png" :alt="appName" class="dashboard-brand-mark" />
        <h1 class="dashboard-brand-name">{{ appName }}</h1>
        <div class="chroma-track">
          <span />
          <span />
          <span />
        </div>
      </div>

      <p class="dash-status">{{ statusText }}</p>

      <!-- Power Button -->
      <div class="power-wrap">
        <img src="/branding/chameleon-mark.png" :alt="appName" class="power-chameleon" />
        <div class="chameleon-scan" />
        <div class="aurora-orbit" :class="vpn.connectionState" />
        <!-- Outer ring glow -->
        <div class="power-glow" :class="vpn.connectionState"/>

        <!-- Animated ring -->
        <svg class="power-svg" :class="{ spinning: vpn.isTransitioning }" viewBox="0 0 220 220">
          <defs>
            <linearGradient id="ringGrad" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" :stop-color="gradColors[0]"/>
              <stop offset="50%" :stop-color="gradColors[1]"/>
              <stop offset="100%" :stop-color="gradColors[2]"/>
            </linearGradient>
            <filter id="ringBlur"><feGaussianBlur stdDeviation="6"/></filter>
          </defs>
          <!-- Glow layer -->
          <circle cx="110" cy="110" r="96" fill="none" stroke="url(#ringGrad)" stroke-width="6" filter="url(#ringBlur)" opacity="0.25"/>
          <!-- Main ring -->
          <circle cx="110" cy="110" r="96" fill="none" stroke="url(#ringGrad)" stroke-width="5" stroke-linecap="round"
            :stroke-dasharray="circum" :stroke-dashoffset="ringOffset"
            style="transition: stroke-dashoffset .5s cubic-bezier(.4,0,.2,1)"/>
          <!-- Tick marks -->
          <circle cx="110" cy="110" r="84" fill="none" stroke="rgba(255,255,255,0.03)" stroke-width="1" stroke-dasharray="2 8"/>
        </svg>

        <!-- Button disc -->
        <div class="power-disc" :class="vpn.connectionState">
          <div class="disc-ring"/>
          <button class="power-btn" @click="handleToggle" :disabled="vpn.isTransitioning">
            <svg width="52" height="52" viewBox="0 0 52 52">
              <defs>
                <linearGradient id="iconGrad" x1="0" y1="0" x2="52" y2="52">
                  <stop offset="0%" :stop-color="gradColors[0]"/>
                  <stop offset="100%" :stop-color="gradColors[2]"/>
                </linearGradient>
              </defs>
              <path d="M16 10 A16 16 0 1 0 36 10" fill="none" stroke="url(#iconGrad)" stroke-width="3.5" stroke-linecap="round"/>
              <line x1="26" y1="8" x2="26" y2="26" stroke="url(#iconGrad)" stroke-width="3.5" stroke-linecap="round"/>
            </svg>
          </button>
        </div>
      </div>

      <!-- Glow bar -->
      <div class="glow-bar" :class="vpn.connectionState"/>

      <!-- Traffic -->
      <div class="traffic-row">
        <div class="traffic-col">
          <div class="traffic-icon down">&darr;</div>
          <span class="traffic-val">{{ dlDisplay }}</span>
          <span class="traffic-lbl">{{ t('dashboard.download') }}</span>
        </div>
        <div class="traffic-sep"/>
        <div class="traffic-col">
          <div class="traffic-icon up">&uarr;</div>
          <span class="traffic-val">{{ ulDisplay }}</span>
          <span class="traffic-lbl">{{ t('dashboard.upload') }}</span>
        </div>
      </div>

      <!-- Info cards -->
      <div v-if="vpn.connectionState === 'connected'" class="info-cards">
        <div class="info-card" v-if="vpn.status.latency_ms">
          <span class="info-lbl">{{ t('dashboard.latency') }}</span>
          <span class="info-val">{{ vpn.status.latency_ms.toFixed(0) }}{{ t('common.ms') }}</span>
        </div>
        <div class="info-card" v-if="vpn.status.connected_at">
          <span class="info-lbl">{{ t('dashboard.uptime') }}</span>
          <span class="info-val">{{ uptime }}</span>
        </div>
        <div class="info-card" v-if="vpn.status.current_node_name">
          <span class="info-lbl">{{ t('dashboard.node') }}</span>
          <span class="info-val truncate">{{ vpn.status.current_node_name }}</span>
        </div>
      </div>

      <!-- Status Message -->
      <div v-if="statusMsg" class="status-msg" :class="statusMsg.type">
        <span class="status-msg-icon" v-html="statusMsg.icon"/>
        <div class="status-msg-text">
          <p class="status-msg-title">{{ statusMsg.title }}</p>
          <p class="status-msg-desc">{{ statusMsg.desc }}</p>
        </div>
      </div>

      <div v-if="connectError" class="err-box">{{ connectError }}</div>

      <!-- Quick Actions -->
      <div class="quick-actions">
        <!-- Bypass RU toggle -->
        <button class="qa-btn" :class="{ active: bypassRu }" @click="toggleBypassRu">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"><circle cx="7" cy="7" r="5.5"/><path d="M2 7h10M7 1.5a8 8 0 010 11M7 1.5a8 8 0 000 11"/></svg>
          <span>{{ t('dashboard.bypass_ru') }}</span>
          <span class="qa-indicator" :class="{ on: bypassRu }"/>
        </button>

        <!-- Import subscription -->
        <button class="qa-btn" @click="showImport = true">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"><path d="M7 2v7M3.5 5.5L7 9l3.5-3.5"/><path d="M1 10v1.5A1.5 1.5 0 002.5 13h9a1.5 1.5 0 001.5-1.5V10"/></svg>
          <span>{{ t('dashboard.import_sub') }}</span>
        </button>
      </div>

      <!-- Import Modal -->
      <Teleport to="body">
        <div v-if="showImport" class="import-overlay" @click.self="showImport = false">
          <div class="import-panel">
            <h3 class="import-title">{{ t('nodes.import_modal_title') }}</h3>
            <div class="import-field">
              <label>{{ t('nodes.subscription_url') }}</label>
              <input v-model="importUrl" type="text" :placeholder="t('nodes.subscription_placeholder')" class="import-input"/>
            </div>
            <div class="import-field">
              <label>{{ t('nodes.group_name') }}</label>
              <input v-model="importGroupName" type="text" :placeholder="t('nodes.group_name_placeholder')" class="import-input"/>
            </div>
            <div class="import-actions">
              <button @click="showImport = false" class="import-btn cancel">{{ t('nodes.cancel') }}</button>
              <button @click="doImport" class="import-btn primary" :disabled="!importUrl">{{ t('nodes.import') }}</button>
            </div>
          </div>
        </div>
      </Teleport>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { useVpnStore } from '~/stores/vpn'
import { useNodesStore } from '~/stores/nodes'
import { useI18n } from '~/composables/useI18n'
import { useTauri } from '~/composables/useTauri'
import { useNodes } from '~/composables/useNodes'
import { useToast } from '~/composables/useToast'

definePageMeta({ layout: 'default' })

const vpn = useVpnStore()
const nodesStore = useNodesStore()
const { t } = useI18n()
const toast = useToast()
const { invoke } = useTauri()
const { importSubscription } = useNodes()
const appName = computed(() => t('app.name'))

const verified = ref<boolean | null>(null)
const verifying = ref(false)
const connectError = ref('')
const uptime = ref('--')
let uptimeTimer: ReturnType<typeof setInterval> | null = null

const bypassRu = ref(true)
const showImport = ref(false)
const importUrl = ref('')
const importGroupName = ref('')

const circum = 2 * Math.PI * 96

const statusText = computed(() => {
  switch (vpn.connectionState) {
    case 'connecting': return t('dashboard.connecting')
    case 'connected': return t('dashboard.connected')
    case 'disconnecting': return t('dashboard.disconnecting')
    default: return t('dashboard.disconnected')
  }
})

const gradColors = computed(() => {
  switch (vpn.connectionState) {
    case 'connected': return ['#00e5c8', '#00c6ff', '#7b2ff7']
    case 'connecting':
    case 'disconnecting': return ['#f59e0b', '#ef4444', '#7b2ff7']
    default: return ['#3b82f6', '#7b2ff7', '#3b82f6']
  }
})

const ringOffset = computed(() => vpn.isTransitioning ? circum * 0.25 : 0)

const statusMsg = computed(() => {
  if (connectError.value) {
    return {
      type: 'error',
      icon: '<svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><circle cx="8" cy="8" r="7"/><line x1="8" y1="5" x2="8" y2="8.5"/><circle cx="8" cy="11" r="0.5" fill="currentColor"/></svg>',
      title: t('dashboard.connect_error'),
      desc: connectError.value,
    }
  }
  if (verifying.value) {
    return {
      type: 'checking',
      icon: '<svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><circle cx="8" cy="8" r="7"/><path d="M8 4v4l2.5 1.5"/></svg>',
      title: t('dashboard.verifying'),
      desc: t('dashboard.verifying_desc'),
    }
  }
  if (vpn.connectionState === 'connected' && verified.value === false) {
    return {
      type: 'warning',
      icon: '<svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><path d="M8 1.5L1.5 13h13L8 1.5z"/><line x1="8" y1="6.5" x2="8" y2="9.5"/><circle cx="8" cy="11.5" r="0.5" fill="currentColor"/></svg>',
      title: t('dashboard.no_internet'),
      desc: t('dashboard.no_internet_desc'),
    }
  }
  if (vpn.connectionState === 'connected' && verified.value === true) {
    return {
      type: 'ok',
      icon: '<svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><circle cx="8" cy="8" r="7"/><path d="M5 8l2 2 4-4"/></svg>',
      title: t('dashboard.verified'),
      desc: t('dashboard.verified_desc'),
    }
  }
  if (vpn.connectionState === 'disconnected' && !nodesStore.selectedNodeId) {
    return {
      type: 'info',
      icon: '<svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><circle cx="8" cy="8" r="7"/><line x1="8" y1="5" x2="8" y2="8.5"/><circle cx="8" cy="11" r="0.5" fill="currentColor"/></svg>',
      title: t('dashboard.select_node'),
      desc: t('dashboard.select_node_desc'),
    }
  }
  return null
})

const dlDisplay = computed(() => {
  if (vpn.connectionState !== 'connected') return '--'
  return fmtSpeed(vpn.status.download_speed)
})
const ulDisplay = computed(() => {
  if (vpn.connectionState !== 'connected') return '--'
  return fmtSpeed(vpn.status.upload_speed)
})

function fmtSpeed(value: number) {
  if (!Number.isFinite(value) || value <= 0) return '--'
  if (value >= 1000) return `${(value / 1000).toFixed(2)} ${t('common.Gbps')}`
  return `${value.toFixed(value >= 100 ? 0 : 1)} ${t('common.Mbps')}`
}

function getErrorMessage(error: unknown, fallback: string) {
  if (typeof error === 'string') return error
  if (error instanceof Error && error.message) return error.message
  return fallback
}
async function toggleBypassRu() {
  bypassRu.value = !bypassRu.value
  try { await invoke('update_settings', { settings: { bypass_ru: bypassRu.value } }) } catch (error) { toast.error(getErrorMessage(error, t('toast.settings_save_failed'))) }
  // If connected, reconnect to apply new routing rules
  if (vpn.connectionState === 'connected' && nodesStore.selectedNodeId) {
    vpn.setConnectionState('disconnecting')
    try { await invoke('vpn_disconnect') } catch (error) { toast.error(getErrorMessage(error, t('toast.disconnect_failed'))) }
    vpn.setConnectionState('disconnected')
    // Small delay then reconnect
    await new Promise(r => setTimeout(r, 500))
    vpn.setConnectionState('connecting')
    try {
      await Promise.race([
        invoke('vpn_connect', { nodeId: nodesStore.selectedNodeId }),
        new Promise((_, rej) => setTimeout(() => rej(new Error('Timeout')), 20000)),
      ])
      vpn.setConnectionState('connected')
            vpn.setStatus({ ...vpn.status, connected: true, current_node_id: nodesStore.selectedNodeId, current_node_name: vpn.status.current_node_name })
      verified.value = null
      verifying.value = true
      try { verified.value = await invoke<boolean>('verify_connection') } catch { verified.value = false }
      verifying.value = false
    } catch {
      vpn.setConnectionState('disconnected')
    }
  }
}

async function doImport() {
  if (!importUrl.value) return
  try {
    const nodes = await importSubscription(importUrl.value, importGroupName.value || undefined)
    nodesStore.setNodes(nodes)
    showImport.value = false
    importUrl.value = ''
    importGroupName.value = ''
  } catch (error) {
    toast.error(getErrorMessage(error, t('toast.nodes_import_failed')))
  }
}

async function loadBypassRu() {
  try {
    const s = await invoke<Record<string, boolean | string>>('get_settings')
    if (s.bypass_ru !== undefined) {
      bypassRu.value = typeof s.bypass_ru === 'boolean' ? s.bypass_ru : s.bypass_ru === 'true' || s.bypass_ru === true
    }
  } catch (error) { toast.error(getErrorMessage(error, t('toast.settings_load_failed'))) }
}

function updateUptime() {
  if (vpn.connectionState === 'connected' && vpn.status.connected_at) {
    const s = Math.floor(Date.now() / 1000 - vpn.status.connected_at)
    uptime.value = `${Math.floor(s / 3600)}${t('common.h')} ${Math.floor((s % 3600) / 60)}${t('common.m')}`
  } else {
    uptime.value = '--'
  }
}

async function handleToggle() {
  if (vpn.isTransitioning) return

  if (vpn.connectionState === 'connected') {
    verified.value = null
    verifying.value = false
    connectError.value = ''
    vpn.setConnectionState('disconnecting')
    try { await invoke('vpn_disconnect') } catch (error) { toast.error(getErrorMessage(error, t('toast.disconnect_failed'))) }
    vpn.setConnectionState('disconnected')
    return
  }

  const nodeId = nodesStore.selectedNodeId
  if (!nodeId) return

  connectError.value = ''
  vpn.setConnectionState('connecting')

  let ok = false
  try {
    await Promise.race([
      invoke('vpn_connect', { nodeId }),
      new Promise((_, rej) => setTimeout(() => rej(new Error('Timeout')), 20000)),
    ])
    ok = true
  } catch (error) {
    const msg = getErrorMessage(error, t('dashboard.connect_error'))
    if (msg.includes('tun') || msg.includes('TUN') || msg.includes('adapter') || msg.includes('permission') || msg.includes('Access denied')) {
      connectError.value = t('dashboard.tun_error')
    } else {
      connectError.value = msg
    }
  }

  if (!ok) { vpn.setConnectionState('disconnected'); return }

  vpn.setConnectionState('connected')
    vpn.setStatus({ ...vpn.status, connected: true, current_node_id: nodeId, current_node_name: vpn.status.current_node_name })

  verifying.value = true
  try { verified.value = await invoke<boolean>('verify_connection') } catch { verified.value = false }
  verifying.value = false
}

onMounted(() => { uptimeTimer = setInterval(updateUptime, 2000); updateUptime(); loadBypassRu() })
onUnmounted(() => { if (uptimeTimer) clearInterval(uptimeTimer) })
</script>

<style scoped>
.dashboard-page{height:100%;display:flex;align-items:center;justify-content:center;position:relative;overflow:hidden;background:linear-gradient(135deg,rgba(4,10,20,0.96),rgba(6,16,24,0.98) 46%,rgba(10,7,22,0.96))}
.dashboard-page::before{content:'';position:absolute;inset:0;background:linear-gradient(120deg,transparent 0 22%,rgba(125,247,104,0.035) 22% 23%,transparent 23% 48%,rgba(46,231,205,0.04) 48% 49%,transparent 49% 74%,rgba(122,90,255,0.035) 74% 75%,transparent 75%);opacity:.85;animation:bg-shift 14s linear infinite}
.dashboard-page.connected .power-disc{box-shadow:0 0 40px rgba(125,247,104,0.14)}
.clouds-container{position:absolute;inset:0;pointer-events:none;z-index:0;overflow:hidden}
.cloud{
  position:absolute;
  width:120px;height:40px;
  background:radial-gradient(ellipse,rgba(0,229,200,0.06),transparent 70%);
  border-radius:50%;
  filter:blur(20px);
  animation:cloud-drift linear infinite;
}
html.light .cloud{background:radial-gradient(ellipse,rgba(0,153,122,0.04),transparent 70%)}
.c1{width:140px;height:50px;top:15%;animation-duration:25s;animation-delay:0s}
.c2{width:100px;height:35px;top:35%;animation-duration:30s;animation-delay:-5s}
.c3{width:160px;height:55px;top:55%;animation-duration:22s;animation-delay:-10s}
.c4{width:90px;height:30px;top:70%;animation-duration:28s;animation-delay:-15s}
.c5{width:130px;height:45px;top:85%;animation-duration:32s;animation-delay:-8s}
@keyframes cloud-drift{
  0%{transform:translateX(-200px);opacity:0}
  10%{opacity:1}
  90%{opacity:1}
  100%{transform:translateX(calc(100vw + 200px));opacity:0}
}
.dash-bg{position:absolute;inset:-20%;pointer-events:none;transition:background 1.5s ease;border-radius:50%}
.dash-bg.disconnected{background:radial-gradient(ellipse,rgba(123,47,247,0.06),transparent 70%)}
.dash-bg.connecting{background:radial-gradient(ellipse,rgba(245,158,11,0.05),transparent 70%)}
.dash-bg.connected{background:radial-gradient(ellipse,rgba(0,229,200,0.08),transparent 70%)}
.dash-bg.disconnecting{background:radial-gradient(ellipse,rgba(239,68,68,0.04),transparent 70%)}

.dash-center{position:relative;z-index:1;display:flex;flex-direction:column;align-items:center;gap:2px;max-width:380px;width:100%;padding:20px}
.dashboard-brand{display:flex;flex-direction:column;align-items:center;margin-bottom:8px;animation:brand-in .55s ease both}
.dashboard-brand-mark{width:54px;height:54px;object-fit:contain;filter:drop-shadow(0 10px 24px rgba(46,231,205,0.22));animation:mark-breathe 4s ease-in-out infinite}
.dashboard-brand-name{margin:2px 0 0;font-size:32px;line-height:1;font-weight:800;letter-spacing:0;background:linear-gradient(90deg,#9bff6f,#37efd3 42%,#5ba8ff 70%,#a56fff);-webkit-background-clip:text;background-clip:text;color:transparent;text-shadow:0 0 28px rgba(46,231,205,0.08)}
.chroma-track{position:relative;width:190px;height:7px;margin-top:10px;display:grid;grid-template-columns:1fr 1fr 1fr;gap:6px;overflow:hidden}
.chroma-track span{height:2px;border-radius:999px;background:linear-gradient(90deg,transparent,#7df768,#2ee7cd,#7a5aff,transparent);animation:chroma-slide 2.8s ease-in-out infinite}
.chroma-track span:nth-child(2){animation-delay:.22s;opacity:.72}.chroma-track span:nth-child(3){animation-delay:.44s;opacity:.5}
.dash-status{font-size:11px;font-weight:600;letter-spacing:2px;text-transform:uppercase;color:rgba(255,255,255,0.34);margin-bottom:8px}

/* === POWER BUTTON === */
.power-wrap{position:relative;width:214px;height:214px;margin:8px 0;display:flex;align-items:center;justify-content:center}
.power-chameleon{position:absolute;width:146px;height:146px;object-fit:contain;opacity:.13;filter:saturate(1.25) blur(.2px);animation:chameleon-drift 8s ease-in-out infinite;pointer-events:none}
.chameleon-scan{position:absolute;width:190px;height:190px;border-radius:50%;background:conic-gradient(from 140deg,transparent 0 10%,rgba(125,247,104,0.28) 18%,rgba(46,231,205,0.16) 32%,transparent 44%,rgba(91,168,255,0.18) 56%,rgba(165,111,255,0.24) 72%,transparent 88%);mask:radial-gradient(circle,transparent 57%,#000 60%,#000 63%,transparent 67%);animation:scan-turn 13s linear infinite;opacity:.76;filter:drop-shadow(0 0 18px rgba(46,231,205,.15))}
.aurora-orbit{position:absolute;inset:-4px;border-radius:50%;pointer-events:none;background:conic-gradient(from 210deg,transparent 0 8%,rgba(155,255,111,.42) 16%,rgba(55,239,211,.30) 29%,transparent 42%,rgba(91,168,255,.24) 56%,rgba(165,111,255,.34) 72%,transparent 88%);mask:radial-gradient(circle,transparent 63%,#000 66%,#000 70%,transparent 74%);filter:blur(.45px) drop-shadow(0 0 18px rgba(46,231,205,.20));animation:aurora-turn 15s cubic-bezier(.4,0,.2,1) infinite;opacity:.84}
.aurora-orbit.connected{background:conic-gradient(from 210deg,transparent 0 8%,rgba(155,255,111,.50) 16%,rgba(46,231,205,.36) 31%,transparent 43%,rgba(91,168,255,.26) 57%,rgba(165,111,255,.30) 74%,transparent 90%);filter:blur(.45px) drop-shadow(0 0 22px rgba(125,247,104,.22))}.aurora-orbit.connecting,.aurora-orbit.disconnecting{background:conic-gradient(from 210deg,transparent 0 8%,rgba(255,209,102,.44) 16%,rgba(255,91,125,.28) 31%,transparent 44%,rgba(91,168,255,.22) 58%,rgba(165,111,255,.28) 74%,transparent 90%)}

.power-glow{
  position:absolute;inset:-10px;border-radius:50%;
  transition:box-shadow 1s ease,opacity 1s ease;opacity:0.3;
}
.power-glow.disconnected{box-shadow:0 0 60px 10px rgba(123,47,247,0.15)}
.power-glow.connecting{box-shadow:0 0 80px 15px rgba(245,158,11,0.2);opacity:0.5}
.power-glow.connected{box-shadow:0 0 80px 15px rgba(0,229,200,0.2);opacity:0.6}
.power-glow.disconnecting{box-shadow:0 0 60px 10px rgba(239,68,68,0.15)}

.power-svg{width:200px;height:200px;position:absolute;top:7px;left:7px}
.power-svg.spinning{animation:spin 2.5s linear infinite;transform-origin:center}

.power-disc{
  position:absolute;inset:18px;border-radius:50%;
  display:flex;align-items:center;justify-content:center;
  background:radial-gradient(circle at 38% 32%,#1a1626,#100e18,#0a0910);
  transition:box-shadow .6s ease,background .6s ease;
}
.power-disc::before{
  content:'';position:absolute;inset:0;border-radius:50%;
  border:1px solid rgba(255,255,255,0.04);
}
.power-disc::after{
  content:'';position:absolute;inset:8px;border-radius:50%;
  border:1px solid rgba(255,255,255,0.02);
}
.disc-ring{
  position:absolute;inset:-3px;border-radius:50%;
  border:2px solid transparent;
  transition:border-color .6s ease,box-shadow .6s ease;
}
.power-disc.connected .disc-ring{
  border-color:rgba(0,229,200,0.2);
  box-shadow:inset 0 0 30px rgba(0,229,200,0.05),0 0 20px rgba(0,229,200,0.05);
}
.power-disc.connecting .disc-ring,.power-disc.disconnecting .disc-ring{
  border-color:rgba(245,158,11,0.15);
  box-shadow:inset 0 0 20px rgba(245,158,11,0.03);
}

.power-btn{
  position:relative;z-index:2;
  background:none;border:none;cursor:pointer;padding:0;
  transition:transform .2s cubic-bezier(.4,0,.2,1),filter .3s;
  filter:drop-shadow(0 0 0px transparent);
}
.power-btn:hover{transform:scale(1.1);filter:drop-shadow(0 0 12px rgba(123,47,247,0.3))}
.power-disc.connected .power-btn:hover{filter:drop-shadow(0 0 12px rgba(0,229,200,0.4))}
.power-btn:active{transform:scale(0.92)}
.power-btn:disabled{opacity:.5;cursor:default;transform:none;filter:none}

.glow-bar{width:120px;height:16px;border-radius:50%;filter:blur(16px);transition:background 1s ease;margin:-8px 0 4px}
.glow-bar.disconnected{background:rgba(123,47,247,0.1)}
.glow-bar.connecting{background:rgba(245,158,11,0.08)}
.glow-bar.connected{background:rgba(0,229,200,0.12)}
.glow-bar.disconnecting{background:rgba(239,68,68,0.08)}

/* Traffic */
.traffic-row{display:flex;align-items:center;gap:24px;margin:4px 0}
.traffic-col{display:flex;flex-direction:column;align-items:center;gap:1px}
.traffic-icon{width:24px;height:24px;border-radius:6px;display:flex;align-items:center;justify-content:center;font-size:12px;font-weight:700}
.traffic-icon.down{background:rgba(239,68,68,0.08);color:#ef4444}
.traffic-icon.up{background:rgba(0,200,160,0.08);color:#00c8a0}
.traffic-val{font-size:15px;font-weight:500;color:rgba(255,255,255,0.75);font-variant-numeric:tabular-nums;min-width:60px;text-align:center}
.traffic-lbl{font-size:9px;color:rgba(255,255,255,0.2)}
.traffic-sep{width:1px;height:30px;background:rgba(255,255,255,0.05)}

/* Info cards */
.info-cards{display:flex;gap:6px;width:100%;margin-top:6px}
.info-card{flex:1;padding:8px 10px;border-radius:8px;background:rgba(255,255,255,0.02);border:1px solid rgba(255,255,255,0.03);text-align:center}
.info-lbl{display:block;font-size:9px;color:rgba(255,255,255,0.2);margin-bottom:2px}
.info-val{font-size:12px;color:rgba(255,255,255,0.55);font-variant-numeric:tabular-nums}
.truncate{white-space:nowrap;overflow:hidden;text-overflow:ellipsis}

/* Status Message */
.status-msg{
  display:flex;align-items:flex-start;gap:8px;width:100%;
  padding:10px 14px;border-radius:8px;margin-top:4px;
}
.status-msg.error{background:rgba(239,68,68,0.06);border:1px solid rgba(239,68,68,0.1)}
.status-msg.warning{background:rgba(245,158,11,0.06);border:1px solid rgba(245,158,11,0.1)}
.status-msg.ok{background:rgba(0,229,200,0.05);border:1px solid rgba(0,229,200,0.08)}
.status-msg.checking{background:rgba(59,130,246,0.05);border:1px solid rgba(59,130,246,0.08)}
.status-msg.info{background:rgba(123,47,247,0.05);border:1px solid rgba(123,47,247,0.08)}
.status-msg-icon{flex-shrink:0;margin-top:1px}
.status-msg.error .status-msg-icon{color:rgba(239,68,68,0.6)}
.status-msg.warning .status-msg-icon{color:rgba(245,158,11,0.6)}
.status-msg.ok .status-msg-icon{color:rgba(0,229,200,0.6)}
.status-msg.checking .status-msg-icon{color:rgba(59,130,246,0.6)}
.status-msg.info .status-msg-icon{color:rgba(123,47,247,0.5)}
.status-msg-text{display:flex;flex-direction:column;gap:1px}
.status-msg-title{font-size:11px;font-weight:600}
.status-msg.error .status-msg-title{color:rgba(239,68,68,0.7)}
.status-msg.warning .status-msg-title{color:rgba(245,158,11,0.7)}
.status-msg.ok .status-msg-title{color:rgba(0,229,200,0.7)}
.status-msg.checking .status-msg-title{color:rgba(59,130,246,0.7)}
.status-msg.info .status-msg-title{color:rgba(123,47,247,0.6)}
.status-msg-desc{font-size:10px;color:rgba(255,255,255,0.25);line-height:1.4}

.err-box{margin-top:6px;padding:8px 14px;border-radius:8px;font-size:11px;background:rgba(239,68,68,0.06);color:rgba(239,68,68,0.7);text-align:center;width:100%}

.quick-actions{display:flex;gap:8px;margin-top:8px;width:100%}
.qa-btn{flex:1;display:flex;align-items:center;justify-content:center;gap:6px;padding:8px 12px;border-radius:8px;font-size:11px;font-weight:500;background:rgba(255,255,255,0.025);border:1px solid rgba(255,255,255,0.05);color:rgba(255,255,255,0.4);cursor:pointer;transition:all .2s}
.qa-btn:hover{background:rgba(255,255,255,0.05);color:rgba(255,255,255,0.6);border-color:rgba(255,255,255,0.08)}
.qa-btn.active{background:rgba(0,229,200,0.06);border-color:rgba(0,229,200,0.12);color:rgba(0,229,200,0.7)}
.qa-indicator{width:6px;height:6px;border-radius:50%;background:rgba(255,255,255,0.12);transition:all .2s}
.qa-indicator.on{background:#00e5c8;box-shadow:0 0 6px rgba(0,229,200,0.4)}

.import-overlay{position:fixed;inset:0;z-index:100;display:flex;align-items:center;justify-content:center;background:rgba(0,0,0,0.6);backdrop-filter:blur(4px)}
.import-panel{background:#0f0d16;border:1px solid rgba(255,255,255,0.08);border-radius:12px;padding:20px;width:100%;max-width:360px}
.import-title{font-size:14px;font-weight:600;color:rgba(255,255,255,0.8);margin-bottom:14px}
.import-field{margin-bottom:12px}
.import-field label{display:block;font-size:9px;font-weight:600;text-transform:uppercase;letter-spacing:.8px;color:rgba(255,255,255,0.22);margin-bottom:4px}
.import-input{width:100%;padding:8px 10px;border-radius:6px;font-size:12px;background:rgba(255,255,255,0.03);border:1px solid rgba(255,255,255,0.06);color:rgba(255,255,255,0.7);outline:none}
.import-input:focus{border-color:rgba(0,229,200,0.25)}
.import-input::placeholder{color:rgba(255,255,255,0.12)}
.import-actions{display:flex;gap:6px;margin-top:14px;justify-content:flex-end}
.import-btn{padding:6px 14px;border-radius:6px;font-size:11px;font-weight:500;border:1px solid;cursor:pointer;transition:all .2s}
.import-btn.cancel{background:rgba(255,255,255,0.03);border-color:rgba(255,255,255,0.07);color:rgba(255,255,255,0.45)}
.import-btn.primary{background:rgba(0,229,200,0.1);border-color:rgba(0,229,200,0.18);color:#00e5c8}
.import-btn:disabled{opacity:.35;cursor:default}

.proxy-hint{display:flex;align-items:center;gap:6px;font-size:10px;color:rgba(255,255,255,0.25);padding:6px 12px;border-radius:6px;background:rgba(255,255,255,0.02);border:1px solid rgba(255,255,255,0.03);width:100%;text-align:center;justify-content:center}
.proxy-hint b{color:rgba(0,229,200,0.6);font-weight:500}
.proxy-hint-icon{font-size:12px;opacity:0.4}

html.light .dashboard-page {
  background:
    radial-gradient(circle at 18% 12%, rgba(58,191,103,0.18), transparent 30%),
    radial-gradient(circle at 82% 18%, rgba(46,158,177,0.14), transparent 28%),
    radial-gradient(circle at 52% 92%, rgba(122,90,255,0.09), transparent 34%),
    linear-gradient(135deg, #f8fcff, #effbf6 48%, #f7f3ff);
}
html.light .dashboard-page::before {
  background: linear-gradient(120deg, transparent 0 22%, rgba(58,191,103,0.085) 22% 23%, transparent 23% 48%, rgba(46,158,177,0.075) 48% 49%, transparent 49% 74%, rgba(122,90,255,0.055) 74% 75%, transparent 75%);
  opacity: .72;
}
html.light .dashboard-brand-mark { filter: drop-shadow(0 12px 26px rgba(46,158,177,0.22)); }
html.light .dashboard-brand-name {
  background: linear-gradient(90deg, #258151, #16a99b 42%, #297ac1 70%, #7b4bd8);
  -webkit-background-clip: text;
  background-clip: text;
  text-shadow: 0 8px 28px rgba(46,158,177,0.10);
}
html.light .chroma-track span { background: linear-gradient(90deg, transparent, #2cae5b, #20bdb0, #7b4bd8, transparent); }
html.light .power-chameleon {
  opacity: .18;
  filter: saturate(1.08) drop-shadow(0 16px 26px rgba(46,158,177,0.16));
}
html.light .chameleon-scan {
  background: conic-gradient(from 140deg, transparent 0 10%, rgba(58,191,103,0.28) 18%, rgba(46,158,177,0.18) 32%, transparent 44%, rgba(41,122,193,0.16) 56%, rgba(122,90,255,0.20) 72%, transparent 88%);
  opacity: .68;
}
html.light .aurora-orbit {
  background: conic-gradient(from 210deg, transparent 0 8%, rgba(58,191,103,.36) 16%, rgba(46,158,177,.28) 31%, transparent 43%, rgba(41,122,193,.22) 57%, rgba(122,90,255,.20) 74%, transparent 90%);
  filter: blur(.4px) drop-shadow(0 0 16px rgba(46,158,177,.16));
}
html.light .power-glow.disconnected { box-shadow: 0 0 62px 10px rgba(122,90,255,0.10); }
html.light .power-glow.connected { box-shadow: 0 0 86px 15px rgba(46,158,177,0.18); }
html.light .power-glow.connecting { box-shadow: 0 0 80px 15px rgba(236,157,22,0.14); }
html.light .power-glow.disconnecting { box-shadow: 0 0 62px 10px rgba(205,54,67,0.12); }
html.light .traffic-icon.down { background: rgba(205,54,67,0.08); color: #c93546; }
html.light .traffic-icon.up { background: rgba(32,189,176,0.10); color: #159b90; }
html.light .status-msg {
  background: rgba(255,255,255,0.58);
  border-color: rgba(42,109,120,0.11);
  box-shadow: 0 10px 28px rgba(39,75,86,0.08);
}
html.light .quick-actions .qa-btn { box-shadow: 0 10px 24px rgba(39,75,86,0.07); }
html.light .import-overlay { background: rgba(233,246,245,0.62); backdrop-filter: blur(8px); }
html.light .import-panel {
  background: linear-gradient(180deg, rgba(255,255,255,0.96), rgba(244,250,248,0.94));
  border-color: rgba(42,109,120,0.14);
  box-shadow: 0 24px 54px rgba(39,75,86,0.18);
}
html.light .import-title { color: rgba(16,33,58,0.84); }
html.light .import-field label { color: rgba(16,48,60,0.52); }
html.light .import-input {
  background: rgba(255,255,255,0.78);
  border-color: rgba(42,109,120,0.14);
  color: rgba(16,33,58,0.78);
}
html.light .import-input:focus { border-color: rgba(58,191,103,0.30); box-shadow: 0 0 0 3px rgba(58,191,103,0.10); }
html.light .import-input::placeholder { color: rgba(16,48,60,0.34); }
html.light .import-btn.cancel {
  background: rgba(255,255,255,0.70);
  border-color: rgba(42,109,120,0.12);
  color: rgba(16,48,60,0.62);
}
html.light .import-btn.primary {
  background: linear-gradient(90deg, rgba(58,191,103,0.14), rgba(46,158,177,0.10));
  border-color: rgba(58,191,103,0.24);
  color: #207645;
}
@keyframes spin{from{transform:rotate(0deg)}to{transform:rotate(360deg)}}
@keyframes bg-shift{0%{transform:translateX(-3%)}50%{transform:translateX(3%)}100%{transform:translateX(-3%)}}
@keyframes brand-in{from{opacity:0;transform:translateY(8px)}to{opacity:1;transform:translateY(0)}}
@keyframes mark-breathe{0%,100%{transform:translateY(0) scale(1)}50%{transform:translateY(-3px) scale(1.03)}}
@keyframes chroma-slide{0%,100%{transform:translateX(-22%);opacity:.34}50%{transform:translateX(22%);opacity:1}}
@keyframes chameleon-drift{0%,100%{transform:translateY(0) rotate(-3deg)}50%{transform:translateY(-5px) rotate(3deg)}}
@keyframes scan-turn{0%{transform:rotate(0deg) scale(.99);opacity:.62}50%{transform:rotate(180deg) scale(1.015);opacity:.82}100%{transform:rotate(360deg) scale(.99);opacity:.62}}
@keyframes aurora-turn{0%{transform:rotate(0deg) scale(.99);opacity:.62}50%{transform:rotate(180deg) scale(1.025);opacity:.86}100%{transform:rotate(360deg) scale(.99);opacity:.62}}
@keyframes pip-pulse{0%,100%{opacity:1}50%{opacity:.3}}
</style>






