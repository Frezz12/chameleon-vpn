import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { UnlistenFn } from '@tauri-apps/api/event'

export function useTauri() {
  const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

  async function invokeCommand<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
    if (!isTauri) {
      console.warn(`Tauri not available, cannot invoke: ${cmd}`)
      throw new Error('Tauri not available')
    }
    return invoke<T>(cmd, args)
  }

  async function listenEvent<T>(event: string, handler: (payload: T) => void): Promise<UnlistenFn> {
    if (!isTauri) {
      console.warn(`Tauri not available, cannot listen: ${event}`)
      return () => {}
    }
    return listen<T>(event, (e) => handler(e.payload))
  }

  return {
    isTauri,
    invoke: invokeCommand,
    listen: listenEvent,
  }
}
