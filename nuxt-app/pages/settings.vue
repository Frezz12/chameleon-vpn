<template>
  <div class="settings-page" :class="{ 'is-light': isLight }">
    <div class="settings-shell">
      <section class="settings-hero">
        <div class="hero-top">
          <div class="hero-mark-wrap">
            <span class="hero-mark-glow" />
            <img src="/branding/chameleon-mark.png" :alt="appName" class="hero-mark" />
          </div>
          <div class="hero-copy">
            <p class="hero-eyebrow">{{ t('settings.title') }}</p>
            <h1 class="hero-brand-name">{{ appName }}</h1>
            <p class="hero-brand-subtitle">Центр управления подключением</p>
          </div>
        </div>
        <p class="hero-description">
Настройки подключения, маршрутизации, автоматизации и резервных копий собраны компактно, чтобы ими было удобно пользоваться даже в узком окне приложения.
        </p>
        <div class="hero-chips">
          <span class="hero-chip">{{ t('settings.proxy_mode') }}: {{ proxyModeLabel }}</span>
          <span class="hero-chip">DNS: {{ dnsLabel }}</span>
          <span class="hero-chip">{{ t('settings.max_retries') }}: {{ settings.max_retries }}</span>
        </div>
      </section>

      <section class="settings-card stats-card">
        <div class="section-head compact-head section-head-static">
          <div>
            <p class="section-kicker">Состояние</p>
            <h2 class="section-title">Текущий профиль</h2>
          </div>
        </div>
        <div class="stat-grid">
          <article class="mini-stat lime">
            <span class="mini-stat-label">Таймаут</span>
            <strong class="mini-stat-value">{{ settings.connection_timeout }}s</strong>
          </article>
          <article class="mini-stat cyan">
            <span class="mini-stat-label">Авто-пинг</span>
            <strong class="mini-stat-value">{{ settings.auto_ping ? 'ON' : 'OFF' }}</strong>
          </article>
          <article class="mini-stat violet">
            <span class="mini-stat-label">Источники</span>
            <strong class="mini-stat-value">{{ subscriptions.length }}</strong>
          </article>
        </div>
      </section>

      <section class="settings-card">
        <div class="section-head section-head-static">
          <div>
            <p class="section-kicker">{{ t('settings.general') }}</p>
            <h2 class="section-title">Маршрутизация</h2>
          </div>
        </div>
        <div class="stack-list">
          <article v-for="item in generalList" :key="item.key" class="setting-item setting-item-toggle">
            <div class="setting-copy">
              <p class="setting-label">{{ item.label }}</p>
              <p class="setting-desc">{{ item.desc }}</p>
            </div>
            <label class="toggle">
              <input :checked="Boolean(settings[item.key])" type="checkbox" @change="saveBoolean(item.key, ($event.target as HTMLInputElement).checked)" />
              <span class="toggle-slider" />
            </label>
          </article>
        </div>
      </section>

      <section class="settings-card">
        <div class="section-head section-head-static">
          <div>
            <p class="section-kicker">{{ t('settings.network') }}</p>
            <h2 class="section-title">Сеть и транспорт</h2>
          </div>
        </div>
        <div class="form-stack">
          <div class="form-group">
            <label>{{ t('settings.dns_server') }}</label>
            <CustomSelect v-model="settings.dns_server" :options="dnsOptions" @change="(value) => saveSetting('dns_server', String(value))" />
          </div>
          <div class="form-group">
            <label>{{ t('settings.proxy_mode') }}</label>
            <CustomSelect v-model="settings.proxy_mode" :options="proxyModeOptions" @change="(value) => saveSetting('proxy_mode', value as ProxyMode)" />
            <p v-if="settings.proxy_mode === 'tunnel'" class="inline-hint">{{ t('settings.proxy_tunnel_desc') }}</p>
          </div>
          <div class="form-group">
            <label>{{ t('settings.split_processes') }}</label>
            <input v-model="settings.split_processes" class="form-input" :placeholder="t('settings.split_processes_placeholder')" @change="saveSetting('split_processes', settings.split_processes)" />
          </div>
          <div class="dual-inputs">
            <div class="form-group compact-field">
              <label>{{ t('settings.connection_timeout') }}</label>
              <input v-model.number="settings.connection_timeout" type="number" min="5" max="60" class="form-input" @change="saveSetting('connection_timeout', settings.connection_timeout)" />
            </div>
            <div class="form-group compact-field">
              <label>{{ t('settings.max_retries') }}</label>
              <input v-model.number="settings.max_retries" type="number" min="1" max="10" class="form-input" @change="saveSetting('max_retries', settings.max_retries)" />
            </div>
          </div>
        </div>
      </section>

      <section class="settings-card">
        <div class="section-head section-head-static">
          <div>
            <p class="section-kicker">{{ t('settings.thresholds') }}</p>
            <h2 class="section-title">Автоматическая проверка</h2>
          </div>
        </div>
        <div class="dual-inputs">
          <div class="form-group compact-field">
            <label>{{ t('settings.latency_threshold') }}</label>
            <input v-model.number="settings.latency_threshold_ms" type="number" class="form-input" @change="saveSetting('latency_threshold_ms', settings.latency_threshold_ms)" />
          </div>
          <div class="form-group compact-field">
            <label>{{ t('settings.error_threshold') }}</label>
            <input v-model.number="settings.error_threshold_pct" type="number" class="form-input" @change="saveSetting('error_threshold_pct', settings.error_threshold_pct)" />
          </div>
        </div>
        <article class="setting-item setting-item-toggle top-gap">
          <div class="setting-copy">
            <p class="setting-label">{{ t('settings.auto_ping') }}</p>
            <p class="setting-desc">{{ t('settings.auto_ping_desc') }}</p>
          </div>
          <label class="toggle">
            <input :checked="settings.auto_ping" type="checkbox" @change="saveBoolean('auto_ping', ($event.target as HTMLInputElement).checked)" />
            <span class="toggle-slider" />
          </label>
        </article>
        <div v-if="settings.auto_ping" class="form-group top-gap-sm">
          <label>{{ t('settings.ping_interval') }}</label>
          <CustomSelect v-model="settings.ping_interval" :options="pingIntervalOptions" @change="(value) => saveSetting('ping_interval', Number(value))" />
        </div>
      </section>

      <section class="settings-card">
        <div class="section-head section-head-action">
          <div>
            <p class="section-kicker">{{ t('settings.subscriptions') }}</p>
            <h2 class="section-title">Источники серверов</h2>
          </div>
          <button class="btn btn-primary section-action-btn" @click="showAddSub = true">+ {{ t('nodes.add') }}</button>
        </div>
        <div v-if="subscriptions.length === 0" class="empty-state">
          <p>{{ t('settings.no_subscriptions') }}</p>
        </div>
        <div v-else class="subscription-list">
          <article v-for="sub in subscriptions" :key="sub.id" class="subscription-card">
            <div class="subscription-copy">
              <strong class="subscription-name">{{ sub.name || sub.url }}</strong>
              <span class="subscription-url">{{ sub.url }}</span>
              <div class="subscription-tags">
                <span class="tag">{{ sub.group_name || 'Default' }}</span>
                <span class="tag">{{ sub.interval_mins }}m</span>
              </div>
            </div>
            <div class="subscription-actions">
              <button class="btn-icon" :title="t('settings.import_now')" @click="importSub(sub.id)">
                <svg width="11" height="11" viewBox="0 0 11 11" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"><path d="M1 7.5V9a1 1 0 001 1h7a1 1 0 001-1V7.5M5.5 1v6M3.5 4.5L5.5 7l2-2.5"/></svg>
              </button>
              <button class="btn-icon danger" @click="deleteSub(sub.id)">
                <svg width="11" height="11" viewBox="0 0 11 11" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><path d="M2 2.5h7M4 2V1.5a.5.5 0 01.5-.5h2a.5.5 0 01.5.5V2M2.5 3v6a1 1 0 001 1h4a1 1 0 001-1V3"/></svg>
              </button>
            </div>
          </article>
        </div>
      </section>

      <section class="settings-card">
        <div class="section-head section-head-static">
          <div>
            <p class="section-kicker">{{ t('settings.theme') }}</p>
            <h2 class="section-title">Оформление интерфейса</h2>
          </div>
        </div>
        <div class="theme-switcher">
          <button class="theme-pill" :class="{ active: !isLight }" @click="applyTheme('dark')">{{ t('settings.theme_dark') }}</button>
          <button class="theme-pill" :class="{ active: isLight }" @click="applyTheme('light')">{{ t('settings.theme_light') }}</button>
        </div>
        <div class="proxy-box">
          <div class="proxy-row"><span>SOCKS5</span><span class="proxy-val">127.0.0.1:2080</span></div>
          <div class="proxy-row"><span>HTTP</span><span class="proxy-val">127.0.0.1:2080</span></div>
        </div>
      </section>

      <section class="settings-card">
        <div class="section-head section-head-static">
          <div>
            <p class="section-kicker">{{ t('settings.backup') }}</p>
            <h2 class="section-title">Резервные копии</h2>
          </div>
        </div>
        <div class="backup-actions">
          <button @click="doExport" class="btn btn-ghost wide-action-btn">
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"><path d="M1 8v2.5A1.5 1.5 0 002.5 12h7a1.5 1.5 0 001.5-1.5V8M6 1v7M3.5 5.5L6 8l2.5-2.5"/></svg>
            {{ t('settings.export_settings') }}
          </button>
          <button @click="triggerImport" class="btn btn-ghost wide-action-btn">
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"><path d="M1 8v2.5A1.5 1.5 0 002.5 12h7a1.5 1.5 0 001.5-1.5V8M6 8V1M3.5 3.5L6 1l2.5 2.5"/></svg>
            {{ t('settings.import_settings') }}
          </button>
          <input ref="importInput" type="file" accept=".json" class="hidden-input" @change="doImport" />
        </div>
        <div class="about-list">
          <div class="about-row"><span class="about-label">{{ t('settings.version') }}</span><span class="about-value">0.1.0</span></div>
          <div class="about-row"><span class="about-label">{{ t('settings.singbox') }}</span><span class="about-value">1.9.3</span></div>
          <div class="about-row"><span class="about-label">{{ t('settings.frontend') }}</span><span class="about-value">Nuxt 3 + TailwindCSS</span></div>
          <div class="about-row"><span class="about-label">{{ t('settings.backend') }}</span><span class="about-value">Rust + Tauri v2</span></div>
          <div class="about-row"><span class="about-label">{{ t('settings.platform') }}</span><span class="about-value">{{ platform }}</span></div>
        </div>
      </section>
    </div>

    <Teleport to="body">
      <div v-if="showAddSub" class="modal-overlay" @click.self="showAddSub = false">
        <div class="modal-panel">
          <h3 class="modal-title">{{ t('settings.add_subscription') }}</h3>
          <div class="form-group"><label>{{ t('settings.name') }}</label><input v-model="newSub.name" class="form-input" placeholder="My Provider" /></div>
          <div class="form-group"><label>{{ t('settings.subscription_url') }}</label><input v-model="newSub.url" class="form-input" placeholder="https://" /></div>
          <div class="form-group"><label>{{ t('settings.group') }}</label><input v-model="newSub.group" class="form-input" /></div>
          <div class="form-group"><label>{{ t('settings.subscription_interval') }}</label><input v-model.number="newSub.interval" type="number" min="15" class="form-input" /></div>
          <div class="modal-actions">
            <button @click="showAddSub = false" class="btn btn-ghost modal-btn">{{ t('nodes.cancel') }}</button>
            <button @click="addSub" class="btn btn-primary modal-btn" :disabled="!newSub.url">{{ t('nodes.add') }}</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import type { Subscription } from '~/stores/nodes'
