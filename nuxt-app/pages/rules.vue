<template>
  <div class="page">
    <div class="page-inner">
      <div class="page-header">
        <h1 class="page-title">{{ t('rules.title') }}</h1>
        <button @click="openAdd" class="btn btn-primary">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="7" y1="2" x2="7" y2="12"/><line x1="2" y1="7" x2="12" y2="7"/></svg>
          {{ t('rules.add_rule') }}
        </button>
      </div>

      <div class="test-bar">
        <input v-model="testDomain" @keyup.enter="testRule" class="test-input" :placeholder="t('rules.domain_placeholder')"/>
        <button @click="testRule" class="btn btn-accent">
          <svg width="13" height="13" viewBox="0 0 13 13" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><polygon points="2 2 11 6.5 2 11 2 2"/></svg>
          {{ t('rules.test') }}
        </button>
      </div>
      <div v-if="rulesStore.testResult" class="test-result">
        <template v-if="rulesStore.testResult.matched">
          <span class="result-matched">{{ t('rules.matched') }}</span>
          <span class="result-value">{{ rulesStore.testResult.rule_value }}</span>
          <span class="result-arrow">-></span>
          <span class="result-node">{{ rulesStore.testResult.node_id }}</span>
        </template>
        <template v-else>
          <span class="result-unmatched">{{ t('rules.no_match') }} -> {{ t('rules.direct') }}</span>
        </template>
      </div>

      <div class="drop-zone" :class="{ active: dragging }" @dragenter.prevent="dragging = true" @dragover.prevent="dragging = true" @dragleave.prevent="dragging = false" @drop.prevent="onDrop">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="rgba(255,255,255,0.12)" stroke-width="1.5" stroke-linecap="round"><path d="M12 2v12M8 10l4 4 4-4"/><path d="M4 14v4a2 2 0 002 2h12a2 2 0 002-2v-4"/></svg>
        <p>{{ t('rules.drop_text') }}</p>
      </div>

      <div v-if="rulesStore.sortedRules.length === 0" class="empty-state">{{ t('rules.no_rules') }}</div>
      <div class="rules-list">
        <draggable v-model="orderedRules" item-key="id" handle=".drag-handle" :animation="200" ghost-class="ghost" @change="onReorder">
          <template #item="{ element: rule }">
            <div class="rule-item" :class="{ disabled: !rule.enabled }">
              <span class="drag-handle">
                <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor"><circle cx="4" cy="2.5" r="1"/><circle cx="8" cy="2.5" r="1"/><circle cx="4" cy="6" r="1"/><circle cx="8" cy="6" r="1"/><circle cx="4" cy="9.5" r="1"/><circle cx="8" cy="9.5" r="1"/></svg>
              </span>
              <span class="rule-type" :style="{ color: typeColor(rule.rule_type) }">{{ rule.rule_type.replace('_', ' ') }}</span>
              <span class="rule-value">{{ rule.value }}</span>
              <span class="rule-node" :class="{ direct: rule.node_id === 'direct' }">{{ rule.node_id || 'direct' }}</span>
              <div class="rule-actions">
                <button @click="toggleRule(rule)" class="action-btn" :class="{ on: rule.enabled }">
                  <svg width="11" height="11" viewBox="0 0 11 11" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><polyline points="2 6 5 9 9 3"/></svg>
                </button>
                <button @click="editRule(rule)" class="action-btn">
                  <svg width="11" height="11" viewBox="0 0 11 11" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"><path d="M2 9V10h1L8.5 4.5 7.5 3.5 2 9z"/><path d="M7.5 3.5l-1-1"/></svg>
                </button>
                <button @click="removeRule(rule)" class="action-btn danger">
                  <svg width="11" height="11" viewBox="0 0 11 11" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><polyline points="2.5 2.5 8.5 8.5M8.5 2.5l-6 6"/></svg>
                </button>
              </div>
            </div>
          </template>
        </draggable>
      </div>
    </div>

    <Modal v-if="showEditor" @close="closeEditor">
      <div class="modal-body">
        <h2 class="modal-title">{{ editingRule ? t('rules.edit_modal_title') : t('rules.add_modal_title') }}</h2>
        <div class="form-group"><label>{{ t('rules.type_label') }}</label>
          <select v-model="form.rule_type" class="form-input">
            <option value="domain_full">{{ t('rules.type_domain_full') }}</option>
            <option value="domain_suffix">{{ t('rules.type_domain_suffix') }}</option>
            <option value="domain_keyword">{{ t('rules.type_domain_keyword') }}</option>
            <option value="domain_regex">{{ t('rules.type_domain_regex') }}</option>
            <option value="ip_cidr">{{ t('rules.type_ip_cidr') }}</option>
            <option value="process_name">{{ t('rules.type_process_name') }}</option>
            <option value="geoip">{{ t('rules.type_geoip') }}</option>
            <option value="geosite">{{ t('rules.type_geosite') }}</option>
          </select>
        </div>
        <div class="form-group"><label>{{ t('rules.value_label') }}</label><input v-model="form.value" class="form-input" placeholder="e.g., google.com"/></div>
        <div class="form-group"><label>{{ t('rules.target_node_label') }}</label>
          <select v-model="form.node_id" class="form-input">
            <option value="direct">{{ t('rules.direct_option') }}</option>
            <option value="block">{{ t('rules.block_option') }}</option>
            <option v-for="n in nodesStore.nodes" :key="n.id" :value="n.id">{{ n.name }}</option>
          </select>
        </div>
        <label class="checkbox-label"><input v-model="form.enabled" type="checkbox"/> {{ t('rules.enabled') }}</label>
        <div class="modal-actions">
          <button @click="closeEditor" class="btn btn-ghost">{{ t('rules.cancel') }}</button>
          <button @click="saveRule" class="btn btn-primary">{{ editingRule ? t('rules.update') : t('rules.add') }}</button>
        </div>
      </div>
    </Modal>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import draggable from 'vuedraggable'
