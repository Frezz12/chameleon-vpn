<template>
  <div class="page">
    <div class="page-inner">
      <div class="page-header">
        <h1 class="page-title">{{ t('nodes.title') }}</h1>
        <div class="header-actions">
          <button @click="showImport = true" class="btn btn-ghost">
            <svg width="13" height="13" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M7 2v7M3.5 5.5L7 9l3.5-3.5"/><path d="M1 10v1.5A1.5 1.5 0 002.5 13h9a1.5 1.5 0 001.5-1.5V10"/></svg>
            {{ t('nodes.import') }}
          </button>
          <button @click="showAdd = true" class="btn btn-primary">
            <svg width="13" height="13" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="7" y1="2" x2="7" y2="12"/><line x1="2" y1="7" x2="12" y2="7"/></svg>
            {{ t('nodes.add_node') }}
          </button>
          <button @click="testAllNodes" :disabled="nodesStore.testingAll" class="btn btn-accent">
            <svg width="13" height="13" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><polygon points="3 2 12 7 3 12 3 2"/></svg>
            {{ nodesStore.testingAll ? t('nodes.testing') : t('nodes.test_all') }}
          </button>
        </div>
      </div>

      <div class="search-row">
        <div class="search-bar">
          <svg width="13" height="13" viewBox="0 0 14 14" fill="none" stroke="rgba(255,255,255,0.2)" stroke-width="1.5" stroke-linecap="round"><circle cx="6" cy="6" r="4.5"/><line x1="9.5" y1="9.5" x2="13" y2="13"/></svg>
          <input v-model="nodesStore.searchQuery" type="text" :placeholder="t('nodes.search')" class="search-input"/>
        </div>
        <select v-model="nodesStore.filterProtocol" class="filter-select">
          <option value="">{{ t('nodes.all_protocols') }}</option>
          <option v-for="p in nodesStore.protocols" :key="p" :value="p">{{ p }}</option>
        </select>
      </div>

      <!-- Test Progress -->
      <div v-if="nodesStore.testingAll" class="test-progress">
        <div class="progress-info">
          <span class="progress-dot"/>
          <span>{{ t('nodes.testing_nodes') }}</span>
          <span class="progress-count">{{ nodesStore.testProgress.current }}/{{ nodesStore.testProgress.total }}</span>
        </div>
        <div class="progress-bar">
          <div class="progress-fill" :style="{ width: `${(nodesStore.testProgress.current / nodesStore.testProgress.total) * 100}%` }"/>
        </div>
      </div>

      <!-- Groups + Cards -->
      <template v-for="(nodes, group) in filteredGrouped" :key="group">
        <div class="group-header">
          <span class="group-name">{{ group }}</span>
          <span class="group-count">{{ nodes.length }}</span>
          <div class="group-line"/>
        </div>
        <div class="nodes-grid">
          <div v-for="node in nodes" :key="node.id" class="node-card" :class="{ selected: nodesStore.selectedNodeId === node.id }" @click="selectNode(node)">
            <div class="card-row">
              <span class="card-dot" :class="nodeHealthClass(node)"/>
              <span class="card-name">{{ node.name }}</span>
              <span class="card-proto">{{ node.protocol }}</span>
            </div>
            <div class="card-row card-detail">
              <span class="card-addr">{{ node.server }}:{{ node.port }}</span>
              <span class="card-lat" :class="nodeHealthClass(node)">
                {{ nodeStatusText(node) }}
              </span>
            </div>
            <div class="card-actions">
              <button @click.stop="testNodeSpeed(node.id)" class="ca-btn" :class="{ testing: testingNodes.has(node.id) }" :title="t('nodes.test')">
                <svg v-if="!testingNodes.has(node.id)" width="10" height="10" viewBox="0 0 10 10" fill="currentColor"><polygon points="1,0 10,5 1,10"/></svg>
                <span v-else class="ca-spinner"/>
              </button>
              <button @click.stop="deleteNodeById(node.id)" class="ca-btn danger" :title="t('nodes.delete')">
                <svg width="9" height="9" viewBox="0 0 9 9" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><path d="M1 1l7 7M8 1l-7 7"/></svg>
              </button>
            </div>
          </div>
        </div>
      </template>

      <div v-if="Object.keys(filteredGrouped).length === 0 && !nodesStore.loading" class="empty-state">
        <p>{{ t('nodes.no_nodes') }}</p>
        <p class="empty-hint">{{ t('nodes.no_nodes_hint') }}</p>
      </div>
    </div>

    <!-- Add Modal -->
    <Modal v-if="showAdd" @close="showAdd = false">
      <div class="modal-body">
        <h2 class="modal-title">{{ t('nodes.add_modal_title') }}</h2>
        <div class="form-group"><label>{{ t('nodes.name_label') }}</label><input v-model="form.name" class="form-input" :placeholder="t('nodes.name_placeholder')"/></div>
        <div class="form-group"><label>{{ t('nodes.protocol_label') }}</label>
          <select v-model="form.protocol" class="form-input">
            <option value="">{{ t('nodes.select_protocol') }}</option>
            <option value="vless">VLESS</option><option value="vmess">VMess</option><option value="trojan">Trojan</option>
            <option value="shadowsocks">Shadowsocks</option><option value="hysteria2">Hysteria2</option><option value="wireguard">WireGuard</option>
          </select>
        </div>
        <div class="form-row">
          <div class="form-group flex-1"><label>{{ t('nodes.server_label') }}</label><input v-model="form.server" class="form-input" :placeholder="t('nodes.server_placeholder')"/></div>
          <div class="form-group" style="width:80px"><label>{{ t('nodes.port_label') }}</label><input v-model.number="form.port" type="number" class="form-input"/></div>
        </div>
        <div class="form-group"><label>{{ t('nodes.config_label') }}</label><textarea v-model="form.configStr" rows="5" class="form-input mono" placeholder="{}"/></div>
        <div class="modal-actions">
          <button @click="showAdd = false" class="btn btn-ghost">{{ t('nodes.cancel') }}</button>
          <button @click="submitAdd" class="btn btn-primary">{{ t('nodes.add') }}</button>
        </div>
      </div>
    </Modal>

    <!-- Import Modal -->
    <Modal v-if="showImport" @close="showImport = false">
      <div class="modal-body">
        <h2 class="modal-title">{{ t('nodes.import_modal_title') }}</h2>
        <div class="form-group">
          <label>{{ t('nodes.subscription_url') }}</label>
          <input v-model="importUrl" class="form-input" :placeholder="t('nodes.subscription_placeholder')"/>
        </div>
        <div class="form-group">
          <label>{{ t('nodes.group_name') }}</label>
          <input v-model="importGroupName" class="form-input" :placeholder="t('nodes.group_name_placeholder')"/>
        </div>
        <p class="import-hint">{{ t('nodes.import_hint') }}</p>
        <div class="modal-actions">
          <button @click="showImport = false" class="btn btn-ghost">{{ t('nodes.cancel') }}</button>
          <button @click="doImport" class="btn btn-primary" :disabled="!importUrl">{{ t('nodes.import') }}</button>
        </div>
      </div>
    </Modal>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { useNodesStore } from '~/stores/nodes'
