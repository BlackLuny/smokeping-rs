<template>
  <div>
    <h1>{{ target ? target.name : 'Loading...' }}</h1>
    <div ref="chart" style="width: 100%; height: 400px;"></div>
    <div ref="lossChart" style="width: 100%; height: 200px;"></div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import { useTargetsStore } from '../stores/targets'
import { storeToRefs } from 'pinia'
import * as echarts from 'echarts'

const route = useRoute()
const store = useTargetsStore()
const { selectedTarget: target, probeData } = storeToRefs(store)

const chart = ref(null)
const lossChart = ref(null)
let chartInstance = null
let lossChartInstance = null
let ws = null

const timeRange = ref('1h') // Default to 1 hour

onMounted(async () => {
  await store.fetchTargetDetails(route.params.id)
  await fetchData()

  chartInstance = echarts.init(chart.value)
  lossChartInstance = echarts.init(lossChart.value)

  renderChart()
  renderLossChart()

  ws = new WebSocket(`ws://${window.location.host}/ws`)
  ws.onmessage = (event) => {
    const data = JSON.parse(event.data)
    if (target.value && data.target_id === target.value.id) {
      const now = new Date()
      probeData.value.push({ time: now, rtt_ms: data.rtt_ms, is_lost: data.is_lost })
      renderChart()
      renderLossChart()
    }
  }
})

onUnmounted(() => {
  if (ws) {
    ws.close()
  }
})

watch(timeRange, fetchData)

async function fetchData() {
  if (!target.value) return

  const now = new Date()
  let start = new Date()

  switch (timeRange.value) {
    case '1h':
      start.setHours(now.getHours() - 1)
      break
    case '24h':
      start.setDate(now.getDate() - 1)
      break
    case '7d':
      start.setDate(now.getDate() - 7)
      break
  }

  await store.fetchProbeData(target.value.id, start.toISOString(), now.toISOString())
  renderChart()
  renderLossChart()
}

function renderChart() {
  if (!chartInstance) return
  const option = {
    xAxis: {
      type: 'time'
    },
    yAxis: {
      type: 'value',
      name: 'RTT (ms)'
    },
    series: [
      {
        data: probeData.value.map(p => [p.time, p.rtt_ms]),
        type: 'scatter',
        symbolSize: 5
      }
    ]
  }
  chartInstance.setOption(option)
}

function renderLossChart() {
  if (!lossChartInstance) return
  const option = {
    xAxis: {
      type: 'time'
    },
    yAxis: {
      type: 'value',
      name: 'Packet Loss',
      max: 1
    },
    series: [
      {
        data: probeData.value.map(p => [p.time, p.is_lost ? 1 : 0]),
        type: 'line',
        step: 'end',
        color: 'red'
      }
    ]
  }
  lossChartInstance.setOption(option)
}
</script>
