import { useToast as useToastification } from 'vue-toastification'

export function useToast() {
  const toast = useToastification()

  return {
    success: (msg: string) => toast.success(msg),
    error: (msg: string) => toast.error(msg),
    info: (msg: string) => toast.info(msg),
    warning: (msg: string) => toast.warning(msg),
  }
}