import { useNodes } from '~/composables/useNodes'
import { useToast } from '~/composables/useToast'
import { useI18n } from '~/composables/useI18n'

const nodesStore = useNodesStore()
const { fetchNodes, addNode, deleteNode, testSpeed: doTestSpeed, testAll, importSubscription, setupListeners } = useNodes()
const toast = useToast()
const { t } = useI18n()

const showAdd = ref(false)
const showImport = ref(false)
const importUrl = ref('')
const importGroupName = ref('')
const form = ref({ name: '', protocol: '', server: '', port: 443, configStr: '{}' })
const testingNodes = reactive(new Set<string>())

const filteredGrouped = computed(() => {
  const g: Record<string, any[]> = {}
  for (const n of nodesStore.filteredNodes) {
    const k = n.group_name?.trim() || 'Default'
    if (!g[k]) g[k] = []
    g[k].push(n)
  }
  return g
})

function nodeHealthClass(node: any) {
  if (node.test_status === 'testing') return 'testing'
  if (node.test_status === 'failed') return 'failed'
  const ms = node.latency_ms
  if (ms == null) return 'untested'
  if (ms < 100) return 'fast'
  if (ms < 300) return 'medium'
  return 'slow'
}

function nodeStatusText(node: any) {
  if (node.test_status === 'testing') return t('nodes.testing')
  if (node.test_status === 'failed') return t('nodes.unavailable')
  return node.latency_ms != null ? `${node.latency_ms.toFixed(0)}${t('common.ms')}` : t('nodes.untested')
}

function selectNode(node: any) {
  nodesStore.setSelectedNodeId(nodesStore.selectedNodeId === node.id ? null : node.id)
}

async function testNodeSpeed(id: string) {
  testingNodes.add(id)
  try { await doTestSpeed(id) } finally { testingNodes.delete(id) }
}
async function testAllNodes() { await testAll() }
async function deleteNodeById(id: string) { await deleteNode(id) }

