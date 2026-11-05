export type UploadedFile =
  | {
      source: "picker" | "browser-drop"
      name: string
      size: number
      file: File
      path?: never
    }
  | {
      source: "tauri-drop"
      name: string
      path: string
      file?: never
      size?: never
    }
