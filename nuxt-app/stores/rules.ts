import { defineStore } from 'pinia'

export interface RouteRule {
  id: string
  rule_type: string
  value: string
  node_id: string
  priority: number
  enabled: boolean
}

export interface RuleTestResult {
  matched: boolean
  rule_id?: string
  rule_type?: string
  rule_value?: string
  node_id?: string
  priority?: number
  reason?: string
}

export const useRulesStore = defineStore('rules', {
  state: () => ({
    rules: [] as RouteRule[],
    loading: false,
    testResult: null as RuleTestResult | null,
    editRule: null as RouteRule | null,
    showEditor: false,
  }),

  getters: {
    sortedRules: (state) => [...state.rules].sort((a, b) => b.priority - a.priority),
    enabledRules: (state) => state.rules.filter((r) => r.enabled),
  },

  actions: {
    setRules(rules: RouteRule[]) {
      this.rules = rules
    },
    setLoading(val: boolean) {
      this.loading = val
    },
    setTestResult(result: RuleTestResult | null) {
      this.testResult = result
    },
    setEditRule(rule: RouteRule | null) {
      this.editRule = rule
    },
    setShowEditor(val: boolean) {
      this.showEditor = val
    },
  },
})