async function submitAdd() {
  try {
    await addNode({ name: form.value.name, protocol: form.value.protocol, server: form.value.server, port: form.value.port, config: JSON.parse(form.value.configStr) })
    showAdd.value = false
    form.value = { name: '', protocol: '', server: '', port: 443, configStr: '{}' }
  } catch { toast.error(t('nodes.invalid_config')) }
}

async function doImport() {
  if (!importUrl.value) return
  try {
    const nodes = await importSubscription(importUrl.value, importGroupName.value || undefined)
    if (importGroupName.value && nodes.length > 0) {
      toast.success(`${t('toast.nodes_imported')} ${nodes.length} -> ${importGroupName.value}`)
    } else {
      toast.success(`${t('toast.nodes_imported')} ${nodes.length}`)
    }
    showImport.value = false
    importUrl.value = ''
    importGroupName.value = ''
  } catch {}
}

onMounted(() => { fetchNodes(); setupListeners() })
</script>

<style scoped>
.page { height: 100%; overflow-y: auto; }
.page-inner { max-width: 960px; margin: 0 auto; padding: 20px 24px; display: flex; flex-direction: column; gap: 12px; }
.page-header { display: flex; align-items: center; justify-content: space-between; }
.page-title { font-size: 17px; font-weight: 600; color: rgba(255,255,255,0.85); }
.header-actions { display: flex; gap: 6px; }

