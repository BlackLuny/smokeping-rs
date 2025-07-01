<template>
  <div>
    <el-button type="primary" @click="showAddDialog = true">Add Target</el-button>
    <el-table :data="targets">
      <el-table-column prop="name" label="Name"></el-table-column>
      <el-table-column prop="host" label="Host"></el-table-column>
      <el-table-column prop="probe_type" label="Type"></el-table-column>
      <el-table-column prop="probe_interval_secs" label="Interval (s)"></el-table-column>
      <el-table-column prop="is_active" label="Active">
        <template #default="{ row }">
          <el-tag :type="row.is_active ? 'success' : 'danger'">{{ row.is_active ? 'Yes' : 'No' }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column label="Actions">
        <template #default="{ row }">
          <el-button size="small" @click="showEditDialog(row)">Edit</el-button>
          <el-button size="small" type="danger" @click="deleteTarget(row.id)">Delete</el-button>
          <el-button size="small" @click="goToTargetDetails(row)">Details</el-button>
        </template>
      </el-table-column>
    </el-table>

    <TargetForm v-model:visible="showAddDialog" @submit="handleFormSubmit" />
    <TargetForm v-model:visible="showEditDialogFlag" :target="selectedTarget" @submit="handleFormSubmit" />
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { useTargetsStore } from '../stores/targets'
import { storeToRefs } from 'pinia'
import { useRouter } from 'vue-router'
import TargetForm from '../components/TargetForm.vue'
import axios from 'axios'

const router = useRouter()
const store = useTargetsStore()
const { targets } = storeToRefs(store)

const showAddDialog = ref(false)
const showEditDialogFlag = ref(false)
const selectedTarget = ref(null)

onMounted(() => {
  store.fetchTargets()
})

function showEditDialog(target) {
  selectedTarget.value = target
  showEditDialogFlag.value = true
}

async function deleteTarget(id) {
  await axios.delete(`/api/targets/${id}`)
  await store.fetchTargets()
}

function handleFormSubmit() {
  store.fetchTargets()
}

const goToTargetDetails = (row) => {
  router.push({ name: 'TargetDetails', params: { id: row.id } })
}
</script>