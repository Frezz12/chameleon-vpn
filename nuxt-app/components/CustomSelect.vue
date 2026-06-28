<template>
  <div ref="root" class="custom-select" :class="{ open, disabled }">
    <button class="custom-select-trigger" type="button" :disabled="disabled" @click="toggle">
      <span class="custom-select-text">{{ selectedLabel }}</span>
      <span class="custom-select-icon" :class="{ open }">
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
          <path d="M3.5 5.5L7 9l3.5-3.5" />
        </svg>
      </span>
    </button>

    <Transition name="select-pop">
      <div v-if="open" class="custom-select-menu">
        <button
          v-for="option in options"
          :key="String(option.value)"
          type="button"
          class="custom-select-option"
          :class="{ selected: option.value === modelValue }"
          @click="select(option.value)"
        >
          <span>{{ option.label }}</span>
          <span v-if="option.value === modelValue" class="custom-select-check">
            <svg width="13" height="13" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
              <path d="M3 7.2l2.4 2.4L11 4" />
            </svg>
          </span>
        </button>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'

type SelectValue = string | number

type SelectOption = {
  label: string
  value: SelectValue
}

const props = withDefaults(defineProps<{
  modelValue: SelectValue
  options: SelectOption[]
  disabled?: boolean
}>(), {
  disabled: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: SelectValue]
  change: [value: SelectValue]
}>()

const root = ref<HTMLElement | null>(null)
const open = ref(false)

const selectedLabel = computed(() => props.options.find((option) => option.value === props.modelValue)?.label ?? '')

function toggle() {
  if (!props.disabled) open.value = !open.value
}

function select(value: SelectValue) {
  emit('update:modelValue', value)
  emit('change', value)
  open.value = false
}

function closeOnOutsideClick(event: MouseEvent) {
  if (root.value && !root.value.contains(event.target as Node)) {
    open.value = false
  }
}

onMounted(() => document.addEventListener('mousedown', closeOnOutsideClick))
onBeforeUnmount(() => document.removeEventListener('mousedown', closeOnOutsideClick))
</script>

<style scoped>
.custom-select {
  position: relative;
  width: 100%;
}
.custom-select-trigger {
  width: 100%;
  min-height: 42px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 9px 10px 9px 12px;
  border-radius: 12px;
  border: 1px solid rgba(150, 214, 255, 0.12);
  background:
    linear-gradient(180deg, rgba(255,255,255,0.055), rgba(255,255,255,0.025)),
    rgba(8, 16, 30, 0.72);
  color: var(--text-secondary);
  cursor: pointer;
  transition: border-color .18s ease, background .18s ease, box-shadow .18s ease;
}
.custom-select.open .custom-select-trigger,
.custom-select-trigger:hover {
  border-color: rgba(125, 247, 104, 0.24);
  background:
    linear-gradient(180deg, rgba(125,247,104,0.08), rgba(46,231,205,0.04)),
    rgba(8, 18, 32, 0.86);
  box-shadow: 0 0 0 3px rgba(125,247,104,0.055);
}
.custom-select-text {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 12px;
  font-weight: 700;
  text-align: left;
}
.custom-select-icon {
  width: 24px;
  height: 24px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  border-radius: 8px;
  color: #7df768;
  background: rgba(125,247,104,0.08);
  transition: transform .18s ease, color .18s ease;
}
.custom-select-icon.open {
  transform: rotate(180deg);
  color: #2ee7cd;
}
.custom-select-menu {
  position: absolute;
  left: 0;
  right: 0;
  top: calc(100% + 6px);
  z-index: 40;
  display: grid;
  gap: 4px;
  padding: 6px;
  border-radius: 14px;
  border: 1px solid rgba(126,182,255,0.14);
  background: linear-gradient(180deg, rgba(10,18,35,0.98), rgba(7,13,26,0.98));
  box-shadow: 0 18px 44px rgba(0,0,0,0.34);
  backdrop-filter: blur(16px);
}
.custom-select-option {
  min-height: 36px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 8px 9px;
  border: 0;
  border-radius: 10px;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 12px;
  font-weight: 650;
  text-align: left;
  transition: background .16s ease, color .16s ease;
}
.custom-select-option:hover {
  background: rgba(46,231,205,0.08);
  color: var(--text-secondary);
}
.custom-select-option.selected {
  background: linear-gradient(90deg, rgba(125,247,104,0.14), rgba(46,231,205,0.08), rgba(122,90,255,0.10));
  color: var(--text);
}
.custom-select-check {
  display: inline-flex;
  color: #7df768;
}
.select-pop-enter-active,
.select-pop-leave-active {
  transition: opacity .14s ease, transform .14s ease;
}
.select-pop-enter-from,
.select-pop-leave-to {
  opacity: 0;
  transform: translateY(-4px) scale(.98);
}
html.light .custom-select-trigger {
  background: linear-gradient(180deg, rgba(255,255,255,0.95), rgba(244,250,248,0.88));
  border-color: rgba(42,109,120,0.14);
  color: rgba(16,33,58,0.78);
}
html.light .custom-select.open .custom-select-trigger,
html.light .custom-select-trigger:hover {
  background: #fff;
  border-color: rgba(58,191,103,0.30);
  box-shadow: 0 0 0 3px rgba(58,191,103,0.10);
}
html.light .custom-select-icon {
  color: #258151;
  background: rgba(58,191,103,0.11);
}
html.light .custom-select-menu {
  background: linear-gradient(180deg, rgba(255,255,255,0.98), rgba(244,250,248,0.98));
  border-color: rgba(42,109,120,0.14);
  box-shadow: 0 18px 44px rgba(17,39,67,0.16);
}
html.light .custom-select-option { color: rgba(16,48,60,0.58); }
html.light .custom-select-option:hover {
  background: rgba(46,158,177,0.08);
  color: rgba(16,33,58,0.82);
}
html.light .custom-select-option.selected {
  background: linear-gradient(90deg, rgba(58,191,103,0.14), rgba(46,158,177,0.09), rgba(122,90,255,0.08));
  color: rgba(16,33,58,0.86);
}
html.light .custom-select-check { color: #258151; }</style>