import { useTauri } from '~/composables/useTauri'
import { useToast } from '~/composables/useToast'
import { useI18n } from '~/composables/useI18n'
import { useTheme } from '~/composables/useTheme'

type ProxyMode = 'tunnel' | 'system' | 'manual'
type ThemeMode = 'dark' | 'light'

type AppSettings = {
  autostart: boolean
  system_tray: boolean
  auto_switch: boolean
  latency_threshold_ms: number
  error_threshold_pct: number
  log_level: 'error' | 'warn' | 'info' | 'debug'
  dns_server: string
  proxy_mode: ProxyMode
  bypass_ru: boolean
  bypass_local: boolean
  kill_switch: boolean
  connection_timeout: number
  max_retries: number
  split_processes: string
  auto_ping: boolean
  ping_interval: number
}

type SettingsKey = keyof AppSettings

type NewSubscription = {
  name: string
  url: string
  group: string
  interval: number
}

const { invoke } = useTauri()
const toast = useToast()
const { t } = useI18n()
const { isLight, setTheme, initTheme } = useTheme()
const appName = computed(() => t('app.name'))

const settings = reactive<AppSettings>({
  autostart: false,
  system_tray: true,
  auto_switch: true,
  latency_threshold_ms: 2000,
  error_threshold_pct: 30,
  log_level: 'info',
  dns_server: '1.1.1.1',
  proxy_mode: 'system',
  bypass_ru: true,
  bypass_local: true,
  kill_switch: false,
  connection_timeout: 20,
  max_retries: 3,
  split_processes: '',
  auto_ping: true,
  ping_interval: 300,
})

