import { useTauri } from '~/composables/useTauri'
import { useToast } from 'vue-toastification'

export default defineNuxtPlugin(() => {
  const { isTauri } = useTauri()

  if (isTauri) {
    console.log('Tauri environment detected')

    // Provide toast globally
    const toast = useToast()

    return {
      provide: {
        toast,
      },
    }
  }
})
