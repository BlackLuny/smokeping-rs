import { defineStore } from 'pinia'
import axios from 'axios'

export const useTargetsStore = defineStore('targets', {
  state: () => ({
    targets: [],
    selectedTarget: null,
    probeData: []
  }),
  actions: {
    async fetchTargets() {
      const response = await axios.get('/api/targets')
      this.targets = response.data
    },
    async fetchTargetDetails(id) {
      const response = await axios.get(`/api/targets/${id}`)
      this.selectedTarget = response.data
    },
    async fetchProbeData(id, startTime, endTime) {
      const response = await axios.get(`/api/targets/${id}/data?start_time=${startTime}&end_time=${endTime}`)
      this.probeData = response.data
    }
  }
})
