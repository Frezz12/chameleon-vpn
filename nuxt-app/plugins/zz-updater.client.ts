import { useTauri } from '~/composables/useTauri'
import { useToast } from '~/composables/useToast'

const CHECK_INTERVAL_MS = 6 * 60 * 60 * 1000
const LAST_CHECK_KEY = 'chameleon_last_update_check'

export default defineNuxtPlugin(() => {
  if (!import.meta.client) return

  const { isTauri } = useTauri()
  if (!isTauri) return

  const toast = useToast()

  window.setTimeout(() => {
    checkForUpdates(toast).catch((error) => {
      console.warn('Update check failed', error)
    })
  }, 3500)
})

async function checkForUpdates(toast: ReturnType<typeof useToast>) {
  const lastCheck = Number(localStorage.getItem(LAST_CHECK_KEY) || 0)
  if (Date.now() - lastCheck < CHECK_INTERVAL_MS) return
  localStorage.setItem(LAST_CHECK_KEY, String(Date.now()))

  const [{ check }, { relaunch }] = await Promise.all([
    import('@tauri-apps/plugin-updater'),
    import('@tauri-apps/plugin-process'),
  ])

  const update = await check()
  if (!update) return

  toast.info(`Доступно обновление Chameleon ${update.version}. Скачиваю и устанавливаю...`)
  await update.downloadAndInstall()
  toast.success('Обновление установлено. Chameleon сейчас перезапустится.')

  window.setTimeout(() => {
    relaunch().catch(() => {})
  }, 1500)
}
