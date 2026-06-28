<template>
  <div class="app-root" :class="{ 'sidebar-expanded': !collapsed }">
    <aside class="sidebar-left" :class="{ collapsed }">
      <div class="sl-inner">
        <button class="sl-collapse-edge" @click="collapsed = !collapsed" :title="collapsed ? t('sidebar.expand') : t('sidebar.collapse')">
          <svg width="15" height="15" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
            <rect x="2.5" y="2.5" width="11" height="11" rx="2.5"/>
            <path v-if="!collapsed" d="M7 5.5L4.8 8 7 10.5"/>
            <path v-else d="M9 5.5L11.2 8 9 10.5"/>
            <path d="M8 3v10"/>
          </svg>
        </button>

        <div class="sl-brand">
          <div class="brand-mark-wrap">
            <span class="brand-halo" />
            <img src="/branding/chameleon-mark.png" :alt="appName" class="brand-mark" />
          </div>
          <div v-show="!collapsed" class="brand-text">
            <p class="brand-name">{{ appName }}</p>
          </div>
        </div>

        <nav class="sl-nav">
          <NuxtLink v-for="item in navItems" :key="item.path" :to="item.path" class="sl-nav-item" :class="{ active: route.path === item.path }">
            <span class="sl-nav-icon" v-html="item.icon"/>
            <span v-show="!collapsed" class="sl-nav-label">{{ t(item.labelKey) }}</span>
          </NuxtLink>
        </nav>

        <div class="sl-footer">
          <button @click="handleThemeToggle" class="theme-btn" :title="isLight ? t('settings.theme_dark') : t('settings.theme_light')">
            <svg v-if="isLight" width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"><circle cx="8" cy="8" r="3.5"/><path d="M8 1.5v2M8 12.5v2M1.5 8h2M12.5 8h2M3.3 3.3l1.4 1.4M11.3 11.3l1.4 1.4M3.3 12.7l1.4-1.4M11.3 4.7l1.4-1.4"/></svg>
            <svg v-else width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"><path d="M13.5 9.5a5.5 5.5 0 01-7-7 5.5 5.5 0 107 7z"/></svg>
          </button>
          <button @click="toggleLocale" class="lang-btn" :title="locale === 'en' ? 'English' : 'Русский'">
            {{ locale === 'en' ? 'EN' : 'RU' }}
          </button>
        </div>
      </div>
    </aside>

    <main class="main-area">
      <slot />
    </main>

    <aside class="sidebar-right">
      <div class="sr-inner">
        <div class="sr-header">
          <span class="sr-title">{{ t('sidebar.servers') }}</span>
          <span class="sr-count" v-if="nodesStore.nodes.length">{{ nodesStore.nodes.length }}</span>
        </div>

        <div class="sr-search">
          <svg width="13" height="13" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><circle cx="6" cy="6" r="4.5"/><line x1="9.5" y1="9.5" x2="13" y2="13"/></svg>
          <input v-model="searchQuery" type="text" :placeholder="t('nodes.search')" class="sr-search-input"/>
        </div>

        <div class="sr-scroll">
          <div v-if="groupedServers.length === 0 && !nodesStore.loading" class="sr-empty">{{ t('sidebar.no_nodes') }}</div>

          <template v-for="group in groupedServers" :key="group.name">
            <div class="sr-group" :class="{ open: !collapsedGroups[group.name] }" @click="toggleGroup(group.name)">
              <svg class="sr-group-arrow" width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><path d="M3 4l2 2 2-2"/></svg>
              <span class="sr-group-name">{{ group.name }}</span>
              <span class="sr-group-count">{{ group.nodes.length }}</span>
            </div>
            <transition name="slide">
              <div v-show="!collapsedGroups[group.name]" class="sr-group-items">
                <div v-for="node in group.nodes" :key="node.id" class="sr-node" :class="{ active: nodesStore.selectedNodeId === node.id, connected: isNodeConnected(node.id) }">
                  <div class="sr-node-main" @click="selectNode(node)">
                    <span class="sr-dot" :class="nodeHealthClass(node)"/>
                    <div class="sr-node-info">
                      <span class="sr-node-name">{{ node.name }}</span>
                      <span class="sr-node-meta">
                        {{ node.protocol.toUpperCase() }}
                        <template v-if="node.test_status === 'testing'"> · {{ t('nodes.testing') }}</template>
                        <template v-else-if="node.test_status === 'failed'"> · {{ t('nodes.unavailable') }}</template>
                        <template v-else-if="node.latency_ms != null"> · {{ node.latency_ms.toFixed(0) }}{{ t('common.ms') }}</template>
                        <template v-else> · {{ t('nodes.untested') }}</template>
                      </span>
                    </div>
                  </div>
                  <button class="sr-node-btn" :class="{ connected: isNodeConnected(node.id) }" @click.stop="toggleConnection(node)" :disabled="vpn.isTransitioning">
                    <svg v-if="isNodeConnected(node.id)" width="8" height="8" viewBox="0 0 8 8" fill="currentColor"><rect width="8" height="8" rx="2"/></svg>
                    <svg v-else width="8" height="8" viewBox="0 0 8 8" fill="currentColor"><polygon points="1,0 8,4 1,8"/></svg>
                  </button>
                </div>
              </div>
            </transition>
          </template>
        </div>

        <div v-if="nodesStore.testingAll" class="sr-progress">
          <div class="sr-progress-bar"><div class="sr-progress-fill" :style="{ width: progressPct + '%' }"></div></div>
          <span class="sr-progress-text">{{ nodesStore.testProgress.current }}/{{ nodesStore.testProgress.total }}</span>
        </div>

        <div class="sr-footer">
          <button class="sr-test-btn" @click="testAllNodes" :disabled="nodesStore.testingAll">
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><polygon points="2 1 10 6 2 11 2 1"/></svg>
            {{ nodesStore.testingAll ? t('nodes.testing') : t('nodes.test_all') }}
          </button>
        </div>
      </div>
    </aside>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch, onMounted, onUnmounted } from 'vue'