import { useRulesStore, type RouteRule } from '~/stores/rules'
import { useNodesStore } from '~/stores/nodes'
import { useTauri } from '~/composables/useTauri'
import { useToast } from '~/composables/useToast'
import { useI18n } from '~/composables/useI18n'

const rulesStore = useRulesStore()
const nodesStore = useNodesStore()
const { invoke } = useTauri()
const toast = useToast()
const { t } = useI18n()

interface RuleForm {
  rule_type: string
  value: string
  node_id: string
  enabled: boolean
}

const testDomain = ref('')
const dragging = ref(false)
const showEditor = ref(false)
const editingRule = ref<RouteRule | null>(null)
const orderedRules = ref<RouteRule[]>([])
const form = ref<RuleForm>({ rule_type: 'domain_suffix', value: '', node_id: 'direct', enabled: true })

const typeColor = (type: string) => ({ domain_full: '#3b82f6', domain_suffix: '#00e5c8', domain_keyword: '#f59e0b', domain_regex: '#a855f7', ip_cidr: '#ec4899', process_name: '#f97316', geoip: '#14b8a6', geosite: '#8b5cf6' }[type] || '#6b7280')

function getErrorMessage(error: unknown, fallback: string) {
  if (typeof error === 'string') return error
  if (error instanceof Error && error.message) return error.message
  return fallback
}

async function loadRules() {
  try {
    const rules = await invoke<RouteRule[]>('get_rules')
    rulesStore.setRules(rules)
    orderedRules.value = [...rulesStore.sortedRules]
  } catch (error) {
    toast.error(getErrorMessage(error, t('toast.rules_load_failed')))
  }
}

async function testRule() {
  if (!testDomain.value) return
  try {
    rulesStore.setTestResult(await invoke<any>('test_rule', { domain: testDomain.value }))
  } catch (error) {
    toast.error(getErrorMessage(error, t('toast.rule_test_failed')))
  }
}

