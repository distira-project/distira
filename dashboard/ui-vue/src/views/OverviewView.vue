<template>
  <div>
    <header class="view-header">
      <div>
        <h2>Overview</h2>
        <p class="muted">A visual control plane for AI traffic and context optimization.</p>
      </div>
    </header>
    <div class="metrics-grid">
      <MetricCard label="Raw Tokens" :value="metrics.rawTokens.toLocaleString()" hint="Estimated before compilation" accent="warn">
        <SparklineChart :data="rawHistory" color="#ffa940" :height="40" />
      </MetricCard>
      <MetricCard label="Compiled Tokens" :value="metrics.compiledTokens.toLocaleString()" hint="Estimated after compiler" accent="primary">
        <SparklineChart :data="compiledHistory" color="var(--primary)" :height="40" />
      </MetricCard>
      <MetricCard label="Memory Reused" :value="metrics.memoryReusedTokens.toLocaleString()" hint="Tokens not re-sent" accent="secondary">
        <SparklineChart :data="memoryHistory" color="var(--secondary)" :height="40" />
      </MetricCard>
      <MetricCard label="Local Routing" :value="metrics.localRatio + '%'" hint="Requests kept local" accent="accent">
        <SparklineChart :data="localHistory" color="var(--accent)" :height="40" />
      </MetricCard>
      <MetricCard label="Cache Saved Tokens" :value="metrics.cacheSavedTokens.toLocaleString()" hint="Provider tokens avoided via cached responses" accent="primary">
        <SparklineChart :data="cacheSavedHistory" color="var(--primary)" :height="40" />
      </MetricCard>
    </div>
    <div class="two-col">
      <EfficiencyGauge :score="metrics.efficiencyScore" />
      <FlowVisualizer />
    </div>
    <section class="card chart-section">
      <h3>Token Trends (24h)</h3>
      <TvChart :series="trendSeries" :labels="trendLabels" :height="220" />
    </section>

    <section class="card model-efficiency-section">
      <h3>Live AI Efficiency by Model</h3>
      <p class="muted">Per-model efficiency score with Sovereign Routing visibility.</p>
      <div class="model-table-wrap">
        <table class="model-table">
          <thead>
            <tr>
              <th>Model</th>
              <th>Provider</th>
              <th>Requests</th>
              <th>Efficiency</th>
              <th>Sovereign</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="entry in modelRows" :key="entry.key">
              <td>{{ entry.model }}</td>
              <td>{{ entry.provider }}</td>
              <td>{{ entry.requests }}</td>
              <td>
                <strong>{{ entry.efficiency }}%</strong>
              </td>
              <td>
                <span class="sovereign-pill" :class="entry.sovereignClass">{{ entry.sovereignLabel }}</span>
              </td>
            </tr>
            <tr v-if="!modelRows.length">
              <td colspan="5" class="muted">No model data yet. Send a few requests to populate live stats.</td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useMetricsStore } from '../store/metrics'
import MetricCard from '../components/MetricCard.vue'
import EfficiencyGauge from '../components/EfficiencyGauge.vue'
import FlowVisualizer from '../components/FlowVisualizer.vue'
import SparklineChart from '../components/SparklineChart.vue'
import TvChart from '../components/TvChart.vue'

const metrics = useMetricsStore()

// Sparklines: direct SSE history arrays (reactive)
const rawHistory = computed(() => metrics.historyRaw.length ? metrics.historyRaw : [0])
const compiledHistory = computed(() => metrics.historyCompiled.length ? metrics.historyCompiled : [0])
const memoryHistory = computed(() => metrics.historyReused.length ? metrics.historyReused : [0])
const localHistory = computed(() => {
  const total = metrics.routesLocal + metrics.routesCloud + metrics.routesMidtier
  if (!total || !metrics.historyRaw.length) return [0]
  // Derive a local-ratio sparkline from history length (same cardinality)
  return metrics.historyRaw.map((_: number, i: number) =>
    Math.round((metrics.routesLocal / Math.max(1, total)) * 100)
  )
})

const cacheSavedHistory = computed(() => {
  if (!metrics.historyRaw.length) return [0]
  const steps = metrics.historyRaw.length
  const current = metrics.cacheSavedTokens
  return Array.from({ length: steps }, (_, idx) => Math.round((current * (idx + 1)) / steps))
})

// TvChart: label each history point as #1, #2, …
const trendLabels = computed(() =>
  metrics.historyRaw.length
    ? metrics.historyRaw.map((_: number, i: number) => `#${i + 1}`)
    : ['—']
)
const trendSeries = computed(() => [
  { name: 'Raw', data: metrics.historyRaw.length ? [...metrics.historyRaw] : [0], color: '#ffa940' },
  { name: 'Compiled', data: metrics.historyCompiled.length ? [...metrics.historyCompiled] : [0], color: 'var(--primary)' },
  { name: 'Reused', data: metrics.historyReused.length ? [...metrics.historyReused] : [0], color: 'var(--secondary)' },
])

const modelRows = computed(() => {
  return Object.entries(metrics.modelStats)
    .map(([key, stat]) => {
      const sovereignRatio = Math.round(stat.sovereign_ratio ?? 0)
      return {
        key,
        model: stat.model,
        provider: stat.provider,
        requests: stat.requests,
        efficiency: Math.round(stat.efficiency_score ?? 0),
        sovereignLabel: sovereignRatio >= 100 ? 'Sovereign' : `${sovereignRatio}% sovereign`,
        sovereignClass: sovereignRatio >= 100 ? 'sovereign' : 'mixed',
      }
    })
    .sort((a, b) => b.requests - a.requests)
})
</script>

<style scoped>
.chart-section {
  margin-top: 20px;
}
.chart-section h3 {
  margin: 0 0 16px;
  font-size: 1rem;
}

.model-efficiency-section {
  margin-top: 20px;
}

.model-efficiency-section h3 {
  margin: 0 0 6px;
  font-size: 1rem;
}

.model-table-wrap {
  margin-top: 12px;
  overflow-x: auto;
}

.model-table {
  width: 100%;
  border-collapse: collapse;
  min-width: 620px;
}

.model-table th,
.model-table td {
  text-align: left;
  padding: 10px 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  font-size: 0.88rem;
}

.model-table th {
  color: var(--muted);
  font-weight: 600;
}

.sovereign-pill {
  display: inline-flex;
  align-items: center;
  padding: 3px 10px;
  border-radius: 999px;
  font-size: 0.75rem;
  font-weight: 700;
  letter-spacing: 0.2px;
}

.sovereign-pill.sovereign {
  background: rgba(44, 255, 179, 0.15);
  color: var(--accent);
}

.sovereign-pill.mixed {
  background: rgba(255, 169, 64, 0.15);
  color: #ffa940;
}
</style>
