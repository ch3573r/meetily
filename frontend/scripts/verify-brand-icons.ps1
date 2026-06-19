param(
    [string]$FrontendRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")).Path
)

$ErrorActionPreference = "Stop"

$FrontendRoot = (Resolve-Path -LiteralPath $FrontendRoot).Path

function Resolve-IconPath {
    param([Parameter(Mandatory=$true)][string]$RelativePath)
    Join-Path $FrontendRoot $RelativePath
}

function Assert-FileExists {
    param([Parameter(Mandatory=$true)][string]$RelativePath)

    $path = Resolve-IconPath $RelativePath
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "Missing icon asset: $RelativePath"
    }
    return $path
}

function Read-BigEndianUInt32 {
    param(
        [Parameter(Mandatory=$true)][byte[]]$Bytes,
        [Parameter(Mandatory=$true)][int]$Offset
    )

    return [uint32](
        ([uint32]$Bytes[$Offset] -shl 24) -bor
        ([uint32]$Bytes[$Offset + 1] -shl 16) -bor
        ([uint32]$Bytes[$Offset + 2] -shl 8) -bor
        [uint32]$Bytes[$Offset + 3]
    )
}

function Get-PngSize {
    param([Parameter(Mandatory=$true)][string]$Path)

    $bytes = [System.IO.File]::ReadAllBytes($Path)
    $signature = @(137, 80, 78, 71, 13, 10, 26, 10)
    if ($bytes.Length -lt 24) {
        throw "Invalid PNG '$Path': file is too small."
    }
    for ($i = 0; $i -lt $signature.Count; $i++) {
        if ($bytes[$i] -ne $signature[$i]) {
            throw "Invalid PNG '$Path': signature mismatch."
        }
    }

    [pscustomobject]@{
        Width = Read-BigEndianUInt32 -Bytes $bytes -Offset 16
        Height = Read-BigEndianUInt32 -Bytes $bytes -Offset 20
    }
}

function Assert-PngSize {
    param(
        [Parameter(Mandatory=$true)][string]$RelativePath,
        [Parameter(Mandatory=$true)][int]$Width,
        [Parameter(Mandatory=$true)][int]$Height
    )

    $path = Assert-FileExists $RelativePath
    $size = Get-PngSize $path
    if ($size.Width -ne $Width -or $size.Height -ne $Height) {
        throw "$RelativePath must be ${Width}x${Height}, got $($size.Width)x$($size.Height)."
    }
}

function Get-IcoSizes {
    param([Parameter(Mandatory=$true)][string]$Path)

    $bytes = [System.IO.File]::ReadAllBytes($Path)
    if ($bytes.Length -lt 6) {
        throw "Invalid ICO '$Path': file is too small."
    }

    $reserved = [BitConverter]::ToUInt16($bytes, 0)
    $type = [BitConverter]::ToUInt16($bytes, 2)
    $count = [BitConverter]::ToUInt16($bytes, 4)
    if ($reserved -ne 0 -or $type -ne 1 -or $count -lt 1) {
        throw "Invalid ICO '$Path': bad header."
    }

    $sizes = @()
    for ($i = 0; $i -lt $count; $i++) {
        $entryOffset = 6 + ($i * 16)
        if (($entryOffset + 16) -gt $bytes.Length) {
            throw "Invalid ICO '$Path': truncated directory."
        }

        $width = if ($bytes[$entryOffset] -eq 0) { 256 } else { [int]$bytes[$entryOffset] }
        $height = if ($bytes[$entryOffset + 1] -eq 0) { 256 } else { [int]$bytes[$entryOffset + 1] }
        $sizes += "${width}x${height}"
    }

    return $sizes | Sort-Object -Unique
}

function Assert-IcoSizes {
    param(
        [Parameter(Mandatory=$true)][string]$RelativePath,
        [Parameter(Mandatory=$true)][int[]]$Sizes
    )

    $path = Assert-FileExists $RelativePath
    $actual = @(Get-IcoSizes $path)
    $expected = @($Sizes | ForEach-Object { "${_}x${_}" } | Sort-Object -Unique)
    if (($actual -join ",") -ne ($expected -join ",")) {
        throw "$RelativePath must contain ICO sizes '$($expected -join ",")', got '$($actual -join ",")'."
    }
}

function Assert-Icns {
    param([Parameter(Mandatory=$true)][string]$RelativePath)

    $path = Assert-FileExists $RelativePath
    $bytes = [System.IO.File]::ReadAllBytes($path)
    if ($bytes.Length -lt 8) {
        throw "Invalid ICNS '$RelativePath': file is too small."
    }
    $magic = [System.Text.Encoding]::ASCII.GetString($bytes, 0, 4)
    if ($magic -ne "icns") {
        throw "Invalid ICNS '$RelativePath': signature mismatch."
    }
}