const platform = ref('Unknown')
const subscriptions = ref<Subscription[]>([])
const showAddSub = ref(false)
const importInput = ref<HTMLInputElement | null>(null)
const newSub = ref<NewSubscription>({ name: '', url: '', group: '', interval: 60 })

const dnsOptions = computed(() => [
  { label: 'Cloudflare 1.1.1.1', value: '1.1.1.1' },
  { label: 'Google 8.8.8.8', value: '8.8.8.8' },
  { label: 'Quad9 9.9.9.9', value: '9.9.9.9' },
  { label: 'OpenDNS 208.67.222.222', value: '208.67.222.222' },
])
const proxyModeOptions = computed(() => [
  { label: t('settings.proxy_tunnel'), value: 'tunnel' },
  { label: t('settings.proxy_system'), value: 'system' },
  { label: t('settings.proxy_manual'), value: 'manual' },
])
const pingIntervalOptions = computed(() => [
  { label: `1 ${t('settings.minutes')}`, value: 60 },
  { label: `3 ${t('settings.minutes')}`, value: 180 },
  { label: `5 ${t('settings.minutes')}`, value: 300 },
  { label: `10 ${t('settings.minutes')}`, value: 600 },
  { label: `15 ${t('settings.minutes')}`, value: 900 },
])
const dnsLabel = computed(() => dnsOptions.value.find((option) => option.value === settings.dns_server)?.label ?? settings.dns_server)
const proxyModeLabel = computed(() => proxyModeOptions.value.find((option) => option.value === settings.proxy_mode)?.label ?? settings.proxy_mode)

const generalList = computed<Array<{ key: SettingsKey; label: string; desc: string }>>(() => [
  { key: 'autostart', label: t('settings.autostart'), desc: t('settings.autostart_desc') },
  { key: 'system_tray', label: t('settings.system_tray'), desc: t('settings.system_tray_desc') },
  { key: 'auto_switch', label: t('settings.auto_switch'), desc: t('settings.auto_switch_desc') },
  { key: 'bypass_ru', label: t('settings.bypass_ru'), desc: t('settings.bypass_ru_desc') },
  { key: 'bypass_local', label: t('settings.bypass_local'), desc: t('settings.bypass_local_desc') },
  { key: 'kill_switch', label: t('settings.kill_switch'), desc: t('settings.kill_switch_desc') },
])

function formatError(error: unknown) {
  return error instanceof Error ? error.message : String(error)
}

async function loadSettings() {
  try {
    Object.assign(settings, await invoke<Partial<AppSettings>>('get_settings'))
  } catch (error) {
    toast.error(`${t('toast.settings_load_failed')}: ${formatError(error)}`)
  }
}

async function loadSubscriptions() {
  try {
    subscriptions.value = await invoke<Subscription[]>('get_subscriptions')
  } catch (error) {
    toast.error(`${t('toast.export_failed')}: ${formatError(error)}`)
  }
}

