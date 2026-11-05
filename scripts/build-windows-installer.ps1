$ErrorActionPreference = 'Stop'
Push-Location (Join-Path $PSScriptRoot '..')
try {
    foreach ($binary in @('release/win/ffmpeg.exe', 'release/win/ffprobe.exe')) {
        if (-not (Test-Path -LiteralPath $binary -PathType Leaf)) {
            throw "Required binary missing: $binary"
        }
    }
    Push-Location tauri-app
    try {
        & ..\node_modules\.bin\tauri.cmd build --bundles nsis --ci
        if ($LASTEXITCODE -ne 0) { throw 'Tauri installer build failed' }
    } finally { Pop-Location }
    $config = Get-Content -LiteralPath tauri-app/tauri.conf.json -Raw | ConvertFrom-Json
    $installer = "tauri-app/target/release/bundle/nsis/uniconvert_$($config.version)_x64-setup.exe"
    if (-not (Test-Path -LiteralPath $installer)) { throw "Installer missing: $installer" }
    $destination = "release/UniConvert-$($config.version)-Windows-x64-Setup.exe"
    Copy-Item -LiteralPath $installer -Destination $destination -Force
    Copy-Item -LiteralPath tauri-app/target/release/uniconvert.exe -Destination release/win/uniconvert.exe -Force
    Get-Item -LiteralPath $destination | Select-Object FullName, Length, LastWriteTime
} finally { Pop-Location }
