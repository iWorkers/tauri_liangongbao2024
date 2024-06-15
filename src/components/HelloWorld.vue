<template>
  <div class="hello">
    <div v-html="msg"></div>
    
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api'

// 定义响应式数据
const msg = ref('')
const scanResult = ref(null)
// 定义方法
async function getQRCodeUrl() {
  try {
    const qrCodeUrl = await invoke('qrcode_url')
    //console.log('QR Code URL:', qrCodeUrl)
    msg.value= qrCodeUrl
    // 在这里处理获取到的二维码 URL
    await invoke('my_custom_command')

    window.__TAURI__.event.listen('scan-result', (event) => {
      if (event.payload === 'scan_timeout') {
        scanResult.value = 'timeout'
      } else {
        scanResult.value = 'success'
        console.log('微信扫码成功，前端获取到 wx_code:', event.payload)
      }
  })


  } catch (error) {
    console.error('Error getting QR code URL:', error)
  }
}

// 在组件挂载后调用 getQRCodeUrl 方法
onMounted(() => {
  getQRCodeUrl()
 
  
})

// 在组件创建后调用 invoke 方法

</script>


<style>

</style>