async function onReorder() {
  try {
    await invoke('reorder_rules', { ruleIds: orderedRules.value.map((rule) => rule.id) })
    rulesStore.setRules(orderedRules.value)
  } catch (error) {
    toast.error(getErrorMessage(error, t('toast.rule_reorder_failed')))
    orderedRules.value = [...rulesStore.sortedRules]
  }
}

function openAdd() {
  editingRule.value = null
  form.value = { rule_type: 'domain_suffix', value: '', node_id: 'direct', enabled: true }
  showEditor.value = true
}

function editRule(rule: RouteRule) {
  editingRule.value = rule
  form.value = { rule_type: rule.rule_type, value: rule.value, node_id: rule.node_id, enabled: rule.enabled }
  showEditor.value = true
}

function closeEditor() {
  showEditor.value = false
  editingRule.value = null
}

async function saveRule() {
  if (!form.value.value) return
  try {
    if (editingRule.value) {
      await invoke('update_rule', { ruleId: editingRule.value.id, rule: form.value })
      toast.success(t('toast.rule_updated'))
    } else {
      await invoke('add_rule', { rule: form.value })
      toast.success(t('toast.rule_added'))
    }
    closeEditor()
    await loadRules()
  } catch (error) {
    toast.error(getErrorMessage(error, t('toast.rule_save_failed')))
  }
}

async function removeRule(rule: RouteRule) {
  try {
    await invoke('delete_rule', { ruleId: rule.id })
    toast.success(t('toast.rule_deleted'))
    await loadRules()
  } catch (error) {
    toast.error(getErrorMessage(error, t('toast.rule_delete_failed')))
  }
}

async function toggleRule(rule: RouteRule) {
  try {
    await invoke('update_rule', { ruleId: rule.id, rule: { ...rule, enabled: !rule.enabled } })
    await loadRules()
  } catch (error) {
    toast.error(getErrorMessage(error, t('toast.rule_save_failed')))
  }
}

function onDrop(event: DragEvent) {
  dragging.value = false
  const data = event.dataTransfer?.getData('text')
  if (!data) return

  try {
    const url = new URL(data)
    form.value = { rule_type: 'domain_suffix', value: url.hostname, node_id: 'direct', enabled: true }
  } catch {
    form.value = { rule_type: 'domain_suffix', value: data, node_id: 'direct', enabled: true }
  }

  showEditor.value = true
}

onMounted(loadRules)
</script>