.btn { display: inline-flex; align-items: center; gap: 5px; padding: 6px 12px; border-radius: 7px; font-size: 11px; font-weight: 500; border: 1px solid; cursor: pointer; transition: all 0.2s; }
.btn-ghost { background: rgba(255,255,255,0.03); border-color: rgba(255,255,255,0.07); color: rgba(255,255,255,0.45); }
.btn-ghost:hover { background: rgba(255,255,255,0.06); color: rgba(255,255,255,0.65); }
.btn-primary { background: rgba(0,229,200,0.1); border-color: rgba(0,229,200,0.18); color: #00e5c8; }
.btn-primary:hover { background: rgba(0,229,200,0.18); }
.btn-accent { background: rgba(0,229,200,0.06); border-color: rgba(0,229,200,0.12); color: rgba(0,229,200,0.7); }
.btn-accent:hover { background: rgba(0,229,200,0.12); }
.btn:disabled { opacity: 0.35; cursor: default; }

.search-row { display: flex; gap: 8px; }
.search-bar { flex: 1; display: flex; align-items: center; gap: 7px; padding: 0 10px; background: rgba(255,255,255,0.025); border: 1px solid rgba(255,255,255,0.05); border-radius: 7px; }
.search-input { flex: 1; background: none; border: none; outline: none; padding: 8px 2px; font-size: 12px; color: rgba(255,255,255,0.65); }
.search-input::placeholder { color: rgba(255,255,255,0.18); }
.filter-select { background: rgba(255,255,255,0.025); border: 1px solid rgba(255,255,255,0.05); border-radius: 7px; padding: 6px 10px; font-size: 11px; color: rgba(255,255,255,0.45); outline: none; cursor: pointer; }
.filter-select option { background: #0f0d16; }

.test-progress { padding: 10px 14px; border-radius: 8px; background: rgba(0,229,200,0.03); border: 1px solid rgba(0,229,200,0.08); }
.progress-info { display: flex; align-items: center; gap: 7px; margin-bottom: 6px; font-size: 11px; color: rgba(255,255,255,0.55); }
.progress-dot { width: 5px; height: 5px; border-radius: 50%; background: #00e5c8; animation: pip-pulse 0.8s ease infinite; }
.progress-count { margin-left: auto; font-variant-numeric: tabular-nums; color: rgba(255,255,255,0.25); }
.progress-bar { height: 2px; border-radius: 2px; background: rgba(255,255,255,0.05); overflow: hidden; }
.progress-fill { height: 100%; border-radius: 2px; background: linear-gradient(90deg, #00e5c8, #00c6ff); transition: width 0.4s ease; }

.group-header { display: flex; align-items: center; gap: 7px; margin-top: 6px; }
.group-name { font-size: 9px; font-weight: 600; text-transform: uppercase; letter-spacing: 1px; color: rgba(255,255,255,0.18); }
.group-count { font-size: 9px; color: rgba(255,255,255,0.12); }
.group-line { flex: 1; height: 1px; background: rgba(255,255,255,0.035); }

.nodes-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 6px; }
.node-card { padding: 8px 10px; border-radius: 7px; background: rgba(255,255,255,0.018); border: 1px solid rgba(255,255,255,0.04); cursor: pointer; transition: all 0.15s; }
.node-card:hover { background: rgba(255,255,255,0.035); border-color: rgba(255,255,255,0.07); }
.node-card.selected { background: rgba(0,229,200,0.04); border-color: rgba(0,229,200,0.15); }

.card-row { display: flex; align-items: center; gap: 6px; }
.card-row + .card-row { margin-top: 3px; }
.card-dot { width: 5px; height: 5px; border-radius: 50%; flex-shrink: 0; transition: all 0.3s; }
.card-dot.fast { background: #00e5c8; box-shadow: 0 0 4px rgba(0,229,200,0.35); }
.card-dot.medium { background: #f59e0b; }
.card-dot.slow { background: #ef4444; }
.card-dot.untested { background: rgba(255,255,255,0.08); }
.card-dot.failed { background: #ff5b7d; box-shadow: 0 0 5px rgba(255,91,125,0.35); }
.card-dot.testing { background: #00c6ff; animation: pip-pulse 0.8s ease infinite; }
.card-name { font-size: 12px; font-weight: 500; color: rgba(255,255,255,0.75); flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; min-width: 0; }
.card-proto { font-size: 8px; text-transform: uppercase; letter-spacing: 0.3px; padding: 1px 5px; border-radius: 3px; background: rgba(255,255,255,0.035); color: rgba(255,255,255,0.25); flex-shrink: 0; }

.card-detail { justify-content: space-between; }
.card-addr { font-size: 10px; color: rgba(255,255,255,0.25); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.card-lat { font-size: 10px; font-variant-numeric: tabular-nums; flex-shrink: 0; }
.card-lat.fast { color: rgba(0,229,200,0.7); }
.card-lat.medium { color: rgba(245,158,11,0.7); }
.card-lat.slow { color: rgba(239,68,68,0.7); }
.card-lat.untested { color: rgba(255,255,255,0.15); }
.card-lat.failed { color: rgba(255,91,125,0.82); }
.card-lat.testing { color: rgba(0,198,255,0.78); }

.card-actions { display: flex; gap: 3px; margin-top: 5px; border-top: 1px solid rgba(255,255,255,0.03); padding-top: 4px; }
.ca-btn { width: 20px; height: 20px; border-radius: 4px; display: flex; align-items: center; justify-content: center; background: rgba(255,255,255,0.025); border: 1px solid rgba(255,255,255,0.04); color: rgba(255,255,255,0.25); cursor: pointer; transition: all 0.15s; }
.ca-btn:hover { background: rgba(0,229,200,0.08); color: rgba(0,229,200,0.7); border-color: rgba(0,229,200,0.15); }
.ca-btn.testing { opacity: 0.5; pointer-events: none; }
.ca-btn.danger:hover { background: rgba(239,68,68,0.08); color: rgba(239,68,68,0.7); border-color: rgba(239,68,68,0.15); }
.ca-spinner { width: 8px; height: 8px; border: 1.5px solid rgba(0,229,200,0.3); border-top-color: #00e5c8; border-radius: 50%; animation: spin 0.6s linear infinite; }

.empty-state { padding: 40px; text-align: center; }
.empty-state p { color: rgba(255,255,255,0.25); font-size: 12px; }
.empty-hint { color: rgba(255,255,255,0.12); font-size: 10px; margin-top: 3px; }

.modal-body { padding: 22px; min-width: 360px; }
.modal-title { font-size: 15px; font-weight: 600; color: rgba(255,255,255,0.8); margin-bottom: 16px; }
.form-group { margin-bottom: 12px; }
.form-group label { display: block; font-size: 9px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.8px; color: rgba(255,255,255,0.22); margin-bottom: 4px; }
.form-input { width: 100%; padding: 8px 10px; border-radius: 6px; font-size: 12px; background: rgba(255,255,255,0.025); border: 1px solid rgba(255,255,255,0.06); color: rgba(255,255,255,0.7); outline: none; }
.form-input:focus { border-color: rgba(0,229,200,0.25); }
.form-input.mono { font-family: monospace; font-size: 10px; resize: none; line-height: 1.5; }
select.form-input { cursor: pointer; }
select.form-input option { background: #0f0d16; }
.form-row { display: flex; gap: 8px; }
.flex-1 { flex: 1; }
.modal-actions { display: flex; justify-content: flex-end; gap: 6px; margin-top: 16px; }
.import-hint { font-size: 10px; color: rgba(255,255,255,0.15); margin: -4px 0 8px; line-height: 1.4; }

@keyframes pip-pulse { 0%,100% { opacity: 1; } 50% { opacity: 0.3; } }
@keyframes spin { to { transform: rotate(360deg); } }
</style>