function Assert-TauriIconConfig {
    $configPath = Assert-FileExists "src-tauri/tauri.conf.json"
    $config = Get-Content -LiteralPath $configPath -Raw | ConvertFrom-Json

    foreach ($target in @("msi", "nsis")) {
        if (-not (@($config.bundle.targets) -contains $target)) {
            throw "Tauri bundle target '$target' is missing."
        }
    }

    foreach ($icon in @("icons/icon.png", "icons/app_icon.icns", "icons/app_icon.ico")) {
        if (-not (@($config.bundle.icon) -contains $icon)) {
            throw "Tauri bundle.icon is missing '$icon'."
        }
    }

    if ($config.bundle.windows.nsis.installerIcon -ne "icons/app_icon.ico") {
        throw "NSIS installerIcon must be icons/app_icon.ico."
    }
    if ($config.bundle.windows.nsis.uninstallerIcon -ne "icons/app_icon.ico") {
        throw "NSIS uninstallerIcon must be icons/app_icon.ico."
    }
}

$pngSizes = [ordered]@{
    "src-tauri/icons/icon.png" = @(512, 512)
    "src-tauri/icons/128x128.png" = @(128, 128)
    "src-tauri/icons/128x128@2x.png" = @(256, 256)
    "src-tauri/icons/32x32.png" = @(32, 32)
    "src-tauri/icons/icon_16x16.png" = @(16, 16)
    "src-tauri/icons/icon_16x16@2x.png" = @(32, 32)
    "src-tauri/icons/icon_24x24.png" = @(24, 24)
    "src-tauri/icons/icon_32x32.png" = @(32, 32)
    "src-tauri/icons/icon_32x32@2x.png" = @(64, 64)
    "src-tauri/icons/icon_48x48.png" = @(48, 48)
    "src-tauri/icons/icon_64x64.png" = @(64, 64)
    "src-tauri/icons/icon_128x128.png" = @(128, 128)
    "src-tauri/icons/icon_128x128@2x.png" = @(256, 256)
    "src-tauri/icons/icon_256x256.png" = @(256, 256)
    "src-tauri/icons/icon_256x256@2x.png" = @(512, 512)
    "src-tauri/icons/icon_512x512.png" = @(512, 512)
    "src-tauri/icons/icon_512x512@2x.png" = @(1024, 1024)
    "src-tauri/icons/Square30x30Logo.png" = @(30, 30)
    "src-tauri/icons/Square44x44Logo.png" = @(44, 44)
    "src-tauri/icons/Square71x71Logo.png" = @(71, 71)
    "src-tauri/icons/Square89x89Logo.png" = @(89, 89)
    "src-tauri/icons/Square107x107Logo.png" = @(107, 107)
    "src-tauri/icons/Square142x142Logo.png" = @(142, 142)
    "src-tauri/icons/Square150x150Logo.png" = @(150, 150)
    "src-tauri/icons/Square284x284Logo.png" = @(284, 284)
    "src-tauri/icons/Square310x310Logo.png" = @(310, 310)
    "src-tauri/icons/StoreLogo.png" = @(50, 50)
    "public/icon_128x128.png" = @(128, 128)
    "public/icon_32x32@2x.png" = @(64, 64)
    "public/logo-collapsed.png" = @(512, 512)
    "public/logo.png" = @(845, 295)
}

foreach ($size in @(16, 24, 32, 48, 64, 128, 256, 512)) {
    $pngSizes["public/brand/clawscribe-icon-$size.png"] = @($size, $size)
}

foreach ($entry in $pngSizes.GetEnumerator()) {
    Assert-PngSize -RelativePath $entry.Key -Width $entry.Value[0] -Height $entry.Value[1]
}

$icoSizes = @(16, 24, 32, 48, 64, 128, 256)
Assert-IcoSizes -RelativePath "src-tauri/icons/app_icon.ico" -Sizes $icoSizes
Assert-IcoSizes -RelativePath "src-tauri/icons/icon.ico" -Sizes $icoSizes
Assert-IcoSizes -RelativePath "src/app/favicon.ico" -Sizes $icoSizes

Assert-Icns "src-tauri/icons/app_icon.icns"
Assert-Icns "src-tauri/icons/icon.icns"

Assert-FileExists "public/brand/clawscribe-icon.svg" | Out-Null
Assert-FileExists "src-tauri/icons/clawscribe-icon.svg" | Out-Null
Assert-TauriIconConfig

Write-Host "ClawScribe icon assets verified for app, tray, installer, executable, favicon, and public brand surfaces."