import { useVpnStore } from '~/stores/vpn'
import { useNodesStore, type VpnNode } from '~/stores/nodes'
import { useI18n } from '~/composables/useI18n'
import { useTauri } from '~/composables/useTauri'
import { useNodes } from '~/composables/useNodes'
import { useTheme } from '~/composables/useTheme'

defineEmits(['openAddNode', 'openImport'])

type VpnSnapshot = {
  connected: boolean
  current_node_id: string | null
  current_node_name: string | null
}

const vpn = useVpnStore()
const nodesStore = useNodesStore()
const { t, locale, toggleLocale } = useI18n()
const route = useRoute()
const { invoke, listen } = useTauri()
const { testAll, setupListeners } = useNodes()

const collapsed = ref(false)
const searchQuery = ref('')
const collapsedGroups = ref<Record<string, boolean>>({})
const { isLight, toggleTheme, initTheme } = useTheme()
const appName = computed(() => t('app.name'))

async function handleThemeToggle() {
  toggleTheme()
  await invoke('update_settings', { settings: { theme: isLight.value ? 'light' : 'dark' } }).catch(() => {})
}

onMounted(() => initTheme())
onMounted(() => {
  collapsed.value = localStorage.getItem('vpn_sidebar_collapsed') === 'true'
})
watch(collapsed, (value) => localStorage.setItem('vpn_sidebar_collapsed', String(value)))


const progressPct = computed(() => {
  const { current, total } = nodesStore.testProgress
  return total > 0 ? (current / total) * 100 : 0
})

function isNodeConnected(nodeId: string) {
  return vpn.status.current_node_id === nodeId && vpn.connectionState === 'connected'
}