async function saveSetting<K extends SettingsKey>(key: K, value: AppSettings[K]) {
  settings[key] = value
  try {
    await invoke('update_settings', { settings: { [key]: value } })
  } catch (error) {
    toast.error(`${t('toast.settings_save_failed')}: ${formatError(error)}`)
  }
}

function saveBoolean<K extends SettingsKey>(key: K, value: boolean) {
  void saveSetting(key, value as AppSettings[K])
}

async function applyTheme(theme: ThemeMode) {
  setTheme(theme)
  try {
    await invoke('update_settings', { settings: { theme } })
  } catch (error) {
    toast.error(`${t('toast.settings_save_failed')}: ${formatError(error)}`)
  }
}

async function addSub() {
  if (!newSub.value.url.trim()) return
  try {
    await invoke('add_subscription', {
      url: newSub.value.url.trim(),
      name: newSub.value.name.trim(),
      groupName: newSub.value.group.trim(),
      intervalMins: newSub.value.interval,
    })
    newSub.value = { name: '', url: '', group: '', interval: 60 }
    showAddSub.value = false
    await loadSubscriptions()
  } catch (error) {
    toast.error(`${t('toast.nodes_import_failed')}: ${formatError(error)}`)
  }
}

async function deleteSub(id: string) {
  try {
    await invoke('delete_subscription', { subId: id })
    await loadSubscriptions()
  } catch (error) {
    toast.error(`${t('toast.export_failed')}: ${formatError(error)}`)
  }
}

async function importSub(id: string) {
  try {
    const imported = await invoke<unknown[]>('import_subscription_url', { subId: id })
    toast.success(`${t('toast.nodes_imported')} ${imported.length}`)
  } catch (error) {
    toast.error(`${t('toast.nodes_import_failed')}: ${formatError(error)}`)
  }
}

async function doExport() {
  try {
    const { save } = await import('@tauri-apps/plugin-dialog')
    const path = await save({
      defaultPath: 'chameleon-settings-backup.json',
      filters: [{ name: 'JSON', extensions: ['json'] }],
    })
    if (!path) return
    await invoke('export_settings_to_file', { path })
    toast.success(t('toast.settings_exported'))
  } catch (error) {
    toast.error(`${t('toast.export_failed')}: ${formatError(error)}`)
  }
}

function triggerImport() {
  importInput.value?.click()
}

async function doImport(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0]
  if (!file) return
  try {
    const data = JSON.parse(await file.text())
    await invoke('import_settings', { data })
    await Promise.all([loadSettings(), loadSubscriptions()])
    toast.success(t('toast.settings_imported'))
  } catch (error) {
    toast.error(`${t('toast.export_failed')}: ${formatError(error)}`)
  }
  if (importInput.value) importInput.value.value = ''
}

onMounted(async () => {
  initTheme()
  await Promise.all([loadSettings(), loadSubscriptions()])
  try {
    const os = await import('@tauri-apps/plugin-os')
    platform.value = `${await os.platform()} ${await os.arch()}`
  } catch {
    platform.value = navigator.platform
  }
})
</script>

