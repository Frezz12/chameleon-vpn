import { useTauri } from './useTauri'

export function useVPN() {
  const store = useVpnStore()
  const { invoke, listen } = useTauri()
  const toast = useToast()

  async function connect(nodeId: string) {
    store.setConnecting(true)
    try {
      await invoke('vpn_connect', { nodeId })
      toast.success('VPN connected')
    } catch (e: any) {
      toast.error(`Failed to connect: ${e}`)
    } finally {
      store.setConnecting(false)
    }
  }

  async function disconnect() {
    try {
      await invoke('vpn_disconnect')
      toast.info('VPN disconnected')
    } catch (e: any) {
      toast.error(`Failed to disconnect: ${e}`)
    }
  }

  async function switchNode(nodeId: string) {
    store.setConnecting(true)
    try {
      await invoke('vpn_switch_node', { nodeId })
      toast.success('Switched node')
    } catch (e: any) {
      toast.error(`Failed to switch: ${e}`)
    } finally {
      store.setConnecting(false)
    }
  }

  async function refreshStatus() {
    try {
      const status = await invoke<VpnStatus>('vpn_status')
      store.setStatus(status)
    } catch (e) {
      console.error('Failed to get status:', e)
    }
  }

  function setupListeners() {
    listen<VpnStatus>('vpn-status', (status) => {
      store.setStatus(status)
    })

    listen<{ level: string; message: string; timestamp: string }>('vpn-log', (log) => {
      store.addLog(log)
    })
  }

  return {
    connect,
    disconnect,
    switchNode,
    refreshStatus,
    setupListeners,
  }
}