const groupedServers = computed(() => {
  let nodes = nodesStore.nodes
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    nodes = nodes.filter((node) => node.name.toLowerCase().includes(query) || node.server.toLowerCase().includes(query))
  }
  const groups: Record<string, VpnNode[]> = {}
  for (const node of nodes) {
    let name = 'Default'
    if (node.subscription_id) {
      const sub = nodesStore.subscriptions.find((item) => item.id === node.subscription_id)
      name = sub?.name || sub?.group_name || 'Subscription'
    } else if (node.group_name?.trim()) {
      name = node.group_name.trim()
    }
    if (!groups[name]) groups[name] = []
    groups[name].push(node)
  }
  return Object.entries(groups)
    .map(([name, groupedNodes]) => ({ name, nodes: groupedNodes }))
    .sort((a, b) => {
      if (a.name === 'Default') return 1
      if (b.name === 'Default') return -1
      return a.name.localeCompare(b.name)
    })
})

function toggleGroup(name: string) {
  collapsedGroups.value[name] = !collapsedGroups.value[name]
}

function nodeHealthClass(node: VpnNode) {
  if (node.test_status === 'testing') return 'testing'
  if (node.test_status === 'failed') return 'failed'
  const ms = node.latency_ms
  if (ms == null) return 'unknown'
  if (ms < 100) return 'fast'
  if (ms < 300) return 'medium'
  return 'slow'
}

function selectNode(node: VpnNode) {
  nodesStore.setSelectedNodeId(node.id)
}

function makeDisconnectedSnapshot(): VpnSnapshot {
  return { connected: false, current_node_id: null, current_node_name: null }
}

async function toggleConnection(node: VpnNode) {
  if (vpn.isTransitioning) return
  if (isNodeConnected(node.id)) {
    vpn.setConnectionState('disconnecting')
    try { await invoke('vpn_disconnect') } catch {}
    vpn.setConnectionState('disconnected')
    vpn.setStatus({ ...vpn.status, ...makeDisconnectedSnapshot() })
    return
  }
  nodesStore.setSelectedNodeId(node.id)
  vpn.setConnectionState('connecting')
  try {
    await Promise.race([
      invoke('vpn_connect', { nodeId: node.id }),
      new Promise((_, reject) => setTimeout(() => reject(new Error('Timeout')), 20000)),
    ])
    vpn.setConnectionState('connected')
    vpn.startTrafficSimulation()
    vpn.setStatus({ ...vpn.status, connected: true, current_node_id: node.id, current_node_name: node.name })
  } catch {
    vpn.setConnectionState('disconnected')
  }
}

async function testAllNodes() {
  await testAll()
}

let unlisteners: Array<() => void> = []
async function loadData() {
  try { nodesStore.setNodes(await invoke<VpnNode[]>('get_nodes')) } catch {}
  try {
    const status = await invoke<any>('vpn_status')
    vpn.setStatus(status)
    if (status.current_node_id) nodesStore.setSelectedNodeId(status.current_node_id)
  } catch {}
  try { nodesStore.setSubscriptions(await invoke('get_subscriptions')) } catch {}
}

onMounted(async () => {
  await loadData()
  setupListeners()
  const unlistenStatus = await listen<any>('vpn-status', (status) => vpn.setStatus(status))
  const unlistenLog = await listen<any>('vpn-log', (log) => vpn.addLog(log))
  unlisteners.push(unlistenStatus, unlistenLog)
})

onUnmounted(() => {
  unlisteners.forEach((unlisten) => unlisten())
})