<style scoped>
.settings-page {
  min-height: 100%;
}
.settings-shell {
  width: 100%;
  max-width: 680px;
  margin: 0 auto;
  padding: 14px 14px 28px;
  display: grid;
  gap: 12px;
}
.settings-hero,
.settings-card {
  border-radius: 22px;
  background: linear-gradient(180deg, rgba(10,18,34,0.94), rgba(8,14,28,0.90));
  border: 1px solid rgba(126,182,255,0.10);
  box-shadow: 0 14px 38px rgba(4, 9, 20, 0.18);
}
.settings-hero {
  padding: 16px;
  background:
    radial-gradient(circle at 12% 18%, rgba(125,247,104,0.16), transparent 32%),
    radial-gradient(circle at 82% 18%, rgba(46,231,205,0.14), transparent 30%),
    radial-gradient(circle at 82% 82%, rgba(122,90,255,0.16), transparent 28%),
    linear-gradient(160deg, rgba(8,16,31,0.98), rgba(10,18,35,0.94));
}
.hero-top {
  display: flex;
  align-items: center;
  gap: 12px;
}
.hero-mark-wrap {
  position: relative;
  width: 58px;
  height: 58px;
  flex-shrink: 0;
}
.hero-mark-glow {
  position: absolute;
  inset: -12px;
  background: radial-gradient(circle, rgba(125,247,104,0.46), rgba(46,231,205,0.24) 42%, rgba(122,90,255,0.20) 66%, transparent 80%);
  filter: blur(14px);
}
.hero-mark {
  position: relative;
  z-index: 1;
  width: 58px;
  height: 58px;
  object-fit: contain;
  display: block;
}
.hero-copy {
  min-width: 0;
}
.hero-eyebrow {
  margin: 0 0 4px;
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.18em;
  color: rgba(218,240,255,0.60);
}
.hero-brand-name {
  margin: 0;
  font-size: clamp(22px, 5vw, 32px);
  line-height: 1;
  font-weight: 800;
  background: linear-gradient(90deg, #9bff6f 0%, #37efd3 42%, #5ba8ff 70%, #a56fff 100%);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}
.hero-brand-subtitle {
  margin: 6px 0 0;
  font-size: 10px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: rgba(218,240,255,0.42);
}
.hero-description {
  margin: 12px 0 0;
  color: rgba(223,236,255,0.72);
  line-height: 1.55;
  font-size: 12px;
}
.hero-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 12px;
}
.hero-chip {
  padding: 8px 11px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 700;
  background: rgba(255,255,255,0.04);
  border: 1px solid rgba(255,255,255,0.08);
  color: #eefbff;
}
.settings-card {
  padding: 16px;
}
.section-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 12px;
}
.compact-head {
  margin-bottom: 10px;
}
.section-kicker {
  margin: 0 0 5px;
  font-size: 10px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--text-muted);
}
.section-title {
  margin: 0;
  font-size: 17px;
  line-height: 1.2;
  color: var(--text);
}
.section-action-btn {
  flex-shrink: 0;
}
.btn {
  appearance: none;
  min-height: 38px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 7px;
  padding: 9px 14px;
  border-radius: 14px;
  border: 1px solid rgba(150,214,255,0.12);
  background:
    linear-gradient(180deg, rgba(255,255,255,0.055), rgba(255,255,255,0.025)),
    rgba(8,16,30,0.74);
  color: rgba(235,248,255,0.78);
  font-size: 12px;
  font-weight: 800;
  line-height: 1;
  text-decoration: none;
  cursor: pointer;
  box-shadow: inset 0 1px 0 rgba(255,255,255,0.06), 0 12px 28px rgba(2,8,20,0.16);
  transition: transform .18s ease, border-color .18s ease, background .18s ease, color .18s ease, box-shadow .18s ease, opacity .18s ease;
}
.btn svg,
.btn-icon svg {
  flex-shrink: 0;
}
.btn:hover:not(:disabled) {
  transform: translateY(-1px);
  border-color: rgba(125,247,104,0.24);
  color: #f3fff1;
  background:
    linear-gradient(135deg, rgba(125,247,104,0.12), rgba(46,231,205,0.08), rgba(122,90,255,0.08)),
    rgba(8,18,32,0.90);
  box-shadow: inset 0 1px 0 rgba(255,255,255,0.08), 0 16px 34px rgba(2,8,20,0.22), 0 0 24px rgba(46,231,205,0.08);
}
.btn:active:not(:disabled) {
  transform: translateY(0) scale(.98);
}
.btn:disabled {
  opacity: .44;
  cursor: not-allowed;
  box-shadow: none;
}
.btn-primary {
  border-color: rgba(125,247,104,0.30);
  background: linear-gradient(135deg, #9bff6f 0%, #37efd3 56%, #5ba8ff 100%);
  color: #07150f;
  box-shadow: 0 14px 30px rgba(46,231,205,0.16), inset 0 1px 0 rgba(255,255,255,0.42);
}
.btn-primary:hover:not(:disabled) {
  border-color: rgba(155,255,111,0.56);
  background: linear-gradient(135deg, #b7ff90 0%, #4df6de 54%, #7bbcff 100%);
  color: #06130d;
}
.btn-ghost {
  background: rgba(255,255,255,0.035);
  border-color: rgba(255,255,255,0.075);
  color: rgba(223,236,255,0.70);
}
.btn-ghost:hover:not(:disabled) {
  background: linear-gradient(135deg, rgba(125,247,104,0.10), rgba(46,231,205,0.075));
}
.btn-icon {
  appearance: none;
  width: 34px;
  height: 34px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 12px;
  border: 1px solid rgba(150,214,255,0.12);
  background: rgba(255,255,255,0.04);
  color: rgba(223,236,255,0.66);
  cursor: pointer;
  box-shadow: inset 0 1px 0 rgba(255,255,255,0.055);
  transition: transform .18s ease, border-color .18s ease, background .18s ease, color .18s ease, box-shadow .18s ease;
}
.btn-icon:hover {
  transform: translateY(-1px);
  border-color: rgba(46,231,205,0.26);
  background: rgba(46,231,205,0.10);
  color: #dffeff;
  box-shadow: 0 10px 22px rgba(2,8,20,0.18), 0 0 18px rgba(46,231,205,0.08);
}
.btn-icon.danger {
  color: rgba(255,137,154,0.78);
  border-color: rgba(255,91,125,0.12);
}
.btn-icon.danger:hover {
  border-color: rgba(255,91,125,0.30);
  background: rgba(255,91,125,0.10);
  color: #ffdce3;
}
.stat-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
}
.mini-stat {
  padding: 13px;
  border-radius: 18px;
  background: rgba(255,255,255,0.035);
  border: 1px solid rgba(255,255,255,0.08);
  min-width: 0;
}
.mini-stat-label {
  display: block;
  font-size: 10px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: rgba(223,236,255,0.54);
  margin-bottom: 8px;
}
.mini-stat-value {
  display: block;
  font-size: clamp(18px, 4vw, 22px);
  color: #fff;
}
.stack-list,
.form-stack,
.subscription-list,
.about-list,
.backup-actions {
  display: grid;
  gap: 10px;
}
.setting-item {
  border-radius: 18px;
  background: rgba(255,255,255,0.025);
  border: 1px solid rgba(255,255,255,0.06);
  padding: 13px;
}
.setting-item-toggle {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}
.setting-copy {
  min-width: 0;
}
.setting-label {
  margin: 0 0 4px;
  font-size: 14px;
  color: var(--text);
}
.setting-desc {
  margin: 0;
  color: var(--text-secondary);
  line-height: 1.5;
  font-size: 12px;
}
.dual-inputs {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}
.compact-field,
.proxy-box,
.subscription-card,
.empty-state,
.about-row {
  border-radius: 18px;
  background: rgba(255,255,255,0.025);
  border: 1px solid rgba(255,255,255,0.06);
}
.compact-field {
  padding: 12px;
}
.inline-hint {
  margin-top: 6px;
  font-size: 11px;
  color: #9edbff;
}
.top-gap {
  margin-top: 10px;
}
.top-gap-sm {
  margin-top: 8px;
}
.subscription-card {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 10px;
  padding: 13px;
}
.subscription-copy {
  min-width: 0;
}
.subscription-name {
  display: block;
  color: var(--text);
  margin-bottom: 4px;
  word-break: break-word;
}
.subscription-url {
  display: block;
  color: var(--text-muted);
  font-size: 11px;
  line-height: 1.45;
  overflow-wrap: anywhere;
}
.subscription-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 8px;
}
.subscription-actions {
  display: flex;
  gap: 6px;
  align-items: flex-start;
  flex-wrap: wrap;
}
.theme-switcher {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
  margin-bottom: 14px;
  padding: 8px;
  border-radius: 18px;
  background: rgba(255,255,255,0.025);
  border: 1px solid rgba(255,255,255,0.05);
}
.theme-pill {
  min-height: 42px;
  border-radius: 14px;
  border: 1px solid transparent;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all .2s ease;
  font-weight: 700;
}
.theme-pill.active {
  background: linear-gradient(90deg, rgba(125,247,104,0.16), rgba(46,231,205,0.12), rgba(122,90,255,0.14));
  border-color: rgba(125,247,104,0.16);
  color: var(--text);
}
.proxy-box {
  padding: 13px;
}
.proxy-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.proxy-row + .proxy-row {
  margin-top: 10px;
}
.proxy-val {
  font-weight: 700;
  text-align: right;
  word-break: break-word;
}
.about-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 12px 14px;
}
.about-label {
  color: var(--text-muted);
}
.about-value {
  color: var(--text);
  text-align: right;
  word-break: break-word;
}
.empty-state {
  padding: 16px;
  text-align: center;
  border-style: dashed;
}
.wide-action-btn,
.modal-btn {
  justify-content: center;
}
.hidden-input {
  display: none;
}
.modal-overlay {
  position: fixed;
  inset: 0;
  z-index: 100;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: rgba(3, 8, 20, 0.78);
  backdrop-filter: blur(14px);
}
.modal-panel {
  width: min(100%, 430px);
  border-radius: 24px;
  padding: 20px;
  background: linear-gradient(180deg, rgba(10,18,35,0.98), rgba(8,15,30,0.96));
  border: 1px solid rgba(255,255,255,0.08);
  box-shadow: 0 20px 60px rgba(0,0,0,0.35);
}
.modal-title {
  margin: 0 0 14px;
}
.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 18px;
}

