<template>
  <el-dialog v-model="dialogVisible" :title="isEdit ? 'Edit Target' : 'Add Target'">
    <el-form :model="form">
      <el-form-item label="Name">
        <el-input v-model="form.name"></el-input>
      </el-form-item>
      <el-form-item label="Host">
        <el-input v-model="form.host"></el-input>
      </el-form-item>
      <el-form-item label="Probe Type">
        <el-select v-model="form.probe_type">
          <el-option label="ICMP" value="icmp"></el-option>
        </el-select>
      </el-form-item>
      <el-form-item label="Interval (s)">
        <el-input-number v-model="form.probe_interval_secs"></el-input-number>
      </el-form-item>
      <el-form-item label="Active">
        <el-switch v-model="form.is_active"></el-switch>
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="dialogVisible = false">Cancel</el-button>
      <el-button type="primary" @click="handleSubmit">Confirm</el-button>
    </template>
  </el-dialog>
</template>

<script setup>
import { ref, watch, defineProps, defineEmits } from 'vue'
import { useTargetsStore } from '../stores/targets'
import axios from 'axios'

const props = defineProps({
  visible: Boolean,
  target: Object
})

const emit = defineEmits(['update:visible', 'submit'])

const store = useTargetsStore()
const dialogVisible = ref(props.visible)
const isEdit = ref(false)
const form = ref({
  name: '',
  host: '',
  probe_type: 'icmp',
  probe_interval_secs: 60,
  is_active: true
})

watch(() => props.visible, (val) => {
  dialogVisible.value = val
  if (val) {
    if (props.target) {
      isEdit.value = true
      form.value = { ...props.target }
    } else {
      isEdit.value = false
      form.value = {
        name: '',
        host: '',
        probe_type: 'icmp',
        probe_interval_secs: 60,
        is_active: true
      }
    }
  }
})

watch(dialogVisible, (val) => {
  if (!val) {
    emit('update:visible', false)
  }
})

async function handleSubmit() {
  console.log('Form data being sent:', form.value)
  try {
    if (isEdit.value) {
      await axios.put(`/api/targets/${form.value.id}`, form.value)
    } else {
      await axios.post('/api/targets', form.value)
    }
    await store.fetchTargets()
    emit('submit')
    dialogVisible.value = false
  } catch (error) {
    console.error('Error submitting form:', error)
    console.error('Error response:', error.response?.data)
  }
}
</script>
