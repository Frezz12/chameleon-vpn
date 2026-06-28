import { useTauri } from './useTauri'
import type { VpnNode } from '~/stores/nodes'

let listenersSetup = false

export function useNodes() {
  const store = useNodesStore()
  const toast = useToast()
  const { t } = useI18n()
  const { invoke, listen } = useTauri()

  function ensureListeners() {
    if (listenersSetup) return
    listenersSetup = true

    listen<{ current: number; total: number; node_id: string; status: 'testing' | 'done' | 'failed'; latency_ms?: number | null }>(
      'speed-test-progress',
      (progress) => {
        store.setTestProgress(progress)
        store.updateNodeTestResult(progress.node_id, progress.status, progress.latency_ms ?? null)
      },
    )

    listen<{ results: Array<{ node_id: string; success: boolean; latency_ms: number | null }> }>('speed-test-complete', (payload) => {
      for (const result of payload.results) {
        store.updateNodeTestResult(result.node_id, result.success ? 'done' : 'failed', result.latency_ms)
      }
      store.setTestingAll(false)
      toast.success(t('toast.speed_test_complete'))
      fetchNodes()
    })
  }

  async function fetchNodes() {
    store.setLoading(true)
    try {
      const nodes = await invoke<VpnNode[]>('get_nodes')
      store.setNodes(nodes)
    } catch (e: any) {
      toast.error(`${t('toast.nodes_load_failed')}: ${e}`)
    } finally {
      store.setLoading(false)
    }
  }

  async function addNode(node: Partial<VpnNode>) {
    try {
      await invoke('add_node', { node })
      toast.success(t('toast.node_added'))
      await fetchNodes()
    } catch (e: any) {
      toast.error(`${t('toast.node_add_failed')}: ${e}`)
    }
  }

  async function deleteNode(nodeId: string) {
    try {
      await invoke('delete_node', { nodeId })
      toast.success(t('toast.node_deleted'))
      await fetchNodes()
    } catch (e: any) {
      toast.error(`${t('toast.node_delete_failed')}: ${e}`)
    }
  }

  async function testSpeed(nodeId: string) {
    ensureListeners()
    try {
      const result = await invoke<any>('test_node_speed', { nodeId })
      store.updateNodeTestResult(nodeId, result.success ? 'done' : 'failed', result.latency_ms ?? null)
      return result
    } catch (e: any) {
      toast.error(`${t('toast.speed_test_failed')}: ${e}`)
      return null
    }
  }

  async function testAll() {
    ensureListeners()
    store.setTestingAll(true)
    try {
      await invoke('test_all_nodes_speed')
    } catch (e: any) {
      toast.error(`${t('toast.speed_test_failed')}: ${e}`)
      store.setTestingAll(false)
    }
  }

  async function importSubscription(url: string, groupName?: string) {
    try {
      const nodes = await invoke<VpnNode[]>('import_subscription', { url, groupName: groupName || null })
      await fetchNodes()
      return nodes
    } catch (e: any) {
      toast.error(`${t('toast.nodes_import_failed')}: ${e}`)
      return []
    }
  }

  function setupListeners() {
    ensureListeners()
  }

  return {
    fetchNodes,
    addNode,
    deleteNode,
    testSpeed,
    testAll,
    importSubscription,
    setupListeners,
  }
}