@media (max-width: 900px) {
  .settings-shell {
    max-width: none;
  }
  .section-head-action {
    flex-direction: column;
    align-items: stretch;
  }
  .section-action-btn {
    width: 100%;
  }
}

@media (max-width: 720px) {
  .settings-shell {
    padding: 12px 12px 24px;
  }
  .stat-grid,
  .dual-inputs,
  .theme-switcher {
    grid-template-columns: 1fr;
  }
  .subscription-card,
  .proxy-row,
  .about-row,
  .modal-actions {
    grid-template-columns: 1fr;
  }
  .subscription-card {
    grid-template-columns: 1fr;
  }
  .subscription-actions {
    justify-content: flex-start;
  }
  .setting-item-toggle,
  .proxy-row,
  .about-row,
  .modal-actions {
    flex-direction: column;
    align-items: stretch;
  }
  .wide-action-btn,
  .modal-btn {
    width: 100%;
  }
}

@media (max-width: 520px) {
  .settings-hero,
  .settings-card,
  .modal-panel {
    padding: 14px;
    border-radius: 18px;
  }
  .hero-top {
    align-items: flex-start;
    flex-direction: column;
  }
  .hero-mark-wrap,
  .hero-mark {
    width: 52px;
    height: 52px;
  }
  .hero-chips {
    display: grid;
    grid-template-columns: 1fr;
  }
  .hero-chip,
  .setting-item,
  .compact-field,
  .proxy-box,
  .subscription-card,
  .about-row {
    border-radius: 16px;
  }
  .section-title {
    font-size: 16px;
  }
}

@media (max-width: 400px) {
  .settings-shell {
    padding: 10px 10px 20px;
  }
  .settings-hero,
  .settings-card,
  .modal-panel {
    padding: 12px;
  }
  .hero-brand-subtitle,
  .mini-stat-label,
  .section-kicker {
    letter-spacing: 0.12em;
  }
}