<style scoped>
.page { height: 100%; overflow-y: auto; }
.page-inner { max-width: 800px; margin: 0 auto; padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.page-header { display: flex; align-items: center; justify-content: space-between; }
.page-title { font-size: 18px; font-weight: 600; color: rgba(255,255,255,0.85); }
.btn { display: inline-flex; align-items: center; gap: 6px; padding: 7px 14px; border-radius: 8px; font-size: 12px; font-weight: 500; border: 1px solid; cursor: pointer; transition: all 0.2s; }
.btn-ghost { background: rgba(255,255,255,0.03); border-color: rgba(255,255,255,0.08); color: rgba(255,255,255,0.5); }
.btn-ghost:hover { background: rgba(255,255,255,0.06); color: rgba(255,255,255,0.7); }
.btn-primary { background: rgba(0,229,200,0.12); border-color: rgba(0,229,200,0.2); color: #00e5c8; }
.btn-primary:hover { background: rgba(0,229,200,0.2); }
.btn-accent { background: rgba(0,229,200,0.08); border-color: rgba(0,229,200,0.15); color: #00e5c8; }
.btn-accent:hover { background: rgba(0,229,200,0.15); }

.test-bar { display: flex; gap: 8px; }
.test-input { flex: 1; padding: 9px 12px; border-radius: 7px; font-size: 13px; background: rgba(255,255,255,0.03); border: 1px solid rgba(255,255,255,0.07); color: rgba(255,255,255,0.7); outline: none; }
.test-input:focus { border-color: rgba(0,229,200,0.3); }
.test-input::placeholder { color: rgba(255,255,255,0.15); }

.test-result { font-size: 12px; padding: 8px 12px; border-radius: 8px; background: rgba(255,255,255,0.02); }
.result-matched { color: #00e5c8; }
.result-value { color: rgba(255,255,255,0.6); margin-left: 4px; font-family: monospace; }
.result-arrow { color: rgba(255,255,255,0.2); margin: 0 4px; }
.result-node { color: #3b82f6; font-family: monospace; }
.result-unmatched { color: rgba(255,255,255,0.3); }

.drop-zone { display: flex; flex-direction: column; align-items: center; gap: 6px; padding: 20px; border-radius: 10px; border: 1px dashed rgba(255,255,255,0.06); background: rgba(255,255,255,0.01); transition: all 0.2s; }
.drop-zone p { font-size: 12px; color: rgba(255,255,255,0.2); }
.drop-zone.active { border-color: rgba(0,229,200,0.2); background: rgba(0,229,200,0.02); }

.empty-state { text-align: center; padding: 24px; font-size: 12px; color: rgba(255,255,255,0.2); }

.rule-item { display: flex; align-items: center; gap: 10px; padding: 10px 14px; border-radius: 8px; background: rgba(255,255,255,0.02); border: 1px solid rgba(255,255,255,0.04); margin-bottom: 4px; transition: all 0.2s; }
.rule-item:hover { background: rgba(255,255,255,0.04); }
.rule-item.disabled { opacity: 0.4; }
.drag-handle { cursor: grab; color: rgba(255,255,255,0.12); flex-shrink: 0; }
.rule-type { font-size: 10px; text-transform: uppercase; letter-spacing: 0.5px; font-weight: 600; padding: 2px 6px; border-radius: 4px; background: rgba(255,255,255,0.03); flex-shrink: 0; }
.rule-value { font-size: 12px; font-family: monospace; color: rgba(255,255,255,0.6); flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.rule-node { font-size: 11px; color: #3b82f6; font-family: monospace; flex-shrink: 0; }
.rule-node.direct { color: rgba(255,255,255,0.3); }
.rule-actions { display: flex; gap: 3px; flex-shrink: 0; }
.action-btn { width: 24px; height: 24px; border-radius: 5px; display: flex; align-items: center; justify-content: center; background: rgba(255,255,255,0.03); border: none; color: rgba(255,255,255,0.25); cursor: pointer; transition: all 0.15s; }
.action-btn:hover { background: rgba(255,255,255,0.07); color: rgba(255,255,255,0.5); }
.action-btn.on { color: #00e5c8; background: rgba(0,229,200,0.08); }
.action-btn.danger:hover { background: rgba(239,68,68,0.08); color: rgba(239,68,68,0.7); }
.ghost { opacity: 0.2; }

.modal-body { padding: 24px; min-width: 380px; }
.modal-title { font-size: 16px; font-weight: 600; color: rgba(255,255,255,0.8); margin-bottom: 18px; }
.form-group { margin-bottom: 14px; }
.form-group label { display: block; font-size: 10px; font-weight: 600; text-transform: uppercase; letter-spacing: 1px; color: rgba(255,255,255,0.25); margin-bottom: 5px; }
.form-input { width: 100%; padding: 9px 12px; border-radius: 7px; font-size: 13px; background: rgba(255,255,255,0.03); border: 1px solid rgba(255,255,255,0.07); color: rgba(255,255,255,0.7); outline: none; }
.form-input:focus { border-color: rgba(0,229,200,0.3); }
select.form-input { cursor: pointer; }
select.form-input option { background: #12101a; }
.checkbox-label { display: flex; align-items: center; gap: 8px; font-size: 12px; color: rgba(255,255,255,0.5); cursor: pointer; }
.modal-actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 18px; }
</style>
