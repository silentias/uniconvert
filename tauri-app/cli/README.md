# UniConvert CLI

Отдельный Rust-бинарник `uniconvert`, использующий `uniconvert-core` без запуска Tauri. CLI конвертирует аудио, изображения и видео, показывает сведения об аудио, извлекает метаданные и проверяет структуру Gerber X2/X3.

## Запуск и сборка

Из корня репозитория:

```powershell
pnpm cli --help
pnpm cli:build
```

Готовый файл: `tauri-app/target/cli/release/uniconvert.exe` на Windows или `uniconvert` на Linux/macOS. Node.js и pnpm для запуска собранного файла не нужны.

Для аудио, видео и `probe` требуются FFmpeg и FFprobe. Изображения, `image-meta` и `gerber-meta` обрабатываются самим приложением. Поиск FFmpeg/FFprobe выполняется через переменные `UNICONVERT_FFMPEG_PATH` и `UNICONVERT_FFPROBE_PATH`, каталог `tauri-app/binaries`, каталог рядом с CLI, затем системный `PATH`.

## Команды

```text
uniconvert audio INPUT --to FORMAT [OPTIONS]
uniconvert image INPUT --to FORMAT [OPTIONS]
uniconvert video INPUT --to FORMAT [OPTIONS]
uniconvert probe INPUT [--json]
uniconvert gerber-meta INPUT [--json]
uniconvert image-meta INPUT [--json]
```

Старое имя `convert` сохранено как псевдоним команды `audio`.

### Аудио

Форматы: `mp3`, `wav`, `flac`, `aac`, `ogg`, `opus`, `m4a`, `wma`, `aiff` (`aif`).

| Параметр | Значение |
| --- | --- |
| `--bitrate KBPS` | 8–1536 кбит/с; неприменим к WAV, FLAC и AIFF |
| `--sample-rate HZ` | 8000–384000 Гц |
| `--channels N` | 1 — моно, 2 — стерео |
| `--volume DB` | От -60 до 30 дБ |
| `--tempo MULTIPLIER` | Темп от 0.5 до 2.0 с сохранением высоты тона |
| `--pitch SEMITONES` | Высота тона от -12 до 12 полутонов с сохранением длительности |
| `--normalize` | Включить LUFS-нормализацию |
| `--target-lufs VALUE` | От -70 до -5; требует `--normalize` |
| `--start`, `--end` | Границы фрагмента в секундах, `мм:сс.ммм` или `чч:мм:сс.ммм` |
| `--strip-metadata` | Удалить метаданные |

```powershell
uniconvert audio "recording.wav" --to mp3 --bitrate 192
uniconvert audio "recording.wav" --to opus --start 10 --end 45 --normalize
```

### Изображения

Форматы: `jpg`, `png`, `webp`, `avif`, `gif`, `bmp`, `tiff`, `ico`.

```powershell
uniconvert image "photo.png" --to webp --quality 85
uniconvert image "scan.tiff" --to jpg --strip-metadata --output-dir "ready"
```

`--quality` принимает значение от 0 до 100 и по умолчанию равен 80. Для lossless-форматов параметр может не влиять на размер.

### Видео

Контейнеры: `mp4`, `mkv`, `webm`, `mov`, `avi`. Видеокодеки: `h264`, `h265`, `vp9`, `av1`. Аудиокодеки: `aac`, `mp3`, `opus`, `flac`.

```powershell
uniconvert video "source.mkv" --to mp4 --video-codec h264 --crf 23
uniconvert video "source.mp4" --to mkv --copy-streams
uniconvert video "source.mov" --to webm --video-codec vp9 --height 1080 --fps 30 --audio-codec opus --audio-bitrate 128
```

`--height`: 480, 720, 1080, 1440 или 2160. `--fps`: 24, 30, 50 или 60. `--crf`: 0–51. При `--copy-streams` параметры перекодирования игнорируются; совместимость потоков с выбранным контейнером проверяет FFmpeg.

### Анализ изображений, аудио и Gerber

```powershell
uniconvert probe "recording.flac" --json
uniconvert image-meta "photo.jpg" --json
uniconvert gerber-meta "board-F_Cu.gtl" --json
```

`probe` возвращает формат, кодек, длительность, размер, битрейт, частоту и каналы первой аудиодорожки. `image-meta` показывает технические свойства изображения и полезные EXIF-поля: камеру, объектив, дату, параметры съёмки, программу, автора и GPS. `gerber-meta` читает `.GenerationSoftware`, `.CreationDate`, `.FileFunction`, единицы `MO` и формат координат `FS`. Команда считает графические операции, выводит список `ADD`-апертур с параметрами и числом использований, сообщает используемые без определения, неиспользуемые и повторно определённые D-коды, а также проверяет структуру файла. В структурный отчёт входят обязательные `FS`, `MO`, `M02`, корректность `ADD`, баланс `G36/G37`, выбор апертуры и неизвестные или устаревшие команды. Отсутствующий EXIF возвращается пустым массивом.

## Общие параметры и результат

`-o, --output-dir DIR` задаёт каталог результата для конвертации. Если имя занято, используется суффикс `_converted`, `_converted_2` и далее. Исходный и существующие файлы не перезаписываются. `--strip-metadata` доступен у всех трёх команд конвертации.

Глобальные параметры: `--json`, `--quiet`, `--help`, `--version`. `--json` выводит один объект в stdout и отключает прогресс. Коды завершения: `0` — успех, `1` — ошибка процесса/файловой системы, `2` — неверные аргументы/вход/формат, `130` — Ctrl+C.

```json
{"ok":true,"result":{"path":"C:\\media\\result.mp4","input_size":1000000,"output_size":500000,"elapsed_millis":900}}
```

Ctrl+C прерывает аудио- или видеоконвертацию. Незавершённый файл удаляется. Очистка после принудительного убийства процесса операционной системой не гарантируется.

## Проверки

```powershell
cargo test --manifest-path tauri-app/Cargo.toml -p uniconvert-cli
cargo test --manifest-path tauri-app/Cargo.toml -p uniconvert-cli -- --include-ignored
```

Вторая команда требует настроенных FFmpeg и FFprobe. Тесты проверяют аудио, изображения, видео, Gerber Meta, JSON, Unicode-пути, обрезку, защиту существующих файлов и очистку после ошибок/отмены.