:global(html.light) .settings-page {
  background:
    radial-gradient(circle at 8% 8%, rgba(58,191,103,0.14), transparent 28%),
    radial-gradient(circle at 92% 12%, rgba(36,143,180,0.12), transparent 26%),
    linear-gradient(180deg, #f7fbff, #edf6f5);
}
:global(html.light) .settings-hero,
:global(html.light) .settings-card {
  background: linear-gradient(180deg, rgba(255,255,255,0.92), rgba(244,250,249,0.88));
  border-color: rgba(42, 109, 120, 0.14);
  box-shadow: 0 14px 36px rgba(39, 75, 86, 0.10);
}
:global(html.light) .settings-hero {
  background:
    radial-gradient(circle at 14% 18%, rgba(58,191,103,0.16), transparent 34%),
    radial-gradient(circle at 84% 22%, rgba(46,158,177,0.14), transparent 30%),
    linear-gradient(150deg, rgba(255,255,255,0.96), rgba(236,248,245,0.94));
}
:global(html.light) .hero-eyebrow,
:global(html.light) .hero-brand-subtitle,
:global(html.light) .hero-description,
:global(html.light) .mini-stat-label,
:global(html.light) .section-kicker {
  color: rgba(16, 48, 60, 0.56);
}
:global(html.light) .hero-chip,
:global(html.light) .mini-stat,
:global(html.light) .setting-item,
:global(html.light) .compact-field,
:global(html.light) .proxy-box,
:global(html.light) .subscription-card,
:global(html.light) .empty-state,
:global(html.light) .about-row,
:global(html.light) .theme-switcher {
  background: rgba(255,255,255,0.58);
  border-color: rgba(42,109,120,0.11);
  color: rgba(16,33,58,0.78);
}
.form-group label {
  display: block;
  margin-bottom: 7px;
  font-size: 10px;
  font-weight: 800;
  letter-spacing: .12em;
  text-transform: uppercase;
  color: rgba(184, 208, 232, 0.58);
}
.form-input {
  min-height: 42px;
  border-radius: 12px;
  padding: 10px 12px;
  background:
    linear-gradient(180deg, rgba(255,255,255,0.055), rgba(255,255,255,0.025)),
    rgba(8,16,30,0.72);
  border: 1px solid rgba(150,214,255,0.12);
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 700;
  outline: none;
  transition: border-color .18s ease, background .18s ease, box-shadow .18s ease;
}
.form-input:hover,
.form-input:focus {
  border-color: rgba(125,247,104,0.24);
  background:
    linear-gradient(180deg, rgba(125,247,104,0.08), rgba(46,231,205,0.04)),
    rgba(8,18,32,0.86);
  box-shadow: 0 0 0 3px rgba(125,247,104,0.055);
}
.form-input::placeholder {
  color: rgba(184,208,232,0.35);
}
.form-input[type='number'] {
  font-variant-numeric: tabular-nums;
}
:global(html.light) .form-group label {
  color: rgba(16,48,60,0.54);
}
:global(html.light) .form-input {
  background: linear-gradient(180deg, rgba(255,255,255,0.95), rgba(244,250,248,0.88));
  border-color: rgba(42,109,120,0.14);
  color: rgba(16,33,58,0.78);
}
:global(html.light) .form-input:hover,
:global(html.light) .form-input:focus {
  border-color: rgba(58,191,103,0.30);
  background: #fff;
  box-shadow: 0 0 0 3px rgba(58,191,103,0.10);
}
.backup-actions {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}
.wide-action-btn {
  min-height: 44px;
  border-radius: 12px;
  background: linear-gradient(90deg, rgba(125,247,104,0.12), rgba(46,231,205,0.08)) !important;
  border-color: rgba(125,247,104,0.16) !important;
  color: #bdfbb6 !important;
  font-weight: 800;
}
.wide-action-btn:hover {
  transform: translateY(-1px);
  background: linear-gradient(90deg, rgba(125,247,104,0.20), rgba(46,231,205,0.14)) !important;
  color: #f4fff3 !important;
}
:global(html.light) .wide-action-btn {
  background: linear-gradient(90deg, rgba(58,191,103,0.12), rgba(46,158,177,0.09)) !important;
  border-color: rgba(58,191,103,0.22) !important;
  color: #207645 !important;
}
:global(html.light) .wide-action-btn:hover {
  background: linear-gradient(90deg, rgba(58,191,103,0.18), rgba(46,158,177,0.13)) !important;
  color: #113f2a !important;
}
.settings-page.is-light {
  background:
    radial-gradient(circle at 8% 8%, rgba(58,191,103,0.16), transparent 28%),
    radial-gradient(circle at 92% 12%, rgba(36,143,180,0.13), transparent 26%),
    radial-gradient(circle at 52% 100%, rgba(122,90,255,0.08), transparent 34%),
    linear-gradient(180deg, #f8fcff, #edf8f5) !important;
  color: rgba(16,33,58,0.82);
}
.settings-page.is-light .settings-hero,
.settings-page.is-light .settings-card {
  background: linear-gradient(180deg, rgba(255,255,255,0.94), rgba(244,250,249,0.90)) !important;
  border-color: rgba(42,109,120,0.14) !important;
  box-shadow: 0 16px 40px rgba(39,75,86,0.10) !important;
}
.settings-page.is-light .settings-hero {
  background:
    radial-gradient(circle at 14% 18%, rgba(58,191,103,0.17), transparent 34%),
    radial-gradient(circle at 84% 22%, rgba(46,158,177,0.15), transparent 30%),
    linear-gradient(150deg, rgba(255,255,255,0.97), rgba(236,248,245,0.95)) !important;
}
.settings-page.is-light .hero-mark-glow {
  background: radial-gradient(circle, rgba(58,191,103,0.34), rgba(46,158,177,0.18) 42%, rgba(122,90,255,0.12) 66%, transparent 80%);
}
.settings-page.is-light .hero-eyebrow,
.settings-page.is-light .hero-brand-subtitle,
.settings-page.is-light .hero-description,
.settings-page.is-light .mini-stat-label,
.settings-page.is-light .section-kicker,
.settings-page.is-light .setting-desc,
.settings-page.is-light .subscription-url,
.settings-page.is-light .about-label {
  color: rgba(16,48,60,0.56) !important;
}
.settings-page.is-light .section-title,
.settings-page.is-light .setting-label,
.settings-page.is-light .subscription-name,
.settings-page.is-light .about-value,
.settings-page.is-light .mini-stat-value,
.settings-page.is-light .modal-title {
  color: rgba(16,33,58,0.88) !important;
}
.settings-page.is-light .hero-chip,
.settings-page.is-light .mini-stat,
.settings-page.is-light .setting-item,
.settings-page.is-light .compact-field,
.settings-page.is-light .proxy-box,
.settings-page.is-light .subscription-card,
.settings-page.is-light .empty-state,
.settings-page.is-light .about-row,
.settings-page.is-light .theme-switcher {
  background: rgba(255,255,255,0.66) !important;
  border-color: rgba(42,109,120,0.12) !important;
  color: rgba(16,33,58,0.78) !important;
}
.settings-page.is-light .tag {
  background: rgba(58,191,103,0.10) !important;
  color: rgba(16,48,60,0.62) !important;
}
.settings-page.is-light .inline-hint,
.settings-page.is-light .proxy-val {
  color: #258151 !important;
}
.settings-page.is-light .btn {
  background: linear-gradient(180deg, rgba(255,255,255,0.96), rgba(244,250,248,0.88)) !important;
  border-color: rgba(42,109,120,0.14) !important;
  color: rgba(16,33,58,0.74) !important;
  box-shadow: inset 0 1px 0 rgba(255,255,255,0.72), 0 12px 26px rgba(39,75,86,0.08) !important;
}
.settings-page.is-light .btn:hover:not(:disabled) {
  background: linear-gradient(135deg, rgba(58,191,103,0.13), rgba(46,158,177,0.10), rgba(122,90,255,0.06)), #fff !important;
  border-color: rgba(58,191,103,0.26) !important;
  color: rgba(16,33,58,0.88) !important;
}
.settings-page.is-light .btn-primary {
  background: linear-gradient(135deg, #7ee26f 0%, #35cfc1 58%, #5ea7ed 100%) !important;
  border-color: rgba(58,191,103,0.30) !important;
  color: #062416 !important;
  box-shadow: 0 14px 28px rgba(46,158,177,0.14), inset 0 1px 0 rgba(255,255,255,0.55) !important;
}
.settings-page.is-light .btn-ghost {
  background: rgba(255,255,255,0.66) !important;
  color: rgba(16,48,60,0.62) !important;
}
.settings-page.is-light .btn-icon {
  background: rgba(255,255,255,0.68) !important;
  border-color: rgba(42,109,120,0.13) !important;
  color: rgba(16,48,60,0.62) !important;
  box-shadow: 0 10px 22px rgba(39,75,86,0.08) !important;
}
.settings-page.is-light .btn-icon:hover {
  background: rgba(46,158,177,0.10) !important;
  border-color: rgba(46,158,177,0.24) !important;
  color: rgba(16,33,58,0.84) !important;
}
.settings-page.is-light .btn-icon.danger {
  color: rgba(180,50,68,0.76) !important;
  border-color: rgba(205,54,67,0.16) !important;
}
.settings-page.is-light .btn-icon.danger:hover {
  background: rgba(205,54,67,0.09) !important;
  border-color: rgba(205,54,67,0.26) !important;
}
.settings-page.is-light .theme-pill {
  color: rgba(16,48,60,0.58) !important;
}
.settings-page.is-light .theme-pill:hover {
  background: rgba(46,158,177,0.08) !important;
  color: rgba(16,33,58,0.82) !important;
}
.settings-page.is-light .theme-pill.active {
  background: linear-gradient(90deg, rgba(58,191,103,0.15), rgba(46,158,177,0.11), rgba(122,90,255,0.08)) !important;
  border-color: rgba(58,191,103,0.22) !important;
  color: #1d6f43 !important;
}
.settings-page.is-light .form-group label {
  color: rgba(16,48,60,0.54) !important;
}
.settings-page.is-light .form-input {
  background: linear-gradient(180deg, rgba(255,255,255,0.96), rgba(244,250,248,0.90)) !important;
  border-color: rgba(42,109,120,0.14) !important;
  color: rgba(16,33,58,0.80) !important;
}
.settings-page.is-light .form-input:hover,
.settings-page.is-light .form-input:focus {
  border-color: rgba(58,191,103,0.30) !important;
  background: #fff !important;
  box-shadow: 0 0 0 3px rgba(58,191,103,0.10) !important;
}
.settings-page.is-light .form-input::placeholder {
  color: rgba(16,48,60,0.34) !important;
}
.settings-page.is-light .wide-action-btn {
  background: linear-gradient(90deg, rgba(58,191,103,0.13), rgba(46,158,177,0.10)) !important;
  border-color: rgba(58,191,103,0.24) !important;
  color: #207645 !important;
}
.settings-page.is-light .wide-action-btn:hover {
  background: linear-gradient(90deg, rgba(58,191,103,0.20), rgba(46,158,177,0.14)) !important;
  color: #113f2a !important;
}
.settings-page.is-light .modal-overlay {
  background: rgba(233,246,245,0.62) !important;
}
.settings-page.is-light .modal-panel {
  background: linear-gradient(180deg, rgba(255,255,255,0.97), rgba(244,250,248,0.95)) !important;
  border-color: rgba(42,109,120,0.14) !important;
  box-shadow: 0 24px 54px rgba(39,75,86,0.18) !important;
}
</style>
