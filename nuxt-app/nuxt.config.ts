export default defineNuxtConfig({
  ssr: process.env.NODE_ENV === 'development',
  devtools: { enabled: true },

  modules: [
    '@pinia/nuxt',
    '@nuxtjs/color-mode',
    '@nuxtjs/tailwindcss',
  ],

  colorMode: {
    classSuffix: '',
    preference: 'dark',
    fallback: 'dark',
  },

  tailwindcss: {
    cssPath: '~/assets/css/main.css',
    config: {
      darkMode: 'class',
      theme: {
        extend: {
          colors: {
            'bg-primary': '#0f1117',
            'bg-secondary': '#1a1d27',
            'bg-tertiary': '#242736',
            'accent-green': '#22bb66',
            'accent-yellow': '#f0b429',
            'accent-red': '#e74c3c',
            'accent-blue': '#3b82f6',
            border: '#2e3142',
            primary: '#e4e6f0',
            secondary: '#9ca0b0',
            tertiary: '#242736',
          },
        },
      },
    },
  },

  app: {
    baseURL: process.env.NODE_ENV === 'development' ? '/' : './',
    buildAssetsDir: '/_nuxt/',
    head: {
      title: 'Chameleon',
      meta: [
        { charset: 'utf-8' },
        { name: 'viewport', content: 'width=device-width, initial-scale=1, minimum-scale=1' },
      ],
    },
  },

  devServer: {
    port: 1420,
    host: '127.0.0.1',
  },

  vite: {
    clearScreen: false,
    server: {
      strictPort: true,
      port: 1420,
      host: '127.0.0.1',
    },
    envPrefix: ['VITE_', 'TAURI_'],
    build: {
      target: ['es2021', 'chrome100', 'safari13'],
      minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
      sourcemap: !!process.env.TAURI_DEBUG,
    },
  },

  nitro: {
    preset: 'static',
  },

  compatibilityDate: '2024-08-14',
})