const navItems = [
  { path: '/', labelKey: 'sidebar.dashboard', icon: '<svg width="18" height="18" viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="1.5" y="1.5" width="6.5" height="6.5" rx="1.5"/><rect x="10" y="1.5" width="6.5" height="6.5" rx="1.5"/><rect x="1.5" y="10" width="6.5" height="6.5" rx="1.5"/><rect x="10" y="10" width="6.5" height="6.5" rx="1.5"/></svg>' },
  { path: '/nodes', labelKey: 'sidebar.nodes', icon: '<svg width="18" height="18" viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><circle cx="9" cy="5" r="2.5"/><circle cx="5" cy="13" r="2.5"/><circle cx="13" cy="13" r="2.5"/><line x1="7.5" y1="7" x2="6" y2="11"/><line x1="10.5" y1="7" x2="12" y2="11"/></svg>' },
  { path: '/rules', labelKey: 'sidebar.rules', icon: '<svg width="18" height="18" viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><line x1="3" y1="5" x2="15" y2="5"/><line x1="3" y1="9" x2="15" y2="9"/><line x1="3" y1="13" x2="15" y2="13"/></svg>' },
  { path: '/settings', labelKey: 'sidebar.settings', icon: '<svg width="18" height="18" viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 5h10"/><path d="M4 9h10"/><path d="M4 13h10"/><circle cx="7" cy="5" r="1.5" fill="currentColor" stroke="none"/><circle cx="11" cy="9" r="1.5" fill="currentColor" stroke="none"/><circle cx="8.5" cy="13" r="1.5" fill="currentColor" stroke="none"/></svg>' },
]
</script>

