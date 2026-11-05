const OUTPUT_DIRECTORY_STORAGE_KEY = "uniconvert-output-directory"

export function getDefaultOutputDirectory() {
  return localStorage.getItem(OUTPUT_DIRECTORY_STORAGE_KEY) ?? ""
}

export function saveDefaultOutputDirectory(path: string) {
  if (path) {
    localStorage.setItem(OUTPUT_DIRECTORY_STORAGE_KEY, path)
  } else {
    localStorage.removeItem(OUTPUT_DIRECTORY_STORAGE_KEY)
  }
}
