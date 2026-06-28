import { defineStore } from 'pinia'

export interface NodeConfig {
  [key: string]: unknown
}

export interface VpnNode {
  id: string
  name: string
  remark?: string
  protocol: string
  server: string
  port: number
  config: NodeConfig
  latency_ms: number | null
  uptime: number | null
  error_count: number
  last_test_at: string | null
  enabled: boolean
  subscription_id?: string | null
  group_name?: string
  test_status?: 'testing' | 'done' | 'failed' | 'untested'
}

export interface Subscription {
  id: string
  url: string
  name: string
  group_name: string
  interval_mins: number
  enabled: boolean
}

export const useNodesStore = defineStore('nodes', {
  state: () => ({
    nodes: [] as VpnNode[],
    subscriptions: [] as Subscription[],
    loading: false,
    testingAll: false,
    testProgress: { current: 0, total: 0, node_id: '', status: '' },
    filterProtocol: '',
    searchQuery: '',
    selectedNodeId: null as string | null,
  }),

  getters: {
    filteredNodes: (state) => {
      let result = state.nodes
      if (state.filterProtocol) {
        result = result.filter((n) => n.protocol === state.filterProtocol)
      }
      if (state.searchQuery) {
        const q = state.searchQuery.toLowerCase()
        result = result.filter((n) => n.name.toLowerCase().includes(q) || n.server.toLowerCase().includes(q))
      }
      return result
    },
    selectedNode: (state) => state.nodes.find((n) => n.id === state.selectedNodeId) || null,
    protocols: (state) => [...new Set(state.nodes.map((n) => n.protocol))],
    groupedNodes: (state) => {
      const groups: Record<string, VpnNode[]> = {}
      for (const node of state.nodes) {
        const g = node.group_name?.trim() || 'Default'
        if (!groups[g]) groups[g] = []
        groups[g].push(node)
      }
      return groups
    },
  },

  actions: {
    setNodes(nodes: VpnNode[]) {
      const previous = new Map(this.nodes.map((node) => [node.id, node]))
      this.nodes = nodes.map((node) => ({
        ...node,
        test_status: previous.get(node.id)?.test_status ?? (node.latency_ms != null ? 'done' : 'untested'),
      }))
    },
    setSubscriptions(subs: Subscription[]) {
      this.subscriptions = subs
    },
    setLoading(val: boolean) {
      this.loading = val
    },
    setTestingAll(val: boolean) {
      this.testingAll = val
    },
    setTestProgress(progress: { current: number; total: number; node_id: string; status: string; latency_ms?: number | null }) {
      this.testProgress = progress
    },
    setFilterProtocol(protocol: string) {
      this.filterProtocol = protocol
    },
    setSearchQuery(query: string) {
      this.searchQuery = query
    },
    setSelectedNodeId(id: string | null) {
      this.selectedNodeId = id
    },
    updateNodeLatency(nodeId: string, latencyMs: number) {
      const node = this.nodes.find((n) => n.id === nodeId)
      if (node) {
        node.latency_ms = latencyMs
        node.test_status = 'done'
      }
    },
    updateNodeTestResult(nodeId: string, status: 'testing' | 'done' | 'failed', latencyMs?: number | null) {
      const node = this.nodes.find((n) => n.id === nodeId)
      if (node) {
        node.test_status = status
        node.latency_ms = status === 'done' && latencyMs != null ? latencyMs : null
      }
    },
  },
})