<style>
body{overflow:hidden}
.app-root{display:flex;height:100vh;width:100vw;overflow:hidden}
.sidebar-left{position:relative;width:60px;min-width:60px;background:linear-gradient(180deg,rgba(8,17,31,0.96),rgba(7,12,24,0.94));border-right:1px solid var(--border);display:flex;flex-direction:column;user-select:none;z-index:10;transition:width .25s ease,min-width .25s ease;backdrop-filter:blur(20px)}
.sidebar-left:not(.collapsed){width:220px;min-width:220px}
.sl-inner{display:flex;flex-direction:column;height:100%;overflow:hidden}
.sl-collapse-edge{position:absolute;top:18px;right:-13px;width:26px;height:26px;display:flex;align-items:center;justify-content:center;border-radius:9px;background:linear-gradient(180deg,rgba(15,27,48,0.96),rgba(8,15,28,0.96));border:1px solid rgba(126,182,255,0.14);color:var(--text-muted);cursor:pointer;transition:transform .2s ease,color .2s ease,border-color .2s ease,background .2s ease;box-shadow:0 8px 22px rgba(0,0,0,0.24);z-index:20}
.sl-collapse-edge:hover{transform:translateX(1px);color:#9bff6f;border-color:rgba(125,247,104,0.22);background:linear-gradient(180deg,rgba(16,34,42,0.98),rgba(8,18,30,0.98))}
.sidebar-left.collapsed .sl-collapse-edge:hover{transform:translateX(-1px)}
.sl-brand{display:flex;align-items:center;gap:12px;padding:16px 14px 14px;border-bottom:1px solid var(--border)}
.sidebar-left.collapsed .sl-brand{justify-content:center;padding:14px 0}
.brand-mark-wrap{position:relative;width:44px;height:44px;flex-shrink:0}
.brand-halo{position:absolute;inset:-6px;background:radial-gradient(circle, rgba(125,247,104,0.34), rgba(46,231,205,0.18) 45%, rgba(122,90,255,0.14) 68%, transparent 78%);filter:blur(12px)}
.brand-mark{position:relative;z-index:1;width:44px;height:44px;object-fit:contain;display:block;filter:drop-shadow(0 10px 22px rgba(0,0,0,0.28))}
.brand-text{display:flex;flex-direction:column;justify-content:center;min-width:0}
.brand-name{font-size:20px;line-height:1;font-weight:800;letter-spacing:.01em;margin:0;background:linear-gradient(90deg,#9bff6f 0%, #37efd3 42%, #5ba8ff 70%, #a56fff 100%);-webkit-background-clip:text;background-clip:text;color:transparent}

.sl-nav{flex:1;padding:10px 8px;display:flex;flex-direction:column;gap:4px}
.sl-nav-item{display:flex;align-items:center;gap:12px;padding:11px 12px;border-radius:14px;text-decoration:none;color:var(--text-muted);transition:all .2s;font-size:12px;font-weight:600}
.sidebar-left.collapsed .sl-nav-item{justify-content:center;padding:11px 0}
.sl-nav-item:hover{color:var(--text-secondary);background:rgba(125,247,104,0.06)}
.sl-nav-item.active{color:var(--text);background:linear-gradient(90deg, rgba(125,247,104,0.14), rgba(46,231,205,0.10), rgba(93,91,255,0.12));border:1px solid rgba(125,247,104,0.12);box-shadow:inset 0 1px 0 rgba(255,255,255,0.06)}
.sl-nav-icon{flex-shrink:0;display:flex}
.sl-nav-label{white-space:nowrap;overflow:hidden}
.sl-footer{padding:10px 8px 12px;border-top:1px solid var(--border);display:flex;gap:6px;justify-content:center}
.theme-btn,.lang-btn{width:34px;height:30px;border-radius:10px;display:flex;align-items:center;justify-content:center;background:rgba(255,255,255,0.03);border:1px solid rgba(255,255,255,0.06);color:var(--btn-text);cursor:pointer;transition:all .2s;flex-shrink:0}
.theme-btn:hover,.lang-btn:hover{background:rgba(46,231,205,0.10);border-color:rgba(46,231,205,0.16);color:var(--text-secondary)}
.lang-btn{font-size:10px;font-weight:700}
.main-area{flex:1;overflow-y:auto;background:var(--bg);position:relative;min-width:0}
.sidebar-right{width:292px;min-width:292px;background:linear-gradient(180deg,rgba(8,17,31,0.96),rgba(7,12,24,0.94));border-left:1px solid var(--border);display:flex;flex-direction:column;user-select:none;backdrop-filter:blur(20px)}
.sr-inner{display:flex;flex-direction:column;height:100%;overflow:hidden}
.sr-header{display:flex;align-items:center;justify-content:space-between;padding:16px 16px 12px}
.sr-title{font-size:10px;font-weight:700;text-transform:uppercase;letter-spacing:1.5px;color:var(--text-muted)}
.sr-count{font-size:10px;font-weight:700;padding:2px 8px;border-radius:999px;background:rgba(255,255,255,0.06);color:var(--text-muted)}
.sr-search{display:flex;align-items:center;gap:8px;padding:0 14px;margin:0 0 8px;color:var(--text-muted)}
.sr-search-input{flex:1;background:rgba(255,255,255,0.04);border:1px solid rgba(255,255,255,0.06);border-radius:12px;padding:8px 11px;font-size:12px;color:var(--text-secondary);outline:none;transition:border-color .2s, background .2s}
.sr-search-input:focus{border-color:rgba(46,231,205,0.24);background:rgba(46,231,205,0.06)}
.sr-search-input::placeholder{color:var(--text-muted)}
.sr-scroll{flex:1;overflow-y:auto;padding:0 8px 8px}
.sr-scroll::-webkit-scrollbar{width:3px}.sr-scroll::-webkit-scrollbar-track{background:transparent}.sr-scroll::-webkit-scrollbar-thumb{background:var(--scrollbar-thumb);border-radius:3px}
.sr-empty{padding:34px 12px;text-align:center;font-size:11px;color:var(--text-dim)}
.sr-group{display:flex;align-items:center;gap:6px;padding:8px 10px;cursor:pointer;border-radius:10px;transition:background .15s}
.sr-group:hover{background:rgba(255,255,255,0.04)}
.sr-group-arrow{transition:transform .2s;color:var(--text-dim);flex-shrink:0;transform:rotate(-90deg)}
.sr-group.open .sr-group-arrow{transform:rotate(0deg)}
.sr-group-name{font-size:10px;font-weight:700;text-transform:uppercase;letter-spacing:1px;color:var(--text-muted);flex:1;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.sr-group-count{font-size:9px;color:var(--text-dim);flex-shrink:0}
.sr-group-items{padding:0 0 4px}
.sr-node{display:flex;align-items:center;gap:8px;padding:7px 9px;border-radius:12px;transition:all .15s}.sr-node:hover{background:rgba(255,255,255,0.04)}
.sr-node.active,.sr-node.connected{background:linear-gradient(90deg, rgba(125,247,104,0.12), rgba(46,231,205,0.09), rgba(93,91,255,0.10))}
.sr-node-main{display:flex;align-items:center;gap:8px;flex:1;min-width:0;cursor:pointer}
.sr-dot{width:7px;height:7px;border-radius:50%;flex-shrink:0;transition:all .3s}.sr-dot.fast{background:#7df768;box-shadow:0 0 8px rgba(125,247,104,0.45)}.sr-dot.medium{background:#3ce3ff;box-shadow:0 0 6px rgba(60,227,255,0.38)}.sr-dot.slow{background:#8f5fff;box-shadow:0 0 6px rgba(143,95,255,0.34)}.sr-dot.failed{background:#ff5b7d;box-shadow:0 0 7px rgba(255,91,125,0.32)}.sr-dot.testing{background:#3ce3ff;animation:pip-pulse .8s ease infinite}.sr-dot.unknown{background:rgba(255,255,255,0.14)}
.sr-node-info{display:flex;flex-direction:column;min-width:0;gap:1px}.sr-node-name{font-size:12px;font-weight:600;color:var(--text-secondary);white-space:nowrap;overflow:hidden;text-overflow:ellipsis}.sr-node-meta{font-size:10px;color:var(--text-muted);white-space:nowrap}
.sr-node-btn{width:26px;height:26px;border-radius:9px;display:flex;align-items:center;justify-content:center;background:rgba(255,255,255,0.04);border:1px solid rgba(255,255,255,0.07);color:var(--text-muted);cursor:pointer;transition:all .2s;flex-shrink:0}
.sr-node-btn:hover{background:rgba(46,231,205,0.12);color:#7df768;border-color:rgba(46,231,205,0.18)}
.sr-node-btn.connected{background:rgba(143,95,255,0.16);color:#d6c2ff;border-color:rgba(143,95,255,0.20)}
.sr-node-btn:disabled{opacity:.3;cursor:default}
.sr-progress{padding:8px 14px;border-top:1px solid var(--border)}
.sr-progress-bar{height:4px;border-radius:999px;background:rgba(255,255,255,0.05);overflow:hidden;margin-bottom:6px}.sr-progress-fill{height:100%;border-radius:999px;background:linear-gradient(90deg,#7df768,#2ee7cd,#5d5bff);transition:width .4s ease}
.sr-progress-text{font-size:10px;color:var(--text-muted);font-variant-numeric:tabular-nums}
.sr-footer{padding:10px 14px 14px;border-top:1px solid var(--border)}
.sr-test-btn{width:100%;display:flex;align-items:center;justify-content:center;gap:6px;padding:9px 10px;border-radius:12px;font-size:11px;font-weight:700;background:linear-gradient(90deg, rgba(125,247,104,0.16), rgba(46,231,205,0.12));border:1px solid rgba(125,247,104,0.14);color:#a8fca2;cursor:pointer;transition:all .2s}
.sr-test-btn:hover{background:linear-gradient(90deg, rgba(125,247,104,0.24), rgba(46,231,205,0.18));color:#f3fff2}.sr-test-btn:disabled{opacity:.4;cursor:default}
html.light .sl-collapse-edge{background:linear-gradient(180deg,rgba(255,255,255,0.98),rgba(238,248,246,0.96));border-color:rgba(58,121,136,0.18);color:rgba(16,48,60,0.62);box-shadow:0 8px 22px rgba(31,77,89,0.14)}
html.light .sl-collapse-edge:hover{color:#258151;border-color:rgba(58,191,103,0.34);background:linear-gradient(180deg,#fff,rgba(236,250,246,0.98))}
html.light .sidebar-left,html.light .sidebar-right{background:linear-gradient(180deg,rgba(252,255,255,0.96),rgba(239,248,247,0.94))}
html.light .sl-nav-item.active{box-shadow:inset 0 1px 0 rgba(255,255,255,0.74)}
@keyframes pip-pulse{0%,100%{opacity:1}50%{opacity:.35}}
</style>




