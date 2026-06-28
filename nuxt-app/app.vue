<template>
  <NuxtLayout>
    <NuxtPage />
  </NuxtLayout>
</template>

<script setup lang="ts">
import { useTheme } from '~/composables/useTheme'

useTheme().initTheme()
</script>

<style>
body {
  margin: 0;
  padding: 0;
}
</style>
