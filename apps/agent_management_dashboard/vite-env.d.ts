/// <reference types="vite/client" />

declare global {
  interface ImportMetaEnv {
    readonly VITE_API_URL?: string
    readonly VITE_PUBLIC_API_URL?: string
    readonly DEV: boolean
    readonly PROD: boolean
    readonly MODE: string
  }

  interface ImportMeta {
    readonly env: ImportMetaEnv
  }
}
