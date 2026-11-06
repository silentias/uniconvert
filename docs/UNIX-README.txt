UniConvert — Linux / macOS

Готовые бинарники появляются после нативной сборки; этот файл не является сборкой.

Сборка из корня проекта на целевой системе:
  pnpm install --frozen-lockfile
  bash scripts/build-unix-release.sh

Нужны Node.js 24, pnpm 12, стабильный Rust и системные зависимости Tauri.
Linux Ubuntu 24.04:
  sudo apt-get install build-essential pkg-config libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf ffmpeg
macOS: установите Xcode Command Line Tools и FFmpeg (brew install ffmpeg).

Результат: release/linux-x64, release/linux-arm64, release/macos-x64
или release/macos-arm64 в зависимости от системы сборки.
Внутри: uniconvert (GUI), uniconvert-cli; на macOS также UniConvert.app.
Запуск Linux: ./uniconvert
Запуск macOS: откройте UniConvert.app.

FFmpeg/FFprobe в Unix-архивы пока не включаются. Они должны быть доступны
в PATH либо заданы абсолютными путями UNICONVERT_FFMPEG_PATH и
UNICONVERT_FFPROBE_PATH. На macOS при запуске из Finder PATH может отличаться
от терминала; для диагностики запускайте ./uniconvert из терминала.
Linux-сборка требует совместимых системных библиотек WebKitGTK.
macOS-сборка не подписана и не нотарифицирована.
