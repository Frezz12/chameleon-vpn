import { defineStore } from 'pinia'

export type ConnectionState = 'disconnected' | 'connecting' | 'connected' | 'disconnecting'

export interface VpnStatus {
  connected: boolean
  current_node_id: string | null
  current_node_name: string | null
  connected_at: number | null
  download_speed: number
  upload_speed: number
  total_download: number
  total_upload: number
  latency_ms: number | null
}

export interface LogEntry {
  timestamp: string
  level: string
  message: string
}

export const useVpnStore = defineStore('vpn', {
  state: () => ({
    connectionState: 'disconnected' as ConnectionState,
    status: {
      connected: false,
      current_node_id: null,
      current_node_name: null,
      connected_at: null,
      download_speed: 0,
      upload_speed: 0,
      total_download: 0,
      total_upload: 0,
      latency_ms: null,
    } as VpnStatus,
    logs: [] as LogEntry[],
    connecting: false,
    downloadTraffic: 0,
    uploadTraffic: 0,
  }),

  getters: {
    isConnected: (state) => state.status.connected,
    recentLogs: (state) => state.logs.slice(-5),
    isTransitioning: (state) => state.connectionState === 'connecting' || state.connectionState === 'disconnecting',
  },

  actions: {
    setStatus(status: VpnStatus) {
      this.status = status
      this.downloadTraffic = status.total_download
      this.uploadTraffic = status.total_upload
    },

    addLog(log: LogEntry) {
      this.logs.push(log)
      if (this.logs.length > 200) {
        this.logs.splice(0, this.logs.length - 200)
      }
    },

    setConnecting(val: boolean) {
      this.connecting = val
    },

    setConnectionState(state: ConnectionState) {
      this.connectionState = state
      if (state === 'disconnected') {
        this.downloadTraffic = 0
        this.uploadTraffic = 0
      }
    },

    startTrafficSimulation() {
      this.downloadTraffic = this.status.total_download
      this.uploadTraffic = this.status.total_upload
    },

    stopTrafficSimulation() {
      this.downloadTraffic = this.status.total_download
      this.uploadTraffic = this.status.total_upload
    },
  },
})
